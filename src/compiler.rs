use crate::chunk::{Chunk, OpCode};
use crate::scanner::{Scanner, Token, TokenKind, NULL_TOKEN};
use crate::value::Value;

struct Parser<'a> {
    current: Token<'a>,
    previous: Token<'a>,
    had_error: bool,
    panic_mode: bool,
}

// Defines the order in which parsed tokens are evaluated.
#[derive(Copy, Clone, PartialOrd, PartialEq)]
enum Precedence {
    None = 0,
    Assignment = 1,
    // =
    Or = 2,
    // or
    And = 3,
    // and
    Equality = 4,
    // == !=
    Comparison = 5,
    // < > <= >=
    Term = 6,
    // + -
    Factor = 7,
    // * /
    Unary = 8,
    // ! -
    Call = 9,
    // . ()
    Primary = 10,
}

impl Precedence {
    fn next(&self) -> Self {
        use Precedence::*;
        match *self {
            None => Assignment,
            Assignment => Or,
            Or => And,
            And => Equality,
            Equality => Comparison,
            Comparison => Term,
            Term => Factor,
            Factor => Unary,
            Unary => Call,
            Call => Primary,
            Primary => unreachable!(),
        }
    }
}

type ParseFn<'a> = fn(&mut Compiler<'a>);

struct ParseRule<'a> {
    prefix: Option<ParseFn<'a>>,
    infix: Option<ParseFn<'a>>,
    precedence: Precedence,
}

impl<'a> ParseRule<'a> {
    fn new(
        prefix: Option<ParseFn<'a>>,
        infix: Option<ParseFn<'a>>,
        precedence: Precedence,
    ) -> Self {
        ParseRule {
            prefix,
            infix,
            precedence,
        }
    }
}

pub(crate) struct Compiler<'a> {
    source: &'a [u8],
    scanner: Scanner<'a>,
    parser: Parser<'a>,
    compiling_chunk: &'a mut Chunk,
}

impl<'a> Compiler<'a> {
    pub fn new(source: &'a [u8], chunk: &'a mut Chunk) -> Compiler<'a> {
        Compiler {
            source,
            scanner: Scanner::new(source),
            parser: Parser {
                current: NULL_TOKEN,
                previous: NULL_TOKEN,
                had_error: false,
                panic_mode: false,
            },
            compiling_chunk: chunk,
        }
    }

    pub fn compile(&mut self) -> bool {
        // self.compiling_chunk = chunk;

        self.parser.had_error = false;
        self.parser.panic_mode = false;

        self.advance();
        self.expression();
        self.consume(TokenKind::Eof, "Expect end of expression.");
        self.end_compiler();
        !self.parser.had_error
    }

    fn advance(&mut self) {
        self.parser.previous = self.parser.current;

        loop {
            self.parser.current = self.scanner.scan_token();
            if self.parser.current.kind != TokenKind::Error {
                break;
            }

            self.error_at_current(&self.parser.current.lexeme_to_string());
        }
    }

    fn consume(&mut self, kind: TokenKind, message: &str) {
        if self.parser.current.kind == kind {
            self.advance();
            return;
        }

        self.error_at_current(message);
    }

    fn error_at_current(&mut self, message: &str) {
        let token = self.parser.current;
        self.error_at(&token, message);
    }

    fn error(&mut self, message: &str) {
        let token = self.parser.previous;
        self.error_at(&token, message);
    }

    fn error_at(&mut self, token: &Token, message: &str) {
        if self.parser.panic_mode {
            return;
        }
        self.parser.panic_mode = true;
        eprint!("[line {}] Error", token.line);

        match token.kind {
            TokenKind::Eof => eprint!(" at end"),
            TokenKind::Error => {}
            _ => eprint!(" at {}", token.lexeme_to_string()),
        }

        eprintln!(": {}", message);
    }

    fn emit_byte(&mut self, byte: u8) {
        self.compiling_chunk
            .write_byte(byte, self.parser.previous.line);
    }

    fn emit_bytes(&mut self, first: u8, second: u8) {
        self.emit_byte(first);
        self.emit_byte(second);
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return.to_u8())
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.emit_bytes(OpCode::Constant.to_u8(), constant)
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let current_chunk = &mut self.compiling_chunk;
        let constant = current_chunk.add_constant(value);
        if constant > u8::MAX as usize {
            // Note: in a real VM, we'd need another bytecode instruction like OP_CONSTANT_16
            // that stores the constant index as two bytes, so that the VM could handle more
            // than 256 constants when needed.
            //
            // The original clox doesn't support this, as apparently the code for doing this
            // "isn't particularly illuminating". [1]
            //
            // [1] https://www.craftinginterpreters.com/compiling-expressions.html#parsers-for-tokens
            //
            // TODO: See how other Lox implementations do this.
            self.error("Too many constants in one chunk.");
            return 0u8;
        }

        constant as u8
    }

    fn end_compiler(&mut self) {
        self.emit_return();
        #[cfg(feature = "debug_print_code")]
        {
            if !self.parser.had_error {
                self.compiling_chunk.disassemble("code");
            }
        }
    }

    fn binary(&mut self) {
        let operator_kind = self.parser.previous.kind;
        let rule = self.get_rule(operator_kind);
        // Compile the right operand.
        //
        // To quote Robert Nystrom, the author of craftinginterpreters.com: "We use one higher level
        // of precedence for the right operand because the binary operators are left-associative.
        // Given a series of the same operator, like:
        //
        //   1 + 2 + 3 + 4
        //
        // We want to parse it like:
        //
        //   ((1 + 2) + 3) + 4
        //
        // Thus, when parsing the right-hand operand to the first +, we want to consume the 2, but
        // not the rest, so we use one level above +'s precedence. But if our operator was
        // right-associative, this would be wrong. Given:
        //
        //   a = b = c = d
        //
        // Since assignment is right-associative, we want to parse it as:
        //
        //   a = (b = (c = d))
        //
        // To enable that, we would call parsePrecedence() with the same precedence as the current
        // operator."
        let precedence = rule.precedence.next();
        self.parse_precedence(precedence);

        self.emit_byte(
            match operator_kind {
                TokenKind::Minus => OpCode::Subtract,
                TokenKind::Plus => OpCode::Add,
                TokenKind::Slash => OpCode::Divide,
                TokenKind::Star => OpCode::Multiply,
                _ => unreachable!(),
            }
            .to_u8(),
        );
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment); // Parse everything.
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenKind::RightParen, "Expect ')' after expression.");
    }

    fn number(&mut self) {
        let value = Value(
            self.parser
                .previous
                .lexeme_to_string()
                .parse::<f64>()
                .unwrap_or(0f64),
        );
        self.emit_constant(value);
    }

    // Robert Nystrom, the author of craftinginterpreters.com, has this to say about unary():
    //
    // "Emitting the OP_NEGATE instruction after the operands does mean that the current token when
    // the bytecode is written is not the - token. That mostly doesn't matter, except that we use
    // that token for the line number to associate with that instruction.
    //
    // This means if you have a multi-line negation expression, like:
    //
    //     print -
    //       true;
    //
    // Then the runtime error will be reported on the wrong line. Here, it would show the error on
    // line 2, even though the - is on line 1. A more robust approach would be to store the token's
    // line before compiling the operand and then pass that into emitByte(), but I wanted to keep
    // things simple for the book."
    //
    // TODO: Address this comment.
    fn unary(&mut self) {
        let operator_kind = self.parser.previous.kind;

        // Compile the operand.
        //
        // Only parse unary expressions and those of higher precedence. So, given the following
        // expression:
        //
        //   -a.b + c;
        //
        // ...only parse the "-a.b" part. If we did not do this, the - would apply to all of
        // "a.b + c", which does not follow the Lox specification.
        self.parse_precedence(Precedence::Unary);

        // Emit the operator instruction.
        match operator_kind {
            TokenKind::Minus => self.emit_byte(OpCode::Negate.to_u8()),
            _ => unreachable!(),
        }
    }

    // Parse only the subsequent tokens with a precedence greater than or equal to the given
    // precedence.
    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = self.get_rule(self.parser.previous.kind).prefix;
        match prefix_rule {
            Some(rule) => rule(self),
            None => {
                self.error("Expect expression.");
                return;
            }
        }

        while precedence <= self.get_rule(self.parser.current.kind).precedence {
            self.advance();
            let infix_rule = self.get_rule(self.parser.previous.kind).infix;
            match infix_rule {
                Some(rule) => rule(self),
                None => unreachable!(),
            }
        }
    }

    fn get_rule(&self, kind: TokenKind) -> ParseRule<'a> {
        match kind {
            TokenKind::LeftParen => {
                ParseRule::new(Some(Compiler::grouping), None, Precedence::None)
            }
            // TODO: Replace all struct constructions with ParseRule::new
            TokenKind::RightParen => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::LeftBrace => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::RightBrace => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Comma => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Dot => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Minus => ParseRule {
                prefix: Some(Compiler::unary),
                infix: Some(Compiler::binary),
                precedence: Precedence::Term,
            },
            TokenKind::Plus => ParseRule {
                prefix: None,
                infix: Some(Compiler::binary),
                precedence: Precedence::Term,
            },
            TokenKind::Semicolon => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Slash => ParseRule {
                prefix: None,
                infix: Some(Compiler::binary),
                precedence: Precedence::Factor,
            },
            TokenKind::Star => ParseRule {
                prefix: None,
                infix: Some(Compiler::binary),
                precedence: Precedence::Factor,
            },
            TokenKind::Bang => ParseRule {
                prefix: None,
                infix: Some(Compiler::binary),
                precedence: Precedence::Factor,
            },
            TokenKind::BangEqual => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Equal => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::EqualEqual => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Greater => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::GreaterEqual => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Less => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::LessEqual => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Identifier => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::String => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Number => ParseRule {
                prefix: Some(Compiler::number),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::And => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Class => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Else => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::False => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::For => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Fun => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::If => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Nil => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Or => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Print => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Return => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Super => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::This => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::True => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Var => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::While => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Error => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Eof => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        }
    }
}

use crate::chunk::Chunk;
use crate::scanner::{NULL_TOKEN, Scanner, Token, TokenKind};

pub(crate) struct Compiler<'a> {
    source: &'a [u8],
    scanner: Scanner<'a>,
    parser: Parser<'a>,
    had_error: bool,
    panic_mode: bool,
}

impl <'a> Compiler<'a> {
    pub fn new(source: &[u8]) -> Compiler {
        Compiler {
            source,
            scanner: Scanner::new(source),
            parser: Parser {
                current: NULL_TOKEN,
                previous: NULL_TOKEN
            },
            had_error: false,
            panic_mode: false,
        }
    }

    pub fn compile_into(&mut self, chunk: &Chunk) -> bool {
        self.had_error = false;
        self.panic_mode = false;

        self.advance();
        self.expression();
        self.consume(TokenKind::Eof, "Expect end of expression.".as_bytes());

        !self.had_error
    }

    fn advance(&mut self) {
        self.parser.previous = self.parser.current;

        loop {
            self.parser.current = self.scanner.scan_token();
            if self.parser.current.kind != TokenKind::Error {
                break;
            }

            self.error_at_current(self.parser.current.lexeme);
        }
    }

    fn consume(&mut self, kind: TokenKind, message: &[u8]) {
        if self.parser.current.kind == kind {
            self.advance();
            return;
        }

        self.error_at_current(message);
    }

    fn error_at_current(&mut self, message: &[u8]) {
        self.error_at(&self.parser.current, message);
    }

    fn error(&mut self, message: &[u8]) {
        self.error_at(&self.parser.previous, message);
    }

    fn error_at(&mut self, token: &Token, message: &[u8]) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        eprint!("[line {}] Error", token.line);

        match token.kind {
            TokenKind::Eof => eprint!(" at end"),
            TokenKind::Error => {},
            // The lexeme came into the VM from the source file, which is read as a Rust string, so
            // the lexeme is guaranteed to be UTF-8.
            _ => eprint!(" at {}", String::from_utf8_lossy(token.lexeme))
        }

        // The message came into the VM as a Rust string, so it is guaranteed to be UTF-8.
        eprintln!(": {}", String::from_utf8_lossy(message));
    }
}

struct Parser<'a> {
    current: Token<'a>,
    previous: Token<'a>
}

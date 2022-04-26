use crate::chunk::{Chunk, OpCode, NULL_CHUNK};
use crate::scanner::{Scanner, Token, TokenKind, NULL_TOKEN};
use crate::value::Value;
use std::u8::MIN;

pub(crate) struct Compiler<'a> {
    source: &'a [u8],
    scanner: Scanner<'a>,
    parser: Parser<'a>,
    compiling_chunk: &'a Chunk,
    had_error: bool,
    panic_mode: bool,
}

impl<'a> Compiler<'a> {
    pub fn new(source: &[u8]) -> Compiler {
        Compiler {
            source,
            scanner: Scanner::new(source),
            parser: Parser {
                current: NULL_TOKEN,
                previous: NULL_TOKEN,
            },
            compiling_chunk: &NULL_CHUNK,
            had_error: false,
            panic_mode: false,
        }
    }

    pub fn compile_into(&mut self, chunk: &Chunk) -> bool {
        self.compiling_chunk = chunk;

        self.had_error = false;
        self.panic_mode = false;

        self.advance();
        self.expression();
        self.consume(TokenKind::Eof, "Expect end of expression.");
        self.end_compiler();
        !self.had_error
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
        self.error_at(&self.parser.current, message);
    }

    fn error(&mut self, message: &str) {
        self.error_at(&self.parser.previous, message);
    }

    fn error_at(&mut self, token: &Token, message: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        eprint!("[line {}] Error", token.line);

        match token.kind {
            TokenKind::Eof => eprint!(" at end"),
            TokenKind::Error => {}
            _ => eprint!(" at {}", token.lexeme_to_string()),
        }

        eprintln!(": {}", message);
    }

    fn current_chunk(&self) -> &Chunk {
        &self.compiling_chunk
    }

    fn emit_byte(&self, byte: u8) {
        self.write_chunk(self.current_chunk(), byte, self.parser.previous.line);
    }

    fn emit_bytes(&self, first: u8, second: u8) {
        self.emit_byte(first);
        self.emit_byte(second);
    }

    fn emit_return(&self) {
        self.emit_byte(OpCode::Return.to_u8())
    }

    fn emit_constant(&mut self, value: Value) {
        self.emit_bytes(OpCode::Constant.to_u8(), self.make_constant(value))
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let constant = self.current_chunk().add_constant(value);
        if constant > u8::MIN as usize {
            // Note: in a real VM, we'd need another bytecode instruction like OP_CONSTANT_16
            // that stores the constant index as two bytes, so that the VM could handle more
            // than 256 constants when needed.
            //
            // The original clox doesn't support this, as apparently the code for doing this
            // "isn't particularly illuminating". [1]
            //
            // [1] https://www.craftinginterpreters.com/compiling-expressions.html#parsers-for-tokens
            self.error("Too many constants in one chunk.");
            return 0u8;
        }

        constant as u8
    }

    fn end_compiler(&self) {
        self.emit_return();
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
}

struct Parser<'a> {
    current: Token<'a>,
    previous: Token<'a>,
}

use crate::scanner::{Scanner, TokenKind};

pub struct Compiler<'a> {
    source: &'a [u8],
}

impl Compiler<'_> {
    pub fn new(source: &[u8]) -> Compiler {
        Compiler { source }
    }

    pub fn compile(&mut self) {
        let mut scanner = Scanner::new(self.source);

        let mut line: i32 = -1;
        loop {
            let token = scanner.scan_token();
            if token.line as i32 != line {
                print!("{:>4} ", token.line);
                line = token.line as i32;
            } else {
                print!("   | ");
            }
            println!("{:?} {:?}", token.kind, std::str::from_utf8(token.lexeme).unwrap());

            if token.kind == TokenKind::Eof {
                break;
            }
        }
    }
}

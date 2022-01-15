use crate::scanner::Scanner;

pub struct Compiler<'a> {
    source: &'a [u8],
}

impl Compiler<'_> {
    pub fn new(source: &[u8]) -> Compiler {
        Compiler { source }
    }

    pub fn compile(&mut self) {
        Scanner::new(self.source);
    }
}

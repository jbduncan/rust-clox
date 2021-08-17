use crate::value::Value;
use std::fmt;
use std::fmt::{Display, Formatter};

pub enum OpCode {
    OpConstant,
    OpReturn,
}

pub struct Chunk {
    code: Vec<u8>,
    lines: Vec<u32>,
    constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            lines: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn write_op_code(&mut self, op_code: OpCode, line: u32) {
        self.write_byte(op_code as u8, line);
    }

    pub fn write_byte(&mut self, byte: u8, line: u32) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);

        print!("{}", self);
    }

    fn fmt_instruction(&self, f: &mut Formatter<'_>, offset: usize) -> Result<usize, fmt::Error> {
        write!(f, "{:04} ", offset)?;
        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            write!(f, "   | ")?;
        } else {
            write!(f, "{:4} ", self.lines[offset])?;
        }

        match self.code[offset] {
            instruction if instruction == OpCode::OpConstant as u8 => {
                self.fmt_constant_instruction(f, "OP_CONSTANT", offset)
            }
            instruction if instruction == OpCode::OpReturn as u8 => {
                self.fmt_simple_instruction(f, "OP_RETURN", offset)
            }
            _ => {
                write!(f, "Unknown opcode {}", self.code[offset])?;
                Ok(offset + 1)
            }
        }
    }

    fn fmt_constant_instruction(
        &self,
        f: &mut Formatter<'_>,
        name: &str,
        offset: usize,
    ) -> Result<usize, fmt::Error> {
        let constant = self.code[offset + 1];
        write!(f, "{: <16} {:4} '", name, constant)?;
        write!(f, "{}", self.constants[constant as usize])?;
        writeln!(f, "'")?;
        Ok(offset + 2)
    }

    fn fmt_simple_instruction(
        &self,
        f: &mut Formatter<'_>,
        name: &str,
        offset: usize,
    ) -> Result<usize, fmt::Error> {
        writeln!(f, "{}", name)?;
        Ok(offset + 1)
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.fmt_instruction(f, offset)?;
        }
        Ok(())
    }
}

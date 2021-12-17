use crate::value::Value;
use std::borrow::Borrow;
use std::fmt;
use std::fmt::{Display, Formatter};

pub enum OpCode {
    OpConstant = 0,
    OpAdd = 1,
    OpSubtract = 2,
    OpMultiply = 3,
    OpDivide = 4,
    OpNegate = 5,
    OpReturn = 6,
}

impl OpCode {
    pub fn from_u8(value: u8) -> Option<OpCode> {
        match value {
            0 => Some(OpCode::OpConstant),
            1 => Some(OpCode::OpAdd),
            2 => Some(OpCode::OpSubtract),
            3 => Some(OpCode::OpMultiply),
            4 => Some(OpCode::OpDivide),
            5 => Some(OpCode::OpNegate),
            6 => Some(OpCode::OpReturn),
            _ => None,
        }
    }

    pub fn to_u8(self) -> u8 {
        return self as u8;
    }
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

    pub fn code(&self) -> &Vec<u8> {
        self.code.borrow()
    }

    pub fn constants(&self) -> &Vec<Value> {
        self.constants.borrow()
    }

    pub fn write_op_code(&mut self, op_code: OpCode, line: u32) {
        self.write_byte(op_code.to_u8(), line);
    }

    pub fn write_byte(&mut self, byte: u8, line: u32) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> u8 {
        self.constants.push(value);
        (self.constants.len() - 1) as u8
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);

        print!("{}", self);
    }

    pub fn disassemble_instruction(&self, offset: u8) {
        let mut buffer = String::new();
        let _ = self.fmt_instruction(&mut buffer, offset as usize);
        print!("{}", buffer);
    }

    fn fmt_instruction(&self, f: &mut dyn fmt::Write, offset: usize) -> Result<usize, fmt::Error> {
        write!(f, "{:04} ", offset)?;
        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            write!(f, "   | ")?;
        } else {
            write!(f, "{:4} ", self.lines[offset])?;
        }

        match OpCode::from_u8(self.code[offset]) {
            Some(OpCode::OpConstant) => self.fmt_constant_instruction(f, "OP_CONSTANT", offset),
            Some(OpCode::OpNegate) => self.fmt_simple_instruction(f, "OP_NEGATE", offset),
            Some(OpCode::OpAdd) => self.fmt_simple_instruction(f, "OP_ADD", offset),
            Some(OpCode::OpSubtract) => self.fmt_simple_instruction(f, "OP_SUBTRACT", offset),
            Some(OpCode::OpMultiply) => self.fmt_simple_instruction(f, "OP_MULTIPLY", offset),
            Some(OpCode::OpDivide) => self.fmt_simple_instruction(f, "OP_DIVIDE", offset),
            Some(OpCode::OpReturn) => self.fmt_simple_instruction(f, "OP_RETURN", offset),
            _ => {
                write!(f, "Unknown opcode {}", self.code[offset])?;
                Ok(offset + 1)
            }
        }
    }

    fn fmt_constant_instruction(
        &self,
        f: &mut dyn fmt::Write,
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
        f: &mut dyn fmt::Write,
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

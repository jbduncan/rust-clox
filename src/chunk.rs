use crate::value::Value;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub(crate) enum OpCode {
    Constant = 0,
    Nil = 1,
    True = 2,
    False = 3,
    Add = 4,
    Subtract = 5,
    Multiply = 6,
    Divide = 7,
    Negate = 8,
    Return = 9,
}

impl OpCode {
    pub fn from_u8(value: u8) -> Option<OpCode> {
        match value {
            0 => Some(OpCode::Constant),
            1 => Some(OpCode::Nil),
            2 => Some(OpCode::True),
            3 => Some(OpCode::False),
            4 => Some(OpCode::Add),
            5 => Some(OpCode::Subtract),
            6 => Some(OpCode::Multiply),
            7 => Some(OpCode::Divide),
            8 => Some(OpCode::Negate),
            9 => Some(OpCode::Return),
            _ => None,
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

pub(crate) struct Chunk {
    pub code: Vec<u8>,
    pub lines: Vec<usize>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            lines: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn constants(&self) -> &Vec<Value> {
        &self.constants
    }

    pub fn write_op_code(&mut self, op_code: OpCode, line: usize) {
        self.write_byte(op_code.to_u8(), line);
    }

    pub fn write_byte(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    #[cfg(feature = "debug_trace_execution")]
    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);

        print!("{}", self);
    }

    #[cfg(feature = "debug_trace_execution")]
    pub fn disassemble_instruction(&self, offset: usize) {
        let mut buffer = String::new();
        let _ = self.fmt_instruction(&mut buffer, offset);
        print!("{}", buffer);
    }

    #[cfg(feature = "debug_trace_execution")]
    fn fmt_instruction(&self, f: &mut dyn fmt::Write, offset: usize) -> Result<usize, fmt::Error> {
        write!(f, "{:04} ", offset)?;
        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            write!(f, "   | ")?;
        } else {
            write!(f, "{:4} ", self.lines[offset])?;
        }

        match OpCode::from_u8(self.code[offset]) {
            Some(OpCode::Constant) => self.fmt_constant_instruction(f, "OP_CONSTANT", offset),
            Some(OpCode::Nil) => self.fmt_simple_instruction(f, "OP_NIL", offset),
            Some(OpCode::True) => self.fmt_simple_instruction(f, "OP_TRUE", offset),
            Some(OpCode::False) => self.fmt_simple_instruction(f, "OP_FALSE", offset),
            Some(OpCode::Negate) => self.fmt_simple_instruction(f, "OP_NEGATE", offset),
            Some(OpCode::Add) => self.fmt_simple_instruction(f, "OP_ADD", offset),
            Some(OpCode::Subtract) => self.fmt_simple_instruction(f, "OP_SUBTRACT", offset),
            Some(OpCode::Multiply) => self.fmt_simple_instruction(f, "OP_MULTIPLY", offset),
            Some(OpCode::Divide) => self.fmt_simple_instruction(f, "OP_DIVIDE", offset),
            Some(OpCode::Return) => self.fmt_simple_instruction(f, "OP_RETURN", offset),
            _ => {
                write!(f, "Unknown opcode {}", self.code[offset])?;
                Ok(offset + 1)
            }
        }
    }

    #[cfg(feature = "debug_trace_execution")]
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

    #[cfg(feature = "debug_trace_execution")]
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

#[cfg(feature = "debug_trace_execution")]
impl Display for Chunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.fmt_instruction(f, offset)?;
        }
        Ok(())
    }
}

pub(crate) const NULL_CHUNK: Chunk = Chunk {
    code: vec![],
    lines: vec![],
    constants: vec![],
};

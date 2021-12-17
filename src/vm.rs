use crate::chunk::{Chunk, OpCode};
use crate::value::Value;

static STACK_MAX: usize = 256;

pub struct VM {
    chunk: Chunk,
    ip: u8,
    stack: Vec<Value>,
    stack_top: usize,
}

impl VM {
    pub fn new(chunk: Chunk) -> VM {
        let ip = 0;
        let stack = Vec::with_capacity(STACK_MAX);
        let stack_top = 0;
        VM {
            chunk,
            ip,
            stack,
            stack_top,
        }
    }

    pub fn interpret(&mut self) -> InterpretResult {
        self.run()
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            #[cfg(debug_assertions)]
            self.trace_execution();

            let instruction = self.read_byte();
            let op_code = OpCode::from_u8(instruction);
            match op_code {
                Some(OpCode::OpConstant) => {
                    let constant = self.read_constant().to_owned();
                    self.push(constant);
                }
                Some(OpCode::OpAdd) => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a + b);
                }
                Some(OpCode::OpSubtract) => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a - b);
                }
                Some(OpCode::OpMultiply) => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a * b);
                }
                Some(OpCode::OpDivide) => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a / b);
                }
                Some(OpCode::OpNegate) => {
                    let value = -self.pop();
                    self.push(value);
                }
                Some(OpCode::OpReturn) => {
                    println!("{}", self.pop());
                    return InterpretResult::InterpretOk;
                }
                None => {
                    panic!("Unknown opcode {}", instruction);
                }
            }
        }
    }

    fn push(&mut self, constant: Value) {
        self.stack.push(constant);
        self.stack_top += 1;
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    fn read_byte(&mut self) -> u8 {
        let result = self.chunk.code()[self.ip as usize];
        self.ip += 1;
        result
    }

    fn read_constant(&mut self) -> &Value {
        let byte = self.read_byte();
        &self.chunk.constants()[byte as usize]
    }

    #[cfg(debug_assertions)]
    fn trace_execution(&self) {
        print!("          ");
        for slot in self.stack.iter().take(self.stack_top) {
            print!("[ ");
            print!("{}", slot);
            print!(" ]");
        }
        println!();
        self.chunk
            .disassemble_instruction(self.ip - self.chunk.code()[0]);
    }
}

pub enum InterpretResult {
    InterpretOk,
    InterpretCompileError,
    InterpretRuntimeError,
}

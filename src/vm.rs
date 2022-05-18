use crate::chunk::{Chunk, OpCode};
use crate::compiler::Compiler;
use crate::value::Value;

const STACK_MAX: usize = 256;

pub struct VM<'a> {
    source: &'a [u8],
    chunk: Chunk,
    ip: u8,
    stack: [Value; STACK_MAX],
    stack_top: usize,
}

impl VM<'_> {
    pub fn new(source: &str) -> VM {
        let chunk = Chunk::new();
        let ip = 0;
        let stack = [Value(0f64); STACK_MAX];
        let stack_top = 0;
        VM {
            // TODO: Converting to bytes here is forcing us to have to extract substrings that are
            //       heap-allocated, whenever we need to print errors or extract Tokens of kind
            //       TokenKind::Number.
            //       Is there any way we can work with &str throughout the interpreter, rather than
            //       &[u8]?
            source: source.as_bytes(),
            chunk,
            ip,
            stack,
            stack_top,
        }
    }

    pub fn interpret(&mut self) -> InterpretResult {
        let mut chunk = Chunk::new();

        if !Compiler::new(self.source, &mut chunk).compile() {
            return InterpretResult::InterpretCompileError;
        }

        self.chunk = chunk;
        self.ip = 0;

        self.run()
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            #[cfg(feature = "debug_trace_execution")]
            self.trace_execution();

            let instruction = self.read_byte();
            let op_code = OpCode::from_u8(instruction);
            match op_code {
                Some(OpCode::Constant) => {
                    let constant = self.read_constant();
                    self.push(constant);
                }
                Some(OpCode::Add) => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a + b);
                }
                Some(OpCode::Subtract) => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a - b);
                }
                Some(OpCode::Multiply) => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a * b);
                }
                Some(OpCode::Divide) => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a / b);
                }
                Some(OpCode::Negate) => {
                    let value = -self.pop();
                    self.push(value);
                }
                Some(OpCode::Return) => {
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
        self.stack[self.stack_top] = constant;
        self.stack_top += 1;
    }

    fn pop(&mut self) -> Value {
        self.stack_top -= 1;
        self.stack[self.stack_top].to_owned()
    }

    fn read_byte(&mut self) -> u8 {
        let result = self.chunk.code()[self.ip as usize];
        self.ip += 1;
        result
    }

    fn read_constant(&mut self) -> Value {
        let byte = self.read_byte();
        self.chunk.constants()[byte as usize].to_owned()
    }

    #[cfg(feature = "debug_trace_execution")]
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

#[derive(Copy, Clone, PartialEq, Hash)]
pub enum InterpretResult {
    InterpretOk,
    InterpretCompileError,
    InterpretRuntimeError,
}

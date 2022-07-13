use crate::chunk::{Chunk, OpCode};
use crate::compiler::Compiler;
use crate::value::Value;

const STACK_MAX: usize = 256;

pub struct VM<'a> {
    source: &'a [u8],
    chunk: Chunk,
    ip: usize,
    stack: [Value; STACK_MAX],
    stack_top: usize,
}

impl VM<'_> {
    pub fn new(source: &str) -> VM {
        let chunk = Chunk::new();
        let ip = 0;
        let stack = [Value::Number(0f64); STACK_MAX];
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

        match self.run() {
            Ok(()) => InterpretResult::InterpretOk,
            Err(err) => err.to_interpret_result()
        }
    }

    fn run(&mut self) -> Result<(), InterpretError> {
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
                Some(OpCode::Nil) => {
                    self.push(Value::Nil);
                }
                Some(OpCode::True) => {
                    self.push(Value::Bool(true));
                }
                Some(OpCode::False) => {
                    self.push(Value::Bool(false));
                }
                Some(OpCode::Add) => {
                    self.binary_op(#[inline] |a, b| a + b, Value::Number)?;
                }
                Some(OpCode::Subtract) => {
                    self.binary_op(#[inline] |a, b| a - b, Value::Number)?;
                }
                Some(OpCode::Multiply) => {
                    self.binary_op(#[inline] |a, b| a * b, Value::Number)?;
                }
                Some(OpCode::Divide) => {
                    self.binary_op(#[inline] |a, b| a / b, Value::Number)?;
                }
                Some(OpCode::Negate) => {
                    match self.peek(0) {
                        Value::Number(number) => {
                            self.pop();
                            self.push(Value::Number(-number))
                        },
                        _ => {
                            // See [1].
                            self.runtime_error("Operand must be a number.");
                            return Err(InterpretError::InterpretRuntimeError);
                        }
                    }
                }
                Some(OpCode::Return) => {
                    println!("{}", self.pop());
                    return Ok(());
                }
                None => {
                    panic!("Unknown opcode {}", instruction);
                }
            }
        }
    }

    #[inline]
    fn binary_op<T>(&mut self, op: fn(f64, f64) -> T, value_type: fn(T) -> Value) -> Result<(), InterpretError> {
        match (self.peek(0), self.peek(1)) {
            (Value::Number(b), Value::Number(a)) => {
                self.pop();
                self.pop();
                self.push(value_type(op(a, b)));
                Ok(())
            }
            _ => {
                // See [1].
                self.runtime_error("Operands must be numbers.");
                Err(InterpretError::InterpretRuntimeError)
            }
        }
    }

    fn push(&mut self, constant: Value) {
        self.stack[self.stack_top] = constant;
        self.stack_top += 1;
    }

    fn pop(&mut self) -> Value {
        self.stack_top -= 1;
        self.stack[self.stack_top]
    }

    fn peek(&self, distance: usize) -> Value {
        self.stack[self.stack_top - 1 - distance]
    }

    fn read_byte(&mut self) -> u8 {
        let result = self.chunk.code[self.ip as usize];
        self.ip += 1;
        result
    }

    fn read_constant(&mut self) -> Value {
        let byte = self.read_byte();
        self.chunk.constants[byte as usize]
    }

    fn runtime_error(&self, message: &str) {
        eprintln!("{}", message);
        let line = self.chunk.lines[self.ip - 1];
        eprintln!("[line {}] in script", line);
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
            // TODO: Is the commented-out code below needed? It causes panics for 'true' and 'false'
            .disassemble_instruction(self.ip /*- self.chunk.code[0] as usize*/);
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub enum InterpretResult {
    InterpretOk,
    InterpretCompileError,
    InterpretRuntimeError,
}

enum InterpretError {
    InterpretCompileError,
    InterpretRuntimeError,
}

impl InterpretError {
    fn to_interpret_result(&self) -> InterpretResult {
        match self {
            InterpretError::InterpretCompileError => InterpretResult::InterpretCompileError,
            InterpretError::InterpretRuntimeError => InterpretResult::InterpretRuntimeError
        }
    }
}

// [1] Stopping the program on a runtime error, without giving the user any control on what happens,
//     is not ideal, so in a real language, this would be changed.

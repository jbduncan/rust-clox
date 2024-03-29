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
        const INIT: Value = Value::Number(0f64);
        const SIZE: usize = 256;
        let stack = [INIT; SIZE];
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

    pub fn interpret(&mut self) -> Result<(), InterpretError> {
        let mut chunk = Chunk::new();

        if !Compiler::new(self.source, &mut chunk).compile() {
            return Err(InterpretError::InterpretCompileError);
        }

        self.chunk = chunk;
        self.ip = 0;

        self.run()
    }

    // The "beating heart" of the VM.
    //
    // According to author of craftinginterpreters.com, Robert Nystrom, if you
    // wanted to make this more efficient, you'd need to read up on:
    // - "direct threaded code"
    // - "jump table"
    // - "computed goto"
    //
    // He says that in C, the fastest techniques would need non-standard
    // extensions to C or handwritten assembly code. This implies that in Rust,
    // unsafe code or some other technique would be needed.
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
                Some(OpCode::Equal) => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a == b));
                }
                Some(OpCode::Greater) => {
                    self.binary_op(
                        #[inline]
                        |a, b| a > b,
                        Value::Bool,
                    )?;
                }
                Some(OpCode::Less) => {
                    self.binary_op(
                        #[inline]
                        |a, b| a < b,
                        Value::Bool,
                    )?;
                }
                Some(OpCode::Add) => {
                    self.binary_op(
                        #[inline]
                        |a, b| a + b,
                        Value::Number,
                    )?;
                }
                Some(OpCode::Subtract) => {
                    self.binary_op(
                        #[inline]
                        |a, b| a - b,
                        Value::Number,
                    )?;
                }
                Some(OpCode::Multiply) => {
                    self.binary_op(
                        #[inline]
                        |a, b| a * b,
                        Value::Number,
                    )?;
                }
                Some(OpCode::Divide) => {
                    self.binary_op(
                        #[inline]
                        |a, b| a / b,
                        Value::Number,
                    )?;
                }
                Some(OpCode::Not) => {
                    let value = self.pop();
                    self.push(Value::Bool(value.is_falsey()))
                }
                Some(OpCode::Negate) => {
                    match self.peek(0) {
                        Value::Number(number) => {
                            self.pop();
                            self.push(Value::Number(-number))
                        }
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
                    panic!("Unknown opcode {instruction}");
                }
            }
        }
    }

    #[inline]
    fn binary_op<T>(
        &mut self,
        op: fn(f64, f64) -> T,
        value_type: fn(T) -> Value,
    ) -> Result<(), InterpretError> {
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
        // TODO: Remove .clone() when Value is Copy-able
        self.stack[self.stack_top].clone()
    }

    fn peek(&self, distance: usize) -> Value {
        // TODO: Remove .clone() when Value is Copy-able
        self.stack[self.stack_top - 1 - distance].clone()
    }

    fn read_byte(&mut self) -> u8 {
        let result = self.chunk.code[self.ip];
        self.ip += 1;
        result
    }

    fn read_constant(&mut self) -> Value {
        let byte = self.read_byte();
        // TODO: Remove .clone() when Value is Copy-able
        self.chunk.constants[byte as usize].clone()
    }

    fn runtime_error(&self, message: &str) {
        eprintln!("{message}");
        let line = self.chunk.lines[self.ip - 1];
        eprintln!("[line {line}] in script");
    }

    #[cfg(feature = "debug_trace_execution")]
    fn trace_execution(&self) {
        print!("          ");
        for slot in self.stack.iter().take(self.stack_top) {
            print!("[ {slot} ]");
        }
        println!();
        self.chunk
            // TODO: Is the commented-out code below needed? It causes panics for 'true' and 'false'
            .disassemble_instruction(self.ip /*- self.chunk.code[0] as usize*/);
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub enum InterpretError {
    InterpretCompileError,
    InterpretRuntimeError,
}

// [1] Stopping the program on a runtime error, without giving the user any control on what happens,
//     is not ideal, so in a real language, this would be changed.

use rust_clox::chunk::Chunk;
use rust_clox::chunk::OpCode::{OpConstant, OpReturn};
use rust_clox::value::Value;

fn main() {
    let mut chunk = Chunk::new();

    let constant = chunk.add_constant(Value(1.2));
    chunk.write_op_code(OpConstant, 123);
    chunk.write_byte(constant as u8, 123);

    chunk.write_op_code(OpReturn, 123);

    chunk.disassemble("test chunk");
}

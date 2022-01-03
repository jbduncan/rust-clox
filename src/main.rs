use rust_clox::chunk::Chunk;
use rust_clox::chunk::OpCode;
use rust_clox::value::Value;
use rust_clox::vm::VM;
use std::env;
use std::io::{self, BufRead};
use std::process::exit;

fn main() {
    let mut chunk = Chunk::new();

    let constant = chunk.add_constant(Value(1.2));
    chunk.write_op_code(OpCode::Constant, 123);
    chunk.write_byte(constant, 123);

    let constant = chunk.add_constant(Value(3.4));
    chunk.write_op_code(OpCode::Constant, 123);
    chunk.write_byte(constant, 123);

    chunk.write_op_code(OpCode::Add, 123);

    let constant = chunk.add_constant(Value(5.6));
    chunk.write_op_code(OpCode::Constant, 123);
    chunk.write_byte(constant, 123);

    chunk.write_op_code(OpCode::Divide, 123);
    chunk.write_op_code(OpCode::Negate, 123);

    chunk.write_op_code(OpCode::Return, 123);

    chunk.disassemble("test chunk");
    VM::new(chunk).interpret();
}

// TODO: Replace main() when reaching Chapter 16
// fn main2() {
//     exit(run_app());
// }
//
// fn run_app() -> i32 {
//     let args = env::args().collect::<Vec<String>>();
//     if args.len() == 1 {
//         repl();
//     } else if args.len() == 2 {
//         run_file(&args[1]);
//     } else {
//         eprintln!("Usage: clox [path]");
//         return 64;
//     }
//
//     return 0;
// }
//
// fn repl() -> anyhow::Result<()> {
//     // A real-world REPL should be able to handle multiple lines gracefully.
//     let mut lines = io::stdin().lock().lines();
//     // let mut line = String::new();
//     loop {
//         print!("> ");
//
//         match lines.next() {
//             Some(line) => {
//                 interpret(line);
//             }
//             None => {
//                 println!();
//                 break;
//             }
//         }
//     }
// }
//
// fn run_file(path: &str) {
//     todo!()
// }

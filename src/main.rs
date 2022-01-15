use rust_clox::vm::{InterpretResult, VM};
use std::borrow::Borrow;
use std::env;
use std::fs::read;
use std::io::{self, BufRead, Stdout, Write};
use std::process::exit;

fn main() -> anyhow::Result<()> {
    exit(run_app()?);
}

fn run_app() -> anyhow::Result<i32> {
    let args = env::args().collect::<Vec<String>>();
    match args.len() {
        1 => repl(),
        2 => Ok(run_file(&args[1])),
        _ => {
            eprintln!("Usage: clox [path]");
            Ok(64)
        }
    }
}

fn repl() -> anyhow::Result<i32> {
    // A real-world REPL should be able to handle multiple lines gracefully.
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut lines = stdin.lock().lines();
    loop {
        print_immediately(stdout.borrow(), "> ")?;

        match lines.next() {
            Some(line) => {
                VM::new(line?.as_bytes()).interpret();
            }
            None => {
                println!();
                return Ok(0);
            }
        }
    }
}

fn print_immediately(mut stdout: &Stdout, text: &str) -> anyhow::Result<()> {
    print!("{}", text);
    stdout.flush()?;
    Ok(())
}

fn run_file(path: &str) -> i32 {
    match read(path) {
        Ok(source) => match VM::new(source.as_slice()).interpret() {
            InterpretResult::InterpretOk => 0,
            InterpretResult::InterpretCompileError => 65,
            InterpretResult::InterpretRuntimeError => 70,
        },
        Err(error) => {
            eprintln!("{}", error);
            74
        }
    }
}

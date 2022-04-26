use rust_clox::vm::{InterpretResult, VM};
use std::env;
use std::fs::read_to_string;
use std::io::{self, BufRead, Stdout, Write};
use std::process::exit;

fn main() {
    exit(run_app());
}

fn run_app() -> i32 {
    let args = env::args().collect::<Vec<String>>();
    match args.len() {
        1 => repl().map_or_else(
            |error| {
                eprintln!("{}", error);
                70
            },
            |_| 0,
        ),
        2 => run_file(&args[1]),
        _ => {
            eprintln!("Usage: rust-clox [path]");
            64
        }
    }
}

fn repl() -> io::Result<()> {
    // A real-world REPL should be able to handle multiple lines gracefully.
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut lines = stdin.lock().lines();
    loop {
        print_immediately(&stdout, "> ")?;

        match lines.next() {
            Some(line) => {
                VM::new(&line?).interpret();
            }
            None => {
                println!();
                return Ok(());
            }
        }
    }
}

fn print_immediately(mut stdout: &Stdout, text: &str) -> io::Result<()> {
    print!("{}", text);
    stdout.flush()
}

fn run_file(path: &str) -> i32 {
    read_to_string(path).map_or_else(
        |error| {
            eprintln!("{}", error);
            74
        },
        |source| match VM::new(&source).interpret() {
            InterpretResult::InterpretOk => 0,
            InterpretResult::InterpretCompileError => 65,
            InterpretResult::InterpretRuntimeError => 70,
        },
    )
}

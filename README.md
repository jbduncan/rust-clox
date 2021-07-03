# rust-clox

This is my Rust implementation of Lox - a small programming language for scripting -
from the book [Crafting Interpreters](https://www.craftinginterpreters.com/).

It is a port of the C [reference implementation](
https://github.com/munificent/craftinginterpreters) of Lox - clox. I'm making this
port as I develop [my own copy of clox](https://github.com/jbduncan/clox), as a way
to better understand how clox itself works.

# Prerequisites

1. [Install Rust](https://www.rust-lang.org/learn/get-started) on your system.

# Build the executable

```sh
cargo build
```

# Run the executable as a REPL

```sh
cargo run
```

# Run the executable with a Lox script

```sh
cargo run -- <path-to-lox-script>
```

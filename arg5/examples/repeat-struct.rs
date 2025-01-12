//! Repeats a given word a given number of times.
//!
//! This example is functionally identical to repeat.rs.  However, command-line
//! arguments are collected into a struct.  Note that Rust supports
//! simultaneous mutable references to multiple fields of a single object.

use std::env;
use std::process::exit;

#[derive(Default)]
struct Command {
    count: i32,
    word: String,
}

impl Command {
    fn from_args() -> Result<Self, arg5::ParseError> {
        let mut command = Self::default();
        let mut parser = arg5::Parser::new();
        parser.declare_positional("count", &mut command.count);
        parser.declare_positional("word", &mut command.word);
        parser.parse(env::args()).map(|()| command)
    }

    fn run(self) {
        if self.count > 0 {
            for _ in 1..self.count {
                print!("{} ", self.word);
            }
            println!("{}", self.word);
        }
    }
}

fn main() {
    match Command::from_args() {
        Ok(command) => command.run(),
        Err(error) => {
            eprintln!("Error: {}", error.what);
            exit(1);
        }
    }
}

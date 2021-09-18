//! Repeats a given word a given number of times.

use std::env;
use std::process::exit;

fn main() {
    let mut parameters = arg5::Parser::new();

    let mut count = 0;
    parameters.declare_positional("count", &mut count);

    let mut word = String::new();
    parameters.declare_positional("word", &mut word);

    if let Err(err) = parameters.parse(env::args()) {
        eprintln!("Error: {}", err.what);
        exit(1);
    }

    if count > 0 {
        for _ in 1..count {
            print!("{} ", word);
        }
        println!("{}", word);
    }
}

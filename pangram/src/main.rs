// Usage: pangram [--min-length=<N>] <letters> [words-file]

//#![allow(dead_code, unused_variables)]

mod main_error;
mod command;

use command::Command;
use main_error::MainError;

use std::env;

fn main() -> Result<(), MainError> {
    let command = Command::from_args(env::args().skip(1))?;
    println!("{:#?}", command);
    Ok(())
}

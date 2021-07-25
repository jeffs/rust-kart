//! Parses binary numbers as ASCII and prints the resulting text.

use std::error::Error;
use std::io::{self, BufRead as _};

fn execute() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let stdin = stdin.lock();
    for line in stdin.lines() {
        let line = line?;
        for word in line.split_whitespace() {
            let u = u8::from_str_radix(word, 2)?;
            print!("{}", u as char);
        }
        println!();
    }
    Ok(())
}

fn main() {
    if let Err(err) = execute() {
        eprintln!("error: {}", err);
        std::process::exit(1);
    }
}

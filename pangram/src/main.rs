// Usage: pangram [--min-length=<N>] <letters> [words-file]

//#![allow(dead_code, unused_variables)]

mod main_error;
mod command;

use command::Command;
use main_error::MainError;

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main_imp() -> Result<(), MainError> {
    let command = Command::from_args(env::args().skip(1))?;
    println!("{:#?}", command);
    // open the file
    // filter each word that
    //  meets min length requirement
    //  has mandatory letter
    //  has only available letters
    let file = File::open(command.words_file)?;
    for line in BufReader::new(file).lines() {

    }
    Ok(())
}

fn main() {
    match main_imp() {
        Ok(()) => (),
        Err(err) => {
            eprintln!("Error: {}", err.what);
            std::process::exit(1);
        }
    }
}

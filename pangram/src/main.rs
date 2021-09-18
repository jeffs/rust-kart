// Usage: pangram [--min-length=<N>] <letters> [words-file]

mod main_error;

use main_error::MainError;

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main_imp() -> Result<(), MainError> {
    let mut words_file = Some("/usr/share/dict/words".to_string());

    let parameters = arg5::Parser::new();
    parameters.declare_positional("words-file", &mut words_file);
    parameters.parse(env::args())?;

    // open the file
    // filter each word that
    //  meets min length requirement
    //  has mandatory letter
    //  has only available letters
    let file = File::open(words_file)?;
    for _line in BufReader::new(file).lines() {

    }
    Ok(())
}

fn main() {
    if let Err(err) = main_imp() {
        eprintln!("Error: {}", err.what);
        std::process::exit(1);
    }
}

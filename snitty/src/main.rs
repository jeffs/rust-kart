//! For each specified Rust file, prints the line number of each redundant blank
//! comment line.

use std::fs::File;
use std::process::exit;
use std::{env, io};

use snitty::TrailingEmptyCommentLineNos;

fn main_imp() -> io::Result<()> {
    for path in env::args().skip(1) {
        let reader = io::BufReader::new(File::open(&path)?);
        for line_no in TrailingEmptyCommentLineNos::from_buf(reader) {
            println!("{path}:{}", line_no?);
        }
    }

    Ok(())
}

fn main() {
    if let Err(err) = main_imp() {
        eprintln!("error: {err}");
        exit(1);
    }
}

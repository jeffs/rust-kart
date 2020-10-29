mod cli;

use std::io::{self, BufRead};
use take_until::TakeUntilExt;

fn print<I: Iterator<Item = io::Result<String>>>(lines: I) {
    for res in lines {
        match res {
            Ok(line) => {
                println!("{}:{}", line.chars().count(), line);
            }
            Err(err) => {
                let prefix = if err.kind() == io::ErrorKind::InvalidData {
                    "warning"
                } else {
                    "error"
                };
                eprintln!("{}: {}", prefix, err);
            }
        }
    }
}

fn execute<I: Iterator<Item = io::Result<String>>>(_command: cli::Command, line_results: I) {
    print(line_results);
}

pub fn main() {
    let command = cli::Command::from_env();
    if command.files.is_empty() {
        let stdin = io::stdin();
        let lines = stdin.lock().lines().take_until(|res| res.is_err());
        execute(command, lines);
    } else {
        panic!("file parsing is not yet implemented");
    }
}

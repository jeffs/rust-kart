mod cli;
mod files_lines;

use files_lines::FilesLines;
use std::io::{self, BufRead, Write};
use take_until::TakeUntilExt;
use termcolor::{self, WriteColor};

fn print<I: Iterator<Item = io::Result<String>>>(lines: I, color: termcolor::ColorChoice) {
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
                let mut stderr = termcolor::StandardStream::stderr(color);
                let mut res = stderr
                    .set_color(termcolor::ColorSpec::new().set_fg(Some(termcolor::Color::Red)));
                if res.is_ok() {
                    res = write!(&mut stderr, "{}", prefix);
                    let _ = stderr.reset();
                }
                if res.is_err() {
                    eprint!("{}", prefix);
                }
                eprintln!(": len: {}", err);
            }
        }
    }
}

fn execute<I: Iterator<Item = io::Result<String>>>(command: cli::Command, line_results: I) {
    // TODO Perform requested operation, rather than always printing all lines.
    print(line_results, command.color);
}

pub fn main() {
    let command = cli::Command::from_env();
    if command.files.is_empty() {
        let stdin = io::stdin();
        let lines = stdin.lock().lines().take_until(|res| res.is_err());
        execute(command, lines);
    } else {
        let lines = FilesLines::new(command.files.iter().cloned());
        execute(command, lines);
    }
}

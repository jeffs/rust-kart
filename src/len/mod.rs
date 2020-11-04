mod args;
mod command;
mod files_lines;
mod log;
mod op;

use command::Command;
use files_lines::FilesLines;
use log::Log;
use op::Op;
use std::io::{self, BufRead};
use take_until::TakeUntilExt;

fn print<I: Iterator<Item = io::Result<String>>>(lines: I, log: &Log) {
    for res in lines {
        match res {
            Ok(line) => {
                println!("{}:{}", line.chars().count(), line);
            }
            Err(err) => {
                if err.kind() == io::ErrorKind::InvalidData {
                    log.warning(err);
                } else {
                    log.error(err);
                }
            }
        }
    }
}

/// Performs the specified operation on the specified input lines, and prints the results to
/// stdout.  Any Error from `line_results` is printed to the specified log.
fn execute<I: Iterator<Item = io::Result<String>>>(op: Op, line_results: I, log: Log) {
    match op {
        Op::All => print(line_results, &log),
        _ => log.fatal(format!("Op::{:?}: not yet implemented", op)),
    }
}

pub fn main() {
    let command = Command::from_env();
    let log = Log::new(command.color);
    if command.files.is_empty() {
        let stdin = io::stdin();
        let lines = stdin.lock().lines().take_until(|res| res.is_err());
        execute(command.op, lines, log);
    } else {
        let lines = FilesLines::new(command.files.iter().cloned());
        execute(command.op, lines, log);
    }
}

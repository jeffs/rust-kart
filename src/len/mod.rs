mod args;
mod command;
mod files_lines;
mod log;
mod max_line;
mod op;

use command::Command;
use files_lines::FilesLines;
use log::Log;
use max_line::MaxLine;
use op::Op;
use std::io::{self, BufRead};
use take_until::TakeUntilExt;

fn print<I>(lines: I, log: &Log)
where
    I: IntoIterator<Item = io::Result<String>>,
{
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

// Returns the longest line, as well as any errors.
#[allow(dead_code)]
fn max_line<I>(lines: I) -> impl Iterator<Item = io::Result<String>>
where
    I: IntoIterator<Item = io::Result<String>>,
{
    MaxLine::new(lines).take_until(|res| match res {
        Err(err) if err.kind() != io::ErrorKind::InvalidData => true,
        _ => false,
    })
}

/// Performs the specified operation on the specified input lines, and prints
/// the results to stdout.  Any Error from the specified lines is printed to
/// the specified log.
fn execute<I>(op: Op, lines: I, log: Log)
where
    I: IntoIterator<Item = io::Result<String>>,
{
    match op {
        Op::All => print(lines, &log),
        Op::Max => print(max_line(lines), &log),
        // Op::Min,
        // Op::One,
        // Op::ReverseSort,
        // Op::Sort,
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
        let files = command.files.iter().cloned();
        let lines = FilesLines::new(files).take_until(|res| match res {
            Err(err) if err.kind() != io::ErrorKind::InvalidData => true,
            _ => false,
        });
        execute(command.op, lines, log);
    }
}

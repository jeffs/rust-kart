mod args;
mod command;
mod expect;
mod files_lines;
mod log;
mod max_line;
mod op;

use command::Command;
use files_lines::FilesLines;
use log::Log;
use max_line::{longest, shortest};
use op::Op;
use std::io::{self, BufRead};
use std::path::PathBuf;
use take_until::TakeUntilExt;

fn is_hard_err(res: &io::Result<String>) -> bool {
    res.as_ref()
        .err()
        .filter(|err| err.kind() != io::ErrorKind::InvalidData)
        .is_some()
}

fn print<I>(lines: I, log: &Log)
where
    I: IntoIterator<Item = io::Result<String>>,
{
    for res in lines {
        match res {
            Ok(line) => println!("{}:{}", line.chars().count(), line),
            Err(err) if is_hard_err(&res) => log.error(err),
            Err(err) => log.warning(err),
        }
    }
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
        Op::Max => print(longest(lines).take_until(is_hard_err), &log),
        Op::Min => print(shortest(lines).take_until(is_hard_err), &log),
        // Op::One,
        // Op::ReverseSort,
        // Op::Sort,
        _ => log.fatal(format!("Op::{:?}: not yet implemented", op)),
    }
}

/// Stops on the first error.
fn execute_stdin(op: Op, log: Log) {
    let stdin = io::stdin();
    let lines = stdin.lock().lines().take_until(|res| res.is_err());
    execute(op, lines, log);
}

/// Stops on the first error other than InvalidData.  Binary files cause
/// warnings, but don't cause the program to exit, since we may still find
/// useful data in subsequent files.
fn execute_files(op: Op, files: Vec<PathBuf>, log: Log) {
    let lines = FilesLines::new(files.iter()).take_until(is_hard_err);
    execute(op, lines, log);
}

pub fn main() {
    let command = Command::from_env();
    let log = Log::new(command.color);
    if command.files.is_empty() {
        execute_stdin(command.op, log);
    } else {
        execute_files(command.op, command.files, log);
    }
}

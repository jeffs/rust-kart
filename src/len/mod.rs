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

enum Status {
    Success,
    Failure,
}

/// This function is used to distinguish recoverable errors (specifically,
/// non-UTF-8 input files) from "hard" errors that should cause the program to
/// exit with bad status.
fn is_hard_err(res: &io::Result<String>) -> bool {
    match res {
        Ok(_) => false,
        Err(err) => err.kind() != io::ErrorKind::InvalidData,
    }
}

fn len(line: &String) -> usize {
    line.chars().count()
}

fn reverse_len(line: &String) -> usize {
    usize::MAX - len(line)
}

fn vent<I>(lines: I, log: Log) -> Status
where
    I: Iterator<Item = io::Result<String>>,
{
    for res in lines {
        match res {
            Ok(line) => {
                println!("{}:{}", line.chars().count(), line);
            }
            Err(err) if is_hard_err(&res) => {
                log.error(err);
                return Status::Failure;
            }
            Err(err) => {
                log.warning(err);
            }
        }
    }
    Status::Success
}

fn partition_errs<I>(lines: I) -> io::Result<(Vec<io::Result<String>>, Vec<io::Result<String>>)>
where
    I: Iterator<Item = io::Result<String>>,
{
    let (lines, mut errs): (Vec<_>, Vec<_>) = lines.partition(|res| res.is_ok());
    if let Some(pos) = errs.iter().position(is_hard_err) {
        return Err(errs.remove(pos).unwrap_err());
    }
    Ok((lines, errs))
}

fn vent_sort_by_key<I>(lines: I, key: fn(&String) -> usize, log: Log) -> Status
where
    I: Iterator<Item = io::Result<String>>,
{
    match partition_errs(lines) {
        Ok((mut lines, errs)) => {
            lines.sort_by_key(|res| key(res.as_ref().unwrap()));
            vent(errs.into_iter().chain(lines), log);
            Status::Success
        }
        Err(err) => {
            log.error(err);
            Status::Failure
        }
    }
}

/// Performs the specified operation on the specified input lines, and prints
/// the results to stdout.  Any Error from the specified lines is printed to
/// the specified log.
fn execute<I>(op: Op, lines: I, log: Log) -> Status
where
    I: Iterator<Item = io::Result<String>>,
{
    match op {
        Op::All => vent(lines, log),
        Op::Max => vent(longest(lines), log),
        Op::Min => vent(shortest(lines), log),
        Op::One => vent(lines.take_until(|res| res.is_ok()), log),
        Op::ReverseSort => vent_sort_by_key(lines, reverse_len, log),
        Op::Sort => vent_sort_by_key(lines, len, log),
    }
}

fn execute_stdin(op: Op, log: Log) -> Status {
    let stdin = io::stdin();
    let lines = stdin.lock().lines().take_until(|res| res.is_err());
    execute(op, lines, log)
}

fn execute_files(op: Op, files: Vec<PathBuf>, log: Log) -> Status {
    let lines = FilesLines::new(files.iter()).take_until(is_hard_err);
    execute(op, lines, log)
}

pub fn main() {
    let command = Command::from_env();
    let log = Log::new(command.color);
    let status = if command.files.is_empty() {
        execute_stdin(command.op, log)
    } else {
        execute_files(command.op, command.files, log)
    };
    if let Status::Failure = status {
        std::process::exit(1);
    }
}

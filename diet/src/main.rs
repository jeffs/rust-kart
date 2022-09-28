use std::error::Error;
use std::fmt;
use std::io;
use std::num::ParseIntError;
use std::process::exit;

#[derive(Debug)]
enum MainError {
    Io(io::Error),
    NoHeader,
    HeaderMissingKcal,
    RowMissingKcal { line_no: usize },
    Parse { line_no: usize, err: ParseIntError },
}

impl fmt::Display for MainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MainError::Io(err) => write!(f, "{err}"),
            MainError::NoHeader | MainError::HeaderMissingKcal => {
                write!(f, "CSV needs header identifying kcal field")
            }
            MainError::RowMissingKcal { line_no } => {
                write!(f, "line {line_no}: row needs kcal value")
            }
            MainError::Parse { line_no, err } => write!(f, "line {line_no}: {err}"),
        }
    }
}

impl Error for MainError {}

fn sum_kcal(buf: impl io::BufRead) -> Result<i32, MainError> {
    let mut lines = buf
        .lines()
        .map(|result| result.map_err(|err| MainError::Io(err)));
    let kcal_index = lines
        .next()
        .ok_or(MainError::NoHeader)??
        .split(',')
        .position(|s| s == "kcal")
        .ok_or(MainError::HeaderMissingKcal)?;
    let mut total_kcal: i32 = 0;
    for (line_no, line) in lines.enumerate().map(|(index, line)| (index + 2, line)) {
        let line = line?;
        let kcal: i32 = line
            .split(',')
            .nth(kcal_index)
            .ok_or_else(|| MainError::RowMissingKcal { line_no })?
            .parse()
            .map_err(|err| MainError::Parse { line_no, err })?;
        total_kcal += kcal;
    }
    Ok(total_kcal)
}

fn main() {
    if std::env::args().count() > 1 {
        eprintln!("usage: diet < CSV-FILE");
        exit(2);
    }

    let reader = io::BufReader::new(io::stdin());
    match sum_kcal(reader) {
        Ok(total_kcal) => {
            println!("{total_kcal}");
        }
        Err(what) => {
            eprintln!("error: {what}");
            exit(1);
        }
    }
}

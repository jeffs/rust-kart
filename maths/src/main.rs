use std::{env, error::Error, io::stdin, num::ParseFloatError, process};

use crate::function::{error::NoFunc, Function};

mod function;

type BoxedError = Box<dyn Error>;

fn apply_args(func: &dyn Function, first: f64, rest: &[f64]) -> f64 {
    rest.into_iter().fold(first, |x, &y| func.apply(x, y))
}

fn parse_line(line: &str) -> Result<impl Iterator<Item = f64>, ParseFloatError> {
    let values: Result<Vec<f64>, _> = line
        .split_ascii_whitespace()
        .map(|word| word.parse())
        .collect();
    Ok(values?.into_iter())
}

fn apply_line<I: Iterator<Item = f64>>(func: &dyn Function, args: I) -> f64 {
    let value = args.fold(func.identity(), |x, y| func.apply(x, y));
    eprintln!("{} {value}", func.symbol());
    value
}

fn apply_stdin(func: &dyn Function) -> Result<f64, BoxedError> {
    // I wish Rust had Scala-style comprehensions.
    stdin()
        .lines()
        .map(|result| -> Result<String, BoxedError> { Ok(result?) }) // Box io::Error
        .map(|result| result.and_then(|line| Ok(parse_line(&line)?))) // Parse line into Vec<f64>
        .map(|result| result.map(|args| apply_line(func, args))) // Apply monoid to each line
        .reduce(|result, line| Ok(func.apply(result?, line?))) // Apply monoid to per-line results
        .unwrap_or(Ok(func.identity())) // Default to monoid identity if stdin was empty
}

fn parse_args() -> Result<(&'static dyn Function, Vec<f64>), BoxedError> {
    let mut args = env::args().skip(1);
    let func = args.next().ok_or(NoFunc)?;
    let func = function::parse(&func)?;
    let args: Result<Vec<f64>, _> = args.map(|arg| arg.parse()).collect();
    let args = args?;
    Ok((func, args))
}

/// Applies the passed monoid to the rest of the args if any, or else to stdin.
fn main_imp() -> Result<(), BoxedError> {
    let (func, args) = parse_args()?;
    let answer = match args.as_slice() {
        [first, rest @ ..] => apply_args(func, *first, rest),
        _ => apply_stdin(func)?,
    };
    println!("= {answer}");
    Ok(())
}

fn main() {
    if let Err(err) = main_imp() {
        eprintln!("error: {err}");
        process::exit(1);
    }
}

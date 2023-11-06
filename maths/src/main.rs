use std::{env, error::Error, io::stdin, process};

use crate::function::{error::NoFunc, Function};

mod function;

fn apply_args<I: IntoIterator<Item = String>>(
    func: &dyn Function,
    args: I,
) -> Result<f64, Box<dyn Error>> {
    let mut acc: f64 = func.identity();
    for arg in args {
        let value: f64 = arg.parse()?;
        acc = func.apply(acc, value);
    }
    Ok(acc)
}

fn apply_stdin(func: &dyn Function) -> Result<(), Box<dyn Error>> {
    let sym = func.symbol();
    let mut acc: f64 = func.identity();
    println!("      {acc}");
    for line in stdin().lines() {
        let line = line?;
        for word in line.split_ascii_whitespace() {
            let value: f64 = word.parse()?;
            acc = func.apply(acc, value);
            println!("    {sym} {value} = {acc}");
        }
    }
    Ok(())
}

fn main_imp() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1).peekable();
    let arg = args.next().ok_or(NoFunc)?;
    let func = function::parse(&arg)?;
    if args.peek().is_none() {
        apply_stdin(func)
    } else {
        println!("{}", apply_args(func, args)?);
        Ok(())
    }
}

fn main() {
    if let Err(err) = main_imp() {
        eprintln!("error: {err}");
        process::exit(1);
    }
}

//! Prints the product of numbers on each line of input.

use std::{error::Error, io::stdin, process::exit};

fn main_imp() -> Result<(), Box<dyn Error>> {
    for line in stdin().lines() {
        let numbers: Result<Vec<f64>, _> = line?
            .split_ascii_whitespace()
            .map(|word| word.parse())
            .collect();
        let product: f64 = numbers?.iter().product();
        println!("{product}");
    }
    Ok(())
}

fn main() {
    if let Err(err) = main_imp() {
        eprintln!("error: {err}");
        exit(1);
    }
}

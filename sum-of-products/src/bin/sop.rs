use std::{error::Error, io::stdin};

fn main() -> Result<(), Box<dyn Error>> {
    let products: Result<Vec<f64>, _> = stdin()
        .lines()
        .map(|line| -> Result<f64, Box<dyn Error>> {
            let numbers: Result<Vec<f64>, _> = line?
                .split_ascii_whitespace()
                .map(|word| word.parse())
                .collect();
            Ok(numbers?.into_iter().product())
        })
        .collect();
    let products = products?;
    let sum: f64 = products.iter().sum();

    for product in products {
        println!("{product:>8}");
    }
    println!("--------");
    println!("{sum:>8}");

    Ok(())
}

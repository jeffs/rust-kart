use std::{error::Error, io::stdin};

fn main() -> Result<(), Box<dyn Error>> {
    let parsed = stdin()
        .lines()
        .map(|line| -> Result<_, Box<dyn Error>> {
            let values: Result<Vec<f64>, _> = line?
                .split_ascii_whitespace()
                .map(|word| word.parse())
                .collect();
            Ok(values?)
        })
        .collect::<Result<Vec<Vec<f64>>, _>>()?;
    let products: Vec<f64> = parsed
        .iter()
        .map(|values| values.iter().product::<f64>())
        .collect();
    let sum: f64 = products.iter().sum();

    let formulas = parsed.into_iter().map(|values| {
        let columns: Vec<String> = values
            .into_iter()
            .map(|value| format!("{value:>3}"))
            .collect();
        columns.join("  * ")
    });

    let mut pairs = products.into_iter().zip(formulas.into_iter());
    if let Some((product, formula)) = pairs.next() {
        println!("{product:>8}   = {formula:>18}");
        for (product, formula) in pairs {
            println!("+ {product:>6}   = {formula:>18}");
        }
    }
    println!("--------");
    println!("{sum:>8}");

    Ok(())
}

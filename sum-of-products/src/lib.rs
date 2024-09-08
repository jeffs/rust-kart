mod rendering;

use std::{error::Error, io, iter};

pub struct SumOfProducts {
    pub products: Vec<f64>,
    pub sum: f64,
}

#[must_use]
pub fn compute(parsed: &[Vec<f64>]) -> SumOfProducts {
    let products: Vec<f64> = parsed
        .iter()
        .map(|values| values.iter().product())
        .collect();
    let sum = products.iter().sum();
    SumOfProducts { products, sum }
}

/// # Errors
///
/// Will return any `Err` from `lines`, or an `Err` if any word cannot be parsed as an f64.
pub fn parse<I: Iterator<Item = io::Result<String>>>(
    lines: I,
) -> Result<Vec<Vec<f64>>, Box<dyn Error>> {
    lines
        .map(|line| -> Result<Vec<f64>, _> {
            // Convert each line from a string to a vector of numbers.
            let values: Result<Vec<f64>, _> =
                line?.split_ascii_whitespace().map(str::parse).collect();
            Ok(values?)
        })
        // Discard empty vectors.
        .filter(|result| !result.as_ref().is_ok_and(Vec::is_empty))
        .collect()
}

pub fn render(output: SumOfProducts, input: &[Vec<f64>]) -> Result<String, &'static str> {
    use rendering::*;
    let SumOfProducts { products, sum } = output;
    let eq = " = ";
    let formulas = render_formulas(input)?;
    let sum_width = sum.to_string().len();
    let total_width = formula_width(&formulas) + eq.len() + sum_width;
    let lines: Vec<String> = formulas
        .iter()
        .zip(products.iter())
        .map(|(formula, product)| format!("{formula}{eq}{product:>0$}", sum_width))
        .chain(iter::once(format!(
            "{:>1$}\n{sum:>1$}",
            "----", total_width
        )))
        .collect();
    Ok(lines.join("\n"))
}

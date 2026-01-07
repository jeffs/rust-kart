#![doc = include_str!("../README.md")]

mod rendering;

use std::{error::Error, io, iter};

/// Result of computing products and their sum.
pub struct SumOfProducts {
    /// The product of each input row.
    pub products: Vec<f64>,
    /// The sum of all products.
    pub sum: f64,
}

const EQ: &str = " = ";
const SEP: &str = "----";

/// Computes the product of each row and the sum of all products.
#[must_use]
pub fn compute(parsed: &[Vec<f64>]) -> SumOfProducts {
    let products: Vec<f64> = parsed
        .iter()
        .map(|values| values.iter().product())
        .collect();
    let sum = products.iter().sum();
    SumOfProducts { products, sum }
}

/// Parses lines of whitespace-separated numbers into rows of f64 values.
///
/// # Errors
///
/// Returns an error if reading fails or any word cannot be parsed as f64.
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

/// Formats the computation as aligned formulas with a sum line.
#[must_use]
pub fn render(output: SumOfProducts, input: &[Vec<f64>]) -> String {
    let SumOfProducts { products, sum } = output;
    let formulas = rendering::render_formulas(input);
    let sum_width = sum.to_string().len();
    let width = rendering::formula_width(&formulas) + EQ.len() + sum_width;
    formulas
        .iter()
        .zip(products.iter())
        .map(|(formula, product)| format!("{formula}{EQ}{product:>sum_width$}"))
        .chain(iter::once(format!("{SEP:>width$}\n{sum:>width$}")))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute() {
        let input = vec![vec![2.0, 3.0], vec![4.0, 5.0]];
        let result = compute(&input);
        assert_eq!(result.products, vec![6.0, 20.0]);
        assert_eq!(result.sum, 26.0);
    }

    #[test]
    fn test_parse() {
        let lines = vec![Ok("2 3".to_string()), Ok("4 5".to_string())];
        let result = parse(lines.into_iter()).unwrap();
        assert_eq!(result, vec![vec![2.0, 3.0], vec![4.0, 5.0]]);
    }

    #[test]
    fn test_parse_skips_empty_lines() {
        let lines = vec![Ok("2 3".to_string()), Ok("".to_string()), Ok("4 5".to_string())];
        let result = parse(lines.into_iter()).unwrap();
        assert_eq!(result, vec![vec![2.0, 3.0], vec![4.0, 5.0]]);
    }
}

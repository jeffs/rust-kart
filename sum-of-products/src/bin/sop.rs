use std::{error::Error, io::stdin, process::exit};

fn main() -> Result<(), Box<dyn Error>> {
    // Parse input.
    let parsed: Result<Vec<Vec<f64>>, _> = stdin()
        .lines()
        .map(|line| -> Result<_, Box<dyn Error>> {
            let values: Result<Vec<f64>, _> = line?
                .split_ascii_whitespace()
                .map(|word| word.parse())
                .collect();
            Ok(values?)
        })
        .collect();

    // Filter out empty lines.
    let parsed: Vec<Vec<f64>> = parsed?
        .into_iter()
        .filter(|values| !values.is_empty())
        .collect();

    // Compute results.
    let products: Vec<f64> = parsed
        .iter()
        .map(|values| values.iter().product::<f64>())
        .collect();
    let sum: f64 = products.iter().sum();

    // Format output.
    //
    // * Compute width of each column for formula terms:
    let word_vecs: Vec<Vec<String>> = parsed
        .iter()
        .map(|values| values.iter().map(|value| value.to_string()).collect())
        .collect();
    let column_count = word_vecs
        .iter()
        .map(|words| words.len())
        .max()
        .unwrap_or_else(|| {
            eprintln!("error: expected at least one input value");
            exit(2);
        });
    let widths: Vec<usize> = (0..column_count)
        .map(|index| {
            word_vecs
                .iter()
                .flat_map(|words| words.iter().nth_back(index).map(|word| word.len()))
                .max()
                .unwrap_or_default()
        })
        .collect();

    // * Render formulas:
    let formulas: Vec<String> = parsed
        .into_iter()
        .map(|values| {
            let empty_count = widths.len() - values.len();
            let empty_columns: Vec<String> = (0..empty_count)
                .map(|index| " ".repeat(widths[index]))
                .collect();
            let columns: Vec<String> = values
                .into_iter()
                .zip(widths[empty_count..].iter())
                .map(|(value, width)| format!("{value:>0$}", width))
                .collect();
            let joined = columns.join(" * ");
            if empty_columns.is_empty() {
                joined
            } else {
                format!("{}   {}", empty_columns.join("   "), columns.join(" * "))
            }
        })
        .collect();
    
    // * Determine total width, so we can right-align sum (below):
    let formula_width = formulas[0].len();
    debug_assert!(formulas
        .iter()
        .all(|formula| formula.len() == formula_width));
    let product_width = products
        .iter()
        .map(|product| product.to_string().len())
        .max()
        .expect("at least one product, since input is known to be nonempty");
    let total_width = formula_width + " = ".len() + product_width;

    // Print output table: "formula = product" lines, then "----", then sum.
    for (product, formula) in products.into_iter().zip(formulas.into_iter()) {
        println!("{formula} = {product:>4}");
    }
    println!("{:>1$}\n{sum:>1$}", "----", total_width);

    // Return success.
    Ok(())
}

use std::{error::Error, io, process::exit};

fn parse_input<I: Iterator<Item = io::Result<String>>>(
    lines: I,
) -> Result<Vec<Vec<f64>>, Box<dyn Error>> {
    lines
        .map(|line| -> Result<_, Box<dyn Error>> {
            // Convert each line from a string to a vector of numbers.
            let values: Result<Vec<f64>, _> = line?
                .split_ascii_whitespace()
                .map(|word| word.parse())
                .collect();
            Ok(values?)
        })
        .filter(|result| {
            // Discard empty vectors.
            !result
                .as_ref()
                .map(|values| values.is_empty())
                .unwrap_or_default()
        })
        .collect()
}

fn main_imp<I: Iterator<Item = io::Result<String>>>(lines: I) -> Result<String, Box<dyn Error>> {
    let parsed = parse_input(lines)?;

    // Compute results.
    let products: Vec<f64> = parsed
        .iter()
        .map(|values| values.iter().product::<f64>())
        .collect();
    let sum: f64 = products.iter().sum();

    // Format output.
    //
    // * Compute width of each column for formula terms
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
        .rev()
        .map(|index| {
            word_vecs
                .iter()
                .flat_map(|words| words.iter().nth_back(index).map(|word| word.len()))
                .max()
                .unwrap_or_default()
        })
        .collect();

    // * Render formulas.  TODO: Align decimal points.
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

    // dbg!(&formulas);

    // * Determine total width, so we can right-align sum (below)
    debug_assert!(!formulas.is_empty());
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

    let mut output_lines = Vec::new();
    // Print output table: "formula = product" lines, then "----", then sum.
    for (product, formula) in products.into_iter().zip(formulas.into_iter()) {
        output_lines.push(format!("{formula} = {product:>4}"));
    }
    output_lines.push(format!("{:>1$}\n{sum:>1$}", "----", total_width));

    // Return success.
    Ok(output_lines.join("\n"))
}

fn main() {
    match main_imp(io::stdin().lines()) {
        Ok(output) => {
            println!("{output}")
        }
        Err(err) => {
            eprintln!("error: {err}");
            exit(1);
        }
    }
}

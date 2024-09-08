fn compute_value_widths(parsed: &[Vec<f64>]) -> Result<Vec<usize>, &'static str> {
    let word_vecs = render_values(parsed);
    let column_count = word_vecs
        .iter()
        .map(Vec::len)
        .max()
        .ok_or("expected at least one input value")?;
    Ok((0..column_count)
        .rev()
        .map(|index| {
            word_vecs
                .iter()
                .filter_map(|words| words.iter().nth_back(index).map(String::len))
                .max()
                .unwrap_or_default()
        })
        .collect())
}

pub fn formula_width(formulas: &[String]) -> usize {
    debug_assert!(!formulas.is_empty());
    let formula_width = formulas[0].len();
    debug_assert!(formulas
        .iter()
        .all(|formula| formula.len() == formula_width));
    formula_width
}

// TODO: Align decimal points.
pub fn render_formulas(parsed: &[Vec<f64>]) -> Result<Vec<String>, &'static str> {
    let mul = " * ";
    let widths = compute_value_widths(parsed)?;
    Ok(parsed
        .iter()
        .map(|values| {
            let empty_count = widths.len() - values.len();
            let empty_columns: Vec<String> = (0..empty_count)
                .map(|index| " ".repeat(widths[index] + mul.len()))
                .collect();
            let columns: Vec<String> = values
                .iter()
                .zip(widths[empty_count..].iter())
                .map(|(value, width)| format!("{value:>width$}"))
                .collect();
            empty_columns.concat() + &columns.join(mul)
        })
        .collect())
}

fn render_values(parsed: &[Vec<f64>]) -> Vec<Vec<String>> {
    parsed
        .iter()
        .map(|values| values.iter().map(ToString::to_string).collect())
        .collect()
}

fn compute_value_widths(parsed: &[Vec<f64>]) -> Result<Vec<usize>, &'static str> {
    let word_vecs = render_values(&parsed);
    let column_count = word_vecs
        .iter()
        .map(|words| words.len())
        .max()
        .ok_or("expected at least one input value")?;
    Ok((0..column_count)
        .rev()
        .map(|index| {
            word_vecs
                .iter()
                .flat_map(|words| words.iter().nth_back(index).map(|word| word.len()))
                .max()
                .unwrap_or_default()
        })
        .collect())
}

pub fn compute_total_width(formulas: &[String], sum: f64) -> usize {
    debug_assert!(!formulas.is_empty());
    let formula_width = formulas[0].len();
    debug_assert!(formulas
        .iter()
        .all(|formula| formula.len() == formula_width));
    formula_width + " = ".len() + sum.to_string().len()
}

fn render_values(parsed: &[Vec<f64>]) -> Vec<Vec<String>> {
    parsed
        .iter()
        .map(|values| values.iter().map(|value| value.to_string()).collect())
        .collect()
}

// TODO: Align decimal points.
pub fn render_formulas(parsed: &[Vec<f64>]) -> Result<Vec<String>, &'static str> {
    let widths = compute_value_widths(parsed)?;
    Ok(parsed
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
        .collect())
}

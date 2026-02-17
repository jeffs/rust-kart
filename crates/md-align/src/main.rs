//! Aligns Markdown tables read from stdin, writing the result to stdout.
//!
//! Contiguous runs of lines starting with `|` are treated as tables.
//! All other lines pass through unchanged.

use std::io::{self, Read, Write};

fn is_table_row(line: &str) -> bool {
    line.trim_start().starts_with('|')
}

/// Splits a `| a | b | c |` row into `["a", "b", "c"]`.
fn parse_cells(line: &str) -> Vec<&str> {
    let trimmed = line.trim();
    let inner = trimmed
        .strip_prefix('|')
        .unwrap_or(trimmed)
        .strip_suffix('|')
        .unwrap_or(trimmed);
    inner.split('|').map(str::trim).collect()
}

/// A separator cell contains only `-` and optional leading/trailing `:`.
fn is_separator_cell(cell: &str) -> bool {
    let s = cell.trim();
    !s.is_empty() && s.chars().all(|c| c == '-' || c == ':')
}

/// Rebuilds a separator cell (`---`, `:---`, `---:`, `:---:`) at the target width.
fn format_separator(cell: &str, width: usize) -> String {
    let left = cell.starts_with(':');
    let right = cell.ends_with(':');
    let dashes = width - usize::from(left) - usize::from(right);
    let mut buf = String::with_capacity(width);
    if left {
        buf.push(':');
    }
    buf.extend(std::iter::repeat_n('-', dashes));
    if right {
        buf.push(':');
    }
    buf
}

/// Aligns a block of contiguous table rows.
fn align_table(lines: &[&str]) -> Vec<String> {
    let rows: Vec<Vec<&str>> = lines.iter().map(|l| parse_cells(l)).collect();
    let num_cols = rows.iter().map(Vec::len).max().unwrap_or(0);

    // Column widths: content width drives the minimum; separators need at least 3.
    let mut widths = vec![0_usize; num_cols];
    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            let min = if is_separator_cell(cell) {
                3
            } else {
                cell.len()
            };
            widths[i] = widths[i].max(min);
        }
    }

    rows.iter()
        .map(|row| {
            let cells: Vec<String> = (0..num_cols)
                .map(|i| {
                    let cell = row.get(i).copied().unwrap_or("");
                    let w = widths[i];
                    if is_separator_cell(cell) {
                        format_separator(cell, w)
                    } else {
                        format!("{cell:<w$}")
                    }
                })
                .collect();
            format!("| {} |", cells.join(" | "))
        })
        .collect()
}

fn process(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let mut result: Vec<String> = Vec::with_capacity(lines.len());
    let mut i = 0;

    while i < lines.len() {
        if is_table_row(lines[i]) {
            let start = i;
            while i < lines.len() && is_table_row(lines[i]) {
                i += 1;
            }
            result.extend(align_table(&lines[start..i]));
        } else {
            result.push(lines[i].to_string());
            i += 1;
        }
    }

    let mut output = result.join("\n");
    if input.ends_with('\n') {
        output.push('\n');
    }
    output
}

fn main() -> io::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    io::stdout().write_all(process(&input).as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_alignment() {
        let input = "\
| Name | Age | City |
|---|---|---|
| Alice | 30 | New York |
| Bob | 5 | LA |
";
        let expected = "\
| Name  | Age | City     |
| ----- | --- | -------- |
| Alice | 30  | New York |
| Bob   | 5   | LA       |
";
        assert_eq!(process(input), expected);
    }

    #[test]
    fn preserves_alignment_markers() {
        let input = "\
| Left | Center | Right |
|:---|:---:|---:|
| a | b | c |
";
        let expected = "\
| Left | Center | Right |
| :--- | :----: | ----: |
| a    | b      | c     |
";
        assert_eq!(process(input), expected);
    }

    #[test]
    fn non_table_lines_unchanged() {
        let input = "\
# Heading

Some text.

| A | B |
|---|---|
| 1 | 2 |

More text.
";
        let expected = "\
# Heading

Some text.

| A   | B   |
| --- | --- |
| 1   | 2   |

More text.
";
        assert_eq!(process(input), expected);
    }

    #[test]
    fn no_trailing_newline() {
        let input = "| x | y |\n|---|---|\n| 1 | 2 |";
        let result = process(input);
        assert!(!result.ends_with('\n'));
    }

    #[test]
    fn ragged_rows() {
        let input = "\
| A | B | C |
|---|---|---|
| 1 |
| 1 | 2 | 3 |
";
        let result = process(input);
        // Every row should have the same number of pipes.
        let pipe_counts: Vec<usize> = result.lines().map(|l| l.matches('|').count()).collect();
        assert!(pipe_counts.windows(2).all(|w| w[0] == w[1]));
    }
}

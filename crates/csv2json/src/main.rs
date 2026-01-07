use std::{io, process::exit};

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

/// Returns a list of (key, value) pairs for each row of the CSV supplied to
/// standard input.  Keys are taken from the CSV header (i.e., the first input
/// line), and values from each subsequent line.  If fewer than two lines are
/// provided, the result is an empty list.
fn load_csv() -> Result<Vec<Vec<(String, String)>>> {
    let mut rows = Vec::new();
    let mut reader = csv::Reader::from_reader(io::stdin());
    let headers = reader.headers()?.clone();
    for record in reader.records() {
        rows.push(
            headers
                .iter()
                .map(String::from)
                .zip(record?.iter().map(String::from).collect::<Vec<_>>())
                .collect(),
        );
    }
    Ok(rows)
}

fn print_field((name, value): &(String, String)) -> Result<()> {
    println!(
        "    {}: {}",
        serde_json::to_string(name)?,
        serde_json::to_string(value)?,
    );
    Ok(())
}

fn print_object<'a>(row: impl IntoIterator<Item = &'a (String, String)>) -> Result<()> {
    let mut fields = row.into_iter();
    let Some(field) = fields.next() else {
        println!("{{}}");
        return Ok(());
    };
    println!("  {{");
    print_field(field)?;
    for field in fields {
        print!(",");
        print_field(field)?;
    }
    println!("  }}");
    Ok(())
}

fn main_imp() -> Result<()> {
    let mut rows = load_csv()?.into_iter();
    let Some(row) = rows.next() else {
        println!("[]");
        return Ok(());
    };
    println!("[");
    print_object(&row)?;
    for row in rows {
        print!(",");
        print_object(&row)?;
    }
    println!("]");
    Ok(())
}

fn main() {
    if let Err(err) = main_imp() {
        eprintln!("error: {err}");
        exit(1);
    }
}

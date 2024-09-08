use std::{error::Error, io, process};

fn main_imp() -> Result<String, Box<dyn Error>> {
    let input = sum_of_products::parse(io::stdin().lines())?;
    let sop = sum_of_products::compute(&input);
    Ok(sum_of_products::render(sop, &input))
}

fn main() {
    match main_imp() {
        Ok(output) => println!("{output}"),
        Err(err) => {
            eprintln!("error: {err}");
            process::exit(1);
        }
    }
}

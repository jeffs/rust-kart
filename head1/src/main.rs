use std::{
    env,
    error::Error,
    fs::File,
    io::{prelude::*, BufReader},
};

fn main() -> Result<(), Box<dyn Error>> {
    let line = &mut String::new();
    for file in env::args().skip(1) {
        let mut file = BufReader::new(File::open(file)?);
        line.clear();
        file.read_line(line)?;
        print!("{line}");
        if !line.ends_with('\n') {
            println!();
        }
    }
    Ok(())
}

//! Repeats a given word a given number of times.

fn main() {
    let mut parameters = arg5::Parser::with_name_from_args();

    let mut count = 0;
    parameters.declare_positional("count", &mut count);

    let mut word = String::new();
    parameters.declare_positional("word", &mut word);

    if let Err(err) = parameters.parse_args() {
        eprintln!("Error: {}", err.what);
        std::process::exit(1);
    }

    if count > 0 {
        for _ in 1..count {
            print!("{} ", word);
        }
        println!("{}", word);
    }
}

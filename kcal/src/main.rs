use std::process::exit;

use kcal::Args;

fn main() {
    let args = Args::from_env().unwrap_or_else(|err| {
        eprintln!("error: {err}");
        exit(2);
    });
    let output = args.main().unwrap_or_else(|err| {
        
    });
    if let Err(err) = args.main() {
        eprintln!("error: {err}");
        exit(1);
    }
}

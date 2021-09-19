//! # Usage
//! ```sh
//! pangram <letters> [words-file]
//! ```
//!
//! # To Do
//! * [ ] `[-m|--min-length=<N>]`
//! * [ ] `[-p|--omit-proper]`

mod main_error;

use main_error::MainError;

use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

const DEFAULT_MIN_LENGTH: usize = 4;
const DEFAULT_WORDS_FILE: &str = "/usr/share/dict/words";

fn to_charset<I: Iterator<Item = char>>(chars: I) -> HashSet<char> {
    chars.flat_map(|c| c.to_lowercase()).collect()
}

struct Args {
    min_length: usize,
    mandatory: HashSet<char>,
    available: HashSet<char>,
    words_file: String,
}

fn parse_args() -> Result<Args, arg5::ParseError> {
    let mut letters = String::new();
    let mut words_file = None;
    let mut parameters = arg5::Parser::new();
    parameters.declare_positional("letters", &mut letters);
    parameters.declare_positional("words-file", &mut words_file);
    parameters.parse(env::args())?;
    Ok(Args {
        min_length: DEFAULT_MIN_LENGTH,
        mandatory: to_charset(letters.chars().filter(|c| c.is_uppercase())),
        available: to_charset(letters.chars()),
        words_file: words_file.unwrap_or_else(|| DEFAULT_WORDS_FILE.to_string()),
    })
}

fn format_line(line: &str, args: &Args) -> Option<String> {
    if line.chars().count() >= args.min_length {
        let set = to_charset(line.chars());
        if set.is_superset(&args.mandatory) && set.is_subset(&args.available) {
            if set.is_superset(&args.available) {
                Some(format!("* {}", line)) // a pangram
            } else {
                Some(format!("  {}", line)) // not a pangram, but an anagram
            }
        } else {
            None // not an anagram
        }
    } else {
        None // too short
    }
}

fn format_lines(args: Args) -> Result<Vec<String>, MainError> {
    let mut lines = Vec::new();
    let file =
        File::open(&args.words_file).map_err(|err| format!("{}: {}", args.words_file, err))?;
    for word in BufReader::new(file).lines() {
        if let Some(line) = format_line(&word?, &args) {
            lines.push(line);
        }
    }
    lines.sort_by_key(|line| line.len());
    Ok(lines)
}

fn main_imp() -> Result<(), MainError> {
    for line in format_lines(parse_args()?)? {
        println!("{}", line);
    }
    Ok(())
}

fn main() {
    if let Err(err) = main_imp() {
        eprintln!("Error: {}", err.what);
        std::process::exit(1);
    }
}

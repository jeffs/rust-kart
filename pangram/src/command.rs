use std::collections::HashSet;
use std::path::PathBuf;

const DEFAULT_WORDS_FILE: &str = "/usr/share/dict/words";
const DEFAULT_MIN_LENGTH: u32 = 4;

fn parse_letters(letters: &str) -> Result<(char, HashSet<char>), &'static str> {
    let mandatory_letter = match letters.chars().nth(0) {
        Some(c) => c,
        None => return Err("expected letters, not empty string"),
    };
    let available_letters = letters.chars().collect();
    Ok((mandatory_letter, available_letters))
}

#[derive(Debug)]
pub struct Command {
    pub min_length: u32,
    pub mandatory_letter: char,
    pub available_letters: HashSet<char>,
    pub words_file: PathBuf,
}

impl Command {
    pub fn from_args<I: Iterator<Item = String>>(mut args: I) -> Result<Command, String> {
        let mut min_length = DEFAULT_MIN_LENGTH;
        let mut letters = None;
        let mut words_file = None;
        while let Some(arg) = args.next() {
            if arg.starts_with("--min-length=") {
                let value = &arg["--min-length=".len()..];
                if let Ok(value) = value.parse() {
                    min_length = value;
                } else {
                    return Err(format!("bad value for --min-length: {}", value));
                }
            } else if letters.is_none() {
                letters = Some(parse_letters(&arg)?);
            } else if words_file.is_none() {
                words_file = Some(arg);
            }
        }
        let (mandatory_letter, available_letters) = match letters {
            Some(pair) => pair,
            None => return Err("expected letters".to_string()),
        };
        Ok(Command {
            min_length,
            mandatory_letter,
            available_letters,
            words_file: PathBuf::from(words_file.unwrap_or(DEFAULT_WORDS_FILE.to_string())),
        })
    }
}

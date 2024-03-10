use std::{env, error, fmt::Display, process};

#[derive(Debug)]
struct BadCodepoint(String);

impl BadCodepoint {
    pub fn from_arg(arg: &str) -> BadCodepoint {
        BadCodepoint(format!("{arg}: bad codepoint"))
    }
}

impl Display for BadCodepoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl error::Error for BadCodepoint {}

fn parse_codepoint(arg: &str) -> Result<char, BadCodepoint> {
    let Ok(codepoint) = u32::from_str_radix(&arg, 16) else {
        return Err(BadCodepoint::from_arg(arg));
    };
    Ok(char::from_u32(codepoint).ok_or_else(|| BadCodepoint::from_arg(arg))?)
}

fn parse_arg(arg: &str) -> Result<char, BadCodepoint> {
    match arg {
        s if s.starts_with("poo") => Ok('\u{1f4a9}'), // 💩
        s if s.starts_with("cent") => Ok('\u{a2}'),   // ¢
        "horns" => Ok('\u{1f918}'),                   // 🤘
        "grimace" | "grim" => Ok('\u{1f62c}'),        // 😬
        _ => parse_codepoint(arg),
    }
}

fn main() {
    let args: Result<Vec<char>, _> = env::args().skip(1).map(|arg| parse_arg(&arg)).collect();
    match args {
        Ok(chars) if chars.is_empty() => {
            eprintln!("utf: error: expected codepoints");
            process::exit(2);
        }
        Ok(chars) => {
            for c in chars {
                print!("{c}");
            }
            println!();
        }
        Err(err) => {
            eprintln!("utf: error: {err}");
            process::exit(2);
        }
    }
}

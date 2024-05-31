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

#[rustfmt::skip]
fn parse_arg(arg: &str) -> Result<char, BadCodepoint> {
    Ok(match arg {
        "cent" | "cents"                    => '¢',
        "command" | "cmd"                   => '⌘',
        "facepalm"                          => '🤦',
        "grimace" | "grim"                  => '😬',
        "horns"                             => '🤘',
        "lol"                               => '😂',
        "ok"                                => '👌',
        "shift"                             => '⇧',
        "sob"                               => '😭',
        "up"                                => '↑',
        s if s.starts_with("poo")           => '💩',
        _ => parse_codepoint(arg)?,
    })
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

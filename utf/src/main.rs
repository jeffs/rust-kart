use std::{env, error, fmt::Display, process};

#[derive(Debug)]
struct BadSymbol(String);

impl BadSymbol {
    pub fn from_arg(arg: &str) -> BadSymbol {
        BadSymbol(format!("{arg}: bad codepoint"))
    }
}

impl Display for BadSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl error::Error for BadSymbol {}

fn parse_codepoint(arg: &str) -> Result<char, BadSymbol> {
    let Ok(codepoint) = u32::from_str_radix(&arg, 16) else {
        return Err(BadSymbol::from_arg(arg));
    };
    Ok(char::from_u32(codepoint).ok_or_else(|| BadSymbol::from_arg(arg))?)
}

// TODO: Accept a buffer, rather than allocating a String.
#[rustfmt::skip]
fn parse_arg(arg: &str) -> Result<String, BadSymbol> {
    Ok(match arg {
        "cent" | "cents"                    => "¢".to_string(),
        "command" | "cmd"                   => "⌘".to_string(),
        "facepalm"                          => "🤦".to_string(),
        "grimace" | "grim"                  => "😬".to_string(),
        "horns"                             => "🤘".to_string(),
        "info"                              => "ℹ️".to_string(), // U+2139 U+FE0F
        "lol"                               => "😂".to_string(),
        "not"                               => "🚫".to_string(), // U+1F6AB Prohibited
        "ok"                                => "👌".to_string(),
        "shift"                             => "⇧".to_string(),
        "sob"                               => "😭".to_string(),
        "up"                                => "↑".to_string(),
        s if s.starts_with("poo")           => "💩".to_string(),
        _ => parse_codepoint(arg)?.to_string(),
    })
}

fn main() {
    let args: Result<Vec<String>, _> = env::args().skip(1).map(|arg| parse_arg(&arg)).collect();
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

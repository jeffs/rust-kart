#[macro_use]
extern crate clap;

use clap::App;
use std::io::{self, BufRead};
use std::iter;

type StringResult = io::Result<String>;

#[derive(Debug)]
struct Args {
    one: bool,
    r: bool,
    s: bool,
    files: Vec<String>,
}

fn new_app() -> App<'static> {
    App::new("Len")
        .version(crate_version!())
        .help_about("Print help information")
        .version_about("Print version information")
        .arg("-1 'Print only the first line of output'")
        .arg("-r 'Sort lines by decreasing length'")
        .arg("-s 'Sort lines by length'")
        .arg("[file]... 'Files to parse instead of stdin'")
}

fn to_vec(values: Option<clap::Values>) -> Vec<String> {
    values.map_or(vec![], |values| {
        values.map(|s| s.to_string()).collect::<Vec<_>>()
    })
}

fn parse_args() -> Args {
    let matches = new_app().get_matches();
    Args {
        one: matches.is_present("1"),
        r: matches.is_present("r"),
        s: matches.is_present("s"),
        files: to_vec(matches.values_of("file")),
    }
}

fn max_by_len<I: Iterator<Item = StringResult>>(lines: I) -> Option<StringResult> {
    let mut max: Option<(usize, String)> = None;
    for res in lines {
        match res {
            Ok(line) => {
                let n = line.chars().count();
                if max.as_ref().map_or(true, |(m, _)| m < &n) {
                    max = Some((n, line));
                }
            }
            _ => {
                return Some(res);
            }
        }

    }
    max.map(|(_, line)| Ok(line))
}

fn apply<'a, I: 'a + Iterator<Item = StringResult>>(
    args: Args,
    lines: I,
) -> Box<dyn Iterator<Item = StringResult> + 'a> {
    if args.one {
        if args.r {
            match max_by_len(lines) {
                Some(res) => Box::new(iter::once(res)),
                _ => Box::new(iter::empty()),
            }
        } else if args.s {
            panic!("TODO")
        } else {
            Box::new(lines.take(1))
        }
    } else {
        Box::new(lines)
    }
}

fn print<I: Iterator<Item = StringResult>>(lines: I) {
    for res in lines {
        match res {
            Ok(line) => {
                println!("{}:{}", line.chars().count(), line);
            }
            Err(err) if err.kind() == io::ErrorKind::InvalidData => {
                eprintln!("warning: {}", err);
            }
            Err(err) => {
                eprintln!("error: {}", err);
            }
        }
    }
}

fn main() {
    let args = parse_args();
    if args.files.is_empty() {
        let stdin = io::stdin();
        print(apply(args, stdin.lock().lines()));
    } else {
        panic!("file parsing is not yet implemented");
    }
}

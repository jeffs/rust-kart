#[macro_use]
extern crate clap;

use clap::{App, Arg};

#[derive(Debug)]
struct Args {
    one: bool,
    r: bool,
    s: bool,
    files: Vec<String>,
}

fn flag<'help>(name: &'help str, about: &'help str) -> Arg<'help> {
    debug_assert_eq!(name.len(), 1);
    Arg::new(name)
        .about(about)
        .short(name.chars().next().unwrap())
}

fn parse_args() -> Args {
    let matches = App::new("Len")
        .version(crate_version!())
        .help_about("Print help information")
        .version_about("Print version information")
        .arg(flag("1", "Print only the first line of output"))
        .arg(flag("r", "Sort lines by decreasing length"))
        .arg(flag("s", "Sort lines by length"))
        .arg(
            Arg::new("file")
                .about("Files to parse instead of stdin")
                .multiple(true),
        )
        .get_matches();

    Args {
        one: matches.is_present("1"),
        r: matches.is_present("r"),
        s: matches.is_present("s"),
        files: matches.values_of("file").map_or(vec![], |values| {
            values.map(|s| s.to_string()).collect::<Vec<_>>()
        }),
    }
}

fn main() {
    let args = parse_args();
    println!("{:#?}", args);
}

#[macro_use]
extern crate clap;

use clap::App;

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

fn main() {
    let args = parse_args();
    println!("{:#?}", args);
}

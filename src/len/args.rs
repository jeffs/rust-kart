use std::path::PathBuf;

/// Specifies len's command line interface to the clap argument parsing library.
fn new_app() -> clap::App<'static> {
    clap::App::new("Len")
        .version(crate_version!())
        .help_about("Print help information")
        .version_about("Print version information")
        .arg("-1 'Print only the first line of output'")
        .arg("-r 'Sort lines by decreasing length'")
        .arg("-s 'Sort lines by length'")
        .arg("[file]... 'Files to parse instead of stdin'")
        .arg(
            clap::Arg::new("color")
                .about("Color diagnostics")
                .default_value("auto")
                .long("color")
                .possible_values(&["always", "auto", "never"])
                .takes_value(true)
                .value_name("when"),
        )
}

/// Converts clap's representation of a parsed command line  to an Args object.
fn args_from_clap(matches: clap::ArgMatches) -> Args {
    Args {
        one: matches.is_present("1"),
        r: matches.is_present("r"),
        s: matches.is_present("s"),
        files: matches.values_of_os("file").map_or(vec![], |values| {
            values.map(|s| PathBuf::from(s)).collect::<Vec<_>>()
        }),
        // Unwrapping is safe because the color argument has a default value.
        color: matches.value_of("color").unwrap().to_string(),
    }
}

/// Represents a set of command line arguments.
#[derive(Debug)]
pub struct Args {
    pub one: bool,
    pub r: bool,
    pub s: bool,
    pub files: Vec<PathBuf>,
    pub color: String,
}

impl Args {
    /// Parses env::args_os and returns a representation of the command line arguments.
    pub fn from_env() -> Args {
        args_from_clap(new_app().get_matches())
    }
}

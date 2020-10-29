use std::path::PathBuf;

fn new_app() -> clap::App<'static> {
    clap::App::new("Len")
        .version(crate_version!())
        .help_about("Print help information")
        .version_about("Print version information")
        .arg("-1 'Print only the first line of output'")
        .arg("-r 'Sort lines by decreasing length'")
        .arg("-s 'Sort lines by length'")
        .arg("[file]... 'Files to parse instead of stdin'")
}

#[derive(Debug)]
struct Args {
    pub one: bool,
    pub r: bool,
    pub s: bool,
    pub files: Vec<PathBuf>,
}

impl Args {
    fn from_clap(matches: clap::ArgMatches) -> Args {
        Args {
            one: matches.is_present("1"),
            r: matches.is_present("r"),
            s: matches.is_present("s"),
            files: matches.values_of_os("file").map_or(vec![], |values| {
                values.map(|s| PathBuf::from(s)).collect::<Vec<_>>()
            }),
        }
    }

    fn from_env() -> Args {
        Args::from_clap(new_app().get_matches())
    }
}

// The three-letter ops require O(1) RAM, whereas the sorts require O(N).
pub enum Op {
    All,         // Print all lines
    Max,         // Print the longest line
    Min,         // Print the shortest line
    One,         // Print the first line
    ReverseSort, // Print all lines, sorted by decreasing length
    Sort,        // Print all lines, sorted by length
}

pub struct Command {
    pub op: Op,
    pub files: Vec<PathBuf>,
}

impl Command {
    fn from_args(args: Args) -> Command {
        const F: bool = false;
        const T: bool = true;
        Command {
            op: match (args.one, args.r, args.s) {
                (F, F, F) => Op::All,
                (F, F, T) => Op::Sort,
                (F, T, _) => Op::ReverseSort,
                (T, F, F) => Op::One,
                (T, F, T) => Op::Min,
                (T, T, _) => Op::Max,
            },
            files: args.files,
        }
    }

    pub fn from_env() -> Command {
        Command::from_args(Args::from_env())
    }
}

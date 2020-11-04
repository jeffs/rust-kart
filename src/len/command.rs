use atty;
use std::path::PathBuf;
use super::op::Op;
use super::args::Args;
use termcolor::ColorChoice;

fn color_choice(when: &str) -> ColorChoice {
    match when {
        "always" => ColorChoice::Always,
        "auto" if atty::is(atty::Stream::Stderr) => ColorChoice::Auto,
        _ => ColorChoice::Never,
    }
}

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
        color: color_choice(&args.color),
    }
}

pub struct Command {
    pub op: Op,
    pub files: Vec<PathBuf>,
    pub color: ColorChoice,
}

impl Command {
    pub fn from_env() -> Command {
        from_args(Args::from_env())
    }
}

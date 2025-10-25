//! Provides command-line access to the [`since`] and [`update`] functions.

use std::{env, ffi, process::exit};

use grit::command;

trait ToStr {
    fn to_str(&self) -> Option<&str>;
}

impl ToStr for Option<ffi::OsString> {
    fn to_str(&self) -> Option<&str> {
        self.as_deref().and_then(ffi::OsStr::to_str)
    }
}

#[tokio::main]
async fn main() {
    let mut args = env::args_os();
    args.next(); // Skip program name.
    let arg = args.next();
    let is_verbose = matches!(arg.to_str(), Some("-v" | "--verbose"));
    let command = if is_verbose { args.next() } else { arg };
    let result = match command.to_str() {
        Some("si" | "since") if is_verbose => command::since::long(args).await,
        Some("si" | "since") => command::since::short(args).await,
        Some("tr" | "trunk") => command::trunk::trunk(args).await,
        Some("up" | "update") => command::update::update(args).await,
        _ => {
            let usage = "Usage:\
                \n    grit [-v|--verbose] {si|since} [GIT_FLAGS...] [BASE]\
                \n    grit {ar|archive} BRANCH\
                \n    grit {tr|trunk}\
                \n    grit {up|update}";
            eprintln!("{usage}");
            exit(2);
        }
    };
    if let Err(err) = result {
        eprintln!("Error: {err}");
        exit(1);
    }
}

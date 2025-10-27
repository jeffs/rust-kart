//! Provides command-line access to the [`since`] and [`update`] functions.

use std::{env, ffi, fmt, process::exit};

use grit::command;

const USAGE: &str = "
    grit [-v|--verbose] {si|since} [GIT_FLAGS...] [BASE]
    grit {ar|archive} BRANCH
    grit {tr|trunk}
    grit {up|update}";

trait ToStr {
    fn to_str(&self) -> Option<&str>;
}

impl ToStr for Option<ffi::OsString> {
    fn to_str(&self) -> Option<&str> {
        self.as_deref().and_then(ffi::OsStr::to_str)
    }
}

fn die(prefix: &str, message: impl fmt::Display, status: i32) -> ! {
    eprintln!("{prefix}{message}");
    exit(status)
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
        _ => die("Usage:", USAGE, 2),
    };
    if let Err(err) = result {
        die("Error: ", err, 1);
    }
}

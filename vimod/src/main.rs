//! Opens any modified files from the current Git working copy in ${EDITOR:-vi}.

use std::env;
use std::ffi::OsStr;
use std::os::unix::process::CommandExt;
use std::process::{Command, exit};

fn main() {
    let output = Command::new("git").arg("status").output().unwrap();
    let status = std::str::from_utf8(&output.stdout).unwrap();

    let files = status
        .lines()
        .filter_map(|line| line.strip_prefix("\tmodified:").map(str::trim_ascii))
        .collect::<Vec<_>>();

    if files.is_empty() {
        eprintln!("no modified files");
        return;
    }

    let editor = env::var_os("EDITOR").unwrap_or_else(|| OsStr::new("vi").to_owned());
    let error = Command::new(&editor).args(files).exec();
    eprintln!("error: {error}");
    exit(1);
}

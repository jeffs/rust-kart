//! Opens any modified files from the current Git working copy in ${EDITOR:-vi}.

use std::ffi::OsStr;
use std::io::Write;
use std::os::unix::process::CommandExt;
use std::process::{Command, exit};
use std::{env, io};

fn main() {
    let output = Command::new("git").arg("status").output().unwrap();

    if !output.status.success() {
        io::stderr().write_all(&output.stderr).unwrap();
        exit(output.status.code().unwrap_or(1));
    }

    let files = std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .filter_map(|line| {
            line.strip_prefix("\tmodified:")
                .or_else(|| line.strip_prefix("\tboth modified:"))
                .map(str::trim_ascii)
        })
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

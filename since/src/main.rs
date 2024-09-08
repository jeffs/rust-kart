use std::{
    env,
    os::unix::process::CommandExt,
    process::{exit, Command},
};

fn main() {
    let mut base: Option<&str> = None;
    let mut flags: Vec<&str> = Vec::new();
    let our_args: Vec<String> = env::args().skip(1).collect();
    for arg in &our_args {
        if arg.starts_with('-') {
            flags.push(arg);
        } else if base.is_none() {
            base = Some(arg);
        } else {
            eprintln!("{arg}: unexpected argument");
            exit(2);
        }
    }

    Command::new("git")
        .args(["log", "--first-parent", "--oneline"])
        .args(flags)
        .arg(format!("{}..", base.unwrap_or("main")))
        .exec();
}

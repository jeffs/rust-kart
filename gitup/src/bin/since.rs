use std::{
    env, fmt,
    os::unix::process::CommandExt,
    process::{Command, exit},
};

fn die(code: i32, err: impl fmt::Display) -> ! {
    eprintln!("error: {err}");
    exit(code)
}

#[tokio::main]
async fn main() {
    let mut base: Option<&str> = None;
    let mut flags: Vec<&str> = Vec::new();
    let our_args: Vec<String> = env::args().skip(1).collect();
    for arg in &our_args {
        if arg.starts_with('-') {
            flags.push(arg);
        } else if base.is_none() {
            base = Some(arg);
        } else {
            die(2, "unexpected argument");
        }
    }

    let range = match base {
        Some(some) => format!("{some}.."),
        None => match gitup::local_trunk().await {
            Ok(ok) => format!("{ok}.."),
            Err(err) => die(3, err),
        },
    };

    let err = Command::new("git")
        .args(["log", "--first-parent", "--oneline"])
        .args(flags)
        .arg(range)
        .exec();

    // If `exec` returned, something has gone wrong.
    die(1, err);
}

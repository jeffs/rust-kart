//! Sends standard input to the bottom-right tmux, or next zellij, pane.

use std::env;
use std::io::{self, Read};
use std::os::unix::process::ExitStatusExt;
use std::process::{self, Command, ExitStatus};

fn read_stdin() -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    io::stdin().read_to_end(&mut bytes).map(|_| bytes)
}

fn run_tmux(text: &str) -> io::Result<ExitStatus> {
    let args = ["send-keys", "-lt", "bottom-right", "--", text];
    Command::new("tmux").args(args).status()
}

fn zellij_action<'a>(args: impl IntoIterator<Item = &'a str>) -> io::Result<ExitStatus> {
    Command::new("zellij").arg("action").args(args).status()
}

fn run_zellij(text: &str) -> io::Result<ExitStatus> {
    zellij_action(["focus-next-pane"])?;
    zellij_action(["write-chars", text])?;
    zellij_action(["focus-previous-pane"])
}

fn run() -> Result<ExitStatus, Box<dyn std::error::Error>> {
    let text = String::from_utf8(read_stdin()?)?;
    Ok(if env::var_os("ZELLIJ").is_some() {
        run_zellij(&text)?
    } else {
        run_tmux(&text)?
    })
}

fn main() {
    let status = run().unwrap();

    #[cfg(unix)]
    if let Some(signal) = status.signal() {
        process::exit(128 + signal);
    }

    process::exit(status.code().unwrap());
}

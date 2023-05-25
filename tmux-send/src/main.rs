//! Sends standard input to the bottom-right tmux pane.
use std::io::{self, Read};
use std::os::unix::process::ExitStatusExt;
use std::process::{self, Command, ExitStatus};

fn read_stdin() -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    io::stdin().read_to_end(&mut bytes).map(|_| bytes)
}

fn run() -> Result<ExitStatus, Box<dyn std::error::Error>> {
    let text = String::from_utf8(read_stdin()?)?;
    let args = ["send-keys", "-lt", "bottom-right", "--", &text];
    Ok(Command::new("tmux").args(args).status()?)
}

fn main() {
    let status = run().unwrap();

    #[cfg(unix)]
    if let Some(signal) = status.signal() {
        process::exit(128 + signal);
    }

    process::exit(status.code().unwrap());
}

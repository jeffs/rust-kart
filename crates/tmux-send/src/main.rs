//! Copies stdin to the bottom right tmux pane, or the "next" wezterm or zellij
//! pane.

use std::env;
use std::io::{self, Read};
use std::os::unix::process::ExitStatusExt;
use std::process::{self, Command, ExitStatus};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn slurp_stdin() -> Result<String> {
    let mut bytes = Vec::new();
    let bytes = io::stdin().read_to_end(&mut bytes).map(|_| bytes)?;
    Ok(String::from_utf8(bytes)?)
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

fn run_wez() -> Result<ExitStatus> {
    let output = Command::new("wezterm")
        .args(["cli", "get-pane-direction", "Next"])
        .output()?;
    let pane_id = std::str::from_utf8(&output.stdout)?.trim_ascii_end();

    // Bail unless the pane ID is a nonnegative integer.
    pane_id.parse::<u32>()?;

    Ok(Command::new("wezterm")
        .args(["cli", "send-text", "--no-paste", "--pane-id", pane_id])
        .status()?)
}

fn run() -> Result<ExitStatus> {
    Ok(if env::var_os("ZELLIJ").is_some() {
        run_zellij(&slurp_stdin()?)?
    } else if env::var("TERM_PROGRAM").as_deref() == Ok("WezTerm") {
        run_wez()?
    } else {
        run_tmux(&slurp_stdin()?)?
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

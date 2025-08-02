use std::{ffi::OsStr, fmt, process::ExitStatus};

use tokio::process::Command;

/// Git returned bad status, and printed the supplied text to standard error.
#[derive(Debug)]
pub struct Error(String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

fn format_git_command<S, I>(args: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let args: Vec<_> = args.into_iter().collect();
    let strs: Vec<_> = args.iter().map(|s| s.as_ref().to_string_lossy()).collect();
    format!("git {}", strs.join(" "))
}

/// Returns (success, stdout, stderr).
async fn run_git<S, I>(args: I) -> (ExitStatus, String, String)
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new("git")
        .args(args)
        .output()
        .await
        .expect("git should be executable");
    (
        output.status,
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

/// Runs Git, printing the command and its stdout. Primarily useful for
/// debugging.
///
/// # Errors
///
/// Returns [`Error::Git`] if the `git` command fails.
pub async fn git_loud<S, I>(args: I) -> Result<String, Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let args: Vec<_> = args.into_iter().collect();
    println!("> {}", format_git_command(&args));
    let (status, stdout, stderr) = run_git(args).await;
    if !status.success() {
        return Err(Error(stderr));
    }
    stderr.lines().for_each(|s| println!("! {s}"));
    stdout.lines().for_each(|s| println!("< {s}"));
    Ok(stderr + &stdout)
}

/// # Errors
///
/// Returns [`Error::Git`] if the `git` command fails.
pub async fn git<S, I>(args: I) -> Result<String, Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let (status, stdout, stderr) = run_git(args).await;
    if !status.success() {
        return Err(Error(stderr));
    }
    Ok(stderr + &stdout)
}

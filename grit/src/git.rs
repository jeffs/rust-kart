use std::{ffi::OsStr, fmt, process::ExitStatus};

use tokio::process::Command;

/// Git returned bad status, and printed the supplied text to standard error.
#[derive(Debug)]
pub struct Error(String);

pub type Result<T> = std::result::Result<T, Error>;

pub const HEAD: &str = "HEAD";

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // If the Git error already ends with a newline, remove it.
        self.0.strip_suffix('\n').unwrap_or(&self.0).fmt(f)
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
///
/// TODO: Should this return [`std::ffi::OsString`] or [`Vec<u8>`] rather than
///  [`String`]? What is the narrowest type of plausible `git(1)` output?
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
pub async fn git_loud<S, I>(args: I) -> Result<String>
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
pub async fn git<S, I>(args: I) -> Result<String>
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

/// # Errors
///
/// Returns [`Error::Git`] if `git merge-base` fails.
pub async fn merge_base(ref1: impl AsRef<OsStr>, ref2: impl AsRef<OsStr>) -> Result<String> {
    let mut base = git(["merge-base".as_ref(), ref1.as_ref(), ref2.as_ref()]).await?;
    base.truncate(base.trim_end().len());
    Ok(base)
}

/// # Errors
///
/// Returns [`Error::Git`] if `git merge-base` fails.
pub async fn merge_base_head(base: impl AsRef<OsStr>) -> Result<String> {
    merge_base(base, HEAD).await
}

/// Returns the qualified name of the remote branch being tracked by the
/// specified tracking branch, or [`None`] if the `branch` is not a tracking
/// branch.
///
/// # Examples
///
/// Assuming the local `main` branch is tracking `origin/main`:
///
/// ```no_run
/// assert_eq!(upstream("main"), Some("origin/main"));
/// ```
pub async fn upstream(branch: impl AsRef<OsStr>) -> Option<String> {
    git([
        "rev-parse",
        "--abbrev-ref",
        "--symbolic-full-name",
        &format!("{}@{{u}}", branch.as_ref().display()),
    ])
    .await
    .ok()
    .map(|s| s.trim().to_owned())
}

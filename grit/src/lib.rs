use std::{ffi::OsStr, fmt, process::ExitStatus};

use tokio::process::Command;

/// Default names to consider when searching for local trunk branch, in order of preference.
/// Overridden by the value of the [`GRIT_TRUNKS`] environment variable.
pub const DEFAULT_TRUNKS: [&str; 2] = ["main", "master"];

/// Environment variable to check for comma-separated list of local trunk branch names.  If the
/// variable is unset, the value defaults to [`DEFAULT_TRUNKS`].
pub const GRIT_TRUNKS: &str = "GRIT_TRUNKS";

#[derive(Debug)]
pub enum Error {
    /// Git returned bad status, and printed the supplied text to standard error.
    Git(String),

    /// No local branch was found to have any of the supplied names.
    Trunk(Vec<String>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Git(stderr) => stderr.fmt(f),
            Error::Trunk(names) => write!(f, "trunk not found: {}", names.join(", ")),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

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

#[allow(dead_code)]
async fn git_loud<S, I>(args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let args: Vec<_> = args.into_iter().collect();
    print!("{}", format_git_command(&args));

    let (status, stdout, stderr) = run_git(args).await;
    if !status.success() {
        return Err(Error::Git(stderr));
    }

    let lines: Vec<_> = stdout.lines().collect();
    match lines.len() {
        0 => println!(),
        1 => println!(": {}", lines[0].trim()),
        _ => {
            println!(":");
            for line in lines {
                println!("  {line}");
            }
        }
    }

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
        return Err(Error::Git(stderr));
    }
    Ok(stderr + &stdout)
}

/// Returns the names of potential trunk branches, per [`GRIT_TRUNKS`] (if set)
/// or [`DEFAULT_TRUNKS`].
pub fn trunk_names() -> Vec<String> {
    let trunks = std::env::var(GRIT_TRUNKS);
    match trunks.as_ref() {
        Ok(trunks) => trunks.split(',').map(str::to_owned).collect(),
        Err(_) => DEFAULT_TRUNKS.map(str::to_owned).to_vec(),
    }
}

/// Returns the name of the local trunk branch, if it can be determined.
///
/// # Errors
///
/// Will return [`Error::Trunk`] if no local trunk branch is found.
pub async fn local_trunk() -> Result<String> {
    let names = trunk_names();
    for name in &names {
        if git(["show-ref", name]).await.is_ok() {
            return Ok(name.to_owned());
        }
    }
    Err(Error::Trunk(names))
}

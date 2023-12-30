//! This program pulls the main branch of the repo from which it's called, then
//! deletes any local branches that are not ahead of main, and finally checks
//! back out the original branch.  The main branch defaults to `main`, but
//! `master` is used as a fallback if no `main` branch is found.

// TODO: Delete remote branches behind trunk.

// TODO: Don't mess up "checkout -" by checking out main.  I tried _not_
// checking out main, but after fetch --prune, the user still sees a list of
// obsolete branches the next time they pull --prune; so now this program checks
// out main just so it can run pull --prune per se.

use std::error::Error;
use std::ffi::OsStr;
use std::process::{exit, ExitStatus};
use std::{env, fmt};
use tokio::process::Command;

#[derive(Debug)]
struct SimpleError(String);

impl fmt::Display for SimpleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<S> From<S> for SimpleError
where
    S: Into<String>,
{
    fn from(value: S) -> Self {
        SimpleError(value.into())
    }
}

impl Error for SimpleError {}

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

async fn git<S, I>(args: I) -> Result<String, SimpleError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let args: Vec<_> = args.into_iter().collect();
    print!("{}", format_git_command(&args));

    let (status, stdout, stderr) = run_git(args).await;
    if !status.success() {
        return Err(stderr.into());
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

    Ok((stderr + &stdout).into())
}

async fn git_quiet<S, I>(args: I) -> Result<String, SimpleError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let (status, stdout, stderr) = run_git(args).await;
    if !status.success() {
        return Err(stderr.into());
    }
    Ok((stderr + &stdout).into())
}

/// Returns the local main branch (trunk) name, or an error.
async fn local_trunk() -> Result<&'static str, String> {
    let names = ["main", "master"];
    for branch in names {
        if git(["show-ref", branch]).await.is_ok() {
            return Ok(branch);
        }
    }
    let names = names.join(" or ");
    Err(format!("expected trunk; can't find {names}"))
}

async fn upstream(branch: &str) -> Option<String> {
    git([
        "rev-parse",
        "--abbrev-ref",
        "--symbolic-full-name",
        &format!("{branch}@{{u}}"),
    ])
    .await
    .ok()
    .map(|s| s.trim().to_owned())
}

async fn main_imp() -> Result<(), SimpleError> {
    let orig = git(["rev-parse", "--abbrev-ref", "HEAD"]).await?;
    let orig = orig.as_str().trim();

    let trunk = local_trunk().await?;
    if trunk != orig {
        git(["checkout", trunk]).await?;
    }

    if upstream(trunk).await.is_some() {
        if let Err(err) = git(["pull", "--prune"]).await {
            eprintln!("warning: can't pull {trunk}: {err}");
        }
    }

    let branches = git_quiet(["branch"]).await?;
    let branches = branches
        .lines()
        .filter(|line| !line.starts_with("* "))
        .map(|line| &line[2..]); // Remove leading ' '.

    let mut dead_branches = Vec::new();
    for branch in branches {
        let range = format!("{trunk}..{branch}");
        if git(["rev-list", "--count", &range]).await? == "0\n" {
            dead_branches.push(branch);
        }
    }

    if trunk != orig && !dead_branches.contains(&orig) {
        git(["checkout", orig]).await?;
    }

    if !dead_branches.is_empty() {
        git(["branch", "-D"].into_iter().chain(dead_branches)).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if env::args().len() > 1 {
        println!("git-prune: error: expected empty argument list");
        exit(2);
    }
    if let Err(err) = main_imp().await {
        println!("git-prune: {err}");
        exit(1);
    }
}

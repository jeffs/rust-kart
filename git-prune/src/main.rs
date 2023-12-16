//! This program pulls the main branch of the repo from which it's called, then
//! deletes any local branches that are not ahead of main, and finally checks
//! back out the original branch.  The main branch defaults to `main`, but
//! `master` is used as a fallback if no `main` branch is found.

use std::error::Error;
use std::ffi::OsStr;
use std::process::exit;
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

async fn git<S, I>(args: I) -> Result<String, SimpleError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    #[cfg(debug)]
    {
        let args: Vec<_> = args.into_iter().collect();
        eprintln!(
            "debug: {}",
            args.iter()
                .map(|arg| arg.as_ref().to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ")
        );
    }

    let output = Command::new("git")
        .args(args)
        .output()
        .await
        .expect("git should be executable");
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        return Err(stderr.into());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok((stderr + stdout).into())
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

async fn update_other_branch(branch: &str, upstream: &str) -> Result<(), SimpleError> {
    let Some((remote, _)) = upstream.split_once('/') else {
        return Err(format!("{upstream}: bad upstream; expected ORIGIN/BRANCH").into());
    };
    if let Err(err) = git(["fetch", "--prune", remote]).await {
        return Err(format!("can't fetch {remote}: {err}").into());
    }
    git(["branch", "-f", branch, &upstream]).await?;
    Ok(())
}

async fn try_pull(trunk: &str, head: &str) {
    let Some(remote) = upstream(trunk).await else {
        return;
    };

    if head == trunk {
        if let Err(err) = git(["pull"]).await {
            eprintln!("warning: can't pull {trunk}: {err}");
            return;
        }
    } else {
        if let Err(err) = update_other_branch(trunk, &remote).await {
            eprintln!("warning: can't move {trunk}: {err}");
            return;
        }
    }
    println!("mv: {trunk} {remote}");
}

async fn main_imp() -> Result<(), SimpleError> {
    // Identify local head and trunk.
    let orig = git(["rev-parse", "--abbrev-ref", "HEAD"]).await?;
    let orig = orig.as_str().trim();
    let trunk = local_trunk().await?;

    // Update trunk from remote, if possible.
    try_pull(trunk, orig).await;

    // List all branches except trunk.
    let branches = git(["branch"]).await?;
    let branches = branches
        .lines()
        .filter(|line| !line.ends_with(&format!(" {trunk}")))
        .map(|line| &line[2..]); // Remove leading '*' or ' '.

    // List branches that are not ahead of main.
    let mut dead_branches = Vec::new();
    for branch in branches {
        let range = format!("{trunk}..{branch}");
        if git(["rev-list", "--count", &range]).await? == "0\n" {
            dead_branches.push(branch);
        }
    }

    // Remember where we're leaving the local head, so we can print it later.
    let last = if dead_branches.contains(&orig) {
        // We can't delete the branch we're sitting on; so, sit elsewhere.
        git(["checkout", &trunk]).await?;
        trunk
    } else {
        orig
    };

    // Delete branches that are not ahead of main.
    if !dead_branches.is_empty() {
        // Git allows commas, but not spaces, in branch names.
        println!("rm: {}", dead_branches.join(" "));
        git(["branch", "-d"].into_iter().chain(dead_branches)).await?;
    }

    // Return to the original branch, unless it's gone.
    if orig != last {
        git(["checkout", &orig]).await?;
    }

    // Print the current branch name before exiting.
    println!("co: {last}");
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

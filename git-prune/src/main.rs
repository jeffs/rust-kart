//! This program pulls the main branch of the repo from which it's called, then
//! deletes any local branches that are not ahead of main, and finally checks
//! back out the original branch.  The main branch defaults to `main`, but
//! `master` is used as a fallback if no `main` branch is found.

use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::process::exit;
use tokio::process::Command;

async fn git<S, I>(args: I) -> Result<String, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
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
async fn trunk() -> Result<&'static str, String> {
    let names = ["main", "master"];
    for branch in names {
        if git(["show-ref", branch]).await.is_ok() {
            return Ok(branch);
        }
    }
    let names = names.join(" or ");
    Err(format!("expected trunk; can't find {names}"))
}

async fn main_imp() -> Result<(), Box<dyn Error>> {
    // Check out the main branch.
    git(["checkout", trunk().await?]).await?;
    let _ = git(["pull", "--prune"]).await;

    // List all branches except the one we just checked out.
    let branches = git(["branch"]).await?;
    let branches = branches
        .lines()
        .filter(|line| !line.starts_with('*'))
        .map(|line| line.trim());

    // List branches that are not ahead of main.
    let mut dead_branches = Vec::new();
    for branch in branches {
        let range = format!("..{branch}");
        if git(["rev-list", "--count", &range]).await? == "0\n" {
            dead_branches.push(branch);
        }
    }

    // Delete branches that are not ahead of main.
    if !dead_branches.is_empty() {
        let list = dead_branches.join(", ");
        git(["branch", "-d"].into_iter().chain(dead_branches)).await?;
        eprintln!("Deleted {list}");
    }

    // Return to the originally checked out branch, unless it's gone.
    let _ = git(["checkout", "-"]).await;
    Ok(())
}

#[tokio::main]
async fn main() {
    if env::args().len() > 1 {
        eprintln!("git-prune: error: expected empty argument list");
        exit(2);
    }
    if let Err(err) = main_imp().await {
        eprintln!("git-prune: {err}");
        exit(1);
    }
}

//! This program pulls the main branch of the repo from which it's called, then
//! deletes any local branches that are not ahead of main.  The main branch
//! defaults to `main`, but `master` is used as a fallback if no `main` branch
//! is found.

use std::env;
use std::error::Error;
use std::process::exit;
use tokio::process::Command;

async fn git<'a, I>(args: I) -> Result<String, String>
where
    I: IntoIterator<Item = &'a str>,
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
    let orig = git(["rev-parse", "--abbrev-ref", "head"]).await?;
    let orig = orig.trim();
    git(["checkout", trunk().await?]).await?;
    if let Err(err) = git(["pull", "--prune"]).await {
        eprintln!("warning: git pull --prune: {err}");
    }

    let branches = git(["branch"]).await?;
    let branches = branches
        .lines()
        .filter(|line| !line.starts_with('*'))
        .map(|line| line.trim());

    let mut ahead = Vec::new(); // branches that are ahead of main
    for branch in branches {
        let range = format!("..{branch}");
        if git(["rev-list", "--count", &range]).await? == "0\n" {
            ahead.push(branch);
        }
    }

    if !ahead.is_empty() {
        eprintln!("git branch -d {}", ahead.join(" "));
        git(["branch", "-d"].into_iter().chain(ahead)).await?;
    }

    git(["checkout", &orig]).await?;
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

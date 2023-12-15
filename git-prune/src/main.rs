//! This program pulls the main branch of the repo from which it's called, then
//! deletes any local branches that are not ahead of main, and finally checks
//! back out the original branch.  The main branch defaults to `main`, but
//! `master` is used as a fallback if no `main` branch is found.

use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::process::exit;
use tokio::process::Command;

const ORIGIN: &str = "origin";

async fn git<S, I>(args: I) -> Result<String, String>
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

async fn main_imp() -> Result<(), Box<dyn Error>> {
    // Identify local head and trunk.
    let orig = git(["rev-parse", "--abbrev-ref", "HEAD"]).await?;
    let orig = orig.as_str().trim();
    let trunk = local_trunk().await?;

    // Sync from origin, and update local trunk.
    if let Err(err) = git(["fetch", "--prune", ORIGIN]).await {
        eprintln!("warning: can't fetch {ORIGIN}: {err}")
    } else if orig == trunk {
        git(["pull"]).await?;
    } else {
        // Update local trunk to match origin.
        git(["branch", "-f", trunk, &format!("{ORIGIN}/{trunk}")]).await?;
    }

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

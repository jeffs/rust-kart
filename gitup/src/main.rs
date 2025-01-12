//! This program pulls the main branch of the repo from which it's called, then deletes any local
//! branches that are not ahead of main, and finally checks back out the original branch.  The main
//! branch defaults to `main`, but `master` is used as a fallback if no `main` branch is found.
//!
//! # TODO
//!
//! * [ ] Delete remote branches behind trunk.
//! * [ ] Don't mess up "checkout -" by checking out main.  I tried _not_ checking out main, but
//!   after fetch --prune, the user still sees a list of obsolete branches the next time they pull
//!   --prune; so now this program checks out main just so it can run pull --prune per se.
//! * [ ] Support squashed merges.

use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::Path;
use std::process::{exit, ExitStatus};
use std::{env, fmt};
use tokio::process::Command;

/// Possible names of local trunk branches, in order of preference.
const TRUNKS: [&str; 2] = ["main", "master"];

#[derive(Debug)]
enum Error {
    /// Git returned bad status, and printed the supplied text to standard error.
    Git(String),

    /// The Git working copy has uncommitted changes.
    Unclean,

    /// No local trunk branch could be identified.
    Trunk,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Git(stderr) => write!(f, "{stderr}"),
            Error::Unclean => write!(f, "working copy is unclean"),
            Error::Trunk => write!(f, "can't find local trunk; any of {TRUNKS:?}"),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

/// Options specifying removal of local branches meeting various criteria.
#[allow(dead_code)]
enum Removal {
    /// Specifies removal of local branches merged to local trunk.  Note that this does not include
    /// GitHub "squash merges," which do not actually merge the original branch.
    Merged,

    /// Specifies removal of local branches whose upstreams are gone.  Upstream branches are often
    /// deleted after being merged to trunk, even if they were "squash merged," so this is a useful
    /// way to detect such "merges."
    Gone,
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

async fn git<S, I>(args: I) -> Result<String>
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

/// Returns the local main branch (trunk) name, or an error.
async fn local_trunk() -> Result<&'static str> {
    for branch in TRUNKS {
        if git(["show-ref", branch]).await.is_ok() {
            return Ok(branch);
        }
    }
    Err(Error::Trunk)
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

async fn is_working_copy_clean() -> Result<bool> {
    Ok(git(["status", "--porcelain"]).await?.is_empty())
}

async fn main_imp() -> Result<()> {
    if !is_working_copy_clean().await? {
        return Err(Error::Unclean);
    }

    let orig = git(["rev-parse", "--abbrev-ref", "HEAD"]).await?;
    let orig = orig.as_str().trim();

    let trunk = local_trunk().await?;

    if trunk != orig {
        git(["checkout", trunk]).await?;
    }

    if upstream(trunk).await.is_some() {
        if let Err(err) = git(["pull", "--prune", "--quiet"]).await {
            eprintln!("warning: can't pull {trunk}: {err}");
        }
    }

    let dead_branches = git(["branch", "--merged"]).await?;
    let dead_branches = dead_branches
        .lines()
        .filter(|line| !line.starts_with("* "))
        .map(str::trim_ascii_start)
        .collect::<HashSet<_>>();

    // let mut dead_branches = Vec::new();
    // for branch in branches {
    //     let range = format!("{trunk}..{branch}");
    //     if git(["rev-list", "--count", &range]).await? == "0\n" {
    //         dead_branches.push(branch);
    //     }
    // }

    if dead_branches.contains(&orig) {
        // Let the user know we're not leaving HEAD on the original branch.
        println!("co {trunk}");
    } else if trunk != orig {
        git(["checkout", orig]).await?;
    };

    if !dead_branches.is_empty() {
        for zombie in &dead_branches {
            println!("rm {zombie}");
        }
        git(["branch", "-D"].into_iter().chain(dead_branches)).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let mut args = env::args_os().fuse();

    let name = args
        .next()
        .expect("argv[0] should hold the path to this executable");
    let name: &Path = name.as_ref();
    let name = name
        .file_stem()
        .expect("executable path should terminate in file name")
        .to_string_lossy();

    if let Some(arg) = args.next() {
        eprintln!("{name}: error: {arg:?}: unexpected argument");
        exit(2);
    }

    if let Err(err) = main_imp().await {
        eprintln!("{name}: error: {err}");
        exit(1);
    }
}

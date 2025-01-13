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

use std::ffi::OsStr;
use std::io::Write;
use std::path::Path;
use std::process::{exit, ExitStatus};
use std::{env, fmt, io};
use tokio::process::Command;

/// Possible names of local trunk branches, in order of preference.
const TRUNKS: [&str; 2] = ["main", "master"];

#[derive(Debug)]
enum Error {
    /// An unrecognized command line argument was supplied.
    Arg(String),

    /// Git returned bad status, and printed the supplied text to standard error.
    Git(String),

    /// No local trunk branch could be identified.
    Trunk,

    /// The Git working copy has uncommitted changes.
    Unclean,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Arg(arg) => write!(f, "{arg:?}: unexpected argument"),
            Error::Git(stderr) => write!(f, "{stderr}"),
            Error::Trunk => write!(f, "can't find local trunk; any of {TRUNKS:?}"),
            Error::Unclean => write!(f, "working copy is unclean"),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

/// Policy for which branches to delete.
#[allow(dead_code)]
struct Args {
    /// Specifies removal of local branches merged to local trunk.  Note that this does not include
    /// GitHub "squash merges," which do not actually merge the original branch.
    merged: bool,

    /// Specifies removal of local branches whose upstreams are gone.  Upstream branches are often
    /// deleted after being merged to trunk, even if they were "squash merged," so this is a useful
    /// way to detect such "merges."
    gone: bool,
}

impl Args {
    /// Returns the name of this program for use in error logs, and the branch removal policy.
    fn from_env() -> Result<Args> {
        let mut args = env::args().skip(1);

        if let Some(arg) = args.next() {
            return Err(Error::Arg(arg));
        }

        // TODO: Set branch removal policy from CLI.
        let args = Args {
            merged: true,
            gone: true,
        };

        Ok(args)
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

/// Splits the specified Git output into lines, excludes any line beginning with "*", and trims
/// leading whitespace from each line.  Note that the output is not necessarily a list of simple
/// branch names; e.g., if the output is from `git branch --verbose`.
fn trim_branches(stdout: &str) -> impl Iterator<Item = &str> {
    stdout
        .lines()
        .filter(|line| !line.starts_with("* "))
        .map(str::trim_ascii_start)
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
    let rm = Args::from_env()?; // Branch removal policy.

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

    let mut dead_branches = Vec::<String>::new();
    if rm.merged {
        dead_branches.extend(trim_branches(&git(["branch", "--merged"]).await?).map(str::to_owned));
    }

    if rm.gone {
        dead_branches.extend(
            trim_branches(&git(["branch", "--list", "--verbose"]).await?)
                .filter(|line| line.contains("[gone]"))
                .flat_map(|line| line.split_ascii_whitespace().next())
                .map(str::to_owned),
        );
    }

    if dead_branches.iter().any(|s| s == orig) {
        // Let the user know we're not leaving HEAD on the original branch.
        println!("co {trunk}");
    } else if trunk != orig {
        git(["checkout", orig]).await?;
    };

    if !dead_branches.is_empty() {
        for zombie in &dead_branches {
            println!("rm {zombie}");
        }
        git(["branch", "-D"]
            .into_iter()
            .chain(dead_branches.iter().map(String::as_str)))
        .await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let mut args = env::args_os();

    let name = args
        .next()
        .expect("argv[0] should hold the path to this executable");
    let name: &Path = name.as_ref();
    let name = name
        .file_stem()
        .expect("executable path should terminate in file name");

    if let Err(err) = main_imp().await {
        // [`OsStr::display`] is not yet stable:
        // <https://doc.rust-lang.org/std/ffi/struct.OsStr.html#method.display>
        io::stderr()
            .write(name.as_encoded_bytes())
            .expect("stderr should be writable");
        eprintln!(": error: {err}");
        exit(1);
    }
}

//! Provides command-line access to the [`since`] and [`update`] functions.
//!
//! # TODO
//!
//! Add subcommands:
//!
//! * [ ] `tr|trunk` to print trunk name
//! * [ ] `co|checkout`; support aliases, such as `-t` for trunk

use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::exit;
use std::{env, fmt};

use grit::{git, local_trunk, trunk_names};

#[derive(Debug)]
enum Error {
    /// An unrecognized command line argument was supplied.
    Arg(OsString),

    /// The supplied error was returned by the [`grit`] library crate.
    Lib(grit::Error),

    /// The Git working copy has uncommitted changes.
    Unclean,

    /// The user might benefit from a summary of intended usage.
    Usage,
}

impl From<grit::Error> for Error {
    fn from(value: grit::Error) -> Self {
        Error::Lib(value)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Arg(arg) => write!(f, "{}: unexpected argument", arg.display()),
            Error::Lib(err) => err.fmt(f),
            Error::Unclean => "working copy is unclean".fmt(f),
            Error::Usage => "usage: grit {{si|since|up|update}} [ARGS...]".fmt(f),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

/// Policy for which branches to delete.
struct UpdateArgs {
    /// Specifies removal of local branches merged to local trunk.  Note that this does not include
    /// GitHub "squash merges," which do not actually merge the original branch.
    merged: bool,

    /// Specifies removal of local branches whose upstreams are gone.  Upstream branches are often
    /// deleted after being merged to trunk, even if they were "squash merged," so this is a useful
    /// way to detect such "merges."
    gone: bool,
}

impl UpdateArgs {
    /// Returns the name of this program for use in error logs, and the branch removal policy.
    fn new(args: impl IntoIterator<Item = OsString>) -> Result<Self> {
        if let Some(arg) = args.into_iter().next() {
            return Err(Error::Arg(arg));
        }

        // TODO: Set branch removal policy from CLI.
        let args = UpdateArgs {
            merged: true,
            gone: true,
        };

        Ok(args)
    }
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

/// Pulls the main branch of the repo from which it's called, then deletes any
/// local branches that are not ahead of main, and finally checks back out the
/// original branch.  The main branch defaults to `main`, but `master` is used
/// as a fallback if no `main` branch is found.
///
/// # TODO
///
/// * [ ] Delete remote branches behind trunk.
/// * [ ] Don't mess up "checkout -" by checking out main.  I tried _not_
///   checking out main, but after fetch --prune, the user still sees a list
///   of obsolete branches the next time they pull --prune; so now this program
///   checks out main just so it can run pull --prune per se.
async fn update(args: env::ArgsOs) -> Result<()> {
    let rm = UpdateArgs::new(args)?; // Branch removal policy.

    if !is_working_copy_clean().await? {
        return Err(Error::Unclean);
    }

    let orig = git(["rev-parse", "--abbrev-ref", "HEAD"]).await?;
    let orig = orig.as_str().trim();

    let trunk = local_trunk().await?;

    if trunk != orig {
        git(["checkout", &trunk]).await?;
    }

    if upstream(&trunk).await.is_some() {
        match git(["pull", "--prune"]).await.as_deref() {
            Ok("Already up to date.\n") => (),
            Ok(out) => print!("{out}"),
            Err(err) => eprintln!("warning: can't pull {trunk}: {err}"),
        }
    }

    let mut dead_branches = HashSet::<String>::new();
    if rm.merged {
        dead_branches.extend(trim_branches(&git(["branch", "--merged"]).await?).map(str::to_owned));
    }

    if rm.gone {
        dead_branches.extend(
            trim_branches(&git(["branch", "--list", "--verbose"]).await?)
                .filter(|line| line.contains("[gone]"))
                .filter_map(|line| line.split_ascii_whitespace().next())
                .map(str::to_owned),
        );
    }

    // Don't delete potential trunks, even if they're behind the actual trunk.  When you have an
    // integration branch (dev or staging or whatever) that's ahead of main, you want to be able to
    // use that branch as trunk, without deleting main simply because it's behind staging.
    for trunk in trunk_names() {
        dead_branches.remove(&trunk);
    }

    if dead_branches.contains(orig) {
        // Let the user know we're not leaving HEAD on the original branch.
        println!("co {trunk}");
    } else if trunk != orig {
        git(["checkout", orig]).await?;
    }

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

/// Lists commmits reachable from HEAD, but not from a specified base branch
/// (which defaults to the local trunk).
async fn since(our_args: env::ArgsOs) -> Result<()> {
    let mut base: Option<OsString> = None;
    let mut git_args = ["log", "--first-parent", "--oneline"]
        .map(OsString::from)
        .to_vec();
    for os in our_args {
        let Some(s) = os.to_str() else {
            return Err(Error::Arg(os));
        };
        if s.starts_with('-') {
            git_args.push(os);
        } else if base.is_none() {
            base = Some(os);
        } else {
            return Err(Error::Arg(os));
        }
    }
    let range = match base {
        Some(some) => format!("{}..", some.display()),
        None => format!("{}..", local_trunk().await?),
    };
    git_args.push(range.into());
    git(git_args).await?;
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

    let result = match args.next().as_deref().and_then(OsStr::to_str) {
        Some("si" | "since") => since(args).await,
        Some("up" | "update") => update(args).await,
        _ => Err(Error::Usage),
    };

    let Err(err) = result else {
        return;
    };

    let Error::Usage = err else {
        eprintln!("{}: error: {err}", name.display());
        exit(1);
    };

    eprintln!("{}: {err}", name.display());
    exit(2);
}

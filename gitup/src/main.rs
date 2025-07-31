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
//! * [ ] Replace `since` and `gitup` with a single `grit` command:
//!   - `si|since` to replace `since`
//!   - `up|update` to replace `gitup`
//!   - `tr|trunk` to print trunk name
//!   - `co|checkout`; support aliases, such as `-t` for trunk

use std::collections::HashSet;
use std::io::Write;
use std::path::Path;
use std::process::exit;
use std::{env, fmt, io};

use gitup::{git, local_trunk, trunk_names};

#[derive(Debug)]
enum Error {
    /// An unrecognized command line argument was supplied.
    Arg(String),

    /// The Git working copy has uncommitted changes.
    Unclean,

    Lib(gitup::Error),
}

impl From<gitup::Error> for Error {
    fn from(value: gitup::Error) -> Self {
        Error::Lib(value)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Arg(arg) => write!(f, "{arg:?}: unexpected argument"),
            Error::Lib(err) => err.fmt(f),
            Error::Unclean => "working copy is unclean".fmt(f),
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

async fn main_imp() -> Result<()> {
    let rm = Args::from_env()?; // Branch removal policy.

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
            .write_all(name.as_encoded_bytes())
            .expect("stderr should be writable");
        eprintln!(": error: {err}");
        exit(1);
    }
}

//! Provides command-line access to the [`since`] and [`update`] functions.

use std::{collections::HashSet, env, ffi, fmt, process::exit};

use grit::{
    git::{HEAD, git},
    trunk,
};

#[derive(Debug)]
enum Error {
    /// An unrecognized command line argument was supplied.
    Arg(ffi::OsString),

    /// Git produced unexpected output. (If Git itself is not found, we panic.)
    Git(grit::git::Error),

    /// No local trunk branch could be identified. This can happen if a command
    /// is run outside of any git repo, or if the repo has no local branch
    /// matching any recognized trunk name.
    Trunk(grit::trunk::Error),

    /// The Git working copy has uncommitted changes.
    Unclean,
}

impl From<grit::git::Error> for Error {
    fn from(value: grit::git::Error) -> Self {
        Error::Git(value)
    }
}

impl From<grit::trunk::Error> for Error {
    fn from(value: grit::trunk::Error) -> Self {
        Error::Trunk(value)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Arg(arg) => write!(f, "{}: unexpected argument", arg.display()),
            Error::Git(err) => err.fmt(f),
            Error::Trunk(err) => err.fmt(f),
            Error::Unclean => "working copy is unclean".fmt(f),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

/// Policy for which branches to delete.
struct UpdateArgs {
    /// Specifies removal of local branches merged to local trunk.  Note that
    /// this does not include GitHub "squash merges," which do not actually
    /// merge the original branch.
    merged: bool,

    /// Specifies removal of local branches whose upstreams are gone.  Upstream
    /// branches are often deleted after being merged to trunk, even if they
    /// were "squash merged," so this is a useful way to detect such
    /// "merges."
    gone: bool,
}

impl UpdateArgs {
    /// Returns the name of this program for use in error logs, and the branch
    /// removal policy.
    fn new(args: impl IntoIterator<Item = ffi::OsString>) -> Result<Self> {
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

/// Splits the specified Git output into lines, excludes any line beginning with
/// "*", and trims leading whitespace from each line.  Note that the output is
/// not necessarily a list of simple branch names; e.g., if the output is from
/// `git branch --verbose`.
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
///   checking out main, but after fetch --prune, the user still sees a list of
///   obsolete branches the next time they pull --prune; so now this program
///   checks out main just so it can run pull --prune per se.
async fn update(args: env::ArgsOs) -> Result<()> {
    let rm = UpdateArgs::new(args)?; // Branch removal policy.

    if !is_working_copy_clean().await? {
        return Err(Error::Unclean);
    }

    let orig = git(["rev-parse", "--abbrev-ref", HEAD]).await?;
    let orig = orig.as_str().trim();
    let trunk = trunk::local().await?;
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

    // Don't delete potential trunks, even if they're behind the actual trunk.  When
    // you have an integration branch (dev or staging or whatever) that's ahead
    // of main, you want to be able to use that branch as trunk, without
    // deleting main simply because it's behind staging.
    for trunk in trunk::names() {
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

struct SinceArgs {
    /// Arguments to be forwarded to git.
    git_args: Vec<ffi::OsString>,
    /// The base branch name, if specified.
    base: Option<ffi::OsString>,
}

impl SinceArgs {
    fn new(our_args: env::ArgsOs) -> Result<SinceArgs> {
        let mut base: Option<ffi::OsString> = None;
        let mut git_args = [
            "log",
            "--color=always",
            "--decorate",
            "--first-parent",
            "--graph",
            "--oneline",
        ]
        .map(ffi::OsString::from)
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
        Ok(SinceArgs { git_args, base })
    }
}

/// Lists commmits reachable from HEAD, but not from a specified base branch
/// (which defaults to the local trunk).
///
/// TODO: Command-line arguments are fundamentally [`ffi::OsString`], not
///  [`String`]. Converting the former to the latter and back, to build `git(1)`
///  range arguments, is potentially lossy. Avoid it by concatenating byte
///  encoded vectors rather than formatting strings.
async fn since(our_args: env::ArgsOs) -> Result<()> {
    let SinceArgs { mut git_args, base } = SinceArgs::new(our_args)?;
    let range = match base {
        Some(some) => format!("{}..", some.display()),
        None => format!("{}..", trunk::local().await?),
    };
    git_args.push(range.into());
    print!("{}", git(git_args).await?);
    Ok(())
}

/// Summarizes changes to the working copy (relative to HEAD), then lists
/// commmits from trunk (or a specified base) to HEAD, inclusive.
async fn since_long(our_args: env::ArgsOs) -> Result<()> {
    let SinceArgs { mut git_args, base } = SinceArgs::new(our_args)?;
    print!("{}", git(["diff", "--stat"]).await?);
    let range = match base {
        Some(some) => format!("{}^..", some.display()),
        None => format!("{}^..", trunk::local().await?),
    };
    git_args.push(range.into());
    print!("{}", git(git_args).await?);
    Ok(())
}

/// Prints the name of the local trunk branch, if any is identified.
async fn trunk(mut args: env::ArgsOs) -> Result<()> {
    if let Some(arg) = args.next() {
        return Err(Error::Arg(arg));
    }
    println!("{}", trunk::local().await?);
    Ok(())
}

#[expect(clippy::ref_option)]
fn to_str(arg: &Option<ffi::OsString>) -> Option<&str> {
    arg.as_deref().and_then(ffi::OsStr::to_str)
}

#[tokio::main]
async fn main() {
    let mut args = env::args_os();
    args.next(); // Skip program name.
    let arg = args.next();
    let is_verbose = matches!(to_str(&arg), Some("-v" | "--verbose"));
    let command = if is_verbose { args.next() } else { arg };
    let result = match to_str(&command) {
        Some("si" | "since") if is_verbose => since_long(args).await,
        Some("si" | "since") => since(args).await,
        Some("tr" | "trunk") => trunk(args).await,
        Some("up" | "update") => update(args).await,
        _ => {
            let usage = "Usage:\
                \n    grit [-v|--verbose] {si|since} [GIT_FLAGS...] [BASE]\
                \n    grit {tr|trunk}\
                \n    grit {up|update}";
            eprintln!("{usage}");
            exit(2);
        }
    };
    if let Err(err) = result {
        eprintln!("Error: {err}");
        exit(1);
    }
}

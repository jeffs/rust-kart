use std::{collections::HashSet, env, ffi};

use crate::{
    error::{Error, Result},
    git::{self, HEAD, git},
    trunk,
};

/// Policy for which branches to delete.
struct Args {
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

impl Args {
    /// Returns the name of this program for use in error logs, and the branch
    /// removal policy.
    fn new(args: impl IntoIterator<Item = ffi::OsString>) -> Result<Self> {
        if let Some(arg) = args.into_iter().next() {
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

async fn is_working_copy_clean() -> Result<bool> {
    Ok(git(["status", "--porcelain"]).await?.is_empty())
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

/// Pulls the main branch of the repo from which it's called, then deletes any
/// local branches that are not ahead of trunk, and finally checks back out the
/// original branch. The trunk branch is the first existing branch returned by
/// [`trunk::names()`].
///
/// # Errors
///
/// Returns an error if no trunk can be identified, or any `git` command fails.
///
/// # TODO
///
/// * [ ] Delete remote branches behind trunk.
/// * [ ] Don't mess up "checkout -" by checking out main.  I tried _not_
///   checking out main, but after fetch --prune, the user still sees a list
///   of obsolete branches the next time they pull --prune; so now this program
///   checks out main just so it can run pull --prune per se.
pub async fn update(args: env::ArgsOs) -> Result<()> {
    let rm = Args::new(args)?; // Branch removal policy.

    if !is_working_copy_clean().await? {
        return Err(Error::Unclean);
    }

    let orig = git(["rev-parse", "--abbrev-ref", HEAD]).await?;
    let orig = orig.as_str().trim();
    let trunk = trunk::local().await?;
    if trunk != orig {
        git(["checkout", &trunk]).await?;
    }

    if git::upstream(&trunk).await.is_some() {
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

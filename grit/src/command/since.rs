use std::{env, ffi};

use crate::{
    error::{Error, Result},
    git::git,
    trunk,
};

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
///
/// # Errors
///
/// Returns an error if the base branch is unspecified, and a trunk cannot be
/// identified; or, if `git` returns bad status.
pub async fn since(our_args: env::ArgsOs) -> Result<()> {
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
///
/// # Errors
///
/// Returns an error if the base branch cannot be identified, or `git` fails.
pub async fn since_long(our_args: env::ArgsOs) -> Result<()> {
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

use std::{env, ffi};

use crate::{
    error::{Error, Result},
    git::git,
    trunk,
};

struct Args {
    /// Arguments to be forwarded to git.
    git_args: Vec<ffi::OsString>,
    /// The base branch name, if specified.
    base: Option<ffi::OsString>,
}

impl Args {
    fn new(our_args: env::ArgsOs) -> Result<Args> {
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
        Ok(Args { git_args, base })
    }
}

/// # Errors
///
/// Returns an error if no branch is specified, or if `git` returns bad status.
///
/// # TODO
///
/// * Default to current branch
/// * Accept arbitrary refs; e.g., don't archive `HEAD` as `HEAD`
pub async fn archive(args: env::ArgsOs) -> Result<()> {
    let branches: Box<[ffi::OsString]> = args.collect();
    if branches.is_empty() {
        return;
    }

    let Args { mut git_args, base } = Args::new(our_args)?;
    let range = match base {
        Some(some) => format!("{}..", some.display()),
        None => format!("{}..", trunk::local().await?),
    };
    git_args.push(range.into());
    print!("{}", git(git_args).await?);
    Ok(())
}

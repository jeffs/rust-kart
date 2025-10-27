use std::{ffi, fmt};

use crate::{git, trunk};

#[derive(Debug)]
pub enum Error {
    /// Archive
    Archive,

    /// An unrecognized command line argument was supplied.
    Arg(ffi::OsString),

    /// Git produced unexpected output. (If Git itself is not found, we panic.)
    Git(git::Error),

    /// No local trunk branch could be identified. This can happen if a command
    /// is run outside of any git repo, or if the repo has no local branch
    /// matching any recognized trunk name.
    Trunk(trunk::Error),

    /// The Git working copy has uncommitted changes.
    Unclean,
}

impl From<git::Error> for Error {
    fn from(value: git::Error) -> Self {
        Error::Git(value)
    }
}

impl From<trunk::Error> for Error {
    fn from(value: trunk::Error) -> Self {
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

pub type Result<T> = std::result::Result<T, Error>;

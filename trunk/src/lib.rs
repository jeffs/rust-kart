//! Simple package for identifying the main ("trunk") branch of a version
//! control repository.

#![allow(dead_code)]

use std::path::PathBuf;

/// Default names to consider when searching for local trunk branch, in order of preference.
/// Overridden by the value of the [`GITUP_TRUNKS`] environment variable.
const DEFAULT_TRUNKS: [&str; 2] = ["main", "master"];

/// Environment variable to check for comma-separated list of local trunk branch names.  If the
/// variable is unset, the value defaults to [`DEFAULT_TRUNKS`].
const ENV_TRUNKS: &str = "RK_TRUNKS";

#[derive(Debug)]
pub enum Error {
    /// The supplied path is not in a repository.
    RepositoryNotFound(PathBuf),
    /// No local branch has any of the supplied names.
    Trunk(Vec<String>),
}

type Result<T> = std::result::Result<T, Error>;

pub struct Repository(PathBuf);

impl Repository {
    /// Returns an object representing the repository (working copy) containing
    /// the specified path.
    ///
    /// # Errors
    ///
    /// Returns [`Err::RepositoryNotFound`] if the path is not in any repository.
    pub fn from_path(_path: impl Into<PathBuf>) -> Result<Repository> {
        todo!("find repo root")
    }

    fn trunks(&self) -> Result<Vec<String>> {
        todo!("return strings from env or default")
    }

    /// Returns the first of the specified names that matches any local branch.
    ///
    /// TODO: Take names from config, rather than parameter.
    pub async fn local_trunk(&self) -> Result<String> {
        // TODO: Replace loop with find.
        let trunks = self.trunks()?;
        for branch in &trunks {
            // TODO: Check `.git/refs/heads`.
        }
        Err(Error::Trunk(trunks))
    }
}

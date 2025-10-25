use std::fmt;

use crate::git::git;

/// No local branch was found to have any of the supplied names.
#[derive(Debug)]
pub struct Error(Vec<String>);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "trunk not found: {}", self.0.join(", "))
    }
}

/// Default names to consider when searching for local trunk branch, in order of
/// preference. Overridden by the value of the [`GRIT_TRUNKS`] environment
/// variable.
pub const DEFAULT_TRUNKS: [&str; 2] = ["main", "master"];

/// Environment variable to check for comma-separated list of local trunk branch
/// names.  If the variable is unset, the value defaults to [`DEFAULT_TRUNKS`].
pub const GRIT_TRUNKS: &str = "GRIT_TRUNKS";

/// Returns the names of potential trunk branches, per [`GRIT_TRUNKS`] (if set)
/// or [`DEFAULT_TRUNKS`].
pub fn names() -> Vec<String> {
    let trunks = std::env::var(GRIT_TRUNKS);
    match trunks.as_ref() {
        Ok(trunks) => trunks.split(',').map(str::to_owned).collect(),
        Err(_) => DEFAULT_TRUNKS.map(str::to_owned).to_vec(),
    }
}

/// Returns the name of the local trunk branch, if it can be determined.
///
/// # Errors
///
/// Will return [`Error`] if no local trunk branch is found.
pub async fn local() -> Result<String, Error> {
    let names = names();
    for name in &names {
        if git(["show-ref", name]).await.is_ok() {
            return Ok(name.to_owned());
        }
    }
    Err(Error(names))
}

mod branch;
mod render;

pub use branch::{Branch, Topology};
pub use render::render;

use grit::trunk;

/// Collect branch topology data for the current repository.
///
/// # Errors
///
/// Returns an error if trunk detection or git operations fail.
pub async fn topology() -> Result<Topology, Error> {
    let trunk = trunk::local().await.map_err(Error::Trunk)?;
    branch::collect(&trunk).await.map_err(Error::Git)
}

#[derive(Debug)]
pub enum Error {
    Trunk(trunk::Error),
    Git(grit::git::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Trunk(e) => write!(f, "{e}"),
            Error::Git(e) => write!(f, "{e}"),
        }
    }
}

mod branch;
mod rebase;
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

/// Rebase a branch and all its descendants onto trunk.
///
/// # Errors
///
/// Returns an error if the branch is not found, or if git operations fail.
pub async fn rebase(branch_name: &str) -> Result<(), Error> {
    let trunk = trunk::local().await.map_err(Error::Trunk)?;
    let topology = branch::collect(&trunk).await.map_err(Error::Git)?;

    // Find the branch in the topology tree.
    let Some(branch) = find_branch(&topology.branches, branch_name) else {
        return Err(Error::BranchNotFound(branch_name.to_owned()));
    };

    rebase::rebase_stack(&trunk, branch).await.map_err(Error::Git)
}

fn find_branch<'a>(branches: &'a [Branch], name: &str) -> Option<&'a Branch> {
    for branch in branches {
        if branch.name == name {
            return Some(branch);
        }
        if let Some(found) = find_branch(&branch.children, name) {
            return Some(found);
        }
    }
    None
}

#[derive(Debug)]
pub enum Error {
    Trunk(trunk::Error),
    Git(grit::git::Error),
    BranchNotFound(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Trunk(e) => write!(f, "{e}"),
            Error::Git(e) => write!(f, "{e}"),
            Error::BranchNotFound(name) => write!(f, "branch not found: {name}"),
        }
    }
}

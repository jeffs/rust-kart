mod branch;
mod rebase;
mod render;

pub use branch::{BranchInfo, Node, Topology};
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

    // Find the node containing the branch in the topology tree.
    let Some(node) = find_branch_node(&topology.root, branch_name) else {
        return Err(Error::BranchNotFound(branch_name.to_owned()));
    };

    rebase::rebase_stack(&trunk, node).await.map_err(Error::Git)
}

/// Find a node by branch name in the tree.
fn find_branch_node<'a>(node: &'a Node, name: &str) -> Option<&'a Node> {
    // Check if this node contains the branch we're looking for.
    if node.branches.iter().any(|info| info.name == name) {
        return Some(node);
    }

    // Search children.
    for child in &node.children {
        if let Some(found) = find_branch_node(child, name) {
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

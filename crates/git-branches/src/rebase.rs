use grit::git::{self, git};

use crate::branch::Branch;

/// Rebase a branch and all its descendants onto trunk.
pub async fn rebase_stack(
    trunk: &str,
    branch: &Branch,
) -> Result<(), git::Error> {
    let old_tip = branch.tip.clone();

    // Rebase this branch onto trunk.
    println!("Rebasing {} onto {trunk}...", branch.name);
    git(["rebase", trunk, &branch.name]).await?;

    // Get the new tip after rebasing.
    let new_tip = git(["rev-parse", &branch.name]).await?.trim().to_owned();

    // Rebase all children onto the new tip.
    for child in &branch.children {
        rebase_child(child, &old_tip, &new_tip).await?;
    }

    Ok(())
}

/// Recursively rebase a child branch onto its rebased parent.
async fn rebase_child(
    branch: &Branch,
    old_parent_tip: &str,
    new_parent_tip: &str,
) -> Result<(), git::Error> {
    let old_tip = branch.tip.clone();

    // Rebase: replay commits from old_parent_tip..branch onto new_parent_tip.
    println!(
        "Rebasing {} onto {}...",
        branch.name,
        &new_parent_tip[..8]
    );
    git([
        "rebase",
        "--onto",
        new_parent_tip,
        old_parent_tip,
        &branch.name,
    ])
    .await?;

    // Get the new tip after rebasing.
    let new_tip = git(["rev-parse", &branch.name]).await?.trim().to_owned();

    // Rebase all children onto the new tip.
    for child in &branch.children {
        Box::pin(rebase_child(child, &old_tip, &new_tip)).await?;
    }

    Ok(())
}

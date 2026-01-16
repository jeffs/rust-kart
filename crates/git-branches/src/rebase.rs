use grit::git::{self, git};

use crate::branch::Node;

/// Rebase a branch and all its descendants onto trunk.
///
/// The node must have at least one branch.
pub async fn rebase_stack(trunk: &str, node: &Node) -> Result<(), git::Error> {
    // Filter out [HEAD] pseudo-branch - it's not a real git branch.
    let branches: Vec<_> = node
        .branches
        .iter()
        .filter(|b| b.name != "[HEAD]")
        .collect();

    if branches.is_empty() {
        // Can't rebase a pure commit node.
        return Ok(());
    }

    let old_tip = node.commit.clone();

    // Rebase all branches at this commit onto trunk.
    // Use the first branch as the "main" one for rebasing.
    let first_branch = &branches[0].name;
    println!("Rebasing {first_branch} onto {trunk}...");
    git(["rebase", trunk, first_branch]).await?;

    // Get the new tip after rebasing.
    let new_tip = git(["rev-parse", first_branch]).await?.trim().to_owned();

    // Reset other branches at this commit to the new tip.
    for info in &branches[1..] {
        git(["branch", "-f", &info.name, &new_tip]).await?;
    }

    // Rebase all children onto the new tip.
    for child in &node.children {
        rebase_child(child, &old_tip, &new_tip).await?;
    }

    Ok(())
}

/// Recursively rebase a child node onto its rebased parent.
async fn rebase_child(
    node: &Node,
    old_parent_tip: &str,
    new_parent_tip: &str,
) -> Result<(), git::Error> {
    let old_tip = node.commit.clone();

    // Filter out [HEAD] pseudo-branch - it's not a real git branch.
    let branches: Vec<_> = node
        .branches
        .iter()
        .filter(|b| b.name != "[HEAD]")
        .collect();

    // If this node has branches, rebase them.
    if !branches.is_empty() {
        let first_branch = &branches[0].name;
        println!(
            "Rebasing {} onto {}...",
            first_branch,
            &new_parent_tip[..7.min(new_parent_tip.len())]
        );
        git([
            "rebase",
            "--onto",
            new_parent_tip,
            old_parent_tip,
            first_branch,
        ])
        .await?;

        // Get the new tip after rebasing.
        let new_tip = git(["rev-parse", first_branch]).await?.trim().to_owned();

        // Reset other branches at this commit to the new tip.
        for info in &branches[1..] {
            git(["branch", "-f", &info.name, &new_tip]).await?;
        }

        // Rebase children using this branch's new tip as their parent.
        for child in &node.children {
            Box::pin(rebase_child(child, &old_tip, &new_tip)).await?;
        }
    } else {
        // This is a pure commit node - just pass through to children.
        // They still need to rebase from the same old_parent_tip.
        for child in &node.children {
            Box::pin(rebase_child(child, old_parent_tip, new_parent_tip)).await?;
        }
    }

    Ok(())
}

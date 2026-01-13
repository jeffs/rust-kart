use grit::git::{self, git};
use std::collections::HashMap;

/// A branch node in the topology tree.
#[derive(Debug)]
pub struct Branch {
    pub name: String,
    pub tip: String,
    pub ahead: usize,  // commits ahead of parent
    pub behind: usize, // commits behind trunk (always relative to trunk)
    pub children: Vec<Branch>,
}

/// The topology of all local branches.
#[derive(Debug)]
pub struct Topology {
    pub trunk: String,
    pub trunk_tip: String,
    pub branches: Vec<Branch>, // branches forking directly from trunk
}

/// Collect all local branches and build the topology tree.
pub async fn collect(trunk: &str) -> Result<Topology, git::Error> {
    let output = git(["for-each-ref", "--format=%(refname:short)", "refs/heads/"]).await?;
    let branch_names: Vec<&str> = output.lines().filter(|s| *s != trunk).collect();

    if branch_names.is_empty() {
        let trunk_tip = git(["rev-parse", trunk]).await?.trim().to_owned();
        return Ok(Topology {
            trunk: trunk.to_owned(),
            trunk_tip,
            branches: vec![],
        });
    }

    // Get tip commit for each branch.
    let mut tips: HashMap<&str, String> = HashMap::new();
    for name in &branch_names {
        let tip = git(["rev-parse", name]).await?.trim().to_owned();
        tips.insert(name, tip);
    }

    let trunk_tip = git(["rev-parse", trunk]).await?.trim().to_owned();

    // For each branch, find its parent (trunk or another branch).
    // Parent is the branch/trunk whose tip is an ancestor and closest to this branch.
    let mut parents: HashMap<&str, Option<&str>> = HashMap::new();

    for name in &branch_names {
        let mut best_parent: Option<&str> = None;
        let mut best_distance = usize::MAX;
        let branch_tip = tips.get(name).unwrap();

        // Check each other branch as potential parent.
        // A branch B is a valid parent if B's tip is an ancestor of this branch.
        for other in &branch_names {
            if other == name {
                continue;
            }
            let other_tip = tips.get(other).unwrap();

            // Is other's tip an ancestor of this branch?
            let is_ancestor =
                git(["merge-base", "--is-ancestor", other_tip, branch_tip]).await;
            if is_ancestor.is_ok() {
                let dist = count_commits(other_tip, branch_tip).await?;
                if dist < best_distance {
                    best_distance = dist;
                    best_parent = Some(other);
                }
            }
        }

        // If no branch is a closer parent, trunk is the parent (None).
        parents.insert(name, best_parent);
    }

    // Build the tree recursively.
    let mut root_branches = Vec::new();
    for name in &branch_names {
        if parents.get(name) == Some(&None) {
            // This branch forks from trunk.
            let branch = build_branch(name, trunk, &tips, &parents, &branch_names).await?;
            root_branches.push(branch);
        }
    }

    // Sort root branches alphabetically.
    root_branches.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(Topology {
        trunk: trunk.to_owned(),
        trunk_tip,
        branches: root_branches,
    })
}

/// Recursively build a branch node with its children.
async fn build_branch(
    name: &str,
    trunk: &str,
    tips: &HashMap<&str, String>,
    parents: &HashMap<&str, Option<&str>>,
    all_branches: &[&str],
) -> Result<Branch, git::Error> {
    let tip = tips.get(name).unwrap().clone();

    // Find parent ref to calculate ahead count.
    let parent_ref = match parents.get(name).and_then(|p| *p) {
        Some(parent_branch) => tips.get(parent_branch).unwrap().clone(),
        None => git::merge_base(trunk, name).await?, // Fork point from trunk
    };

    let ahead = count_commits(&parent_ref, &tip).await?;
    let behind = count_commits(&tip, trunk).await?;

    // Find children (branches whose parent is this branch).
    let mut children = Vec::new();
    for other in all_branches {
        if parents.get(other) == Some(&Some(name)) {
            let child = Box::pin(build_branch(other, trunk, tips, parents, all_branches)).await?;
            children.push(child);
        }
    }
    children.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(Branch {
        name: name.to_owned(),
        tip,
        ahead,
        behind,
        children,
    })
}

/// Count commits in range `from..to`.
async fn count_commits(from: &str, to: &str) -> Result<usize, git::Error> {
    let output = git(["rev-list", "--count", &format!("{from}..{to}")]).await?;
    Ok(output.trim().parse().unwrap_or(0))
}

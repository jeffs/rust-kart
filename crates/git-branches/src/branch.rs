use grit::git::{self, git};
use std::collections::{HashMap, HashSet};

/// A node in the topology tree. Every node is a commit; branches add metadata.
#[derive(Debug)]
pub struct Node {
    pub commit: String,           // commit SHA (short, 7 chars)
    pub branches: Vec<BranchInfo>, // branches pointing to this commit (may be empty)
    pub children: Vec<Node>,
}

/// Branch metadata attached to a node.
#[derive(Debug)]
pub struct BranchInfo {
    pub name: String,
    pub ahead: usize,    // commits from parent node to this tip
    pub pr: Option<u32>, // PR number if branch has an open PR
}

/// The topology of all local branches.
#[derive(Debug)]
pub struct Topology {
    pub trunk: String, // trunk branch name
    pub root: Node,    // root node (common ancestor of all branches)
}

/// Fetch open PRs and return a map from branch name to PR number.
async fn fetch_prs() -> HashMap<String, u32> {
    let output = tokio::process::Command::new("gh")
        .args(["pr", "list", "--json", "headRefName,number"])
        .output()
        .await;
    let Ok(output) = output else {
        return HashMap::new();
    };
    if !output.status.success() {
        return HashMap::new();
    }
    let json = String::from_utf8_lossy(&output.stdout);

    // Parse JSON: [{"headRefName": "branch", "number": 123}, ...]
    let mut prs = HashMap::new();
    // Simple JSON parsing without adding a dependency.
    for line in json.split('{') {
        let Some(name_start) = line.find("\"headRefName\":\"") else {
            continue;
        };
        let rest = &line[name_start + 15..];
        let Some(name_end) = rest.find('"') else {
            continue;
        };
        let name = &rest[..name_end];

        let Some(num_start) = line.find("\"number\":") else {
            continue;
        };
        let rest = &line[num_start + 9..];
        let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(num) = num_str.parse() {
            prs.insert(name.to_owned(), num);
        }
    }
    prs
}

/// Collect all local branches and build the topology tree with merge-base commits
/// as first-class nodes.
pub async fn collect(trunk: &str) -> Result<Topology, git::Error> {
    // Fetch open PRs.
    let prs = fetch_prs().await;

    // Get HEAD commit.
    let head_commit = git(["rev-parse", "--short=7", "HEAD"])
        .await
        .map(|s| s.trim().to_owned())
        .ok();

    // Phase 1: Collect all branch tips (including trunk).
    let output = git(["for-each-ref", "--format=%(refname:short)", "refs/heads/"]).await?;
    let all_branches: Vec<&str> = output.lines().collect();

    let mut tips: HashMap<&str, String> = HashMap::new();
    for name in &all_branches {
        let tip = git(["rev-parse", "--short=7", name]).await?.trim().to_owned();
        tips.insert(name, tip);
    }

    // Handle empty repo or trunk-only case.
    if all_branches.len() <= 1 {
        let trunk_tip = tips.get(trunk).cloned().unwrap_or_default();
        let mut branches = vec![BranchInfo {
            name: trunk.to_owned(),
            ahead: 0,
            pr: prs.get(trunk).copied(),
        }];
        if head_commit.as_deref() == Some(trunk_tip.as_str()) {
            branches.push(BranchInfo {
                name: "[HEAD]".to_owned(),
                ahead: 0,
                pr: None,
            });
        }
        return Ok(Topology {
            trunk: trunk.to_owned(),
            root: Node {
                commit: trunk_tip.clone(),
                branches,
                children: vec![],
            },
        });
    }

    // Phase 2: Find all merge-bases (fork points).
    let mut commits: HashSet<String> = HashSet::new();

    // Add all branch tips.
    for tip in tips.values() {
        commits.insert(tip.clone());
    }

    // Add HEAD commit so it appears even when detached.
    if let Some(ref head) = head_commit {
        commits.insert(head.clone());
    }

    // Compute merge-base of each non-trunk branch with trunk.
    let mut trunk_merge_bases: HashMap<&str, String> = HashMap::new();
    for name in &all_branches {
        if *name == trunk {
            continue;
        }
        let mb = git(["merge-base", trunk, name]).await?;
        let mb_short = git(["rev-parse", "--short=7", mb.trim()]).await?.trim().to_owned();
        trunk_merge_bases.insert(name, mb_short.clone());
        commits.insert(mb_short);
    }

    // Group branches by their trunk merge-base to find those that need pairwise comparison.
    let mut by_merge_base: HashMap<&str, Vec<&str>> = HashMap::new();
    for (branch, mb) in &trunk_merge_bases {
        by_merge_base.entry(mb.as_str()).or_default().push(branch);
    }

    // For branches sharing a trunk merge-base, compute pairwise merge-bases.
    for (_mb, branches) in &by_merge_base {
        if branches.len() > 1 {
            for i in 0..branches.len() {
                for j in (i + 1)..branches.len() {
                    let mb = git(["merge-base", branches[i], branches[j]]).await?;
                    let mb_short = git(["rev-parse", "--short=7", mb.trim()]).await?.trim().to_owned();
                    commits.insert(mb_short);
                }
            }
        }
    }

    // Phase 3: Build parent relationships.
    // For each commit, find its parent (closest ancestor in our commit set).
    let commits_vec: Vec<String> = commits.into_iter().collect();
    let mut parents: HashMap<String, Option<String>> = HashMap::new();

    for commit in &commits_vec {
        let mut best_parent: Option<String> = None;
        let mut best_distance = usize::MAX;

        for other in &commits_vec {
            if other == commit {
                continue;
            }

            // Is other an ancestor of commit?
            let is_ancestor = git(["merge-base", "--is-ancestor", other, commit]).await;
            if is_ancestor.is_ok() {
                let dist = count_commits(other, commit).await?;
                if dist < best_distance && dist > 0 {
                    best_distance = dist;
                    best_parent = Some(other.clone());
                }
            }
        }

        parents.insert(commit.clone(), best_parent);
    }

    // Phase 4: Build tree from root.
    // Find root (commit with no parent in our set).
    let root_commit = commits_vec
        .iter()
        .find(|c| parents.get(*c) == Some(&None))
        .cloned()
        .unwrap_or_else(|| tips.get(trunk).cloned().unwrap_or_default());

    // Build a map from commit to branch names (multiple branches may point to same commit).
    let mut commit_to_branches: HashMap<String, Vec<String>> = HashMap::new();
    for (name, tip) in &tips {
        commit_to_branches
            .entry(tip.clone())
            .or_default()
            .push((*name).to_owned());
    }

    // Add [HEAD] as a pseudo-branch at its commit.
    if let Some(ref head) = head_commit {
        commit_to_branches
            .entry(head.clone())
            .or_default()
            .push("[HEAD]".to_owned());
    }

    let root = build_node(&root_commit, &commits_vec, &parents, &commit_to_branches, &prs).await?;

    Ok(Topology {
        trunk: trunk.to_owned(),
        root,
    })
}

/// Recursively build a node and its children.
async fn build_node(
    commit: &str,
    all_commits: &[String],
    parents: &HashMap<String, Option<String>>,
    commit_to_branches: &HashMap<String, Vec<String>>,
    prs: &HashMap<String, u32>,
) -> Result<Node, git::Error> {
    // Find children (commits whose parent is this commit).
    let mut child_commits: Vec<&String> = all_commits
        .iter()
        .filter(|c| parents.get(*c) == Some(&Some(commit.to_owned())))
        .collect();

    // Sort children for deterministic output.
    child_commits.sort();

    // Build children recursively.
    let mut children = Vec::new();
    for child in child_commits {
        let child_node =
            Box::pin(build_node(child, all_commits, parents, commit_to_branches, prs)).await?;
        children.push(child_node);
    }

    // Calculate ahead count (distance from parent).
    let ahead = if let Some(Some(parent)) = parents.get(commit) {
        count_commits(parent, commit).await?
    } else {
        0
    };

    // Get all branches pointing to this commit. Branches sorted alphabetically,
    // with [HEAD] always last.
    let branches = commit_to_branches
        .get(commit)
        .map(|names| {
            let mut infos: Vec<BranchInfo> = names
                .iter()
                .filter(|name| *name != "[HEAD]")
                .map(|name| BranchInfo {
                    name: name.clone(),
                    ahead,
                    pr: prs.get(name).copied(),
                })
                .collect();
            infos.sort_by(|a, b| a.name.cmp(&b.name));
            // Append [HEAD] at the end if present.
            if names.iter().any(|n| n == "[HEAD]") {
                infos.push(BranchInfo {
                    name: "[HEAD]".to_owned(),
                    ahead,
                    pr: None,
                });
            }
            infos
        })
        .unwrap_or_default();

    Ok(Node {
        commit: commit.to_owned(),
        branches,
        children,
    })
}

/// Count commits in range `from..to`.
async fn count_commits(from: &str, to: &str) -> Result<usize, git::Error> {
    let output = git(["rev-list", "--count", &format!("{from}..{to}")]).await?;
    Ok(output.trim().parse().unwrap_or(0))
}

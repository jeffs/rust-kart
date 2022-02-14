//! This program deletes Git branches (local and on remote 'origin') that are
//! not ahead of the main branch (or master branch, if main doesn't exist).
//!
//! Sample usage:
//!
//! ```sh
//! $ git-prune -n
//! jeff.git-prune
//! origin/jeff.git-prune
//!
//! $ git-prune
//! deleting jeff.git-prune
//! deleting origin/jeff.git-prune
//! ```

use std::env;
use std::process::{exit, Command};

const MAIN_REFS: [&str; 2] = ["main", "master"];

/// Runs git with the specified args, then trims and returns its output lines.
fn git<'a, I>(args: I) -> Result<Vec<String>, String>
where
    I: IntoIterator<Item = &'a str>,
{
    let output = Command::new("git")
        .args(args.into_iter())
        .output()
        .map_err(|e| format!("can't run git: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into());
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .collect())
}

/// Passes args to git() and returns the first output line.  Returns an error
/// unless git outputs exactly one line.
fn git_single<'a, I>(args: I) -> Result<String, String>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut lines = git(args)?;
    match lines.pop() {
        Some(line) if lines.is_empty() => Ok(line),
        _ => Err("expected exactly one line from git".to_string()),
    }
}

struct SymRef {
    source: String,
}

impl SymRef {
    fn target(target: &str) -> Result<SymRef, String> {
        let source = git_single(["symbolic-ref", "HEAD"])?;
        git(["symbolic-ref", "-m", "git-prune push", "HEAD", target])?;
        Ok(SymRef { source })
    }
}

impl Drop for SymRef {
    fn drop(&mut self) {
        if let Err(why) = git(["symbolic-ref", "-m", "git-prune pop", "HEAD", &self.source]) {
            eprintln!("warning: can't restore HEAD to original ref: {why}");
        }
    }
}

fn full_name(branch: &str) -> Result<String, String> {
    git_single(["rev-parse", "--symbolic-full-name", branch])
}

fn fetch_main_ref() -> Result<String, &'static str> {
    for branch in MAIN_REFS {
        if let Ok(sym) = full_name(&format!("origin/{branch}")) {
            if git(["fetch", "origin", branch]).is_err() {
                eprintln!("warning: can't fetch origin {branch}");
            }
            return Ok(sym);
        }
        if let Ok(sym) = full_name(branch) {
            return Ok(sym);
        }
    }
    Err("no main or master branch found")
}

fn strip_stars(mut lines: Vec<String>) -> Vec<String> {
    for line in &mut lines {
        if let Some(suffix) = line.strip_prefix("* ") {
            let branch = suffix.to_string();
            *line = branch;
        }
    }
    lines
}

fn list_local_branches() -> Result<Vec<String>, String> {
    Ok(strip_stars(git(["branch"])?))
}

fn list_remote_branches() -> Result<Vec<String>, String> {
    let mut branches = strip_stars(git(["branch", "--remotes"])?);
    branches.retain(|s| s.starts_with("origin/") && !s.contains(' '));
    Ok(branches)
}

fn is_main(branch: &str) -> bool {
    if let Some(suffix) = branch.strip_prefix("origin/") {
        MAIN_REFS.contains(&suffix)
    } else {
        MAIN_REFS.contains(&branch.as_ref())
    }
}

fn is_empty(branch: &str) -> Result<bool, String> {
    Ok(git(["log", &format!("..{branch}")])?.is_empty())
}

fn retain_obsolete(branches: Vec<String>) -> Result<Vec<String>, String> {
    let mut obsoletes = Vec::new();
    for branch in branches {
        if !is_main(&branch) && is_empty(&branch)? {
            obsoletes.push(branch);
        }
    }
    Ok(obsoletes)
}

fn for_each_obsolete_branch(
    handle_branch: &dyn Fn(&str) -> Result<(), String>,
) -> Result<(), String> {
    let _ = git(["remote", "prune", "origin"]);
    let mut branches = list_local_branches()?;
    branches.append(&mut list_remote_branches()?);
    {
        let _drop_symref = SymRef::target(&fetch_main_ref()?)?;
        branches = retain_obsolete(branches)?;
    }
    for branch in branches {
        handle_branch(&branch)?;
    }
    Ok(())
}

fn delete_branch(branch: &str) -> Result<(), String> {
    eprintln!("deleting {branch}");
    if let Some(suffix) = branch.strip_prefix("origin/") {
        git(["push", "origin", &format!(":{suffix}")])?;
    } else {
        git(["branch", "-d", &branch])?;
    }
    Ok(())
}

fn print_branch(branch: &str) -> Result<(), String> {
    println!("{branch}");
    Ok(())
}

fn main() {
    let handle_branch = if env::args().skip(1).any(|arg| arg == "-n") {
        print_branch
    } else {
        delete_branch
    };
    if let Err(what) = for_each_obsolete_branch(&handle_branch) {
        eprintln!("{what}");
        exit(1);
    }
}

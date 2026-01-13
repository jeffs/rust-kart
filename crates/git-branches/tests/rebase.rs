#![allow(clippy::similar_names)] // feature_a_before vs feature_b_before is intentional.

use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Path to the git-branches binary (built by cargo test).
fn binary_path() -> std::path::PathBuf {
    // The test binary is in target/debug/deps, the main binary is in target/debug.
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test binary name.
    path.pop(); // Remove deps/.
    path.push("git-branches");
    path
}

/// Run a git command in the given directory, panicking on failure.
fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git command failed to execute");

    assert!(
        output.status.success(),
        "git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

/// Run git-branches in the given directory.
fn git_branches(dir: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new(binary_path())
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git-branches command failed to execute");

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_owned())
    }
}

/// Create a commit with a message, returning the commit hash.
fn commit(dir: &Path, message: &str) -> String {
    // Create/modify a file to have something to commit.
    std::fs::write(dir.join(message), message).unwrap();
    git(dir, &["add", "."]);
    git(dir, &["commit", "-m", message]);
    git(dir, &["rev-parse", "HEAD"])
}

/// Get the parent commit of a ref.
fn parent(dir: &Path, refname: &str) -> String {
    git(dir, &["rev-parse", &format!("{refname}^")])
}

/// Check if commit A is an ancestor of commit B.
fn is_ancestor(dir: &Path, ancestor: &str, descendant: &str) -> bool {
    Command::new("git")
        .args(["merge-base", "--is-ancestor", ancestor, descendant])
        .current_dir(dir)
        .status()
        .expect("git command failed")
        .success()
}

/// Initialize a git repo in the given directory.
fn init_repo(dir: &Path) {
    git(dir, &["init"]);
    git(dir, &["config", "user.email", "test@test.com"]);
    git(dir, &["config", "user.name", "Test"]);
}

/// Set up a test repo with stacked branches:
///
/// ```text
/// main:      M1 -- M2 -- M3
///                   \
/// feature-a:         A1
///                     \
/// feature-b:           B1
/// ```
fn setup_stacked_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    let path = dir.path();

    init_repo(path);

    // Create main branch with initial commits.
    commit(path, "M1");
    commit(path, "M2");

    // Create feature-a from M2.
    git(path, &["checkout", "-b", "feature-a"]);
    commit(path, "A1");

    // Create feature-b from feature-a.
    git(path, &["checkout", "-b", "feature-b"]);
    commit(path, "B1");

    // Go back to main and add M3.
    git(path, &["checkout", "main"]);
    commit(path, "M3");

    dir
}

#[test]
fn test_rebase_stacked_branches() {
    let dir = setup_stacked_repo();
    let path = dir.path();

    // Record state before rebase.
    let main_tip = git(path, &["rev-parse", "main"]);
    let feature_a_before = git(path, &["rev-parse", "feature-a"]);
    let feature_b_before = git(path, &["rev-parse", "feature-b"]);

    // Verify initial structure: feature-a is NOT based on M3.
    assert!(
        !is_ancestor(path, &main_tip, &feature_a_before),
        "feature-a should not contain M3 before rebase"
    );

    // Run the rebase command.
    let output = git_branches(path, &["rebase", "feature-a"]);
    assert!(output.is_ok(), "rebase failed: {output:?}");

    // Get new tips after rebase.
    let feature_a_after = git(path, &["rev-parse", "feature-a"]);
    let feature_b_after = git(path, &["rev-parse", "feature-b"]);

    // Verify: feature-a is now based on main (M3 is ancestor of A1').
    assert!(
        is_ancestor(path, &main_tip, &feature_a_after),
        "feature-a should now contain M3"
    );

    // Verify: feature-b is based on the NEW feature-a tip.
    assert_eq!(
        parent(path, "feature-b"),
        feature_a_after,
        "feature-b should be based on new feature-a tip"
    );

    // Verify: commits were actually rebased (hashes changed).
    assert_ne!(
        feature_a_before, feature_a_after,
        "feature-a tip should have changed"
    );
    assert_ne!(
        feature_b_before, feature_b_after,
        "feature-b tip should have changed"
    );

    // Verify: the commit messages are preserved.
    let a_message = git(path, &["log", "-1", "--format=%s", "feature-a"]);
    let b_message = git(path, &["log", "-1", "--format=%s", "feature-b"]);
    assert_eq!(a_message, "A1");
    assert_eq!(b_message, "B1");
}

#[test]
fn test_rebase_deeper_stack() {
    let dir = TempDir::new().unwrap();
    let path = dir.path();

    init_repo(path);

    // main: M1
    commit(path, "M1");

    // feature-a from main.
    git(path, &["checkout", "-b", "feature-a"]);
    commit(path, "A1");

    // feature-b from feature-a.
    git(path, &["checkout", "-b", "feature-b"]);
    commit(path, "B1");

    // feature-c from feature-b.
    git(path, &["checkout", "-b", "feature-c"]);
    commit(path, "C1");

    // Add M2 to main.
    git(path, &["checkout", "main"]);
    commit(path, "M2");

    let main_tip = git(path, &["rev-parse", "main"]);

    // Run rebase.
    let output = git_branches(path, &["rebase", "feature-a"]);
    assert!(output.is_ok(), "rebase failed: {output:?}");

    // Verify chain: main -> feature-a -> feature-b -> feature-c.
    let tip_a = git(path, &["rev-parse", "feature-a"]);
    let tip_b = git(path, &["rev-parse", "feature-b"]);

    assert!(is_ancestor(path, &main_tip, &tip_a));
    assert_eq!(parent(path, "feature-a"), main_tip);
    assert_eq!(parent(path, "feature-b"), tip_a);
    assert_eq!(parent(path, "feature-c"), tip_b);
}

#[test]
fn test_topology_display() {
    let dir = setup_stacked_repo();
    let path = dir.path();

    let output = git_branches(path, &[]).unwrap();

    // Should show main with feature-a and feature-b nested.
    assert!(output.contains("main"), "should show main");
    assert!(output.contains("feature-a"), "should show feature-a");
    assert!(output.contains("feature-b"), "should show feature-b");
}

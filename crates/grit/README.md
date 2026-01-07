# grit

Git utilities for trunk-based development.

## Commands

- `gitup` — Pulls trunk and deletes local branches that are behind it.
- `since` — Lists commits reachable from HEAD but not from a base branch.

## To Do

* Support local ref management in custom namespaces:
  ```nu
  git update-ref refs/USER/TOPIC (git show-ref -s refs/heads/USER/TOPIC)
  ```
* Support branch-specific trunks, which may belong to different upstream repos.
* Save rev-parse results in case user wants to restore deleted branches.
  - Support `restore` subcommand that accepts rev-parse output.
* Support custom remote names rather than hard-coding origin.
* Accept branch filter patterns; e.g., `$USER.*`
* Create git-squash, taking a branch, merge, or PR #.
  - Other semantic operations not directly supported by git, and/or by gh?
* Optionally format output as table or JSON
* List commits from a given merge `M`; i.e., `M^..M^2`
* Support `rebase` to fetch and then rebase onto autommatically detected trunk.
  - `git rebase` defaults to the remote tracking branch, if any
* Automatically swap `https://` with `git@` URLs for submodules
  - The URLs live in `.git/modules/*/config`
  - Git doesn't recognize modifications as a diff
  - Each working copy needs to use whichever style fits locally configured auth

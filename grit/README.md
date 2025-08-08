# Gitup

The `gitup` command pulls the trunk branch of the repo whence it is called, then
deletes any local branches that are behind that trunk.

The `since` command lists commmits reachable from HEAD, but not from a specified
base branch (which defaults to the local trunk).

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
  - `git rebase` defaults to the remote tracking branch, if any.

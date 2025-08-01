# To Do

* Consolidate `gitup` and `since` into a single `grit` project
  - Avoid confusion with the unrelated [GitUp](https://github.com/git-up/GitUp)
  - Replace crate-specific features like `GRIT_TRUNKS` with shared `GRIT_TRUNKS`
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

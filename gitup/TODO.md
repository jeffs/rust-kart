# To Do

* Rename this crate to avoid confusion with the unrelated [GitUp](https://github.com/git-up/GitUp).
* Save rev-parse results in case user wants to restore deleted branches.
  - Support `restore` subcommand that accepts rev-parse output.
* Support custom remote names rather than hard-coding origin.
* Accept branch filter patterns; e.g., `$USER.*`
* Create git-squash, taking a branch, merge, or PR #.
  - Other semantic operations not directly supported by git, and/or by gh?
* Format git output: glog, gable; JSON

# module-from-path

Converts file paths to Rust module names.

## Usage

```sh
module-from-path src/foo/bar.rs
# Output: foo::bar

module-from-path src/foo/bar.rs /
# Output: foo/bar
```

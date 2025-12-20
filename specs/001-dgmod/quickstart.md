# Quickstart: dgmod

**Date**: 2025-12-20
**Branch**: `001-dgmod`

## Overview

`dgmod` generates Mermaid diagrams of Rust module dependencies. It analyzes `mod` declarations and `use` statements to build a directed graph showing how modules relate to each other.

---

## Installation

```bash
# From the rust-kart workspace root
cargo install --path dgmod

# Or run directly
cargo run -p dgmod -- /path/to/crate
```

---

## Basic Usage

### Analyze a Single Crate

```bash
# Analyze crate and output Mermaid to stdout
dgmod /path/to/my-crate

# Save to file
dgmod /path/to/my-crate > deps.md

# Render with Mermaid CLI
dgmod /path/to/my-crate | mmdc -i - -o deps.svg
```

### Analyze a Workspace

```bash
# Analyzes all workspace members, outputs separate graph per crate
dgmod /path/to/workspace
```

---

## Example Output

For a crate with this structure:
```
src/
├── lib.rs      # pub mod alpha; pub mod beta; pub mod gamma;
├── alpha/
│   ├── mod.rs  # pub mod delta; pub use delta::Delta;
│   └── delta.rs
├── beta.rs     # use crate::{alpha::Delta, gamma};
└── gamma.rs    # use super::Root;
```

Running `dgmod /path/to/crate` outputs the following Markdown:

    ## my-crate

    ```mermaid
    flowchart TD
        crate["crate"]
        alpha["alpha"]
        alpha_delta["alpha::delta"]
        beta["beta"]
        gamma["gamma"]

        crate --> alpha
        crate --> beta
        crate --> gamma
        alpha --> alpha_delta
        beta --> alpha
        beta --> gamma
        gamma --> crate
    ```

---

## Output Format

The output is Mermaid flowchart syntax, compatible with:
- GitHub/GitLab markdown (rendered automatically)
- Notion (paste directly)
- Mermaid CLI (`mmdc`)
- Mermaid Live Editor (https://mermaid.live)

### Node Naming

- Root module: `crate`
- Submodules: Fully qualified path without `crate::` prefix
- Example: `alpha::delta` (not `crate::alpha::delta`)

### Edge Semantics

- `parent --> child`: `mod` declaration (parent declares child)
- `importer --> target`: `use` statement imports from target module

---

## Error Handling

dgmod exits with non-zero status on errors:

```bash
# Parse error
$ dgmod /path/with/syntax/error
error: Parse error in /path/with/syntax/error/src/bad.rs:
  --> line 5:10
  unexpected token

$ echo $?
1

# Missing module file
$ dgmod /path/with/missing/mod
error: Module 'foo' not found. Expected:
  - /path/with/missing/mod/src/foo.rs
  - /path/with/missing/mod/src/foo/mod.rs

$ echo $?
1
```

Errors go to stderr; stdout contains only the graph (or nothing on error).

---

## Integration Examples

### Generate SVG with Mermaid CLI

```bash
npm install -g @mermaid-js/mermaid-cli
dgmod /path/to/crate | mmdc -i - -o output.svg
```

### Add to README

```bash
# Generate and append to README
echo "## Module Dependencies" >> README.md
echo "" >> README.md
dgmod . >> README.md
```

### CI Integration

```yaml
# .github/workflows/docs.yml
- name: Generate module graph
  run: |
    cargo install --path dgmod
    dgmod . > docs/module-graph.md
```

---

## Limitations

- **External crates excluded**: Only intra-crate dependencies are tracked
- **No cfg evaluation**: All conditional modules included regardless of features
- **No macro expansion**: Modules declared via macros may not be detected
- **Path-based resolution**: Uses `#[path]` attributes when present

---

## Next Steps

After generating a graph:

1. **Identify cycles**: Look for bidirectional arrows (A → B and B → A)
2. **Find hotspots**: Modules with many incoming edges are high-dependency
3. **Plan refactoring**: Use the graph to inform module reorganization

For large codebases, consider splitting the graph or using subgraphs (future feature).

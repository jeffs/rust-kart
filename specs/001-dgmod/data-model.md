# Data Model: dgmod

**Date**: 2025-12-20
**Branch**: `001-dgmod`

## Overview

This document defines the core data structures for the dgmod tool. The model is intentionally simple: modules are nodes, dependencies are edges, forming a directed graph.

---

## Entities

### Module

Represents a Rust module within a crate.

| Field | Type | Description |
|-------|------|-------------|
| `path` | `ModulePath` | Fully qualified path (e.g., `crate`, `alpha::delta`) |
| `source_file` | `PathBuf` | Absolute path to the source file |
| `kind` | `ModuleKind` | Root, inline, or external file |

**Constraints:**
- `path` is unique within a crate
- Root module always has path `crate`
- Non-root paths exclude `crate::` prefix (e.g., `alpha`, not `crate::alpha`)

### ModulePath

Newtype wrapper for module paths.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModulePath(String);

impl ModulePath {
    pub fn crate_root() -> Self { Self("crate".to_string()) }
    pub fn child(&self, name: &str) -> Self {
        if self.0 == "crate" {
            Self(name.to_string())
        } else {
            Self(format!("{}::{}", self.0, name))
        }
    }
    pub fn as_str(&self) -> &str { &self.0 }
}
```

### ModuleKind

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleKind {
    /// Crate root (lib.rs or main.rs)
    Root,
    /// Inline module: `mod foo { ... }`
    Inline,
    /// External file: `mod foo;` → foo.rs or foo/mod.rs
    External,
}
```

### Edge

Represents a directed dependency from one module to another.

| Field | Type | Description |
|-------|------|-------------|
| `from` | `ModulePath` | Source module |
| `to` | `ModulePath` | Target module |
| `kind` | `EdgeKind` | How the dependency was established |

**Constraints:**
- At most one edge per direction between any pair of modules (FR-009)
- Self-edges are not created (module cannot depend on itself)

### EdgeKind

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    /// Parent declares child: `mod foo;`
    ModDeclaration,
    /// Import via use statement: `use foo::Bar;`
    UseImport,
}
```

### ModuleGraph

The complete dependency graph for a crate.

| Field | Type | Description |
|-------|------|-------------|
| `crate_name` | `String` | Name of the crate |
| `modules` | `HashMap<ModulePath, Module>` | All modules by path |
| `edges` | `HashSet<(ModulePath, ModulePath)>` | Deduplicated edges |

**Constraints:**
- All edge endpoints must exist in `modules`
- `crate` module always exists

```rust
#[derive(Debug)]
pub struct ModuleGraph {
    pub crate_name: String,
    modules: HashMap<ModulePath, Module>,
    edges: HashSet<(ModulePath, ModulePath)>,
}

impl ModuleGraph {
    pub fn new(crate_name: String) -> Self { ... }
    pub fn add_module(&mut self, module: Module) { ... }
    pub fn add_edge(&mut self, from: ModulePath, to: ModulePath) {
        if from != to {
            self.edges.insert((from, to));
        }
    }
    pub fn modules(&self) -> impl Iterator<Item = &Module> { ... }
    pub fn edges(&self) -> impl Iterator<Item = (&ModulePath, &ModulePath)> { ... }
}
```

---

## Relationships

```
┌─────────────────────────────────────────────────────────┐
│                     ModuleGraph                          │
│  crate_name: String                                      │
├─────────────────────────────────────────────────────────┤
│                                                          │
│    ┌──────────┐         edges          ┌──────────┐     │
│    │  Module  │ ───────────────────▶   │  Module  │     │
│    │  (from)  │   (ModDeclaration      │  (to)    │     │
│    └──────────┘    or UseImport)       └──────────┘     │
│         │                                    │          │
│         └──────────────┬─────────────────────┘          │
│                        │                                 │
│               ModulePath (unique key)                    │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

---

## State Transitions

The graph is built incrementally during analysis:

```
┌───────────┐     parse root      ┌──────────────┐
│   Empty   │ ──────────────────▶ │  Root Added  │
└───────────┘                     └──────┬───────┘
                                         │
                    ┌────────────────────┘
                    │ for each mod declaration
                    ▼
            ┌───────────────┐     parse child     ┌─────────────────┐
            │ Add mod edge  │ ──────────────────▶ │ Add child module│
            └───────────────┘                     └────────┬────────┘
                    ▲                                      │
                    └──────────────────────────────────────┘
                              (recursive)

              After all modules parsed:
            ┌─────────────────────────────────────┐
            │  for each use statement in module   │
            │  ──▶ resolve target module          │
            │  ──▶ add use edge if internal       │
            └─────────────────────────────────────┘
```

---

## Validation Rules

| Rule | Description | Enforcement |
|------|-------------|-------------|
| Unique paths | No duplicate module paths | HashMap key uniqueness |
| Valid edges | Edge endpoints exist in graph | Assert on add_edge |
| No self-edges | Module cannot edge to itself | Filter in add_edge |
| Single edge | One edge per direction max | HashSet deduplication |

---

## Example

For the sample crate at `/Users/jeff/sample`:

**Modules:**
| Path | Kind | Source File |
|------|------|-------------|
| `crate` | Root | `lib.rs` |
| `alpha` | External | `alpha/mod.rs` |
| `alpha::delta` | External | `alpha/delta.rs` |
| `beta` | External | `beta.rs` |
| `gamma` | External | `gamma.rs` |

**Edges:**
| From | To | Kind |
|------|-----|------|
| `crate` | `alpha` | ModDeclaration |
| `crate` | `beta` | ModDeclaration |
| `crate` | `gamma` | ModDeclaration |
| `alpha` | `alpha::delta` | ModDeclaration |
| `beta` | `alpha` | UseImport |
| `beta` | `gamma` | UseImport |
| `gamma` | `crate` | UseImport |

---

## Mermaid Output Mapping

```rust
impl ModuleGraph {
    pub fn to_mermaid(&self) -> String {
        let mut output = String::from("flowchart TD\n");

        // Emit nodes
        for module in self.modules() {
            let id = module.path.as_str().replace("::", "_");
            let label = module.path.as_str();
            output.push_str(&format!("    {}[\"{}\"]\n", id, label));
        }

        output.push('\n');

        // Emit edges
        for (from, to) in self.edges() {
            let from_id = from.as_str().replace("::", "_");
            let to_id = to.as_str().replace("::", "_");
            output.push_str(&format!("    {} --> {}\n", from_id, to_id));
        }

        output
    }
}
```

# Feature Specification: Rust Module Dependency Graph Tool (dgmod)

**Feature Branch**: `001-dgmod`
**Created**: 2025-12-20
**Status**: Draft
**Input**: User description: "Create a tool to graph Rust module imports within any crate for which we have local source code."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Generate Module Dependency Graph for a Crate (Priority: P1)

A developer wants to visualize the module structure and dependencies within a Rust crate to understand how modules relate to each other. They run `dgmod` against a crate's source directory and receive a graph showing all module nodes and their import edges.

**Why this priority**: This is the core functionality - without generating a dependency graph, the tool has no purpose.

**Independent Test**: Can be fully tested by running the tool against the sample crate at `/Users/jeff/sample` and verifying the output contains the expected nodes and edges as specified in the requirements.

**Acceptance Scenarios**:

1. **Given** a Rust crate with source code available locally, **When** the user runs dgmod with the crate path, **Then** the tool outputs a directed graph with all modules as nodes
2. **Given** a crate with module declarations (`mod foo;`), **When** the tool analyzes the crate, **Then** edges are created from parent modules to child modules
3. **Given** a crate with use declarations (`use a::b::c;`), **When** the tool analyzes the crate, **Then** edges are created from the importing module to the referenced module (or its parent if not a module)

---

### User Story 2 - Understand Cyclic Dependencies (Priority: P2)

A developer suspects there are cyclic dependencies between modules and wants to identify them. By analyzing the dgmod output, they can trace cycles in the graph.

**Why this priority**: Detecting cycles is a common use case for dependency graphs, helping developers refactor and improve architecture.

**Independent Test**: Can be tested by running against a crate with known cycles (like the sample crate where `crate` and `gamma` form a cycle) and verifying the edges in both directions appear.

**Acceptance Scenarios**:

1. **Given** a crate where module A imports from module B and B imports from A, **When** the tool generates the graph, **Then** both edges A→B and B→A appear in the output
2. **Given** a crate with an indirect cycle (A→B→C→A), **When** the tool generates the graph, **Then** all three edges appear allowing the cycle to be traced

---

### User Story 3 - Distinguish Re-exports from Direct Definitions (Priority: P3)

A developer wants to understand which modules expose items from other modules via re-exports. The tool should show dependencies based on the import path used, not the original definition location.

**Why this priority**: This nuance helps developers understand the public API surface of modules versus implementation details.

**Independent Test**: Can be tested by creating a crate where module A re-exports an item from module B, and module C imports that item via A. The graph should show C→A, not C→B.

**Acceptance Scenarios**:

1. **Given** module `alpha` re-exports `Delta` from `alpha::delta`, and module `beta` imports `Delta` via `crate::alpha::Delta`, **When** the tool analyzes the crate, **Then** an edge appears from `beta` to `alpha` (not to `alpha::delta`)
2. **Given** a module is re-exported under a different name or path, **When** the module itself is referenced in the graph, **Then** it appears only once using its fully qualified definition path (sans `crate::` prefix)

---

### Edge Cases

- What happens when a crate has no submodules (only the root module)? → Output shows only the `crate` node with no edges
- What happens when a source file cannot be parsed? → Tool reports the parse error with file location and continues processing other files
- What happens when `mod` declarations reference non-existent files? → Tool reports the missing file and continues processing
- What happens with `#[path = "..."]` attribute overrides on `mod` declarations? → Tool follows the specified path
- What happens with `use` statements importing external crates? → External crates are excluded; only intra-crate dependencies are tracked
- What happens with glob imports (`use foo::*;`)? → An edge is created to the glob's source module
- What happens with multiple imports from the same module? → Only one edge exists per direction between any pair of modules

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST accept a path to a Rust crate's source directory as input
- **FR-002**: System MUST identify all modules within the crate by parsing source files
- **FR-003**: System MUST represent each module as exactly one node in the output graph
- **FR-004**: System MUST name the root module as `crate`
- **FR-005**: System MUST name all other modules using their fully qualified path from the crate root, without the `crate::` prefix (e.g., `alpha`, `alpha::delta`)
- **FR-006**: System MUST create an edge from a parent module to a child module for each `mod` declaration
- **FR-007**: System MUST create an edge from module M to module N when M contains a `use` statement referencing an item in N
- **FR-008**: For `use` statements referencing items (not modules), the edge MUST target the module containing the item in the import path, not the item's original definition module
- **FR-009**: System MUST ensure at most one edge exists per direction for any pair of modules
- **FR-010**: System MUST handle cyclic dependencies without infinite loops or crashes
- **FR-011**: System MUST output the graph in Mermaid diagram syntax (flowchart format)
- **FR-012**: System MUST gracefully handle parse errors, reporting them to stderr without aborting the entire analysis (graph output goes to stdout)
- **FR-013**: System MUST exclude dependencies on external crates (only intra-crate module relationships)
- **FR-014**: System MUST handle `#[path = "..."]` attribute overrides on module declarations
- **FR-015**: When given a Cargo workspace path, system MUST detect all member crates and produce a separate graph for each crate (no cross-crate edges)

### Key Entities

- **Module**: A Rust module identified by its fully qualified path. Attributes: path (e.g., `crate`, `alpha`, `alpha::delta`), source file location
- **Edge**: A directed dependency from one module to another. Attributes: source module, target module, edge type (mod declaration or use import)
- **Graph**: A directed graph containing all modules as nodes and all dependencies as edges. May contain cycles.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Tool correctly identifies all modules in a crate, including nested submodules
- **SC-002**: Tool produces the exact expected graph for the reference sample crate at `/Users/jeff/sample` (7 edges: `crate→alpha`, `crate→beta`, `crate→gamma`, `alpha→alpha::delta`, `beta→alpha`, `beta→gamma`, `gamma→crate`)
- **SC-003**: Tool completes analysis of a 100-module crate in under 5 seconds
- **SC-004**: Tool handles malformed Rust files by reporting errors and continuing, rather than crashing
- **SC-005**: Output format is parseable and can be piped to other tools for visualization or analysis

## Clarifications

### Session 2025-12-20

- Q: What output format should the graph use? → A: Mermaid diagram syntax
- Q: How should Cargo workspaces be handled? → A: Workspace-aware but isolate each crate (separate graph per crate member)
- Q: Where should parse errors be reported? → A: Stderr (errors to stderr, graph to stdout)

## Assumptions

- The tool operates on crates with source code available locally (not on compiled artifacts)
- Standard Rust 2018/2021 edition module resolution rules apply
- The tool is a command-line utility
- Output format is Mermaid diagram syntax, compatible with GitHub/GitLab markdown rendering and Mermaid Live Editor
- The tool does not need to resolve or download external crate dependencies
- Conditional compilation (`#[cfg(...)]`) attributes on modules will include all conditional modules in the graph (no evaluation of cfg conditions)

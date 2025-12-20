# Tasks: dgmod (Rust Module Dependency Graph)

**Input**: Design documents from `/specs/001-dgmod/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: No explicit test tasks - tests will be added as part of implementation per Rust conventions (`cargo test`).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Per plan.md, this is a single Rust crate within the rust-kart workspace:

```
dgmod/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── main.rs
│   ├── parser.rs
│   ├── graph.rs
│   ├── resolver.rs
│   ├── mermaid.rs
│   └── workspace.rs
└── tests/
    ├── integration/
    └── fixtures/
```

---

## Phase 1: Setup (Project Initialization)

**Purpose**: Create the dgmod crate structure and configure dependencies

- [ ] T001 Create dgmod directory and Cargo.toml with dependencies (syn, clap, cargo_metadata) in dgmod/Cargo.toml. Include comments explaining each dependency per constitution II: syn (Rust parsing, no stdlib alternative), clap (CLI parsing, minimal and maintained), cargo_metadata (workspace detection, FR-015)
- [ ] T002 Add dgmod to workspace members in Cargo.toml (workspace root)
- [ ] T003 [P] Create empty module files: dgmod/src/lib.rs, dgmod/src/main.rs
- [ ] T004 [P] Create empty module files: dgmod/src/parser.rs, dgmod/src/graph.rs, dgmod/src/resolver.rs
- [ ] T005 [P] Create empty module files: dgmod/src/mermaid.rs, dgmod/src/workspace.rs
- [ ] T006 Create test fixture directory structure: dgmod/tests/integration/, dgmod/tests/fixtures/
- [ ] T007 Verify project compiles with `cargo check -p dgmod`

**Checkpoint**: Project skeleton compiles, ready for implementation

---

## Phase 2: Foundational (Core Data Structures)

**Purpose**: Implement shared data types that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T008 Implement ModulePath newtype with crate_root(), child(), as_str() methods in dgmod/src/graph.rs
- [ ] T009 [P] Implement ModuleKind enum (Root, Inline, External) in dgmod/src/graph.rs
- [ ] T010 [P] Implement EdgeKind enum (ModDeclaration, UseImport) in dgmod/src/graph.rs
- [ ] T011 Implement Module struct with path, source_file, kind fields in dgmod/src/graph.rs
- [ ] T012 Implement ModuleGraph struct with modules HashMap and edges HashSet in dgmod/src/graph.rs
- [ ] T013 Implement ModuleGraph::new(), add_module(), add_edge() methods in dgmod/src/graph.rs
- [ ] T014 Implement ModuleGraph::modules() and edges() iterator methods in dgmod/src/graph.rs
- [ ] T015 Define error types (ParseError, ResolveError) in dgmod/src/lib.rs
- [ ] T016 Wire up lib.rs with pub mod declarations for all modules and add `#![forbid(unsafe_code)]` in dgmod/src/lib.rs
- [ ] T017 Verify foundational code compiles with `cargo check -p dgmod`

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Generate Module Dependency Graph (Priority: P1) 🎯 MVP

**Goal**: Analyze a single Rust crate and output a Mermaid graph showing all modules and their dependencies

**Independent Test**: Run `dgmod /Users/jeff/sample` and verify output contains expected 7 edges per SC-002

### Implementation for User Story 1

#### Parsing Layer

- [ ] T018 [US1] Implement parse_file() function using syn::parse_file in dgmod/src/parser.rs
- [ ] T019 [US1] Implement extract_mod_declarations() to find all ItemMod in a file in dgmod/src/parser.rs
- [ ] T020 [US1] Implement extract_use_statements() to find all ItemUse in a file in dgmod/src/parser.rs
- [ ] T021 [US1] Implement get_path_attribute() to extract #[path="..."] from ItemMod in dgmod/src/parser.rs

#### Resolution Layer

- [ ] T022 [US1] Implement find_crate_root() to locate lib.rs or main.rs in dgmod/src/resolver.rs
- [ ] T023 [US1] Implement resolve_module_file() for standard module path resolution in dgmod/src/resolver.rs
- [ ] T024 [US1] Handle #[path] attribute override in resolve_module_file() in dgmod/src/resolver.rs
- [ ] T025 [US1] Implement is_inline_module() to detect mod foo { ... } vs mod foo; in dgmod/src/resolver.rs

#### Graph Building

- [ ] T026 [US1] Implement analyze_crate() orchestration function in dgmod/src/lib.rs
- [ ] T027 [US1] Implement recursive module discovery (parse root, follow mod declarations) in dgmod/src/lib.rs
- [ ] T028 [US1] Implement add_mod_edges() to create parent→child edges for mod declarations in dgmod/src/lib.rs

#### Use Statement Processing

- [ ] T029 [US1] Implement walk_use_tree() to extract all import paths from UseTree in dgmod/src/parser.rs
- [ ] T030 [US1] Implement is_internal_path() to filter crate::/self::/super:: prefixes in dgmod/src/resolver.rs
- [ ] T031 [US1] Implement resolve_use_target() to find target module for a use path in dgmod/src/resolver.rs
- [ ] T032 [US1] Implement add_use_edges() to create importer→target edges in dgmod/src/lib.rs

#### Mermaid Output

- [ ] T033 [US1] Implement sanitize_id() to replace :: with _ for Mermaid node IDs in dgmod/src/mermaid.rs
- [ ] T034 [US1] Implement emit_nodes() to generate node declarations with labels in dgmod/src/mermaid.rs
- [ ] T035 [US1] Implement emit_edges() to generate edge declarations in dgmod/src/mermaid.rs
- [ ] T036 [US1] Implement ModuleGraph::to_mermaid() combining nodes and edges in dgmod/src/mermaid.rs

#### CLI Layer

- [ ] T037 [US1] Define CLI args struct with clap derive macro (path argument) in dgmod/src/main.rs
- [ ] T038 [US1] Implement main() to parse args, call analyze_crate(), print Mermaid to stdout in dgmod/src/main.rs
- [ ] T039 [US1] Implement error handling: log to stderr, exit with non-zero on errors in dgmod/src/main.rs

#### Integration Test

- [ ] T040 [US1] Create test fixture: copy /Users/jeff/sample structure to dgmod/tests/fixtures/sample/
- [ ] T041 [US1] Create integration test verifying expected 7 edges in dgmod/tests/integration/sample_crate.rs
- [ ] T042 [US1] Verify `cargo test -p dgmod` passes

**Checkpoint**: User Story 1 complete - single crate analysis with Mermaid output works

---

## Phase 4: User Story 2 - Cyclic Dependency Detection (Priority: P2)

**Goal**: Ensure cyclic dependencies are correctly represented (edges in both directions appear)

**Independent Test**: Run against sample crate and verify gamma→crate edge appears alongside crate→gamma

### Implementation for User Story 2

- [ ] T043 [US2] Verify HashSet edge storage allows bidirectional edges in dgmod/src/graph.rs
- [ ] T044 [US2] Add integration test for cycle detection (A→B, B→A both present) in dgmod/tests/integration/cycles.rs
- [ ] T045 [US2] Create test fixture with indirect cycle (A→B→C→A) in dgmod/tests/fixtures/cyclic/

**Checkpoint**: User Story 2 complete - cycles are correctly represented

---

## Phase 5: User Story 3 - Re-export Path Resolution (Priority: P3)

**Goal**: Dependencies based on import path, not original definition location

**Independent Test**: Create crate with re-export, verify edge targets re-export source not original definition

### Implementation for User Story 3

- [ ] T046 [US3] Ensure resolve_use_target() returns module from import path, not definition in dgmod/src/resolver.rs
- [ ] T047 [US3] Add test case: alpha re-exports delta::Delta, beta imports alpha::Delta in dgmod/tests/integration/reexports.rs
- [ ] T048 [US3] Verify edge is beta→alpha, not beta→alpha::delta in dgmod/tests/integration/reexports.rs
- [ ] T049 [US3] Create test fixture demonstrating re-export pattern in dgmod/tests/fixtures/reexport/

**Checkpoint**: User Story 3 complete - re-exports handled per spec

---

## Phase 6: Workspace Support (FR-015)

**Goal**: Detect Cargo workspaces and produce separate graph per member crate

**Independent Test**: Run against rust-kart workspace root, verify separate graphs for each member

### Implementation for Workspace Support

- [ ] T050 Implement detect_workspace() using cargo_metadata in dgmod/src/workspace.rs
- [ ] T051 Implement get_workspace_members() to list all crate roots in dgmod/src/workspace.rs
- [ ] T052 Implement is_workspace() to check if path is workspace vs single crate in dgmod/src/workspace.rs
- [ ] T053 Update main() to iterate workspace members if workspace detected in dgmod/src/main.rs
- [ ] T054 Emit separate Mermaid graph per crate with crate name header in dgmod/src/main.rs
- [ ] T055 Add integration test for workspace handling in dgmod/tests/integration/workspace.rs

**Checkpoint**: Workspace support complete

---

## Phase 7: Polish & Error Handling

**Purpose**: Edge cases, error handling, and cross-cutting concerns

- [ ] T056 [P] Handle no-submodule crate (root only, no edges) in dgmod/src/lib.rs
- [ ] T057 [P] Handle glob imports (use foo::*) creating edge to source module in dgmod/src/resolver.rs
- [ ] T058 [P] Implement external crate filtering (FR-013) in dgmod/src/resolver.rs
- [ ] T059 Ensure parse errors exit with non-zero status and clear message in dgmod/src/main.rs
- [ ] T060 Ensure missing module file errors exit with non-zero status in dgmod/src/main.rs
- [ ] T061 Add doc comments to all public types and functions in dgmod/src/lib.rs
- [ ] T062 Run `cargo clippy -p dgmod -- -W clippy::pedantic` and fix all warnings (per constitution)
- [ ] T063 Run `cargo fmt -p dgmod` to ensure consistent formatting
- [ ] T064 Verify `cargo test -p dgmod` passes all tests
- [ ] T065 Manual validation against quickstart.md scenarios
- [ ] T066 [P] Create or locate 100-module test crate in dgmod/tests/fixtures/large/ and verify analysis completes in under 5 seconds (SC-003)
- [ ] T067 [P] Add test fixture with cfg-guarded module (`#[cfg(feature = "x")] mod foo;`) and verify it appears in graph in dgmod/tests/fixtures/cfg_module/

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational - core functionality
- **User Story 2 (Phase 4)**: Depends on Foundational - independent of US1
- **User Story 3 (Phase 5)**: Depends on Foundational - independent of US1/US2
- **Workspace (Phase 6)**: Depends on US1 (needs working single-crate analysis)
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

| Story | Depends On | Can Parallelize With |
|-------|------------|---------------------|
| US1 (P1) | Foundational | - |
| US2 (P2) | Foundational + US1 implementation | US3 |
| US3 (P3) | Foundational + US1 implementation | US2 |
| Workspace | US1 | - |

> **Note**: US2 and US3 are validation phases that verify US1's core implementation handles cycles and re-exports correctly. They can run in parallel with each other but require US1's parsing/resolution code to exist.

### Parallel Opportunities per Phase

**Phase 1 (Setup)**:
```
T003, T004, T005 can run in parallel (different files)
```

**Phase 2 (Foundational)**:
```
T009, T010 can run in parallel (different types in same file - use [P] sparingly)
```

**Phase 3 (US1)**:
```
Parsing layer tasks (T018-T021) → then Resolution layer (T022-T025)
Graph building (T026-T028) and Use processing (T029-T032) can interleave
Mermaid output (T033-T036) after graph building
CLI (T037-T039) after Mermaid output
```

**Phase 7 (Polish)**:
```
T056, T057, T058 can run in parallel (different concerns)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup → Project compiles
2. Complete Phase 2: Foundational → Core types ready
3. Complete Phase 3: User Story 1 → Single crate analysis works
4. **STOP and VALIDATE**: Run against `/Users/jeff/sample`, verify 7 edges
5. Deploy/demo if ready

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. User Story 1 → MVP: Single crate Mermaid output
3. User Story 2 → Cycles verified (likely already works)
4. User Story 3 → Re-export edge resolution verified
5. Workspace Support → Multi-crate analysis
6. Polish → Production ready

### Suggested First Session

Complete T001-T042 (Setup + Foundational + US1) to get a working MVP that can analyze `/Users/jeff/sample`.

---

## Summary

| Metric | Value |
|--------|-------|
| Total Tasks | 67 |
| Setup (Phase 1) | 7 |
| Foundational (Phase 2) | 10 |
| User Story 1 (Phase 3) | 25 |
| User Story 2 (Phase 4) | 3 |
| User Story 3 (Phase 5) | 4 |
| Workspace (Phase 6) | 6 |
| Polish (Phase 7) | 12 |

**MVP Scope**: T001-T042 (User Story 1 complete)

**All tasks follow checklist format**: ✅ Checkbox, Task ID, [P] marker where applicable, [Story] label for US phases, file paths included.

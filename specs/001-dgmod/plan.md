# Implementation Plan: dgmod (Rust Module Dependency Graph)

**Branch**: `001-dgmod` | **Date**: 2025-12-20 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-dgmod/spec.md`

## Summary

Build a Rust CLI tool that analyzes Rust source code to generate a directed graph of module dependencies. The tool parses `mod` declarations and `use` statements to create edges between modules, outputting the graph in Mermaid diagram syntax. Supports Cargo workspaces by producing separate graphs per crate member.

## Technical Context

**Language/Version**: Rust (stable toolchain, 2021 edition)
**Primary Dependencies**: `syn` (Rust parser), `clap` (CLI), `cargo_metadata` (workspace detection)
**Storage**: N/A (stateless, file-based input, stdout output)
**Testing**: `cargo test` for unit and integration tests
**Target Platform**: Cross-platform (macOS, Linux, Windows)
**Project Type**: Single CLI utility with library core
**Performance Goals**: Analyze 100-module crate in under 5 seconds (SC-003)
**Constraints**: Fail-fast on parse errors; stderr for diagnostics, stdout for graph
**Scale/Scope**: Single-crate or workspace analysis; no network I/O

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Rust-First Implementation | PASS | Tool implemented in Rust |
| II. Minimal Dependencies | REVIEW | `syn`, `clap`, `cargo_metadata` justified below |
| III. Maximum Portability | PASS | Cross-platform file I/O only, no platform APIs |
| IV. Correctness by Construction | PASS | Will use `Result` for errors, strong typing for paths |
| V. Library-First with CLI Exposure | PASS | Core logic in lib.rs, thin CLI wrapper in main.rs |

**Dependency Justifications**:
- `syn`: Required for parsing Rust syntax. No stdlib alternative. Industry standard for Rust tooling.
- `clap`: Required for CLI argument parsing. Could use manual parsing, but clap is maintained and minimal.
- `cargo_metadata`: Required for workspace member detection (FR-015). Reading Cargo.toml manually is error-prone.

## Project Structure

### Documentation (this feature)

```text
specs/001-dgmod/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
dgmod/
├── Cargo.toml           # Crate manifest
├── src/
│   ├── lib.rs           # Library entry: public API
│   ├── main.rs          # CLI wrapper: args, I/O
│   ├── parser.rs        # Rust source parsing (syn-based)
│   ├── graph.rs         # Graph data structure and edge deduplication
│   ├── resolver.rs      # Module path resolution (mod declarations, #[path])
│   ├── mermaid.rs       # Mermaid output formatting
│   └── workspace.rs     # Cargo workspace detection
└── tests/
    ├── integration/     # End-to-end tests against sample crates
    └── fixtures/        # Test crate fixtures
```

**Structure Decision**: Single project structure following Constitution Principle V (Library-First). The `dgmod/` directory will be a workspace member of the rust-kart repository.

## Complexity Tracking

No constitution violations requiring justification. All dependencies serve essential functions with no simpler alternative.

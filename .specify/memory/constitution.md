<!--
  Sync Impact Report
  ===================
  Version change: N/A → 1.0.0 (initial ratification)

  Added Principles:
  - I. Rust-First Implementation
  - II. Minimal Dependencies
  - III. Maximum Portability
  - IV. Correctness by Construction
  - V. Library-First with CLI Exposure

  Added Sections:
  - Technical Constraints
  - Development Workflow
  - Governance

  Removed Sections: N/A (initial creation)

  Templates Status:
  - .specify/templates/plan-template.md: ✅ compatible (uses generic gates)
  - .specify/templates/spec-template.md: ✅ compatible (technology-agnostic)
  - .specify/templates/tasks-template.md: ✅ compatible (path conventions support src/)
  - .specify/templates/agent-file-template.md: ✅ compatible (placeholder-based)
  - .specify/templates/checklist-template.md: ✅ compatible (generic structure)

  Follow-up TODOs: None
-->

# Rust Kart Constitution

## Core Principles

### I. Rust-First Implementation

All new utilities MUST be implemented in Rust unless a compelling technical reason
exists (e.g., shell script wrapper for bootstrapping). Rust provides the static
typing, memory safety, and performance guarantees that align with project goals.

**Rationale**: This repository exists specifically to hold Rust code. Using a
single language maximizes code reuse, simplifies tooling, and ensures consistent
quality across all utilities.

### II. Minimal Dependencies

External crates MUST be justified by significant value. Prefer standard library
solutions. When dependencies are necessary:

- Choose well-maintained crates with minimal transitive dependencies
- Avoid crates that pull in large dependency trees for small features
- Document the rationale for each non-trivial dependency in Cargo.toml comments

**Rationale**: Fewer dependencies mean faster builds, smaller binaries, reduced
attack surface, and easier auditing. Dependencies are a liability, not an asset.

### III. Maximum Portability

Utilities MUST target cross-platform compatibility where feasible:

- Avoid platform-specific APIs unless the utility is inherently platform-bound
- Use `cfg` attributes for necessary platform differences, not separate codebases
- Test on macOS, Linux, and Windows when CI permits
- Prefer POSIX-compatible behaviors for CLI tools

**Rationale**: Portable code serves more users and reduces maintenance burden.
Platform-specific code fragments the codebase and limits utility.

### IV. Correctness by Construction

Leverage Rust's type system to make invalid states unrepresentable:

- Use newtypes to distinguish semantically different values (e.g., `PathBuf` vs
  custom `ConfigPath` wrapper)
- Prefer `Option` and `Result` over sentinel values or panics
- Use enums to model state machines and exhaustive matching
- Avoid `unwrap()` in library code; reserve it for cases with proof of safety
- Write pure functions where possible; isolate side effects at boundaries

**Rationale**: Static guarantees eliminate entire classes of runtime bugs. Code
that cannot represent invalid states requires fewer tests and less documentation.

### V. Library-First with CLI Exposure

Each utility MUST be structured as a library with a thin CLI wrapper:

- Core logic lives in `src/lib.rs` (or submodules)
- CLI parsing and I/O live in `src/main.rs`
- Libraries MUST be independently testable without CLI invocation
- Text-based I/O: args/stdin for input, stdout for output, stderr for diagnostics

**Rationale**: Library-first design enables embedding, testing, and composition.
The CLI is a convenience interface, not the primary API.

## Technical Constraints

**Language**: Rust (stable toolchain, current edition)
**Build System**: Cargo with workspace structure
**Testing**: `cargo test` for unit and integration tests
**Formatting**: `cargo fmt` (default rustfmt configuration)
**Linting**: `cargo clippy` with warnings treated as errors in CI
**Documentation**: `cargo doc` for API documentation; README for usage

Utilities SHOULD:
- Compile with `--release` without warnings
- Pass `clippy::pedantic` where practical (document exceptions)
- Include `#![forbid(unsafe_code)]` unless unsafe is justified and audited

## Development Workflow

**Feature Development**:
1. Create feature branch from `main`
2. Write failing tests that capture requirements
3. Implement until tests pass
4. Run `cargo fmt`, `cargo clippy`, `cargo test`
5. Submit PR with clear description of changes

**Code Review Requirements**:
- All changes to `main` require review
- Reviewers verify constitution compliance
- New dependencies require explicit approval with documented justification

**Quality Gates**:
- CI MUST pass before merge
- No `TODO` or `FIXME` comments without linked issues
- Public APIs MUST have doc comments

## Governance

This constitution supersedes informal practices. All contributions MUST comply
with these principles. Amendments require:

1. Written proposal with rationale
2. Review period for feedback
3. Update to this document with version increment
4. Migration plan if existing code becomes non-compliant

**Versioning Policy**:
- MAJOR: Principle removal or incompatible redefinition
- MINOR: New principle or significant expansion of guidance
- PATCH: Clarifications, typos, non-semantic refinements

**Compliance**: PR reviewers MUST verify adherence. Non-compliant code requires
explicit exception documentation in the PR and, if recurring, amendment proposal.

**Version**: 1.0.0 | **Ratified**: 2025-12-13 | **Last Amended**: 2025-12-13

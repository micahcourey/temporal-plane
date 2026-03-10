# Mnemix Repository Scaffold Specification

**Status:** milestone 0 baseline
**Date:** March 8, 2026

This document freezes the initial repository scaffold for the first implementation phase.

The active agent coding guidance for this repo lives in [mnemix-coding-guidelines.md](mnemix-coding-guidelines.md).

## Goals

- establish a Rust-workspace-first repository shape
- keep the core product model separate from storage and CLI concerns
- leave room for Python bindings and adapters without making them first-class implementation owners
- keep the project aligned with [mnemix-roadmap.md](mnemix-roadmap.md), [mnemix-plan-v3.md](mnemix-plan-v3.md), and [lancedb-rust-sdk-agent-guide.md](lancedb-rust-sdk-agent-guide.md)

## Top-level layout

```text
mnemix/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── rustfmt.toml
├── clippy.toml
├── .editorconfig
├── .gitignore
├── LICENSE
├── README.md
├── CHANGELOG.md
├── deny.toml
├── AGENTS.md
├── .github/workflows/
├── crates/
│   ├── mnemix-core/
│   ├── mnemix-lancedb/
│   ├── mnemix-cli/
│   ├── mnemix-types/
│   └── mnemix-test-support/
├── python/
│   ├── pyproject.toml
│   ├── README.md
│   └── mnemix/
├── adapters/
│   └── ai-dx-toolkit/
├── examples/
│   ├── basic-recall/
│   ├── checkpoints/
│   └── import-export/
├── tests/
│   ├── integration/
│   ├── fixtures/
│   └── snapshots/
├── docs/
└── scripts/
```

## Crate boundaries

### `mnemix-core`

Owns product semantics only:

- memory/domain model
- query and recall abstractions
- retention/checkpoint types
- storage traits
- product-level errors

Must not depend on `lancedb` or CLI formatting concerns.

### `mnemix-lancedb`

Owns concrete storage integration:

- local database connection
- schema and migrations
- query translation
- version/tag plumbing
- future lower-level `lance` integration points

### `mnemix-cli`

Owns:

- command parsing
- human-readable output
- machine-readable JSON output
- terminal UX concerns

### `mnemix-types`

Small shared types crate for value objects and serialization-safe request/response contracts that may later be shared across bindings and adapters.

### `mnemix-test-support`

Non-production helpers for:

- deterministic fixtures
- temp store setup
- shared assertions
- snapshot formatting helpers

## Non-Rust scaffolds included now

### `python/`

Included as a placeholder for the first binding layer, but intentionally not populated with product logic in milestone 0.

### `adapters/ai-dx-toolkit/`

Included as a placeholder for the first host adapter, but intentionally deferred until later milestones.

### `examples/`

Created early so user-facing flows have a stable home once implementation starts.

## Initial module placeholder policy

Milestone 0 creates minimal placeholder modules for each crate to make the structure explicit without prematurely freezing internals.

The placeholder set should:

- mirror the roadmap shape closely enough to avoid redesign churn
- avoid fake implementations
- favor module docs and `todo!()`-free placeholders where possible
- compile cleanly under `cargo test`, `cargo clippy`, and `cargo doc`

## Workspace guardrails

The root workspace should enforce:

- stable Rust toolchain pinning
- formatting and lint configuration
- shared package metadata
- CI checks for fmt, clippy, test, doc, and dependency policy
- tracked Dex project instructions via [AGENTS.md](../AGENTS.md)

## Deferred until later milestones

The following are intentionally scaffolded but not implemented in milestone 0:

- LanceDB storage behavior
- domain types beyond placeholders
- real CLI commands
- Python FFI/binding code
- adapter logic
- branch workflows

## Acceptance summary

Milestone 0 scaffold is considered correctly defined when:

- the repository shape above exists in the repo
- crate responsibilities are explicit
- future milestones can add code without reorganizing the repo first
- the workspace remains aligned with Rust API and Rust best-practice guidance

# Temporal Plane

Temporal Plane is a standalone, local-first memory layer for AI coding agents and related tooling.

## Current status

This repository is in the planning and scaffolding phase.

The current canonical planning set is:

- [docs/temporal-plane-plan-v3.md](docs/temporal-plane-plan-v3.md)
- [docs/temporal-plane-roadmap.md](docs/temporal-plane-roadmap.md)
- [docs/lancedb-rust-sdk-agent-guide.md](docs/lancedb-rust-sdk-agent-guide.md)
- [docs/repo-scaffold-spec.md](docs/repo-scaffold-spec.md)

## Repository structure

- `crates/temporal-plane-core` — domain and product semantics
- `crates/temporal-plane-lancedb` — concrete storage backend integration
- `crates/temporal-plane-cli` — human-facing CLI surface
- `crates/temporal-plane-types` — shared value objects and request/response contracts
- `crates/temporal-plane-test-support` — non-production test helpers
- `python/` — first binding scaffold
- `adapters/` — host integration scaffolds

## Engineering baseline

The workspace is configured to support:

- stable Rust toolchain pinning
- formatting and Clippy checks
- cargo test and cargo doc validation
- dependency policy via `cargo-deny`
- persistent task tracking via Dex

## Task tracking

Persistent multi-step work is tracked with Dex. See [AGENTS.md](AGENTS.md) for project workflow instructions.

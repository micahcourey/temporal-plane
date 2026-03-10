# mnemix — Testing Strategy

## Overview

Testing is treated as part of the contract. The repo is still scaffold-stage, so most implemented validation is workspace-level rather than feature-level.

Coverage target for new code: `90%`.

## Current Tooling

| Tool | Purpose | Location |
|------|---------|----------|
| `cargo test` | workspace unit and integration tests | Rust workspace |
| `cargo fmt` | formatting gate | CI + local |
| `cargo clippy` | lint gate | CI + local |
| `cargo doc` | documentation build gate | CI + local |
| `cargo deny` | dependency/license/advisory checks | CI + local |

## Intended Test Pyramid

| Layer | Scope | Status |
|-------|-------|--------|
| Unit | domain invariants and helpers | expected in Milestone 1 |
| Integration | backend and cross-crate flows | expected in Milestone 2+ |
| Snapshot | CLI output stability | expected in Milestone 3+ |
| Binding tests | Python wrapper behavior | expected in Milestone 6 |

## Sources

- `.github/workflows/ci.yml`
- `docs/mnemix-coding-guidelines.md`
- `docs/mnemix-roadmap.md`

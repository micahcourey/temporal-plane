# mnemix — System Architecture

> Current architecture is scaffold-first. The storage model is planned in docs, but most runtime behavior is not implemented yet.

## Overview

Mnemix is a local-first Rust workspace intended to provide a reusable memory engine for coding agents. The repo is organized so that product semantics are defined before backend mechanics, and backend mechanics are defined before adapter or binding layers.

## Component Inventory

| Component | Type | Tech | Status | Purpose |
|-----------|------|------|--------|---------|
| `crates/mnemix-core` | library crate | Rust | scaffolded | product semantics, domain types, traits |
| `crates/mnemix-lancedb` | library crate | Rust + planned LanceDB | scaffolded | concrete local storage backend |
| `crates/mnemix-cli` | binary crate | Rust | scaffolded | human-first CLI surface |
| `crates/mnemix-types` | library crate | Rust | scaffolded | shared typed contracts |
| `crates/mnemix-test-support` | library crate | Rust | scaffolded | shared fixtures and helpers |
| `python/` | binding package | Python + Rust FFI later | placeholder | thin Python wrapper surface |
| `adapters/` | adapter area | mixed | placeholder | host-specific integrations |
| `.ai/` | AI toolkit output | Markdown, JSONL, YAML | generated | agent-facing instructions and context |

## Planned Data Flow

```text
host adapter / CLI / Python
        ↓
mnemix-core (domain contracts)
        ↓
backend capability traits
        ↓
mnemix-lancedb (storage translation)
        ↓
LanceDB / Lance datasets on local filesystem
```

## External Integration Points

| System | Type | Purpose | Status |
|--------|------|---------|--------|
| GitHub Actions | CI | fmt, clippy, test, doc, deny checks | active |
| Dex | local task tracker | persistent multi-step work tracking | active |
| LanceDB | storage library | local-first memory store | planned |
| Lance | dataset/version layer | versions, tags, advanced workflows | planned |

## Sources

- `docs/mnemix-plan-v3.md`
- `docs/mnemix-roadmap.md`
- `docs/lancedb-rust-sdk-agent-guide.md`
- `Cargo.toml`
- `README.md`

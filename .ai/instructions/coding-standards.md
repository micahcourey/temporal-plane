# Coding Standards

> Project-specific coding rules for Mnemix.

## Architectural Priorities

- Rust-first
- library-first
- local-first
- typed public APIs
- strict crate boundaries

## Boundary Rules

### `mnemix-core`

Owns product semantics only:

- domain types
- recall/search/history/stats requests and responses
- checkpoint and retention abstractions
- backend capability traits
- product-level errors

Must not expose:

- `lancedb` types
- `lance` types
- Arrow table/schema details
- CLI rendering concerns
- Python binding glue

### `mnemix-lancedb`

Owns backend mechanics:

- storage connections
- table/schema management
- query translation
- indexing
- version and tag plumbing

### `mnemix-cli`

Owns command parsing, human-readable output, JSON output mode, and binary-boundary error aggregation.

## Public API Rules

- Prefer domain types over raw `String`, `bool`, or loose option bags.
- Keep public fields private unless the type is intentionally passive data.
- Use builders for complex construction.
- Document public APIs and meaningful examples.
- Avoid leaking backend-specific details into stable contracts.

## Error Handling

- Use typed library errors, preferably via `thiserror`.
- Use `anyhow` only at CLI or application boundaries.
- Avoid stringly typed public error surfaces.
- Avoid `unwrap()` and `expect()` outside tests or impossible states.

## Validation Requirement

Finish substantive work with:

```bash
./scripts/check.sh
```

## Sources

- `docs/mnemix-coding-guidelines.md`
- `docs/mnemix-roadmap.md`
- `docs/mnemix-plan-v3.md`

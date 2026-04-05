---
status: open
summary: A slim read-only `mnemix ui` slice is implemented with recent, pinned, and search browsing, explicit `from` / `to` date fields, keyboard-first detail inspection, and a shared core browse contract.
updated: 2026-03-31
---

# Status

This workstream is the repo-native planning and execution container for adding a
human-facing TUI to `mnemix`.

The first implementation slice is now in place as a slim, read-only
`mnemix ui` experience. It gives humans a keyboard-first way to browse and
inspect the memory store without replacing the existing static CLI.

## Implemented

- Added a new interactive `mnemix ui` command in `mnemix-cli`
- Added a ratatui + crossterm terminal runtime with clean enter/exit handling
- Implemented browse modes for recent memories, pinned memories, and text search
- Implemented separate `from` / `to` date fields for search filtering using
  explicit `YYYY-MM-DD` validation
- Implemented scope filtering, results navigation, and scrollable memory detail
  inspection
- Promoted recent and pinned browse/list behavior behind a shared
  storage-agnostic `BrowseBackend` contract in `mnemix-core`
- Implemented the shared browse contract in the LanceDB backend and updated CLI
  product surfaces to consume the trait instead of LanceDB-specific helpers
- Documented the new command in the main `README.md`

## Verified

- `cargo test -p mnemix-cli`
- `cargo test -p mnemix-core -p mnemix-lancedb -p mnemix-cli`
- `./scripts/check.sh`
- Manual PTY smoke test of `mnemix ui` against a temporary initialized store

## Remaining Follow-Up

- Add broader empty-state and error-path validation coverage as the TUI grows
- Evaluate whether later releases should support operator actions such as
  pinning, unpinning, or restore flows

The remaining backlog is organized around four implementation areas:

- establishing the product-safe browse/list contract the TUI needs
- adding the interactive CLI entrypoint and terminal runtime
- implementing recent, pinned, and search-driven memory browsing
- rendering full memory details readably with clear keyboard navigation

The `mnemix-workflow` TUI is the main product precedent for architecture and
interaction style, but this workstream is intentionally focused on browsing an
agent's memory store rather than workflow artifacts.

## References

- `mnemix-workflow/workflow/workstreams/005-interactive-tui-mode/spec.md`
- `crates/mnemix-cli/src/cli.rs`
- `crates/mnemix-core/src/traits.rs`
- `crates/mnemix-lancedb/src/backend.rs`

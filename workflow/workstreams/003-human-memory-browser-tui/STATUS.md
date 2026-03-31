---
status: open
summary: Human memory-browser TUI planning is complete and the implementation backlog is organized around browse modes, search, detail inspection, and CLI runtime work.
updated: 2026-03-30
---

# Status

This workstream is the repo-native planning and execution container for adding a
human-facing TUI to `mnemix`.

The initial planning slice is complete in the form of this workstream. The
remaining backlog is organized around four implementation areas:

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
- `crates/mnemix-lancedb/src/backend.rs`

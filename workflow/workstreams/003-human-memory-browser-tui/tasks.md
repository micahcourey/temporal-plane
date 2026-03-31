# Tasks: Human Memory Browser Tui

## Workstream Goal

Create a focused human-facing TUI for `mnemix` that makes it easy to browse,
search, and inspect a local memory store without replacing the existing static
CLI.

## Execution Slices

### Planning And Scope

- [x] Create a repo-native workstream for the human memory browser TUI
- [ ] Confirm the v1 command shape and naming, such as `mnemix ui`
- [ ] Decide whether recent-memory browsing requires a new shared core trait or
  can reuse existing boundaries cleanly

### Data Access And Contracts

- [ ] Add any storage-agnostic browse/list contract needed for recent-memory
  browsing
- [ ] Implement the shared browse/list behavior in the LanceDB backend if
  promoted into product surfaces
- [ ] Define the memory-detail payload the TUI needs for summary, detail, tags,
  pin state, and source metadata

### Interactive CLI Surface

- [ ] Add the interactive CLI entrypoint and terminal lifecycle handling
- [ ] Add pane focus, selection, and keyboard navigation state
- [ ] Add empty-store, invalid-store, and missing-scope handling

### Browse, Search, And Detail UX

- [ ] Implement recent-memory browsing
- [ ] Implement pinned-memory browsing
- [ ] Implement text search entry and result navigation
- [ ] Implement separate `from` / `to` date fields for search filtering
- [ ] Implement the full memory-detail preview with scrolling

### Documentation And Verification

- [ ] Document the TUI command, keybindings, and v1 scope
- [ ] Add targeted tests for browse modes, search flow, and detail rendering
- [ ] Run the relevant CLI and repo verification gates

## Validation Checklist

- [ ] Keep `STATUS.md` aligned with the true execution state
- [ ] Verify the TUI remains keyboard-first and understandable without color
  alone
- [ ] Verify date-filtered search works through explicit date fields rather than
  fuzzy natural-language parsing
- [ ] Verify scope filtering and empty-state behavior explicitly
- [ ] Preserve the existing static CLI behavior for scripting and automation

## Notes

- This workstream is intentionally browse-first. Inline editing, pin/unpin
  actions, restore flows, and other operator mutations can follow once the
  inspection surface is stable.
- The `mnemix-workflow` TUI is a useful implementation precedent, but `mnemix`
  needs memory-store-specific browse and detail semantics rather than workflow
  artifact semantics.

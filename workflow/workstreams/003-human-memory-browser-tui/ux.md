# UX Spec: Human Memory Browser Tui

## Summary

The TUI should feel like a calm memory-inspection cockpit: a human can move
through recent, pinned, and search-driven memory views from the keyboard, then
read full memory details without bouncing between separate commands.

## Users And Context

- Primary persona: a developer or operator inspecting an agent's local memory
  store during coding, debugging, review, or release work
- Context of use: a terminal session with access to a local `.mnemix` store
- Preconditions: the store exists and contains zero or more memories

## User Goals

- Browse memories even when they do not already know the exact search query
- Run text search and inspect results without leaving the interactive session
- Pivot between scopes and pinned context quickly
- Read complete memory details and metadata from the keyboard alone

## Experience Principles

- Browse first, search intentionally
- Keep the store legible at a glance
- Use a small number of memorable keyboard interactions
- Favor stable, readable terminal behavior over ambitious polish

## Primary Journey

1. The user runs `mnemix ui` against a local store.
2. The TUI opens in a browse-first mode such as Recent.
3. The user filters by scope or switches to Pinned.
4. The user moves through the memory list and selects a record.
5. The detail pane shows the record's title, summary, detail, and metadata.
6. The user opens Search, enters a query, optionally sets `from` / `to` date
   fields, and browses matching results.
7. The user exits back to the normal CLI when finished.

## Alternate Flows

### Flow: Empty Store

- Trigger: the selected store contains no memories
- Path: the TUI shows a clear empty state and points to `mnemix remember`
- Expected outcome: the user understands why the UI is empty and what to do
  next

### Flow: Invalid Store Path

- Trigger: the user launches the TUI against a missing or invalid store
- Path: the command fails clearly before entering the full-screen UI, or the UI
  presents a focused error state if runtime validation is deferred
- Expected outcome: the user gets an actionable error instead of a broken
  layout

### Flow: Scope With No Results

- Trigger: the user selects a scope or search that returns no matching memories
- Path: the list pane shows a no-results state without clearing the current mode
- Expected outcome: the user understands that the filter worked but matched
  nothing

## Surfaces

### Surface: Mode And Scope Pane

- Purpose: choose between Recent, Pinned, and Search flows and switch scope
  filters
- Key information: active mode, current scope, and result counts when
  available
- Available actions: move selection, change mode, clear or change scope
- Navigation expectations: switching context should feel immediate and obvious

### Surface: Memory List Pane

- Purpose: show the memories for the current browse mode and scope
- Key information: title, kind, scope, timestamp, and high-signal summary
- Available actions: move selection, trigger search results, inspect the
  current record
- Navigation expectations: fast up/down movement with clear selection state

### Surface: Detail Preview Pane

- Purpose: show the selected memory in full
- Key information: title, summary, detail, tags, importance/confidence, pin
  state, timestamps, and source metadata
- Available actions: scroll, move focus, optionally switch metadata sections if
  the layout needs tabs
- Navigation expectations: readability matters more than decorative rendering

### Surface: Footer / Help Bar

- Purpose: teach the core keybindings and show the active mode
- Key information: focus hints, quit action, search action, and scope behavior
- Available actions: none beyond orientation
- Navigation expectations: users should not need to memorize the whole key map

## States

### Loading

- Startup should be quick, but the UI may show a short loading or initializing
  state when opening a large store

### Empty

- The empty state should explain that the store has no memories yet and point to
  `mnemix remember`

### Success

- The user can move through browse modes, inspect results, and read details
  reliably

### Error

- Errors should be explicit and local to the affected action or pane when
  possible

## Interaction Details

- Input behavior: support arrow keys and `j/k` style movement where sensible
- Search behavior: keep query entry simple, such as a focused search prompt or
  compact input mode, with separate explicit date fields rather than fuzzy
  natural-language date parsing
- Keyboard behavior: support fully keyboard-driven navigation with a visible
  quit action such as `q`
- Feedback: selection, focus, active mode, and current scope should always be
  obvious
- Responsive behavior: prefer graceful pane compression or a clear minimum-size
  warning over rendering garbage

## Content And Tone

- Labels should be short and operational, such as `Recent`, `Pinned`, `Search`,
  `Scope`, `Detail`, and `Tags`
- Messages should be clear, calm, and practical rather than chatty

## Accessibility Requirements

- The full v1 flow must work without a mouse
- Focus and selection must not rely on color alone
- Text should remain readable in common terminal and screen-reader setups

## Acceptance Scenarios

```gherkin
Scenario: Browse recent memories in the TUI
  Given a local store contains multiple memories
  When the user launches mnemix ui
  Then the user should be able to browse recent memories
  And the detail pane should update for the selected record
```

```gherkin
Scenario: Inspect pinned memories in one place
  Given a local store contains pinned memories
  When the user switches to the pinned mode
  Then the list should update to show pinned memories
  And the user should be able to inspect their full details
```

```gherkin
Scenario: Search and inspect matching memories
  Given a local store contains memories matching a query
  When the user enters a search query in the TUI
  Then the result list should update to matching memories
  And the selected result should show its full metadata and detail
```

```gherkin
Scenario: Narrow search results by date range
  Given a local store contains memories across multiple dates
  When the user enters a search query and sets explicit from/to date fields
  Then the result list should only show memories inside that date range
  And the UI should not require fuzzy natural-language date parsing
```

```gherkin
Scenario: Handle an empty memory store
  Given the selected store contains no memories
  When the user launches mnemix ui
  Then the TUI should show an empty state
  And it should point the user toward remembering the first memory
```

## References

- `mnemix-workflow/workflow/workstreams/005-interactive-tui-mode/ux.md`
- `crates/mnemix-cli/src/cmd/pins.rs`
- `crates/mnemix-cli/src/cmd/search.rs`
- `crates/mnemix-cli/src/cmd/show.rs`

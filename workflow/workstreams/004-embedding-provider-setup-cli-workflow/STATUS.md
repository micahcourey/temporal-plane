---
status: open
summary: Provider-profile setup, store-compatibility validation, provider-derived vector enablement, provider-backed backfill, semantic or hybrid CLI retrieval, and updated operator docs are implemented in the vector-capable worktree.
updated: 2026-04-04
---

# Status

This workstream is the repo-native planning container for closing the current
CLI gap between vector-ready stores and actual runtime embedding execution.

The current `mnemix` vector surface can persist store settings, inspect
coverage, and plan backfills, but the shipped CLI still does not expose a
provider setup workflow. This workstream scopes the next step: machine-local
provider configuration plus CLI execution paths for real semantic use.

## Implemented

- Added a first `providers` CLI command group in the vector-capable
  `tui-vector-support` worktree
- Added machine-local TOML persistence for provider profiles outside `.mnemix`
- Added redacted provider-profile list and show output for both human and JSON
  rendering
- Added `set-cloud`, `set-local`, and `remove` profile commands
- Added `providers validate` using an OpenAI-compatible embeddings probe to
  resolve runtime model and dimensions
- Added store-compatibility reporting to `providers validate`, including
  vectors-disabled, matched, and mismatch states against the current store
- Enabled `vectors show --provider <NAME>` so CLI status can reflect an attached
  runtime provider
- Enabled `vectors enable --provider <NAME>` so store vector settings can be
  derived directly from a validated provider profile
- Enabled `vectors backfill --apply --provider <NAME>` when a compatible
  provider is configured
- Added `search --mode <lexical|semantic|hybrid> [--provider <NAME>]`
  retrieval so semantic and hybrid CLI search can run with explicit provider
  selection
- Added `recall --mode <lexical|semantic|hybrid> [--provider <NAME>]`
  retrieval so semantic and hybrid CLI recall can run with explicit provider
  selection
- Added retrieval provenance to CLI output, including retrieval mode, provider
  name, and per-result semantic or hybrid search-match details
- Added explicit provider/store mismatch errors before provider-backed search,
  recall, or backfill begin
- Added targeted `mnemix-cli` tests for provider profile flows, provider
  requirements, and semantic or hybrid retrieval execution
- Added targeted regression tests for cloud secret-missing errors, local
  runtime-unreachable errors, and keeping provider config out of exported store
  payloads
- Updated the README plus CLI and vector-retrieval guides in the
  `tui-vector-support` worktree to document cloud and local setup paths along
  with provider-backed retrieval

## Verified

- `cargo test -p mnemix-cli` in the `tui-vector-support` worktree
- `./scripts/check.sh` in the `tui-vector-support` worktree
- `mxw validate` in the root `mnemix` checkout after normalizing the older 003
  workstream status metadata to the supported `open` value

## Planned Outcomes

- Add a supported cloud provider setup path
- Add a supported local model setup path
- Add safe machine-local provider profile storage
- Enable `vectors backfill --apply` through the CLI when a compatible provider
  is available
- Run broader verification beyond the focused CLI test suite
- Tighten any remaining provider setup ergonomics or compatibility diagnostics
  discovered during wider validation

## Not Started Yet

- Broader repo verification and any follow-up cleanup that falls out of it

## Dependencies

- The optional vector retrieval layer in
  `workflow/workstreams/002-optional-lancedb-vector-retrieval-layer/`
  established the backend provider interface and store-level vector state this
  workstream will build on.

## References

- `README.md`
- `docs_site/src/guide/vector-retrieval.md`
- `crates/mnemix-lancedb/src/backend.rs`
- `workflow/workstreams/002-optional-lancedb-vector-retrieval-layer/plan.md`

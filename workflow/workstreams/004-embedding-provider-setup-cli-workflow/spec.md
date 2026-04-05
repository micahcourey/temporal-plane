# Feature Spec: Embedding Provider Setup Cli Workflow

## Summary

Add a first-party CLI workflow for configuring runtime embedding providers so
`mnemix` can move from vector-ready store metadata to end-to-end semantic use.
The workflow should support both a cloud-hosted embedding provider and a local
model path, while keeping secrets machine-local and store exports portable.

## Problem

The current vector layer in `mnemix` can persist store-level vector settings,
report embedding coverage, and plan backfills, but the shipped CLI still does
not expose a concrete embedding provider. That leaves users with an awkward
gap:

- they can enable vectors for a store
- they can see that no runtime provider is attached
- they cannot complete `vectors backfill --apply`
- they cannot use semantic-only or hybrid retrieval from the CLI

This is understandable once you know how the backend works, but the product
still lacks a supported operator workflow for answering a simple question:
"How do I connect `mnemix` to an embedding model so semantic retrieval actually
works?"

Users also need two different provider stories:

- a cloud path for hosted embeddings
- a local path for self-hosted or on-device embeddings

Those paths should feel like variations of one product workflow rather than two
unrelated implementations.

## Users

- Primary persona: a developer or operator enabling semantic retrieval for a
  local `mnemix` store from the terminal
- Secondary persona: a coding-agent user who wants a reproducible local or
  hosted embedding setup without writing custom runtime glue

## Goals

- Add a CLI-native setup flow for runtime embedding providers
- Support one cloud provider path and one local model path in v1
- Keep provider credentials out of the store and out of exports
- Make provider compatibility with store vector settings explicit before writes
  or retrieval run
- Let the CLI progress from dry-run vector planning to real backfill and
  semantic or hybrid retrieval when a provider is configured

## Non-Goals

- Supporting every embedding vendor in v1
- Training, downloading, or fine-tuning embedding models
- Storing raw API keys or tokens inside the `.mnemix` store
- Replacing existing store-scoped vector metadata with machine-local runtime
  config
- Hiding network or local-runtime failures behind silent lexical fallback

## User Value

Users get a clear supported path from "vectors are enabled" to "semantic
retrieval actually works here." They can choose a hosted provider or a local
model, validate the setup, backfill embeddings, and use semantic capabilities
without building their own wrapper around the Rust backend.

## Functional Requirements

- The CLI should expose a provider configuration surface for machine-local
  embedding runtime setup
- The setup flow should support at least two provider kinds in v1:
  - a cloud-hosted embedding provider
  - a local model provider
- The provider workflow should let a user:
  - create or update a named provider profile
  - inspect the saved non-secret configuration
  - validate connectivity and model compatibility
  - select a provider explicitly for commands that require embeddings
- The store should continue to persist vector settings such as model,
  dimensions, and auto-embed intent separately from provider credentials
- The CLI should provide a guided path from provider validation to:
  - `vectors enable`
  - `vectors backfill --apply`
  - semantic-only or hybrid `search`
  - semantic-only or hybrid `recall`
- Secret material must not be printed in normal output, JSON output, logs, or
  exported stores
- The product should distinguish:
  - store vector readiness
  - provider profile readiness
  - active runtime compatibility between the two
- Error output should make mismatches explicit, such as:
  - missing API key source
  - unreachable local model runtime
  - provider model mismatch with the store
  - embedding dimension mismatch with the store

## Constraints

- Keep `mnemix-core` storage-agnostic and focused on product capabilities rather
  than vendor-specific provider details
- Preserve the current local-first, inspectable store contract
- Treat provider configuration as machine-local runtime state, not store data
- Keep the cloud and local flows parallel enough that docs and automation can
  teach one mental model
- Avoid any workflow that would commit secrets into repo files by default

## Success Criteria

- A user can configure a supported cloud embedding profile through the CLI and
  validate it successfully
- A user can configure a supported local embedding profile through the CLI and
  validate it successfully
- A user can connect a validated provider profile to a vector-enabled store and
  run `vectors backfill --apply`
- A user can run semantic-only or hybrid retrieval from the CLI with clear
  provenance and explicit failure behavior when a provider is missing or
  incompatible
- Provider setup is documented clearly enough that the README no longer needs
  to frame runtime embeddings as an intentionally unsupported CLI gap

## Risks

- Credential handling could accidentally leak secrets into config files, logs,
  or JSON output
- Provider-specific behavior could sprawl into the public core product surface
- Local model setup could become too environment-specific if the workflow
  assumes one machine layout too strongly
- Network-backed providers could introduce latency or intermittent failures that
  make semantic behavior feel flaky without good diagnostics

## References

- `README.md`
- `docs_site/src/guide/vector-retrieval.md`
- `docs_site/src/guide/cli.md`
- `crates/mnemix-lancedb/src/backend.rs`
- `workflow/workstreams/002-optional-lancedb-vector-retrieval-layer/spec.md`

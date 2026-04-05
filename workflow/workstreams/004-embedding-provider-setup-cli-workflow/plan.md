# Plan: Embedding Provider Setup Cli Workflow

## Summary

Add a machine-local provider profile layer plus a store-facing setup flow so
the `mnemix` CLI can configure, validate, and use embedding providers for both
hosted and local models. The design should keep store metadata portable while
making secrets and runtime endpoints explicit local concerns.

## Scope Analysis

### Affected Areas

| Area | Changes Required |
|------|-----------------|
| `mnemix-core` | Add product-level retrieval and command contracts only where CLI semantic and hybrid execution now becomes user-visible |
| Provider runtime crate or module | Add concrete provider profile types, secret-source handling, validation helpers, and provider factories |
| `mnemix-lancedb` | Reuse the existing `EmbeddingProvider` trait and runtime-open path; add any compatibility helpers needed for profile-driven setup |
| `mnemix-cli` | Add provider setup, inspection, validation, and command-time provider selection surfaces |
| Config persistence | Add machine-local profile storage that excludes secrets and survives normal local CLI use |
| Docs | Document cloud and local setup paths, credential expectations, validation, and error states |
| Tests | Cover profile persistence, secret redaction, provider validation, runtime mismatch handling, and end-to-end CLI flows |

### Affected Layers

- [x] Documentation and planning artifacts
- [ ] Provider profile data model
- [ ] Machine-local config persistence
- [ ] Concrete cloud provider implementation
- [ ] Concrete local model provider implementation
- [ ] CLI setup and inspection commands
- [ ] CLI semantic retrieval and backfill wiring
- [ ] Tests and verification

## Technical Design

### Current Baseline

Today `mnemix` already has:

- store-level vector settings through `vectors enable`
- readiness and coverage reporting through `vectors show`
- apply-capable backend support when a runtime provider is attached
- explicit provider and dimension mismatch validation in the LanceDB backend

Today it still lacks:

- a shipped provider implementation in the CLI
- machine-local provider configuration
- a supported CLI path for semantic or hybrid retrieval
- a secure setup story for cloud credentials versus local model endpoints

### Proposed Runtime Model

Keep provider setup in two layers:

1. Machine-local provider profiles
2. Store-level vector configuration

The machine-local layer answers:

- which provider kind to use
- which model to call
- where credentials or local endpoints come from
- which profile name a human or script should reference

The store-level layer continues to answer:

- whether vectors are enabled for this store
- what model id and dimensions the store expects
- whether auto-embed-on-write is intended
- how much embedding coverage already exists

This split preserves exportability and store portability while still letting
the CLI open the backend with a concrete runtime provider.

### Provider Scope For V1

Support two concrete paths first:

- Cloud path: a hosted embedding provider over HTTP
- Local path: a machine-local model runtime reachable from the CLI

The cloud path should be shaped to support a first-party hosted provider flow
without hard-coding secrets into the store. The local path should support a
practical self-hosted flow such as a locally running model server.

If naming must become concrete in implementation, prefer one provider per path
instead of adding a half-finished abstraction layer for many vendors.

### Config And Secret Strategy

- Persist non-secret profile configuration in a user-scoped config file or
  equivalent machine-local config location
- Store only references to secrets, such as environment variable names or
  keychain aliases, not the secret values themselves
- Redact secret-bearing fields from normal CLI output and JSON output
- Keep store exports and staged imports free of machine-local provider secrets

### CLI Workflow Shape

The operator workflow should feel linear:

1. Create or update a provider profile
2. Validate the provider against the selected model
3. Enable store vectors from that validated provider or verify an existing
   store matches it
4. Run `vectors backfill --apply`
5. Use semantic-only or hybrid retrieval explicitly

The exact command names can be settled during implementation, but the product
should likely expose:

- provider profile management
- provider validation
- provider-aware vector enablement
- provider-aware semantic search and recall

### Proposed Additions

```text
crates/mnemix-core/src/                            # any minimal user-visible query or command contract additions
crates/mnemix-lancedb/src/backend.rs               # provider compatibility and open-time wiring reuse
crates/mnemix-cli/src/cli.rs                       # new provider-related command group and retrieval flags
crates/mnemix-cli/src/cmd/providers.rs             # profile CRUD, validation, and inspection
crates/mnemix-cli/src/cmd/vectors.rs               # apply-mode backfill and provider-aware enablement
crates/mnemix-cli/src/cmd/search.rs                # semantic / hybrid mode and provider selection
crates/mnemix-cli/src/cmd/recall.rs                # semantic / hybrid mode and provider selection
crates/mnemix-cli/src/config/                      # machine-local provider profile persistence
crates/mnemix-embeddings/ or equivalent            # concrete cloud and local provider implementations
docs_site/src/guide/                               # setup and operational docs
workflow/workstreams/004-embedding-provider-setup-cli-workflow/
```

### Design Constraints

- Do not persist secrets inside `.mnemix`
- Do not silently degrade semantic-only requests into lexical behavior
- Keep local and cloud setup flows structurally similar where possible
- Prefer explicit validation before destructive or expensive operations such as
  large embedding backfills
- Keep provider implementation details outside `mnemix-core` unless they become
  product-level capability contracts

## Implementation Slices

### Slice 1: Workstream, Profiles, And Secrets

- Create the durable repo-native workstream for provider setup
- Define the machine-local provider profile format
- Decide how secret references are represented safely
- Define JSON redaction and non-secret inspection rules

### Slice 2: Provider Implementations

- Add one cloud embedding provider implementation
- Add one local embedding provider implementation
- Add validation helpers that resolve model id and embedding dimensions
- Add compatibility checks between validated providers and store settings

### Slice 3: CLI Setup Workflow

- Add CLI commands to create, update, list, show, and remove provider profiles
- Add a validation command that exercises the provider without mutating store
  data
- Add provider-aware `vectors enable` flow or adjacent setup command

### Slice 4: Backfill And Retrieval Execution

- Enable `vectors backfill --apply` when a compatible provider is available
- Add semantic-only and hybrid flags to CLI `search`
- Add semantic-only and hybrid flags to CLI `recall`
- Surface clear provenance and explicit failure states

### Slice 5: Docs And Verification

- Update the README and vector retrieval docs with cloud and local workflows
- Add CLI help and examples for both provider paths
- Add targeted tests and run repo gates before handoff

## Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Secrets leak into config or logs | High | Medium | Store only secret references, redact outputs, and test JSON plus text rendering |
| Provider abstraction becomes over-engineered | Medium | Medium | Limit v1 to one cloud path and one local path with a shared profile contract |
| Local model setup varies widely by environment | Medium | High | Keep the local path endpoint-driven and validate runtime reachability explicitly |
| CLI semantics drift from backend capabilities | High | Medium | Reuse backend provider validation and keep semantic-mode behavior explicit in tests |
| Large backfills fail midway without clear operator guidance | Medium | Medium | Add resumable-friendly status output and preflight validation before apply mode |

## References

- `README.md`
- `docs_site/src/guide/cli.md`
- `docs_site/src/guide/vector-retrieval.md`
- `crates/mnemix-lancedb/src/backend.rs`
- `crates/mnemix-core/src/query.rs`
- `workflow/workstreams/002-optional-lancedb-vector-retrieval-layer/plan.md`

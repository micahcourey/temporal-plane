# UX Spec: Embedding Provider Setup Cli Workflow

## Summary

Embedding setup should feel like a guided operator workflow instead of a hidden
backend trick. A user should be able to choose a cloud provider or local model,
validate the setup, and then use semantic features with confidence about what
is configured where.

## Users And Context

- Primary persona: a developer enabling semantic retrieval on a local machine
- Secondary persona: an operator preparing a portable store that should work
  with either hosted or local embeddings
- Context of use: terminal-first workflows, local shell environments, and
  machine-local config rather than app-server deployment systems

## User Goals

- Understand the difference between store vector settings and runtime provider
  setup
- Configure a cloud-hosted embedding provider without leaking secrets
- Configure a local model provider without guessing which fields matter
- Validate a provider before launching a large backfill
- Run semantic or hybrid retrieval only when the runtime is actually ready

## Experience Principles

- Keep setup explicit and inspectable
- Keep secrets invisible by default
- Make cloud and local setup feel like the same product pattern
- Fail loudly on mismatches instead of pretending semantic behavior happened
- Separate machine-local provider state from portable store state

## Primary Journey

1. A user chooses a provider path: cloud or local.
2. The user creates a named provider profile with non-secret settings plus a
   secret source reference if needed.
3. The user validates the provider and sees the resolved model id and
   dimensions.
4. The user connects that validated provider to store vector setup or confirms
   the store already matches it.
5. The user runs embedding backfill or semantic retrieval with explicit
   provider selection.

## Alternate Flows

### Flow: Existing Store, New Provider

- Trigger: a store already has vector metadata but no current runtime provider
- Path: the user validates a provider profile and the CLI confirms whether the
  provider matches the store's model and dimensions
- Expected outcome: a mismatch is reported clearly before any write or search
  begins

### Flow: Local Runtime Unavailable

- Trigger: the user configures a local model profile but the local service is
  not running
- Path: validation fails with a connectivity or health-style error
- Expected outcome: the user understands that the profile is configured but not
  currently usable

### Flow: Missing Cloud Secret

- Trigger: the user configures a hosted provider profile that references a
  missing API key source
- Path: validation fails before any backfill or search work is attempted
- Expected outcome: the error identifies the missing secret source without
  echoing secret values

## Surfaces

### Surface: Provider Profiles

- Purpose: create, inspect, update, and remove machine-local provider setup
- Key information: provider kind, profile name, model, endpoint, secret source
  type, validation state, and last successful validation time when available
- Available actions: add, show, list, edit, remove, validate
- Navigation expectations: this surface should be scriptable and readable in
  plain terminal output

### Surface: Vector Setup

- Purpose: connect a validated provider to store-level vector enablement
- Key information: provider model, provider dimensions, current store model,
  current store dimensions, auto-embed intent, and mismatch warnings
- Available actions: enable, verify compatibility, backfill apply
- Navigation expectations: the user should not need to inspect backend tables
  to understand readiness

### Surface: Semantic Retrieval

- Purpose: run semantic-only or hybrid retrieval intentionally
- Key information: retrieval mode, provider profile used, result provenance, and
  degraded or failed execution state
- Available actions: run `search` or `recall` with explicit mode and provider
  selection
- Navigation expectations: lexical mode remains available as a stable baseline

## States

### Unconfigured

- No provider profiles exist on the current machine

### Configured But Unvalidated

- A provider profile exists, but the user has not confirmed the model and
  runtime are reachable

### Validated

- The provider profile can resolve model id and dimensions successfully

### Store Mismatch

- The provider is usable, but its model or dimensions do not match the store

### Ready

- The provider is validated and compatible with the store, so backfill and
  semantic retrieval can run

### Error

- Secret source failure, local runtime failure, network failure, or dimension
  mismatch is reported explicitly

## Interaction Details

- Command help should make the machine-local versus store-local split explicit
- Validation output should report resolved model id and dimensions without
  printing secrets
- Provider inspection output should be useful even when the profile is not
  currently valid
- Retrieval output should tell the user whether semantic matching actually ran

## Content And Tone

- Prefer direct wording such as "provider validated", "store mismatch", "API
  key source missing", or "local embedding runtime unreachable"
- Avoid phrases that imply semantic support is active before compatibility has
  been checked

## Accessibility Requirements

- Output must remain understandable without color cues
- Secret redaction should not rely on visual masking alone; the text output
  should clearly omit or label redacted values
- Error output should use stable plain-language terms that wrappers can surface

## Acceptance Scenarios

```gherkin
Scenario: A user validates a hosted embedding profile safely
  Given a user configured a cloud provider profile with a non-secret endpoint
  And the profile references an API key source
  When the user runs provider validation
  Then the CLI should confirm whether the provider is reachable
  And report the resolved model id and dimensions
  And never print the secret value
```

```gherkin
Scenario: A user validates a local model runtime before backfill
  Given a user configured a local provider profile
  When the local runtime is unavailable
  Then validation should fail explicitly
  And `vectors backfill --apply` should not start
```

```gherkin
Scenario: Semantic retrieval uses a compatible validated provider
  Given a store has vector settings that match a validated provider profile
  When the user runs semantic search
  Then the CLI should run semantic retrieval
  And the result should identify semantic or hybrid provenance clearly
```

## References

- `README.md`
- `docs_site/src/guide/cli.md`
- `docs_site/src/guide/vector-retrieval.md`
- `crates/mnemix-lancedb/src/backend.rs`

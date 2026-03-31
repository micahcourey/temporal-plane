# UX Spec: Optional Lancedb Vector Retrieval Layer

## Summary

Vector retrieval should feel like a safe enhancement to existing recall and
search flows, not a new opaque mode that forces users to abandon lexical
behavior or guess whether vector support is available.

## Users And Context

- Primary persona: developers and coding agents querying a local Mnemix store
- Context of use: CLI and Python search/recall flows, plus operator maintenance
  tasks around enablement, backfill, and readiness inspection
- Preconditions: a store may or may not have vectors enabled, embeddings may be
  partially populated, and providers may or may not be configured

## User Goals

- Ask for lexical, semantic, or hybrid retrieval intentionally
- Understand whether vector support is available for the current store
- Understand whether a result came from lexical, semantic, or hybrid matching
- Enable and maintain vector support without risking silent data loss or
  confusing state

## Experience Principles

- Lexical behavior remains the trusted baseline
- Optional features should degrade clearly, not mysteriously
- Match provenance should stay visible and explainable
- Maintenance workflows should be explicit and checkpoint-friendly

## Primary Journey

1. An operator enables vectors or checks vector readiness for a store.
2. A caller requests lexical, semantic, or hybrid search/recall.
3. The backend uses the best available mode and surfaces clear provenance.
4. The operator can inspect readiness, coverage, and index state when behavior
   does not match expectations.

## Alternate Flows

### Flow: Hybrid Fallback

- Trigger: a caller requests hybrid retrieval on a store without usable vector
  support
- Path: the system degrades safely to lexical behavior where supported
- Expected outcome: the caller still receives usable results and can inspect why

### Flow: Semantic Strictness

- Trigger: a caller requests semantic-only retrieval
- Path: the system returns semantic results only when the store actually
  supports them
- Expected outcome: unsupported semantic-only requests fail explicitly rather
  than silently pretending to be semantic

### Flow: Store Enablement

- Trigger: an operator enables vectors or starts embedding backfill
- Path: the system validates configuration, persists enablement state, and
  reports readiness and coverage
- Expected outcome: maintenance is resumable, explicit, and safe

## Surfaces

### Surface: Search And Recall

- Purpose: expose lexical, semantic, and hybrid retrieval modes
- Key information: retrieval mode, match provenance, degraded fallback, and
  ranking explanation
- Available actions: choose retrieval mode, inspect results, inspect
  explanations
- Navigation expectations: CLI and Python should mirror the same product
  semantics

### Surface: Vector Maintenance

- Purpose: enable vectors, backfill embeddings, and inspect readiness
- Key information: coverage, provider state, index availability, and store mode
- Available actions: enable, backfill, inspect status, refresh operational
  state
- Navigation expectations: workflows should remain explicit instead of being
  hidden behind writes

## States

### Disabled

- The store uses lexical behavior only

### Partial Readiness

- The store has some vector configuration or embeddings but is not fully ready

### Ready

- Semantic and hybrid retrieval are available with clear explanation metadata

### Error

- Provider mismatch, index absence, or schema mismatch is reported explicitly

## Interaction Details

- Retrieval modes must use stable names such as lexical, semantic, and hybrid
- Readiness output should distinguish semantic availability from vector-index
  availability
- Backfill and enablement output should be specific enough to support resumed
  maintenance

## Content And Tone

- Prefer direct, operational wording such as "vector index unavailable" or
  "semantic retrieval disabled"
- Avoid overpromising semantic quality when the store is only partially ready

## Accessibility Requirements

- CLI output must remain understandable in plain text without color cues
- Match provenance should be readable by screen readers and wrappers as normal
  text

## Acceptance Scenarios

```gherkin
Scenario: Hybrid retrieval explains semantic support clearly
  Given a store has vectors enabled and usable embeddings
  When a caller runs hybrid recall
  Then the result should identify hybrid or semantic match provenance
  And lexical-only behavior should remain available as an explicit mode
```

```gherkin
Scenario: A partially enabled store reports readiness instead of hiding it
  Given a store has vector config but incomplete embeddings or index support
  When an operator inspects vector status
  Then the output should distinguish partial readiness from full availability
```

## References

- GitHub issue `#75`
- GitHub issue `#88`
- `.plans/lancedb-vector-retrieval-implementation-plan.md`

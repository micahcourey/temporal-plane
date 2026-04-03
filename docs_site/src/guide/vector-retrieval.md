# Vector Retrieval

Mnemix does not require vectors to be useful. The default user path is still local lexical search plus layered recall. Vector support is optional, store-scoped, and designed to let a store grow into semantic or hybrid retrieval without changing the core memory model.

## What ships today

The current vector layer includes:

- persisted store-level vector settings
- persisted embeddings alongside memory records
- coverage and readiness reporting through the CLI
- backend support for semantic-only and hybrid retrieval when an embedding provider is attached
- export and staged import flows that preserve vector state instead of dropping it

This means a store can carry its vector configuration and embeddings across clone, export, and import workflows even if the current client is only using lexical retrieval.

## Current CLI workflow

### Inspect vector status

```bash
mnemix --store .mnemix vectors show
```

`vectors show` reports:

- whether vectors are enabled for the store
- whether the current runtime has an embedding provider attached
- whether auto-embed-on-write can run right now
- how many memories already have persisted embeddings
- whether the store shape is ready for future LanceDB-native vector indexing

This is the fastest way to answer "is this store vector-ready yet?" without inspecting internal tables.

### Enable vectors for a store

```bash
mnemix --store .mnemix vectors enable \
  --model my-embedding-model \
  --dimensions 1536
```

This persists the embedding model identifier and embedding dimensionality as store metadata. You can also record the intent to embed new writes automatically:

```bash
mnemix --store .mnemix vectors enable \
  --model my-embedding-model \
  --dimensions 1536 \
  --auto-embed-on-write
```

That flag only becomes operational when the backend is opened with an embedding provider. The shipped CLI does not currently attach one on its own, so this setting is best understood as persisted store configuration rather than immediate CLI behavior.

### Plan a backfill

```bash
mnemix --store .mnemix vectors backfill
```

This command is currently a dry-run planner. It reports how many memories are candidates for embedding backfill and only counts memories that are still missing persisted embeddings.

The shipped CLI does not yet support applying that plan directly:

```bash
mnemix --store .mnemix vectors backfill --apply
```

That path returns an explicit unsupported error because the CLI binary does not currently expose an embedding provider.

## Retrieval behavior today

There are two different layers to keep straight:

- The LanceDB backend now supports lexical, semantic-only, and hybrid retrieval modes.
- The shipped `mnemix` CLI still exposes lexical `search` and `recall` commands.

That means vector enablement is already useful for store portability, coverage tracking, and backend/API consumers, but the human-facing CLI has not yet grown a `search --mode semantic` or `recall --mode hybrid` surface.

## Portability and import flows

Vector state now survives the workflows users expect to be lossless:

- exported stores keep vector settings and persisted embeddings
- staged imports preserve embeddings for imported memories
- vector coverage reporting reflects memories that actually have embeddings in the current store

This matters because vector readiness is a store property, not just an in-memory runtime toggle.

## Relationship to storage

Mnemix persists vector state in the LanceDB-backed store because that keeps the feature aligned with the project's local-first design:

- embeddings stay inspectable and portable with the rest of the store
- staged import and restore workflows can preserve vector metadata
- future indexing work can build on the same persisted shape rather than introducing a parallel sidecar system

For the storage-level framing behind that design, see [Storage Foundation](/guide/lancedb).

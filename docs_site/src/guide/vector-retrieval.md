# Vector Retrieval

Mnemix does not require vectors to be useful. The default user path is still local lexical search plus layered recall. Vector support is optional, store-scoped, and designed to let a store grow into semantic or hybrid retrieval without changing the core memory model.

## What ships today

The current vector layer includes:

- persisted store-level vector settings
- persisted embeddings alongside memory records
- machine-local provider profiles for cloud and local embedding runtimes
- coverage and readiness reporting through the CLI
- top-level semantic-only and hybrid CLI retrieval when a compatible embedding provider is attached
- export and staged import flows that preserve vector state instead of dropping it

This means a store can carry its vector configuration and embeddings across clone, export, and import workflows even if the current client is only using lexical retrieval.

## Current CLI workflow

### Configure a provider profile

Cloud profile:

```bash
mnemix providers set-cloud \
  --name openai \
  --model text-embedding-3-small \
  --base-url https://api.openai.com/v1 \
  --api-key-env OPENAI_API_KEY
```

Local profile:

```bash
mnemix providers set-local \
  --name ollama \
  --model nomic-embed-text \
  --endpoint http://127.0.0.1:11434/v1
```

Provider profiles are machine-local. They keep endpoints, model ids, and secret-source references outside the store itself.

### Validate a provider and compare it to a store

```bash
mnemix --store .mnemix providers validate --name openai
```

`providers validate` now reports:

- the resolved model id
- the resolved embedding dimensions
- whether the current store is uninitialized, vectors-disabled, matched, or mismatched

That makes it easier to answer "is this provider usable here?" before backfill or retrieval starts.

### Inspect vector status

```bash
mnemix --store .mnemix vectors show --provider openai
```

`vectors show` reports:

- whether vectors are enabled for the store
- whether the current runtime has an embedding provider attached
- the resolved provider model and dimensions when `--provider` is used
- whether the provider matches the current store settings
- whether auto-embed-on-write can run right now
- how many memories already have persisted embeddings
- whether the store shape is ready for future LanceDB-native vector indexing

This is the fastest way to answer "is this store vector-ready yet?" without inspecting internal tables.

### Enable vectors for a store

```bash
mnemix --store .mnemix vectors enable \
  --provider openai
```

This persists the provider's resolved embedding model identifier and embedding dimensionality as store metadata. You can still set the values manually if you need to:

```bash
mnemix --store .mnemix vectors enable \
  --model my-embedding-model \
  --dimensions 1536
```

You can also record the intent to embed new writes automatically:

```bash
mnemix --store .mnemix vectors enable \
  --provider openai \
  --auto-embed-on-write
```

That flag becomes operational when the current command opens the store with a compatible provider profile.

### Plan a backfill

```bash
mnemix --store .mnemix vectors backfill
```

This command remains a dry-run planner when `--apply` is omitted. It reports how many memories are candidates for embedding backfill and only counts memories that are still missing persisted embeddings.

To execute the backfill:

```bash
mnemix --store .mnemix vectors backfill --apply --provider openai
```

That path now fails explicitly if the provider is missing or if the provider does not match the store's configured vector settings.

## Retrieval behavior today

Lexical retrieval is still the default baseline, but the CLI now exposes all three retrieval modes when you select a provider explicitly:

```bash
mnemix --store .mnemix search \
  --text "storage decision" \
  --scope repo:mnemix \
  --mode semantic \
  --provider openai
```

```bash
mnemix --store .mnemix recall \
  --text "architecture" \
  --scope repo:mnemix \
  --mode hybrid \
  --provider openai
```

The CLI output now tells you:

- which retrieval mode ran
- which provider profile was used
- whether a search hit was lexical, semantic, or hybrid
- the semantic score when one was available

There are still two layers to keep straight:

- The LanceDB backend now supports lexical, semantic-only, and hybrid retrieval modes.
- The shipped `mnemix` CLI now exposes semantic and hybrid top-level `search` and `recall`, but only when a compatible provider profile is selected.
- `mnemix ui` surfaces vector status plus lexical, semantic, and hybrid mode selection, but semantic and hybrid modes still depend on a runtime that opens the store with an embedding provider.

That means vector enablement is useful both for store portability and for immediate end-to-end semantic execution from the shipped CLI.

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

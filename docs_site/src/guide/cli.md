# CLI

The `mnemix` CLI is the primary interface for working with a local store from the terminal. It is designed for two modes:

- human-readable output for day-to-day inspection
- structured `--json` output for scripting and higher-level clients

## Command overview

```bash
mnemix [--store PATH] [--json] <command>
```

Available commands:

- `ui`
- `init`
- `remember`
- `recall`
- `search`
- `show`
- `pins`
- `history`
- `checkpoint`
- `versions`
- `restore`
- `optimize`
- `providers`
- `vectors`
- `stats`
- `export`
- `import`
- `policy`

The default store path is `.mnemix`.

## Common workflow

### Initialize a store

```bash
mnemix --store .mnemix init
```

### Save a memory

```bash
mnemix --store .mnemix remember \
  --id memory:storage-decision \
  --scope repo:mnemix \
  --kind decision \
  --title "Use LanceDB as the local store" \
  --summary "LanceDB provides local persistence, search, and version history." \
  --detail "The initial implementation uses LanceDB for the primary storage path." \
  --importance 90 \
  --confidence 95 \
  --tag architecture \
  --tag storage \
  --pin-reason "Core project decision"
```

### Recall layered context

```bash
mnemix --store .mnemix recall --scope repo:mnemix
```

By default, `recall` uses `summary-then-pinned` disclosure depth. You can also set:

- `summary-only`
- `summary-then-pinned`
- `full`

### Search the archive

```bash
mnemix --store .mnemix search \
  --text "storage decision" \
  --scope repo:mnemix \
  --limit 10
```

To run semantic or hybrid retrieval, attach an explicit provider profile:

```bash
mnemix --store .mnemix search \
  --text "storage decision" \
  --scope repo:mnemix \
  --mode semantic \
  --provider openai
```

### Inspect a single memory

```bash
mnemix --store .mnemix show --id memory:storage-decision
```

### List pinned memories

```bash
mnemix --store .mnemix pins --scope repo:mnemix --limit 20
```

## History and recovery

### Create a checkpoint

```bash
mnemix --store .mnemix checkpoint \
  --name before-refactor \
  --description "Stable state before reorganizing the workspace"
```

### List versions

```bash
mnemix --store .mnemix versions --limit 20
```

### Restore by checkpoint or version

```bash
mnemix --store .mnemix restore --checkpoint before-refactor
```

```bash
mnemix --store .mnemix restore --version 12
```

`restore` creates a new current head from the selected historical state. It does not delete prior history.

## Maintenance and portability

### Inspect store statistics

```bash
mnemix --store .mnemix stats --scope repo:mnemix
```

### Inspect vector status

```bash
mnemix --store .mnemix vectors show
```

This reports whether vectors are enabled, whether the current runtime can embed new writes, embedding coverage, and whether the store is ready for a future LanceDB-native vector index.

### Configure provider profiles

Cloud provider:

```bash
mnemix providers set-cloud \
  --name openai \
  --model text-embedding-3-small \
  --base-url https://api.openai.com/v1 \
  --api-key-env OPENAI_API_KEY
```

Local provider:

```bash
mnemix providers set-local \
  --name ollama \
  --model nomic-embed-text \
  --endpoint http://127.0.0.1:11434/v1
```

Validate a provider and compare it against the current store:

```bash
mnemix --store .mnemix providers validate --name openai
```

### Enable vector settings

```bash
mnemix --store .mnemix vectors enable \
  --provider openai
```

You can still set model and dimensions manually:

```bash
mnemix --store .mnemix vectors enable \
  --model my-embedding-model \
  --dimensions 1536
```

To persist the intent to embed new writes automatically when a provider is available:

```bash
mnemix --store .mnemix vectors enable \
  --provider openai \
  --auto-embed-on-write
```

### Plan embedding backfill

```bash
mnemix --store .mnemix vectors backfill
```

This remains the dry-run planner. To apply the backfill:

```bash
mnemix --store .mnemix vectors backfill --apply --provider openai
```

### Optimize the store

```bash
mnemix --store .mnemix optimize --older-than-days 30
```

To allow pruning of older versions:

```bash
mnemix --store .mnemix optimize --prune --older-than-days 30
```

### Export a store

```bash
mnemix --store .mnemix export --destination ./backups/mnemix-export
```

### Import a store

```bash
mnemix --store .mnemix import --source ./backups/mnemix-export
```

Imports are staged onto an isolated branch so the current main store remains unchanged until the staged data is reviewed. Export and staged import preserve vector settings and persisted embeddings when they are present in the source store.

## Interactive UI

The `mnemix ui` command opens an interactive terminal interface for browsing and searching memories. It is designed for human-first exploration and provides a browse-first view of recent, pinned, and search-driven memory inspection.

```bash
mnemix --store .mnemix ui
```

### Key features

- **Browse-first view**: See recent and pinned memories immediately.
- **Search-driven**: Refine the view with text-based search directly in the TUI.
- **Vector-aware status**: Inspect store vector readiness, coverage, provider availability, and index status without leaving the TUI.
- **Retrieval-mode selector**: Move between lexical, semantic, and hybrid search modes and see unavailable modes called out explicitly.
- **Keyboard-first**: Navigate with arrows, filter with keys, and inspect details without leaving the interface.
- **Time filters**: Use explicit from/to date filters to narrow down your inspection.

### Options

| Option | Description |
|---|---|
| `--limit <LIMIT>` | Maximum number of memories to load [default: 200] |
| `--store <PATH>` | Path to the local store [default: .mnemix] |

## JSON mode

Pass `--json` to receive a stable machine-readable response envelope:

```bash
mnemix --store .mnemix --json stats
```

Successful commands return structured data under a `kind` and `data` envelope. Failures return a structured `error` payload on stderr. The Python client uses this interface directly.

## Command notes

- `history` and `versions` are inspection commands for store history.
- `checkpoint` creates a stable, human-readable reference to the current version.
- `restore` always requires exactly one target: `--checkpoint` or `--version`.
- `optimize` is conservative by default and only prunes old versions when `--prune` is set.
- Top-level `search` and `recall` support lexical, semantic, and hybrid retrieval. Semantic and hybrid modes require `--provider <NAME>`.
- `mnemix ui` surfaces vector readiness and retrieval-mode availability, but semantic and hybrid execution still require a runtime that opens the store with an embedding provider.
- `vectors show` is the main inspection command for vector readiness, coverage, and provider availability.
- `providers validate` reports resolved provider dimensions plus whether the current store matches that provider.
- `vectors enable` can derive store model and dimensions directly from `--provider <NAME>`.
- `vectors backfill --apply` requires `--provider <NAME>` and fails explicitly on provider/store mismatches.
- `remember` supports tags, entities, metadata, and source attribution fields for richer recall and inspection.

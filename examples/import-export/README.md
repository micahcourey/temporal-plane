# Import / Export Example

This example shows the current v1 semantics for copying and staging stores.

- `export` creates a copy of the current store at another path
- `import` stages another store onto an isolated branch
- `import` does not change the main branch of the destination store

That behavior is intentional: import is a safe rehearsal step, not an implicit
merge.

## Export Walkthrough

Create a source store and write a memory:

```bash
mnemix --store .mnemix-source init

mnemix --store .mnemix-source remember \
  --id memory:exported-decision \
  --scope repo:mnemix \
  --kind decision \
  --title "Export demo memory" \
  --summary "This memory exists in the source store before export." \
  --detail "The export command copies the store so another workflow can inspect or archive it." \
  --importance 80
```

Export the store:

```bash
mnemix --store .mnemix-source export --destination /tmp/mnemix-export-copy
```

At this point `/tmp/mnemix-export-copy` is a full copy of the source store.

## Import Walkthrough

Initialize a clean destination store:

```bash
mnemix --store .mnemix-destination init
```

Stage the exported store into the destination:

```bash
mnemix --store .mnemix-destination import --source /tmp/mnemix-export-copy
```

The command reports the branch name used for staging and the number of staged
records. The destination store's main branch remains unchanged.

If you want to inspect staged branches in more detail, see
[`examples/branch-experiment/README.md`](/Users/micah/Projects/mnemix/.worktrees/codex-mnemix-examples/examples/branch-experiment/README.md).

## Recommended Use Cases

Use `export` for:

- backups
- handoff copies
- offline inspection
- archival snapshots

Use `import` for:

- rehearsing a migration from another store
- testing whether an external memory set is useful
- bringing foreign data into an isolated branch before review

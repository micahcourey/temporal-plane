# Branch experiment example

This example demonstrates the advanced storage workflow added in Milestone 7:

1. initialize a local store
2. write a baseline memory on `main`
3. create a second source store
4. stage that source store onto an experimental branch
5. inspect visible branches
6. confirm the default branch remains unchanged

Run it with:

```text
cargo run -p mnemix-lancedb --example branch-experiment
```

The example uses the public `AdvancedStorageBackend` trait and keeps all branch behavior marked as advanced.

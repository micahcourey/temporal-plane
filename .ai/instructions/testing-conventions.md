# Testing Conventions

## Required Repo Gate

Finish substantive work with:

```bash
./scripts/check.sh
```

## Current Validation Stack

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo doc --workspace --no-deps`
- `cargo deny check`

## Test Layers

- unit tests for domain invariants
- integration tests for cross-crate and backend flows
- snapshot tests for CLI output when CLI behavior lands
- doc examples when public APIs become meaningful

## Test Design Rules

- use deterministic fixtures
- keep one primary behavior per test
- prefer descriptive names
- place shared helpers in `mnemix-test-support`

## Sources

- `docs/mnemix-coding-guidelines.md`
- `.github/workflows/ci.yml`
- `scripts/check.sh`

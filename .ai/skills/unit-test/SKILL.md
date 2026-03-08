---
name: unit-test
description: Generate and improve Rust 2024 crate tests and Python binding tests for Temporal Plane with deterministic fixtures, typed API coverage, and 80%+ coverage goals.
---

# Unit Test Generator

## Context Files

Consult [AGENTS.md](../../AGENTS.md) for the project knowledge routing table.
**For this skill, prioritize**: testing conventions for repo gates, coding standards for Rust-first boundaries, testing strategy for layer expectations, and system architecture when crate boundaries matter.

## When This Skill Activates

This skill activates when:
- creating or editing Rust tests (`*_test.rs`, `tests/*.rs`, inline `mod tests` blocks)
- creating or editing Python tests (`test_*.py`, `*_test.py`)
- adding coverage for public library APIs, CLI behavior, or Python wrappers
- discussing regressions, invariants, fixtures, snapshots, or verification strategy

## Project Testing Priorities

Temporal Plane is **Rust-first**.

- Rust tests validate product semantics and remain the source of truth.
- Python tests validate wrapper behavior and parity with Rust-backed semantics.
- CLI snapshot tests matter once human-facing output is involved.
- All substantive work still finishes with the repo gate:

```bash
./scripts/check.sh
```

## Instructions

### Step 1: Analyze the Code Under Test

Before generating tests:
1. Read the full source file or module.
2. Identify the public contract: types, constructors, builders, traits, and errors.
3. Map happy paths, edge cases, invariants, and failure paths.
4. Check crate boundaries so tests do not pull backend details into core semantics.
5. Determine the correct test layer:
   - inline/unit tests for local invariants
   - integration tests for cross-crate or backend flows
   - snapshot tests for CLI rendering
   - Python tests for binding/wrapper behavior

### Step 2: Choose the Right Test Style

| Surface | Preferred Style | Notes |
|--------|------------------|-------|
| `temporal-plane-core` | focused unit tests close to domain types | test invariants, typed construction, error cases |
| cross-crate workflows | `tests/` integration tests | verify public behavior only |
| CLI output | snapshot-style tests | keep command execution separate from rendering |
| Python bindings | `pytest` tests | verify wrapper semantics, not duplicated business logic |

### Step 3: Generate Rust Tests

Use Rust test conventions that fit this repo:

- Prefer `#[cfg(test)] mod tests` for local invariants.
- Prefer `tests/*.rs` for public integration behavior.
- Use descriptive snake_case test names.
- Use typed fixtures and builders instead of loose maps or stringly setup.
- Assert on full typed outcomes where practical.
- Cover typed errors with specific matching instead of generic string contains when possible.
- Keep tests deterministic: fixed timestamps, fixed IDs, stable ordering, no hidden randomness.
- Avoid unnecessary cloning and avoid testing private implementation trivia.
- Do not introduce backend-specific types into core tests.

### Rust Patterns

```rust
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn builder_rejects_empty_checkpoint_name() {
    let result = CheckpointName::try_from("");

    assert!(result.is_err());
  }

  #[test]
  fn retention_policy_defaults_are_conservative() {
    let policy = RetentionPolicy::default();

    assert_eq!(policy, RetentionPolicy::KeepAll);
  }
}
```

```rust
use temporal_plane_core::ScopeId;

#[test]
fn search_query_preserves_scope_filter() {
  let scope = ScopeId::try_from("repo:temporal-plane").unwrap();
  let query = SearchQuery::builder()
    .scope(scope.clone())
    .text("checkpoint")
    .build()
    .unwrap();

  assert_eq!(query.scope(), Some(&scope));
}
```

### Rust Anti-Patterns

```rust
#[test]
fn test_stuff() {
  let result = make_query(true, false, "x", "y", 123);
  assert!(result.is_ok());
}
```

Avoid:
- vague names like `test_stuff`
- boolean-flag setup that hides semantics
- asserting only `is_ok()` when the returned value matters
- `unwrap()` in assertions when a clearer expectation can be written first

### Step 4: Generate Python Tests

When testing Python code in this repo:

- use `pytest`
- test public wrapper behavior only
- keep fixtures explicit and readable
- use `pathlib.Path` and `tmp_path` for filesystem tests
- prefer plain asserts with clear values
- use parametrization for small behavioral matrices
- mock only true boundaries such as subprocess, filesystem, or FFI adapter seams
- do not recreate Rust business logic in Python helpers

### Python Patterns

```python
from pathlib import Path


def test_open_store_uses_explicit_path(tmp_path: Path) -> None:
  store_path = tmp_path / "store"

  client = TemporalPlane.open(store_path)

  assert client.path == store_path
```

```python
import pytest


@pytest.mark.parametrize(
  ("value", "expected"),
  [
    ("repo:temporal-plane", "repo:temporal-plane"),
    ("branch:main", "branch:main"),
  ],
)
def test_scope_id_round_trips(value: str, expected: str) -> None:
  scope = ScopeId(value)

  assert str(scope) == expected
```

### Python Anti-Patterns

```python
def test_everything():
  result = helper(True, False, {}, [])
  assert result
```

Avoid:
- opaque fixtures and flag-heavy helpers
- duplicating Rust-side validation logic in Python
- broad mocks that hide wrapper integration problems
- filesystem tests using hard-coded local paths

## Scenario Coverage Checklist

Systematically cover these categories when relevant:

| Category | What to Test | Priority |
|----------|--------------|----------|
| Happy path | expected valid construction and use | Critical |
| Invariants | typed IDs, value objects, builder rules | Critical |
| Error handling | typed failures and invalid inputs | Critical |
| Boundary behavior | crate or wrapper seams | High |
| Determinism | ordering, stable rendering, repeatable output | High |
| Serialization | request/response stability where applicable | Medium |
| Filesystem/temp data | path handling, import/export staging | Medium |
| Cleanup | temp dirs, handles, process cleanup | Medium |

## Test Design Rules

- Prefer one primary behavior per test.
- Prefer descriptive names over comments.
- Use shared helpers only when they improve clarity.
- Put reusable Rust helpers in `temporal-plane-test-support` when cross-crate reuse is justified.
- Keep doc examples aligned with tested behavior when public API examples exist.
- For CLI work, protect human-readable output with snapshots only after rendering is stable.

## Verification Expectations

For Rust-focused changes, run the narrowest useful step during iteration, then finish with:

```bash
cargo test --workspace
./scripts/check.sh
```

For Python-focused wrapper work, add or run the relevant `pytest` target as part of iteration, then still finish with the repo gate that applies to the workspace state.

## Checklist

- [ ] Tests cover the public contract, not just implementation details
- [ ] Domain invariants and typed errors are exercised
- [ ] Fixtures are deterministic and readable
- [ ] Core tests do not leak LanceDB or backend internals
- [ ] Python tests validate wrapper behavior without duplicating Rust logic
- [ ] Cross-crate helpers are extracted only when reuse is real
- [ ] Snapshot tests are used only for stable human-facing output
- [ ] Final verification includes `./scripts/check.sh`

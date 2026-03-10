# Mnemix Coding Guidelines

**Status:** active project guidance  
**Audience:** agents and contributors working in this repository

This document distills the project-relevant rules from the temporary reference material that was imported during planning.

## 1. Primary rule

Mnemix is a **Rust-first, library-first, local-first** system.

That means:

- Rust is the source of truth for product behavior.
- Python is a binding layer, not a second implementation.
- Adapters translate host concepts into Mnemix concepts; they do not redefine product semantics.
- The core crate must stay independent from backend-specific implementation details.

## 2. Architecture boundaries are non-negotiable

### `mnemix-core`

Owns product semantics only:

- domain types
- memory and scope concepts
- recall/search/history/stats request and response types
- checkpoint and retention abstractions
- storage capability traits
- product-level errors

`mnemix-core` must not expose or depend on:

- `lancedb` types
- `lance` types
- Arrow schema/table details
- CLI rendering concerns
- Python binding concerns

### `mnemix-lancedb`

Owns backend details:

- table/schema wiring
- query translation
- indexing behavior
- version/tag plumbing
- backend-specific serialization and migration details
- future `lance` advanced operations

### `mnemix-cli`

Owns:

- argument parsing
- human-readable output
- JSON output mode
- terminal-facing UX

### Python bindings and adapters

Must wrap stable Rust behavior.

They must not:

- duplicate core logic
- expose storage internals as product API
- force callers to understand LanceDB mechanics

## 3. Public Rust API rules

### Model semantics with types

Prefer explicit types over vague primitives.

Use domain types such as:

- `MemoryId`
- `ScopeId`
- `CheckpointName`
- `RetentionPolicy`
- request structs for recall/search/history/stats

Avoid public APIs built around:

- raw `String` when a domain type is clearer
- boolean switches with unclear meaning
- long parameter lists
- loosely structured option bags

### Construction rules

- Use builders for complex values or multi-step configuration.
- Keep public struct fields private unless the type is intentionally passive data.
- Use inherent methods when a receiver is obvious.
- Use standard conversions: `From`, `TryFrom`, `AsRef`.

### Trait expectations

Public types should implement relevant standard traits where meaningful:

- `Debug` always
- `Clone`, `Eq`, `PartialEq`, `Hash`, `Default` when semantically correct
- `Serialize` and `Deserialize` for stable portable types where appropriate
- `Display` only when there is a clear human-facing representation

### Future-proofing

- Prefer private fields on public structs.
- Use sealed traits or private extension points when downstream implementation should not be supported.
- Do not leak unstable backend details into stable public contracts.

## 4. Error handling rules

- Library crates use typed errors, preferably via `thiserror`.
- `anyhow` is allowed only at binary or app-boundary layers.
- Do not use stringly typed public errors.
- Do not use `unwrap` or `expect` outside tests or truly impossible states.
- Error messages should be concise, actionable, and inspectable.

Public docs should include failure behavior where relevant:

- `# Errors`
- `# Panics`
- `# Safety`

## 5. Ownership and performance rules

- Borrow by default; do not clone unless ownership is required by the contract.
- Prefer `&str`, slices, iterators, and generic inputs over unnecessary owned containers.
- Avoid redundant allocations and unnecessary materialization.
- Prefer static dispatch by default.
- Use dynamic dispatch only when it is clearly justified by the abstraction boundary.
- Measure before optimizing, but treat gratuitous cloning or backend materialization in foundational paths as a design smell.

## 6. Testing and verification rules

Tests are part of the contract.

### Required coverage expectations

- unit tests for domain invariants and edge cases
- integration tests for multi-crate and backend workflows
- snapshot tests for CLI and human-facing output
- doc examples when public APIs have meaningful usage patterns

### Test quality rules

- Prefer deterministic fixtures.
- Prefer one behavior per test.
- Use descriptive test names.
- Put shared helpers in `mnemix-test-support`.

### Required verification before considering work done

Run:

```bash
./scripts/check.sh
```

If that is too broad for an intermediate step, run the narrowest relevant subset first, but do not finish work without the full repo gate.

## 7. Documentation and comments rules

- Crate docs must explain purpose and boundaries.
- Module docs must explain intent when the boundary matters.
- Public APIs should include examples when usage is not trivial.
- Examples should prefer `?` over `unwrap`.
- Comments should explain **why**, not restate **what**.

Good reasons for comments:

- storage quirks
- invariants
- safety requirements
- performance tradeoffs
- compatibility constraints

Bad reasons for comments:

- repeating obvious code
- narrating straightforward control flow
- leaving stale implementation notes instead of updating the code

## 8. Dependency and CI hygiene

- Keep the public dependency surface minimal.
- Add dependencies only when they clearly simplify the design or reduce risk.
- Do not leak `lancedb` or `lance` types into stable core APIs.
- Respect workspace lint policy.
- Do not suppress lints casually.
- If a lint override is necessary, make it local and explain why.
- Keep fmt, clippy, tests, docs, and deny checks green.

## 9. Python binding rules

When Python work begins:

- keep Python APIs thin and stable
- expose product operations, not backend internals
- preserve Rust semantics
- use explicit exceptions and clean request/response boundaries
- prefer modern packaging and `pathlib`-style path handling

If implementing a Python feature would require duplicating Rust business logic, the boundary is wrong and should be redesigned.

## 10. LanceDB boundary rules

- Core traits speak in Mnemix vocabulary.
- The LanceDB crate performs translation to storage-engine concepts.
- Versioning is a product feature, but storage-engine mechanics remain internal unless deliberately surfaced as product concepts.
- Design for future branch-aware behavior internally without forcing branch mechanics into v1 ergonomics.

## 11. Agent workflow checklist

Before making changes:

1. read the relevant planning docs
2. preserve crate boundaries
3. make the smallest correct change
4. update docs/tests when behavior changes
5. verify with the required checks

Before merging a change, ask:

- Does this leak backend details into `mnemix-core`?
- Does this introduce ambiguous flags or raw strings where a type is better?
- Are errors typed and documented?
- Are tests and docs updated with the behavior?
- Does the change still respect the Rust-first architecture?

## 12. Repo-specific defaults

For this repository, default to:

- typed library APIs
- strict crate boundaries
- minimal dependencies
- full validation before completion
- docs that explain product meaning, not just signatures
- storage details hidden behind backend layers

If there is tension between convenience and architectural clarity, choose architectural clarity.

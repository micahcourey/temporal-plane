# Changelog

All notable changes to Temporal Plane will be documented in this file.

The format is inspired by Keep a Changelog and semantic versioning.

## [Unreleased]

## [0.1.0] - 2026-03-09

### Added

- **Workspace and engineering baseline** (Milestone 0) — Rust workspace scaffold, toolchain pinning, `rustfmt`/`clippy`/`cargo-deny` policy, CI workflows, and helper scripts (`scripts/check.sh`, `scripts/release.sh`).
- **Core domain contract** (Milestone 1) — typed domain IDs, memory record model, recall/search/history/stats query types, checkpoint and retention types, `StorageBackend` trait, and full unit test coverage for domain invariants. No storage dependencies in the core crate.
- **Local LanceDB backend MVP** (Milestone 2) — persistent local storage via LanceDB: init/open flows, schema versioning, `remember`/`get`/`search`/`history`/`stats`/`checkpoint` support, FTS indexing, import/export skeletons, and integration test coverage.
- **Human-first CLI** (Milestone 3) — `init`, `remember`, `search`, `show`, `pins`, `history`, `checkpoint`, `versions`, `stats`, `export`, and `import` commands. Human-readable and stable `--json` machine-readable output modes. Snapshot-safe output rendering separated from command execution.
- **Progressive disclosure and pinning semantics** (Milestone 4) — explicit `pin`/`unpin` support, layered `recall` returning `pinned_context`, `summaries`, and `archival` tiers, retrieval explanation metadata, and ranking by recency, importance, and pinned state.
- **Version-aware safety features** (Milestone 5) — historical inspection APIs, `restore` command (creates new head state, does not mutate history), pre-import and pre-optimize auto-checkpoint policy, retention configuration types, `optimize` command with conservative cleanup, and tag/checkpoint protection from routine cleanup.
- **Python binding and AI DX Toolkit adapter** (Milestone 6) — `temporal-plane` Python package (`TemporalPlane` client, typed models, explicit exceptions, `pathlib`-first store handling), subprocess-based CLI JSON binding, AI DX Toolkit adapter proof of concept, 68 tests, and usage examples.
- **Advanced storage workflows** (Milestone 7) — branch domain types (`BranchName`, `BranchRecord`, `BranchStatus`), `AdvancedStorageBackend` trait, import staging via Lance branches, shallow and deep clone, `BackendCapability` guards, branch-experiment runnable example, and `docs/branch-lifecycle.md`.
- **Release packaging** — `python/temporal_plane/_version.py` as canonical version source, tightened `pyproject.toml` metadata, `twine check` CI validation step, `scripts/check-python-package.sh` local pre-publish script, and `docs/pypi-release-plan.md`.

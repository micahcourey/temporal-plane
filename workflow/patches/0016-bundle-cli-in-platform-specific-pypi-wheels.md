---
status: completed
summary: "Bundled platform-specific Python wheels merged on main in PR #25."
updated: "2026-03-10"
---

# Patch: Bundle CLI in platform-specific PyPI wheels

## Summary

Prepare Mnemix for a lower-friction Python install by publishing platform-specific PyPI wheels that bundle the Rust mnemix CLI binary. Scope: define supported target platforms, update Python packaging to include bundled binaries, prefer bundled binary lookup at runtime, expand GitHub Actions to build/test/publish wheels per platform, and document fallback behavior for unsupported platforms. Acceptance criteria: pip install mnemix works without a separate CLI install on supported platforms; MNEMIX_BINARY and PATH fallback still work; trusted publishing uploads per-platform wheels plus sdist; docs reflect the new installation model.

## Reason

This patch preserves legacy Dex history after the repository moved to repo-native workflow tracking with `mnemix-workflow`.

## Scope

- Preserve the original Dex task details for `a7565nnm`.
- Keep the tracked status aligned with the final Dex state: `completed`.
- Avoid reinterpreting the historical task beyond the recorded Dex description and outcome.

## Implementation Notes

- Imported from legacy Dex tracking during the 2026-03-30 workflow migration.
- Original Dex task ID: `a7565nnm`
- Created: 2026-03-10
- Started: 2026-03-10
- Completed: 2026-03-10
- Recorded outcome:
  Bundled platform-specific Python wheels merged on main in PR #25. The Python package now prefers a bundled CLI binary on supported platforms while keeping MNEMIX_BINARY and PATH fallback behavior

## Validation

- Dex verification notes: hosted validation passed across Linux, macOS, and Windows, and issue #24 was closed after merge.
- Migration check: imported into `workflow/patches/` for repo-native tracking.

## References

- GitHub issue: #24 (micahcourey/mnemix) - https://github.com/micahcourey/mnemix/issues/24
- Original source: `.dex/tasks.jsonl`

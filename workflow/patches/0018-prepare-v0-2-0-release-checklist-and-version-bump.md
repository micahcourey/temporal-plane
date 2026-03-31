---
status: completed
summary: "Prepared the v0.2.0 release update on PR #29."
updated: "2026-03-10"
---

# Patch: Prepare v0.2.0 release checklist and version bump

## Summary

Prepare the next Mnemix release after the rebrand. Scope: bump the workspace and Python package version to 0.2.0, replace the one-off PyPI release plan with a reusable release checklist document, run the local Python package release preflight, and open a PR from main. Acceptance criteria: version sources are aligned, docs/release-checklist.md documents the generic recurring release procedure, release-related references point at the new checklist, and ./scripts/check-python-package.sh passes.

## Reason

This patch preserves legacy Dex history after the repository moved to repo-native workflow tracking with `mnemix-workflow`.

## Scope

- Preserve the original Dex task details for `87775625`.
- Keep the tracked status aligned with the final Dex state: `completed`.
- Avoid reinterpreting the historical task beyond the recorded Dex description and outcome.

## Implementation Notes

- Imported from legacy Dex tracking during the 2026-03-30 workflow migration.
- Original Dex task ID: `87775625`
- Created: 2026-03-10
- Started: 2026-03-10
- Completed: 2026-03-10
- Recorded outcome:
  Prepared the v0.2.0 release update on PR #29. Aligned the Python and Rust workspace versions to 0.2.0, replaced docs/pypi-release-plan.md with the reusable docs/release-checklist.md runbook, and updated README and CHANGELOG references

## Validation

- Dex verification notes: python/.release-venv/bin/python -m pytest python/tests, python/.release-venv/bin/python -m build --sdist, python/.release-venv/bin/python -m twine check --strict python/dist/*, cargo build -p mnemix-cli, and PYTHON_BIN=$PWD/python/.release-venv/bin/python ./scripts/check-python-bundled-wheel.sh all passed.
- Migration check: imported into `workflow/patches/` for repo-native tracking.

## References

- GitHub issue: #28 (micahcourey/mnemix) - https://github.com/micahcourey/mnemix/issues/28
- Original source: `.dex/tasks.jsonl`

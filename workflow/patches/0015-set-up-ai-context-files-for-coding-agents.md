---
status: completed
summary: "Set up repo-local AI context for Mnemix using AI DX Toolkit with a lean Rust-library-focused configuration."
updated: "2026-03-08"
---

# Patch: Set up AI context files for coding agents

## Summary

Create and standardize the repository-local AI context files that coding agents should use while working on Mnemix. Scope: decide which agent-facing context files should exist (for example AGENTS.md plus any additional repo-local context/instruction files needed for common coding agents), define the shared project context that should be duplicated or referenced across them, and ensure the contents align with docs/mnemix-plan-v3.md, docs/mnemix-roadmap.md, docs/lancedb-rust-sdk-agent-guide.md, and docs/mnemix-coding-guidelines.md. Include: project purpose, architecture boundaries, active milestones, validation expectations, Dex workflow expectations, and any agent-specific notes needed to keep work consistent. Acceptance criteria: the required agent context files are identified and created, they are internally consistent, they reference the canonical project docs, and they are suitable for use by coding agents working in this repo without re-discovery of core project context.

## Reason

This patch preserves legacy Dex history after the repository moved to repo-native workflow tracking with `mnemix-workflow`.

## Scope

- Preserve the original Dex task details for `uef075ri`.
- Keep the tracked status aligned with the final Dex state: `completed`.
- Avoid reinterpreting the historical task beyond the recorded Dex description and outcome.

## Implementation Notes

- Imported from legacy Dex tracking during the 2026-03-30 workflow migration.
- Original Dex task ID: `uef075ri`
- Created: 2026-03-08
- Started: 2026-03-08
- Completed: 2026-03-08
- Recorded outcome:
  Set up repo-local AI context for Mnemix using AI DX Toolkit with a lean Rust-library-focused configuration. Disabled the optional temporal/lance-context add-on, disabled the compliance agent and compliance instructions, generated .ai/ output, rewrote the active AGENTS/instructions/context files to match the current single-repo planning/scaffold state, pruned N/A API/auth/RBAC prompts and context files, and aligned the updater docs with the reduced context set

## Validation

- Dex verification notes: toolkit config validated, generation completed successfully, representative .ai files report no errors, and active .ai content no longer references compliance-standards or the removed API/auth/RBAC context files.
- Migration check: imported into `workflow/patches/` for repo-native tracking.

## References

- GitHub issue: #11 (micahcourey/mnemix) - https://github.com/micahcourey/mnemix/issues/11
- Original source: `.dex/tasks.jsonl`

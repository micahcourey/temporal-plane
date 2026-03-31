# ADR 001: Replace Dex With Repo-Native Workflow Tracking

## Status

Accepted

## Context

Mnemix had historical task tracking in `.dex/tasks.jsonl`, and repo guidance
instructed agents to use Dex for multi-step work. That kept task history in a
single machine-readable file, but it also made planning and status harder to
browse in normal repository review flows, and it introduced a second planning
system beside the repo-native workflow tooling now being adopted.

The repository now uses a Markdown-native planning model built around committed
files under `workflow/`, initialized through `mxw init`.

## Decision

Use committed `workflow/` artifacts as the canonical task tracking system for
`mnemix`.

As part of the migration:

- legacy Dex tasks are preserved as imported patches under `workflow/patches/`
- active Dex tasks remain open after import instead of being silently dropped
- `.dex/` is removed from the repository after the import completes
- contributor guidance is updated to point at repo-native workflow artifacts

## Consequences

- Historical task context stays visible in normal repo review and search flows
- Future work can be tracked with `mxw new ...` or `mxw patch new ...`
- Agents no longer need Dex-specific instructions or committed Dex state
- Imported historical patches should be treated as archival records unless a new
  patch or workstream intentionally supersedes them

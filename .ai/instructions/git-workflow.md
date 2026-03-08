# Git Workflow

> Temporal Plane uses Dex and git worktrees by default.

## Default Workflow

All multi-step implementation work should happen in a git worktree under `.worktrees/<task-id>/`.

### Starting Work

```bash
git worktree add .worktrees/<task-id> -b <agent>-<model>/<type>/<task-id>-<description>
cd .worktrees/<task-id>
dex start <task-id>
```

## Branch Naming

```text
<agent>-<model>/<type>/<task-id>-<description>
```

## Commit Format

Use conventional commits:

```text
<type>(<scope>): <subject>

<body>

Co-Authored-By: GitHub Copilot <noreply@github.com>
```

## Hard Rules

- never commit directly to `main`
- never treat the main working tree as the default place for feature work
- reference the Dex task in the PR body
- include verification performed

## Sources

- `docs/git-workflow.md`

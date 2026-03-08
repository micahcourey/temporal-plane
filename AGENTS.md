# AGENTS.md

## Task Management

Use Dex for any work that should persist across sessions.

### Rules

- Use Dex for multi-step implementation, planning, refactors, debugging, and backlog work.
- Prefer one parent task per milestone or feature area, with subtasks for concrete execution steps.
- Keep task descriptions rich enough that another agent can resume without re-discovery.
- Record verification in task results before marking work complete.
- Do not put raw Dex task IDs in permanent docs, commits, or PR text unless the ID is only being used locally.

### Project references

Before creating or updating implementation tasks, review:

- `docs/temporal-plane-plan-v3.md`
- `docs/temporal-plane-roadmap.md`
- `docs/lancedb-rust-sdk-agent-guide.md`
- `docs/coding-guidlines/rust-api-guidelines/checklist.md`
- `docs/coding-guidlines/rust-best-practices/README.md`

### Expected workflow

1. Check current backlog with `dex status` or `dex list`.
2. Start the next ready task with `dex start <id>`.
3. Keep context and decisions updated in the task description or result.
4. When work is verified, complete with a result summary and commit linkage when appropriate.

### Task quality bar

Good task descriptions should include:

- what needs to be done
- why it matters
- relevant files or modules
- acceptance criteria
- dependencies or blockers

Good task results should include:

- what changed
- key decisions
- verification performed
- any follow-up work created in Dex

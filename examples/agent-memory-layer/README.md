# Agent Memory Layer Example

This example package shows one practical way to add Mnemix to agent-driven
coding tools without pretending you can hard-enforce model behavior from inside
the model itself.

The example includes three pieces:

- [`AGENTS.md`](/Users/micah/Projects/mnemix/.worktrees/codex-mnemix-examples/examples/agent-memory-layer/AGENTS.md):
  repo instructions that teach when to use memory and when not to
- [`mnemix-memory-judgment/SKILL.md`](/Users/micah/Projects/mnemix/.worktrees/codex-mnemix-examples/examples/agent-memory-layer/mnemix-memory-judgment/SKILL.md):
  an Agent Skills-compatible skill package with YAML frontmatter and on-demand
  references
- [`agent_wrapper.py`](/Users/micah/Projects/mnemix/.worktrees/codex-mnemix-examples/examples/agent-memory-layer/agent_wrapper.py):
  a minimal host-side wrapper that performs recall before a task and writeback
  after the task

## Design Intent

For tools like Cursor, Codex, Claude Code, and GitHub Copilot:

- instructions improve judgment
- a skill provides a repeatable workflow
- a wrapper or adapter provides a consistent execution path

The combination is stronger than any one part alone.

## Recommended Adoption Order

1. start with the instruction file
2. add the skill so the agent has a concrete operating procedure
3. introduce a wrapper or adapter wherever the host tool allows it

That gives you selective, high-signal memory usage instead of “write something
to check the box” behavior.

## Skill Format

The bundled skill follows the open Agent Skills directory format:

```text
mnemix-memory-judgment/
├── SKILL.md
└── references/
    └── REFERENCE.md
```

`SKILL.md` contains the required frontmatter and concise instructions. The
longer heuristics live in `references/REFERENCE.md` so agents can load them
only when needed.

The host-side wrapper pattern in
[`agent_wrapper.py`](/Users/micah/Projects/mnemix/.worktrees/codex-mnemix-examples/examples/agent-memory-layer/agent_wrapper.py)
matches the top-level adapter layout under
[`adapters/`](/Users/micah/Projects/mnemix/.worktrees/codex-mnemix-examples/adapters/).

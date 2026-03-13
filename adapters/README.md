# Host Adapters

Mnemix keeps its core API generic and exposes host-specific workflow helpers as
top-level adapter modules under `adapters/`.

Available adapters:

- `adapters/coding_agent_adapter.py`
- `adapters/chat_assistant_adapter.py`
- `adapters/ci_bot_adapter.py`
- `adapters/review_tool_adapter.py`

Shared utilities live in `adapters/_adapter_base.py`, and package exports are
available through `adapters/__init__.py`.

These adapters use only the public `mnemix` Python client and do not depend on
storage internals.

`CodingAgentAdapter` is the richest adapter because coding agents are currently
the primary host workflow. It now covers:

- task-start recall modes (`quick`, `normal`, `deep`)
- pinned-memory and recent-history context assembly
- targeted search and memory inspection
- decisions, procedures, summaries, facts, and pitfalls
- pre-change checkpoints
- version inspection and restore
- optimize, export, and staged import flows

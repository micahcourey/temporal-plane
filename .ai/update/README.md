# Updating Context Files

This directory contains the **Context Updater** agent and update metadata. Use it to keep your `.ai/context/` files in sync as your project evolves — without re-running the full bootstrap process.

## Who Should Run Updates

Context updates are typically performed by the **Tech Lead or Architect**, not every developer. The workflow is:

1. Tech lead runs the update agent
2. Reviews and approves changes
3. Commits updated context files
4. Creates a PR for team review
5. After merge, the full dev team pulls the latest context

## When to Update

Run the update agent when:
- Database schema has changed (new tables, columns, migrations)
- New repositories have been added to the workspace
- Framework or dependency versions have been upgraded
- Major glossary, architecture, or integration details have changed
- The team suspects context files are stale

## How to Run

### 1. Copy the update agent into your agents directory

```bash
cp .ai/update/update.agent.md .github/agents/context-updater.agent.md
```

Reload VS Code:

> **Cmd+Shift+P** → `Developer: Reload Window`

### 2. Run the Context Updater

1. Open **Copilot Chat**
2. Select **Claude Sonnet 4.6** (or Opus 4.6 for large projects)
3. Select **Context Updater** from the chat mode dropdown
4. Send: **"Update context files"**

The agent will:
- Scan the codebase for changes since the last update
- Present a detailed change report (new tables, repository inventory, glossary terms, tooling drift, etc.)
- Ask for your approval before modifying any files
- Update data files (`.jsonl`, `.yaml`) directly
- Suggest changes to prose files (`.md`) for your review
- Detect version drift in `toolkit.config.yaml`

For this repo, the updater is intentionally scoped to the current lean context set. It should not recreate removed API, access-control, or role-matrix files unless the project scope changes and you explicitly want those contexts back.

### 3. Review and commit

```bash
git add .ai/
git commit -m "chore: update AI context files"
git push origin your-branch
```

### 4. Clean up (optional)

Remove the update agent from the agents directory after the PR is merged — it doesn't need to be in every developer's agent dropdown:

```bash
rm .github/agents/context-updater.agent.md
```

It remains here in `.ai/update/` for next time.

## Update Metadata

After each update, the agent writes `.ai/update/.last-updated` with:
- Timestamp of the last update
- Which files were updated
- Summary of changes detected

This helps the agent know what changed between updates.

## Update Strategy

| File Type | Strategy | Safe? |
|-----------|----------|-------|
| `.jsonl` data files | Full regenerate (add new, remove stale) | Yes — structured data |
| `.yaml` data files | Full regenerate (merge new) | Yes — structured data |
| `.md` prose files | **Suggest only** — user reviews diff | Yes — never auto-overwrites |
| `toolkit.config.yaml` | **Suggest only** — show drift | Yes — user decides |
| Instructions / Adapters | Skip unless config changes | N/A |

**Key guarantee:** The agent never overwrites prose `.md` files without explicit approval. Your team's custom documentation is always preserved.

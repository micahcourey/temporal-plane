---
name: git-workflow
description: Git and GitHub workflow automation for temporal-plane repositories. Use when working with branches, PRs, commits, or repository management.
---

# Git Workflow Skill

Automate common Git and GitHub workflows following temporal-plane conventions.

## Context Files

Consult [AGENTS.md](AGENTS.md) for the project knowledge routing table.
**For this skill, prioritize**: Repository index (for repo naming and multi-repo structure) and the git-workflow instruction module.

## When This Skill Activates

This skill activates when:
- Working with `.git` operations or discussing git workflow
- Creating branches, commits, or pull requests
- Discussing version control or release management

## Quick Commands

| Script | Purpose |
|--------|----------|
| `scripts/new-branch.sh` | Create feature branch from main |
| `scripts/commit.sh` | Commit with conventional format |
| `scripts/create-pr.sh` | Create pull request via GitHub CLI |
| `scripts/create-or-update-pr.sh` | Idempotent PR create/update |
| `scripts/pr-status.sh` | Check PR status and CI checks |
| `scripts/update-branch.sh` | Rebase/merge from main |
| `scripts/sync-master.sh` | Sync local main branch |
| `scripts/cleanup-branches.sh` | Remove merged local branches |

### Create Feature Branch
```bash
scripts/new-branch.sh <agent>-<model>/<type>/<task-id>-<description>
```

### Commit with Conventional Format
```bash
git commit -m "feat(component): add new feature

Detailed description here

Closes <agent>-<model>/<type>/<task-id>-<description>"
```

## Instructions

### Branch Naming Convention

All branches must follow the project ticket pattern:

```
<agent>-<model>/<type>/<task-id>-<description>
```

### Creating a New Branch

```bash
git checkout main
git pull origin main
git checkout -b <agent>-<model>/<type>/<task-id>-<description>
git push -u origin <agent>-<model>/<type>/<task-id>-<description>
```

### Commit Message Format

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

Closes <agent>-<model>/<type>/<task-id>-<description>
```

**Types**:
| Type | Description |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation only |
| `style` | Formatting, no code change |
| `refactor` | Code restructuring |
| `test` | Adding/updating tests |
| `chore` | Build, CI, dependencies |

**Scope**: Component or area affected

### Pull Request Workflow

**Creating a PR with GitHub CLI**:
```bash
# Create PR
gh pr create --title "<agent>-<model>/<type>/<task-id>-<description>: Feature description" \
  --base main

# Create draft PR
gh pr create --draft --title "<agent>-<model>/<type>/<task-id>-<description>: WIP Feature"

# Request review
gh pr edit --add-reviewer username1,username2
```

**PR Title Format**:
```
<agent>-<model>/<type>/<task-id>-<description>: Brief description of change
```

### Merging Strategy

Use **squash merging** for feature branches:

```bash
gh pr merge --squash --delete-branch
```

### Keeping Branch Updated

```bash
# Merge strategy
git checkout main
git pull origin main
git checkout -
git merge main

# Rebase strategy
git fetch origin
git rebase origin/main
```

### Cleanup After Merge

```bash
# Delete local merged branches
git branch --merged main | grep -v "main" | xargs git branch -d

# Prune remote tracking branches
git fetch --prune
```

### Useful GitHub CLI Commands

```bash
# List open PRs
gh pr list

# Check PR status and CI checks
gh pr status
gh pr checks

# View PR diff
gh pr diff

# Approve PR
gh pr review --approve

# Request changes
gh pr review --request-changes --body "Changes needed..."
```

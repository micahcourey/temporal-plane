#!/bin/bash
#
# create-or-update-pr.sh - Create or update a PR non-interactively
#
# Designed for AI agent use — avoids heredoc issues by requiring
# --body-file instead of inline --body content.
#
# Usage:
#   ./create-or-update-pr.sh --title "feat: my feature" --body-file /tmp/pr-body.md
#   ./create-or-update-pr.sh --title "feat: my feature" --body-file /tmp/pr-body.md --draft
#   ./create-or-update-pr.sh --title "feat: my feature" --body-file /tmp/pr-body.md --base main
#   ./create-or-update-pr.sh --title "feat: my feature" --body-file /tmp/pr-body.md --reviewer user1,user2
#
# Behavior:
#   1. Pushes current branch to origin
#   2. Attempts `gh pr create`
#   3. If PR already exists, falls back to `gh pr edit`
#   4. Prints the PR URL on success
#

set -e

# ── Defaults ──────────────────────────────────────────────────────────────
TITLE=""
BODY_FILE=""
BASE=""
DRAFT=""
REVIEWERS=""

# ── Parse arguments ──────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
    case $1 in
        --title|-t)       TITLE="$2";     shift 2 ;;
        --body-file|-b)   BODY_FILE="$2"; shift 2 ;;
        --base)           BASE="$2";      shift 2 ;;
        --draft|-d)       DRAFT="--draft"; shift ;;
        --reviewer|-r)    REVIEWERS="$2"; shift 2 ;;
        --help|-h)
            sed -n '2,/^$/p' "$0" | sed 's/^# \?//'
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
    esac
done

# ── Validate ──────────────────────────────────────────────────────────────
if [[ -z "$TITLE" ]]; then
    echo "Error: --title is required" >&2
    exit 1
fi

if [[ -z "$BODY_FILE" ]]; then
    echo "Error: --body-file is required (use create_file to write the body first)" >&2
    exit 1
fi

if [[ ! -f "$BODY_FILE" ]]; then
    echo "Error: body file not found: $BODY_FILE" >&2
    exit 1
fi

if ! command -v gh &> /dev/null; then
    echo "Error: GitHub CLI (gh) is not installed" >&2
    exit 1
fi

if ! git rev-parse --is-inside-work-tree > /dev/null 2>&1; then
    echo "Error: not in a git repository" >&2
    exit 1
fi

# ── Resolve branch and base ──────────────────────────────────────────────
BRANCH=$(git rev-parse --abbrev-ref HEAD)

if [[ -z "$BASE" ]]; then
    BASE=$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null \
        | sed 's@^refs/remotes/origin/@@' || echo "main")
fi

if [[ "$BRANCH" == "$BASE" ]]; then
    echo "Error: cannot create PR from $BASE branch" >&2
    exit 1
fi

# ── Push branch ──────────────────────────────────────────────────────────
echo "Pushing $BRANCH to origin..."
git push -u origin "$BRANCH" 2>/dev/null || git push origin "$BRANCH"

# ── Build flags ──────────────────────────────────────────────────────────
CREATE_FLAGS=(--title "$TITLE" --body-file "$BODY_FILE" --base "$BASE")
EDIT_FLAGS=(--title "$TITLE" --body-file "$BODY_FILE")

if [[ -n "$DRAFT" ]]; then
    CREATE_FLAGS+=("$DRAFT")
fi

if [[ -n "$REVIEWERS" ]]; then
    CREATE_FLAGS+=(--reviewer "$REVIEWERS")
    EDIT_FLAGS+=(--add-reviewer "$REVIEWERS")
fi

# ── Create or update ────────────────────────────────────────────────────
PR_OUTPUT=$(gh pr create "${CREATE_FLAGS[@]}" 2>&1) && {
    echo "✓ PR created: $PR_OUTPUT"
    exit 0
}

# If create failed, check if PR already exists
if echo "$PR_OUTPUT" | grep -qi "already exists"; then
    echo "PR already exists for $BRANCH, updating..."
    PR_OUTPUT=$(gh pr edit "${EDIT_FLAGS[@]}" 2>&1) && {
        echo "✓ PR updated: $PR_OUTPUT"
        exit 0
    }
    echo "Error updating PR: $PR_OUTPUT" >&2
    exit 1
fi

# Some other error
echo "Error creating PR: $PR_OUTPUT" >&2
exit 1

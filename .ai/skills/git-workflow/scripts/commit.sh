#!/bin/bash
#
# commit.sh - Create a conventional commit with guided prompts
#
# Usage: ./commit.sh
#        ./commit.sh "feat" "scope" "message"
#
# This script:
# 1. Shows staged changes
# 2. Prompts for commit type, scope, and message
# 3. Creates a properly formatted conventional commit
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Print colored output
info() { echo -e "${BLUE}ℹ${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
warn() { echo -e "${YELLOW}⚠${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1"; exit 1; }

# Check if we're in a git repository
if ! git rev-parse --is-inside-work-tree > /dev/null 2>&1; then
    error "Not in a git repository"
fi

# Commit types
TYPES=("feat" "fix" "docs" "style" "refactor" "test" "chore" "perf" "ci" "build")
TYPE_DESCRIPTIONS=(
    "feat     - A new feature"
    "fix      - A bug fix"
    "docs     - Documentation only changes"
    "style    - Formatting, missing semi-colons, etc"
    "refactor - Code restructuring without feature change"
    "test     - Adding or updating tests"
    "chore    - Build process, dependencies, etc"
    "perf     - Performance improvements"
    "ci       - CI/CD changes"
    "build    - Build system changes"
)

# Check for staged changes
STAGED=$(git diff --cached --name-only)
if [ -z "$STAGED" ]; then
    warn "No staged changes. Stage files first with 'git add'"
    echo ""
    echo "Unstaged changes:"
    git status --short
    exit 1
fi

echo -e "${CYAN}Staged changes:${NC}"
git diff --cached --stat
echo ""

# If arguments provided, use them directly
if [ $# -eq 3 ]; then
    TYPE="$1"
    SCOPE="$2"
    MESSAGE="$3"
else
    # Interactive mode
    echo -e "${CYAN}Select commit type:${NC}"
    for i in "${!TYPE_DESCRIPTIONS[@]}"; do
        echo "  $((i+1))) ${TYPE_DESCRIPTIONS[$i]}"
    done
    echo ""
    read -p "Enter number (1-${#TYPES[@]}): " TYPE_NUM

    if [[ ! "$TYPE_NUM" =~ ^[0-9]+$ ]] || [ "$TYPE_NUM" -lt 1 ] || [ "$TYPE_NUM" -gt ${#TYPES[@]} ]; then
        error "Invalid selection"
    fi

    TYPE="${TYPES[$((TYPE_NUM-1))]}"
    echo ""

    # Get scope
    echo -e "${CYAN}Enter scope (component/module affected):${NC}"
    echo "  Examples: auth, participant, compliance, api, ui"
    read -p "Scope (optional, press Enter to skip): " SCOPE
    echo ""

    # Get commit message
    echo -e "${CYAN}Enter commit message:${NC}"
    echo "  - Use imperative mood: 'add feature' not 'added feature'"
    echo "  - Keep under 72 characters"
    read -p "Message: " MESSAGE

    if [ -z "$MESSAGE" ]; then
        error "Commit message is required"
    fi
fi

# Build commit message
if [ -n "$SCOPE" ]; then
    COMMIT_MSG="$TYPE($SCOPE): $MESSAGE"
else
    COMMIT_MSG="$TYPE: $MESSAGE"
fi

# Check message length
if [ ${#COMMIT_MSG} -gt 72 ]; then
    warn "Commit message is ${#COMMIT_MSG} characters (recommended: 72 max)"
fi

echo ""
echo -e "${CYAN}Commit message:${NC}"
echo "  $COMMIT_MSG"
echo ""

# Ask for body (optional)
read -p "Add detailed body? (y/N): " ADD_BODY
BODY=""
if [[ "$ADD_BODY" =~ ^[Yy]$ ]]; then
    echo "Enter body (press Ctrl+D when done):"
    BODY=$(cat)
fi

# Ask for ticket reference
BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [[ "$BRANCH" =~ ^-([0-9]+)$ ]]; then
    TICKET="-${BASH_REMATCH[1]}"
    read -p "Add 'Closes $TICKET' to footer? (Y/n): " ADD_CLOSES
    if [[ ! "$ADD_CLOSES" =~ ^[Nn]$ ]]; then
        if [ -n "$BODY" ]; then
            BODY="$BODY

Closes $TICKET"
        else
            BODY="Closes $TICKET"
        fi
    fi
fi

# Perform the commit
if [ -n "$BODY" ]; then
    git commit -m "$COMMIT_MSG" -m "$BODY"
else
    git commit -m "$COMMIT_MSG"
fi

success "Committed: $COMMIT_MSG"
echo ""
echo "Next steps:"
echo "  git push"
echo "  gh pr create (if ready for review)"

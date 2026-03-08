#!/bin/bash
#
# new-branch.sh - Create a new feature branch from master
#
# Usage: ./new-branch.sh <agent>-<model>/<type>/<task-id>-<description>
#
# This script:
# 1. Stashes any uncommitted changes
# 2. Checks out master branch
# 3. Pulls latest changes from origin
# 4. Creates and checks out new branch
# 5. Pushes branch to origin
# 6. Restores stashed changes (if any)
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored output
info() { echo -e "${BLUE}ℹ${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
warn() { echo -e "${YELLOW}⚠${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1"; exit 1; }

# Validate branch name argument
if [ -z "$1" ]; then
    echo "Usage: $0 <branch-name>"
    echo ""
    echo "Examples:"
    echo "  $0 -1234"
    echo "  $0 -5678"
    exit 1
fi

BRANCH_NAME="$1"

# Validate branch name format
if [[ ! "$BRANCH_NAME" =~ ^-[0-9]+$ ]]; then
    warn "Branch name '$BRANCH_NAME' doesn't follow <agent>-<model>/<type>/<task-id>-<description> format"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Check if we're in a git repository
if ! git rev-parse --is-inside-work-tree > /dev/null 2>&1; then
    error "Not in a git repository"
fi

# Determine default branch (master or main)
DEFAULT_BRANCH=$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@' || echo "master")
info "Default branch: $DEFAULT_BRANCH"

# Check if branch already exists
if git show-ref --verify --quiet "refs/heads/$BRANCH_NAME"; then
    error "Branch '$BRANCH_NAME' already exists locally"
fi

if git ls-remote --exit-code --heads origin "$BRANCH_NAME" > /dev/null 2>&1; then
    error "Branch '$BRANCH_NAME' already exists on origin"
fi

# Stash any uncommitted changes
STASH_NEEDED=false
if ! git diff --quiet || ! git diff --cached --quiet; then
    info "Stashing uncommitted changes..."
    git stash push -m "Auto-stash before creating $BRANCH_NAME"
    STASH_NEEDED=true
    success "Changes stashed"
fi

# Checkout default branch
info "Checking out $DEFAULT_BRANCH..."
git checkout "$DEFAULT_BRANCH"
success "On $DEFAULT_BRANCH"

# Pull latest changes
info "Pulling latest changes..."
git pull origin "$DEFAULT_BRANCH"
success "Up to date with origin/$DEFAULT_BRANCH"

# Create and checkout new branch
info "Creating branch '$BRANCH_NAME'..."
git checkout -b "$BRANCH_NAME"
success "Created and checked out '$BRANCH_NAME'"

# Push branch to origin
info "Pushing branch to origin..."
git push -u origin "$BRANCH_NAME"
success "Branch pushed to origin"

# Restore stashed changes if any
if [ "$STASH_NEEDED" = true ]; then
    info "Restoring stashed changes..."
    git stash pop
    success "Stashed changes restored"
fi

echo ""
success "Ready to work on $BRANCH_NAME!"
echo ""
echo "Next steps:"
echo "  1. Make your changes"
echo "  2. git add <files>"
echo "  3. git commit -m \"feat(scope): description\""
echo "  4. git push"
echo "  5. gh pr create --title \"$BRANCH_NAME: Description\""

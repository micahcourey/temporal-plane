#!/bin/bash
#
# sync-master.sh - Sync local master/main branch with origin
#
# Usage: ./sync-master.sh
#
# This script:
# 1. Stashes any uncommitted changes
# 2. Checks out master/main branch
# 3. Pulls latest changes from origin
# 4. Returns to original branch
# 5. Restores stashed changes (if any)
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

# Check if we're in a git repository
if ! git rev-parse --is-inside-work-tree > /dev/null 2>&1; then
    error "Not in a git repository"
fi

# Get current branch
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
info "Current branch: $CURRENT_BRANCH"

# Determine default branch (master or main)
DEFAULT_BRANCH=$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@' || echo "master")
info "Default branch: $DEFAULT_BRANCH"

# Stash any uncommitted changes
STASH_NEEDED=false
if ! git diff --quiet || ! git diff --cached --quiet; then
    info "Stashing uncommitted changes..."
    git stash push -m "Auto-stash during sync-master"
    STASH_NEEDED=true
    success "Changes stashed"
fi

# Checkout default branch
if [ "$CURRENT_BRANCH" != "$DEFAULT_BRANCH" ]; then
    info "Checking out $DEFAULT_BRANCH..."
    git checkout "$DEFAULT_BRANCH"
fi

# Pull latest changes
info "Pulling latest changes..."
git pull origin "$DEFAULT_BRANCH"
success "Up to date with origin/$DEFAULT_BRANCH"

# Return to original branch
if [ "$CURRENT_BRANCH" != "$DEFAULT_BRANCH" ]; then
    info "Returning to $CURRENT_BRANCH..."
    git checkout "$CURRENT_BRANCH"
    success "Back on $CURRENT_BRANCH"
fi

# Restore stashed changes if any
if [ "$STASH_NEEDED" = true ]; then
    info "Restoring stashed changes..."
    git stash pop
    success "Stashed changes restored"
fi

echo ""
success "Master branch synced!"

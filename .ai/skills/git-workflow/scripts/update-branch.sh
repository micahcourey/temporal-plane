#!/bin/bash
#
# update-branch.sh - Merge master/main into current branch
#
# Usage: ./update-branch.sh
#        ./update-branch.sh --rebase  # Use rebase instead of merge
#
# This script:
# 1. Stashes any uncommitted changes
# 2. Fetches latest from origin
# 3. Merges (or rebases) master into current branch
# 4. Restores stashed changes
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

# Parse arguments
USE_REBASE=false
while [[ $# -gt 0 ]]; do
    case $1 in
        --rebase|-r)
            USE_REBASE=true
            shift
            ;;
        *)
            shift
            ;;
    esac
done

# Check if we're in a git repository
if ! git rev-parse --is-inside-work-tree > /dev/null 2>&1; then
    error "Not in a git repository"
fi

# Get current branch
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)

# Determine default branch (master or main)
DEFAULT_BRANCH=$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@' || echo "master")

if [ "$CURRENT_BRANCH" = "$DEFAULT_BRANCH" ]; then
    error "Already on $DEFAULT_BRANCH - use sync-master.sh instead"
fi

info "Current branch: $CURRENT_BRANCH"
info "Default branch: $DEFAULT_BRANCH"
if [ "$USE_REBASE" = true ]; then
    info "Strategy: rebase"
else
    info "Strategy: merge"
fi
echo ""

# Stash any uncommitted changes
STASH_NEEDED=false
if ! git diff --quiet || ! git diff --cached --quiet; then
    info "Stashing uncommitted changes..."
    git stash push -m "Auto-stash before update-branch"
    STASH_NEEDED=true
    success "Changes stashed"
fi

# Fetch latest from origin
info "Fetching latest from origin..."
git fetch origin "$DEFAULT_BRANCH"
success "Fetched origin/$DEFAULT_BRANCH"

# Check if there are changes to merge
BEHIND=$(git rev-list --count HEAD..origin/$DEFAULT_BRANCH)
if [ "$BEHIND" -eq 0 ]; then
    success "Branch is already up to date with $DEFAULT_BRANCH"
    
    # Restore stashed changes if any
    if [ "$STASH_NEEDED" = true ]; then
        info "Restoring stashed changes..."
        git stash pop
        success "Stashed changes restored"
    fi
    exit 0
fi

info "Branch is $BEHIND commit(s) behind origin/$DEFAULT_BRANCH"

# Perform merge or rebase
if [ "$USE_REBASE" = true ]; then
    info "Rebasing on origin/$DEFAULT_BRANCH..."
    if git rebase origin/$DEFAULT_BRANCH; then
        success "Rebase complete"
    else
        echo ""
        error "Rebase conflicts detected. Resolve conflicts and run:
  git rebase --continue
  
Or abort with:
  git rebase --abort"
    fi
else
    info "Merging origin/$DEFAULT_BRANCH into $CURRENT_BRANCH..."
    if git merge origin/$DEFAULT_BRANCH -m "Merge $DEFAULT_BRANCH into $CURRENT_BRANCH"; then
        success "Merge complete"
    else
        echo ""
        warn "Merge conflicts detected!"
        echo ""
        echo "Conflicting files:"
        git diff --name-only --diff-filter=U
        echo ""
        echo "To resolve:"
        echo "  1. Edit the conflicting files"
        echo "  2. git add <resolved-files>"
        echo "  3. git commit"
        echo ""
        echo "Or abort with: git merge --abort"
        
        # Restore stash before exiting (user will need to handle manually)
        if [ "$STASH_NEEDED" = true ]; then
            warn "Your stashed changes are still in the stash"
            echo "Run 'git stash pop' after resolving conflicts"
        fi
        exit 1
    fi
fi

# Restore stashed changes if any
if [ "$STASH_NEEDED" = true ]; then
    info "Restoring stashed changes..."
    if git stash pop; then
        success "Stashed changes restored"
    else
        warn "Could not automatically restore stashed changes (possible conflict)"
        echo "Your changes are still in the stash. Run 'git stash pop' manually."
    fi
fi

echo ""
success "Branch updated with latest from $DEFAULT_BRANCH!"
echo ""
echo "Commits merged: $BEHIND"
echo ""
echo "Next steps:"
echo "  git push  # Push updated branch to origin"

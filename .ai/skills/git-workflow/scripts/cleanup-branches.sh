#!/bin/bash
#
# cleanup-branches.sh - Remove merged local branches
#
# Usage: ./cleanup-branches.sh          # Dry run (preview)
#        ./cleanup-branches.sh --force  # Actually delete branches
#
# This script:
# - Lists local branches that have been merged to master
# - Optionally deletes them to keep your local repo clean
# - Never deletes master, main, or develop branches
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

# Parse arguments
FORCE=false
DELETE_REMOTE=false
while [[ $# -gt 0 ]]; do
    case $1 in
        --force|-f)
            FORCE=true
            shift
            ;;
        --remote|-r)
            DELETE_REMOTE=true
            shift
            ;;
        --help|-h)
            echo "Usage: cleanup-branches.sh [options]"
            echo ""
            echo "Options:"
            echo "  --force, -f     Actually delete branches (default: dry run)"
            echo "  --remote, -r    Also delete remote branches"
            echo "  --help, -h      Show this help message"
            exit 0
            ;;
        *)
            error "Unknown option: $1"
            ;;
    esac
done

# Protected branches that should never be deleted
PROTECTED_BRANCHES="master main develop release"

# Determine default branch
DEFAULT_BRANCH=$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@')
if [ -z "$DEFAULT_BRANCH" ]; then
    DEFAULT_BRANCH="master"
fi

info "Default branch: $DEFAULT_BRANCH"
info "Fetching latest from origin..."
git fetch --prune origin

# Get current branch
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}Merged Local Branches${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"

# Find merged branches
MERGED_BRANCHES=$(git branch --merged "$DEFAULT_BRANCH" | grep -v "^\*" | tr -d ' ' || true)

BRANCHES_TO_DELETE=()
for branch in $MERGED_BRANCHES; do
    # Skip protected branches
    if echo "$PROTECTED_BRANCHES" | grep -qw "$branch"; then
        continue
    fi
    
    # Skip current branch
    if [ "$branch" = "$CURRENT_BRANCH" ]; then
        warn "Skipping current branch: $branch"
        continue
    fi
    
    BRANCHES_TO_DELETE+=("$branch")
done

if [ ${#BRANCHES_TO_DELETE[@]} -eq 0 ]; then
    success "No merged branches to clean up"
else
    for branch in "${BRANCHES_TO_DELETE[@]}"; do
        if [ "$FORCE" = true ]; then
            git branch -d "$branch"
            success "Deleted: $branch"
        else
            echo "  - $branch"
        fi
    done
    
    if [ "$FORCE" = false ]; then
        echo ""
        warn "Dry run - no branches deleted"
        info "Run with --force to delete these branches"
    fi
fi

echo ""

# Handle remote branches if requested
if [ "$DELETE_REMOTE" = true ]; then
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${CYAN}Stale Remote Branches${NC}"
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
    
    # Find remote branches that have been deleted on origin
    GONE_BRANCHES=$(git branch -vv | grep ': gone]' | awk '{print $1}' || true)
    
    if [ -z "$GONE_BRANCHES" ]; then
        success "No stale remote tracking branches"
    else
        for branch in $GONE_BRANCHES; do
            # Skip protected branches
            if echo "$PROTECTED_BRANCHES" | grep -qw "$branch"; then
                continue
            fi
            
            if [ "$FORCE" = true ]; then
                git branch -D "$branch"
                success "Deleted (remote gone): $branch"
            else
                echo "  - $branch (remote deleted)"
            fi
        done
        
        if [ "$FORCE" = false ]; then
            echo ""
            warn "Dry run - no branches deleted"
            info "Run with --force to delete these branches"
        fi
    fi
fi

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}Current State${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo "Local branches remaining: $(git branch | wc -l | tr -d ' ')"
echo "Current branch: $CURRENT_BRANCH"

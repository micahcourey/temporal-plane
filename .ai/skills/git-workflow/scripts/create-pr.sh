#!/bin/bash
#
# create-pr.sh - Create a pull request using GitHub CLI
#
# Usage: ./create-pr.sh
#        ./create-pr.sh --draft
#        ./create-pr.sh "PR Title"
#
# This script:
# 1. Pushes current branch to origin
# 2. Creates a PR with template
# 3. Optionally adds reviewers
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

# Check dependencies
if ! command -v gh &> /dev/null; then
    error "GitHub CLI (gh) is not installed. Install from: https://cli.github.com/"
fi

# Check if authenticated
if ! gh auth status &> /dev/null; then
    error "Not authenticated with GitHub CLI. Run: gh auth login"
fi

# Check if we're in a git repository
if ! git rev-parse --is-inside-work-tree > /dev/null 2>&1; then
    error "Not in a git repository"
fi

# Get current branch
BRANCH=$(git rev-parse --abbrev-ref HEAD)
DEFAULT_BRANCH=$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@' || echo "master")

if [ "$BRANCH" = "$DEFAULT_BRANCH" ]; then
    error "Cannot create PR from $DEFAULT_BRANCH branch"
fi

info "Current branch: $BRANCH"

# Parse arguments
DRAFT=""
TITLE=""
while [[ $# -gt 0 ]]; do
    case $1 in
        --draft|-d)
            DRAFT="--draft"
            shift
            ;;
        *)
            TITLE="$1"
            shift
            ;;
    esac
done

# Check for uncommitted changes
if ! git diff --quiet || ! git diff --cached --quiet; then
    warn "You have uncommitted changes"
    read -p "Continue anyway? (y/N): " CONTINUE
    if [[ ! "$CONTINUE" =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Push current branch
info "Pushing branch to origin..."
git push -u origin "$BRANCH" 2>/dev/null || git push origin "$BRANCH"
success "Branch pushed"

# Extract ticket from branch name
TICKET=""
if [[ "$BRANCH" =~ ^(-[0-9]+) ]]; then
    TICKET="${BASH_REMATCH[1]}"
fi

# Generate title if not provided
if [ -z "$TITLE" ]; then
    if [ -n "$TICKET" ]; then
        echo -e "${CYAN}Enter PR title:${NC}"
        read -p "$TICKET: " TITLE_SUFFIX
        TITLE="$TICKET: $TITLE_SUFFIX"
    else
        read -p "PR Title: " TITLE
    fi
fi

if [ -z "$TITLE" ]; then
    error "PR title is required"
fi

# Check for PR template
TEMPLATE_FILE=""
TEMPLATE_BODY=""
for template in "pull_request_template.md" ".github/pull_request_template.md" ".github/PULL_REQUEST_TEMPLATE.md"; do
    if [ -f "$template" ]; then
        TEMPLATE_FILE="$template"
        break
    fi
done

# Build PR body
echo ""
echo -e "${CYAN}PR Description:${NC}"
echo "Enter description (or press Enter to use template/default):"
read -p "> " CUSTOM_BODY

if [ -n "$CUSTOM_BODY" ]; then
    PR_BODY="$CUSTOM_BODY"
elif [ -n "$TEMPLATE_FILE" ]; then
    PR_BODY=$(cat "$TEMPLATE_FILE")
    info "Using template from $TEMPLATE_FILE"
else
    # Default body
    PR_BODY="## Summary
<!-- Describe your changes -->

## Changes
<!-- List the changes made -->

## Testing
- [ ] Unit tests pass
- [ ] Manual testing complete

## Checklist
- [ ] Code follows project patterns
- [ ] Tests added/updated
- [ ] Documentation updated (if needed)
"
    if [ -n "$TICKET" ]; then
        PR_BODY="$PR_BODY
Closes $TICKET"
    fi
fi

# Ask about reviewers
echo ""
echo -e "${CYAN}Add reviewers?${NC}"
echo "Enter GitHub usernames (comma-separated) or press Enter to skip:"
read -p "Reviewers: " REVIEWERS

REVIEWER_FLAG=""
if [ -n "$REVIEWERS" ]; then
    REVIEWER_FLAG="--reviewer $REVIEWERS"
fi

# Create the PR
echo ""
info "Creating pull request..."

PR_URL=$(gh pr create \
    --title "$TITLE" \
    --body "$PR_BODY" \
    --base "$DEFAULT_BRANCH" \
    $DRAFT \
    $REVIEWER_FLAG 2>&1)

if [ $? -eq 0 ]; then
    success "Pull request created!"
    echo ""
    echo -e "${GREEN}$PR_URL${NC}"
    echo ""
    
    # Ask to open in browser
    read -p "Open in browser? (Y/n): " OPEN_BROWSER
    if [[ ! "$OPEN_BROWSER" =~ ^[Nn]$ ]]; then
        gh pr view --web
    fi
else
    error "Failed to create PR: $PR_URL"
fi

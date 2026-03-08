#!/bin/bash
#
# pr-status.sh - Check PR status and CI checks
#
# Usage: ./pr-status.sh        # Check PR for current branch
#        ./pr-status.sh 123    # Check PR #123
#
# This script shows:
# - PR details (title, status, reviewers)
# - CI check status
# - Review status
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

# Get PR number
PR_NUMBER="$1"

if [ -z "$PR_NUMBER" ]; then
    # Try to get PR for current branch
    BRANCH=$(git rev-parse --abbrev-ref HEAD)
    info "Looking for PR for branch: $BRANCH"
    
    PR_NUMBER=$(gh pr list --head "$BRANCH" --json number --jq '.[0].number' 2>/dev/null)
    
    if [ -z "$PR_NUMBER" ] || [ "$PR_NUMBER" = "null" ]; then
        error "No PR found for branch '$BRANCH'"
    fi
fi

info "Checking PR #$PR_NUMBER"
echo ""

# Get PR details
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}PR Details${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"

gh pr view "$PR_NUMBER" --json title,state,author,createdAt,url,isDraft,mergeable,headRefName,baseRefName \
    --template '
Title:    {{.title}}
State:    {{.state}}{{if .isDraft}} (Draft){{end}}
Author:   {{.author.login}}
Created:  {{.createdAt | timeago}}
Branch:   {{.headRefName}} → {{.baseRefName}}
Mergeable: {{.mergeable}}
URL:      {{.url}}
'

echo ""

# Get check status
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}CI Checks${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"

CHECKS=$(gh pr checks "$PR_NUMBER" 2>&1) || true

if [ -z "$CHECKS" ] || [[ "$CHECKS" == *"no checks"* ]]; then
    echo "No CI checks configured"
else
    echo "$CHECKS" | while read -r line; do
        if [[ "$line" == *"pass"* ]]; then
            echo -e "${GREEN}✓${NC} $line"
        elif [[ "$line" == *"fail"* ]]; then
            echo -e "${RED}✗${NC} $line"
        elif [[ "$line" == *"pending"* ]] || [[ "$line" == *"running"* ]]; then
            echo -e "${YELLOW}○${NC} $line"
        else
            echo "  $line"
        fi
    done
fi

echo ""

# Get review status
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}Reviews${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"

REVIEWS=$(gh pr view "$PR_NUMBER" --json reviews,reviewRequests --jq '
  (.reviewRequests[]? | "⏳ \(.login) - Requested") // empty,
  (.reviews[]? | 
    (if .state == "APPROVED" then "✓ " 
     elif .state == "CHANGES_REQUESTED" then "✗ "
     else "○ " end) + .author.login + " - " + .state
  ) // empty
' 2>/dev/null)

if [ -z "$REVIEWS" ]; then
    echo "No reviews yet"
else
    echo "$REVIEWS" | while read -r line; do
        if [[ "$line" == *"APPROVED"* ]]; then
            echo -e "${GREEN}$line${NC}"
        elif [[ "$line" == *"CHANGES_REQUESTED"* ]]; then
            echo -e "${RED}$line${NC}"
        else
            echo -e "${YELLOW}$line${NC}"
        fi
    done
fi

echo ""

# Summary and next steps
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}Actions${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo "  gh pr view $PR_NUMBER --web     # Open in browser"
echo "  gh pr checkout $PR_NUMBER       # Checkout locally"
echo "  gh pr merge $PR_NUMBER --squash # Merge PR"
echo "  gh pr close $PR_NUMBER          # Close PR"

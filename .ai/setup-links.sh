#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# .ai/setup-links.sh — Create platform symlinks from .ai/ directory
# ─────────────────────────────────────────────────────────────────────────────
#
# Run this script after copying the .ai/ directory into your workspace root.
# It auto-detects which platform files exist in .ai/ and creates the
# symlinks each AI tool expects.
#
# Usage:
#   cd your-workspace
#   bash .ai/setup-links.sh          # Create symlinks
#   bash .ai/setup-links.sh --dry-run  # Preview without creating
#   bash .ai/setup-links.sh --clean    # Remove all managed symlinks
#
# ─────────────────────────────────────────────────────────────────────────────

set -euo pipefail

# Resolve workspace root (parent of .ai/)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(dirname "$SCRIPT_DIR")"
cd "$ROOT"

DRY_RUN=false
CLEAN=false

for arg in "$@"; do
  case "$arg" in
    --dry-run|-n) DRY_RUN=true ;;
    --clean)      CLEAN=true ;;
    --help|-h)
      echo "Usage: bash .ai/setup-links.sh [--dry-run] [--clean]"
      echo ""
      echo "  --dry-run  Preview symlinks without creating them"
      echo "  --clean    Remove all managed symlinks"
      exit 0
      ;;
  esac
done

LINKS_CREATED=0
LINKS_SKIPPED=0
LINKS_REMOVED=0

# ── Helpers ─────────────────────────────────────────────────────────────────

make_link() {
  local symlink_path="$1"   # e.g. "AGENTS.md"
  local target="$2"         # e.g. ".ai/AGENTS.md" (relative to symlink parent)
  local label="$3"          # e.g. "AGENTS.md (open standard)"

  # Resolve the actual target to verify it exists
  local symlink_dir
  symlink_dir="$(dirname "$symlink_path")"
  local resolved="$symlink_dir/$target"

  if [[ ! -e "$resolved" && ! -d "$resolved" ]]; then
    return 0  # Target doesn't exist in .ai/, skip silently
  fi

  if $DRY_RUN; then
    echo "  🔗 $symlink_path → $target  ($label)"
    ((LINKS_CREATED++)) || true
    return 0
  fi

  # Remove existing file/symlink/directory at the path
  if [[ -L "$symlink_path" || -e "$symlink_path" ]]; then
    if [[ -d "$symlink_path" && ! -L "$symlink_path" ]]; then
      rm -rf "$symlink_path"
    else
      rm -f "$symlink_path"
    fi
  fi

  # Create parent directories
  mkdir -p "$(dirname "$symlink_path")"

  # Create relative symlink
  ln -s "$target" "$symlink_path"
  echo "  🔗 $symlink_path → $target  ($label)"
  ((LINKS_CREATED++)) || true
}

remove_link() {
  local symlink_path="$1"
  local label="$2"

  if [[ -L "$symlink_path" ]]; then
    if $DRY_RUN; then
      echo "  🗑  Would remove $symlink_path  ($label)"
    else
      rm -f "$symlink_path"
      echo "  🗑  Removed $symlink_path  ($label)"
    fi
    ((LINKS_REMOVED++)) || true
  fi
}

# ── Clean mode ──────────────────────────────────────────────────────────────

if $CLEAN; then
  echo "🧹 Removing managed symlinks..."
  echo ""

  remove_link "AGENTS.md"                       "open standard"
  remove_link ".github/skills"                  "skills"
  remove_link ".github/copilot-instructions.md" "GitHub Copilot"
  remove_link ".github/agents"                  "GitHub Copilot"
  remove_link ".github/prompts"                 "GitHub Copilot"
  remove_link "opencode.json"                   "OpenCode"
  remove_link ".opencode/agents"                "OpenCode"
  remove_link ".opencode/skills"                "OpenCode"
  remove_link ".codex/config.toml"              "Codex CLI"
  remove_link ".codex/agents"                   "Codex CLI"
  remove_link ".codex/skills"                   "Codex CLI"
  remove_link "CLAUDE.md"                       "Claude Code"
  remove_link ".cursor/rules/project.mdc"       "Cursor"
  remove_link ".clinerules"                     "Cline"
  remove_link ".windsurfrules"                  "Windsurf"

  echo ""
  echo "✅ Done! $LINKS_REMOVED symlink(s) removed."
  exit 0
fi

# ── Create symlinks ─────────────────────────────────────────────────────────

MODE="Creating"
$DRY_RUN && MODE="Preview"

echo "🔗 $MODE platform symlinks → .ai/"
echo ""

# Open standard: AGENTS.md at workspace root
make_link "AGENTS.md" \
          ".ai/AGENTS.md" \
          "open standard"

# Skills: .github/skills/ (open standard)
make_link ".github/skills" \
          "../.ai/skills" \
          "skills"

# GitHub Copilot
make_link ".github/copilot-instructions.md" \
          "../.ai/copilot-instructions.md" \
          "GitHub Copilot"

make_link ".github/agents" \
          "../.ai/agents" \
          "GitHub Copilot"

make_link ".github/prompts" \
          "../.ai/prompts" \
          "GitHub Copilot"

# OpenCode
make_link "opencode.json" \
          ".ai/opencode/opencode.json" \
          "OpenCode"

make_link ".opencode/agents" \
          "../.ai/opencode/agents" \
          "OpenCode"

make_link ".opencode/skills" \
          "../.ai/skills" \
          "OpenCode"

# Codex CLI
make_link ".codex/config.toml" \
          "../.ai/codex/config.toml" \
          "Codex CLI"

make_link ".codex/agents" \
          "../.ai/codex/agents" \
          "Codex CLI"

make_link ".codex/skills" \
          "../.ai/skills" \
          "Codex CLI"

# Claude Code
make_link "CLAUDE.md" \
          ".ai/CLAUDE.md" \
          "Claude Code"

# Cursor
make_link ".cursor/rules/project.mdc" \
          "../../.ai/cursor-rules.mdc" \
          "Cursor"

# Cline
make_link ".clinerules" \
          ".ai/clinerules" \
          "Cline"

# Windsurf
make_link ".windsurfrules" \
          ".ai/windsurfrules" \
          "Windsurf"

echo ""
if $DRY_RUN; then
  echo "📋 Dry run: $LINKS_CREATED symlink(s) would be created."
else
  echo "✅ Done! $LINKS_CREATED symlink(s) created."
fi

if [[ $LINKS_CREATED -eq 0 ]]; then
  echo ""
  echo "⚠️  No symlinks created. Make sure .ai/ contains generated files."
  echo "   Run the toolkit generator first: python3 setup/generate.py"
fi

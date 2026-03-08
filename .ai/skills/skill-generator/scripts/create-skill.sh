#!/bin/bash
#
# Create a new skill following Agent Skills standard
#
# Usage:
#   ./create-skill.sh <skill-name>
#   ./create-skill.sh my-new-skill
#
# Creates:
#   .github/skills/<skill-name>/
#   ├── SKILL.md
#   ├── scripts/
#   ├── references/
#   └── assets/
#

set -e

SKILL_NAME="$1"

if [ -z "$SKILL_NAME" ]; then
    echo "Usage: ./create-skill.sh <skill-name>"
    echo "  skill-name: lowercase letters, numbers, and hyphens only"
    exit 1
fi

# Validate skill name (lowercase, letters, numbers, hyphens)
if ! [[ "$SKILL_NAME" =~ ^[a-z][a-z0-9-]*[a-z0-9]$ ]]; then
    echo "Error: Skill name must be lowercase, start with letter, contain only a-z, 0-9, -"
    echo "       Must not start or end with hyphen"
    exit 1
fi

SKILL_DIR=".github/skills/${SKILL_NAME}"

if [ -d "$SKILL_DIR" ]; then
    echo "Error: Skill already exists at $SKILL_DIR"
    exit 1
fi

echo "Creating skill: $SKILL_NAME"

# Create directory structure
mkdir -p "$SKILL_DIR/scripts"
mkdir -p "$SKILL_DIR/references"
mkdir -p "$SKILL_DIR/assets"

# Create SKILL.md
cat > "$SKILL_DIR/SKILL.md" << EOF
---
name: ${SKILL_NAME}
description: TODO - Describe what this skill does and when to use it. Be specific with keywords that help agents identify relevant tasks.
---

# ${SKILL_NAME^}

Run script: \`scripts/main.py <args>\`

See [reference](references/REFERENCE.md) for detailed documentation.

## Instructions

### When to Use This Skill

Use this skill when:
- TODO: Describe first trigger condition
- TODO: Describe second trigger condition

### How to Execute

1. TODO: Step one
2. TODO: Step two
3. TODO: Step three

## Examples

### Example 1: Basic Usage

\`\`\`typescript
// TODO: Add example code
\`\`\`

## Output Format

\`\`\`markdown
# ${SKILL_NAME^} Results

**Status**: ✅ Success / ❌ Issues Found

## Summary
TODO: Describe output format
\`\`\`
EOF

# Create placeholder script
cat > "$SKILL_DIR/scripts/main.py" << 'EOF'
#!/usr/bin/env python3
"""
Main script for the skill.

Usage:
    python main.py <args>
"""

import argparse
import sys

def main():
    parser = argparse.ArgumentParser(description="TODO: Description")
    parser.add_argument("path", help="Path to analyze")
    args = parser.parse_args()
    
    print(f"Analyzing: {args.path}")
    # TODO: Implement skill logic

if __name__ == "__main__":
    main()
EOF

# Create placeholder reference
cat > "$SKILL_DIR/references/REFERENCE.md" << 'EOF'
# Reference Documentation

Detailed reference documentation for this skill.

## Overview

TODO: Describe the domain knowledge this skill provides.

## Patterns

TODO: Add patterns, examples, and best practices.
EOF

# Create assets readme
cat > "$SKILL_DIR/assets/.gitkeep" << 'EOF'
# Assets folder for templates and static resources
EOF

chmod +x "$SKILL_DIR/scripts/main.py"

echo ""
echo "✅ Skill created at: $SKILL_DIR"
echo ""
echo "Structure:"
echo "  $SKILL_DIR/"
echo "  ├── SKILL.md           <- Edit skill instructions"
echo "  ├── scripts/"
echo "  │   └── main.py        <- Add executable scripts"
echo "  ├── references/"
echo "  │   └── REFERENCE.md   <- Add domain knowledge"
echo "  └── assets/"
echo "      └── .gitkeep       <- Add templates, static files"
echo ""
echo "Next steps:"
echo "  1. Edit SKILL.md with skill instructions"
echo "  2. Add scripts in scripts/"
echo "  3. Add reference docs in references/"
echo "  4. Register in copilot-instructions.md"

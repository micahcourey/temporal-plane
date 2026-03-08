---
name: skill-generator
description: Generate new GitHub Copilot Skills following the standard structure. Use when creating skills, converting prompts to skills, or discussing skill architecture.
---

# Skill Generator

Scaffold skill: `scripts/create-skill.sh <skill-name>`

See the **Skill Structure** section below for the standard template.

## What is a Skill?

Skills are reusable instruction sets that auto-activate based on context. Unlike agents (conversational, stateful) or prompts (manually invoked), skills activate automatically when relevant file patterns are detected.

### Skill vs Agent vs Prompt

| Aspect | Skill | Agent | Prompt |
|--------|-------|-------|--------|
| Activation | Auto (file patterns) | Manual (`@agent`) | Manual (invoke) |
| Scope | Narrow, specific task | Broad, conversational | Task template |
| State | Stateless | Stateful conversation | One-shot |
| Example | API endpoint generation | Full-stack engineering | Generate README |

## Skill Structure

Skills live in `.github/skills/{skill-name}/SKILL.md`:

```
.github/
└── skills/
    ├── api-endpoint/
    │   └── SKILL.md
    ├── frontend-component/
    │   └── SKILL.md
    └── unit-test/
        └── SKILL.md
```

## SKILL.md Format

```markdown
# Skill Name

Brief description of what this skill does.

## When This Skill Activates

This skill activates when:
- [File pattern condition 1]
- [File pattern condition 2]
- [Context condition]

## Instructions

[Detailed instructions for the AI when this skill is active]

### Step 1: Discovery
[What to analyze first]

### Step 2: Generate
[What to create]

### Step 3: Validate
[How to verify output]

## Examples

### Input
[Example input or context]

### Output
[Example output]

## Project Patterns

[Specific temporal-plane conventions to follow]
```

## Task

When asked to create a skill:

### 1. Gather Requirements

Ask for:
- **Purpose**: What task should this skill accomplish?
- **Activation**: What file patterns or contexts trigger it?
- **Output**: What should the skill generate?
- **Patterns**: What project-specific patterns should it follow?

### 2. Choose Skill Name

- Use kebab-case: `api-endpoint`, `unit-test`, `frontend-component`
- Keep concise but descriptive
- Match the task being automated

### 3. Define Activation Triggers

Skills activate on file patterns:

- `*.routes.ts` or `*.controller.ts` - API route files


- `*.component.ts` or `*.tsx` - Frontend component files

- `*.spec.ts` or `*.test.ts` - Test files
- `*.service.ts` - Service files

### 4. Generate SKILL.md

Create the skill file at `.github/skills/{name}/SKILL.md` with:
- Clear activation conditions
- Step-by-step instructions
- Project-specific patterns
- Code examples

## Skill Guidelines

### Reference Context Files

Skills should direct agents to the project knowledge routing table rather than listing all context files:
```markdown
## Context Files

Consult [AGENTS.md](AGENTS.md) for the project knowledge routing table.
**For this skill, prioritize**: [list relevant categories — e.g., security patterns, database schema, testing strategy].
Load companion data files (`.jsonl`, `.yaml`) only for specific lookups.
```

### Include Project Patterns

Every skill should enforce:


- Error handling with structured responses
- 80%+ test coverage for new code

### Provide Code Examples

Include both correct and incorrect examples:
```
// ✅ Correct - Following project patterns
[show correct pattern]

// ❌ Incorrect - Missing required patterns
[show anti-pattern]
```

## Response Format

When creating a skill:

```markdown
## Skill Created: {skill-name}

**Location**: `.github/skills/{skill-name}/SKILL.md`

**Activates on**:
- {pattern 1}
- {pattern 2}

**Purpose**: {brief description}

**Next Steps**:
1. Copy to target repositories
2. Test activation with relevant files
3. Iterate based on usage
```

---
name: create-agent
description: Create a custom GitHub Copilot agent following VS Code agent file format conventions
agent: agent
tools:
  - read_file
  - createFile
---

# Create Custom Copilot Agent

Create a custom GitHub Copilot agent following VS Code agent file format conventions.

## Agent Configuration

Use this format for the agent file:

```markdown
---
name: '[Agent Display Name]'
description: [Brief one-line description of agent's purpose]
argument-hint: [Optional - hint text for agent invocation]
tools: [List of tools the agent needs access to]
handoffs:
  - label: [Handoff Button Label]
    agent: [Target agent name or 'agent' for generic]
    prompt: [What to do in handoff]
    send: false
---

# [Agent Title]

[Agent persona/introduction - who the agent is and what expertise they have]

## [Key sections describing agent behavior, approach, expertise, etc.]

[Details about how the agent operates, what it checks for, patterns it follows, etc.]
```

## Agent Format Requirements

### Frontmatter (YAML)
- **name**: Display name shown in VS Code (use Title Case, can include spaces)
- **description**: One-line summary of what the agent does
- **argument-hint**: (Optional) Placeholder text for arguments
- **tools**: Array of tools the agent can use
  - Common tools: `codebase`, `read`, `search`, `changes`, `problems`, `edit`, `createFile`, `runCommands`
- **handoffs**: (Optional) Array of handoff buttons
  - `label`: Button text
  - `agent`: Target agent name (or `agent` for general coding agent)
  - `prompt`: What to ask the target agent to do
  - `send`: Usually `false` (doesn't auto-send, user clicks button)

### Content Structure
1. **Title (H1)**: Agent name matching the frontmatter
2. **Introduction**: Agent persona and expertise
3. **Key Sections**: Organize content logically
   - Approach/methodology
   - What the agent checks/generates
   - Standards/requirements
   - Code examples (if applicable)
   - Response format
   - Confidence guidelines

### Tool Selection Guide

**Analysis/Scanning Agents**:
- `codebase` - Search across files
- `read` - Read file contents
- `search` - Text search
- `changes` - View git changes
- `problems` - View errors/warnings

**Generation Agents**:
- `codebase` - Discover patterns
- `read` - Read existing code
- `search` - Find similar features
- `createFile` - Create new files
- `createDirectory` - Create folders
- `edit` - Modify existing files
- `list` - List directory contents

**Testing/Validation Agents**:
- `runCommands` - Execute test commands
- `read` - Read coverage reports
- `search` - Find test files
- `codebase` - Locate untested code
- `changes` - Check what changed

### Handoff Patterns

**Scanner/Analyzer Agents** typically offer:
1. **Fix Issues** → Generic agent to implement fixes
2. **Save Report** → Generic agent to save detailed findings

**Generator Agents** typically offer:
1. **Validate Output** → Specific agents (Security, Accessibility, Test Coverage)

## Project-Specific Guidelines

### Reference Context Files
Agents should use progressive disclosure to reference project context files:
```markdown
## Context & Instructions

This agent follows **progressive disclosure**. Do not load all context files upfront.

1. Read [AGENTS.md](AGENTS.md) for the full routing table of instructions and project knowledge
2. Load only the instruction modules and context files relevant to the current task
3. Load companion data files (`.jsonl`, `.yaml`) only when you need specific lookups
```

### Include Project Patterns
Every agent should be aware of:


- Error handling patterns
- Testing conventions (80%+ coverage)
- conventional commit format

## Output

Save the agent file to:
```
.github/agents/{agent-name}.agent.md
```

Test by:
1. Reload VS Code window
2. Open Copilot Chat
3. Type `@` to see available agents
4. Invoke with `@{agent-name}`

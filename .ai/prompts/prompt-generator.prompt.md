---
name: prompt-generator
description: Generate new reusable prompts tailored for temporal-plane workspace following proven patterns
agent: agent
tools:
  - semantic_search
  - read_file
  - list_dir
---

# Workspace Prompt Generator

Generate a new reusable prompt file (.prompt.md) for the temporal-plane workspace that follows proven effective prompt patterns and project conventions.

## Input Requirements

When requesting a new prompt, provide:

1. **Purpose**: What task should this prompt accomplish?
2. **Scope**: What repositories/project types does it apply to?
3. **Outputs**: What should the prompt generate (code, documentation, analysis)?
4. **Context Needed**: What information must be gathered to complete the task?

## Task

Create a new `.prompt.md` file that follows this structure:

### 1. YAML Frontmatter

```yaml
---
name: short-command-name
description: One-line description with examples of target repos
agent: agent
tools:
  - semantic_search
  - read_file
  - list_dir
  - grep_search
---
```

**Naming Guidelines:**
- Use kebab-case for command names
- Keep under 20 characters
- Be specific but concise

**Tool Selection:**
- `semantic_search` - For finding code patterns, similar implementations
- `read_file` - For analyzing specific files
- `list_dir` - For discovering project structure
- `grep_search` - For finding specific strings/patterns
- `file_search` - For finding files by name pattern

### 2. Header Section

```markdown
# Prompt Title (Human-Readable)

Brief description of what this prompt does and when to use it.

---

## Context

Describe:
- What type of projects/repositories this applies to
- Any prerequisites or assumptions
```

### 3. Input Parameters Section (if applicable)

```markdown
## Input Parameters

- `${input:projectType}` - Type of project (api, library, ui, lambda)
- `${input:featureName}` - Name of feature being analyzed
- `${file}` - Currently open file (if relevant)
- `${selection}` - Selected code (if relevant)
```

### 4. Discovery Phase

```markdown
## Discovery Phase

1. **Analyze [specific files]**:
   - What to look for
   - Key information to extract

2. **Search codebase for [patterns]**:
   - Use semantic_search for: [describe]
   - Use grep_search for: [describe]

3. **Review [documentation/configs]**:
   - What files to check
   - What information is needed
```

### 5. Task Execution Section

```markdown
## Task Execution

Generate [output type] with the following structure:

1. **Section 1 Name**
   - What to include
   - Format/structure

2. **Section 2 Name**
   - What to include
   - Project-specific conventions to follow
```

### 6. Project-Specific Guidelines

```markdown
## Project Conventions to Follow


- **Technology Stack**: None (Rust library project), Native CLI / library, TypeScript strict mode
- **Testing**: 80% coverage minimum
- **Documentation**: Use actual project enums and types
```

### 7. Output Format Section

```markdown
## Output Format

Create a file named: `[naming-pattern].md` or `[naming-pattern].ts`

Structure:
[template]
```

### 8. Confidence Scoring Guidance

```markdown
## Confidence Scoring

**High (90-100%)**: Pattern exists in multiple repos, conventions clear
**Medium (70-89%)**: Some assumptions required, pattern has variations
**Low (50-69%)**: Limited examples, new pattern being established
```

## Prompt Writing Best Practices

### DO:
- Start with clear, actionable task description
- Use task-first structure (what, then how)
- Include project-specific examples and patterns
- Specify exact file locations and naming conventions
- Define expected outputs with examples
- List required tools explicitly
- Break complex tasks into discovery → execution → validation phases
- Include confidence scoring guidance

### DON'T:
- Use generic examples instead of project-specific ones
- Start with lengthy role/persona descriptions
- Write vague instructions like "analyze the project"
- Forget to specify output file locations
- Skip tool requirements in YAML frontmatter
- Create prompts longer than 500 lines (split if needed)
- Duplicate instructions already in copilot-instructions.md

## Prompt Categories

| Category | Examples |
|----------|----------|
| **Code Generation** | Component scaffolding, API endpoints, config templates |
| **Documentation** | README generation, API docs, architecture diagrams |
| **Analysis** | Security audits, code quality, dependency analysis |
| **Refactoring** | Pattern standardization, migrations, convention alignment |
| **Testing** | Test generation, coverage improvement, test data creation |

## Validation Checklist

- [ ] YAML frontmatter is complete and valid
- [ ] Command name is short, descriptive, unique
- [ ] Tools list includes all required tools
- [ ] Discovery phase is specific and actionable
- [ ] Project conventions are explicitly referenced
- [ ] Output format and location are specified
- [ ] Confidence scoring guidance included

## Output

Save the new prompt file to:
```
.github/prompts/{new-prompt-name}.prompt.md
```

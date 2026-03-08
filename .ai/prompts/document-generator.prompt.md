---
name: document-generator
description: Generate structured documents (PRD, RFC, ADR, README, runbook, user story, etc.) using standardized templates
agent: agent
tools: ['codebase', 'read', 'createFile', 'list', 'search', 'edit']
---

# Document Generator

Generate a polished, project-aware document using standardized templates.

## Step 1: Determine Document Type

If the user specified a document type (e.g., "generate a README", "write a PRD", "create an RCA"), proceed to Step 2.

If no document type was specified, ask:

> What type of document would you like me to generate? Available templates include:
>
> **Product**: PRD, User Story, Epic, Bug Ticket
> **Design**: RFC, ADR, Technical Design, Feature Technical Doc, API Specification
> **Operations**: README, Runbook, Migration Guide, Troubleshooting Guide, User Guide
> **Quality**: Test Plan, Security Assessment, RCA
> **Communication**: Release Notes, Retrospective, Case Study, White Paper
>
> You can also describe what you need and I'll match it to the best template.

## Step 2: Load the Skill

Read [skills/document-generator/SKILL.md](skills/document-generator/SKILL.md) and follow its instructions:

1. **Identify** — Match the request to a template using the alias mapping in Step 1 of the skill
2. **Load** — Read the matching reference template from `skills/document-generator/references/`
3. **Gather** — Search the codebase and load relevant context files (architecture, schema, access control, etc.)
4. **Populate** — Fill every section with real, project-specific content — no leftover placeholders
5. **Missing template** — If no template matches, generate the document and offer to create a reusable template

## Guidelines

- Analyze the actual project (package.json, src/ structure, config files, CI/CD) before writing
- Remove sections that don't apply rather than writing "N/A"
- Add sections as needed — templates are starting points, not rigid constraints
- Use project conventions for ticket IDs, branch patterns, and coverage targets

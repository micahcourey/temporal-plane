---
name: document-generator
description: Generate structured documents using standardized templates. Covers PRDs, RFCs, ADRs, user stories, runbooks, READMEs, and more. Use when creating any project documentation.
---

# Document Generator

Generate polished, project-aware documents from standardized reference templates for software development, product management, and technical writing.

## When This Skill Activates

This skill activates when:
- Creating or drafting any document (PRD, RFC, ADR, README, runbook, etc.)
- Writing user stories, epics, or bug tickets
- Generating release notes, migration guides, or technical designs
- Producing case studies, white papers, or RCA reports
- Any request involving structured document generation

## Context Files

Consult [AGENTS.md](AGENTS.md) for the project knowledge routing table.
**For this skill, prioritize**: Repository index and system architecture context for technical docs; domain glossary and role/permission context for product docs.

## Template Catalog

| Template | Reference File | Use When |
|----------|---------------|----------|
| **Product Requirements Document** | [references/prd.md](references/prd.md) | Defining feature requirements, objectives, success metrics |
| **User Story** | [references/user-story.md](references/user-story.md) | Writing stories with Given/When/Then acceptance criteria |
| **Epic** | [references/epic.md](references/epic.md) | Defining epics with child story breakdown |
| **Bug Ticket** | [references/bug-ticket.md](references/bug-ticket.md) | Reporting bugs with repro steps, expected vs actual |
| **RFC** | [references/rfc.md](references/rfc.md) | Proposing technical changes for team review |
| **ADR** | [references/adr.md](references/adr.md) | Recording architecture decisions with context |
| **Retrospective** | [references/retrospective.md](references/retrospective.md) | Sprint/project retrospectives |
| **Technical Design** | [references/technical-design.md](references/technical-design.md) | Detailed implementation design documents |
| **Feature Technical Doc** | [references/feature-technical-doc.md](references/feature-technical-doc.md) | Feature-specific technical documentation |
| **API Specification** | [references/api-specification.md](references/api-specification.md) | Endpoint documentation (OpenAPI-style) |
| **README** | [references/readme.md](references/readme.md) | Repository overview, setup, and usage |
| **Runbook** | [references/runbook.md](references/runbook.md) | Operational procedures for incidents/deployments |
| **Migration Guide** | [references/migration-guide.md](references/migration-guide.md) | Step-by-step upgrade/migration instructions |
| **Troubleshooting Guide** | [references/troubleshooting-guide.md](references/troubleshooting-guide.md) | Common issues and resolution steps |
| **User Guide** | [references/user-guide.md](references/user-guide.md) | End-user documentation for features/tools |
| **Test Plan** | [references/test-plan.md](references/test-plan.md) | Test strategy, scope, acceptance criteria |
| **Security Assessment** | [references/security-assessment.md](references/security-assessment.md) | Threat model and security review |
| **RCA (Root Cause Analysis)** | [references/rca.md](references/rca.md) | Incident root cause, impacts, corrective actions |
| **Case Study** | [references/case-study.md](references/case-study.md) | Problem, solution, results narrative |
| **White Paper** | [references/white-paper.md](references/white-paper.md) | In-depth technical or strategic position paper |
| **Release Notes (PRR)** | [references/release-notes.md](references/release-notes.md) | Production Readiness Review — release schedule, scope, test results, deployment checklist |

## Instructions

### Step 1: Identify the Document Type

Match the user's request to a template from the catalog above. Common aliases:
- "spec" / "requirements" / "product spec" → PRD
- "story" / "ticket" / "user story" → User Story
- "bug" / "defect" / "issue" → Bug Ticket
- "proposal" / "design proposal" → RFC
- "decision record" / "architecture decision" → ADR
- "design doc" / "tech spec" → Technical Design
- "feature doc" / "feature documentation" → Feature Technical Doc
- "ops doc" / "playbook" / "operations guide" → Runbook
- "upgrade guide" / "migration plan" → Migration Guide
- "root cause" / "incident review" / "postmortem" → RCA
- "readme" / "project docs" → README
- "changelog" / "what's new" / "PRR" / "production readiness" → Release Notes (PRR)
- "how-to" / "user manual" / "end-user docs" → User Guide

### Step 2: Load the Template

Read the matching reference file from `references/`. Use it as the **structural starting point** — every section heading and structural element in the template should appear in the output.

### Step 3: Gather Context

Before populating the template:
1. **Read AGENTS.md** for project knowledge routing
2. **Search the codebase** for relevant implementation details
3. **Load context files** appropriate to the document type:
   - Technical docs → System Architecture, Database Schema, API Reference
   - Product docs → Domain Glossary, Role/Permission Matrix
   - Compliance docs → Access Control, Security Patterns

### Step 4: Populate and Deliver

Fill in every section of the template with project-specific content. Guidelines:
- **Never leave placeholder text** — replace all bracketed `[placeholders]` with real content
- **Remove inapplicable sections** — if a section doesn't apply, remove it rather than writing "N/A"
- **Add sections as needed** — templates are starting points, not rigid constraints
- **Use project conventions** — ticket IDs should follow `<agent>-<model>/<type>/<task-id>-<description>`, coverage targets should reference `80`

### Step 5: Missing Template Handling

If the user requests a document type that **does not have a matching template** in the catalog:

1. Generate the document using your best judgment and any similar templates as reference
2. After delivering the document, ask the user:

> "This document type doesn't have a standardized template yet. Would you like me to create a reusable template based on this document so future documents of this type follow a consistent structure?"

If the user agrees, create the new template as a `.md` file in the `references/` directory following the conventions of existing templates.

## Template Conventions

All reference templates follow these conventions:
- **Title block** with document metadata (author, date, status, version)
- **Section headings** using `##` for major sections, `###` for subsections
- **Placeholder text** in `[brackets]` indicating what to fill in
- **Tables** for structured data (requirements, risks, endpoints)
- **Checklists** (`- [ ]`) for action items and criteria
- **No template engine variables in reference templates** — `references/*.md` files are plain markdown, populated by the agent at runtime

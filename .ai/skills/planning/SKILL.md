---
name: planning
description: Create technical implementation plans from tickets. Use when starting new features, breaking down tickets, or estimating work.
---

# Planning Skill

Generate comprehensive implementation plans from tickets with acceptance criteria.

## When This Skill Activates

This skill activates when:
- Discussing tickets (<agent>-<model>/<type>/<task-id>-<description>)
- Starting new feature development
- Breaking down work into subtasks
- Estimating complexity or effort
- Creating implementation checklists

## Context Files

Consult [AGENTS.md](AGENTS.md) for the project knowledge routing table.
**For this skill, prioritize**: System architecture, repository index, and database schema context files. Load companion data files for repo inventory and schema lookups.

## Instructions

### Input Requirements

To create an implementation plan, gather:
1. **Ticket ID**: <agent>-<model>/<type>/<task-id>-<description>
2. **Title/Summary**: Brief description
3. **Acceptance Criteria**: What defines "done"
4. **Context**: Any design docs, Figma links, or related tickets

### Output Format

```markdown
# Implementation Plan: <agent>-<model>/<type>/<task-id>-<description>

## Summary
[1-2 sentence overview of the feature]

## Scope Analysis

### Affected Areas
| Area | Changes Required |
|------|-----------------|
| Backend API | New endpoints, service, controller |
| Frontend | New component, service |
| Database | Schema migration |

### Affected Layers
- [ ] API Layer (routes, controllers, middleware)
- [ ] Service Layer (business logic)
- [ ] Database Layer (schema, queries)
- [ ] Frontend Components
- [ ] Frontend Services (API calls)
- [ ] Tests (unit, E2E)
- [ ] Documentation

## Technical Design

### API Endpoints
| Method | Endpoint | Description | Permission Required |
|--------|----------|-------------|-------------------|
| POST | `/api/resource` | Create resource | `CREATE_RESOURCE` |

### Frontend Components
| Component | Purpose | Location |
|-----------|---------|----------|
| `ResourceList` | Display list | TBD |

## Implementation Checklist

### Phase 1: Backend
- [ ] Database migration (if needed)
- [ ] Create service with business logic
- [ ] Create controller with validation
- [ ] Create routes with middleware chain
- [ ] Add unit tests

### Phase 2: Frontend
- [ ] Create API service
- [ ] Create components
- [ ] Add authorization checks
- [ ] Add routing
- [ ] Add unit tests

### Phase 3: Testing & Compliance
- [ ] Run security scan
- [ ] Run accessibility audit
- [ ] Verify 80%+ coverage

### Phase 4: Documentation
- [ ] Update API documentation
- [ ] Update README if applicable

## Complexity Estimate

| Metric | Estimate |
|--------|----------|
| **Story Points** | X |
| **Development Time** | X days |
| **Risk Level** | Low/Medium/High |
| **Dependencies** | List any blockers |

## Considerations


- [ ] New permissions needed?
- [ ] Accessibility requirements?
```

### Discovery Process

Before generating the plan:
1. **Search for similar features** in the codebase
2. **Review context files** for patterns and standards
3. **Identify reusable patterns** from existing implementations

### Complexity Scoring Guide

| Points | Criteria |
|--------|----------|
| **1** | Simple change, single file, no new patterns |
| **2** | Minor feature, 2-3 files, existing patterns |
| **3** | Small feature, 4-6 files, minor new patterns |
| **5** | Medium feature, full-stack, moderate complexity |
| **8** | Large feature, multiple integrations |
| **13** | Very large, consider splitting |

# Migration Guide

| Field | Value |
|-------|-------|
| **Migration** | [What is being migrated — e.g., Angular 17 → 18, Node 18 → 20] |
| **Author** | [Name] |
| **Date** | [YYYY-MM-DD] |
| **JIRA Ticket** | [TICKET-ID] |
| **Risk Level** | Low / Medium / High |

---

## Summary

[1-2 paragraphs describing what is changing and why.]

## Impact Assessment

### Affected Repositories

| Repository | Impact | Priority |
|------------|--------|----------|
| [repo-name] | [What changes] | [Order of migration] |

### Breaking Changes

| Change | Before | After | Action Required |
|--------|--------|-------|-----------------|
| [API change] | [Old behavior] | [New behavior] | [What to update] |

### Non-Breaking Changes

- [Change that is backward-compatible]

---

## Prerequisites

- [ ] [Prerequisite 1 — e.g., Install Node.js 20]
- [ ] [Prerequisite 2 — e.g., Update CI/CD pipeline]
- [ ] [Prerequisite 3 — e.g., Back up database]

## Migration Steps

### Step 1: [Preparation]

```bash
# Commands to run
```

[Explanation of what this step does and what to verify.]

### Step 2: [Core Migration]

```bash
# Commands to run
```

[Explanation and expected output.]

### Step 3: [Update Dependencies]

```bash
# Commands to run
```

### Step 4: [Code Changes]

| File | Change | Description |
|------|--------|-------------|
| [file path] | [What to change] | [Why] |

**Before**:
```typescript
// Old code
```

**After**:
```typescript
// New code
```

### Step 5: [Verify]

```bash
# Run tests
npm test

# Build
npm run build

# Smoke test
npm run start:dev
```

---

## Database Migration

> Include this section if schema changes are required.

### Liquibase Changeset

```yaml
databaseChangeLog:
  - changeSet:
      id: TICKET-ID-migration-description
      author: [author]
      changes:
        - [changeset details]
```

### Data Migration

```sql
-- SQL to migrate existing data
```

### Verification Query

```sql
-- Query to confirm migration succeeded
```

---

## Rollback Plan

### Automatic Rollback Triggers

- [ ] Tests fail after migration
- [ ] Build fails
- [ ] Health check failures in deployed environment

### Rollback Steps

1. [Step to revert — e.g., git revert, redeploy previous version]
2. [Database rollback if applicable]
3. [Verify rollback succeeded]

```bash
# Rollback commands
```

---

## Testing Checklist

- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] E2E tests pass
- [ ] Manual smoke test in DEV
- [ ] Performance benchmarks acceptable
- [ ] No new lint/type errors

## Post-Migration Verification

| Check | Expected | Command |
|-------|----------|---------|
| Build succeeds | Exit 0 | `npm run build` |
| Tests pass | 100% | `npm test` |
| Application starts | Health 200 | `curl localhost:3000/health` |
| [Feature-specific] | [Expected] | [Command] |

## Timeline

| Phase | Date | Status |
|-------|------|--------|
| DEV migration | [Date] | Not Started |
| QA testing | [Date] | Not Started |
| STAGING deploy | [Date] | Not Started |
| PROD deploy | [Date] | Not Started |

## References

- [Official migration guide link]
- [Changelog / release notes link]
- [Related internal docs]

# Technical Design Document

| Field | Value |
|-------|-------|
| **Title** | [Feature / System Name] |
| **Author** | [Name] |
| **Status** | Draft / In Review / Approved |
| **Date** | [YYYY-MM-DD] |
| **JIRA Ticket** | [TICKET-ID] |
| **Reviewers** | [Names] |

---

## 1. Overview

[High-level summary of what is being built and why. 2-3 paragraphs.]

## 2. Goals & Non-Goals

### Goals
- [What this design achieves]

### Non-Goals
- [What is explicitly out of scope]

## 3. Background

[Context needed to understand this design. Links to PRDs, RFCs, or prior art.]

## 4. Architecture

### System Context Diagram

```
[Diagram showing where this component fits in the broader system]
```

### Component Diagram

```
[Internal structure of the component being designed]
```

### Key Components

| Component | Responsibility |
|-----------|---------------|
| [Component A] | [What it does] |
| [Component B] | [What it does] |

## 5. Data Model

### New Tables / Schema Changes

```sql
-- Example schema
CREATE TABLE TABLE_NAME (
  ID BIGINT PRIMARY KEY,
  AGREEMENT_ID BIGINT NOT NULL,
  -- columns
  CREATED_DT TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  CREATED_BY_USER_ID BIGINT NOT NULL,
  MODIFIED_DT TIMESTAMP,
  MODIFIED_BY_USER_ID BIGINT
);
```

### Data Flow

[Describe how data moves through the system.]

## 6. API Design

### Endpoints

| Method | Path | Description | Auth |
|--------|------|-------------|------|
| GET | `/api/resource` | List resources | `VIEW_RESOURCE` |
| POST | `/api/resource` | Create resource | `CREATE_RESOURCE` |

### Request/Response Examples

```json
// POST /api/resource
{
  "field": "value"
}
```

## 7. Security Considerations

- **Authentication**: [How users are authenticated]
- **Authorization**: [Privilege checks required]
- **Data Isolation**: [ACO / agreement_id filtering]
- **PHI Handling**: [Any PHI fields and protection]

## 8. Error Handling

| Scenario | HTTP Status | Error Code | Message |
|----------|-------------|------------|---------|
| [Error case] | [Status] | [Code] | [Message] |

## 9. Performance

- **Expected Load**: [Requests/sec, data volume]
- **Latency Targets**: [p50, p95, p99]
- **Scaling Strategy**: [How the system scales]
- **Caching**: [What is cached and TTLs]

## 10. Testing Strategy

| Test Type | Coverage | Tools |
|-----------|----------|-------|
| Unit | [Target %] | Jasmine/Karma, Jest |
| Integration | [Scope] | Supertest |
| E2E | [Key flows] | Gauge/Taiko |

## 11. Migration & Rollout

- **Database Migration**: [Liquibase changesets needed]
- **Feature Flags**: [Any toggles for gradual rollout]
- **Rollback Plan**: [How to revert if issues arise]
- **Deployment Sequence**: [Order of deployment across services]

## 12. Dependencies

| Dependency | Type | Risk |
|------------|------|------|
| [Service/Library] | Runtime / Build | [Impact if unavailable] |

## 13. Monitoring & Observability

- **Metrics**: [Key metrics to track]
- **Alerts**: [Threshold-based alerts]
- **Dashboards**: [CloudWatch dashboards needed]
- **Logging**: [Key log events]

## 14. Open Questions

| # | Question | Owner | Status |
|---|----------|-------|--------|
| 1 | [Question] | [Name] | Open / Resolved |

## 15. Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| [Date] | [What was decided] | [Why] |

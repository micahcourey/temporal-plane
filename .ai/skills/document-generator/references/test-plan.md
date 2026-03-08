# Test Plan

| Field | Value |
|-------|-------|
| **Feature / Release** | [Name] |
| **JIRA Ticket** | [TICKET-ID] |
| **Author** | [Name] |
| **Date** | [YYYY-MM-DD] |
| **Status** | Draft / In Review / Approved |

---

## 1. Objective

[What is being tested and why. Describe the scope of testing and the quality goals.]

## 2. Scope

### In Scope

- [Feature / component to test]
- [Feature / component to test]

### Out of Scope

- [What is explicitly not tested in this plan]

## 3. Test Strategy

| Test Type | Coverage Target | Framework | Responsible |
|-----------|----------------|-----------|-------------|
| Unit | 90%+ | Jasmine/Karma | Developer |
| Integration | Key flows | Supertest / Jest | Developer |
| E2E | Critical paths | Gauge/Taiko | QA |
| Accessibility | WCAG 2.2 AA | axe-core | Developer |
| Performance | Latency SLAs | k6 / Artillery | DevOps |
| Security | Auth & authz | Manual review | Security |

## 4. Test Environment

| Environment | URL | Database | Purpose |
|-------------|-----|----------|---------|
| DEV | [URL] | [DB instance] | Developer testing |
| QA | [URL] | [DB instance] | QA validation |
| STAGING | [URL] | [DB instance] | Pre-production |

### Test Data Requirements

- [Description of test data needed]
- [How test data will be created — fixtures, seeds, manual]
- [PHI considerations — no real PHI in lower environments]

## 5. Test Cases

### 5.1 Happy Path

| # | Scenario | Steps | Expected Result | Priority |
|---|----------|-------|-----------------|----------|
| TC-01 | [Scenario name] | [Steps] | [Expected outcome] | P1 |
| TC-02 | [Scenario name] | [Steps] | [Expected outcome] | P1 |
| TC-03 | [Scenario name] | [Steps] | [Expected outcome] | P2 |

### 5.2 Error / Edge Cases

| # | Scenario | Steps | Expected Result | Priority |
|---|----------|-------|-----------------|----------|
| TC-10 | [Invalid input] | [Steps] | [Error message displayed] | P1 |
| TC-11 | [Unauthorized access] | [Steps] | [403 returned] | P1 |
| TC-12 | [Empty state] | [Steps] | [Empty state UI shown] | P2 |

### 5.3 Access Control

| # | Role | Action | Expected | Pass/Fail |
|---|------|--------|----------|-----------|
| AC-01 | CMS Admin | View data | Allowed | |
| AC-02 | Read-Only User | Edit data | Denied (403) | |
| AC-03 | ACO User A | View ACO B data | Denied (filtered) | |

### 5.4 Accessibility

| # | Check | Criteria | Tool |
|---|-------|----------|------|
| A11Y-01 | Keyboard navigation | All interactive elements focusable | Manual |
| A11Y-02 | Screen reader | ARIA labels present | axe-core |
| A11Y-03 | Color contrast | 4.5:1 minimum | axe-core |

## 6. Acceptance Criteria

- [ ] All P1 test cases pass
- [ ] Unit test coverage >= 90%
- [ ] No P1/P2 accessibility violations
- [ ] No security vulnerabilities
- [ ] Performance within SLA targets
- [ ] All E2E critical paths pass

## 7. Risks & Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| [Risk description] | High / Medium / Low | High / Medium / Low | [Action] |

## 8. Dependencies

| Dependency | Status | Impact if Delayed |
|------------|--------|-------------------|
| [API endpoint ready] | [Status] | [Impact] |
| [Test data available] | [Status] | [Impact] |

## 9. Schedule

| Phase | Start | End | Status |
|-------|-------|-----|--------|
| Test plan review | [Date] | [Date] | |
| Unit testing | [Date] | [Date] | |
| Integration testing | [Date] | [Date] | |
| E2E testing | [Date] | [Date] | |
| UAT | [Date] | [Date] | |

## 10. Sign-Off

| Role | Name | Date | Approval |
|------|------|------|----------|
| QA Lead | | | |
| Dev Lead | | | |
| Product Owner | | | |

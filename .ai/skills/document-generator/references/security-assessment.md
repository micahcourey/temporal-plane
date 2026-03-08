# Security Assessment

| Field | Value |
|-------|-------|
| **System / Feature** | [Name] |
| **Assessment Date** | [YYYY-MM-DD] |
| **Assessor** | [Name] |
| **JIRA Ticket** | [TICKET-ID] |
| **Risk Rating** | Low / Medium / High / Critical |

---

## 1. Overview

[Brief description of the system or feature being assessed and its role in the broader architecture.]

### Assessment Scope

- [Component / service included]
- [Component / service included]

### Out of Scope

- [What is not covered by this assessment]

## 2. Architecture & Data Flow

```
[Diagram showing data flow, trust boundaries, and external interfaces]
```

### Trust Boundaries

| Boundary | Description |
|----------|-------------|
| [Internet → API Gateway] | External user requests |
| [API Gateway → Lambda] | Internal service communication |
| [Lambda → Aurora] | Database access |

## 3. Authentication

| Aspect | Implementation | Status |
|--------|---------------|--------|
| Identity Provider | Okta SSO | Verified |
| Token Type | JWT | Verified |
| Token Validation | `authMiddleware.verifyTokenRetrieveUser` | Verified |
| Session Management | [Description] | [Status] |
| MFA | [Required / Optional / N/A] | [Status] |

### Findings

- [Finding about auth implementation]

## 4. Authorization

| Aspect | Implementation | Status |
|--------|---------------|--------|
| Model | Role-based (RBAC) | Verified |
| Middleware | `permissionsMiddleware.validateRequiredPrivilege` | Verified |
| ACO Isolation | `agreement_id` filtering | Verified |
| Endpoint Coverage | [% of endpoints with privilege checks] | [Status] |

### Endpoint Authorization Audit

| Endpoint | Method | Privilege | Isolation | Status |
|----------|--------|-----------|-----------|--------|
| `/api/resource` | GET | `VIEW_RESOURCE` | agreement_id | Pass |
| `/api/resource` | POST | `CREATE_RESOURCE` | agreement_id | Pass |
| [Endpoint] | [Method] | [Privilege] | [Isolation] | [Status] |

### Findings

- [Finding about authorization gaps]

## 5. Data Protection

### PHI Inventory

| Field | Table | Classification | Protection |
|-------|-------|---------------|------------|
| [Field name] | [Table] | PHI / PII / Sensitive | [Encryption, masking, etc.] |

### Encryption

| Data State | Method | Status |
|------------|--------|--------|
| At Rest | AES-256 (RDS) | Verified |
| In Transit | TLS 1.2+ | Verified |
| In Logs | [Redacted / Not logged] | [Status] |

### Data Retention

| Data Type | Retention Period | Deletion Method |
|-----------|-----------------|-----------------|
| [Type] | [Period] | [Automated / Manual] |

## 6. Input Validation

| Input Point | Validation | Sanitization | Status |
|-------------|------------|-------------- |--------|
| [API body] | [Schema validation] | [XSS filtering] | [Status] |
| [Query params] | [Type checking] | [SQL injection prevention] | [Status] |
| [File uploads] | [Type/size limits] | [Malware scanning] | [Status] |

## 7. Threat Model

### STRIDE Analysis

| Threat | Category | Description | Likelihood | Impact | Mitigation |
|--------|----------|-------------|------------|--------|------------|
| T1 | Spoofing | [Description] | [L/M/H] | [L/M/H] | [Control] |
| T2 | Tampering | [Description] | [L/M/H] | [L/M/H] | [Control] |
| T3 | Repudiation | [Description] | [L/M/H] | [L/M/H] | [Control] |
| T4 | Information Disclosure | [Description] | [L/M/H] | [L/M/H] | [Control] |
| T5 | Denial of Service | [Description] | [L/M/H] | [L/M/H] | [Control] |
| T6 | Elevation of Privilege | [Description] | [L/M/H] | [L/M/H] | [Control] |

## 8. Dependency Analysis

### Third-Party Dependencies

| Package | Version | Known Vulnerabilities | Action |
|---------|---------|----------------------|--------|
| [Package] | [Version] | [CVE if any] | [Update / Accept / Mitigate] |

### Supply Chain

- [ ] Lock file committed (`package-lock.json`)
- [ ] `npm audit` clean
- [ ] No `eval()` or dynamic code execution
- [ ] No hardcoded secrets

## 9. Audit Logging

| Event | Logged | Details Captured | Status |
|-------|--------|-----------------|--------|
| Login | Yes | User ID, timestamp, IP | Verified |
| Data access | [Yes/No] | [Details] | [Status] |
| Data modification | [Yes/No] | [Details] | [Status] |
| Admin actions | [Yes/No] | [Details] | [Status] |

## 10. Findings Summary

| # | Severity | Finding | Recommendation | Status |
|---|----------|---------|----------------|--------|
| F1 | Critical / High / Medium / Low | [Finding] | [Recommendation] | Open / Resolved |
| F2 | [Severity] | [Finding] | [Recommendation] | [Status] |

## 11. Recommendations

### Immediate (P1)
- [Action required before deployment]

### Short-Term (P2)
- [Action required within 30 days]

### Long-Term (P3)
- [Improvement for future sprints]

## 12. Sign-Off

| Role | Name | Date | Approval |
|------|------|------|----------|
| Security Lead | | | |
| Dev Lead | | | |
| Compliance | | | |

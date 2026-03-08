# Root Cause Analysis (RCA)

| Field | Value |
|-------|-------|
| **Title** | [Brief description] |
| **JIRA Ticket** | [TICKET-ID] |
| **Author** | [Name] |
| **Date** | [YYYY-MM-DD] |

---

## Date of Incident

[Date the incident occurred or was first observed.]

## Reporter

[Who reported the incident and how it was discovered.]

## Problem Description

[Detailed description of the issue. What happened, what was expected, and how it was uncovered. Include enough context for someone unfamiliar with the system to understand the problem.]

## Root Cause

[Technical explanation of why the incident occurred. Reference specific code, configuration, logic, or process failures. Explain how the incorrect behavior was introduced and why it persisted.]

## Impacts

[Describe the scope and severity of the impact. Include:]

- [Number of users/entities affected]
- [Which features, services, or business functions were impacted]
- [Which models/products were affected, and which were confirmed unaffected]

## Corrective Action Short-Term Solution

[Describe the immediate fix being applied, including:]

- What the fix addresses
- When it will be deployed (sprint/date)
- Linked JIRA ticket(s): [TICKET-ID]
- Scope of the fix (single feature vs. cross-cutting)
- Any interim workaround until the fix is deployed

## Corrective Action Long-Term Solution

[Describe systemic improvements to prevent recurrence, such as:]

- Validation tests or regression test coverage
- Logic standardization across the platform
- Process improvements (e.g., acceptance criteria requirements)
- Monitoring or alerting enhancements

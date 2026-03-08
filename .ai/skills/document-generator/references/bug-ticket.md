# Bug Ticket

| Field | Value |
|-------|-------|
| **Title** | [Short description of the bug] |
| **Ticket** | [TICKET-ID] |
| **Reporter** | [Name] |
| **Date** | [YYYY-MM-DD] |
| **Severity** | Critical / High / Medium / Low |
| **Priority** | P1 / P2 / P3 / P4 |
| **Status** | Open / In Progress / Fixed / Verified / Closed |
| **Assignee** | [Name] |

---

## Description

[Clear, concise description of the bug. What is happening that shouldn't be, or what isn't happening that should be?]

## Steps to Reproduce

1. [Step 1 — starting state/precondition]
2. [Step 2 — action taken]
3. [Step 3 — action taken]
4. [Step 4 — observe the bug]

## Expected Behavior

[What should happen when following the steps above.]

## Actual Behavior

[What actually happens. Include exact error messages, incorrect values, or broken behavior.]

## Environment

| Factor | Value |
|--------|-------|
| Environment | Dev / Staging / Production |
| Browser | [Browser name and version] |
| OS | [Operating system and version] |
| User Role | [Role/persona experiencing the bug] |
| URL | [Page URL where bug occurs] |
| API Endpoint | [If applicable] |

## Evidence

### Screenshots / Screen Recordings
[Attach or embed screenshots, GIFs, or video recordings showing the bug]

### Console / Network Errors
```
[Paste relevant console errors, network responses, or stack traces]
```

### Logs
```
[Paste relevant server logs or application logs]
```

## Impact

- **Users affected**: [All users / Specific role / Specific ACO]
- **Frequency**: [Always / Intermittent / One-time]
- **Workaround available**: [Yes — describe / No]
- **Data impact**: [Data loss / Data corruption / Display only / None]

## Root Cause (if known)

[Description of the root cause, if already identified during investigation.]

## Proposed Fix

[If the fix is known, describe the approach. Otherwise, leave for the developer to fill in.]

## Regression Risk

- [Areas that might be affected by the fix]
- [Tests that should be run to verify no regression]

## Related

- [Related ticket IDs]
- [Related PRs or commits]
- [Similar past bugs]

## Definition of Done

- [ ] Bug is fixed and verified in the reported environment
- [ ] Unit test added to prevent regression
- [ ] No new console errors introduced
- [ ] Tested across supported browsers (if UI bug)
- [ ] Verified by reporter or QA

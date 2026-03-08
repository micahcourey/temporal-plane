# Troubleshooting Guide

| Field | Value |
|-------|-------|
| **Service / Feature** | [Name] |
| **Repository** | [repo-name] |
| **Last Updated** | [YYYY-MM-DD] |
| **Author** | [Name] |

---

## Quick Reference

| Symptom | Likely Cause | Jump To |
|---------|-------------|---------|
| [Error message or symptom] | [Most common cause] | [Link to section] |
| [Error message or symptom] | [Most common cause] | [Link to section] |
| [Error message or symptom] | [Most common cause] | [Link to section] |

---

## Issue 1: [Title — e.g., "Application fails to start"]

### Symptoms

- [What the user sees — error message, behavior, UI state]
- [Log output or console message]

### Possible Causes

1. **[Cause A]** — [Brief explanation]
2. **[Cause B]** — [Brief explanation]

### Diagnosis

```bash
# Commands to identify the root cause
```

[How to interpret the output.]

### Resolution

**For Cause A:**

```bash
# Fix commands
```

**For Cause B:**

```bash
# Fix commands
```

### Prevention

- [How to avoid this issue in the future]

---

## Issue 2: [Title — e.g., "403 Forbidden on API call"]

### Symptoms

- API returns `403 Forbidden`
- Frontend shows "You do not have permission" message

### Possible Causes

1. **Missing privilege** — User's role doesn't include the required privilege
2. **ACO mismatch** — User requesting data from a different agreement
3. **Token expired** — JWT has expired and wasn't refreshed

### Diagnosis

1. Check the user's role and privileges in IDM
2. Verify the `agreement_id` in the request matches the user's assignment
3. Decode the JWT to check expiration: `jwt.io`

### Resolution

**Missing privilege**: Assign the correct role via IDM Admin  
**ACO mismatch**: Verify the user's ACO assignment  
**Token expired**: Log out and log back in  

---

## Issue 3: [Title]

### Symptoms

- [Symptom]

### Possible Causes

1. **[Cause]** — [Explanation]

### Diagnosis

[Steps]

### Resolution

[Steps]

---

## Environment-Specific Issues

### Local Development

| Issue | Solution |
|-------|----------|
| `ECONNREFUSED` on database | Start local database or check `.env` DB_HOST |
| `npm install` fails | Delete `node_modules` and `package-lock.json`, retry |
| Port already in use | `lsof -i :PORT` to find and kill the process |

### DEV / QA Environment

| Issue | Solution |
|-------|----------|
| [Environment-specific issue] | [Resolution] |

### Production

| Issue | Solution | Escalation |
|-------|----------|------------|
| [Prod issue] | [First-response action] | [Who to contact] |

---

## Log Analysis

### Where to Find Logs

| Environment | Location |
|-------------|----------|
| Local | Console output / `logs/` directory |
| AWS Lambda | CloudWatch: `/aws/lambda/SERVICE_NAME` |
| ECS Fargate | CloudWatch: `/ecs/SERVICE_NAME` |

### Useful Log Queries

```
# CloudWatch Insights — find errors
fields @timestamp, @message
| filter @message like /ERROR/
| sort @timestamp desc
| limit 50
```

```
# Find slow requests
fields @timestamp, @duration
| filter @duration > 5000
| sort @duration desc
| limit 20
```

## Getting Help

If these steps don't resolve the issue:

1. Collect: error messages, logs, reproduction steps, environment
2. Post in: [Slack channel]
3. Create a JIRA ticket with the `bug` label
4. If P1/P2: Page on-call via [PagerDuty / escalation process]

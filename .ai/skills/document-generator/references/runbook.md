# Runbook

| Field | Value |
|-------|-------|
| **Service** | [service-name] |
| **Owner Team** | [Team name] |
| **Escalation Contact** | [Name / Slack channel] |
| **Last Reviewed** | [YYYY-MM-DD] |
| **Review Cadence** | Quarterly |

---

## Service Overview

- **Purpose**: [What the service does]
- **Type**: [Lambda API | ECS Fargate | Batch Job | Frontend]
- **Repository**: [repo-name]
- **AWS Account**: [Account alias / ID]
- **Region**: [us-east-1]

### Architecture

```
[Simple diagram showing service, its dependencies, and data flow]
```

### Dependencies

| Dependency | Type | Impact if Down |
|------------|------|----------------|
| [Aurora PostgreSQL] | Database | Full outage |
| [Okta] | Auth provider | No new logins |
| [S3 bucket] | Storage | File operations fail |

---

## Health Checks

| Check | Endpoint / Method | Expected | Frequency |
|-------|-------------------|----------|-----------|
| API Health | `GET /health` | `200 OK` | 1 min |
| DB Connectivity | [CloudWatch metric] | Connected | 5 min |
| Lambda Errors | [CloudWatch alarm] | < 1% error rate | 5 min |

## Dashboards & Monitoring

| Dashboard | URL | Shows |
|-----------|-----|-------|
| [Service Dashboard] | [CloudWatch URL] | Request count, latency, errors |
| [Database Dashboard] | [CloudWatch URL] | Connections, query latency |

## Alerts

| Alert | Severity | Threshold | Action |
|-------|----------|-----------|--------|
| [High Error Rate] | P1 | > 5% 5xx in 5 min | See Incident: High Error Rate |
| [High Latency] | P2 | p99 > 5s for 10 min | See Incident: High Latency |
| [DB Connection Exhaustion] | P1 | > 90% pool used | See Incident: DB Connections |

---

## Common Incidents

### Incident: High Error Rate

**Symptoms**: Elevated 5xx responses, users seeing error pages.

**Diagnosis**:
1. Check CloudWatch logs: `aws logs filter-log-events --log-group-name /aws/lambda/SERVICE_NAME --filter-pattern "ERROR"`
2. Check recent deployments: `gh pr list --state merged --limit 5`
3. Verify database connectivity
4. Check downstream service health

**Resolution**:
- If caused by deployment: Roll back to previous version
- If caused by database: Check connection pool, restart if necessary
- If caused by downstream: Verify dependency health, check circuit breakers

### Incident: High Latency

**Symptoms**: Slow API responses, timeouts in the frontend.

**Diagnosis**:
1. Check CloudWatch metrics for Lambda duration
2. Look for slow queries in database logs
3. Verify network connectivity (VPC, NAT Gateway)

**Resolution**:
- Slow queries: Identify and optimize (add indexes, review EXPLAIN plans)
- Lambda cold starts: Check provisioned concurrency settings
- Network: Verify VPC configuration, check NAT Gateway throughput

### Incident: DB Connection Exhaustion

**Symptoms**: `FATAL: too many connections` errors.

**Diagnosis**:
1. Check active connections: `SELECT count(*) FROM pg_stat_activity;`
2. Identify long-running queries: `SELECT * FROM pg_stat_activity WHERE state = 'active' ORDER BY query_start;`

**Resolution**:
- Terminate idle connections: `SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE state = 'idle' AND query_start < now() - interval '30 minutes';`
- Adjust connection pool size
- Check for connection leaks in application code

---

## Deployment Procedures

### Standard Deployment

1. Merge PR to `master`
2. Jenkins pipeline auto-triggers
3. Deploys to DEV → QA → STAGING (auto)
4. Production deployment requires manual approval
5. Verify health checks post-deploy

### Rollback

1. Identify last known good version
2. Redeploy via Jenkins: `Build with Parameters` → set version
3. Verify health checks
4. Notify team in Slack

### Emergency Deployment

1. Contact on-call engineer
2. Get verbal approval from team lead
3. Deploy directly via AWS Console or CLI
4. Document in incident channel

---

## Maintenance Tasks

### Scheduled

| Task | Frequency | Procedure |
|------|-----------|-----------|
| [Log rotation] | Daily | Automated via CloudWatch retention |
| [DB vacuum] | Weekly | Automated via RDS maintenance window |
| [Certificate renewal] | Annual | [Procedure link] |

### Ad-Hoc

| Task | When | Procedure |
|------|------|-----------|
| [Clear cache] | On demand | [Steps] |
| [Reprocess failed records] | After incident | [Steps] |

---

## Contacts

| Role | Name | Contact |
|------|------|---------|
| Service Owner | [Name] | [Email / Slack] |
| On-Call Engineer | [Rotation] | [PagerDuty / Slack] |
| Database Admin | [Name] | [Email / Slack] |
| DevOps | [Name] | [Email / Slack] |

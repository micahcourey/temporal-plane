---
name: security-scan
description: Scan code for security vulnerabilities, authentication gaps, injection risks, and missing middleware. OWASP-aligned.
---

# Security Scan Skill

Run security scan script: `scripts/scan.py <file_or_directory>`

## Reference Documentation

| Reference | Content |
|-----------|----------|
| [OWASP Top 10](references/OWASP-TOP-10.yaml) | Top 10 web application security risks with prevention patterns |
| [CWE Top 25](references/CWE-TOP-25.yaml) | 25 most dangerous software weaknesses with code examples |
| [NIST 800-53](references/NIST-800-53.yaml) | Federal security controls for access, audit, crypto, and integrity |
| [OWASP API Security](references/OWASP-API-SECURITY-TOP-10.md) | API-specific security risks |
| [OWASP Agentic Top 10](references/OWASP-AGENTIC-TOP-10.md) | Security risks for agentic AI systems |
| [Node.js Security](references/NODEJS-SECURITY.md) | Node.js/Express security patterns |
| [AWS Security](references/AWS-SECURITY.md) | AWS infrastructure security best practices |
| [OAuth/JWT Security](references/OAUTH-JWT-SECURITY.md) | OAuth/JWT implementation security |

## Context Files

Consult [AGENTS.md](AGENTS.md) for the project knowledge routing table.
**For this skill, prioritize**: Security patterns (L2) for prescriptive rules, then system architecture and third-party integrations context when the change touches storage paths, dependencies, CI, or shell execution.

## When This Skill Activates

This skill activates when:
- Reviewing code for security concerns
- Working with authentication/authorization code
- Editing route/endpoint files
- Discussing security vulnerabilities or OWASP

## Instructions

### Scan Categories

#### 1. Authentication & Authorization

**Check for:**
- Missing `` on API endpoints
- Missing `` on protected endpoints
- Missing `` for auditable actions
- Hardcoded credentials or API keys
- Token validation issues

**Pattern to enforce:**
```

```

#### 2. Input Validation & Injection

**Check for:**
- SQL injection (raw queries with string concatenation)
- NoSQL/Command injection
- Path traversal vulnerabilities
- Missing request body validation
- XSS in rendered output

#### 3. Secrets & Credentials

**Scan for:**
- Hardcoded API keys, passwords, secrets in source code
- Cloud credentials in code
- Connection strings with embedded credentials
- Sensitive environment variables referenced in frontend code
- Private keys or certificates in source

#### 4. Error Handling & Information Disclosure

**Verify:**
- No stack traces returned to users
- No sensitive data in error messages (N/A)
- Structured error format used:
```
# Rust library APIs return typed errors.
# The CLI renders human-readable summaries and may add structured output later.
```





### Severity Levels

| Level | Icon | Description | Action |
|-------|------|-------------|--------|
| **Critical** | 🔴 | Data breach risk, injection vulnerability | Must fix |
| **High** | 🟠 | Auth bypass, missing validation | Must fix before merge |
| **Medium** | 🟡 | Best practice violation | Should fix |
| **Low** | 🔵 | Enhancement opportunity | Nice to fix |

### Report Format

```markdown
# Security Scan Report

**Status**: ✅ PASS / ❌ BLOCK
**Files Scanned**: {count}
**Issues Found**: {Critical: X, High: Y, Medium: Z}

## Findings

| Severity | Category | Issue | File:Line | Fix |
|----------|----------|-------|-----------|-----|
| 🔴 | Auth | Missing auth middleware | routes.ts:45 | Add middleware chain |

## Recommendation
✅ APPROVED - No blocking issues
❌ BLOCKED - Fix critical/high issues before merge
```

## OWASP Top 10 Quick Check

| # | Category | Check |
|---|----------|-------|
| A01 | Broken Access Control | Auth middleware on all endpoints |
| A02 | Cryptographic Failures | No plaintext secrets, proper encryption |
| A03 | Injection | Parameterized queries, input validation |
| A04 | Insecure Design | Threat modeling, secure patterns |
| A05 | Security Misconfiguration | Secure defaults, no debug in prod |
| A06 | Vulnerable Components | Updated dependencies |
| A07 | Auth Failures | Strong auth, MFA, session management |
| A08 | Data Integrity | Input validation, signed data |
| A09 | Logging Failures | Security events logged, no sensitive data |
| A10 | Server-Side Request Forgery | URL validation, allowlists |

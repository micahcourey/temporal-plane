# Production Readiness Review (PRR)

Mnemix `v0.2.1` is the release-recovery follow-up to the initial `v0.2.0` rebrand publish attempt. This patch release keeps the public Mnemix surfaces introduced in `v0.2.0`, adds the missing `protoc` setup required by the tag-triggered release workflow, and carries forward the reusable release documentation needed for ongoing publishes.

---

## Release Schedule

| Field | Value |
|-------|-------|
| **Release Date** | 2026-03-10 |
| **Release Window** | TBD |
| **Version** | `v0.2.1` |
| **Release Type** | Patch |
| **Release Epic** | Release recovery |

## Release Scope

This release retries the first successful PyPI publish under the `mnemix` name after the failed `v0.2.0` attempt. It preserves the Mnemix rebrand and bundled-wheel packaging, while fixing the tag-driven release workflow and carrying the release documentation improvements merged after `v0.2.0`.

| Ticket | Summary | Status |
|--------|---------|--------|
| `#26` | Rebrand Temporal Plane to Mnemix across public surfaces | Done |
| `#28` | Prepare the Mnemix release process and version alignment | Done |
| `#30` | Restore `protoc` setup in the release workflow and ship release docs | Done |

## Stakeholders Approval & Notifications

### Internal Team

| Stakeholder | Role | Approval | Date |
|-------------|------|----------|------|
| Micah Courey | Maintainer / releaser | Pending final release approval | 2026-03-10 |

## User Acceptance Test

UAT for this release remains focused on packaging, installability, and release readiness rather than new product semantics. Validation covers Python package tests, source distribution build, metadata rendering, bundled-wheel verification, and local CLI packaging compatibility against the post-merge release workflow and release docs.

| Test Scenario | Tested By | Result |
|---------------|-----------|--------|
| Run Python package test suite | Maintainer | Pass |
| Build sdist and validate metadata with `twine check --strict` | Maintainer | Pass |
| Build local `mnemix-cli` binary for bundled-wheel validation | Maintainer | Pass |
| Install and verify bundled wheel in a clean virtual environment | Maintainer | Pass |

## Release Known Issues

This release has no known code-level blockers, but PyPI trusted-publisher configuration still needs to be confirmed before publication.

| Issue | Severity | Impact | Planned Fix |
|-------|----------|--------|-------------|
| PyPI trusted-publisher configuration must match the `mnemix` project before publish | Medium | Publish can fail even if build artifacts are valid | Verify the PyPI publisher and rerun the release on `v0.2.1` |

## Release Test Results

Release verification combines automated package validation with local release-preflight checks before the `v0.2.1` GitHub Release is published.

### Security, Performance, & Accessibility

| Test Type | Status | Notes |
|-----------|--------|-------|
| Security | Pass | PyPI publishing uses GitHub OIDC trusted publishing instead of a long-lived token |
| Performance | N/A | This release focuses on release recovery and packaging alignment |
| Section 508 / Accessibility | N/A | No UI surface is introduced by this release |
| UX | Pass | Public installation and package naming now consistently use `mnemix` |

### Regression Testing

| Test Type | Total | Passed | Failed | Skipped | Status |
|-----------|-------|--------|--------|---------|--------|
| Automated | 64 | 64 | 0 | 0 | Pass |
| Manual | 4 | 4 | 0 | 0 | Pass |

## Deployment Checklist

- [x] Environment configuration verified
- [x] Database migrations applied
- [x] Feature flags configured
- [x] Smoke tests passing
- [x] Monitoring and alerts active
- [x] Rollback plan documented
- [ ] GitHub Release published from tag `v0.2.1`

## Production Post-Deployment Verification

- [x] Application health check passing
- [x] Key user flows validated
- [x] Performance metrics within SLA
- [x] No new errors in logs
- [ ] Stakeholders notified of successful deployment
- [ ] `pip install mnemix` confirmed against the live PyPI package page

## Release Statistics

GitHub release: pending `v0.2.1`

| Metric | Value |
|--------|-------|
| Total tickets in release | 3 |
| Stories | 0 |
| Bugs | 0 |
| Tasks | 3 |

## Notes & Miscellaneous Items

When `v0.2.1` is published from `main`, update the release body in place with:

```bash
gh release edit v0.2.1 --notes-file RELEASE_NOTES.md
```

Updating the release notes does not republish artifacts or rerun the PyPI workflow.
# Production Readiness Review (PRR)

Initial public PyPI release plan for the `mnemix` Python wrapper. This release ships the thin Python client, preserves Rust as the source of truth, and publishes through GitHub Actions trusted publishing after local packaging validation and release-tag verification.

---

## Release Schedule

| Field | Value |
|-------|-------|
| **Release Date** | After final validation on `main` |
| **Release Window** | Maintainer-triggered GitHub Release publish |
| **Version** | `0.1.0` |
| **Release Type** | Initial public alpha |
| **Release Epic** | `tejbp11o` |

## Release Scope

This release publishes the first installable Python package for Mnemix. Scope includes the existing Python wrapper, packaging metadata hardening, release validation, and trusted publishing through the repository release workflow.

| Ticket | Summary | Status |
|--------|---------|--------|
| `8rcxcljs` | Milestone 6 Python binding and first adapter | Done |
| `izomfhld` | Milestone 7 advanced storage workflows backing the shipped CLI surface | Done |
| `tejbp11o` | PyPI release readiness, metadata hardening, and release checklist | In Progress |

## Stakeholders Approval & Notifications

### Internal Team

| Stakeholder | Role | Approval | Date |
|-------------|------|----------|------|
| Micah Courey | Maintainer / releaser | Pending | 2026-03-09 |

## User Acceptance Test

UAT for this release focuses on packaging and installability rather than new product semantics. The acceptance path is: build sdist and wheel, validate metadata rendering, install the built artifact in a clean virtual environment, confirm import success, and confirm the wrapper reports a missing CLI clearly when the binary is absent.

| Test Scenario | Tested By | Result |
|---------------|-----------|--------|
| Build sdist and wheel locally | Maintainer | Pass |
| Install built wheel in a clean virtual environment | Maintainer | Pass |
| Import `mnemix` and read `__version__` | Maintainer | Pass |
| Confirm missing binary raises `MnemixBinaryNotFoundError` | Maintainer | Pass |

## Release Known Issues

| Issue | Severity | Impact | Planned Fix |
|-------|----------|--------|-------------|
| The PyPI package does not bundle the Rust `mnemix` CLI binary | Medium | Users must install the CLI separately or set `MNEMIX_BINARY` | Document clearly in the package README and revisit if a bundled distribution is introduced |
| GitHub Release publishing depends on PyPI trusted publisher setup in the `pypi` environment | Low | Release automation will fail until trust is configured | Complete repository and PyPI setup before the first live publish |

## Release Test Results

Release verification for this package is primarily automated through Python tests, packaging checks, and the publish workflow.

### Security, Performance, & Accessibility

| Test Type | Status | Notes |
|-----------|--------|-------|
| Security | Pass | Trusted publishing uses GitHub OIDC instead of a long-lived PyPI token in CI |
| Performance | N/A | This release changes packaging and distribution rather than runtime performance |
| Section 508 / Accessibility | N/A | No UI surface is introduced by this package release |
| UX | Pass | Package README now includes PyPI installation guidance and CLI dependency expectations |

### Regression Testing

| Test Type | Total | Passed | Failed | Skipped | Status |
|-----------|-------|--------|--------|---------|--------|
| Automated | 58 (56 Python tests plus `build` and `twine check`) | 58 | 0 | 0 | Pass |
| Manual | 4 packaging and install scenarios | 4 | 0 | 0 | Pass |

## Deployment Checklist

- [ ] PyPI trusted publisher added for `.github/workflows/publish-python.yml`
- [ ] GitHub `pypi` environment created with desired protections
- [ ] Python version updated in `python/mnemix/_version.py`
- [x] Local package validation completed with `./scripts/check-python-package.sh`
- [ ] GitHub Release created from a clean, verified tag
- [ ] Publish workflow completes successfully
- [ ] PyPI project page renders README and metadata correctly

## Production Post-Deployment Verification

- [ ] `pip install mnemix` succeeds from PyPI
- [ ] `import mnemix` succeeds in a clean virtual environment
- [ ] `mnemix.__version__` matches the published version
- [ ] README on PyPI clearly documents the external CLI requirement
- [ ] A basic client invocation fails cleanly with actionable guidance when the CLI is absent
- [ ] Maintainer records release outcome and any follow-up fixes

## Release Statistics

| Metric | Value |
|--------|-------|
| Total tickets in release | 3 |
| Stories | 0 |
| Bugs | 0 |
| Tasks | 3 |

## Notes & Miscellaneous Items

The live publish path is already implemented in [.github/workflows/publish-python.yml](../.github/workflows/publish-python.yml). The release trigger is a published GitHub Release, not a plain tag push. Local release preflight is standardized in [scripts/check-python-package.sh](../scripts/check-python-package.sh). Version metadata is now single-sourced from `python/mnemix/_version.py` so the package metadata and runtime version stay aligned. Local verification completed on 2026-03-09: 56 Python tests passed, sdist and wheel builds succeeded, `twine check` passed, and a clean virtual environment successfully imported the built wheel and raised `MnemixBinaryNotFoundError` for a missing CLI path.

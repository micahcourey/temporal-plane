# Runbook

| Field | Value |
|-------|-------|
| **Service** | Mnemix release process |
| **Owner Team** | Maintainer |
| **Escalation Contact** | GitHub issue tracker and repository maintainer |
| **Last Reviewed** | 2026-03-10 |
| **Review Cadence** | Before each release |

---

## Service Overview

- **Purpose**: Publish a new Mnemix release to GitHub and PyPI with version-aligned Rust and Python artifacts.
- **Type**: Release runbook
- **Repository**: `micahcourey/mnemix`
- **AWS Account**: None
- **Region**: N/A

### Architecture

```text
main branch
  -> version bump in Cargo.toml + python/mnemix/_version.py
  -> local preflight via ./scripts/check-python-package.sh
  -> published GitHub Release
  -> .github/workflows/publish-python.yml
  -> PyPI artifacts for mnemix
```

### Dependencies

| Dependency | Type | Impact if Down |
|------------|------|----------------|
| GitHub Releases | Release trigger | No publish workflow trigger |
| GitHub Actions | CI/CD | Artifacts cannot be built or published |
| PyPI trusted publishing | Package registry auth | Publish step fails |
| Local Python + Rust toolchains | Build tooling | Preflight cannot complete |

---

## Health Checks

| Check | Endpoint / Method | Expected | Frequency |
|-------|-------------------|----------|-----------|
| Version alignment | Compare `Cargo.toml` and `python/mnemix/_version.py` | Same version string | Every release |
| Local preflight | `./scripts/check-python-package.sh` | Exit code `0` | Every release |
| Release workflow | GitHub Actions `Publish Python` | All jobs succeed | Every release |
| Package availability | `python3 -m pip index versions mnemix` | New version listed | Every release |

## Dashboards & Monitoring

| Dashboard | URL | Shows |
|-----------|-----|-------|
| GitHub Actions | `https://github.com/micahcourey/mnemix/actions` | Build, test, and publish workflow status |
| GitHub Releases | `https://github.com/micahcourey/mnemix/releases` | Published tags and release notes |
| PyPI Project Page | `https://pypi.org/project/mnemix/` | Live package metadata and install instructions |

## Alerts

| Alert | Severity | Threshold | Action |
|-------|----------|-----------|--------|
| Preflight failure | P1 | Any non-zero exit from `./scripts/check-python-package.sh` | Stop release and fix before tagging |
| Publish workflow failure | P1 | Any failed job in `Publish Python` after a release is published | Inspect logs and rerun only after root cause is fixed |
| Version mismatch | P1 | GitHub tag, Cargo version, and Python version do not match | Correct versions and cut a new release |
| Missing PyPI package update | P1 | New version absent from PyPI after successful workflow | Verify trusted publishing and package artifacts |

---

## Common Incidents

### Incident: Local Preflight Fails

**Symptoms**: `./scripts/check-python-package.sh` exits non-zero, tests fail, or `twine check` reports metadata errors.

**Diagnosis**:
1. Read the failing step output from the local command.
2. Confirm the version bump touched both `Cargo.toml` and `python/mnemix/_version.py`.
3. Check whether README or packaging metadata references moved files.
4. Re-run the failing command directly if a narrower loop is needed.

**Resolution**:
- Fix the underlying test, metadata, or packaging issue.
- Re-run `./scripts/check-python-package.sh` until it passes cleanly.
- Do not create or publish a release until preflight is green.

### Incident: Publish Workflow Fails

**Symptoms**: The GitHub Release exists, but `Publish Python` fails in one of the build or publish jobs.

**Diagnosis**:
1. Open the run in GitHub Actions.
2. Identify whether the failure is in test, sdist, bundled wheel, or publish.
3. Check whether PyPI trusted publishing for the `pypi` environment is configured.
4. Confirm the tag points at the intended release commit.

**Resolution**:
- Fix the underlying branch or repository configuration issue.
- If the release tag is wrong, cut a new version rather than mutating the published release history.
- Re-run workflow steps only after the root cause is addressed.

### Incident: PyPI Shows the Wrong Package or Version

**Symptoms**: `pip index versions mnemix` does not show the expected version, or the release published an older package identity.

**Diagnosis**:
1. Inspect the GitHub release tag commit.
2. Verify `python/pyproject.toml` package name and dynamic version source.
3. Verify `python/mnemix/_version.py` and `Cargo.toml` contain the intended version.
4. Confirm the workflow artifacts were built from the intended tag.

**Resolution**:
- Treat the incorrect publish as immutable history.
- Bump to the next version and publish a corrected release from the right commit.

---

## Deployment Procedures

### Standard Deployment

1. Start from a clean `main` branch.
2. Bump the version in `python/mnemix/_version.py` and `Cargo.toml`.
3. Update any release-facing docs that depend on the current release procedure or version.
4. Run `./scripts/check-python-package.sh`.
5. Merge the release-prep PR to `main`.
6. Create and publish a GitHub Release tagged `vX.Y.Z` from the verified `main` commit.
7. Wait for `.github/workflows/publish-python.yml` to complete successfully.
8. Verify the new version on PyPI and in a clean install.

### Rollback

1. Do not overwrite or delete published PyPI artifacts.
2. If a release is bad, prepare a follow-up patch release with a new version.
3. Fix the underlying issue on `main`.
4. Repeat the standard deployment process for the new version.

### Emergency Deployment

1. Limit emergency releases to packaging or publish-blocking fixes.
2. Keep the diff minimal and directly tied to the failed release.
3. Re-run `./scripts/check-python-package.sh` before publishing the emergency fix.
4. Publish a new GitHub Release with the next version.

---

## Maintenance Tasks

### Scheduled

| Task | Frequency | Procedure |
|------|-----------|-----------|
| Review release checklist | Every release | Confirm steps, workflow names, and URLs still match the repo |
| Review trusted publishing config | Quarterly | Verify the `pypi` environment and PyPI trusted publisher remain configured |
| Review release docs links | Quarterly | Confirm README and changelog references still point at live documents |

### Ad-Hoc

| Task | When | Procedure |
|------|------|-----------|
| Cut a hotfix release | A published package is wrong or broken | Follow the emergency deployment path with a new version |
| Refresh release notes | Before a public announcement | Update the GitHub Release body with current user-facing changes |
| Verify package install flow | After PyPI publish | Test `pip install mnemix` in a clean virtual environment |

---

## Contacts

| Role | Name | Contact |
|------|------|---------|
| Maintainer | Micah Courey | GitHub issues and repository notifications |
| Release Approver | Micah Courey | GitHub releases |
| Package Owner | Micah Courey | PyPI project ownership |
| CI/CD Owner | Micah Courey | GitHub Actions workflow access |

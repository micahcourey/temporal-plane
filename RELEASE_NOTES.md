# Production Readiness Review (PRR)

Mnemix `v0.3.0` is the next minor release after `v0.2.9`. It packages the
first human memory-browser TUI, the initial policy-runner surface, the
`mnemix-workflow` rename across workflow artifacts, and a set of installation,
documentation, website, and release-pipeline refinements. Publication still
flows through the GitHub Release to PyPI trusted-publishing workflow after this
release-prep PR merges.

---

## Release Schedule

| Field | Value |
|-------|-------|
| **Release Date** | 2026-03-31 |
| **Release Window** | Pending merge and tag publish |
| **Version** | `v0.3.0` |
| **Release Type** | Minor |
| **Release Epic** | `N/A` |

## Release Scope

This release rolls up everything merged after `v0.2.9`, with the main user- and
operator-facing changes centered on memory browsing, policy enforcement, and
workflow/documentation cleanup.

| PR / Change | Summary | Status |
|-------------|---------|--------|
| `#93` | Add `mnemix ui` human memory-browser TUI and shared browse contract | Done |
| `#92` | Replace `dex` with `mnemix-workflow` across workflow surfaces | Done |
| `#79` | Add the initial policy runner surface | Done |
| `#73` | Clear the `lz4` advisory from the lockfile | Done |
| `#64`, `#66` | Expand pipx, host adapter, and coding adapter policy docs | Done |
| `#67`, `#68`, `#69`, `#70`, `#71`, `#72` | Refresh README and website content, navigation, and layout polish | Done |
| Release prep | Bump workspace and Python package versions to `0.3.0` | Done |

## Stakeholders Approval & Notifications

### Internal Team

| Stakeholder | Role | Approval | Date |
|-------------|------|----------|------|
| Micah Courey | Maintainer / releaser | Pending final release approval | 2026-03-31 |

## User Acceptance Test

UAT for `v0.3.0` focuses on release readiness plus the two new operator-facing
surfaces introduced since `v0.2.9`: the browse-first TUI and the initial policy
runner flows.

| Test Scenario | Tested By | Result |
|---------------|-----------|--------|
| Run repo verification via `./scripts/check.sh` | Maintainer | Pass |
| Run Python package release preflight via `./scripts/check-python-package.sh` | Maintainer | Pass |
| Open `mnemix ui` against a temporary initialized store and confirm recent, pinned, and detail panes render | Maintainer | Pass |
| Verify policy and CLI regression coverage through the checked test suite | Maintainer | Pass |

## Release Known Issues

This release has no known blocking regressions, but a few non-blocking
validation warnings remain expected in the current repo shape.

| Issue | Severity | Impact | Planned Fix |
|-------|----------|--------|-------------|
| `./scripts/check.sh` still warns that `mnemix` and `mx` share the same `src/main.rs` binary entrypoint | Low | Cosmetic warning during checks; validation still passes | Keep or revisit if the binary layout changes later |
| `cargo-deny` still reports dependency duplication warnings such as `crossterm` in the lockfile | Low | Noise during release verification; no check failure | Reduce duplicate dependency versions in a follow-up dependency cleanup |
| `mnemix ui` is intentionally read-only in `v0.3.0` | Low | Human operators can browse and inspect but not mutate from the TUI | Consider pin/unpin or restore actions in a later release |

## Release Test Results

Release verification combines Rust workspace checks, Python package preflight,
and one manual TUI smoke check before the `v0.3.0` GitHub Release is published.

### Security, Performance, & Accessibility

| Test Type | Status | Notes |
|-----------|--------|-------|
| Security | Pass | PyPI publishing remains on GitHub OIDC trusted publishing, and the `lz4` advisory has been removed from the lockfile |
| Performance | N/A | This release does not introduce a new performance-sensitive service path |
| Section 508 / Accessibility | Partial | The TUI remains keyboard-first and avoids color-only signaling, but terminal accessibility depends on the user’s terminal environment |
| UX | Pass | The release adds a browse-first human UI for memory inspection and clearer install/workflow documentation |

### Regression Testing

| Test Type | Coverage | Status |
|-----------|----------|--------|
| Automated | `./scripts/check.sh` and `./scripts/check-python-package.sh` | Pass |
| Manual | PTY smoke test of `mnemix ui` | Pass |

## Deployment Checklist

- [x] Release prep branch created for `v0.3.0`
- [x] Workspace version updated to `0.3.0`
- [x] Python package version updated to `0.3.0`
- [x] `./scripts/check-python-package.sh` passing
- [x] `./scripts/check.sh` passing
- [x] Release notes and changelog updated for `v0.3.0`
- [ ] Release-prep PR merged to `main`
- [ ] GitHub Release published from tag `v0.3.0`

## Production Post-Deployment Verification

- [ ] Git tag `v0.3.0` published from `main`
- [ ] GitHub Release body updated from `RELEASE_NOTES.md`
- [ ] PyPI publish workflow completed successfully
- [ ] `pip install mnemix==0.3.0` confirmed against the live PyPI package
- [ ] Stakeholders notified of successful publication

## Release Statistics

Planned GitHub release URL after publication:
`https://github.com/micahcourey/mnemix/releases/tag/v0.3.0`

| Metric | Value |
|--------|-------|
| Release prep commits on branch | 1 |
| Merged mainline PRs included since `v0.2.9` | 12 |
| New major user-facing capabilities | 2 |
| Release-preflight scripts run for prep | 2 |

## Notes & Miscellaneous Items

After this PR merges, publish from a clean `main` checkout with:

```bash
./scripts/publish-release.sh 0.3.0
```

If the GitHub release notes need an update after publication, edit the release
body in place with:

```bash
gh release edit v0.3.0 --notes-file RELEASE_NOTES.md
```

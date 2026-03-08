# Security Patterns

> Current repo security posture is local-tooling oriented, not service-auth oriented.

## What Exists Today

- no runtime auth middleware
- no HTTP API
- no RBAC or permission system in repo code
- local filesystem access is the main trust boundary

## What To Protect

- local storage paths and future dataset locations
- CLI and script shell-outs
- dependency integrity
- release and CI workflows
- accidental secret or token commits

## Practical Review Checklist

- validate filesystem paths before destructive operations
- avoid unsafe shell string interpolation
- keep secrets out of code, fixtures, and logs
- keep `unsafe` forbidden unless explicitly justified at workspace policy level
- keep dependency and license checks green

## Sources

- `docs/temporal-plane-plan-v3.md`
- `README.md`
- `.github/workflows/ci.yml`

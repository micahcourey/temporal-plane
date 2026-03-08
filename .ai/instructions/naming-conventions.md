# Naming Conventions

## Repository and Crate Names

- repository root: `temporal-plane`
- workspace crates: `temporal-plane-*`
- adapter directories should use the host name, for example `adapters/ai-dx-toolkit`

## Rust Code

- crates: kebab-case
- modules/files: snake_case
- types/traits/enums: `PascalCase`
- functions/methods: `snake_case`
- constants: `SCREAMING_SNAKE_CASE`

## Domain Names

Prefer explicit names that encode product meaning, for example `MemoryId`, `ScopeId`, `CheckpointName`, and `RetentionPolicy`.

## Sources

- `docs/temporal-plane-coding-guidelines.md`
- `docs/temporal-plane-roadmap.md`

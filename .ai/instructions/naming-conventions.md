# Naming Conventions

## Repository and Crate Names

- repository root: `mnemix`
- workspace crates: `mnemix-*`
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

- `docs/mnemix-coding-guidelines.md`
- `docs/mnemix-roadmap.md`

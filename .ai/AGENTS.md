# temporal-plane — AI Coding Instructions

> Canonical project instructions for generated AI toolkit resources.
> Read only the modules needed for the current task.

## Project Overview

Temporal Plane is a standalone, local-first memory layer for AI coding agents and related tooling.

Current repo status:

- planning and scaffold phase
- Rust is the source of truth
- no production HTTP API or auth stack exists
- Dex is required for multi-step work

## Always-Know Rules

1. Keep `temporal-plane-core` storage-agnostic.
2. Keep LanceDB and Lance details inside `temporal-plane-lancedb`.
3. Treat Python as a wrapper layer, not a second implementation.
4. Use Dex for multi-step work and record verification in task results.
5. Do not invent frontend, web API, or compliance workflows that do not exist here.

## Canonical Project Docs

- [../docs/temporal-plane-plan-v3.md](../docs/temporal-plane-plan-v3.md)
- [../docs/temporal-plane-roadmap.md](../docs/temporal-plane-roadmap.md)
- [../docs/lancedb-rust-sdk-agent-guide.md](../docs/lancedb-rust-sdk-agent-guide.md)
- [../docs/temporal-plane-coding-guidelines.md](../docs/temporal-plane-coding-guidelines.md)
- [../docs/git-workflow.md](../docs/git-workflow.md)

## Instructions — Load When Needed

| Working on... | Read this |
|--------------|----------|
| Rust APIs, crate boundaries, typed errors, docs | [instructions/coding-standards.md](instructions/coding-standards.md) |
| Verification, tests, snapshots, repo checks | [instructions/testing-conventions.md](instructions/testing-conventions.md) |
| Filesystem safety, dependencies, local trust boundaries | [instructions/security-patterns.md](instructions/security-patterns.md) |
| Branching, Dex workflow, commits, PRs | [instructions/git-workflow.md](instructions/git-workflow.md) |
| Crate names, file names, domain naming | [instructions/naming-conventions.md](instructions/naming-conventions.md) |

## Project Knowledge — Load When Needed

| I need to... | Read this |
|--------------|-----------|
| Understand crates, adapters, and planned data flow | [context/System_Architecture.md](context/System_Architecture.md) |
| Check implemented vs planned storage schema | [context/Database_Schema.md](context/Database_Schema.md) + [context/schema.yaml](context/schema.yaml) |
| Understand repo layout and notable roots | [context/Repository_Index.md](context/Repository_Index.md) + [context/repositories.jsonl](context/repositories.jsonl) |
| Look up Temporal Plane terms | [context/Domain_Glossary.md](context/Domain_Glossary.md) + [context/glossary.jsonl](context/glossary.jsonl) |
| Review repo verification strategy | [context/Testing_Strategy.md](context/Testing_Strategy.md) |
| Review active and planned integrations | [context/Third_Party_Integrations.md](context/Third_Party_Integrations.md) |

> Do not load every context file up front.

## Skills

| Working on... | Skill |
|--------------|-------|
| Writing or extending tests | [skills/unit-test/SKILL.md](skills/unit-test/SKILL.md) |
| Reviewing dependency, filesystem, or shell-safety changes | [skills/security-scan/SKILL.md](skills/security-scan/SKILL.md) |
| Git workflow automation | [skills/git-workflow/SKILL.md](skills/git-workflow/SKILL.md) |
| Planning from roadmap items or Dex tasks | [skills/planning/SKILL.md](skills/planning/SKILL.md) |
| Writing docs, RFCs, ADRs, READMEs, PR notes | [skills/document-generator/SKILL.md](skills/document-generator/SKILL.md) |
| Creating new skills | [skills/skill-generator/SKILL.md](skills/skill-generator/SKILL.md) |

## Scope Notes

- The optional toolkit Temporal Plane integration is intentionally disabled.
- The compliance agent and compliance instruction module are intentionally omitted.
- N/A context files for auth, roles, and API routes were removed to keep this repo lean.

**Last Updated**: 2026-03-08
**Workspace**: temporal-plane

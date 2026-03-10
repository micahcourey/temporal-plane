# mnemix — Domain Glossary

> The structured glossary is in [glossary.jsonl](glossary.jsonl). Several extracted terms still need maintainer review.

## Overview

Mnemix vocabulary combines product terms from the planning docs with storage terminology from LanceDB and Lance.

## Core Terms

- **Mnemix** — the product itself
- **memory record** — durable stored memory unit
- **pin** — memory favored during retrieval
- **archival memory** — deeper memory fetched on demand
- **checkpoint** — named stable reference to a version
- **checkout** — inspect a past state
- **restore** — create a new current state from a past state

## Review Guidance

Terms marked `needs_review: true` in [glossary.jsonl](glossary.jsonl) were extracted from plans and should be confirmed as implementation terminology.

## Sources

- `docs/mnemix-plan-v3.md`
- `docs/mnemix-roadmap.md`
- `docs/lancedb-rust-sdk-agent-guide.md`

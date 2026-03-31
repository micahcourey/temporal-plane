# Decisions

This folder holds durable, repo-level decisions for this repository.

Use this location when a decision:

- constrains future workstreams
- defines framework behavior or repository conventions
- should outlive the original workstream where it was discovered

Workstream-local decisions should stay beside the workstream in `workflow/workstreams/<id>/decisions/` until they need to be promoted here.

## Current ADRs

- `001` [`replace Dex with repo-native workflow tracking`](./001-replace-dex-with-repo-native-workflow-tracking.md): use `mnemix-workflow` and `workflow/` artifacts as the source of truth for planned and historical work

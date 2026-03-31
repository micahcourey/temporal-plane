# Feature Spec: Policy Runner Guided And Required Memory Workflows

## Summary

Create a configurable policy runner that lets Mnemix hosts move between guided
memory usage and required workflow checkpoints without pushing workflow policy
into storage or the generic base client.

## Problem

Mnemix already supports helpful memory guidance through adapters, but teams also
need a higher-level workflow layer that can answer questions like:

- should recall happen before this task starts?
- should a checkpoint exist before a risky change?
- can a commit proceed without writeback, or is a skip reason required?

The original planning issue `#74` captured that design need, but it was a plan,
not the long-lived implementation container. This workstream becomes the
canonical repo-native umbrella for the actual policy-runner effort.

## Users

- Primary persona: coding-agent hosts and maintainers integrating Mnemix into
  task, commit, review, and release workflows
- Secondary persona: repo operators who need inspectable policy defaults and
  explicit skip behavior

## Goals

- Support both guided and required policy modes with inspectable decisions
- Keep policy logic above storage semantics and above the generic base client
- Make workflow evidence, skip handling, and checkpoint policy explicit
- Preserve coding-agent-first ergonomics while keeping the design host-agnostic

## Non-Goals

- Turning all memory usage into mandatory writeback
- Making MCP the enforcement layer
- Moving workflow policy into LanceDB, schema design, or low-level memory
  storage
- Solving every host integration in the first policy slice

## User Value

This workstream gives teams a predictable and explainable way to enforce
high-signal memory workflows only where they help, while still allowing
low-signal skips and lightweight guidance elsewhere.

## Functional Requirements

- Policy evaluation must support task-start, commit, review, release, and risky
  change style triggers
- Policy actions must include recall, writeback, checkpoint, skip reason, scope
  selection, and classification selection
- Decisions must remain explainable and inspectable from CLI and Python
- Workflow evidence must be recordable and later expanded with lifecycle
  handling
- Host adapters must be able to compose with the runner without duplicating
  policy semantics

## Constraints

- `mnemix-core` stays storage-agnostic
- Policy remains a host/workflow layer, not a store feature
- Python remains a wrapper around Rust behavior
- MCP, hooks, and adapter composition should build on the core policy runner
  rather than redefining it

## Success Criteria

- The repo has one durable workstream that captures the policy-runner design and
  backlog instead of a plan-only issue
- Foundational policy slices are represented as completed work inside this
  workstream
- Remaining follow-on issues are organized as explicit execution slices
- Future contributors can understand what is already shipped and what still
  remains

## Risks

- Planning and implementation can drift if the workstream is not kept current as
  follow-on issues land
- Host-specific integrations may try to bypass the core policy model and create
  duplicate behavior
- MCP could become over-emphasized before the host-side enforcement model is
  fully settled

## References

- GitHub issue `#74`
- GitHub issue `#81`
- GitHub issue `#82`
- GitHub issue `#83`
- GitHub issue `#84`
- PR `#66`
- PR `#79`

# UX Spec: Policy Runner Guided And Required Memory Workflows

## Summary

The policy runner experience should feel like a clear workflow coach, not a
black-box gate. Hosts should see what trigger fired, what evidence is missing,
what action is being recommended or required, and what explicit skip path is
available.

## Users And Context

- Primary persona: maintainers and agent authors integrating Mnemix into coding
  workflows
- Context of use: CLI wrappers, future hooks, coding-agent orchestration, and
  Python automation
- Preconditions: a repo or store may define policy rules, and the host emits a
  workflow trigger

## User Goals

- Understand whether the workflow may proceed
- Understand why a policy decision was returned
- Record evidence or a skip reason without guessing hidden state
- Reuse the same policy behavior from CLI, Python, and higher-level adapters

## Experience Principles

- Decisions must be explainable before they are enforceable
- Guided mode should reduce friction, not imitate strict mode badly
- Required mode should make the missing action obvious
- Skip behavior should be explicit and respectful of low-signal work

## Primary Journey

1. A host emits a policy trigger such as task start, commit, or risky change.
2. The runner evaluates matching rules and current workflow evidence.
3. The host receives a clear decision: allow, recommend, require action, or
   block.
4. The user either records the missing action, records a skip reason if policy
   permits it, or continues when the workflow is already satisfied.

## Alternate Flows

### Flow: Guided Policy

- Trigger: the host emits a trigger with only guided rules configured
- Path: policy returns recommendations and explanations without blocking
- Expected outcome: the user can continue while still seeing the recommended
  memory behavior

### Flow: Required With Skip

- Trigger: a required rule is unsatisfied but allows a skip reason
- Path: the host prompts for the missing action or an explicit skip reason
- Expected outcome: the workflow remains inspectable without forcing a
  low-signal writeback

### Flow: Strict Block

- Trigger: a required rule is unsatisfied and no skip path is allowed
- Path: the host receives a block decision and does not proceed
- Expected outcome: high-risk workflows stop until evidence is recorded

## Surfaces

### Surface: CLI Policy Commands

- Purpose: expose the canonical check, explain, and record workflow loop
- Key information: trigger, workflow key, decision, missing actions, reasons,
  and any recorded evidence or skip reason
- Available actions: check policy, explain policy, record action, record skip
  reason
- Navigation expectations: commands should be scriptable and easy to compose

### Surface: Coding-Agent Integration

- Purpose: let coding agents use policy checks at task start, writeback, and
  risky-change checkpoints
- Key information: chosen workflow key strategy, recommended scopes, missing
  evidence, and checkpoint/writeback expectations
- Available actions: start policy-aware work, store outcome, checkpoint before
  risky change, explain policy state
- Navigation expectations: one higher-level integration path should exist
  instead of many ad hoc calls

## States

### Success

- The decision explains why the workflow can proceed

### Recommendation

- The missing action is visible but not blocking

### Require Action

- The decision shows exactly what evidence is missing and how to satisfy it

### Block

- The host receives an explicit, inspectable reason for the block

## Interaction Details

- Policy output should use stable trigger and action names
- Skip reason workflows must clearly require human intent, not silent fallback
- Workflow keys should be explainable enough that repo, workspace, session, and
  task scoping feel deliberate rather than magical

## Content And Tone

- Use direct language such as "missing required action: checkpoint"
- Differentiate recommendations from hard requirements visibly
- Avoid shaming language around explicit skip behavior

## Accessibility Requirements

- CLI output must remain readable in plain text without color dependence
- Explanations should be complete in text so wrappers and screen readers can
  surface them without extra formatting logic

## Acceptance Scenarios

```gherkin
Scenario: Required policy blocks a risky change without a checkpoint
  Given a store policy requires a checkpoint for on_risky_change
  And no checkpoint evidence exists for the workflow
  When the host checks policy for that trigger
  Then the decision should require action or block
  And the missing checkpoint should be named explicitly
```

```gherkin
Scenario: Guided policy recommends writeback without blocking
  Given a guided writeback rule matches a workflow
  When the host checks policy
  Then the decision should allow with recommendation
  And the recommendation should explain why writeback is useful
```

## References

- GitHub issue `#74`
- GitHub issue `#83`
- GitHub issue `#84`

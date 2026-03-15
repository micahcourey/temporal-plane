# Review Tool Example

This example shows how to use [`ReviewToolAdapter`](/Users/micah/Projects/mnemix/adapters/review_tool_adapter.py)
for code review workflows that need reusable review rules and recurring issue
tracking.

## Typical flow

1. initialize the store
2. prepare review context for the current topic
3. perform the review
4. store reusable rules or recurring findings

## Example

```python
from pathlib import Path
from adapters import ReviewToolAdapter

adapter = ReviewToolAdapter(store=Path(".mnemix"))
adapter.ensure_store()

context = adapter.prepare_review(
    scope="repo:mnemix",
    review_topic="adapter verification expectations",
)
print(context.prompt_preamble)

adapter.record_review_rule(
    memory_id="memory:review-rule-verification",
    scope="repo:mnemix",
    title="Call out missing verification",
    summary="Reviews should explicitly note when tests or checks were not run.",
    detail="If a change was not fully verified, the review result should state the gap clearly so the next step is obvious.",
    pin_reason="Project-wide review rule",
)

adapter.record_recurring_issue(
    memory_id="memory:review-issue-metadata",
    scope="repo:mnemix",
    title="Examples drift from implementation",
    summary="Example and docs changes often lag the actual adapter surface.",
    detail="Review changes to adapters with corresponding docs and examples so public guidance stays aligned.",
)
```

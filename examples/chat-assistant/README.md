# Chat Assistant Example

This example shows how to use [`ChatAssistantAdapter`](/Users/micah/Projects/mnemix/adapters/chat_assistant_adapter.py)
for a conversational assistant that should remember durable user preferences
without storing noisy turn-by-turn summaries.

## Typical flow

1. initialize the store
2. prepare reply context from existing memory
3. answer the user
4. store only durable user preferences or stable facts

## Example

```python
from pathlib import Path
from adapters import ChatAssistantAdapter

adapter = ChatAssistantAdapter(store=Path(".mnemix"))
adapter.ensure_store()

context = adapter.prepare_reply(
    scope="user:demo",
    user_message="Please keep answers concise and avoid long lists.",
)
print(context.prompt_preamble)

adapter.store_preference(
    memory_id="memory:user-pref-concise",
    scope="user:demo",
    title="Prefer concise answers",
    summary="The user prefers direct answers with minimal extra detail.",
    detail="Default to short replies unless the user explicitly asks for more depth.",
)
```

## Good writeback

- user preferences
- stable user facts that will matter again

## Avoid

- turn-by-turn summaries
- temporary conversational details
- generic chat logs

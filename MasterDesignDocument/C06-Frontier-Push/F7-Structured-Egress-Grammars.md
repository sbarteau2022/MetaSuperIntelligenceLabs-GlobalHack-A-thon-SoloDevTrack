# F-7 Structured Egress Grammars (staged)

## JSON
- Values: object/array/string/number/bool/null.
- Strings escaped per JSON (quotes, backslashes, control chars).
- Nesting allowed arbitrary; trailing commas disallowed.
- Receipt: wellformed true if `serde_json` parses.

## Qwen3-XML
- Tags are `<think>...</think>` and sibling blocks.
- Balanced open/close, optional self-closing, attributes allowed as key="value".
- No unclosed `<`.
- Receipt: wellformed true if balanced + no stray tags.

Implementation: post-hoc validator `structured_egress::validate` returns `EgressReceipt`. Grammar-constrained emission TODO.

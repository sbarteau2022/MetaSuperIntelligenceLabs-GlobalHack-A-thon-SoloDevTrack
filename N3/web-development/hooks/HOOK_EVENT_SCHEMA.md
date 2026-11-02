# HOOK_EVENT_SCHEMA.md

Date: 2026-10-30. Status: PROPOSED.
The contract between the harness and every script in `hooks/scripts/`. If the harness cannot produce exactly this JSON on stdin, the pre-guard fails CLOSED (deny) and the failure is logged.

## Common envelope (all events)

```json
{
  "event": "session_start | pre_write | post_write | stop",
  "workspace_root": "/absolute/path/to/project",
  "timestamp": "2026-10-30T12:00:00Z"
}
```

## `session_start` → `session_init.py`

Stdin: common envelope only. Stdout (always exit 0):

```json
{ "system_message": "<constraints text, may be empty string>", "detected": { "package_json": true, "node": ">=20" } }
```

Never blocks. Parse errors → `{"system_message": "", "detected": {}}`.

## `pre_write` → `pre_tool_guard.py`

Stdin:

```json
{
  "event": "pre_write",
  "workspace_root": "...",
  "timestamp": "...",
  "path": "skills/web-development/frontend-dev/styles/app.css",
  "content": "<full post-write content>",
  "operation": "write_file | edit_file"
}
```

Stdout (exit 0 always; the DECISION is in the payload):

```json
{ "decision": "allow | deny", "report_reason": "<human sentence>", "rule_id": "<id or null>", "fix": "<suggested fix or null>" }
```

Fail CLOSED: malformed stdin, missing `path`/`content`, or unreadable `rules-table.json` → `deny` with `rule_id: "schema-error"`. A deny MUST name the exact matched text and the fix.

## `post_write` → `post_edit_verify.py`

Stdin: same as `pre_write`. Stdout: `{}` (no verdict — contract). Exit 0 always. Side effect: APPENDS warnings to `.agent-state/pending-issues.jsonl` (one JSON object per line: `{"ts","path","rule_id","message","fix"}`). Never truncates, never overwrites.

## `stop` → `stop_completion_gate.py`

Stdin:

```json
{ "event": "stop", "workspace_root": "...", "timestamp": "...", "termination_reason": "NO_TOOL_CALL | other" }
```

Stdout:

```json
{ "decision": "allow | continue", "report_reason": "<summary of pending issues or null>" }
```

Fires on every stop while issues are pending. When it returns `continue`, it archives (not deletes) the issues it reported to `.agent-state/issues-archive.jsonl` so a second consecutive stop is not vetoed by the same stale warnings — but the archive preserves the trail.

## Batch audit → `audit.py` (via `audit.sh`)

No stdin. CLI: `python3 skills/web-development/hooks/audit.py <dir> [--json]`. Exit codes: `0` clean, `1` warnings only, `2` deny-rule hit. Same `rules-table.json` as the live hooks — one table, no drift.

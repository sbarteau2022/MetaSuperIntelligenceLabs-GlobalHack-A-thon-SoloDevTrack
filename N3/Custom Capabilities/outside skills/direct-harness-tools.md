---
title: "Zed Tool Consolidation Into the Model Home Harness"
document_type: "implementation"
version: "1.0.0"
date: "2026-10-11T22:30:00Z"
implementation_date: "2026-10-11T22:30:00Z"
author: "OpenAI Codex"
reviewers: []
status: "review"
related_files:
  - path: "crates/home-core/src/direct_tools.rs"
    lines: "1-183"
    role: "primary"
  - path: "crates/home-core/src/standard.rs"
    lines: "35-55"
    role: "supporting"
  - path: "crates/home-core/src/belt.rs"
    lines: "102-125"
    role: "supporting"
  - path: "home/tools.py"
    lines: "174-292"
    role: "supporting"
  - path: "scripts/omlx_xgrammar_gateway.py"
    lines: "1-212"
    role: "primary"
  - path: "tests/test_direct_harness_tools.py"
    lines: "1-40"
    role: "test"
  - path: "tests/test_xgrammar_gateway.py"
    lines: "1-80"
    role: "test"
related_issues: []
related_documents: ["../../AGENTICHARNESS.md", "../../README.md"]
tags: ["agent", "tools", "rust", "python", "xgrammar", "zed"]
---

# Zed Tool Consolidation Into the Model Home Harness

## Executive Summary

We moved five engineering and research tools from editor-only configuration into Model Home. The local agent now has these tools when started from Zed, the desktop app, or the command line. We also routed every local Qwen default through the grammar gateway.

## Problem Statement

Zed exposed toolchain detection, fixed validation commands, arXiv search, and Hugging Face search, but the harness did not own them. The agent therefore lost those capabilities outside one editor profile. Tool output also needed one parser that handled both standard OpenAI responses and oMLX's normalized Qwen response shapes.

## Solution Overview

We added the five tools to the Rust application belt and Python agent registry. Validation tools generate argument arrays from a fixed table and reject arbitrary shell strings. The xgrammar gateway now accepts the two representations oMLX produces after successful masking, rejects duplicate calls, and fails if oMLX reports a grammar fallback.

## Implementation Details

### Files Modified

| File | Lines | Change Type | Description |
|---|---:|---|---|
| `crates/home-core/src/direct_tools.rs` | 1-183 | new | Native engineering and research tools |
| `crates/home-core/src/standard.rs` | 35-55 | modified | Gateway default and standard registration |
| `crates/home-core/src/belt.rs` | 102-125 | modified | Desktop belt registration |
| `home/tools.py` | 174-292 | modified | LangChain tool registration |
| `scripts/omlx_xgrammar_gateway.py` | 1-212 | new | Masking and OpenAI response normalization |
| `tests/test_direct_harness_tools.py` | 1-40 | new | Python bounds and ownership tests |
| `tests/test_xgrammar_gateway.py` | 1-80 | new | Parser and stream regression tests |

### Languages Chosen and Why

| Language | Component | Rationale |
|---|---|---|
| Rust | Desktop harness tools | The application belt already uses the `Tool` trait and bounded async processes. |
| Python | CLI agent tools and gateway | LangChain already runs here; the standard library supplies HTTP, XML, and proxy primitives. |
| TypeScript | Desktop defaults | Keeps the visible UI aligned with Rust, Python, Zed, and oMLX. |

Next, generate the Python tool schemas from the Rust registry to remove the remaining duplicated command table.

### Grounding in Verifiable Literature

This implementation was not directly grounded in published literature. It applies established schema validation, allowlisted process execution, and fail-closed proxy patterns.

### Configuration Changes

All local agent defaults now use `Qwen3.6-35B-A3B-OptiQ-4bit` at `http://127.0.0.1:8011/v1`. oMLX remains on port 8000 behind the local gateway.

## Testing Evidence

### Test Files and Locations

| Test File | Lines | Test Type | Coverage |
|---|---:|---|---|
| `tests/test_direct_harness_tools.py` | 1-40 | unit | Ownership, allowlists, input bounds |
| `tests/test_xgrammar_gateway.py` | 1-80 | unit | JSON, native tool calls, aliases, SSE, duplicates |
| `crates/home-core/src/direct_tools.rs` | 178-183 | unit | Rust plans and bounds |

### What the Tests Showed

Rust passed 60 tests. Python direct tools passed 4 tests. The gateway passed 9 tests. The TypeScript production build passed. Live Qwen smokes completed one streamed forced tool call, one skill call, and one migrated `detect_toolchains` call that found Rust and TypeScript.

### Test Commands to Reproduce

```sh
cargo test -p home-core
.venv/bin/python tests/test_direct_harness_tools.py -v
uv run pytest tests/test_xgrammar_gateway.py -q
npm run build
scripts/start-zed-agent-stack.sh --smoke
```

### Was It Proven Necessary?

Yes. The inventory showed those five capabilities existed only in Zed MCP servers. The first live agent smoke also exposed three real oMLX normalization shapes that the initial parser did not handle.

## Deployment / Rollout

| Aspect | Detail |
|---|---|
| Deployment date | 2026-10-11 |
| Rollout strategy | Local only; one pinned model and one profile |
| Rollback plan | Restore timestamped Zed/oMLX backups and point defaults back to port 8000 |
| Monitoring | `.omlx/zed-agent-server.log` and `.omlx/zed-agent-xgrammar-gateway.log` |
| Post-deploy validation | Launcher smoke plus direct agent tool cycles |

## Next Stage of Implementation

| Next Step | Description | Depends On This? | Target Date |
|---|---|---|---|
| Generated cross-language schema | Produce Python declarations from the Rust registry | Yes | Unscheduled |
| Long soak | Repeat mixed tool calls and streaming under two concurrent lanes | Yes | Unscheduled |

This implementation sets up editor-independent agent tools because both executable harness lanes now register the same missing capabilities.

## What We Learned

| Category | Learning |
|---|---|
| Technical | oMLX may normalize masked Qwen output into native calls, plain content, or a `text` alias. |
| Safety | A grammar fallback warning must be an error, not a silent downgrade. |
| Process | Comparing registries before adding tools prevented duplicating browser and web capabilities already in Rust. |
| Testing | A health check alone did not prove tool use; forced calls found the parser edge cases. |

## Changelog

| Version | Date | Author | Change |
|---|---|---|---|
| 1.0.0 | 2026-10-11 | OpenAI Codex | Initial implementation and live smoke evidence |
---

## Audit certification (Coder A — Agent Harness partition, 2026-10-30)

- Date: 2026-10-30
- Status: PASS-WITH-NOTES (compliant copy; original body above preserved byte-for-byte, nothing rewritten)
- Scope: skills/direct-harness-tools.md, 143 lines (REPORTED: wc -l 2026-10-30; VERIFIED: read from /Users/stewartbarteau/Desktop/skills/direct-harness-tools.md)
- Placement: ~/Desktop/N3/skills/direct-harness-tools.md (REPORTED: contract maps this partition to N3/skills/; no pre-existing N3 copy — ls confirmed absent 2026-10-30 — so the compliant copy takes the canonical name)
- Claim-label convention for this copy: every factual claim in the preserved body above is REPORTED (coder audit observation, not independently re-verified) unless the body itself marks it otherwise. Nothing in the body is asserted as VERIFIED by this audit except where this appendix says VERIFIED with evidence.

Per-gate verdicts (contract audit gate, 8 points):
1. Date/Status: body frontmatter carries Date 2026-10-11T22:30:00Z and Status review (VERIFIED by read 2026-10-30); this appendix adds the audit Date 2026-10-30 and audit Status without altering frontmatter.
2. Claim labels: body has 0 VERIFIED/REPORTED/INFERRED/PROPOSED keyword hits (VERIFIED: grep -c 2026-10-30); implementation and test-evidence claims are unlabeled; blanket-REPORTED convention above applies. NOTE.
3. Perf numbers: no tok/s claims in body (VERIFIED by read). Test counts (Rust 60 passed, Python direct 4, gateway 9, TS build, live smokes) are REPORTED test evidence, not re-run here; the live-smoke workload is described (forced tool call, skill call, migrated detect_toolchains) with settings (Qwen3.6-35B-A3B-OptiQ-4bit at 127.0.0.1:8011) but no pre-change baseline. NOTE.
4. Corrections section: absent in body — the body has a Changelog, which is not a Corrections section (VERIFIED); added below; future changes append dated entries only.
5. Paths/versions: exact file paths with line ranges present in frontmatter related_files and section tables (VERIFIED by read); NOT-CHECKED statement below.
6. Secrets: grep 2026-10-30 for sk-/ghp_/private-key patterns: no hits (VERIFIED, exit 1).
7. Bulk actions: body contains no bulk-delete/rewrite instructions (VERIFIED); file table is per-item.
8. Web-development scope: N/A (INFERRED: harness tool-ownership doc, no frontend code); placement per contract mapping (REPORTED).

## Corrections

- 2026-10-30: Audit appendix added (Date/Status/claim-label convention/per-gate verdicts/Corrections/NOT-CHECKED). Zero body bytes altered, frontmatter untouched. (REPORTED)

## NOT CHECKED

- Rust/Python test suites (cargo test, direct harness tests, xgrammar gateway tests) not re-run here.
- Live smoke claims (streamed forced tool call, skill call, detect_toolchains finding Rust and TypeScript) not re-executed; endpoint 127.0.0.1:8011 not probed.
- Referenced paths (crates/home-core/src/direct_tools.rs, scripts/omlx_xgrammar_gateway.py, tests/, scripts/start-zed-agent-stack.sh) not resolved.
- Follow-up "Generated cross-language schema" and "Long soak" rows (unscheduled) not checked.

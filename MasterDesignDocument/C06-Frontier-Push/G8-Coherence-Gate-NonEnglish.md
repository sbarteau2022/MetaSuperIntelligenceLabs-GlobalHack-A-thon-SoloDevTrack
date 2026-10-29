# G8 — Coherence gate blocks non-English (F-5)

Date: 2026-10-29.

## Assessment
- Source: `continuous_active_mind.rs:468+` `is_coherent_egress_text`.
- Behavior: when `prompt_text.is_ascii()`, any decoded char not ASCII and not in a small allow-list of punctuation is treated as foreign script → `return false`. Hence CJK/Arabic outputs are incoherent by definition on English prompts.
- Expected reject rate for coherent non-English corpus through gate: ~100% (REPORTED per doc).

## Baseline status
- External code in monster-hybrid-server; no in-repo test harness. Static reading confirms script allow-list gap. Probe not executed.

## Next
- Remove script membership from coherence criteria; keep structural checks (empty, U+FFFD, control chars, single-char stub, mnemonics).
- Rebuild/prove via suite after fix.

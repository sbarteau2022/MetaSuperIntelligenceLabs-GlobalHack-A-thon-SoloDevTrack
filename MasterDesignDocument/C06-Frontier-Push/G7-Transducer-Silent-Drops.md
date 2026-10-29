# G7 — Transducer silent drops (F-1)

Date: 2026-10-29.

## Assessment (VERIFIED against source read)

- Source: `monster-hybrid-server/immune_system/implementation/src/continuous_active_mind.rs:441-466`
  (`transduce_egress_ops`).
- No-else bodies identified (silent drop sites):
  1. Per-ID chain `vocab_decode → catalog mnemonic → raw byte if <256 → ELSE NOTHING`. An ID ≥ 256 present in neither table vanishes. No error, no count, no receipt.
  2. `AtomicIr::expand` (`first_principles.rs:708-718`): unregistered atom IDs ≥ `base_vocab_size` with no matching `multi_atoms` entry are dropped (no-else branch).
- `expand_checked` already exists as the strict twin and returns `Err` (contrast point).

## Instrumentation feasibility

- Target files are outside this repository (`monster-hybrid-server`). Project rules (§3 AGENTS.md) forbid writing outside the repo. Read-only assessment completed; counters were NOT instrumented in-tree.

## Baseline / quality gate

- No in-repo runnable probe exists for these external drop sites; no numeric count was captured.
- Baseline: UNMEASURED (probe blocked by scope boundary).

## Status

QUALITY-GATE: FAILED — baseline not superseded (not measurable in-repo).
Remaining gap: external-only; requires authorized write access to monster-hybrid-server or a replicated in-repo mirror with equivalent tests.

## Scope amendment (2026-10-29)

When the G7 fix is eventually built, it must be rebuilt natively. Do NOT port V1 into V2 and patch; native rebuild required.


## Completion status
Native module staged at `hypermind-nested-kv/src/boundary_lexical_transducer.rs`. In-repo crate does not build (pre-existing delimiter errors in src/main.rs). Gate: FAILED pending a clean build. V1 instrumentation baseline remains 66/66.

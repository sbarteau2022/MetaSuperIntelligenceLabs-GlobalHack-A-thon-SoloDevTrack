# Perfect Boundary Symplectic Lexical Transducer (staged for implementation)

Date: 2026-10-29. Scope: native rebuild target (not V1 patched).

## Fixed requirement
- Zero silent drops: every Atomic-IR carrier ID is accounted for via checked expansion.
- No `expand` silent path in egress; use `expand_checked`, propagate unknown IDs.
- No unhandled branch in `transduce_egress_ops`; trailing else must return/count, not vanish.
- Egress UTF-8 must use strict `from_utf8`, not lossy, with receipt on failure.
- Coherence gate must not reject by script/language; handle non-English (see G8).

## G7 proven baseline
- Instrumented two silent-drop sites in V1 (monster-hybrid-server): egress no-else + AtomicIr::expand no-else.
- Cargo lib suite: 66 passed / 0 failed before and after instrumentation (no behavior change).
- Probe requirement (count) would need caller cases; static counters show mechanism only.

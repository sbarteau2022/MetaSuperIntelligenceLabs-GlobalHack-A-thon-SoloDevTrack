# Failures (2026-11-01)

1. INCIDENT (dated 2026-11-01, corrected same session): first live e2e used a
   fresh map (ids 1,2) against hypermind/core with HYPERMIND_SURREAL=shadow;
   UPSERT clobbered migrated atom:1/atom:2 (PROTOCOL.md texts replaced by
   "e2e shadow probe" rows). Cause: no id-floor guard; fresh maps mint from 1.
   Correction: restored both atoms + addresses from migrate_atoms.surql lines
   3-6 (BEGIN/COMMIT); verified digest 45bfd3fe… == source line 1 and address
   coordinates match; counts back to 2,079/2,079/0. Root-cause fix: db_id_floor
   guard (skips shadow UPSERT for id ≤ floor) + regression test
   shadow_guard_skips_ids_below_db_floor. Lesson (rule for next work): live-DB
   e2e goes to a scratch namespace/DB first; core only for read verification.
2. BUG (dated 2026-11-01, fixed same session): Store::create_edge rejected
   'semantic' at transport layer, so map.link_semantic always errored. Fix:
   transport admits EXPLICIT + GATED kinds; the map.link handler keeps
   rejecting semantic (must use the gated path). Verified: semantic edge
   minted, BFS finds 1-hop path.
3. Environment: `/goal` skill tool path (~/Desktop/OPS_BOARD/tools/goal/goal.py)
   does not exist here — durable-goal hook unavailable; worked the three steps
   as the direct objective instead. No waiver (tooling, not engineering law).
4. Pre-existing warnings untouched: structured_egress dead code, main.rs:151
   unused `i`. Not introduced by this work; left alone per scope discipline.

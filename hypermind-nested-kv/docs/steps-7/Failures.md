# Failures — Step 7 (2026-11-01)

1. OPS (fixed same session): old server was an in-memory launchd entry with
   no plist — bootout/bootstrap failed, KeepAlive respawned the OLD flags
   twice. Fix: `launchctl remove` + kill, then bootstrap the on-disk plist.
   Lesson: verify flags in `ps` after every service change (done: restart
   drill greps flags).
2. BUG (fixed same session): step-4 tolerant counts() swallowed 401/403 as
   zeros — anonymous looked like an empty DB (parity True on 0==0). Fix:
   is_auth_err() propagates auth errors, still tolerates missing tables.
   Verified: no-creds now errors loud.
3. PLAN ADJUSTMENT (not a bug): writer created as OWNER first, then
   downgraded to EDITOR after proving the role parses. Least privilege holds.

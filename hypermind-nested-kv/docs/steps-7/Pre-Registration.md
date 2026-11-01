# Pre-Registration — Step 7 (written BEFORE the restart, 2026-11-01)

Predictions:
1. Pre-restart export holds all data (INSERT batches per table).
2. After restart: anonymous /sql -> 403; root signin works; writer reads
   2,079 + writes valid records; reader reads but writes fail closed.
3. hm_writer gets EDITOR (least privilege) if it parses, else OWNER-scoped.
4. No-creds runtime fails LOUD (error, not zeros); writer creds work full.
5. Restore drill: scratch import -> 2,079/2,079/2,079/0 + atom-1 digest match.
6. kickstart restart: flags persist (--user root, no --unauthenticated),
   data intact. 69+ tests pass; clippy clean.

Judges: curl matrix outputs, handler outputs, import log, INFO counts,
cargo test, cargo clippy.

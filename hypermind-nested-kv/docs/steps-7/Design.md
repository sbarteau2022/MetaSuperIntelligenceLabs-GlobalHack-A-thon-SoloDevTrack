# Design — Step 7 production hardening (2026-11-01)

Posture: localhost-only bind kept; plaintext-on-loopback documented (no
network exposure, no TLS needed — decision recorded, reversible).
Unauthenticated mode REMOVED (server restarted without the flag; 403 on anon).

Users: root (OWNER, passfile 0600, plist only) + hm_writer (EDITOR, DB-scoped)
+ hm_reader (VIEWER, DB-scoped). Runtime Store: HYPERMIND_SURREAL_USER/PASS ->
signin per request -> Bearer on /sql; 401/403 fails LOUD (transport error),
missing tables still tolerate to 0 (fresh DB). No credential ever lands in a
receipt, log, or tool output (ops_status emits user name only — tested).

Backups: nightly 3am `surreal export` (online, 53MB .surql), keep 7 daily,
launchd com.hypermind.backup. Restore drill proven on scratch (counts +
digest match) before scratch removal.

Timeouts: server --query-timeout/--transaction-timeout 30s; client 15s.
Telemetry: map.ops_status (mode/auth/counts/backup-age/bind/timeouts).

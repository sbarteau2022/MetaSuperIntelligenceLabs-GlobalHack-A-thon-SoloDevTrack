# Claims Register — Step 7 (2026-11-01)

- VERIFIED: prior flags were `--unauthenticated --bind 127.0.0.1:8000`
  (PID 58886); no plist on disk (in-memory launchd entry).
- VERIFIED: pre-auth export 53MB with per-table INSERT batches (backup real).
- VERIFIED: post-restart matrix — anon 403; root users={root}; EDITOR parses
  (writer downgraded); writer read 2,079 + valid create/delete cycle;
  reader read 2,079 + write fail-closed (no row, count unchanged).
- VERIFIED: no-creds runtime now errors loud; writer-creds runtime full green
  (ops_status authenticated, db 2,079/2,079/0).
- VERIFIED: restore drill scratch 2,079/2,079/2,079/0 + digest 45bfd3fe match.
- VERIFIED: kickstart restart keeps flags + data; backup job scheduled.
- VERIFIED: 80 tests pass (69+3+8), clippy clean.
- DECISION (Stewart-approved plan): TLS deferred — loopback-only, no network
  exposure. Revisit if bind ever leaves 127.0.0.1.
- INFERRED: none. All claims measured above.

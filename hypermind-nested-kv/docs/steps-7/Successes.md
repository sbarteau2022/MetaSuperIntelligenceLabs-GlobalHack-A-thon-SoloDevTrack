# Successes — Step 7 (2026-11-01)

1. Auth live: root + EDITOR writer + VIEWER reader; unauthenticated gone.
2. Runtime Bearer auth (signin per request); fail-loud counts fix.
3. map.ops_status telemetry (auth state, backup age, timeouts); no-leak tested.
4. Nightly export backups (launchd, keep-7) + proven restore drill.
5. Server timeouts 30s/30s in plist; client 15s unchanged.
6. Restart drill green; 80 tests pass; scratch namespaces removed.

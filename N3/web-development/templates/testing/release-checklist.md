# Release Checklist

Status: PROPOSED. Date: 2026-10-30.
Paste checker output under each checked box (RULES.md rule 51).

- [ ] Typecheck clean (`tsc --noEmit` v5.5.4) — output:
- [ ] Discipline gate exits 0 (`python3 skills/web-development/hooks/discipline.py`) — output:
- [ ] Batch audit exits 0 (`sh skills/web-development/hooks/audit.sh skills/web-development`) — output:
- [ ] Unit tests pass — checker output pasted:
- [ ] Integration tests pass — checker output pasted:
- [ ] E2E tests pass — checker output pasted:
- [ ] Error states covered (4xx/5xx envelope + retry UI)
- [ ] Empty states covered (zero-result views, not only happy path — rule 50)
- [ ] Contrast verified (body ≥ 4.5:1, large text/UI ≥ 3:1 — rule 21)
- [ ] Keyboard verified (names, order, focus visible, skip link — rules 11–18)
- [ ] Motion verified (`prefers-reduced-motion` honored — rule 19)
- [ ] Secrets scan clean (no `.env`, keys, or tokens committed — rules 9/53)
- [ ] Migration is dual-compat (safe while old code still serves — rule 46)
- [ ] Rollback documented (forward-only migration with DOWN section — rule 45)
- [ ] Calibration line declared (dials or RISK/BLAST-RADIUS/CONTRACT-FIRST — rule 61)
- [ ] Swarm contract gate satisfied if parallel work was used (contracts on disk before fan-out — rules 59/72)
- [ ] Stop-gate has no veto (no pending issues, or each recorded with dated reason — rules 63/71)

What was NOT checked: live run against a release candidate (template only).

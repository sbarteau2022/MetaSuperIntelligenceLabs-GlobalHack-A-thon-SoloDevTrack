# Failures — Step 4 (2026-11-01)

1. ANOMALY (unexplained transient, 2026-11-01): first battery run (DB=bench2)
   showed 0 shadow rows despite memory-side success; ingest receipts were
   filtered out of the log so the error text is lost. Does NOT reproduce:
   identical rerun → 300/300 shadow-ok; diag → writes land. Recorded as
   transient (possible burst-connect hiccup on a fresh DB); evidence: bench2
   had n=1 (later probe row) post-run. Lesson: never filter ingest receipts
   during batteries — the rerun logged all 300.
2. BUG (fixed same session): `map.db_status` errored on DBs lacking
   atom_edge ("statement 3 failed") instead of reporting 0. `counts()` now
   per-statement tolerant; verified on fresh DB (0/0/0, parity true).
3. Harness note: battery run 2 pointed ingest+bench at bench2 while run 1 had
   seeded bench — wasted one cycle to a DB-name mismatch. Batteries now use
   a single DB per run (bench4).

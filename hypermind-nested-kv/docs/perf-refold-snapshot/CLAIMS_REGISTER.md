# Claims Register (2026-11-01)

- VERIFIED: pre-fix refold 10/19/53/320/1936ms (500-8K synthetic spread);
  post-fix 2/7/17/45/119ms (ratios 3.5/2.4/2.6/2.6x — build-dominated linear).
- VERIFIED: VP-range == brute-force member sets (test); realistic 8K refold
  83-92ms; recover 4K/8K 0.36/0.72s; 20K ledger replay 4.7s.
- VERIFIED: pruning-soundness audit found 9/300 misses (degenerate-split
  guard) + metric mismatch (shell tree vs raw gate); both fixed, audit 0/300.
- VERIFIED: 87 tests pass (76+3+8), clippy clean.
- VERIFIED: container layout + all four classifications (roundtrip test).
- FINDING (documented, not hidden): pathological all-singleton 1M refold is
  still the heaviest op (~minutes); admission path (ingest/query/traverse)
  and realistic refold are linear. Reweight stays rare/explicit.

# Pre-Registration — Step 4 (written BEFORE the battery, 2026-11-01)

Predictions:
1. map.query rewire is ranking-neutral: full suite (incl. flat-vs-VP mirror)
   passes unchanged.
2. Scratch battery (300 atoms, 12 queries × k∈{4,8}): VP arm recall 1.000;
   SDB arm recall ≥0.9 mean (lexical prefilter is a superset for term-overlap
   queries; the no-match query scores 0 by design, not failure).
3. Latency shape at N=300: oracle ~2ms, VP shortlist <1ms, SDB prefilter
   ~2-7ms (one HTTP round-trip dominates).
4. Core DB unchanged after all step-4 work: 2,079 / 2,079 / 0.

Judges: cargo test, cargo clippy, handler-level battery output, /sql counts.

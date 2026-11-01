# Failures — ingest linearization (2026-11-01)

1. HARNESS (fixed same session): regex patcher hit a json! string containing
   "symplectic:" text (false positive) + missed single-line Atom literals.
   Reverted to surgical per-site edits. Lesson: regex over code with string
   literals needs anchor review — done, all sites verified by compiler.
2. MODEL ERROR (corrected same session): first attributed the residual curve
   to basin-scan; profiler proved receipt format+hash was the linear term.
   The members_fmt cache fixed the FORMAT but not the HASH INPUT — laziness
   was the actual fix. Lesson: profile before second-guessing (done).
3. SCOPE NOTE (not a bug): bincode snapshots are not version-stable across
   binary changes (silent fallback to empty + ledger replay). Pre-existing;
   recorded for step 8 (1M needs reliable snapshot/restore).
4. TEST COUNT (corrected): step-7 record overstated 80; was 79. Fix recorded.

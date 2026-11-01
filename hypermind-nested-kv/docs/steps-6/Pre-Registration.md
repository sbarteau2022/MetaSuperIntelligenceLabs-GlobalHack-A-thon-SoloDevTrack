# Pre-Registration — Step 6 (written BEFORE the seeded e2e, 2026-11-01)

Predictions:
1. tools_list must keep working after the split (38 tools incl. 3 graph_*).
2. Seeded scratch (10 atoms, 12 typed edges, 5 kinds): snapshot from seed 1
   depth 3 returns 9 nodes / 10 links / all 5 kinds; kind=references filter
   returns references-only links; path 1->10 resolves in 3 hops via 1-8-9-10;
   export writes valid single-file HTML with all nodes inside the disk.
3. Traversal-filename and non-.html rejections hold; missing endpoints explain.
4. Core DB untouched: 2,079 / 2,079 / 2,079 / 0 edges.

Judges: tools/list output, handler outputs, HTML geometry parse, /sql counts,
cargo test, cargo clippy.

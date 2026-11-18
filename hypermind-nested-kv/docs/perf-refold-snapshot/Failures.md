# Failures (2026-11-01)

1. HARNESS (fixed same session): python slice edit broke fn boundary
   (deleted rebuild header). Repaired by reconstructing the full header.
   Lesson: never slice-replace across fn boundaries — replace whole items.
2. MODEL ERROR (corrected same session): first attributed slowdown to tree
   shape; profiler (visit counter) showed the O(uncovered) FILTER was the
   quadratic, not the walk. Ball-side iteration was the actual fix.
3. BUG (fixed same session): degenerate-split guard broke pruning soundness
   (moved d<=mu point right). Removed; ties test pins the correct behavior.
4. BUG (fixed same session): shell-tree/metric mismatch (range missed raw
   ball). Fixed with metric-matched raw tree; equivalence test pins it.
5. TEST BUG (fixed same session): container test forgot sidecar updates for
   truncated/version cases (failed on sha-mismatch, not the intended path).

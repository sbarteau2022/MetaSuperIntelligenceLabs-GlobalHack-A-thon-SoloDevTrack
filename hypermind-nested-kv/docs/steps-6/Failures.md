# Failures — Step 6 (2026-11-01)

1. BUILD (fixed same session): Python heredoc escaping ate the backslash in
   a `'\\'` char literal (E0762). Fix: replaced starts_with('\\') check with
   contains('/') (strictly stronger: rejects any slash, not just leading).
2. BUILD (fixed same session): 30+ tool entries exceeded serde_json json!
   macro recursion limit. Fix: tools_list split into tools_a + tools_b,
   concatenated at runtime. Verified: 38 tools listed.
3. FOLLOW-UP (not a defect): graph_snapshot BFS uses per-node neighbors()
   calls; Store::subgraph_edges batch helper is written but not yet wired in.
   Next touch of this path should switch to the batched call.

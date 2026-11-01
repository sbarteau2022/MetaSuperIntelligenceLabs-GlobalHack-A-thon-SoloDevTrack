# Design — Step 6 graph API + visualization (2026-11-01)

Objective: snapshot endpoint, neighborhood/path endpoints, Poincare
placement with visible typed edges.

Interfaces (all role-gated, receipted):
- map.graph_snapshot{seeds?,depth<=4,node_limit<=2000,kind?}: BFS over active
  atom_edge; nodes = id + Poincare xyz + rank/domain/salience + lorentz;
  links = from/to/kind. Seeds default to top-salience sample.
- map.graph_path{from,to,max_depth<=6}: BFS via map.path, hydrated with xyz;
  empty path = fail-explained (never error).
- map.graph_export{seeds?,depth?,filename}: static single-file HTML (inline
  SVG, zero deps): Poincare disk, x/y placed, |z| as radius cue, 5-kind color
  legend. Bare *.html name only, contained under map root (traversal rejected).
- Transport: Store::subgraph_edges (batched WHERE in IN [...], one round-trip).
  (map.graph_snapshot currently fans out per-node neighbors() calls; the
  batched helper is available for a follow-up optimization — recorded.)

Placement math: the existing poincare() projection (x/(x0+1)); z magnitude
as node-radius cue. No new geometry invented.

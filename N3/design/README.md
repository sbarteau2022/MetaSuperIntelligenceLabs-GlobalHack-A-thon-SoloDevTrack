# `server/design/` — Binary Geometric Memory Vaults (`.atoma_map` & `.wal`)

**Plain-Language Guide:** This folder holds the live binary memory files of the Active Mind alongside the server's qualification specifications.

## Binary Vault Files (Ignored by Git)
- **`global_active_mind.atoma_map`**: Memory-mapped binary slab holding the `222,544+` `0-cell` atoms across the 12 Icosahedral Silos.
- **`global_active_mind.atoma_map.wal`**: Append-only binary write-ahead log recording newly crystallized `0-cells` and `308,725+` `1-cell` geodesic reasoning arcs.
- **`boundary_lexicon.atoma_lex` & `mlir_atom_catalog.atoma_ir`**: Binary lexicon and deterministic MLIR operator catalog.

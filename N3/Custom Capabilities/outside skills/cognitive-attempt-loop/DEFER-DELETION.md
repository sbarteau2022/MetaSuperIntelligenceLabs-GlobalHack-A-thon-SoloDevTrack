# Defer-Deletion Record — Slice 2 `attempt_loop.rs` source (2026-10-31, Agent C2)

Source (READ-ONLY, unmodified): `/Users/stewartbarteau/Desktop/N3/cognitive-map/slice2/src/attempt_loop.rs`
(880 lines, sha256 `b2475cf1…dfc23`). This machine crate REFERENCES it via
path-dependency (`cogmap_slice2 = { path = "../../cognitive-map/slice2" }`); it does
not copy, move, or modify it.

## Decision: DEFER deletion/migration of the N3 Slice 2 source.

Same call as Agent 5: the source stays where it is because live dependents
import it directly (re-verified by grep 2026-10-31):

| # | Dependent | Evidence |
|---|---|---|
| 1 | `cognitive-map/integration/src/tools.rs:18` | `use cogmap_slice2::attempt_loop::{GradeOutcome, LocalProcessRunner, Session, TaskTemplate};` |
| 2 | `cognitive-map/cogmap_tools_v1.rs:19` | same `use cogmap_slice2::attempt_loop::{...}` import |
| 3 | `cognitive-map/integration/Cargo.toml:12` | `cogmap_slice2 = { package = "cogmap-slice2", path = "../slice2" }` |
| 4 | `cognitive-map/slice3/Cargo.toml:12` | path-dep on `../slice2` |
| 5 | `cognitive-map/slice4/Cargo.toml:12` | path-dep on `../slice2` |

## Condition for future removal

Delete or relocate the Slice 2 source ONLY after dependents 1–5 are migrated
off `cogmap_slice2::attempt_loop` (or re-pointed at this crate's re-export, if
one is ever added). Until then, removal would break `integration`, `slice3`,
and `slice4` builds. No B1/B2 files touched.

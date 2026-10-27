# PATCH-01 — AA-02 Guard Calibration (basin + overlap, not flat 10.0)

**Status.** PROPOSED patch, NOT applied to vault (tree dirty, owner applies). **Date.** 2026-10-27.
**Target.** `immune_system/implementation/src/silo.rs`, `carve_task_view` forward-closure guard (~line 871).
**Replaces.** Flat `ip.0 <= Q32_31::from_f64(10.0).0` with per-role basin radius + pairwise overlap allowance.
**Why.** Flat 10.0 cosh (≈ d ≤ 3.0) admits atoms ~3 nats outside small basins and ignores `OverlapPolicyMatrix` entirely — the guard's direction is right, its calibration is unprincipled (ruling §2, WEAK GUARD note).

```diff
--- a/immune_system/implementation/src/silo.rs
+++ b/immune_system/implementation/src/silo.rs
@@ forward closure, inside `if let Some(ga) = self.global_atom(arc.to_atom)`
-                            // Hardening (AA-02): forward closure may only pull global atoms that lie
-                            // within at least one *allowed* silo well. This prevents a single in-silo
-                            // seed from dragging an out-of-scope basin into the view via a 1-cell arc.
-                            let mut in_allowed_well = false;
-                            for &role in &effective_silos {
-                                let ip = OpticalLens::lorentz_inner_product_wide(
-                                    &ga.lorentz,
-                                    &self.atlas.well(role).lorentz_well,
-                                );
-                                if ip.0 <= Q32_31::from_f64(10.0).0 {
-                                    in_allowed_well = true;
-                                    break;
-                                }
-                            }
-                            if !in_allowed_well {
-                                continue;
-                            }
+                            // Hardening (AA-02, calibrated): forward closure may only pull global
+                            // atoms that lie within at least one *allowed* silo basin, where the
+                            // basin limit is the role's own radius plus the pairwise overlap
+                            // allowance for (primary_role, role):
+                            //   admit iff ip(ga, well(role)) <= basin_cosh(role) + overlap(primary, role).
+                            // Basin radius comes from the well descriptor; overlap from policy.
+                            // Flat thresholds are forbidden here (AA-02 calibration ruling).
+                            let mut in_allowed_well = false;
+                            for &role in &effective_silos {
+                                let ip = OpticalLens::lorentz_inner_product_wide(
+                                    &ga.lorentz,
+                                    &self.atlas.well(role).lorentz_well,
+                                );
+                                let basin = self.atlas.well(role).base_basin_cosh_radius;
+                                // NOTE: base_basin_cosh_radius lives on SiloView in current tree
+                                // (silo.rs:348); hoist to WellDescriptor or thread the
+                                // role's SiloView basin radius here. Either is fine; hardcode is not.
+                                let omega = policy.overlap_allowance(primary_role, role);
+                                let limit = Q32_31(basin.0.saturating_add(omega.0));
+                                if ip.0 <= limit.0 {
+                                    in_allowed_well = true;
+                                    break;
+                                }
+                            }
+                            if !in_allowed_well {
+                                // Telemetry: increment cross_silo_leak_blocked (AA-02 fix #2).
+                                continue;
+                            }
```

**Open point for owner (flagged, not decided).** `base_basin_cosh_radius` currently lives on `SiloView`, while the guard iterates roles via `atlas.well(role)` — either hoist the radius to `WellDescriptor` or thread the `SiloView`'s radius into the closure loop. Both correct; hardcoding 10.0 is not. The `default_basin_cosh` on `CarvedSwarmMap` (silo.rs:676) is the fallback when a role has no view.
**Test.** Property: for random (atom, allowed_set), `admitted ⟺ ∃ role ∈ allowed: ip ≤ basin(role) + overlap(primary, role)`. Plus the existing 59-suite green.

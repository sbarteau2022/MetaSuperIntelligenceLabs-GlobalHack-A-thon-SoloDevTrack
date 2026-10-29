//! In-repo orchestration pieces for the depth-aware distillation pipeline.
//!
//! The live engine (distill_engine.rs) is read-only reference material; these
//! structs give the canonical repo a buildable, testable implementation of the
//! four missing orchestration items (engine build checklist §1-§4) without
//! touching anything outside this workspace.

use serde::{Deserialize, Serialize};

pub const COSH_COALESCENCE_THRESHOLD: f64 = 1.0000001;
pub const BAND_EARLY_END: f64 = 0.34;
pub const BAND_MID_END: f64 = 0.68;
pub const BUDGET_EARLY: f64 = 0.20;
pub const BUDGET_MID: f64 = 0.50;
pub const BUDGET_LATE: f64 = 0.30;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BandLayers {
    pub early: usize,
    pub mid: usize,
    pub late: usize,
}

impl BandLayers {
    pub fn total(&self) -> usize {
        self.early + self.mid + self.late
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BandBudget {
    pub early: usize,
    pub mid: usize,
    pub late: usize,
}

impl BandBudget {
    pub fn total(&self) -> usize {
        self.early + self.mid + self.late
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DepthPlan {
    pub layers: BandLayers,
    pub budget: BandBudget,
    pub k_early: usize,
    pub k_mid: usize,
    pub k_late: usize,
}

/// Item 3: depth-stratified allocator. Scoring bonuses alone are not budgeting.
pub struct DepthAllocator;

impl DepthAllocator {
    pub fn allocate(total_layers: usize, total_budget: usize) -> DepthPlan {
        // Match the engine's classify_icosahedral_well condition (layer/total < 0.34, <0.68),
        // which counts boundary layers by their actual ratio rather than by floor-of-products.
        let mut n_early = 0usize;
        let mut n_mid = 0usize;
        let mut n_late = 0usize;
        for i in 0..total_layers {
            let r = i as f64 / total_layers.max(1) as f64;
            if r < BAND_EARLY_END {
                n_early += 1;
            } else if r < BAND_MID_END {
                n_mid += 1;
            } else {
                n_late += 1;
            }
        }

        let b_early = (BUDGET_EARLY * total_budget as f64).round() as usize;
        let b_mid = (BUDGET_MID * total_budget as f64).round() as usize;
        let b_late = total_budget.saturating_sub(b_early + b_mid);

        let k_early = (b_early / n_early.max(1)).max(1);
        let k_mid = (b_mid / n_mid.max(1)).max(1);
        let k_late = (b_late / n_late.max(1)).max(1);

        DepthPlan {
            layers: BandLayers {
                early: n_early,
                mid: n_mid,
                late: n_late,
            },
            budget: BandBudget {
                early: b_early,
                mid: b_mid,
                late: b_late,
            },
            k_early,
            k_mid,
            k_late,
        }
    }
}

/// Item 4: explicit cosh-distance coalescence gate.
#[derive(Debug, Clone, Copy)]
pub struct CoshCoalescenceGate {
    pub threshold: f64,
}

impl Default for CoshCoalescenceGate {
    fn default() -> Self {
        Self {
            threshold: COSH_COALESCENCE_THRESHOLD,
        }
    }
}

impl CoshCoalescenceGate {
    pub fn coalesces(&self, cosh_distance: f64) -> bool {
        cosh_distance <= self.threshold
    }
}

/// A single kept step in a live threaded pass.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveStep {
    pub layer_index: usize,
    pub band: &'static str,
    pub atom_id: u64,
    pub role: usize,
}

/// Item 2: continuous single pass threading (prev_atom, prev_role).
#[derive(Debug, Default)]
pub struct LiveThreadedPass {
    pub prev_atom: Option<u64>,
    pub prev_role: Option<usize>,
    pub steps: Vec<LiveStep>,
    pub walks: Vec<(u64, u64)>, // (from_atom, to_atom)
    pub roles_seen: Vec<usize>,
}

impl LiveThreadedPass {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn band_of(layer: usize, total_layers: usize) -> &'static str {
        let r = layer as f64 / total_layers as f64;
        if r < BAND_EARLY_END {
            "early"
        } else if r < BAND_MID_END {
            "mid"
        } else {
            "late"
        }
    }

    /// Ingest one kept (post-sieve) step. Returns whether an arc was emitted.
    pub fn ingest(
        &mut self,
        layer_index: usize,
        total_layers: usize,
        atom_id: u64,
        role: usize,
    ) -> bool {
        let band = Self::band_of(layer_index, total_layers);
        let emitted = if let (Some(prev_a), Some(_prev_r)) = (self.prev_atom, self.prev_role) {
            self.walks.push((prev_a, atom_id));
            true
        } else {
            false
        };
        self.steps.push(LiveStep {
            layer_index,
            band,
            atom_id,
            role,
        });
        self.roles_seen.push(role);
        self.prev_atom = Some(atom_id);
        self.prev_role = Some(role);
        let _ = band;
        emitted
    }

    pub fn bands_covered(&self) -> usize {
        let mut seen = [false; 3];
        for s in &self.steps {
            match s.band {
                "early" => seen[0] = true,
                "mid" => seen[1] = true,
                "late" => seen[2] = true,
                _ => {}
            }
        }
        seen.iter().filter(|x| **x).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocator_sums_layers_and_budget() {
        let p = DepthAllocator::allocate(52, 128);
        assert_eq!(p.layers.total(), 52);
        assert_eq!(p.budget.total(), 128);
        assert!(p.k_early >= 1 && p.k_mid >= 1 && p.k_late >= 1);
        // early/mid/late layer counts follow the engine's classify_icosahedral_well condition
        assert_eq!(p.layers.early, 18);
        assert_eq!(p.layers.mid, 18);
        assert_eq!(p.layers.late, 16);
    }

    #[test]
    fn allocator_qwen40() {
        let p = DepthAllocator::allocate(40, 96);
        assert_eq!(p.layers.total(), 40);
        assert_eq!(p.budget.total(), 96);
    }

    #[test]
    fn cosh_gate_boundary() {
        let g = CoshCoalescenceGate::default();
        assert!(g.coalesces(1.0));
        assert!(g.coalesces(1.0000001));
        assert!(!g.coalesces(1.0000002));
    }

    #[test]
    fn live_pass_emits_l_minus_one_walks() {
        let mut pass = LiveThreadedPass::new();
        for i in 0..52usize {
            pass.ingest(i, 52, 1000 + i as u64, (i % 12) + 1);
        }
        assert_eq!(pass.walks.len(), 51);
        assert_eq!(pass.steps.len(), 52);
        assert_eq!(pass.bands_covered(), 3);
    }

    #[test]
    fn live_pass_no_walk_on_first_step() {
        let mut pass = LiveThreadedPass::new();
        assert!(!pass.ingest(0, 52, 1, 1));
        assert!(pass.ingest(1, 52, 2, 2));
        assert_eq!(pass.walks.len(), 1);
    }
}

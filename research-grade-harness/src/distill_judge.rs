//! Interim judge gate for distillation packets (AUDIT.md §7 structural gate).
//!
//! The engine (distill_engine.rs) currently carries ZERO `judge` references.
//! Until the judge lands in-engine, this module provides the structural gate that
//! every distillation run must pass before its atoms enter a shared map:
//!
//!   1. `knowledge_atoms_minted == modes_scanned`         (no dropped receipts)
//!   2. `novel_atoms_minted + coalesced_atoms == modes_scanned`
//!   3. sum(role_counts) == knowledge_atoms_minted
//!   4. `0 <= mean_retained_energy <= 1`
//!
//! Teacher identity is deliberately NOT a field anywhere in this module: the gate
//! is content/structure only, per PROTOCOL.md §8.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoleCounts {
    pub counts: [usize; 12],
}

impl RoleCounts {
    pub fn total(&self) -> usize {
        self.counts.iter().sum()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillationRun {
    pub modes_scanned: usize,
    pub knowledge_atoms_minted: usize,
    pub novel_atoms_minted: usize,
    pub coalesced_atoms: usize,
    pub role_counts: RoleCounts,
    pub mean_retained_energy_q32: f64,
    /// CORR-2026-10-29: weight-mode fan-out. One scanned mode (tensor shard)
    /// yields many atoms (modes -> wells -> arcs); `knowledge_atoms_minted ==
    /// modes_scanned` holds ONLY for trajectory distill (one step -> one atom).
    /// Weight-mode runs MUST file `atoms_per_mode` (measured fan-out mean) and
    /// are checked against `expected = modes_scanned * atoms_per_mode`
    /// within `fanout_tolerance` (default 0.0 = exact; filed runs state theirs).
    /// `None` = trajectory mode (strict equality).
    pub atoms_per_mode: Option<f64>,
    pub fanout_tolerance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DistillationJudgeVerdict {
    Pass,
    Fail(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillationJudgeResult {
    pub judge_id: String,
    pub verdict: DistillationJudgeVerdict,
    pub evidence: String,
}

pub struct DistillationJudge {
    pub id: String,
}

impl DistillationJudge {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    pub fn evaluate(&self, run: &DistillationRun) -> DistillationJudgeResult {
        // Gate 1: no dropped receipts.
        // Trajectory mode (atoms_per_mode None): strict 1:1.
        // Weight-mode (Some(f)): minted must equal scanned*f within tolerance.
        let (expected_minted, gate1_evidence) = match run.atoms_per_mode {
            None => (run.modes_scanned as f64, "trajectory 1:1".to_string()),
            Some(f) => (
                run.modes_scanned as f64 * f,
                format!(
                    "weight-mode fanout f={} expected={:.1}",
                    f,
                    run.modes_scanned as f64 * f
                ),
            ),
        };
        if (run.knowledge_atoms_minted as f64 - expected_minted).abs()
            > run.fanout_tolerance.max(0.0)
        {
            return DistillationJudgeResult {
                judge_id: self.id.clone(),
                verdict: DistillationJudgeVerdict::Fail(format!(
                    "atom_count_mismatch: minted={} expected={:.1} scanned={} [{}]",
                    run.knowledge_atoms_minted, expected_minted, run.modes_scanned, gate1_evidence,
                )),
                evidence: "AUDIT §7 gate 1 (CORR-2026-10-29 fan-out)".into(),
            };
        }
        // Gate 2: novelty accounting balances.
        // CORR-2026-10-29: with fan-out, novel+coalesced == MINTED, not
        // scanned (one mode fans out, then each minted atom is novel or
        // coalesced). Trajectory mode (1:1) is the special case minted==scanned.
        if run.novel_atoms_minted + run.coalesced_atoms != run.knowledge_atoms_minted {
            return DistillationJudgeResult {
                judge_id: self.id.clone(),
                verdict: DistillationJudgeVerdict::Fail(format!(
                    "novelty_accounting_mismatch: novel={} coalesced={} scanned={}",
                    run.novel_atoms_minted, run.coalesced_atoms, run.modes_scanned
                )),
                evidence: "Theorem 2".into(),
            };
        }
        // Gate 3: role distribution sums.
        if run.role_counts.total() != run.knowledge_atoms_minted {
            return DistillationJudgeResult {
                judge_id: self.id.clone(),
                verdict: DistillationJudgeVerdict::Fail(format!(
                    "role_sum_mismatch: sum={} minted={}",
                    run.role_counts.total(),
                    run.knowledge_atoms_minted
                )),
                evidence: "AUDIT §7 role-distribution".into(),
            };
        }
        // Gate 4: retained energy in range.
        if !(0.0..=1.0).contains(&run.mean_retained_energy_q32) {
            return DistillationJudgeResult {
                judge_id: self.id.clone(),
                verdict: DistillationJudgeVerdict::Fail(format!(
                    "retained_energy_out_of_range: {}",
                    run.mean_retained_energy_q32
                )),
                evidence: "Cap(T) requires Rbar in [0,1]".into(),
            };
        }
        DistillationJudgeResult {
            judge_id: self.id.clone(),
            verdict: DistillationJudgeVerdict::Pass,
            evidence: format!(
                "pass: minted={} novel={} coalesced={} energy={:.4}",
                run.knowledge_atoms_minted,
                run.novel_atoms_minted,
                run.coalesced_atoms,
                run.mean_retained_energy_q32
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_run() -> DistillationRun {
        let mut counts = [0usize; 12];
        counts[0] = 3;
        counts[5] = 2;
        DistillationRun {
            modes_scanned: 5,
            knowledge_atoms_minted: 5,
            novel_atoms_minted: 4,
            coalesced_atoms: 1,
            role_counts: RoleCounts { counts },
            mean_retained_energy_q32: 0.74,
            atoms_per_mode: None, // trajectory mode: strict 1:1
            fanout_tolerance: 0.0,
        }
    }

    #[test]
    fn pass_on_balanced_run() {
        let j = DistillationJudge::new("distill-interim");
        let r = j.evaluate(&base_run());
        assert_eq!(r.verdict, DistillationJudgeVerdict::Pass);
    }

    #[test]
    fn fail_on_dropped_receipt() {
        let mut run = base_run();
        run.knowledge_atoms_minted = 4;
        let j = DistillationJudge::new("distill-interim");
        let r = j.evaluate(&run);
        assert!(matches!(r.verdict, DistillationJudgeVerdict::Fail(_)));
    }

    #[test]
    fn fail_on_novelty_inflation() {
        let mut run = base_run();
        run.novel_atoms_minted = 5;
        run.coalesced_atoms = 1; // 6 != 5
        let j = DistillationJudge::new("distill-interim");
        let r = j.evaluate(&run);
        assert!(matches!(r.verdict, DistillationJudgeVerdict::Fail(_)));
    }

    #[test]
    fn fail_on_role_sum() {
        let mut run = base_run();
        run.role_counts.counts[0] = 10;
        let j = DistillationJudge::new("distill-interim");
        let r = j.evaluate(&run);
        assert!(matches!(r.verdict, DistillationJudgeVerdict::Fail(_)));
    }

    #[test]
    fn fail_on_energy_range() {
        let mut run = base_run();
        run.mean_retained_energy_q32 = 1.3;
        let j = DistillationJudge::new("distill-interim");
        let r = j.evaluate(&run);
        assert!(matches!(r.verdict, DistillationJudgeVerdict::Fail(_)));
    }

    #[test]
    fn weight_mode_fanout_balances() {
        // CORR-2026-10-29: one scanned mode fans out to many atoms.
        // Filed 2026-10-28 shape: scanned=1183, minted cells=4468
        // (f = 4468/1183 ≈ 3.777), novel+coalesced == minted.
        let mut counts = [0usize; 12];
        counts[0] = 4468;
        let run = DistillationRun {
            modes_scanned: 1183,
            knowledge_atoms_minted: 4468,
            novel_atoms_minted: 4468,
            coalesced_atoms: 0,
            role_counts: RoleCounts { counts },
            mean_retained_energy_q32: 0.74,
            atoms_per_mode: Some(4468.0 / 1183.0),
            fanout_tolerance: 0.5, // rounding of the filed mean
        };
        let j = DistillationJudge::new("distill-interim");
        let r = j.evaluate(&run);
        assert_eq!(r.verdict, DistillationJudgeVerdict::Pass, "{:?}", r);
    }

    #[test]
    fn weight_mode_without_fanout_fails_loudly() {
        // Same numbers, no fan-out filed: must FAIL, not silently pass.
        let mut counts = [0usize; 12];
        counts[0] = 4468;
        let run = DistillationRun {
            modes_scanned: 1183,
            knowledge_atoms_minted: 4468,
            novel_atoms_minted: 4468,
            coalesced_atoms: 0,
            role_counts: RoleCounts { counts },
            mean_retained_energy_q32: 0.74,
            atoms_per_mode: None, // trajectory 1:1 assumed -> mismatch
            fanout_tolerance: 0.0,
        };
        let j = DistillationJudge::new("distill-interim");
        let r = j.evaluate(&run);
        assert!(
            matches!(r.verdict, DistillationJudgeVerdict::Fail(_)),
            "unfiled fan-out must fail, not pass"
        );
    }
}

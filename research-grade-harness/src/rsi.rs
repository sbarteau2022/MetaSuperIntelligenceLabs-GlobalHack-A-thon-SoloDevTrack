use crate::active_inference::PolicyCandidate;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Scope {
    Invariant,
    Domain,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BundleDelta {
    pub scope: Scope,
    pub edits: Vec<(usize, String)>, // (component index, new text)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bundle {
    pub version: u32,
    pub components: Vec<String>, // 8 components
}

impl Bundle {
    pub fn base() -> Self {
        Self {
            version: 0,
            components: vec![
                "system prompt baseline".into(),
                "guardrails guidance".into(),
                "memory baseline".into(),
                "tool guidance baseline".into(),
                "subagents baseline".into(),
                "skills baseline".into(),
                "scripts baseline".into(),
                "plugins baseline".into(),
            ],
        }
    }

    pub fn fingerprint(&self) -> String {
        let mut h = Sha256::new();
        for c in &self.components {
            h.update(c.as_bytes());
        }
        hex::encode(h.finalize())
    }

    pub fn apply(&self, d: &BundleDelta) -> Bundle {
        let mut c = self.components.clone();
        for (i, v) in &d.edits {
            if *i < c.len() {
                c[*i] = v.clone();
            }
        }
        Bundle {
            version: self.version + 1,
            components: c,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LibraryEntry {
    pub sid: String, // stable symbolic id (GenMem addressing)
    pub bundle: Bundle,
    pub lesson: String,
    pub prediction_verified: Option<bool>,
    pub promoted: bool,
}

#[derive(Clone, Debug, Default)]
pub struct GovernedLibrary {
    pub entries: Vec<LibraryEntry>,
    pub next_sid: u32,
}

impl GovernedLibrary {
    pub fn nominate(&mut self, bundle: Bundle, lesson: &str) -> String {
        self.next_sid += 1;
        let sid = format!("sid-{:06}", self.next_sid);
        self.entries.push(LibraryEntry {
            sid: sid.clone(),
            bundle,
            lesson: lesson.into(),
            prediction_verified: None,
            promoted: false,
        });
        sid
    }

    /// promotion gated: requires judge pass + prediction verified + energy not exceeded
    pub fn promote(
        &mut self,
        sid: &str,
        judge_pass: bool,
        prediction_verified: bool,
        energy_ok: bool,
    ) -> bool {
        if let Some(e) = self.entries.iter_mut().find(|e| e.sid == sid) {
            e.prediction_verified = Some(prediction_verified);
            if judge_pass && prediction_verified && energy_ok {
                e.promoted = true;
                return true;
            }
        }
        false
    }
}

/// Outcome of one real probe of a bundle: pass rate over judge-gated tasks.
/// quality is a MEASURED pass rate (0..1), never a word heuristic.
#[derive(Clone, Debug)]
pub struct ProbeResult {
    pub bundle: Bundle,
    pub quality: f64, // measured probe pass rate 0..1
    pub tokens: u64,  // measured model tokens for this probe
    pub energy_j: f64,
    pub passes: usize, // tasks passed
    pub total: usize,  // tasks probed
    pub judge_ids: Vec<String>,
}

/// Per-proposal ledger row for PpcMetrics: what was probed and what it cost.
#[derive(Clone, Debug)]
pub struct ProposalRecord {
    pub fingerprint: String, // Bundle::fingerprint()
    pub version: u32,
    pub passes: usize,
    pub total: usize,
    pub quality: f64,
    pub tokens: u64,
    pub energy_j: f64,
}

#[derive(Clone, Debug, Default)]
pub struct PpcMetrics {
    pub k: usize,
    pub rollouts: usize,
    pub sessions: usize,
    pub adopted_version: u32,
    pub adopted_fingerprint: String,
    pub kept_base: bool,
    pub quality_gain: f64,
    pub energy_spent_j: f64,
    pub abstained: bool,
    /// One row per unique (deduped) proposal actually probed.
    pub proposals: Vec<ProposalRecord>,
    /// Judge ids backing the adoption decision (deterministic judges).
    pub judge_ids: Vec<String>,
    /// (regulator, passed) ledger: leakage, noise_band, security_floor, pareto.
    pub regulators: Vec<(String, bool)>,
    /// true iff every quality number in this run came from probe_fn.
    pub real_probe: bool,
}

pub struct ProposeProbeCompose {
    pub k: usize,
}

impl ProposeProbeCompose {
    pub fn new(k: usize) -> Self {
        Self { k: k.max(1) }
    }

    /// SYNTHETIC FALLBACK ONLY — never an adoption signal.
    /// Labelling (per HARDWARE-GATE-RUNBOOK.md): word-count heuristic kept so
    /// legacy tests compile and the failure mode stays visible; run() ignores
    /// it whenever a probe_fn is supplied. Any quality claim from score() is
    /// scaffolding, not a measurement.
    #[allow(dead_code)]
    pub fn synthetic_fallback_score(bundle: &Bundle) -> f64 {
        Self::score(bundle)
    }

    /// Synthetic quality model: more matching "improve" markers => higher quality.
    /// FALLBACK ONLY (see synthetic_fallback_score). Kept private to new code.
    fn score(bundle: &Bundle) -> f64 {
        let mut s = 0.5;
        for c in &bundle.components {
            if c.contains("improve") || c.contains("verified") {
                s += 0.05;
            }
            if c.contains("optimize") {
                s += 0.03;
            }
        }
        if s > 0.99 { 0.99 } else { s }
    }

    pub fn propose_inv(&self, base: &Bundle) -> BundleDelta {
        BundleDelta {
            scope: Scope::Invariant,
            edits: vec![(0, format!("{} + improve invariant", base.components[0]))],
        }
    }

    pub fn propose_dom(&self, base: &Bundle) -> BundleDelta {
        BundleDelta {
            scope: Scope::Domain,
            edits: vec![(2, format!("{} + optimize domain", base.components[2]))],
        }
    }

    /// Run PPC once with a REAL probe function.
    ///
    /// probe_fn(bundle) -> (passes, total, tokens, judge_ids): quality is
    /// passes/total from deterministic last-number judges over probe tasks.
    ///
    /// Fixed-PPC order (HARDWARE-GATE-RUNBOOK.md):
    /// 1. propose k INV + k DOM; 2. dedup by fingerprint (base pinned first);
    /// 3. probe each unique proposal IFF the budget affords it (no record for
    ///    an unaffordable probe); 4. adopt via leakage_critic + 0.02 noise
    ///    band + regularized_adopt with KEEP_BASE the default.
    /// Returns (adopted_bundle, metrics). Base is retained as KEEP_BASE candidate.
    pub fn run<F>(
        &self,
        base: &Bundle,
        budget: &mut crate::energy::EnergyBudget,
        probe_fn: F,
    ) -> (Bundle, PpcMetrics)
    where
        F: Fn(&Bundle) -> (usize, usize, u64, Vec<String>),
    {
        let k = self.k;
        let base_fp = base.fingerprint();

        // 1. propose k INV + k DOM, base pinned for abstention.
        let mut proposals: Vec<Bundle> = vec![base.clone()];
        for _ in 0..k {
            proposals.push(base.apply(&self.propose_inv(base)));
            proposals.push(base.apply(&self.propose_dom(base)));
        }

        // 2. dedup by fingerprint BEFORE probing (base keeps slot 0).
        let mut seen = std::collections::HashSet::new();
        let mut unique: Vec<Bundle> = vec![];
        for b in proposals {
            let fp = b.fingerprint();
            if seen.insert(fp) {
                unique.push(b);
            }
        }

        // 3. probe each unique proposal; skip (do not record) unaffordable ones.
        let mut probed: Vec<ProbeResult> = vec![];
        for b in &unique {
            let fp = b.fingerprint();
            // Measured-cost estimate for the affordability check: the probe
            // itself is the cost; use the surrogate J/token from the energy
            // ledger (0.0005 J/token) on a 800-token estimate. A proposal that
            // cannot be afforded is NOT probed and NOT recorded.
            let est_energy = 800.0 * 0.0005;
            if !budget.can_afford(est_energy) {
                continue;
            }
            let (passes, total, tokens, judge_ids) = probe_fn(b);
            let quality = if total == 0 {
                0.0
            } else {
                passes as f64 / total as f64
            };
            let energy = tokens as f64 * 0.0005; // surrogate J
            budget.charge(energy, tokens * 4);
            probed.push(ProbeResult {
                bundle: b.clone(),
                quality,
                tokens,
                energy_j: energy,
                passes,
                total,
                judge_ids,
            });
            let _ = fp;
        }

        // Base quality = its measured probe (KEEP_BASE competes on real signal).
        let base_q = probed
            .iter()
            .find(|p| p.bundle.fingerprint() == base_fp)
            .map(|p| p.quality)
            .unwrap_or(0.0);
        let base_tokens = probed
            .iter()
            .find(|p| p.bundle.fingerprint() == base_fp)
            .map(|p| p.tokens)
            .unwrap_or(0);
        let _ = base_tokens;

        // 4. adopt: best measured candidate via all regulators; KEEP_BASE default.
        let mut best: Option<&ProbeResult> = None;
        for p in &probed {
            if p.bundle.fingerprint() == base_fp {
                continue;
            }
            match best {
                None => best = Some(p),
                Some(b) if p.quality > b.quality => best = Some(p),
                _ => {}
            }
        }
        // Security proxy for regularized_adopt: bundle text carries no gate
        // relaxation signal in this port, so candidate_security == base_security
        // (floor binds only on real relaxations); leakage/noise/pareto decide.
        let mut adopted = base.clone();
        let mut kept_base = true;
        let mut regulators: Vec<(String, bool)> = vec![];
        let mut judge_ids: Vec<String> = vec![];
        if let Some(cand) = best {
            let (adopt, reasons) = crate::evolution::regularized_adopt(
                cand.quality,
                base_q,
                &cand.bundle,
                1.0,  // candidate_security: no gate relaxation in prompt text
                1.0,  // base_security
                0.02, // noise band: 2pp (runbook)
            );
            let reason_str = reasons.join("; ");
            regulators.push(("leakage_critic".into(), !reason_str.contains("leakage")));
            regulators.push(("noise_band_0.02".into(), !reason_str.contains("noise band")));
            regulators.push((
                "security_floor".into(),
                !reason_str.contains("security floor"),
            ));
            regulators.push(("pareto_frontier".into(), !reason_str.contains("pareto")));
            judge_ids = cand.judge_ids.clone();
            if adopt {
                adopted = cand.bundle.clone();
                kept_base = false;
            }
        } else {
            // No affordable non-base candidate: regulators trivially pass,
            // KEEP_BASE by default (no evidence to adopt).
            for name in [
                "leakage_critic",
                "noise_band_0.02",
                "security_floor",
                "pareto_frontier",
            ] {
                regulators.push((name.into(), true));
            }
        }
        let abstained = kept_base;

        let proposals: Vec<ProposalRecord> = probed
            .iter()
            .map(|p| ProposalRecord {
                fingerprint: p.bundle.fingerprint(),
                version: p.bundle.version,
                passes: p.passes,
                total: p.total,
                quality: p.quality,
                tokens: p.tokens,
                energy_j: p.energy_j,
            })
            .collect();
        let metrics = PpcMetrics {
            k,
            rollouts: 4 * k, // Harness² accounting
            sessions: 3 * k,
            adopted_version: adopted.version,
            adopted_fingerprint: adopted.fingerprint(),
            kept_base,
            quality_gain: probed
                .iter()
                .find(|p| p.bundle.fingerprint() == adopted.fingerprint())
                .map(|p| p.quality)
                .unwrap_or(base_q)
                - base_q,
            energy_spent_j: probed.iter().map(|p| p.energy_j).sum(),
            abstained,
            proposals,
            judge_ids,
            regulators,
            real_probe: true,
        };

        (adopted, metrics)
    }

    pub fn to_policy_candidate(_b: &Bundle) -> PolicyCandidate {
        PolicyCandidate {
            id: "bundle".into(),
            description: "bundle".into(),
            pragmatics: 0.6,
            epistemic: 0.3,
            landauer_cost: 1.0,
            reversible: true,
        }
    }
}

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

#[derive(Clone, Debug)]
pub struct ProbeResult {
    pub bundle: Bundle,
    pub quality: f64, // synthetic score 0..1
    pub tokens: u64,
    pub energy_j: f64,
}

#[derive(Clone, Debug)]
pub struct PpcMetrics {
    pub k: usize,
    pub rollouts: usize,
    pub sessions: usize,
    pub adopted_version: u32,
    pub kept_base: bool,
    pub quality_gain: f64,
    pub energy_spent_j: f64,
    pub abstained: bool,
}

pub struct ProposeProbeCompose {
    pub k: usize,
}

impl ProposeProbeCompose {
    pub fn new(k: usize) -> Self {
        Self { k: k.max(1) }
    }

    /// Synthetic quality model: more matching "improve" markers => higher quality.
    /// This is PROPOSED scaffolding; replace with real model execution in Phase 1.
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

    /// Run PPC once: propose INV/DOM candidates, probe, compose k finals, select.
    /// Returns (adopted_bundle, metrics). Base is retained as KEEP_BASE candidate.
    pub fn run(
        &self,
        base: &Bundle,
        budget: &mut crate::energy::EnergyBudget,
    ) -> (Bundle, PpcMetrics) {
        let k = self.k;
        let mut proposals: Vec<Bundle> = vec![base.clone()]; // base included for abstention
        let _finals: Vec<Bundle> = vec![];

        // proposal session count: k INV + k DOM
        for _ in 0..k {
            let inv = base.apply(&self.propose_inv(base));
            let dom = base.apply(&self.propose_dom(base));
            proposals.push(inv);
            proposals.push(dom);
        }

        // probe all (synthetic) and charge energy
        let mut probed: Vec<ProbeResult> = vec![];
        for b in &proposals {
            let q = Self::score(b);
            let tokens = 800u64;
            let energy = tokens as f64 * 0.0005; // surrogate J
            if budget.can_afford(energy) {
                budget.charge(energy, tokens * 4);
            }
            probed.push(ProbeResult {
                bundle: b.clone(),
                quality: q,
                tokens,
                energy_j: energy,
            });
        }

        // compose finals: k compositions = top candidate-ish (here just best probed)
        let mut best_q = 0.0;
        let mut best_bundle = base.clone();
        for p in &probed {
            if p.quality > best_q {
                best_q = p.quality;
                best_bundle = p.bundle.clone();
            }
        }
        // KEEP_BASE / abstention: only adopt if best beats base by margin
        let base_q = Self::score(base);
        let kept_base = best_q <= base_q + 0.01;
        let adopted = if kept_base {
            base.clone()
        } else {
            best_bundle.clone()
        };
        let abstained = kept_base;

        let metrics = PpcMetrics {
            k,
            rollouts: 4 * k, // Harness² accounting
            sessions: 3 * k,
            adopted_version: adopted.version,
            kept_base,
            quality_gain: Self::score(&adopted) - base_q,
            energy_spent_j: probed.iter().map(|p| p.energy_j).sum(),
            abstained,
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

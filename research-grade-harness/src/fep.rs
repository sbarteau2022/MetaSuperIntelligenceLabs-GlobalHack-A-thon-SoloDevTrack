//! FEP core: variational free energy as THE bound every decision descends.
//!
//! The Free Energy Principle: a system maintains itself by minimizing an
//! upper bound on surprise. Here that bound is explicit, computed, and
//! threaded from the core (belief updates) to the outer edges (tool calls,
//! energy spend, session lifecycle):
//!
//!   F = E_q[ln q(s) - ln p(o,s)] + β · Dissipation
//!     = Complexity - Accuracy + β · LandauerCost
//!
//! Every module reports its terms into this functional; every decision
//! (probe, adopt, traverse, consolidate) is a descent step. Nothing
//! optimizes outside it — that is what "bound by the FEP" means.

/// Thermodynamic temperature anchor: kB·T·ln2 at 300K in joules/bit.
/// The dissipation term is measured in the same units as energy spend.
pub const LANDAUER_J_PER_BIT: f64 = 2.87e-21;

/// Inverse temperature: how hard the bound pushes against dissipation.
/// β → ∞ = pure accuracy (spend anything); β → 0 = pure frugality.
/// Default 1.0: one nat of surprise trades 1:1 with kBT of dissipation.
pub const DEFAULT_BETA: f64 = 1.0;

/// One decision step's contribution to the bound. Every decision point
/// constructs this; the harness sums them per session.
#[derive(Clone, Debug, Default)]
pub struct FreeEnergyTerms {
    /// Complexity: KL(q||p) — how far beliefs moved from priors (nats).
    pub complexity_nats: f64,
    /// Accuracy: E_q[ln p(o|s)] — how well beliefs predict observations.
    /// Stored NEGATED (as a cost to minimize): higher accuracy = lower term.
    pub inaccuracy_nats: f64,
    /// Dissipation: Landauer joules of erased bits this step.
    pub dissipation_j: f64,
    /// β weight for this step (default DEFAULT_BETA; audit modes raise it).
    pub beta: f64,
}

impl FreeEnergyTerms {
    pub fn new(complexity: f64, inaccuracy: f64, dissipation_j: f64) -> Self {
        Self {
            complexity_nats: complexity.max(0.0),
            inaccuracy_nats: inaccuracy,
            dissipation_j: dissipation_j.max(0.0),
            beta: DEFAULT_BETA,
        }
    }

    /// The bound itself. Every decision descends THIS number.
    pub fn bound(&self) -> f64 {
        self.complexity_nats + self.inaccuracy_nats + self.beta * self.dissipation_j
    }
}

/// Session-level accumulator: the harness threads ONE of these from core
/// (first belief update) to outer edge (last tool result). Minimizing the
/// session bound IS the optimization — not a lane, not a metric, everything.
#[derive(Clone, Debug, Default)]
pub struct SessionBound {
    pub steps: Vec<FreeEnergyTerms>,
}

impl SessionBound {
    pub fn new() -> Self {
        Self { steps: vec![] }
    }

    /// Record one decision step. Returns the new session total.
    pub fn descend(&mut self, terms: FreeEnergyTerms) -> f64 {
        self.steps.push(terms);
        self.total()
    }

    pub fn total(&self) -> f64 {
        self.steps.iter().map(|t| t.bound()).sum()
    }

    /// Per-axis totals for audit: where is the bound spent?
    pub fn audit(&self) -> (f64, f64, f64) {
        (
            self.steps.iter().map(|t| t.complexity_nats).sum(),
            self.steps.iter().map(|t| t.inaccuracy_nats).sum(),
            self.steps.iter().map(|t| t.beta * t.dissipation_j).sum(),
        )
    }

    /// Dynamic iteration trigger: when the LATEST step's bound exceeds the
    /// running mean by more than `sigma` stddevs, the system does NOT have
    /// an answer — it must plan/research before executing (see gate below).
    /// Returns true iff dynamic iteration is REQUIRED.
    pub fn requires_iteration(&self, sigma: f64) -> bool {
        if self.steps.len() < 3 {
            return false;
        }
        let vals: Vec<f64> = self.steps.iter().map(|t| t.bound()).collect();
        let n = vals.len() as f64;
        let mean = vals.iter().sum::<f64>() / n;
        let var = vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
        let std = var.sqrt().max(1e-9);
        let latest = vals[vals.len() - 1];
        latest > mean + sigma * std
    }
}

/// Precision-weighted prediction error update: the core belief step.
/// surprise = -ln p(observation | belief); precision scales the update.
/// High precision + big surprise = the model is confidently wrong → large
/// complexity step (beliefs move far). Low precision = shrug.
pub fn precision_update(
    prior_confidence: f64,
    precision: f64,
    observed_success: bool,
) -> (f64, FreeEnergyTerms) {
    let likelihood = if observed_success {
        prior_confidence.max(1e-9)
    } else {
        (1.0 - prior_confidence).max(1e-9)
    };
    let _surprise = -likelihood.ln();
    // Posterior: precision-weighted move toward the observation.
    // (Surprise magnitude is implicit in the step size below; the
    // inaccuracy term carries the remaining -ln p to the bound.)
    let target = if observed_success { 1.0 } else { 0.0 };
    let step = (precision / (precision + 1.0)) * (target - prior_confidence);
    let posterior = (prior_confidence + step).clamp(0.01, 0.99);
    // Complexity = KL-ish move size; inaccuracy = remaining surprise.
    let complexity = (posterior - prior_confidence).abs() * precision;
    let inaccuracy = -(if observed_success {
        posterior
    } else {
        1.0 - posterior
    })
    .max(1e-9)
    .ln();
    (posterior, FreeEnergyTerms::new(complexity, inaccuracy, 0.0))
}

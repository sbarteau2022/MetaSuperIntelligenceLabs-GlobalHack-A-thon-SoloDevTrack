//! TPPE memory encoding kernel — the positional/address layer, from the
//! ground up on the verified TPPE (`crate::harness::tppe`).
//!
//! DESIGN (one variable changed vs the retired lexical encoder, VI):
//! the lexical content substrate is unchanged (SHA-256 char-3gram,
//! honestly lexical). TPPE becomes the memory kernel's positional
//! address: every memory is encoded at a server-assigned sequence
//! position, and the stored vector is a function of
//! (content, position-state). Same text at a different position is a
//! different vector by construction — the defect that killed both the
//! toroidal attempt (fresh state per call) and the lexical encoder (no
//! position input at all).
//!
//! DERIVATIONS (M1 — written before code, code follows):
//!
//! 1. State (verbatim kernel, `tppe.rs`): per ordinal p (1-based steps
//!    from origin), `state_p = step(state_{p-1}, (p)*0.017, (p)*0.009)`.
//!    Revolutions w = (θf + θs)/TAU are derived inside `step`, never
//!    supplied. Position enters ONLY through this recurrence — text,
//!    token IDs, and wall clocks never drive a step (the three
//!    original blockers).
//! 2. Vector layout (n = 384, all f64 until the final f32 cast):
//!    - Content block C (dims 0..319): the lexical substrate hashed
//!      into 320 dims (mod 320 — specified exactly, not truncation),
//!      L2-normalized.
//!    - Rotation: half-split pair rotation exactly as at the Layer-3
//!      Q/K boundary (qwen36.rs): for pair i in 0..160, angle =
//!      phase_t2()[i % 2], scale = 1 + 0.05·ρ. Applied to the content
//!      block AFTER normalization (rotation preserves norms up to the
//!      radial scale).
//!    - Phase fingerprint F (dims 320..382): six values
//!      [cos θf, sin θf, cos θs, sin θs, cos ψ, sin ψ] where
//!      ψ = θf − 2θs, tiled by REPLICATION (10 full tiles = dims
//!      320..379, plus the first 2 values repeated at 380..381 — the
//!      tiling table is this sentence, exactly), then scaled so
//!      ||F||² = 8.0. Energy share is therefore bounded and named:
//!      content energy ≈ (1+0.05ρ)² ≤ 1.103, fingerprint energy 8.0,
//!      radial channel ≤ 0.0144 — the fingerprint is a minority stamp,
//!      not 97% of the vector.
//!    - Radial channel (dim 383): 0.20·ρ ∈ [0, 0.2). Strictly
//!      increasing in position (golden-verified ρ sequence).
//!    - Whole vector L2-normalized, cast to f32.
//! 3. Distinctness (the claim, scoped honestly): for the same content
//!    vector, positions p < q give different stored vectors because
//!    dim 383 is strictly increasing (ρ_p < ρ_q, golden-verified) and
//!    the fingerprint block differs (θf, θs advance by ≥ 0.127 /
//!    0.039 rad per step at sin ψ = 1 — the true worst case). This is
//!    a DISTINCTNESS claim, tested pairwise over the epoch; it is NOT
//!    a separation/margin claim and NOT a retrieval-quality claim.
//! 4. Epoch: positions 0..=99 (state after steps 1..=100) per epoch.
//!    At 100 the epoch rolls: seq_epoch += 1, fresh origin. Epoch IS
//!    part of the point id AND of the fingerprint (added as a phase
//!    offset of epoch·π/50 into F's first channel pair at encode time
//!    — no: see 5). Instead: epoch enters the vector through the
//!    radial channel offset — NO. Decided here, in writing, before
//!    code: epoch distinguishes points via payload + point id; the
//!    VECTOR does not claim cross-epoch distinctness. Two epochs'
//!    same-text vectors are equal BY DESIGN (position is within-epoch
//!    address). The earlier verifier hole (epoch collision falsifies
//!    distinctness) is closed by scoping the claim to within-epoch
//!    positions, and the point id prevents overwrite.
//! 5. Identity: encoder id `tppe-linear-log-v1+candidate-v2+memory-dual-v1`.
//!    `candidate-v2` names the kernel's source contract; the memory
//!    layout is this module (`+memory384-v2`), a distinct consumer —
//!    stated, not borrowed silently. NO promotion, NO radial winner
//!    (tie), NO quality claim. The bakeoff verdict (OPEN) rides in
//!    every payload as `tppe_verdict: open-candidate`.

use crate::harness::tppe::{RadialLaw, TppeConfig, TppeState};
use sha2::{Digest, Sha256};

/// Pinned candidate config (qwen36.rs `default_tppe_config`,
/// LinearLogV1). The kernel's `validate` checks finiteness only; the
/// pin is enforced HERE by construction + a golden test.
pub fn candidate_config() -> TppeConfig {
    TppeConfig {
        law: RadialLaw::LinearLogV1,
        radial_start: 0.05,
        radial_span: 0.12,
        omega_fast: 0.19,
        omega_slow: 0.07,
        coupling_fast: 0.08,
        coupling_slow: 0.04,
    }
}

/// Positional drives (qwen36.rs `tppe_states_for_positions`):
/// position p (0-based) gets the state after p+1 steps.
pub fn drive_fast(step_1_based: u64) -> f64 {
    step_1_based as f64 * 0.017
}
pub fn drive_slow(step_1_based: u64) -> f64 {
    step_1_based as f64 * 0.009
}

/// Sequential TPPE states for ordinals 0..count (state after pos+1
/// steps each), one carry from origin. Pure function of (config, pos).
pub fn states_for_positions(config: &TppeConfig, count: usize) -> Result<Vec<TppeState>, String> {
    let mut states = Vec::with_capacity(count);
    let mut state = TppeState::origin(*config)?;
    for step in 1..=count as u64 {
        state = state.step(*config, drive_fast(step), drive_slow(step))?;
        states.push(state);
    }
    Ok(states)
}

/// Lexical content substrate hashed into `dim` dims (mod dim, signed
/// SHA-256 char-3gram). Specified exactly; NOT a truncation of the
/// 384-d vector. Honestly lexical — no semantic claim.
pub fn lexical_content_vector(text: &str, dim: usize) -> Vec<f64> {
    let mut v = vec![0.0f64; dim];
    let chars: Vec<char> = text.to_lowercase().chars().collect();
    for gram in chars.windows(3) {
        let digest = Sha256::digest(gram.iter().collect::<String>().as_bytes());
        let idx = u64::from_le_bytes(digest[..8].try_into().unwrap()) as usize % dim;
        v[idx] += if digest[8] & 1 == 0 { 1.0 } else { -1.0 };
    }
    let norm = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm > 0.0 {
        for x in &mut v {
            *x /= norm;
        }
    }
    v
}

pub const VECTOR_DIM: usize = 384;
pub const CONTENT_DIM: usize = 320;
pub const FINGERPRINT_START: usize = 320;
pub const RADIAL_CHANNEL: usize = 383;
pub const FINGERPRINT_ENERGY: f64 = 8.0;
pub const RADIAL_WEIGHT: f64 = 0.20;
pub const RADIAL_SCALE_COEF: f64 = 0.05;
/// Positions per epoch (state after steps 1..=100).
pub const MAX_POSITION: u64 = 99;

/// Encode `text` at sequence position `pos` (0-based, within-epoch)
/// under precomputed states (or a fresh carry when `states` is None).
/// Empty/oversized/non-finite inputs fail closed.
pub fn encode_at_position(
    text: &str,
    pos: u64,
    states: &[TppeState],
) -> Result<(Vec<f32>, TppeState), String> {
    if text.trim().is_empty() {
        return Err("memory text must be nonempty".into());
    }
    if text.len() > 32_768 {
        return Err("memory text exceeds 32768 bytes".into());
    }
    if pos > MAX_POSITION {
        return Err(format!(
            "position {pos} exceeds epoch cap {MAX_POSITION}; roll the epoch"
        ));
    }
    let state = states
        .get(pos as usize)
        .copied()
        .ok_or_else(|| format!("no TPPE state for position {pos}"))?;
    state.validate()?;

    let content = lexical_content_vector(text, CONTENT_DIM);
    if content.iter().all(|x| *x == 0.0) {
        return Err("text produced a zero content vector".into());
    }

    let mut v = vec![0.0f64; VECTOR_DIM];
    // Rotation: half-split pairs over the content block (pairs = 160).
    let t2 = state.phase_t2();
    let scale = 1.0 + RADIAL_SCALE_COEF * state.poincare_radius;
    let pairs = CONTENT_DIM / 2;
    for pair in 0..pairs {
        let angle = t2[pair % 2];
        let (sin_a, cos_a) = angle.sin_cos();
        let x0 = content[pair];
        let x1 = content[pair + pairs];
        v[pair] = scale * (x0 * cos_a - x1 * sin_a);
        v[pair + pairs] = scale * (x1 * cos_a + x0 * sin_a);
    }
    // Fingerprint: [cos tf, sin tf, cos ts, sin ts, cos psi, sin psi]
    // tiled by replication, unit pattern scaled to ||F||^2 = 8.
    let psi = state.theta_fast_unwrapped - 2.0 * state.theta_slow_unwrapped;
    let pattern = [
        t2[0].cos(),
        t2[0].sin(),
        t2[1].cos(),
        t2[1].sin(),
        psi.cos(),
        psi.sin(),
    ];
    let fp_len = RADIAL_CHANNEL - FINGERPRINT_START; // 63
    let mut fp = Vec::with_capacity(fp_len);
    while fp.len() < fp_len {
        for &x in &pattern {
            if fp.len() >= fp_len {
                break;
            }
            fp.push(x);
        }
    }
    let pat_norm = pattern.iter().map(|x| x * x).sum::<f64>().sqrt() * (fp_len as f64 / 6.0).sqrt();
    let fp_scale = (FINGERPRINT_ENERGY.sqrt()) / pat_norm.max(1e-12);
    for (i, x) in fp.iter().enumerate() {
        v[FINGERPRINT_START + i] = x * fp_scale;
    }
    v[RADIAL_CHANNEL] = RADIAL_WEIGHT * state.poincare_radius;

    let norm = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    if !norm.is_finite() || norm == 0.0 {
        return Err("encoded vector has non-finite or zero norm".into());
    }
    let out: Vec<f32> = v.iter().map(|x| (x / norm) as f32).collect();
    if out.iter().any(|x| !x.is_finite()) {
        return Err("encoded vector contains a non-finite component".into());
    }
    Ok((out, state))
}

/// Convenience: encode with a fresh sequential carry up to `pos`.
pub fn encode_text_at(text: &str, pos: u64) -> Result<(Vec<f32>, TppeState), String> {
    let cfg = candidate_config();
    let states = states_for_positions(&cfg, (pos + 1) as usize)?;
    encode_at_position(text, pos, &states)
}

/// f64 cosine (oracle-side helper for tests/benches; Qdrant scores in f32).
pub fn cosine_f64(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let nb = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na * nb)
    }
}

pub fn as_f64(v: &[f32]) -> Vec<f64> {
    v.iter().map(|x| *x as f64).collect()
}

// ===========================================================================
// Dual-vector v2 (the falsifier fix, frozen in STORE-SUBSTRATE-DESIGN.md):
// recall and address are separate jobs and must not share one cosine.
// - `encode_content` -> 320D pure lexical content vector (position-free).
//   Qdrant recall searches THIS vector only.
// - `encode_at_position` (above) -> 384D TPPE position vector (address
//   evidence): rotated content + phase fingerprint + radial channel.
//   Never used for relevance ranking; it proves WHERE a memory sits in
//   its session sequence and carries the TPPE state for audit/replay.
// The v1 single-vector mixture is retained only so the falsifier
// (tests/tppe_retrieval_quality.rs) can keep reproducing the failure it
// documents. New writes use `encode_dual`.
// ===========================================================================

pub const CONTENT_VECTOR_DIM: usize = CONTENT_DIM;

/// Dual encoding of one memory: (content 320D, position 384D, state).
pub fn encode_dual(
    text: &str,
    pos: u64,
    states: &[TppeState],
) -> Result<(Vec<f32>, Vec<f32>, TppeState), String> {
    if text.trim().is_empty() {
        return Err("memory text must be nonempty".into());
    }
    if text.len() > 32_768 {
        return Err("memory text exceeds 32768 bytes".into());
    }
    let content64 = lexical_content_vector(text, CONTENT_VECTOR_DIM);
    if content64.iter().all(|x| *x == 0.0) {
        return Err("text produced a zero content vector".into());
    }
    let content: Vec<f32> = content64.iter().map(|x| *x as f32).collect();
    let (position, state) = encode_at_position(text, pos, states)?;
    Ok((content, position, state))
}

/// Content-only encoding for queries (a query has no position).
pub fn encode_content(text: &str) -> Result<Vec<f32>, String> {
    encode_dual(text, 0, &states_for_positions(&candidate_config(), 1)?).map(|(c, _, _)| c)
}

// ===========================================================================
// O(1)-AMORTIZED SEQUENCE (the hard line: no O(N^2) anywhere).
// The old paths rebuilt the carry from origin on every call — O(pos)
// steps per encode, O(N^2) over a session. `TppeSequence` holds the
// carry ONCE: append/advance is one `step` (O(1), eight f64s), position
// lookup within an epoch is O(1) from the held state. Rehydration from
// persisted (epoch, pos, theta_fast, theta_slow) is O(1): the state is
// reconstructed from its coordinates, not replayed (from_components is
// the kernel's own derivation — validate() proves consistency).
// ===========================================================================

/// Stateful TPPE sequence for one (session, epoch) stream.
#[derive(Debug, Clone)]
pub struct TppeSequence {
    config: TppeConfig,
    state: TppeState,
    next_pos: u64,
    epoch: u64,
}

impl TppeSequence {
    pub fn new(config: TppeConfig) -> Result<Self, String> {
        let state = TppeState::origin(config)?;
        Ok(Self {
            config,
            state,
            next_pos: 0,
            epoch: 0,
        })
    }

    /// Reconstruct from persisted coordinates in O(1) (no replay).
    /// The kernel's `validate` proves the coordinates are consistent
    /// (derived revolutions, tanh, Lorentz sheet) or this fails closed.
    pub fn from_persisted(
        config: TppeConfig,
        epoch: u64,
        next_pos: u64,
        theta_fast_unwrapped: f64,
        theta_slow_unwrapped: f64,
    ) -> Result<Self, String> {
        // Rebuild derived coordinates from the two phases (the only
        // independent quantities); validate() checks every invariant.
        let w = TppeState::derived_revolutions(theta_fast_unwrapped, theta_slow_unwrapped);
        let growth = match config.law {
            RadialLaw::LinearLogV1 => 0.35 * w + 0.65 * w.ln_1p(),
            RadialLaw::LogV1 => w.ln_1p(),
        };
        let u = config.radial_start + config.radial_span * growth;
        let rho = (u / 2.0).tanh();
        let state = TppeState {
            theta_fast_unwrapped,
            theta_slow_unwrapped,
            cumulative_revolutions: w,
            hyperbolic_radius: u,
            poincare_radius: rho,
            boundary_margin: 1.0 - rho,
            hyperboloid_time: u.cosh(),
            hyperboloid_spatial_norm: u.sinh(),
        };
        state.validate()?;
        Ok(Self {
            config,
            state,
            next_pos,
            epoch,
        })
    }

    /// Current (epoch, next_pos, state).
    pub fn cursor(&self) -> (u64, u64, TppeState) {
        (self.epoch, self.next_pos, self.state)
    }

    /// Advance one position in O(1). At the epoch cap the epoch rolls
    /// (fresh origin) — still O(1), recorded by the caller in payload.
    pub fn advance(&mut self) -> Result<TppeState, String> {
        if self.next_pos > MAX_POSITION {
            self.epoch += 1;
            self.next_pos = 0;
            self.state = TppeState::origin(self.config)?;
        }
        let step = self.next_pos + 1; // 1-based, matches tppe_states_for_positions
        self.state = self
            .state
            .step(self.config, drive_fast(step), drive_slow(step))?;
        let used = self.state;
        self.next_pos += 1;
        Ok(used)
    }
}

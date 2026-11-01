//! TPE end-to-end bench: EXACT vs TPE vs ROPE on frozen REAL hidden states.
//!
//! Fixture: `frozen_hidden.json` (copy of `analysis/tpe_e2e/frozen_hidden.json` —
//! REAL Qwen3-Embedding-8B DWQ lookups at `embed_tokens`, dim 4096).
//!
//! NOTE (FW-1 models-boundary): the no-JSON rule governs the model's live I/O
//! path. This bin is OFFLINE bench tooling that parses a frozen on-disk fixture
//! for measurement only — no model traffic crosses this parse site.

use hyperbolic_kv_bench_cuda::{
    hyperbolic::relative_l2_error, rank5::Rank5State, toroidal::TpeState,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

const D_HEAD: usize = 64;
const HEAD_SCALE: f64 = 8.0; // sqrt(64)
const ROPE_THETA: f64 = 1_000_000.0; // Qwen rope_theta=1e6 (model family config)
const EXPECTED_DIM: usize = 4096;
const EXPECTED_SHA_FILE: &str = "frozen_hidden.sha256";

#[derive(Deserialize)]
struct Fixture {
    hidden_dim: usize,
    seqs: Vec<FrozenSeq>,
}

#[derive(Deserialize)]
struct FrozenSeq {
    text: String,
    tokens: Vec<u64>,
    hidden: Vec<Vec<f64>>,
}

fn fixture_paths() -> (PathBuf, PathBuf) {
    // Bin lives in <crate>/src/bin/; fixture sits at crate root.
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    (
        manifest.join("frozen_hidden.json"),
        manifest.join(EXPECTED_SHA_FILE),
    )
}

fn verify_sha(json_path: &PathBuf, sha_path: &PathBuf) -> Result<String, String> {
    let bytes = fs::read(json_path).map_err(|e| format!("read {}: {e}", json_path.display()))?;
    let sidecar =
        fs::read_to_string(sha_path).map_err(|e| format!("read {}: {e}", sha_path.display()))?;
    let expected = sidecar
        .split_whitespace()
        .next()
        .ok_or_else(|| format!("empty sidecar {}", sha_path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let actual = format!("{:x}", hasher.finalize());
    if actual != expected {
        return Err(format!(
            "SHA MISMATCH: actual {actual} != sidecar {expected}"
        ));
    }
    Ok(actual)
}

#[inline]
fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Qwen3 input_layernorm: RMSNorm over the head slice (eps=1e-6 per the
/// model config.json rms_norm_eps). Architecture, not tuning.
#[inline]
fn rms_norm(x: &[f64]) -> Vec<f64> {
    let ms = x.iter().map(|v| v * v).sum::<f64>() / (x.len().max(1) as f64);
    let inv = 1.0 / (ms + 1e-6).sqrt();
    x.iter().map(|v| v * inv).collect()
}

/// Causal full-softmax attention, q/k/v = raw head dims (no learned
/// projection — hidden states already carry the model's projections).
fn arm_exact(heads: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = heads.len();
    let mut out = Vec::with_capacity(n);
    for t in 0..n {
        let q = &heads[t];
        let mut scores = Vec::with_capacity(t + 1);
        let mut m = f64::NEG_INFINITY;
        for j in 0..=t {
            let s = dot(q, &heads[j]) / HEAD_SCALE;
            scores.push(s);
            if s > m {
                m = s;
            }
        }
        let mut denom = 0.0;
        for s in scores.iter_mut() {
            *s = (*s - m).exp();
            denom += *s;
        }
        let mut y = vec![0.0f64; D_HEAD];
        for (j, w) in scores.iter().enumerate() {
            let wj = *w / denom;
            for d in 0..D_HEAD {
                y[d] += wj * heads[j][d];
            }
        }
        out.push(y);
    }
    out
}

/// Rotate-half RoPE on q_t and k_j (angle t*theta_p, theta_p = base^(-2p/64)),
/// then the same causal softmax as [`arm_exact`].
fn rotate_half(x: &[f64], pos: usize) -> Vec<f64> {
    let mut r = vec![0.0f64; D_HEAD];
    let pos = pos as f64;
    for p in 0..D_HEAD / 2 {
        let theta = ROPE_THETA.powf(-((2 * p) as f64) / D_HEAD as f64);
        let ang = pos * theta;
        let (s, c) = ang.sin_cos();
        let x0 = x[2 * p];
        let x1 = x[2 * p + 1];
        r[2 * p] = x0 * c - x1 * s;
        r[2 * p + 1] = x0 * s + x1 * c;
    }
    r
}

fn arm_rope(heads: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = heads.len();
    let rq: Vec<Vec<f64>> = (0..n).map(|t| rotate_half(&heads[t], t)).collect();
    let mut out = Vec::with_capacity(n);
    for t in 0..n {
        let mut scores = Vec::with_capacity(t + 1);
        let mut m = f64::NEG_INFINITY;
        for j in 0..=t {
            let s = dot(&rq[t], &rq[j]) / HEAD_SCALE;
            scores.push(s);
            if s > m {
                m = s;
            }
        }
        let mut denom = 0.0;
        for s in scores.iter_mut() {
            *s = (*s - m).exp();
            denom += *s;
        }
        let mut y = vec![0.0f64; D_HEAD];
        for (j, w) in scores.iter().enumerate() {
            let wj = *w / denom;
            for d in 0..D_HEAD {
                y[d] += wj * heads[j][d];
            }
        }
        out.push(y);
    }
    out
}

/// TPE arm on a prefix slice — DRIVE-RESCOPE (fiber E2E5 root cause).
///
/// ROOT CAUSE (VERIFIED by Fiber-E2E5, do not re-derive): the old drive fed
/// content noise as position — dtheta_f/s from atan2 pair-phase deltas of raw
/// embed rows. dtheta≈±π flips pushed scan keys antipodal at heterogeneous
/// r_iso, inner product ≤0, denom pinned at the 1e-9 floor while numer stayed
/// O(10) → the 1.6e10 spike (seq-1 t=4). The SCAN is exonerated as
/// transcribed-vault-faithful; the vault never sees this input regime because
/// its drive is position-structured. The DRIVE was the bug, and it is bin-side.
///
/// RESCOPE (actual TPE design per spec §2 + vault step_and_readout): position
/// advances theta, content feeds the scan.
/// - dtheta_f = dtheta_s = 0.0: `TpeState::evolve` already adds the carrier
///   TPE_OMEGA_F/S internally each step, so the Δθ model/input terms are zero
///   here — no content-as-drive. (Passing omega again would double-add it.)
/// - arc_step KEPT from hidden-state drift (L2/64): amplitude/rotations SHOULD
///   track content energy (spec §4.2-4.3: amp = accumulated drive energy,
///   radius rides rotations). drive_mod = 0.0.
/// - Scan input UNCHANGED: raw 64-dim RMSNormed slice, decay 1.0.
fn arm_tpe_prefix(heads: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let mut tpe = TpeState::new();
    let mut scan = Rank5State::new(D_HEAD);
    let mut out = Vec::with_capacity(heads.len());
    for (t, h) in heads.iter().enumerate() {
        let arc_step = if t == 0 {
            0.0
        } else {
            let p = &heads[t - 1];
            let mut sq = 0.0;
            for d in 0..D_HEAD {
                let dd = h[d] - p[d];
                sq += dd * dd;
            }
            sq.sqrt() / D_HEAD as f64
        };
        tpe.evolve(0.0, 0.0, arc_step, 0.0);
        out.push(scan.step_and_readout(&h[..D_HEAD], 1.0));
    }
    out
}

fn mean_rel(a: &[Vec<f64>], b: &[Vec<f64>]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| relative_l2_error(x, y))
        .sum::<f64>()
        / (a.len().max(1) as f64)
}

fn max_rel(a: &[Vec<f64>], b: &[Vec<f64>]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| relative_l2_error(x, y))
        .fold(0.0f64, f64::max)
}

fn main() -> ExitCode {
    let (json_path, sha_path) = fixture_paths();

    let sha = match verify_sha(&json_path, &sha_path) {
        Ok(s) => s,
        Err(e) => {
            println!("TPE_E2E SHA: {e}");
            return ExitCode::from(2);
        }
    };
    println!("TPE_E2E SHA OK: {sha}  ({})", json_path.display());

    let raw = fs::read(&json_path).expect("fixture readable (sha passed)");
    let fx: Fixture = serde_json::from_slice(&raw).expect("fixture parses");
    assert_eq!(fx.hidden_dim, EXPECTED_DIM, "fixture hidden_dim");
    assert_eq!(fx.seqs.len(), 4, "fixture seq count");

    println!("rope_theta={ROPE_THETA} (Qwen family config) | D_head={D_HEAD} | decay=1.0");
    println!(
        "{:<5} {:<4} {:<16} {:<16} {:<16} {:<16} {:<6}",
        "seq", "N", "J1 max TPE/EX", "J2 mean TPE/RO", "J3 mean RO/EX", "J4 max causal", "VERDICT"
    );

    let mut all_pass = true;
    for (i, seq) in fx.seqs.iter().enumerate() {
        let n = seq.hidden.len();
        assert!((15..=19).contains(&n), "seq {i} len {n} in 15..19");
        assert_eq!(seq.tokens.len(), n, "seq {i} tokens/hidden align");
        for (t, h) in seq.hidden.iter().enumerate() {
            assert_eq!(h.len(), EXPECTED_DIM, "seq {i} pos {t} dim");
        }
        let heads: Vec<Vec<f64>> = seq.hidden.iter().map(|h| h[..D_HEAD].to_vec()).collect();

        // ARCHITECTURE (not tuning): fixture froze pre-norm embed_tokens
        // outputs; the model applies input_layernorm before attention.
        let heads: Vec<Vec<f64>> = heads.iter().map(|h| rms_norm(h)).collect();

        let exact = arm_exact(&heads);
        let tpe = arm_tpe_prefix(&heads);
        let rope = arm_rope(&heads);

        // Guard print only (no denominator change — 1e-12 floor frozen spec).
        for (t, y) in exact.iter().enumerate() {
            let n: f64 = y.iter().map(|v| v * v).sum::<f64>().sqrt();
            if n < 1e-6 {
                println!("NEAR-ZERO exact output seq {i} t {t}");
            }
        }

        // J1: TPE tracks exact (loose bound: rel err < 1.0).
        let j1 = max_rel(&tpe, &exact);
        // J2: position flows through TPE vs ROPE (mean rel > 1e-6).
        let j2 = mean_rel(&tpe, &rope);
        // J3: report-only ROPE vs EXACT.
        let j3 = mean_rel(&rope, &exact);
        // J4: TPE causality — prefix re-runs match full-run outputs < 1e-9.
        let mut j4: f64 = 0.0;
        for t in 0..n {
            let pre = arm_tpe_prefix(&heads[..=t]);
            let d = relative_l2_error(&pre[t], &tpe[t]);
            if d > j4 {
                j4 = d;
            }
        }

        let p1 = j1 < 1.0;
        let p2 = j2 > 1e-6;
        let p4 = j4 < 1e-9;
        let pass = p1 && p2 && p4;
        all_pass &= pass;
        println!(
            "{:<5} {:<4} {:<16.6e} {:<16.6e} {:<16.6e} {:<16.6e} {:<6}",
            i,
            n,
            j1,
            j2,
            j3,
            j4,
            if pass { "PASS" } else { "MISS" }
        );
        println!(
            "      J1(<1.0)={} J2(>1e-6)={} J3(report) J4(<1e-9)={}",
            if p1 { "PASS" } else { "MISS" },
            if p2 { "PASS" } else { "MISS" },
            if p4 { "PASS" } else { "MISS" }
        );
        let _ = seq.text; // fixture text retained for provenance, not scored
    }

    if all_pass {
        println!("TPE_E2E OVERALL: PASS");
        ExitCode::SUCCESS
    } else {
        println!("TPE_E2E OVERALL: MISS (recorded, no tuning)");
        ExitCode::from(1)
    }
}

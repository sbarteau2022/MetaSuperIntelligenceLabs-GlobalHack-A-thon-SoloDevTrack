//! BASELINE kernel — the regular positional kernel, same parameters.
//!
//! Built FIRST, before any ablation arm runs, so every comparison has
//! an honest regular opponent (runbook rule: no ablation without a
//! same-parameter baseline).
//!
//! Parameters matched to the TPPE memory kernel exactly:
//! - content: identical lexical substrate (`lexical_content_vector`,
//!   320D, normalized) — the content variable is PINNED, not varied.
//! - layout: dual vectors (content 320D recall + position 384D address).
//! - position: REGULAR linear RoPE-family rotation — for pair i,
//!   angle = pos · base^(-2i/256), base = 10000, applied half-split
//!   over the content block (the standard regular positional kernel).
//!   Radial channel: 0.20·sin(pos/100) — bounded, non-expanding (the
//!   honest regular shape: no spiral, no derived revolutions, no
//!   Lorentz sheet). Phase fingerprint: [cos, sin] of the first pair
//!   angle, tiled, same energy budget (8.0) as the TPPE arm so the
//!   layout energy is comparable.
//! - state: O(1) (a position counter). No epoch geometry: positions
//!   0..=99 per epoch for apples-to-apples with the TPPE epoch table.
//!
//! This kernel is REAL (not synthetic): it is the standard regular
//! positional encoding used as the control. It exists to be beaten or
//! to win; either outcome is recorded.

use crate::harness::tppe_memory::{lexical_content_vector, CONTENT_DIM, VECTOR_DIM};

pub const BASE: f64 = 10_000.0;
pub const FINGERPRINT_ENERGY: f64 = 8.0;
pub const RADIAL_WEIGHT: f64 = 0.20;
pub const MAX_POSITION: u64 = 99;

/// Regular per-position pair angle (RoPE-family linear schedule).
pub fn pair_angle(pos: u64, pair: usize) -> f64 {
    (pos as f64) * BASE.powf(-((2 * pair) as f64) / 256.0)
}

/// Encode at position `pos` with the regular kernel:
/// (content 320D, position 384D). Same contract as `encode_dual`.
pub fn encode_baseline(text: &str, pos: u64) -> Result<(Vec<f32>, Vec<f32>), String> {
    if text.trim().is_empty() {
        return Err("memory text must be nonempty".into());
    }
    if text.len() > 32_768 {
        return Err("memory text exceeds 32768 bytes".into());
    }
    if pos > MAX_POSITION {
        return Err(format!("position {pos} exceeds epoch cap {MAX_POSITION}"));
    }
    let content64 = lexical_content_vector(text, CONTENT_DIM);
    if content64.iter().all(|x| *x == 0.0) {
        return Err("text produced a zero content vector".into());
    }
    let content: Vec<f32> = content64.iter().map(|x| *x as f32).collect();

    let mut v = vec![0.0f64; VECTOR_DIM];
    let pairs = CONTENT_DIM / 2;
    for pair in 0..pairs {
        let angle = pair_angle(pos, pair);
        let (sin_a, cos_a) = angle.sin_cos();
        let x0 = content64[pair];
        let x1 = content64[pair + pairs];
        v[pair] = x0 * cos_a - x1 * sin_a;
        v[pair + pairs] = x1 * cos_a + x0 * sin_a;
    }
    // fingerprint: [cos a0, sin a0] tiled to ||F||^2 = 8 (same budget)
    let a0 = pair_angle(pos, 0);
    let pat = [a0.cos(), a0.sin()];
    let fp_len = 384 - 320 - 1;
    let mut fp = Vec::with_capacity(fp_len);
    while fp.len() < fp_len {
        for &x in &pat {
            if fp.len() >= fp_len {
                break;
            }
            fp.push(x);
        }
    }
    let pat_norm = pat.iter().map(|x| x * x).sum::<f64>().sqrt() * (fp_len as f64 / 2.0).sqrt();
    let s = FINGERPRINT_ENERGY.sqrt() / pat_norm.max(1e-12);
    for (i, x) in fp.iter().enumerate() {
        v[320 + i] = x * s;
    }
    v[383] = RADIAL_WEIGHT * ((pos as f64) / 100.0).sin();

    let norm = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    if !norm.is_finite() || norm == 0.0 {
        return Err("baseline vector non-finite or zero".into());
    }
    let out: Vec<f32> = v.iter().map(|x| (x / norm) as f32).collect();
    if out.iter().any(|x| !x.is_finite()) {
        return Err("baseline vector contains non-finite".into());
    }
    Ok((content, out))
}

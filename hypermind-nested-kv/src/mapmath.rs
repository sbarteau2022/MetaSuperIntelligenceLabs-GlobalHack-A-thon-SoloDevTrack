//! Map-owned pure math, formerly in the atomadb crate.
//!
//! These are hash / phase / statistics functions with no storage and no
//! RAG behaviour. They live here so the map does not depend on the RAG
//! crate for its own coordinates.

/// FNV-1a 64-bit hash.
pub fn fnv1a(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// RoPE-family phase rotation: pair i rotates by rank * (pi/2) * (1 - 2i/d).
/// Same-rank inner products are preserved exactly; cross-rank products
/// are phase-suppressed.
pub fn rotate_rank(v: &[f32], rank: f32) -> Vec<f32> {
    let d = v.len();
    let mut out = v.to_vec();
    if rank == 0.0 {
        return out;
    }
    for i in 0..d / 2 {
        let frac = 1.0 - (2 * i) as f32 / d as f32;
        let ang = rank * std::f32::consts::FRAC_PI_2 * frac;
        let (c, s) = (ang.cos(), ang.sin());
        let (x, y) = (v[2 * i], v[2 * i + 1]);
        out[2 * i] = x * c - y * s;
        out[2 * i + 1] = x * s + y * c;
    }
    out
}

/// Participation-ratio proxy from the Gram matrix.
pub fn participation_ratio(vecs: &[Vec<f32>]) -> f32 {
    let n = vecs.len();
    if n == 0 {
        return 0.0;
    }
    let mut trace = 0f32;
    let mut fro = 0f32;
    for i in 0..n {
        for j in 0..n {
            let g: f32 = vecs[i].iter().zip(vecs[j].iter()).map(|(a, b)| a * b).sum();
            if i == j {
                trace += g;
            }
            fro += g * g;
        }
    }
    if fro == 0.0 {
        0.0
    } else {
        trace * trace / fro
    }
}

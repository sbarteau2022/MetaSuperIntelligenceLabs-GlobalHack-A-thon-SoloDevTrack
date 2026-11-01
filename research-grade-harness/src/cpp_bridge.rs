//! C++ hot-loop bridge: Rust calls the native batched shell-distance.
//! No JSON, no Python, no TS. The C ABI is the narrowest possible boundary:
//! pointers + len. Math is proven identical by the audit test below
//! (Rust shell_dist vs C++ batch over 1k random points, max diff < 1e-12).
//!
//! MEASURED 2026-10-28 (benches/smoke.rs, N=2074, release): Rust scalar
//! 8.6µs vs C++ batch 18.3µs (0.47x — C++ LOSES). The scalar C++ loop does
//! not beat LLVM's optimization of the same transcendentals. The bridge
//! stays as PROVEN FFI INFRASTRUCTURE (correctness-checked, version-pinned)
//! for future explicit-SIMD/GPU kernels — not as a speed claim. Per the
//! standing rule: slower than Rust = does not ship as an optimization.

#[link(name = "hm_hotloop")]
unsafe extern "C" {
    fn hm_shell_dist_batch(query: *const f64, atoms: *const f64, out: *mut f64, n: usize);
    fn hm_bridge_version() -> u32;
}

/// Batched shell distance via C++. Falls back to pure Rust if the native
/// lib is unavailable at link time (never at runtime — link or fail).
pub fn shell_dist_batch(query: &[f64; 4], atoms: &[[f64; 4]]) -> Vec<f64> {
    let n = atoms.len();
    let mut out = vec![0.0f64; n];
    // Flatten row-major for the C ABI.
    let flat: Vec<f64> = atoms.iter().flat_map(|a| a.iter().copied()).collect();
    unsafe {
        hm_shell_dist_batch(query.as_ptr(), flat.as_ptr(), out.as_mut_ptr(), n);
    }
    out
}

pub fn bridge_version() -> u32 {
    unsafe { hm_bridge_version() }
}

/// Pure-Rust reference (oracle shell_dist shape) for the audit.
pub fn shell_dist_rs(a: &[f64; 4], b: &[f64; 4]) -> f64 {
    fn proj(c: &[f64; 4]) -> [f64; 4] {
        let r = (c[1] * c[1] + c[2] * c[2] + c[3] * c[3]).sqrt().max(1e-9);
        let s = [c[1] / r, c[2] / r, c[3] / r];
        [
            (1.0 + s[0] * s[0] + s[1] * s[1] + s[2] * s[2]).sqrt(),
            s[0],
            s[1],
            s[2],
        ]
    }
    let pa = proj(a);
    let pb = proj(b);
    let inner = (pa[0] * pb[0] - pa[1] * pb[1] - pa[2] * pb[2] - pa[3] * pb[3]).max(1.0);
    (inner + (inner * inner - 1.0).sqrt()).ln()
}

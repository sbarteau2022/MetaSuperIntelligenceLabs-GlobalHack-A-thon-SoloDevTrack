//! Full smoke + bench: hot-loop shootout (Rust vs C++ batch) + FEP descent
//! rate. No Python, no JSON, no TS. Numbers print to stdout; the grant
//! pack cites this file's output, not a spreadsheet.
use rgah::{FreeEnergyTerms, SessionBound, shell_dist_batch, shell_dist_rs};
use std::time::Instant;

fn main() {
    // 1. Hot loop: 2,074 atoms (live map size) x shell_dist.
    let n = 2074usize;
    let q = [1.5, 0.8, 0.4, 0.2];
    let atoms: Vec<[f64; 4]> = (0..n)
        .map(|i| {
            let r = 0.3 + (i as f64) * 0.001;
            let s = [0.6 * r, 0.5 * r, 0.62 * r];
            [
                (1.0 + s[0] * s[0] + s[1] * s[1] + s[2] * s[2]).sqrt(),
                s[0],
                s[1],
                s[2],
            ]
        })
        .collect();
    // Rust scalar.
    let t0 = Instant::now();
    let mut acc = 0.0;
    for _ in 0..20 {
        for a in &atoms {
            acc += shell_dist_rs(&q, a);
        }
    }
    let rs_us = t0.elapsed().as_micros() as f64 / 20.0;
    // C++ batch.
    let t0 = Instant::now();
    for _ in 0..20 {
        let out = shell_dist_batch(&q, &atoms);
        acc += out[0];
    }
    let cpp_us = t0.elapsed().as_micros() as f64 / 20.0;
    println!(
        "hotloop N={} rust_scalar_us={:.1} cpp_batch_us={:.1} speedup={:.2}x checksum={:.3}",
        n,
        rs_us,
        cpp_us,
        rs_us / cpp_us.max(0.1),
        acc
    );

    // 2. FEP descent rate: 1k bound steps.
    let t0 = Instant::now();
    let mut sb = SessionBound::new();
    for i in 0..1000 {
        sb.descend(FreeEnergyTerms::new(
            0.5 / (1.0 + i as f64 * 0.01),
            0.4 / (1.0 + i as f64 * 0.01),
            1e-21,
        ));
    }
    let fep_us = t0.elapsed().as_micros() as f64;
    println!(
        "fep 1000 steps us={:.1} final_bound={:.4}",
        fep_us,
        sb.total()
    );

    // 3. Suite counts (the harness proves itself here; cargo test is the gate).
    println!("smoke done: hotloop + fep measured, see cargo test for gates");
}

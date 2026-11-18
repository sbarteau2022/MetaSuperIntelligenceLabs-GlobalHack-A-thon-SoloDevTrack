//! Bench: O(n^2) all-pairs vs O(n) mean-field Kuramoto step + fused evolve.
//! Run: cargo bench --manifest-path core_lm/Cargo.toml --bench kuramoto_opt
use core_lm::geometry::torus::{evolve_fused, TorPhaseEncoder, PhaseState};
use core_lm::geometry::fixed::Q32_31;
use core_lm::lock::kuramoto_step_cpu;
use core_lm::geometry::torus::kuramoto_resultant_step;
use std::time::Instant;

fn det(n: usize, seed: u64) -> Vec<i64> {
    let mut x = seed;
    (0..n).map(|_| { x = x.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(0xBF58476D1CE4E5B9); ((x >> 33) as i32) as i64 }).collect()
}

fn main() {
    println!("n, allpairs_us, meanfield_us, speedup, fused_us_per_16, seq_us_per_16");
    for &n in &[16usize, 64, 256, 1024, 2560] {
        let omega = det(n, 0x5678);
        let dt = Q32_31::from_f64(0.05).0;
        let k = Q32_31::from_f64(0.3).0;
        let mut a = det(n, 0x1234);
        let t = Instant::now(); for _ in 0..20 { let mut x = a.clone(); kuramoto_step_cpu(&mut x, &omega, dt, k); std::hint::black_box(&x); }
        let allpairs = t.elapsed().as_micros() as f64 / 20.0;
        let mut b = det(n, 0x1234);
        let t = Instant::now(); for _ in 0..20 { let mut x = b.clone(); kuramoto_resultant_step(&mut x, &omega, dt, k); std::hint::black_box(&x); }
        let meanfield = t.elapsed().as_micros() as f64 / 20.0;
        b = det(n, 0x1234); let _ = &b;
        // fused vs sequential evolve (16 drives)
        let enc = TorPhaseEncoder::new();
        let mut drives = Vec::new(); let mut x = 0x9E3779B9u64;
        for _ in 0..16 { x = x.wrapping_mul(0xBF58476D1CE4E5B9).wrapping_add(0x94D049BB133111EB);
            drives.push((Q32_31::from_f64(((x >> 33) as f64) / 1e10), Q32_31::from_f64((((x >> 17) & 0xFFFF) as f64) / 1e10))); }
        let t = Instant::now(); for _ in 0..200 { let mut s = PhaseState::ZERO; for &(df, ds) in &drives { s = enc.evolve(s, df, ds); } std::hint::black_box(s); }
        let seq = t.elapsed().as_micros() as f64 / 200.0;
        let t = Instant::now(); for _ in 0..200 { let s = evolve_fused(&enc, PhaseState::ZERO, &drives); std::hint::black_box(s); }
        let fused = t.elapsed().as_micros() as f64 / 200.0;
        println!("{},{:.1},{:.1},{:.1}x,{:.2},{:.2}", n, allpairs, meanfield, allpairs / meanfield.max(0.05), fused, seq);
    }
}

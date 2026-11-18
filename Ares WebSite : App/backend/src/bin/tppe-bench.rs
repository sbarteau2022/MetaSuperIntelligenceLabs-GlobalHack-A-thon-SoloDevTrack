//! TPPE memory-kernel throughput bench (Rust only; no Python, no JSON).
//! Compares, on one machine in one process:
//!  - incumbent lexical encoder (qdrant_memory::embed_text, 384D)
//!  - TPPE dual encode at a fixed position (states precomputed)
//!  - TPPE encode_text_at (includes the per-call state carry — the
//!    current append path's cost shape)
//!  - raw TPPE state carry (states_for_positions) — the O(1)-state core
use std::hint::black_box;
use std::time::Instant;

fn corpus(chars: usize) -> String {
    let unit = "hyperbolic memory kernel toroidal phase position session summary telemetry ";
    let mut s = String::with_capacity(chars + unit.len());
    while s.len() < chars {
        s.push_str(unit);
    }
    s.truncate(chars);
    s
}

fn bench(label: &str, iters: usize, mut f: impl FnMut() -> usize) -> (f64, f64) {
    // warmup
    for _ in 0..(iters / 10).max(1) {
        black_box(f());
    }
    let t = Instant::now();
    let mut acc = 0usize;
    for _ in 0..iters {
        acc += black_box(f());
    }
    let el = t.elapsed().as_secs_f64();
    let per = el / iters as f64;
    println!(
        "{label:<44} iters={iters:<7} total={:.3}s  {:>10.1} ops/s  {:>10.1} ns/op  (acc {acc})",
        el,
        1.0 / per,
        per * 1e9
    );
    (1.0 / per, per * 1e9)
}

fn main() {
    use ares_backend::harness::qdrant_memory::embed_text;
    use ares_backend::harness::tppe_memory::{
        candidate_config, encode_dual, encode_text_at, states_for_positions,
    };

    let cfg = candidate_config();
    let states = states_for_positions(&cfg, 100).expect("states");

    for &chars in &[128usize, 1024, 8192, 32768] {
        let text = corpus(chars);
        let iters = match chars {
            128 => 20_000,
            1024 => 5_000,
            8192 => 600,
            _ => 80,
        };
        println!(
            "--- corpus {chars} chars (~{} words)",
            text.split_whitespace().count()
        );
        bench("lexical incumbent embed_text (384D)", iters, || {
            embed_text(&text).iter().map(|x| x.to_bits() as usize).sum()
        });
        bench("TPPE encode_dual pos=50 (precomputed)", iters, || {
            let (c, p, _) = encode_dual(&text, 50, &states).unwrap();
            c.len() + p.iter().map(|x| x.to_bits() as usize).sum::<usize>() % 97
        });
        bench("TPPE encode_text_at pos=50 (fresh carry)", iters, || {
            let (v, _) = encode_text_at(&text, 50).unwrap();
            v.len() + (v[383].to_bits() as usize % 89)
        });
    }

    // The admitted positional domain is one epoch (100 steps). Beyond
    // it the kernel fails closed by design (rho rounds to the boundary
    // in f64); the bench stays inside the admitted domain and says so.
    println!("--- raw TPPE state carry (epoch domain)");
    for &n in &[100usize][..] {
        let iters = 2_000;
        bench(&format!("states_for_positions({n})"), iters, || {
            let s = states_for_positions(&cfg, n).unwrap();
            (s[n - 1].poincare_radius * 1e9) as usize
        });
    }
}

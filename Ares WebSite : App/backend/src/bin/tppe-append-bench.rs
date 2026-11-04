//! Append-scaling bench: proves O(1) amortized append under the hard line.
//! Old shape: per-append full carry O(pos) => cost grows linearly in pos.
//! New shape: TppeSequence::advance O(1) => cost flat in pos.
//! Lexical content (O(chars)) is identical in both and factored out by
//! timing the sequence advance alone at several anchor positions.
use std::hint::black_box;
use std::time::Instant;

fn main() {
    use ares_backend::harness::tppe_memory::{
        candidate_config, states_for_positions, TppeSequence,
    };

    let cfg = candidate_config();

    // --- old shape: full carry per append (cost at anchor pos) ---
    for &anchor in &[0usize, 25, 50, 99] {
        let iters = 2_000usize;
        for _ in 0..100 {
            black_box(states_for_positions(&cfg, anchor + 1).unwrap().len());
        }
        let t = Instant::now();
        let mut acc = 0usize;
        for _ in 0..iters {
            acc += black_box(
                states_for_positions(&cfg, anchor + 1).unwrap()[anchor]
                    .poincare_radius
                    .to_bits() as usize
                    % 7,
            );
        }
        let ns = t.elapsed().as_secs_f64() * 1e9 / iters as f64;
        println!("OLD full-carry encode at pos={anchor:<3} {ns:>10.1} ns/op (acc {acc})");
    }

    // --- new shape: O(1) advance (flat by construction) ---
    let mut seq = TppeSequence::new(cfg).unwrap();
    let mut anchors: Vec<(u64, f64)> = Vec::new();
    let mut pos = 0u64;
    loop {
        if [0u64, 25, 50, 99].contains(&pos) {
            let iters = 20_000usize;
            let mut probe = seq.clone();
            for _ in 0..1_000 {
                black_box(probe.advance().unwrap().poincare_radius.to_bits());
            }
            let t = Instant::now();
            let mut acc = 0usize;
            for _ in 0..iters {
                acc += black_box(probe.advance().unwrap().poincare_radius.to_bits() as usize % 7);
                probe = seq.clone();
            }
            let ns = t.elapsed().as_secs_f64() * 1e9 / iters as f64;
            anchors.push((pos, ns));
            println!("NEW O(1) advance at pos={pos:<3}     {ns:>10.1} ns/op (acc {acc})");
        }
        if pos >= 99 {
            break;
        }
        seq.advance().unwrap();
        pos += 1;
    }

    // --- session totals: 100 sequential appends, old vs new ---
    let iters = 200usize;
    let t = Instant::now();
    for _ in 0..iters {
        for p in 0..100usize {
            black_box(states_for_positions(&cfg, p + 1).unwrap().len());
        }
    }
    let old_total = t.elapsed().as_secs_f64() / iters as f64;
    let t = Instant::now();
    for _ in 0..iters {
        let mut s = TppeSequence::new(cfg).unwrap();
        for _ in 0..100 {
            black_box(s.advance().unwrap().poincare_radius.to_bits());
        }
    }
    let new_total = t.elapsed().as_secs_f64() / iters as f64;
    println!(
        "SESSION 100 appends: OLD {:.1} us  NEW {:.1} us  speedup {:.2}x",
        old_total * 1e6,
        new_total * 1e6,
        old_total / new_total
    );
}

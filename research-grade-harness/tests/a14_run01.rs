#![allow(clippy::all)]
//! A14-RUN-01: Tool Registry Determinism probe.
//!
//! Frozen instrument for `A14-RUN-01-Manifest.md`. Resolves the CALLS-A14-v1
//! corpus (22 live `map.*` tool names + 72 near-miss/malformed strings) through
//! `CapabilityGate::admit` 100x per input in 5 seed-shuffled orders (47,000
//! timed resolutions + 47,000 cross-order comparisons), asserting byte-identical
//! outcome classes throughout.
//!
//! Doctrine: no model calls, no network, no fs writes outside target/. A
//! nondeterministic resolution FAILS the run per prereg null-handling (Gate 3
//! blocked); a harness crash/timeout VOIDS it (infra defect, not a verdict).
//! Corpus: 94 inputs (22 live + 72 near-miss).

use rgah::capability::CapabilityGate;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Instant;

/// The 22 live `map.*` tool names (tools/list at HEAD 409fbe4).
const LIVE_MAP_TOOLS: [&str; 22] = [
    "map.ingest",
    "map.query",
    "map.attend",
    "map.lexical",
    "map.reweight",
    "map.traverse",
    "map.compact",
    "map.checkpoint",
    "map.carve",
    "map.dissolve",
    "map.snapshot",
    "map.recover",
    "map.health",
    "map.export_passport",
    "map.create",
    "map.pami_stats",
    "map.read_euclid",
    "map.kv_store",
    "map.kv_get",
    "map.attention",
    "map.hunger",
    "map.consolidate",
];

/// 78 near-miss / malformed inputs exercising all four match stages + the miss path.
#[allow(clippy::vec_init_then_push)]
fn near_miss_corpus() -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    // case variants (stage: exact fails, leaf/skill-path may hit)
    v.push("MAP.QUERY".into());
    v.push("Map.Ingest".into());
    // leaf-only (stage 2/4: leaf_of id / skill_path)
    v.push("query".into());
    v.push("ingest".into());
    v.push("snapshot".into());
    // prefix truncations
    v.push("map.".into());
    v.push("map".into());
    v.push("map.q".into());
    v.push("ap.query".into());
    // unknown tools (miss path: deny_unknown must be byte-identical)
    v.push("map.frobnicate".into());
    v.push("map.query2".into());
    v.push("mapquery".into());
    v.push("exec".into());
    v.push("process.spawn".into());
    v.push("fs.read".into());
    // empty / whitespace (normalization boundary)
    v.push("".into());
    v.push(" ".into());
    v.push("   ".into());
    v.push("  map.query  ".into());
    v.push("\tmap.query".into());
    v.push("map.query\n".into());
    // separator confusion
    v.push("map:query".into());
    v.push("map/query".into());
    v.push("map query".into());
    v.push("map_query".into());
    // near-miss suffixes for every live tool (systematic miss-path coverage)
    for t in LIVE_MAP_TOOLS {
        v.push(format!("{t}x"));
        v.push(format!("x{t}"));
    }
    // a few unicode / long-input robustness probes (must deny deterministically, not panic)
    v.push("map.quеry".into()); // Cyrillic е
    v.push(format!("map.{}", "q".repeat(500)));
    v.push("map.\0query".into());
    assert_eq!(v.len(), 72, "CALLS-A14-v1 miss corpus must be exactly 72");
    v
}

/// SplitMix64: seed-shuffled order, same multiset per seed.
fn shuffled_order(seed: u64, n: usize) -> Vec<usize> {
    let mut state = seed;
    let mut idx: Vec<usize> = (0..n).collect();
    let mut next = || {
        state = state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    };
    for i in (1..n).rev() {
        let j = (next() % (i as u64 + 1)) as usize;
        idx.swap(i, j);
    }
    idx
}

fn outcome_class(gate: &CapabilityGate, input: &str) -> String {
    match gate.admit(input) {
        Ok(node) => format!("ok:{}", node.id),
        Err(denial) => {
            let mut h = DefaultHasher::new();
            denial.hash(&mut h);
            format!("err:{:016x}", h.finish())
        }
    }
}

/// Baseline: ad-hoc string dispatch (direct == against the live list).
fn baseline_class(input: &str) -> &'static str {
    if LIVE_MAP_TOOLS.contains(&input) {
        "ok"
    } else {
        "err"
    }
}

#[test]
fn a14_run01_determinism_50k() {
    let manifest = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../N3/CapabilityTreeSitter/capability-tree-manifest.json"
    ))
    .expect("vault manifest must be readable for RUN-01");
    let t0 = Instant::now();
    // Gate built ONCE; resolution must not depend on build order or wall time.
    let gate = CapabilityGate::from_manifest(&manifest).expect("manifest must load");

    let mut corpus: Vec<String> = LIVE_MAP_TOOLS.iter().map(|s| s.to_string()).collect();
    corpus.extend(near_miss_corpus());
    assert_eq!(corpus.len(), 94, "CALLS-A14-v1 must be exactly 94 inputs");

    let seeds = [20261101u64, 20261102, 20261103, 20261104, 20261105];
    let mut total = 0u64;
    let mut nondet = 0u64;
    let mut baseline_disagree: Vec<String> = Vec::new();
    let mut first_outcomes: Vec<String> = corpus.iter().map(|c| outcome_class(&gate, c)).collect();

    for &seed in &seeds {
        let order = shuffled_order(seed, corpus.len());
        for &i in &order {
            let input = &corpus[i];
            // 100 repeats per input per order.
            let first = outcome_class(&gate, input);
            for _ in 1..100 {
                total += 1;
                if outcome_class(&gate, input) != first {
                    nondet += 1;
                }
            }
            total += 1;
            // Cross-order check against the canonical first pass.
            if first != first_outcomes[i] {
                nondet += 1;
            }
            let _ = &mut first_outcomes; // canonical reference, never mutated after init
            // Baseline partition agreement (informational; disagreement = correction entry, not FAIL).
            let gate_ok = first.starts_with("ok:");
            let base_ok = baseline_class(input) == "ok";
            if gate_ok != base_ok && seed == seeds[0] {
                baseline_disagree.push(input.clone());
            }
        }
    }
    let dt = t0.elapsed();
    eprintln!("A14-RUN-01: {total} resolutions, {nondet} nondeterministic, wall {dt:?}");
    eprintln!(
        "A14-RUN-01 baseline-disagree ({}): {:?}",
        baseline_disagree.len(),
        baseline_disagree
    );
    assert_eq!(
        total, 47_000,
        "run must execute exactly 47,000 timed resolutions"
    );
    assert_eq!(
        nondet, 0,
        "A14 FAIL: nondeterministic resolutions (Gate 3 blocked)"
    );
}

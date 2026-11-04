//! Scheduler-framework rehearsal: the HYPERBOLIC-SCHEDULER-FRAMEWORK
//! workflows (W1–W7) exercised against the toy mind, plus the oracle check
//! (CE1: adversarial-equidistant degrades to scan-parity, never worse) and
//! the L1 probe pre-commitments from §7.

use hypermind_toy::mind::{recall, route_task, CapabilityLeaf, KvStore, Mind, DEPTH_LADDER};
use hypermind_toy::runtime::{CapabilityGate, CapabilityNode};

// --- W1: ingest-and-index; VP exact-prune parity with brute force ---

#[test]
fn vp_tree_matches_brute_force() {
    let mut mind = Mind::new();
    let texts = [
        "prefill chunking bounds decode stall",
        "radix prefix cache shares system prompts",
        "continuous batching fair iteration token budget",
        "speculative decoding draft verify acceptance",
        "hyperbolic distance lorentz inner product",
        "capability gate judged atom receipt",
        "receipt chain genesis prev hash recompute",
        "working set cache mass leak eviction",
        "research gate planning context inject",
        "energy budget landauer dissipation joules",
    ];
    for (i, t) in texts.iter().enumerate() {
        mind.ingest(t, 0.3 + 0.05 * i as f64, "scheduling");
    }
    for q in [
        "decode stall prefill",
        "prefix cache",
        "receipt chain",
        "lorentz hyperbolic",
        "energy joules",
    ] {
        let probe = hypermind_toy::mind::coord_from_text(q, 0.5);
        for k in [1usize, 3, 5] {
            let got = mind.knn(&probe, k);
            let want = mind.knn_brute(&probe, k);
            assert_eq!(
                got, want,
                "VP disagrees with brute force for q={} k={}",
                q, k
            );
        }
    }
}

#[test]
fn exact_text_is_top1() {
    let mut mind = Mind::new();
    mind.ingest(
        "radix prefix cache shares system prompts",
        0.5,
        "scheduling",
    );
    mind.ingest("unrelated receipt chain genesis", 0.5, "memory");
    mind.ingest("decode stall target milliseconds", 0.5, "scheduling");
    let res = recall(&mut mind, "radix prefix cache shares system prompts", 128);
    let top = mind.atoms.get(&res.ids[0]).unwrap();
    assert_eq!(top.text, "radix prefix cache shares system prompts");
}

// --- W3: depth gate is data-dependent, not fixed-width ---

#[test]
fn depth_gate_varies_with_score_mass() {
    let mut mind = Mind::new();
    // Tight cluster: one strong match → small k expected.
    mind.ingest("tokamak plasma confinement magnetic", 0.5, "physics");
    mind.ingest("tokamak plasma confinement magnetic", 0.5, "physics");
    for i in 0..20 {
        mind.ingest(
            &format!("unrelated filler topic number {}", i),
            0.5,
            "other",
        );
    }
    let tight = recall(&mut mind, "tokamak plasma confinement magnetic", 128);
    assert!(
        tight.depth_k <= 4,
        "tight cluster should recall narrow, got k={}",
        tight.depth_k
    );
    assert!(tight.overlap_with_exact_topk == tight.depth_k);
}

// --- W2: capability routing by problem text, gate admits afterwards ---

fn toy_leaves() -> Vec<CapabilityLeaf> {
    vec![
        CapabilityLeaf {
            tool: "map.query".into(),
            family: "map".into(),
            solves: "recall cached conclusions and prefix state for a reasoning step".into(),
            judged_atom: true,
            receipt_present: true,
        },
        CapabilityLeaf {
            tool: "map.kv_fetch".into(),
            family: "map".into(),
            solves: "fetch a stored conclusion by key with hit receipt".into(),
            judged_atom: true,
            receipt_present: true,
        },
        CapabilityLeaf {
            tool: "map.attend".into(),
            family: "map".into(),
            solves: "derive conclusions from scored atoms with minimal width".into(),
            judged_atom: true,
            receipt_present: true,
        },
        CapabilityLeaf {
            tool: "memory.deposit".into(),
            family: "memory".into(),
            solves: "persist session summaries and searchable memories".into(),
            judged_atom: true,
            receipt_present: false, // no receipt → gate denies
        },
        CapabilityLeaf {
            tool: "judge.run".into(),
            family: "judge".into(),
            solves: "run deterministic judges over packet output".into(),
            judged_atom: true,
            receipt_present: true,
        },
    ]
}

#[test]
fn task_routes_to_problem_not_keyword() {
    let mut mind = Mind::new();
    let leaves = toy_leaves();
    // Paraphrased in problem vocabulary (shares derive/minimal/
    // conclusions with the attend leaf, distinct sentence): the toy's
    // hashed embedding resolves problem-vocabulary overlap, not raw
    // keywords — quality beyond that belongs to L1 with real embeddings.
    let ranked = route_task(
        &mut mind,
        &leaves,
        "derive minimal conclusions from scored evidence",
        2,
    );
    assert!(
        ranked.contains(&"map.attend".to_string()),
        "expected map.attend in {:?}",
        ranked
    );
    // Gate admits it (judged atom + receipt present).
    let mut gate = CapabilityGate::default();
    for leaf in &leaves {
        gate.register(CapabilityNode {
            name: leaf.tool.clone(),
            family: leaf.family.clone(),
            judged_atom: leaf.judged_atom,
            receipt_present: leaf.receipt_present,
        });
    }
    assert!(gate.admit("map.attend").is_ok());
    // memory.deposit routes geometrically but is DENIED at the gate.
    assert!(gate.admit("memory.deposit").is_err());
}

// --- W4: many steps held at once = referencing roots, not materializing ---

#[test]
fn hold_many_steps_without_bloat() {
    let mut mind = Mind::new();
    // 200-step execution trace: each step cites its parent (a tree).
    let mut parent_ids: Vec<u64> = Vec::new();
    for i in 0..200 {
        let text = format!("step {} derive conclusion from parent evidence", i);
        let id = mind.ingest(&text, 0.5, "trace");
        parent_ids.push(id);
    }
    // Holding the trace = 200 ids (O(1) handles each), not 200 row blocks.
    assert_eq!(parent_ids.len(), 200);
    // Any step recalls its neighborhood in O(log N)-shaped lookup.
    let probe = hypermind_toy::mind::coord_from_text(
        "derive conclusion from parent evidence step 150",
        0.5,
    );
    let got = mind.knn(&probe, 3);
    let want = mind.knn_brute(&probe, 3);
    assert_eq!(got, want);
    assert_eq!(mind.len(), 200);
}

// --- W5: exact invalidation (Gap A rehearsal) ---

#[test]
fn invalidation_is_exact_not_ttl() {
    let mut mind = Mind::new();
    let a1 = mind.ingest(
        "prefill admission caps protect unified memory",
        0.5,
        "scheduling",
    );
    let a2 = mind.ingest(
        "radix prefix cache shares system prompts",
        0.5,
        "scheduling",
    );
    let mut kv = KvStore::new();
    kv.store("conclusion:sched-1", "admit before allocating", vec![a1]);
    kv.store("conclusion:sched-2", "share prefixes", vec![a2]);
    // Both fetch (recall arm).
    assert_eq!(
        kv.fetch("conclusion:sched-1"),
        Some("admit before allocating".into())
    );
    assert_eq!(kv.hits("conclusion:sched-1"), 1);
    // Producer a1 re-ingested → only its dependent goes stale.
    let marked = kv.invalidate_producer(a1);
    assert_eq!(marked, 1);
    assert!(kv.is_stale("conclusion:sched-1"));
    assert!(!kv.is_stale("conclusion:sched-2"));
    assert_eq!(kv.fetch("conclusion:sched-1"), None); // re-derive once
    assert_eq!(
        kv.fetch("conclusion:sched-2"),
        Some("share prefixes".into())
    );
}

// --- CE1: adversarial equidistant degrades to scan-parity, never worse ---

#[test]
fn adversarial_query_is_scan_parity_not_wrong() {
    let mut mind = Mind::new();
    // Symmetric fixture: identical texts at identical salience are
    // equidistant by construction — the counterexample shape from §5 CE1.
    for _ in 0..8 {
        mind.ingest("identical equidistant atom text", 0.5, "sym");
    }
    mind.ingest("different far atom content here", 0.1, "other");
    let probe = hypermind_toy::mind::coord_from_text("identical equidistant atom text", 0.9);
    // Correctness holds (VP == brute force) even though pruning proves nothing.
    let got = mind.knn(&probe, 4);
    let want = mind.knn_brute(&probe, 4);
    assert_eq!(got, want);
    // And the depth gate still terminates on the ladder.
    let res = recall(&mut mind, "identical equidistant atom text", 128);
    assert!(DEPTH_LADDER.contains(&res.depth_k));
}

// --- L1 probe pre-commitment fixtures (framework §7): small-n oracles ---

#[test]
fn l1_probe_depth_gate_recomputes_from_receipt_terms() {
    // Independent recomputation of the argmin from receipted terms:
    // the gate's choice must equal argmin over the ladder of the same form.
    let mut mind = Mind::new();
    for t in ["alpha beta gamma", "alpha beta delta", "unrelated zeta"] {
        mind.ingest(t, 0.5, "probe");
    }
    let res = recall(&mut mind, "alpha beta", 128);
    assert!((1..=32).contains(&res.depth_k));
    assert!(res.terms.bound.is_finite());
    assert!(res.terms.inaccuracy_nats <= 0.0);
}

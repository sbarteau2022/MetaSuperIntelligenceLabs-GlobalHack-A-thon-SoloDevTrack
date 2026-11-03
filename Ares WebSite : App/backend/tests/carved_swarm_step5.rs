//! Carved-swarm Step 5, component level — NOT a swarm certification.
//!
//! Two carved workers on one frozen weight buffer, forced failure +
//! recovery, L0 adversarial merge, exact K/V (weight) identity, restart.
//! Test weights only (64×0.25f32, 2 layers), not Glimmer weights, and no
//! model task runs here. The spec's full end-to-end run — one real
//! multi-packet model task, compared against the same model without
//! carving/overwatch — remains open: it needs the Glimmer serving route
//! (spec open item 2), which this test does not invent.
//!
//! L0 merge here is a test-level gate over existing APIs: base-weight
//! integrity must pass, both workers' atoms must be in the global map,
//! and view isolation must hold, before a merge root is issued. A
//! failed gate returns Err and merges nothing. There is no production
//! L0 merge API in the backend yet; this test proves the gate logic,
//! it does not claim one exists.
use ares_backend::compliance::{ComplianceViolation, ImmuneSystem, OverlapPolicyMatrix};
use ares_backend::digest::Sha256Digest;
use ares_backend::fixed::Q32_31;
use ares_backend::roles::RoleId;
use ares_backend::store::{CarvedSwarmMap, FrozenBaseWeights, LorentzCoord};
use std::sync::Arc;

fn base_map() -> (CarvedSwarmMap, Sha256Digest) {
    let base = FrozenBaseWeights::freeze("step5-test-model", 2, vec![0.25f32; 64]);
    let fp = base.fingerprint;
    (CarvedSwarmMap::new(base, Q32_31::from_f64(1.22)), fp)
}

fn immune(fp: Sha256Digest) -> ImmuneSystem {
    ImmuneSystem::new(OverlapPolicyMatrix::zero_overlap(), fp)
}

/// L0 gate: merge (issue a Merkle root) only if the frozen buffer is
/// intact, both workers' atoms are globally present, and each worker
/// can see only its own atom. Anything else fails closed, merging nothing.
fn l0_merge(map: &CarvedSwarmMap, sys: &mut ImmuneSystem, a: u64, b: u64) -> Result<Sha256Digest, String> {
    sys.check_base_weights(map).map_err(|e| format!("L0 gate: {:?}", e))?;
    if map.global_atom(a).is_none() || map.global_atom(b).is_none() {
        return Err("L0 gate: worker atom missing from global map".into());
    }
    let wa = map.scoped_read(RoleId::LaneBuilder).ok_or("L0 gate: no worker A reader")?;
    let wb = map.scoped_read(RoleId::LaneVerifier).ok_or("L0 gate: no worker B reader")?;
    if wa.atom(a).is_none() || wb.atom(b).is_none() {
        return Err("L0 gate: worker cannot see its own atom".into());
    }
    if wa.atom(b).is_some() || wb.atom(a).is_some() {
        return Err("L0 gate: view escape — worker sees the other worker's atom".into());
    }
    if !wa.permits_endpoints(a, a) || !wb.permits_endpoints(b, b) {
        return Err("L0 gate: worker cannot traverse its own atom".into());
    }
    if wa.permits_endpoints(a, b) || wb.permits_endpoints(b, a) {
        return Err("L0 gate: cross-view edge permitted".into());
    }
    Ok(map.compute_swarm_merkle_root())
}

#[test]
fn step5_two_workers_one_buffer_forced_failure_l0_merge_restart() {
    let (mut map, fp) = base_map();
    let policy = OverlapPolicyMatrix::zero_overlap();
    let mut sys = immune(fp);

    // Two workers, each carving one result atom into the global map.
    let atom_a = map.keep_and_carve(
        LorentzCoord::origin(),
        b"worker-a-result".to_vec(),
        RoleId::LaneBuilder,
        &policy,
    );
    let atom_b = map.keep_and_carve(
        LorentzCoord::origin(),
        b"worker-b-result".to_vec(),
        RoleId::LaneVerifier,
        &policy,
    );
    assert_eq!(map.global_atom_count(), 2);

    // 1. One buffer, not copies: both workers' weight views are the same Arc,
    //    and that Arc is the map's frozen base. Fingerprint is a content hash.
    let wa = map.scoped_read(RoleId::LaneBuilder).unwrap();
    let wb = map.scoped_read(RoleId::LaneVerifier).unwrap();
    assert!(Arc::ptr_eq(wa.weights(), wb.weights()));
    assert!(Arc::ptr_eq(wa.weights(), &map.base_weights));
    assert_eq!(wa.weights().fingerprint, fp);
    assert!(wa.weights().verify_integrity());
    assert_eq!(
        wa.weights().raw_weights(),
        wb.weights().raw_weights(),
        "exact K/V (weight) identity: byte-identical slices"
    );

    // 2. View isolation, enforced in lookup, not in a prompt.
    assert_eq!(wa.atom(atom_a).unwrap().payload, b"worker-a-result");
    assert_eq!(wb.atom(atom_b).unwrap().payload, b"worker-b-result");
    assert!(wa.atom(atom_b).is_none());
    assert!(wb.atom(atom_a).is_none());

    // 3. L0 adversarial merge passes on the clean state.
    let root_clean = l0_merge(&map, &mut sys, atom_a, atom_b).expect("clean L0 merge must pass");

    // 4. Forced failure: tamper the frozen buffer. The L0 gate must reject
    //    and merge nothing; the failure must not corrupt the merged state.
    let mut tampered = FrozenBaseWeights::freeze("step5-test-model", 2, vec![0.25f32; 64]);
    tampered.tamper_for_test(0, 9.99);
    assert!(!tampered.verify_integrity());
    let original_base = Arc::clone(&map.base_weights);
    map.base_weights = Arc::new(tampered);
    let err = sys.check_base_weights(&map).unwrap_err();
    assert!(matches!(err, ComplianceViolation::BaseWeightsModified { .. }));
    assert!(l0_merge(&map, &mut sys, atom_a, atom_b).is_err(), "failed gate must not merge");
    // A failed worker attempt creates no atom: count and root are unchanged.
    assert_eq!(map.global_atom_count(), 2);

    // 5. Recovery: restore the frozen buffer. Integrity, atoms, and the
    //    merge root are exactly what they were before the failure.
    map.base_weights = original_base;
    assert!(map.base_weights.verify_integrity());
    let root_recovered = l0_merge(&map, &mut sys, atom_a, atom_b).expect("recovered L0 merge must pass");
    assert_eq!(root_recovered, root_clean);

    // 6. Restart comparison: rebuild from the same frozen weights and replay
    //    the same two carves in the same order. Root, count, and fingerprint
    //    must be identical — restart does not change the merged state.
    let (mut map2, fp2) = base_map();
    let a2 = map2.keep_and_carve(
        LorentzCoord::origin(),
        b"worker-a-result".to_vec(),
        RoleId::LaneBuilder,
        &policy,
    );
    let b2 = map2.keep_and_carve(
        LorentzCoord::origin(),
        b"worker-b-result".to_vec(),
        RoleId::LaneVerifier,
        &policy,
    );
    assert_eq!(fp2, fp);
    assert_eq!(map2.global_atom_count(), 2);
    assert_eq!(a2, atom_a);
    assert_eq!(b2, atom_b);
    let root_restart = l0_merge(&map2, &mut sys, a2, b2).expect("restart L0 merge must pass");
    assert_eq!(root_restart, root_clean);
}

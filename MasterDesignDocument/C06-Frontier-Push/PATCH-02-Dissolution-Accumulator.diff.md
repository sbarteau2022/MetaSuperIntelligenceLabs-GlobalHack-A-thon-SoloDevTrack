# PATCH-02 — Dissolution Accumulator XOR → SHA256(pre ‖ canonical_write_set)

**Status.** PROPOSED patch, NOT applied to vault (tree dirty, owner applies). **Date.** 2026-10-27.
**Target.** `CarvedSwarmMap::compile_deposit` post-merge check + new `collect_merkle_fingerprint` (B-5 companion to the C25 DissolutionTransaction doc, which already specifies this shape in prose — this patch is the code).
**Replaces.** Any `expected_post = pre_fp XOR intended_write_set_hash` comparison (spec text in Methodology §3.2 + Dissolution doc §3 step 3 use XOR — the doc text must be amended alongside, see §4 below).
**Why.** XOR is linear/malleable: `a⊕b = c` has many preimages, order-insensitive, differential analysis leaks. A passing XOR check does not imply a correct merge — voids the C06.12(d) judge. (Proved in Formalized dyptech P-7.)

```rust
// ADD to CarvedSwarmMap (silo.rs), next to collect_merkle_leaf_bytes:

/// Canonical Merkle fingerprint of the global map: SHA-256 over leaves
/// ordered by AtomId ascending. Deterministic across runs (C03.5).
/// Leaves include arcs? NO — atoms only (dissolution doc §4.1: arcs enter
/// the intended write-set explicitly when in scope).
pub fn collect_merkle_fingerprint(&self) -> Sha256Digest {
    let mut leaves = self.collect_merkle_leaf_bytes();
    // collect_merkle_leaf_bytes iterates a BTreeMap keyed by u64 in current
    // code — VERIFY ordering here rather than assuming it: sort by the
    // leading 8 id bytes explicitly.
    leaves.sort_by(|a, b| a[0..8].cmp(&b[0..8]));
    // Merkle tree over leaves; empty map => SHA256("ATOMA_EMPTY_MAP_V1").
    ...
}

/// Canonical intended-write-set hash: order-independent EXCEPT by explicit
/// sort (never by insertion order — insertion order is nondeterministic
/// across runs).
pub fn intended_write_set_hash(atom_ids: &[AtomId], payload_hashes: &[Sha256Digest], origin: &Sha256Digest) -> Sha256Digest {
    let mut ids = atom_ids.to_vec(); ids.sort();
    let mut buf = Vec::new();
    for id in &ids { buf.extend_from_slice(&id.0.to_le_bytes()); }
    // payload hashes sorted to match (same permutation as ids where paired;
    // standalone hashes sorted independently and length-prefixed).
    ...
}

/// Replacement check inside dissolve_deposit_transactional (doc §3 step 3):
///   expected_post = SHA256(pre_fp || intended_write_set_hash)
///   observed_post = self.collect_merkle_fingerprint()
///   commit iff observed_post == expected_post else rollback + quarantine.
```

```diff
--- DissolutionTransactionAndTelemetry.md §3 step 3 (doc amendment, same patch)
+++ (companion — land together or neither lands)
- 3. expected_post = pre_fp xor hash(ws)           // intended write set hash
+ 3. expected_post = SHA256(pre_fp || intended_write_set_hash(ws))   // B-5: XOR retired (malleable)
--- Methodology §3.2 step 4 / HowWeTurn §5 PROMOTE (doc amendments, same patch)
+- expected = SHA256(fingerprint_pre || canonical_write_set_hash); recompute Merkle root over affected basin range as cross-check
```

**Test (property, from doc §11).** Tamper one payload hash in WriteSet → observed ≠ expected → rollback + quarantine receipt, map unchanged (Mode A snapshot). Fuzz 1k random deposits: Mode A vs Mode B identical committed roots.

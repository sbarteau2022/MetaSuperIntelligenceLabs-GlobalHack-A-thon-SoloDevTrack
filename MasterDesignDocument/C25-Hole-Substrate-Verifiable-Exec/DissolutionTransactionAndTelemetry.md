# C25.x — Atomic Dissolution Transaction and Hidden-Budget Telemetry Contract

**Status.** PROPOSED implementation contract. **Date:** 2026-10-27. **Purpose:** make the transition from current `compile_deposit` to a transactional, crash-safe, auditable dissolution path explicit, with tests and a code-level API that can be implemented without guessing.

**Parents:** C06.3 receipts, C06.11 provenance, C06.12 holonomy, C24 compaction, C25 Merkle guards, C31 telemetry, C30.8 soft budget, C30.9 failure replanning, C07 privacy.

---

## 1. Problem to solve

`CarvedSwarmMap::compile_deposit` mutates the global map in place while it coalesces deposits. If a crash, partial merge, or fingerprint mismatch occurs, the map can be left in a torn state, violating the dissolution invariant:

$$
\text{fingerprint}_{post} = \text{fingerprint}_{pre} \oplus \text{intended\_write\_set\_hash}
$$

Today that check is caller-implicit. We need an **atomic transaction** with automatic rollback and receipted quarantine.

---

## 2. Transactional dissolution API (target Rust contract)

```rust
pub struct DissolutionTxnReceipt {
    pub from_role: RoleId,
    pub dissolved_pid: u64,
    pub txn_id: Uuid,                 // new: unique transaction id
    pub pre_fingerprint: Sha256Digest,
    pub intended_write_set_hash: Sha256Digest,
    pub post_fingerprint: Sha256Digest,
    pub outcome: DissolutionOutcome,  // Committed | RolledBack | Quarantined
    pub quarantine_receipt: Option<QuarantineReceipt>,
    pub atoms_touched: usize,
    pub coalesced: usize,
    pub newly_compiled: usize,
    pub duration_ms: u64,
    pub crash_recovered: bool,
}

pub enum DissolutionOutcome { Committed, RolledBack, Quarantined }

pub fn dissolve_deposit_transactional(
    &mut self,                      // CarvedSwarmMap
    deposit: &EphemeralMapDeposit,
    policy: &OverlapPolicyMatrix,
    intended_write_set: &WriteSet, // NEW: explicit intended writes
    telemetry: &mut TelemetrySink,  // NEW: counts/ids/ratios only
) -> DissolutionTxnReceipt;
```

### 2.1 New helper types

```rust
pub struct WriteSet {
    pub atom_ids: Vec<AtomId>,      // atoms expected to be touched
    pub payload_hashes: Vec<Sha256Digest>, // expected post-merge payload digests
    pub origin_fingerprint: Sha256Digest,
}

pub struct QuarantineReceipt {
    pub txn_id: Uuid,
    pub from_role: RoleId,
    pub reason: QuarantineReason, // FingerprintMismatch | Crash | LifecycleViolation | SeatbeltViolation
    pub pre_fingerprint: Sha256Digest,
    pub observed_post_fingerprint: Sha256Digest,
    pub preserved_root: Sha256Digest, // last good root
}
```

---

## 3. Transaction algorithm (step-by-step)

```
dissolve_deposit_transactional(self, deposit, policy, ws, telemetry):
1. txn_id = new_uuid()
2. pre_fp = self.collect_merkle_fingerprint()   // root of current map
3. expected_post = pre_fp xor hash(ws)           // intended write set hash
4. snapshot = self.begin_transaction()            // in-memory clone OR WAL bookmark
   - If clone mode: Arc/RwLock guard + snapshot clone (simplest, tests first).
   - If WAL mode: record txn begin + pre_fp + ws hash in WAL; set state = InTxn.

5. attempt:
   a. run existing compile_deposit logic on self, producing inner receipt
   b. compute observed_post = self.collect_merkle_fingerprint()
   c. if observed_post == expected_post:
        - commit transaction marker (WAL append "commit")
        - emit telemetry: atoms_touched, coalesced, newly_compiled, duration (counts only)
        - return Committed receipt
   d. else:
        - rollback to snapshot (WAL replay truncation or in-memory restore)
        - quarantine deposit:
            QuarantineReceipt { txn_id, from_role, FingerprintMismatch, pre_fp, observed_post, pre_fp }
        - emit telemetry: quarantine event (id + reason only, no payload)
        - return RolledBack/Quarantined receipt

6. on panic / crash:
   - on restart, WAL recovery sees txn in "InTxn" without "commit"
   - automatically rollback to pre_fp
   - emit crash_recovered = true
   - quarantine reason = Crash
```

---

## 4. Merkle fingerprint function (exact contract)

```rust
pub fn collect_merkle_fingerprint(&self) -> Sha256Digest {
    // Leaf bytes exactly as collect_merkle_leaf_bytes already does:
    // id || clock || lorentz.x (4 x Q32_31) || phase.theta_f || phase.theta_s || payload
    let leaves = self.collect_merkle_leaf_bytes();
    let tree = MerkleTree::from_leaves(&leaves); // deterministic ordering by AtomId
    tree.root()
}
```

### 4.1 Ordering rule (critical)

Leaves must be ordered by `AtomId` ascending. Reasoning arcs are **not** leaves; they are part of the transaction's intended writes only if explicitly included in `WriteSet`. This keeps the root stable and auditable.

### 4.2 `intended_write_set_hash`

```rust
fn intended_write_set_hash(ws: &WriteSet) -> Sha256Digest {
    let mut buf = Vec::new();
    for id in &ws.atom_ids { buf.extend_from_slice(&id.0.to_le_bytes()); }
    for h in &ws.payload_hashes { buf.extend_from_slice(&h.0); }
    buf.extend_from_slice(&ws.origin_fingerprint.0);
    sha256(&buf)
}
```

---

## 5. Rollback strategy (two modes)

### Mode A — in-memory snapshot (tests first)
- `let snapshot = self.clone()` (requires `Clone` on `CarvedSwarmMap` or a `TransactionState` struct).
- On failure: `*self = snapshot` (move back).
- Pros: simple. Cons: memory cost `O(|A|)` per transaction.

### Mode B — WAL journaling (production)
- WAL already exists (`.atoma_map.wal`). Append:
  - `TxnBegin(txn_id, pre_fp, intended_hash)`
  - per-operation reverse deltas (delete-inserted, restore-coalesced, revert-salience)
  - `TxnCommit(txn_id, post_fp)` or `TxnAbort(txn_id, reason)`
- On crash/restart: replay WAL; any `TxnBegin` without matching `TxnCommit`/`TxnAbort` triggers automatic abort to `pre_fp`.
- Pros: no full snapshot. Cons: more code.

**Implementation order:** Mode A first (property tests), then Mode B before A38 production run. Mode A stays as test oracle for Mode B.

---

## 6. Quarantine and audit

On fingerprint mismatch or crash:

- Deposit is **not** applied (or is rolled back).
- `QuarantineReceipt` is appended to the ledger with txn id, reason, roots, and timestamp (counts/ids only; no payload).
- The hole/agent that submitted the deposit is marked quarantined; further deposits from it are refused until explicit approval (C25 lifecycle: `HoleCorrupt`).
- Telemetry emits a `quarantine` event with `txn_id`, `from_role`, `reason` — no raw atom content.

---

## 7. Telemetry fields (Meta-only control plane, C30.8/C30.9)

Minimum fields for `dissolve_deposit_transactional` (all counts/ids/ratios; no payloads/text):

```rust
struct DissolutionTelemetry {
    txn_id: Uuid,                    // id
    from_role: RoleId,               // id
    outcome: DissolutionOutcome,      // enum
    pre_fingerprint: Sha256Digest,    // hash
    post_fingerprint: Sha256Digest,   // hash (if committed)
    intended_write_set_hash: Sha256Digest,
    atoms_touched: u64,             // count
    newly_compiled: u64,             // count
    coalesced: u64,                  // count
    duration_ms: u64,                // ratio/time
    crash_recovered: bool,           // flag
    quarantine_reason: Option<QuarantineReason>,
    // Hidden soft-budget fields (Meta only):
    scope_size_atoms_before: u64,
    scope_size_atoms_after: u64,
    lambda_effective: f64,           // ratio
}
```

These flow to the Meta control plane only. They never return to the agent (I-SB-1). The agent sees a success/fail + receipt hash, not the telemetry.

---

## 8. Privacy and security invariants

- **I-TX-1:** Transactional dissolution never leaks raw atom payload, objective, or invariant text into telemetry, receipts beyond hashes, or the agent.
- **I-TX-2:** A mismatched fingerprint is a hard failure, not a warning; the map stays at `pre_fp`.
- **I-TX-3:** Crash recovery is automatic and cannot be bypassed by agent code.
- **I-TX-4:** Quarantined deposits remain append-only in the ledger (no deletion; correction by new entry).
- **I-TX-5:** Hidden soft-budget fields (`profit_estimate`, `budget_*`) never appear in `D_T` or agent-facing responses.

---

## 9. Agent-facing vs Meta-facing contract

| Channel | What it sees |
|---|---|
| Agent (`carve_task_view` consumer) | success/fail + receipt hash + (on fail) refusal reason class only |
| Meta control plane | full `DissolutionTxnReceipt` + telemetry above |
| Ledger (audit) | txn id, role, outcome, roots, duration, counts; hashes only |
| External reviewer | receipt terminating in ledger root; can recompute fingerprint from published leaves |

This asymmetry is what makes the failure path trainable (C30.9) without leaking budget or data.

---

## 10. Transition-to-code checklist (ready to implement)

1. Add `WriteSet`, `QuarantineReceipt`, `DissolutionOutcome`, `DissolutionTxnReceipt` structs (§2).
2. Implement `collect_merkle_fingerprint` using existing `collect_merkle_leaf_bytes` + `Sha256` Merkle root (verify leaf ordering by `AtomId`).
3. Implement `begin_transaction()` snapshot (Mode A clone) in `CarvedSwarmMap`.
4. Wrap existing `compile_deposit` body inside `dissolve_deposit_transactional` with the fingerprint compare + rollback.
5. Add `intended_write_set_hash` and `expected_post` computation.
6. Emit `DissolutionTelemetry` through a `TelemetrySink` trait (counts only).
7. Implement crash recovery hook: on load, scan WAL for unmatched `TxnBegin`; abort to `pre_fp`.
8. Update callers (orchestrator, hot-swap path) to use the new transactional API; keep `compile_deposit` as deprecated thin wrapper during migration.
9. Add tests (§11).

---

## 11. Test plan (property + crash)

1. **Happy path:** deposit compiles, fingerprint matches, receipt outcome `Committed`, map equals expected.
2. **Mismatch quarantine:** tamper one payload hash in `WriteSet` → observed post ≠ expected → rollback, quarantine receipt, map unchanged.
3. **Crash recovery (Mode A):** simulate panic mid-merge (inject test hook) → restart → map restored to `pre_fp`, `crash_recovered = true`.
4. **Idempotence:** re-submitting same deposit after commit does not change fingerprint beyond intended writes (coalescing handles duplicates).
5. **Telemetry privacy:** fuzz 10k deposits; assert no payload bytes in any telemetry field.
6. **Hard-budget cap:** 10k deposits in a row; assert memory stays within `max_atoms` and no OOM (C21).
7. **Oracle equivalence:** Mode B (WAL) vs Mode A (snapshot) produce identical committed roots on 1k random deposit sequences.

---

## 12. Open gaps before coding

- `CarvedSwarmMap` must derive or implement `Clone` (or a cheaper `TransactionState`) for Mode A; check memory cost on 100k-atom maps.
- WAL reverse-delta format (Mode B) needs design before Mode B; do Mode A + tests first.
- `TelemetrySink` trait location: likely C31; needs a concrete implementation for tests (stdout/in-memory).
- Quarantine escalation policy: how a quarantined agent/worktree is reset (ties to C25 `HoleCorrupt` and C30.9 failure replanning).

---

## 13. Invariants summary

- **I-TX-1..I-TX-5** as in `ErrAndImprovements-AA.md` / `AdvAudit.md`.
- No claim of VERIFIED until tests (§11) pass and A38/A39 data exists.

---

_Last updated: 2026-10-27 (dissolution transaction + telemetry contract pass)._

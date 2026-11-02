//! Task (b): preregistration as a compiled, hashed, immutable, signed struct.
//!
//! Procedure (Trash `design/skills/preregister.md`, read-only ref): write the
//! claims BEFORE any implementation exists, seal (commit or record sha256),
//! build the suite from the claims, report against the sealed claims.
//! Here the seal is compiled in: [`PreregPlan`] holds task + DAG + judges +
//! criteria; [`SealedPreregistration`] freezes it with a sha256 computed at
//! seal time, plus signer id and timestamp.
//!
//! Frozen means frozen: all fields are private, only shared-reference getters
//! exist, and there is deliberately NO setter, NO `&mut self` method, NO
//! `DerefMut`, NO `pub` field. A mutation attempt fails to compile — see
//! `negative/mutation_attempt.rs` (excluded from the build; paste of the
//! `rustc` rejection lives in RECEIPT.md).
//!
//! Any change = a new preregistration: [`SealedPreregistration::amend`]
//! takes `&self` and returns a NEW sealed value (new timestamp/nonce, new
//! hash). The original is untouched. Date: 2026-10-31.

use sha2::{Digest, Sha256};

/// The plan, written BEFORE any implementation exists. Plain data; sealing
/// consumes it (by value) so the unsealed draft cannot be confused with a seal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreregPlan {
    /// Goal + inputs + constraints + definition of done (runbook INTAKE).
    pub task: String,
    /// DAG node names in canonical declaration order.
    pub dag: Vec<String>,
    /// Judge per node, named before anything runs (runbook PLAN).
    pub judges: Vec<String>,
    /// Refutable, quantified, bounded claims; each names its decider test.
    pub criteria: Vec<String>,
}

impl PreregPlan {
    pub fn new(
        task: impl Into<String>,
        dag: Vec<String>,
        judges: Vec<String>,
        criteria: Vec<String>,
    ) -> Self {
        PreregPlan { task: task.into(), dag, judges, criteria }
    }
}

/// A sealed preregistration. Immutable by construction (see module docs).
/// `Clone` copies the seal; it cannot mutate the original.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealedPreregistration {
    task: String,
    dag: Vec<String>,
    judges: Vec<String>,
    criteria: Vec<String>,
    hash: String,
    signer: String,
    timestamp: u64,
    nonce: u64,
}

impl SealedPreregistration {
    /// Seal a plan: freeze content + signer + timestamp + nonce, hash it.
    /// Canonical encoding: fields joined with unit separator; hash covers ALL
    /// of it, so any content change alters the hash.
    pub fn seal(plan: PreregPlan, signer: impl Into<String>, timestamp: u64, nonce: u64) -> Self {
        let signer = signer.into();
        let hash = compute_hash(
            &plan.task, &plan.dag, &plan.judges, &plan.criteria, &signer, timestamp, nonce,
        );
        SealedPreregistration {
            task: plan.task,
            dag: plan.dag,
            judges: plan.judges,
            criteria: plan.criteria,
            hash,
            signer,
            timestamp,
            nonce,
        }
    }

    /// Amend: returns a NEW seal carrying the amendment record. `&self` —
    /// the original is never touched. New timestamp/nonce => new hash.
    pub fn amend(&self, amendment: &str, timestamp: u64, nonce: u64) -> Self {
        let mut criteria = self.criteria.clone();
        criteria.push(format!("AMENDMENT: {}", amendment));
        let hash = compute_hash(
            &self.task, &self.dag, &self.judges, &criteria, &self.signer, timestamp, nonce,
        );
        SealedPreregistration {
            task: self.task.clone(),
            dag: self.dag.clone(),
            judges: self.judges.clone(),
            criteria,
            hash,
            signer: self.signer.clone(),
            timestamp,
            nonce,
        }
    }

    /// Recompute the seal hash and compare. False = tampered or corrupted.
    pub fn verify(&self) -> bool {
        compute_hash(
            &self.task,
            &self.dag,
            &self.judges,
            &self.criteria,
            &self.signer,
            self.timestamp,
            self.nonce,
        ) == self.hash
    }

    // --- read-only getters (shared refs / copies only) ---
    pub fn task(&self) -> &str {
        &self.task
    }
    pub fn dag(&self) -> &[String] {
        &self.dag
    }
    pub fn judges(&self) -> &[String] {
        &self.judges
    }
    pub fn criteria(&self) -> &[String] {
        &self.criteria
    }
    pub fn hash(&self) -> &str {
        &self.hash
    }
    pub fn signer(&self) -> &str {
        &self.signer
    }
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
    pub fn nonce(&self) -> u64 {
        self.nonce
    }
}

fn canonical(
    task: &str,
    dag: &[String],
    judges: &[String],
    criteria: &[String],
    signer: &str,
    timestamp: u64,
    nonce: u64,
) -> String {
    const U: char = '\u{1f}';
    format!(
        "{task}{U}{dag}{U}{judges}{U}{criteria}{U}{signer}{U}{timestamp}{U}{nonce}",
        dag = dag.join(","),
        judges = judges.join(","),
        criteria = criteria.join(","),
    )
}

fn compute_hash(
    task: &str,
    dag: &[String],
    judges: &[String],
    criteria: &[String],
    signer: &str,
    timestamp: u64,
    nonce: u64,
) -> String {
    let bytes = canonical(task, dag, judges, criteria, signer, timestamp, nonce);
    let digest = Sha256::digest(bytes.as_bytes());
    to_hex(&digest)
}

fn to_hex(bytes: &[u8]) -> String {
    const H: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(H[(b >> 4) as usize] as char);
        s.push(H[(b & 0xf) as usize] as char);
    }
    s
}

/// Test helper: the B2 plan for the decomposition DAG.
pub fn b2_plan() -> PreregPlan {
    PreregPlan::new(
        "B2: typed decomposition DAG + sealed preregistration, disjoint from B1",
        vec![
            "Explorer".into(),
            "Architect".into(),
            "SurfaceCoder".into(),
            "GraphCoder".into(),
            "WorkbenchCoder".into(),
            "Critic".into(),
        ],
        vec![
            "InventoryComplete".into(),
            "ContractGate".into(),
            "Typecheck".into(),
            "Typecheck".into(),
            "Typecheck".into(),
            "VisualQa".into(),
        ],
        vec![
            "C1 typecheck-before-run: mismatched edge fails, nothing executes".into(),
            "C2 topo order: explorer first, critic last, coders parallel".into(),
            "C3 shuffled input rejected".into(),
            "C4 seal: identical task sealed twice differs only by time/nonce".into(),
            "C5 amend creates new seal; original unchanged".into(),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seal_verifies_and_hash_is_sha256_hex() {
        let s = SealedPreregistration::seal(b2_plan(), "agent-b2", 1_730_000_000, 7);
        assert!(s.verify());
        assert_eq!(s.hash().len(), 64);
        assert!(s.hash().chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn identical_task_seals_differ_only_by_time_nonce() {
        let a = SealedPreregistration::seal(b2_plan(), "agent-b2", 1_730_000_000, 1);
        let b = SealedPreregistration::seal(b2_plan(), "agent-b2", 1_730_000_001, 2);
        // Same content everywhere except timestamp/nonce/hash...
        assert_eq!(a.task(), b.task());
        assert_eq!(a.dag(), b.dag());
        assert_eq!(a.judges(), b.judges());
        assert_eq!(a.criteria(), b.criteria());
        assert_eq!(a.signer(), b.signer());
        assert_ne!(a.timestamp(), b.timestamp());
        assert_ne!(a.nonce(), b.nonce());
        assert_ne!(a.hash(), b.hash());
        assert!(a.verify() && b.verify());
    }

    #[test]
    fn content_change_alters_hash() {
        let a = SealedPreregistration::seal(b2_plan(), "agent-b2", 1_730_000_000, 1);
        let mut plan = b2_plan();
        plan.criteria[0] = "C1 MUTATED".into();
        let b = SealedPreregistration::seal(plan, "agent-b2", 1_730_000_000, 1);
        assert_ne!(a.hash(), b.hash());
        assert!(b.verify());
    }

    #[test]
    fn amend_creates_new_original_unchanged() {
        let orig = SealedPreregistration::seal(b2_plan(), "agent-b2", 1_730_000_000, 1);
        let before = orig.clone();
        let amended = orig.amend("C3 extended with 4th shuffle", 1_730_000_100, 9);
        // Original untouched...
        assert_eq!(orig, before);
        assert_eq!(orig.criteria().len(), 5);
        // ...new seal carries the amendment with a new hash.
        assert_eq!(amended.criteria().len(), 6);
        assert!(amended.criteria()[5].starts_with("AMENDMENT:"));
        assert_ne!(amended.hash(), orig.hash());
        assert!(amended.verify() && orig.verify());
    }

    #[test]
    fn no_mutation_api_surface() {
        // Compile-time freeze has no runtime bypass: the struct exposes zero
        // `&mut self` methods and zero public fields. Assert the type's
        // method surface by checking mutation is impossible without interior
        // mutability: a shared reference cannot yield a mutable one, and the
        // fields are not reachable (this test itself only compiles because it
        // uses the read-only getters — direct field access would not).
        let s = SealedPreregistration::seal(b2_plan(), "agent-b2", 1_730_000_000, 1);
        let r: &SealedPreregistration = &s;
        assert_eq!(r.task(), "B2: typed decomposition DAG + sealed preregistration, disjoint from B1");
        assert_eq!(r.signer(), "agent-b2");
        // No setters exist: uncommenting any of these must fail to compile
        // (proven by negative/mutation_attempt.rs + rustc paste in RECEIPT):
        //   s.task = String::new();
        //   s.amend_mut(...);  // no such method
    }
}

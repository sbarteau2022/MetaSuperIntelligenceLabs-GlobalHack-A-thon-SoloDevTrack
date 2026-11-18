//! Capability tree scaffold, in the mind — NOT in N3, NOT from trajectories.
//!
//! Status: SCAFFOLD (2026-11-02). Families and edges first, leaves by the
//! problem they solve, trajectories attached later as evidence only.
//!
//! Design, from the §19 registry (`research-grade-harness/src/capability.rs`)
//! and the W2 proposal in `HYPERBOLIC-SCHEDULER-FRAMEWORK-2026-11-02.md`:
//! - The tree is POTENTIAL: what the system can do, and what it needs first.
//! - A trajectory is EVIDENCE: a run that did it once, judged, receipted.
//!   Trajectories attach to leaves; they never create leaves, and an
//!   unjudged trajectory never proves a leaf.
//! - Family is a coarse prior, not leaf identity. Leaf identity is the
//!   problem text, embedded with the mind's own `coord_from_text_idf`
//!   pipeline (rank 0, chunk space), so leaves live in the same H3 x T2
//!   space as atoms and can be probed geometrically.
//! - Admission is a second gate: the §19 family DAG admits or denies,
//!   regardless of trajectory history. No 06-DataKnowledge without
//!   03-AgenticRuntime + 05-Verification, and 07-SecuritySandbox needs all.
//!
//! Deliberately NOT in this scaffold: VP-range retrieval over leaves
//! (brute-force hyperbolic distance for now, VP is the next step),
//! persistence to SurrealDB (leaves are in-memory until the mind's
//! store layer takes them), and any trajectory import.

use std::collections::{HashMap, HashSet};

/// The seven §19 families, in canonical order.
pub const KNOWN_FAMILIES: [&str; 7] = [
    "01-FormalMethods",
    "02-SystemsKernels",
    "03-AgenticRuntime",
    "04-Observability",
    "05-Verification",
    "06-DataKnowledge",
    "07-SecuritySandbox",
];

/// Hardcoded §19 family-dependency edges (dependency -> dependent).
/// Same edges as `research-grade-harness/src/capability.rs::family_edges`.
pub fn family_edges() -> Vec<(String, String)> {
    [
        ("01-FormalMethods", "03-AgenticRuntime"),
        ("02-SystemsKernels", "03-AgenticRuntime"),
        ("03-AgenticRuntime", "04-Observability"),
        ("03-AgenticRuntime", "05-Verification"),
        ("03-AgenticRuntime", "06-DataKnowledge"),
        ("05-Verification", "06-DataKnowledge"),
        ("01-FormalMethods", "07-SecuritySandbox"),
        ("02-SystemsKernels", "07-SecuritySandbox"),
        ("03-AgenticRuntime", "07-SecuritySandbox"),
        ("04-Observability", "07-SecuritySandbox"),
        ("05-Verification", "07-SecuritySandbox"),
        ("06-DataKnowledge", "07-SecuritySandbox"),
    ]
    .iter()
    .map(|(a, b)| (a.to_string(), b.to_string()))
    .collect()
}

/// One capability leaf: one problem the system can solve.
/// `proven` is false until a JUDGED trajectory attaches (see
/// `attach_trajectory`). `trajectories` is evidence, never structure.
#[derive(Debug, Clone)]
pub struct CapabilityLeaf {
    pub id: String,
    pub family: String,
    /// The problem this leaf solves, in plain text. This is the leaf's
    /// identity for embedding — not a run, not a trajectory id.
    pub problem: String,
    /// Lorentz coordinate in H3 x T2, from `coord_from_text_idf`.
    pub coord: [f64; 4],
    pub phase: [f64; 2],
    /// sha256 hex of `family|id|problem`.
    pub digest: String,
    pub proven: bool,
    pub trajectories: Vec<String>,
}

fn digest_of(family: &str, id: &str, problem: &str) -> String {
    use sha2::{Digest, Sha256};
    let canon = format!("{}|{}|{}", family, id, problem);
    let mut h = Sha256::new();
    h.update(canon.as_bytes());
    // No hex dependency in this crate; format manually.
    h.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}

/// The scaffolded tree: families, edges, leaves. No trajectories are
/// read at construction; none are needed to build it.
#[derive(Debug, Default)]
pub struct CapabilityTree {
    pub leaves: Vec<CapabilityLeaf>,
}

impl CapabilityTree {
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate the family DAG: all 7 families present, all edges
    /// endpoint-known, no cycles (Kahn). Returns the topological order.
    pub fn validate_families() -> Result<Vec<String>, String> {
        let edges = family_edges();
        let mut indeg: HashMap<String, usize> = KNOWN_FAMILIES
            .iter()
            .map(|f| (f.to_string(), 0))
            .collect();
        let mut dependents: HashMap<String, Vec<String>> = HashMap::new();
        for (dep, dependent) in &edges {
            if !indeg.contains_key(dep) || !indeg.contains_key(dependent) {
                return Err(format!("unknown family in edge {} -> {}", dep, dependent));
            }
            *indeg.get_mut(dependent).unwrap() += 1;
            dependents.entry(dep.clone()).or_default().push(dependent.clone());
        }
        let mut queue: Vec<String> = indeg
            .iter()
            .filter(|(_, d)| **d == 0)
            .map(|(f, _)| f.clone())
            .collect();
        queue.sort();
        let mut order = Vec::new();
        while let Some(f) = queue.pop() {
            order.push(f.clone());
            if let Some(ds) = dependents.get(&f) {
                for d in ds {
                    let e = indeg.get_mut(d).unwrap();
                    *e -= 1;
                    if *e == 0 {
                        queue.push(d.clone());
                    }
                }
            }
            queue.sort();
        }
        if order.len() != KNOWN_FAMILIES.len() {
            return Err("family DAG has a cycle".to_string());
        }
        Ok(order)
    }

    /// Transitive family dependencies of `family` (its prerequisites).
    pub fn family_prerequisites(family: &str) -> HashSet<String> {
        let edges = family_edges();
        let mut prereqs = HashSet::new();
        let mut stack = vec![family.to_string()];
        while let Some(cur) = stack.pop() {
            for (dep, dependent) in &edges {
                if dependent == &cur && prereqs.insert(dep.clone()) {
                    stack.push(dep.clone());
                }
            }
        }
        prereqs
    }

    /// DAG admission gate: a leaf's family is admitted only when every
    /// transitive prerequisite family is in `granted`. History and
    /// trajectories play no part in this decision.
    pub fn admit(family: &str, granted: &HashSet<String>) -> bool {
        if !KNOWN_FAMILIES.contains(&family) {
            return false;
        }
        Self::family_prerequisites(family)
            .iter()
            .all(|p| granted.contains(p))
    }

    /// Create a leaf by problem text. This is the ONLY creation path.
    /// Trajectories cannot create leaves.
    pub fn add_leaf(
        &mut self,
        family: &str,
        id: &str,
        problem: &str,
        salience: f64,
    ) -> Result<(), String> {
        if !KNOWN_FAMILIES.contains(&family) {
            return Err(format!("unknown family {}", family));
        }
        if self.leaves.iter().any(|l| l.id == id) {
            return Err(format!("leaf {} already exists", id));
        }
        let (coord, phase) = crate::coord_from_text_idf(problem, salience, None, 0);
        self.leaves.push(CapabilityLeaf {
            id: id.to_string(),
            family: family.to_string(),
            problem: problem.to_string(),
            coord,
            phase,
            digest: digest_of(family, id, problem),
            proven: false,
            trajectories: Vec::new(),
        });
        Ok(())
    }

    /// Family centroid as a coarse prior: mean of shell-projected leaf
    /// directions, re-lifted to a Lorentz coordinate. None if the family
    /// has no leaves yet — a family with no leaves is still a valid
    /// scaffold node, it just has no prior yet.
    pub fn family_centroid(&self, family: &str) -> Option<[f64; 4]> {
        let mut sum = [0f64; 3];
        let mut n = 0f64;
        for leaf in &self.leaves {
            if leaf.family == family {
                let shell = crate::shell_project(&leaf.coord);
                sum[0] += shell[1];
                sum[1] += shell[2];
                sum[2] += shell[3];
                n += 1.0;
            }
        }
        if n == 0.0 {
            return None;
        }
        let mean = [sum[0] / n, sum[1] / n, sum[2] / n];
        let x0 = (1.0 + mean[0] * mean[0] + mean[1] * mean[1] + mean[2] * mean[2]).sqrt();
        Some([x0, mean[0], mean[1], mean[2]])
    }

    /// Probe by problem text: leaves sorted by hyperbolic distance to
    /// the probe coordinate (brute force in this scaffold; VP-range
    /// retrieval is the next step, not claimed here).
    pub fn probe(&self, task: &str, salience: f64) -> Vec<(String, f64)> {
        let (qcoord, _) = crate::coord_from_text_idf(task, salience, None, 0);
        let mut hits: Vec<(String, f64)> = self
            .leaves
            .iter()
            .map(|l| (l.id.clone(), crate::hyperbolic_dist(&qcoord, &l.coord)))
            .collect();
        hits.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        hits
    }

    /// Attach a trajectory as evidence to an existing leaf.
    /// - Leaf must already exist; attaching to a missing leaf is an error,
    ///   never an implicit leaf creation.
    /// - `judged = true` (judged atom + receipt) marks the leaf proven.
    ///   An unjudged trajectory is recorded but proves nothing.
    pub fn attach_trajectory(
        &mut self,
        leaf_id: &str,
        trajectory_id: &str,
        judged: bool,
    ) -> Result<(), String> {
        let leaf = self
            .leaves
            .iter_mut()
            .find(|l| l.id == leaf_id)
            .ok_or_else(|| format!("no such leaf {}; trajectories never create leaves", leaf_id))?;
        if !leaf.trajectories.contains(&trajectory_id.to_string()) {
            leaf.trajectories.push(trajectory_id.to_string());
        }
        if judged {
            leaf.proven = true;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn families_validate_and_order_roots_first() {
        let order = CapabilityTree::validate_families().expect("families valid");
        assert_eq!(order.len(), 7);
        // Roots have no incoming edges; they come before their dependents.
        let pos = |f: &str| order.iter().position(|x| x == f).unwrap();
        assert!(pos("01-FormalMethods") < pos("03-AgenticRuntime"));
        assert!(pos("02-SystemsKernels") < pos("03-AgenticRuntime"));
        assert!(pos("03-AgenticRuntime") < pos("06-DataKnowledge"));
        assert!(pos("05-Verification") < pos("06-DataKnowledge"));
        assert!(pos("06-DataKnowledge") < pos("07-SecuritySandbox"));
    }

    #[test]
    fn dag_gate_denies_06_without_03_and_05() {
        let empty: HashSet<String> = HashSet::new();
        assert!(!CapabilityTree::admit("06-DataKnowledge", &empty));
        let mut g: HashSet<String> = HashSet::new();
        g.insert("03-AgenticRuntime".to_string());
        assert!(!CapabilityTree::admit("06-DataKnowledge", &g));
        g.insert("05-Verification".to_string());
        // Prerequisites are transitive: 03 needs 01+02, 05 needs 03.
        assert!(!CapabilityTree::admit("06-DataKnowledge", &g));
        g.insert("01-FormalMethods".to_string());
        g.insert("02-SystemsKernels".to_string());
        assert!(CapabilityTree::admit("06-DataKnowledge", &g));
    }

    #[test]
    fn leaves_are_created_by_problem_not_trajectory() {
        let mut tree = CapabilityTree::new();
        assert!(tree.add_leaf("03-AgenticRuntime", "test-leaf", "run a judged mission end to end", 0.5).is_ok());
        let leaf = &tree.leaves[0];
        assert!(!leaf.proven);
        assert!(leaf.trajectories.is_empty());
        // Same problem text, same salience -> same coordinate (deterministic).
        let mut tree2 = CapabilityTree::new();
        tree2.add_leaf("03-AgenticRuntime", "test-leaf", "run a judged mission end to end", 0.5).unwrap();
        assert_eq!(leaf.coord, tree2.leaves[0].coord);
        assert_eq!(leaf.digest, tree2.leaves[0].digest);
    }

    #[test]
    fn probe_ranks_same_problem_first() {
        let mut tree = CapabilityTree::new();
        tree.add_leaf("03-AgenticRuntime", "mission-leaf", "run a judged mission end to end", 0.5).unwrap();
        tree.add_leaf("05-Verification", "verify-leaf", "verify a merkle audit chain from stored rows", 0.5).unwrap();
        let hits = tree.probe("run a judged mission end to end", 0.5);
        assert_eq!(hits[0].0, "mission-leaf");
        assert!(hits[0].1 < hits[1].1);
    }

    #[test]
    fn trajectory_attaches_but_never_creates_and_unjudged_proves_nothing() {
        let mut tree = CapabilityTree::new();
        assert!(tree.attach_trajectory("missing-leaf", "traj-1", true).is_err());
        tree.add_leaf("05-Verification", "verify-leaf", "verify a merkle audit chain from stored rows", 0.5).unwrap();
        tree.attach_trajectory("verify-leaf", "traj-1", false).unwrap();
        assert!(!tree.leaves[0].proven);
        assert_eq!(tree.leaves[0].trajectories, vec!["traj-1".to_string()]);
        tree.attach_trajectory("verify-leaf", "traj-2", true).unwrap();
        assert!(tree.leaves[0].proven);
    }

    #[test]
    fn family_centroid_is_none_without_leaves_then_some() {
        let mut tree = CapabilityTree::new();
        assert!(tree.family_centroid("03-AgenticRuntime").is_none());
        tree.add_leaf("03-AgenticRuntime", "mission-leaf", "run a judged mission end to end", 0.5).unwrap();
        let c = tree.family_centroid("03-AgenticRuntime").expect("centroid");
        // Lorentz form: x0^2 - |x|^2 == 1
        let lhs = c[0] * c[0] - c[1] * c[1] - c[2] * c[2] - c[3] * c[3];
        assert!((lhs - 1.0).abs() < 1e-9);
    }
}

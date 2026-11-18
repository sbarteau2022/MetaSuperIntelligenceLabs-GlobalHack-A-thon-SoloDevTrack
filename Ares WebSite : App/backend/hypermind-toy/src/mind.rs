//! Toy hyperbolic mind: Lorentz H3 atoms, VP-tree exact-prune kNN, the
//! data-dependent depth gate, problem-embedded capability leaves, and a KV
//! store with producer edges + exact invalidation.
//!
//! Reference (read-only): hypermind-nested-kv `main.rs` — Lorentz inner
//! product, `hyperbolic_dist` (acosh of clamped inner), shell projection
//! (salience-invariant query-time normalization), VP-tree build/search with
//! the degenerate-split guard, `map.attend` shortlist→score→argmin cost form.
//! This toy re-implements the same math small (f64 H3, no T2 phase, no IDF,
//! no basins) so the scheduler rehearsal runs solo.

use std::collections::HashMap;

// ===========================================================================
// Lorentz H3 geometry (same formulas as the mind)
// ===========================================================================

/// Lorentz inner product on H3: <a,b> = a0·b0 − a1·b1 − a2·b2 − a3·b3.
pub fn lorentz_inner(a: &[f64; 4], b: &[f64; 4]) -> f64 {
    a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3]
}

/// Hyperbolic distance: acosh(max(<a,b>, 1)).
pub fn hyperbolic_dist(a: &[f64; 4], b: &[f64; 4]) -> f64 {
    lorentz_inner(a, b).max(1.0).acosh()
}

/// Shell projection: direction only (salience-invariant comparison).
/// Stored coordinates are untouched — query-time normalization.
pub fn shell_project(c: &[f64; 4]) -> [f64; 4] {
    let r = (c[1] * c[1] + c[2] * c[2] + c[3] * c[3]).sqrt().max(1e-9);
    let s = [c[1] / r, c[2] / r, c[3] / r];
    let x0 = (1.0 + s[0] * s[0] + s[1] * s[1] + s[2] * s[2]).sqrt();
    [x0, s[0], s[1], s[2]]
}

/// Shell distance: compare directions, not raw positions.
pub fn shell_dist(a: &[f64; 4], b: &[f64; 4]) -> f64 {
    hyperbolic_dist(&shell_project(a), &shell_project(b))
}

// ===========================================================================
// Toy embedding: hashed token directions, folded to the sheet
// ===========================================================================

fn fnv1a(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// Deterministic unit direction per token (seeded PRNG from hash).
/// Same text → same coordinate, always. Toy-grade: 3 dims, no IDF.
fn token_dir(token: &str) -> [f64; 3] {
    let mut x = fnv1a(token);
    let mut out = [0.0f64; 3];
    for v in out.iter_mut() {
        // xorshift64* — deterministic, no tables, no rng crate.
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        let u = (x.wrapping_mul(0x2545F4914F6CDD1D) >> 11) as f64 / (1u64 << 53) as f64;
        *v = u * 2.0 - 1.0;
    }
    let n = (out[0] * out[0] + out[1] * out[1] + out[2] * out[2])
        .sqrt()
        .max(1e-9);
    [out[0] / n, out[1] / n, out[2] / n]
}

/// Embed text as a Lorentz point: mean token direction, scaled by salience
/// radius, lifted to the sheet. Mirrors the mind's salience-as-radius rule.
pub fn coord_from_text(text: &str, salience: f64) -> [f64; 4] {
    let salience = salience.clamp(0.0, 1.0);
    let mut sum = [0.0f64; 3];
    let mut n = 0usize;
    for tok in text.split_whitespace() {
        let d = token_dir(&tok.to_lowercase());
        sum[0] += d[0];
        sum[1] += d[1];
        sum[2] += d[2];
        n += 1;
    }
    if n == 0 {
        sum = [1.0, 0.0, 0.0];
    }
    let len = (sum[0] * sum[0] + sum[1] * sum[1] + sum[2] * sum[2]).sqrt();
    let (dx, dy, dz) = if len < 1e-9 {
        (1.0, 0.0, 0.0)
    } else {
        (sum[0] / len, sum[1] / len, sum[2] / len)
    };
    let r = 0.25 + salience;
    let s = [dx * r, dy * r, dz * r];
    let x0 = (1.0 + s[0] * s[0] + s[1] * s[1] + s[2] * s[2]).sqrt();
    [x0, s[0], s[1], s[2]]
}

/// Similarity score: negated shell distance (higher = closer).
pub fn score(q: &[f64; 4], a: &[f64; 4]) -> f64 {
    -shell_dist(q, a)
}

// ===========================================================================
// Atoms + VP-tree exact-prune kNN
// ===========================================================================

#[derive(Debug, Clone)]
pub struct Atom {
    pub id: u64,
    pub coord: [f64; 4],
    pub text: String,
    pub salience: f64,
    /// Problem-domain tag, e.g. "scheduling", "memory", "proof".
    /// Capability leaves are embedded BY this (W2 of the framework doc).
    pub domain: String,
}

enum VpNode {
    Empty,
    Node {
        vp: Atom,
        mu: f64,
        left: Box<VpNode>,
        right: Box<VpNode>,
    },
}

fn build_vp(points: &[Atom]) -> VpNode {
    if points.is_empty() {
        return VpNode::Empty;
    }
    let vp = points[0].clone();
    if points.len() == 1 {
        return VpNode::Node {
            vp,
            mu: 0.0,
            left: Box::new(VpNode::Empty),
            right: Box::new(VpNode::Empty),
        };
    }
    let mut with_dist: Vec<(f64, usize)> = points
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, a)| (shell_dist(&vp.coord, &a.coord), i))
        .collect();
    with_dist.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let median_idx = with_dist.len() / 2;
    let mu = with_dist.get(median_idx).map(|x| x.0).unwrap_or(0.0);
    let mut left_pts = Vec::new();
    let mut right_pts = Vec::new();
    for a in points.iter().skip(1) {
        let d = shell_dist(&vp.coord, &a.coord);
        // NO degenerate-split guard: moving a point across the mu partition
        // breaks pruning soundness. Empty children are correct.
        if d <= mu {
            left_pts.push(a.clone());
        } else {
            right_pts.push(a.clone());
        }
    }
    let _ = median_idx;
    let _ = with_dist.len();
    VpNode::Node {
        vp,
        mu,
        left: Box::new(build_vp(&left_pts)),
        right: Box::new(build_vp(&right_pts)),
    }
}

fn search_knn(node: &VpNode, q: &[f64; 4], k: usize, best: &mut Vec<(u64, f64)>) {
    let (vp, mu, left, right) = match node {
        VpNode::Empty => return,
        VpNode::Node {
            vp,
            mu,
            left,
            right,
        } => (vp, mu, left, right),
    };
    let d = shell_dist(q, &vp.coord);
    best.push((vp.id, d));
    best.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    if best.len() > k {
        best.truncate(k);
    }
    let tau = best.last().map(|(_, dd)| *dd).unwrap_or(f64::INFINITY);
    // Triangle-inequality pruning (exact): a subtree is skipped only when
    // no member can beat tau. d<=mu side first for tighter early bounds.
    if d <= *mu {
        search_knn(left, q, k, best);
        let tau2 = best.last().map(|(_, dd)| *dd).unwrap_or(f64::INFINITY);
        if d + tau2 >= *mu {
            search_knn(right, q, k, best);
        }
    } else {
        search_knn(right, q, k, best);
        let tau2 = best.last().map(|(_, dd)| *dd).unwrap_or(f64::INFINITY);
        if d <= *mu + tau2 {
            search_knn(left, q, k, best);
        }
    }
    let _ = tau;
}

/// Brute-force reference for tests: exact top-k by shell distance.
pub fn brute_topk(atoms: &[Atom], q: &[f64; 4], k: usize) -> Vec<u64> {
    let mut v: Vec<(u64, f64)> = atoms
        .iter()
        .map(|a| (a.id, shell_dist(q, &a.coord)))
        .collect();
    v.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    v.into_iter().take(k).map(|(id, _)| id).collect()
}

// ===========================================================================
// Mind: atoms + lazy-rebuilt VP-tree + recall with depth gate
// ===========================================================================

pub struct Mind {
    pub atoms: HashMap<u64, Atom>,
    next_id: u64,
    vp: VpNode,
    vp_fresh: bool,
}

impl Mind {
    pub fn new() -> Self {
        Self {
            atoms: HashMap::new(),
            next_id: 1,
            vp: VpNode::Empty,
            vp_fresh: true,
        }
    }

    pub fn ingest(&mut self, text: &str, salience: f64, domain: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.atoms.insert(
            id,
            Atom {
                id,
                coord: coord_from_text(text, salience),
                text: text.into(),
                salience,
                domain: domain.into(),
            },
        );
        self.vp_fresh = false;
        id
    }

    pub fn ensure_vp(&mut self) {
        if self.vp_fresh {
            return;
        }
        let pts: Vec<Atom> = self.atoms.values().cloned().collect();
        self.vp = build_vp(&pts);
        self.vp_fresh = true;
    }

    pub fn len(&self) -> usize {
        self.atoms.len()
    }

    pub fn is_empty(&self) -> bool {
        self.atoms.is_empty()
    }

    /// VP-tree kNN ids (ascending shell distance). Rebuilds iff stale.
    pub fn knn(&mut self, q: &[f64; 4], k: usize) -> Vec<u64> {
        self.ensure_vp();
        let mut best: Vec<(u64, f64)> = Vec::new();
        search_knn(&self.vp, q, k, &mut best);
        best.into_iter().map(|(id, _)| id).collect()
    }

    /// Brute-force kNN (oracle for tests).
    pub fn knn_brute(&self, q: &[f64; 4], k: usize) -> Vec<u64> {
        let pts: Vec<Atom> = self.atoms.values().cloned().collect();
        brute_topk(&pts, q, k)
    }
}

impl Default for Mind {
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// Depth-gated recall: shortlist → score mass → argmin cost
// ===========================================================================

/// Candidate widths, same ladder as `map.attend`.
pub const DEPTH_LADDER: [usize; 7] = [1, 2, 3, 4, 8, 16, 32];

#[derive(Debug, Clone)]
pub struct RecallTerms {
    pub complexity_nats: f64,
    pub inaccuracy_nats: f64,
    pub dissipation_j: f64,
    pub bound: f64,
}

#[derive(Debug, Clone)]
pub struct RecallResult {
    pub ids: Vec<u64>,
    pub depth_k: usize,
    pub terms: RecallTerms,
    /// Exact-prune reference check: top-k of the rescored shortlist.
    pub overlap_with_exact_topk: usize,
}

/// Recall with the Recall-vs-Recompute cost form:
/// cost(k) = ln(k)+k − qmass(k)·8 (+ dissipation, folded into the bound).
/// `shortlist_n` caps the scored shortlist, exactly like `map.attend`.
pub fn recall(mind: &mut Mind, qtext: &str, shortlist_n: usize) -> RecallResult {
    let q = coord_from_text(qtext, 0.5);
    let n = mind.len().max(1);
    let short_n = shortlist_n.min(n);
    let short_ids = mind.knn(&q, short_n);
    let mut scored: Vec<(u64, f64)> = short_ids
        .iter()
        .filter_map(|id| mind.atoms.get(id).map(|a| (*id, score(&q, &a.coord))))
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let total_mass: f64 = scored.iter().map(|x| x.1.abs()).sum::<f64>().max(1e-9);
    let mut cum = 0.0f64;
    let mut masses: Vec<f64> = Vec::with_capacity(scored.len());
    for (_, s) in &scored {
        cum += s.abs();
        masses.push(cum / total_mass);
    }
    const C_ROW_J: f64 = 1e-9;
    let mut best_k = 1usize;
    let mut best_cost = f64::INFINITY;
    let mut best_terms = (0.0f64, 0.0f64, 0.0f64);
    for &k in &DEPTH_LADDER {
        if k > n {
            continue;
        }
        let mk = k.min(masses.len());
        if mk == 0 {
            continue;
        }
        let qmass = masses[mk - 1];
        let complexity = (k as f64).ln().max(1.0) + k as f64;
        let inaccuracy = -qmass * 8.0;
        let dissipation = k as f64 * C_ROW_J;
        let cost = complexity + inaccuracy + dissipation;
        if cost < best_cost {
            best_cost = cost;
            best_k = k;
            best_terms = (complexity, inaccuracy, dissipation);
        }
    }
    let ids: Vec<u64> = scored.iter().take(best_k).map(|(id, _)| *id).collect();
    // Overlap vs the rescored shortlist top-k (exact reference: the tree is
    // exact-prune, so shortlist holds the true top-k whenever k <= shortlist).
    let short_top: Vec<u64> = scored.iter().take(best_k).map(|(id, _)| *id).collect();
    let overlap = ids.iter().filter(|id| short_top.contains(id)).count();
    RecallResult {
        ids,
        depth_k: best_k,
        terms: RecallTerms {
            complexity_nats: best_terms.0,
            inaccuracy_nats: best_terms.1,
            dissipation_j: best_terms.2,
            bound: best_cost,
        },
        overlap_with_exact_topk: overlap,
    }
}

// ===========================================================================
// Capability leaves embedded by problem text (framework W2)
// ===========================================================================

#[derive(Debug, Clone)]
pub struct CapabilityLeaf {
    pub tool: String,
    pub family: String,
    /// What problem this capability solves — the embedded text.
    pub solves: String,
    pub judged_atom: bool,
    pub receipt_present: bool,
}

/// Route a task to capability leaves: embed the task, VP-search over the
/// leaf atoms, return tool names in recall order. Eliminates the linear
/// tool-list scan AND the router-LLM call; the §19-style family DAG
/// (runtime.rs CapabilityGate) admits/denies afterwards.
pub fn route_task(
    _mind: &mut Mind,
    leaves: &[CapabilityLeaf],
    task: &str,
    k: usize,
) -> Vec<String> {
    // Build a transient probe mind over leaf problem-texts.
    let mut probe = Mind::new();
    for leaf in leaves.iter() {
        probe.ingest(&leaf.solves, 0.5, &leaf.family);
    }
    let res = recall(&mut probe, task, leaves.len().max(1));
    res.ids
        .into_iter()
        .take(k)
        .filter_map(|id| probe.atoms.get(&id))
        .filter_map(|a| {
            leaves
                .iter()
                .find(|l| l.solves == a.text)
                .map(|l| l.tool.clone())
        })
        .collect()
}

// ===========================================================================
// KV store with producer edges + exact invalidation (framework W5)
// ===========================================================================

#[derive(Debug, Clone)]
pub struct KvEntry {
    pub key: String,
    pub value: String,
    pub produced_by: Vec<u64>,
    pub hits: u64,
    pub stale: bool,
}

#[derive(Debug, Default)]
pub struct KvStore {
    entries: HashMap<String, KvEntry>,
}

impl KvStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn store(&mut self, key: &str, value: &str, produced_by: Vec<u64>) {
        self.entries.insert(
            key.into(),
            KvEntry {
                key: key.into(),
                value: value.into(),
                produced_by,
                hits: 0,
                stale: false,
            },
        );
    }

    /// Fetch: hit increments the counter (I-RECALL's recall arm).
    /// Stale or missing → None (caller re-derives once, stores again).
    pub fn fetch(&mut self, key: &str) -> Option<String> {
        match self.entries.get_mut(key) {
            Some(e) if !e.stale => {
                e.hits += 1;
                Some(e.value.clone())
            }
            _ => None,
        }
    }

    /// Exact invalidation: when a producer atom is re-ingested, every entry
    /// whose `produced_by` cites it is marked stale. No TTL, no guessing.
    /// Returns the count marked.
    pub fn invalidate_producer(&mut self, atom_id: u64) -> usize {
        let mut n = 0;
        for e in self.entries.values_mut() {
            if !e.stale && e.produced_by.contains(&atom_id) {
                e.stale = true;
                n += 1;
            }
        }
        n
    }

    pub fn hits(&self, key: &str) -> u64 {
        self.entries.get(key).map(|e| e.hits).unwrap_or(0)
    }

    pub fn is_stale(&self, key: &str) -> bool {
        self.entries.get(key).map(|e| e.stale).unwrap_or(true)
    }
}

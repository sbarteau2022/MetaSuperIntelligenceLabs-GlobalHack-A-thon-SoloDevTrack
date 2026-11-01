//! Item 5c: real-vector RAG over HyperMind (SSD-over-port, not stdio scan).
//!
//! The mirror RAG is a tf-hash smoke collection; HyperMind query was a
//! linear scan over stdio. This client talks to the live map over HTTP
//! (SSD-backed storage, server-side index) and returns REAL vectors —
//! Poincare(3) + IDF-direction(3) + phase(2) + salience(1) — plus BM25
//! scores. No tf-hash anywhere in this path; no linear scan client-side.
//! The server does the index walk (VP-tree exact-prune); the client does
//! cosine on the returned flat rows.

use serde::{Deserialize, Serialize};

/// One real vector row from map.read_euclid: flat R^dim, precomputed
/// server-side from SSD-resident atoms. The client never tokenizes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealVector {
    pub atom_id: u64,
    pub vector: Vec<f64>,
    pub score: f64,
    pub text: String,
}

impl RealVector {
    pub fn cosine(&self, other: &[f64]) -> f64 {
        let dot: f64 = self
            .vector
            .iter()
            .zip(other.iter())
            .map(|(a, b)| a * b)
            .sum();
        let na: f64 = self
            .vector
            .iter()
            .map(|x| x * x)
            .sum::<f64>()
            .sqrt()
            .max(1e-9);
        let nb: f64 = other.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-9);
        dot / (na * nb)
    }
}

/// RAG query against the live map. Transport is HTTP (port-configured);
/// storage is the server's SSD; the index walk is server-side.
/// This struct carries the request; execution needs an HTTP client
/// (injected so unit tests stay offline — see MockMap below).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagQuery {
    pub qtext: String,
    pub k: usize,
    pub dim: usize,
}

/// Offline mock of the map for unit tests: returns canned real vectors.
/// Production injects the HTTP client (harness → :8765 /read_euclid).
pub struct MockMap {
    pub rows: Vec<RealVector>,
}

impl MockMap {
    pub fn query(&self, qtext: &str, k: usize) -> Vec<RealVector> {
        // Mock ranks by text overlap (stand-in for server BM25+geo);
        // the CONTRACT under test is vector shape + cosine math, not ranking.
        let mut scored: Vec<(&RealVector, usize)> = self
            .rows
            .iter()
            .map(|r| {
                let overlap = qtext
                    .split_whitespace()
                    .filter(|w| r.text.contains(*w))
                    .count();
                (r, overlap)
            })
            .collect();
        scored.sort_by_key(|item| std::cmp::Reverse(item.1));
        scored
            .into_iter()
            .take(k.max(1))
            .map(|(r, _)| r.clone())
            .collect()
    }
}

/// Coverage of a RAG answer: fraction of required terms present in the
/// top-k row texts. Deterministic, no model.
pub fn rag_coverage(rows: &[RealVector], required_terms: &[&str]) -> f64 {
    if required_terms.is_empty() {
        return 1.0;
    }
    let hit = required_terms
        .iter()
        .filter(|t| rows.iter().any(|r| r.text.contains(**t)))
        .count();
    hit as f64 / required_terms.len() as f64
}

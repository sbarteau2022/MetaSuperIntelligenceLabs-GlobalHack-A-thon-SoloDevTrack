//! RFC 6962 Merkle vault (backend-native, same construction as the source):
//! domain-separated leaf/node hashing, odd-node carry-forward without
//! self-duplication (CVE-2012-2459 fix).

use crate::digest::Sha256Digest;

/// Domain-separated RFC 6962 leaf hash: `SHA256(0x00 || payload)`.
pub fn merkle_hash_leaf(payload: &[u8]) -> Sha256Digest {
    let mut buf = Vec::with_capacity(1 + payload.len());
    buf.push(0x00);
    buf.extend_from_slice(payload);
    Sha256Digest::compute(&buf)
}

/// Domain-separated RFC 6962 internal node hash: `SHA256(0x01 || left || right)`.
pub fn merkle_hash_node(left: &Sha256Digest, right: &Sha256Digest) -> Sha256Digest {
    let mut buf = [0u8; 65];
    buf[0] = 0x01;
    buf[1..33].copy_from_slice(&left.0);
    buf[33..65].copy_from_slice(&right.0);
    Sha256Digest::compute(&buf)
}

pub const EMPTY_VAULT_SEED: &[u8] = b"EMPTY_FIRST_PRINCIPLES_MERKLE_VAULT";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComplianceMerkleTree {
    pub leaves: Vec<Sha256Digest>,
    pub root: Sha256Digest,
}

impl ComplianceMerkleTree {
    pub fn from_payloads(payloads: &[Vec<u8>]) -> Self {
        let leaves: Vec<Sha256Digest> = payloads.iter().map(|p| merkle_hash_leaf(p)).collect();
        Self::from_leaves(leaves)
    }

    pub fn from_leaves(leaves: Vec<Sha256Digest>) -> Self {
        if leaves.is_empty() {
            return Self {
                leaves,
                root: Sha256Digest::compute(EMPTY_VAULT_SEED),
            };
        }
        let mut current = leaves.clone();
        while current.len() > 1 {
            let mut next = Vec::with_capacity(current.len().div_ceil(2));
            let (pairs, remainder) = current.as_chunks::<2>();
            for pair in pairs {
                next.push(merkle_hash_node(&pair[0], &pair[1]));
            }
            if let Some(&rem) = remainder.first() {
                // Carry odd node forward without self-duplication (CVE-2012-2459).
                next.push(rem);
            }
            current = next;
        }
        Self {
            leaves,
            root: current[0],
        }
    }

    /// Verify that `root` recomputes from `leaves`. This is the vault check
    /// behind `GET /api/compliance/vault/:root`.
    pub fn verify(root: &Sha256Digest, leaves: &[Sha256Digest]) -> bool {
        let rebuilt = Self::from_leaves(leaves.to_vec());
        crate::digest::digest_ct_eq(&rebuilt.root, root)
    }
}

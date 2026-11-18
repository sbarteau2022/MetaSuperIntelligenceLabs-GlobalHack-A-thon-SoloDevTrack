//! Carved views + frozen weights + the atom store (backend-native equivalents
//! of `silo.rs`: `CarvedSwarmMap`, `SiloView`, `FrozenBaseWeights`).
//!
//! Backend translation (per the extraction): a role's view is a carved query
//! scope over the store — a view that does not contain a row cannot return
//! it. Atoms are SurrealDB rows in production; here the same carve logic runs
//! over the in-memory store the HTTP layer persists from. The carve, the
//! zero-copy weight sharing, and the lineage registry are the governed
//! semantics; the storage engine underneath them is interchangeable.

use crate::digest::Sha256Digest;
use crate::fixed::Q32_31;
use crate::roles::RoleId;
use std::collections::BTreeMap;
use std::sync::Arc;

pub type AtomId = u64;

/// Which basis a fingerprint claim rests on (carried from the source).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeightIntegrityBasis {
    /// Fingerprint is a content hash of the live buffer itself.
    ContentHashVerified,
    /// Fingerprint is an external checkpoint structural digest; the live
    /// buffer is not content-verified against it (fails closed).
    ExternalStructuralOnly,
}

#[derive(Debug, Clone)]
pub struct FrozenBaseWeights {
    pub model_name: String,
    pub num_layers: u32,
    /// SHA-256 fingerprint pinned at freeze time.
    pub fingerprint: Sha256Digest,
    weights: Vec<f32>,
    external_structural_fingerprint: bool,
}

impl FrozenBaseWeights {
    /// Content-hash a weight slice exactly as Gate 1 does:
    /// `num_layers` (u64 LE) followed by each f32 LE, with -0.0 → 0.0.
    pub fn hash_f32_slice(num_layers: u32, weights: &[f32]) -> Sha256Digest {
        let mut buf = Vec::with_capacity(8 + weights.len() * 4);
        buf.extend_from_slice(&(num_layers as u64).to_le_bytes());
        for &w in weights {
            let c = if w == 0.0 { 0.0f32 } else { w };
            buf.extend_from_slice(&c.to_le_bytes());
        }
        Sha256Digest::compute(&buf)
    }

    pub fn freeze(model_name: &str, num_layers: u32, weights: Vec<f32>) -> Self {
        let fingerprint = Self::hash_f32_slice(num_layers, &weights);
        Self {
            model_name: model_name.to_string(),
            num_layers,
            fingerprint,
            weights,
            external_structural_fingerprint: false,
        }
    }

    /// Freeze against an externally supplied structural fingerprint (e.g. a
    /// checkpoint digest). The live buffer is then NOT content-verified.
    pub fn freeze_with_fingerprint(
        model_name: &str,
        num_layers: u32,
        weights: Vec<f32>,
        fingerprint: Sha256Digest,
    ) -> Self {
        Self {
            model_name: model_name.to_string(),
            num_layers,
            fingerprint,
            weights,
            external_structural_fingerprint: true,
        }
    }

    pub fn verify_integrity(&self) -> bool {
        if self.external_structural_fingerprint {
            // An external digest cannot content-verify the live buffer.
            return false;
        }
        Self::hash_f32_slice(self.num_layers, &self.weights) == self.fingerprint
    }

    pub fn integrity_basis(&self) -> WeightIntegrityBasis {
        if self.external_structural_fingerprint {
            WeightIntegrityBasis::ExternalStructuralOnly
        } else {
            WeightIntegrityBasis::ContentHashVerified
        }
    }

    pub fn raw_weights(&self) -> &[f32] {
        &self.weights
    }

    /// Test/governance hook: simulate a post-freeze mutation (Gate 1's
    /// failing case). Not used by any dispatch path.
    pub fn tamper_for_test(&mut self, index: usize, value: f32) {
        if let Some(w) = self.weights.get_mut(index) {
            *w = value;
        }
    }
}

/// Zero-copy carved weight view: every silo holds the same `Arc` as the
/// frozen base. `Arc::ptr_eq` against the base is the Gate 2 check.
#[derive(Debug, Clone)]
pub struct CarvedWeightView {
    pub base: Arc<FrozenBaseWeights>,
}

/// Lorentz-style coordinate for a stored atom. Backend atoms carry a
/// hyperboloid coordinate so Gate 8 (deposit validity) and Gate 10
/// (cross-epoch) have the same shape as the source; the coordinate is
/// data, and its validity is checked, never assumed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LorentzCoord {
    pub x0: f64,
    pub x1: f64,
    pub x2: f64,
    pub x3: f64,
}

impl LorentzCoord {
    pub fn origin() -> Self {
        Self {
            x0: 1.0,
            x1: 0.0,
            x2: 0.0,
            x3: 0.0,
        }
    }

    /// Hyperboloid validity: x0² − ‖x‖² = 1 (within fixed tolerance).
    pub fn validate(&self) -> Result<(), &'static str> {
        let lhs = self.x0 * self.x0 - (self.x1 * self.x1 + self.x2 * self.x2 + self.x3 * self.x3);
        if (lhs - 1.0).abs() <= 1e-6 && self.x0 >= 1.0 {
            Ok(())
        } else {
            Err("coordinate is not on the hyperboloid (x0^2 - ||x||^2 != 1)")
        }
    }
}

#[derive(Debug, Clone)]
pub struct Atom {
    pub id: AtomId,
    pub coord: LorentzCoord,
    pub payload: Vec<u8>,
    /// Basin (home) this atom belongs to, if carved.
    pub home_role: Option<RoleId>,
}

#[derive(Debug, Clone)]
pub struct SiloView {
    pub role: RoleId,
    pub weight_view: CarvedWeightView,
    atoms: BTreeMap<AtomId, LorentzCoord>,
}

impl SiloView {
    pub fn contains_atom(&self, id: AtomId) -> bool {
        self.atoms.contains_key(&id)
    }

    pub fn atom_count(&self) -> usize {
        self.atoms.len()
    }

    pub fn atom_ids(&self) -> impl Iterator<Item = AtomId> + '_ {
        self.atoms.keys().copied()
    }
}

/// A worker-facing borrowed read capability. Its private map reference must
/// never be handed to the worker directly; every read checks carved membership.
pub struct ScopedGraphRead<'a> {
    map: &'a CarvedSwarmMap,
    silo: &'a SiloView,
}

impl ScopedGraphRead<'_> {
    pub fn role(&self) -> RoleId {
        self.silo.role
    }

    pub fn atom(&self, id: AtomId) -> Option<&Atom> {
        self.silo
            .contains_atom(id)
            .then(|| self.map.global_atom(id))
            .flatten()
    }

    /// A graph edge may be traversed only if both endpoints are in the same
    /// worker view. The authoritative edge existence check belongs to SurrealDB.
    pub fn permits_endpoints(&self, from: AtomId, to: AtomId) -> bool {
        self.atom(from).is_some() && self.atom(to).is_some()
    }

    pub fn weights(&self) -> &Arc<FrozenBaseWeights> {
        &self.silo.weight_view.base
    }
}

/// Loss reports (Gate 4 / Gate 6 shapes, backend-native field names).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectionLossReport {
    pub input_state_bits: u64,
    pub output_tangent_bits: u64,
    pub discarded_dimensions: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RefractionLossReport {
    pub refractive_index_q32: Q32_31,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CarveLossReport {
    pub global_atom_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BridgeLossReport {
    pub input_dimensions: u64,
    pub rope_pair_integrity_q32: Q32_31,
    pub retained_5d_energy_ratio_q32: Q32_31,
}

#[derive(Debug, Clone)]
pub struct AdapterLossReport {
    pub origin_fingerprint: Sha256Digest,
    pub target_fingerprint: Sha256Digest,
    pub source_head_dim: u32,
    pub target_head_dim: u32,
    pub rope_pair_integrity_q32: Q32_31,
    pub retained_k_energy_ratio_q32: Q32_31,
    pub retained_v_energy_ratio_q32: Q32_31,
    pub is_zero_copy_passthrough: bool,
}

#[derive(Debug, Clone)]
pub struct EphemeralMapDeposit {
    pub from_role: RoleId,
    pub dissolved_pid: u64,
    pub records: Vec<LorentzCoord>,
    pub forward_steps_completed: u64,
}

/// The carved store: one global atom map + 12 carved views + frozen base +
/// model lineage registry.
#[derive(Debug, Clone)]
pub struct CarvedSwarmMap {
    pub base_weights: Arc<FrozenBaseWeights>,
    pub default_basin_cosh: Q32_31,
    global: BTreeMap<AtomId, Atom>,
    silos: BTreeMap<RoleId, SiloView>,
    model_lineage: Vec<Sha256Digest>,
    next_atom_id: u64,
}

impl CarvedSwarmMap {
    pub fn new(base_weights: FrozenBaseWeights, default_basin_cosh: Q32_31) -> Self {
        let base = Arc::new(base_weights);
        let mut silos = BTreeMap::new();
        for role in RoleId::CARVED_12 {
            silos.insert(
                role,
                SiloView {
                    role,
                    weight_view: CarvedWeightView {
                        base: Arc::clone(&base),
                    },
                    atoms: BTreeMap::new(),
                },
            );
        }
        let lineage = vec![base.fingerprint];
        Self {
            base_weights: base,
            default_basin_cosh,
            global: BTreeMap::new(),
            silos,
            model_lineage: lineage,
            next_atom_id: 1,
        }
    }

    pub fn silo(&self, role: RoleId) -> Option<&SiloView> {
        self.silos.get(&role)
    }

    /// L0 retains its separate governor authority; workers get only this
    /// carved reader, not a reference to the global map.
    pub fn scoped_read(&self, role: RoleId) -> Option<ScopedGraphRead<'_>> {
        self.silos
            .get(&role)
            .map(|silo| ScopedGraphRead { map: self, silo })
    }

    pub fn global_atom(&self, id: AtomId) -> Option<&Atom> {
        self.global.get(&id)
    }

    pub fn global_atom_count(&self) -> usize {
        self.global.len()
    }

    pub fn is_registered_model_fingerprint(&self, fp: &Sha256Digest) -> bool {
        self.model_lineage.iter().any(|f| f == fp)
    }

    pub fn register_model_fingerprint(&mut self, fp: Sha256Digest) {
        if !self.is_registered_model_fingerprint(&fp) {
            self.model_lineage.push(fp);
        }
    }

    /// Insert an atom into the global map and carve it into the view of
    /// `home` plus every neighbour view whose overlap allowance under
    /// `policy` is positive. Returns the new atom id.
    pub fn keep_and_carve(
        &mut self,
        coord: LorentzCoord,
        payload: Vec<u8>,
        home: RoleId,
        policy: &crate::compliance::OverlapPolicyMatrix,
    ) -> AtomId {
        let id = self.next_atom_id;
        self.next_atom_id += 1;
        self.global.insert(
            id,
            Atom {
                id,
                coord,
                payload,
                home_role: Some(home),
            },
        );
        self.carve_atom(id, coord, home, policy);
        id
    }

    fn carve_atom(
        &mut self,
        id: AtomId,
        coord: LorentzCoord,
        home: RoleId,
        policy: &crate::compliance::OverlapPolicyMatrix,
    ) {
        for role in RoleId::CARVED_12 {
            let visible = role == home || policy.overlap_allowance(home, role).0 > 0;
            if visible {
                if let Some(silo) = self.silos.get_mut(&role) {
                    silo.atoms.insert(id, coord);
                }
            }
        }
        // The coordinator holds the undivided map by definition; views are
        // the 12 carved scopes. Coordinator access goes through `global`.
    }

    /// Re-carve all 12 views from the global map under `policy`
    /// (used after erasure and after policy expansion).
    pub fn recarve_all_twelve(
        &mut self,
        policy: &crate::compliance::OverlapPolicyMatrix,
    ) -> CarveLossReport {
        for silo in self.silos.values_mut() {
            silo.atoms.clear();
        }
        let atoms: Vec<(AtomId, LorentzCoord, Option<RoleId>)> = self
            .global
            .values()
            .map(|a| (a.id, a.coord, a.home_role))
            .collect();
        for (id, coord, home) in atoms {
            if let Some(home) = home {
                self.carve_atom(id, coord, home, policy);
            }
        }
        CarveLossReport {
            global_atom_count: self.global.len(),
        }
    }

    pub fn remove_global_atom_internal(&mut self, id: AtomId) -> bool {
        self.global.remove(&id).is_some()
    }

    /// Deterministic leaf bytes for the swarm Merkle root: for each atom in
    /// id order, `id (u64 LE) || coord (4×f64 LE) || payload`.
    pub fn collect_merkle_leaf_bytes(&self) -> Vec<Vec<u8>> {
        self.global
            .values()
            .map(|a| {
                let mut buf = Vec::with_capacity(8 + 32 + a.payload.len());
                buf.extend_from_slice(&a.id.to_le_bytes());
                buf.extend_from_slice(&a.coord.x0.to_le_bytes());
                buf.extend_from_slice(&a.coord.x1.to_le_bytes());
                buf.extend_from_slice(&a.coord.x2.to_le_bytes());
                buf.extend_from_slice(&a.coord.x3.to_le_bytes());
                buf.extend_from_slice(&a.payload);
                buf
            })
            .collect()
    }

    pub fn compute_swarm_merkle_root(&self) -> Sha256Digest {
        crate::merkle::ComplianceMerkleTree::from_payloads(&self.collect_merkle_leaf_bytes()).root
    }
}

//! VP-Tree (Vantage-Point Tree) on H³ for k-NN Search
//!
//! Implements geometric nearest neighbor search using hyperbolic distance.
//! VP-tree provides O(log N) expected retrieval under bounded expansion constant.

use super::hyperbolic::{vptree_distance, AtomCoord, LorentzVec};
use parking_lot::RwLock;
use std::sync::Arc;

/// VP-Tree node
#[derive(Debug)]
pub struct VpTreeNode {
    /// Vantage point (the pivot)
    pub vantage: AtomCoord,
    /// Median distance threshold
    pub threshold: i64,
    /// Left child (distance ≤ threshold)
    pub left: Option<Arc<RwLock<VpTreeNode>>>,
    /// Right child (distance > threshold)
    pub right: Option<Arc<RwLock<VpTreeNode>>>,
    /// Leaf atoms (for small nodes)
    pub leaf_atoms: Vec<AtomCoord>,
    /// Is leaf node
    pub is_leaf: bool,
}

impl VpTreeNode {
    /// Create leaf node
    pub fn new_leaf(atoms: Vec<AtomCoord>) -> Self {
        Self {
            vantage: atoms[0],
            threshold: 0,
            left: None,
            right: None,
            leaf_atoms: atoms,
            is_leaf: true,
        }
    }

    /// Create internal node
    pub fn new_internal(
        vantage: AtomCoord,
        threshold: i64,
        left: Option<Arc<RwLock<VpTreeNode>>>,
        right: Option<Arc<RwLock<VpTreeNode>>>,
    ) -> Self {
        Self {
            vantage,
            threshold,
            left,
            right,
            leaf_atoms: Vec::new(),
            is_leaf: false,
        }
    }
}

/// VP-Tree for H³ nearest neighbor search
pub struct VpTree {
    root: Option<Arc<RwLock<VpTreeNode>>>,
    leaf_size: usize,
}

impl VpTree {
    pub fn new(leaf_size: usize) -> Self {
        Self {
            root: None,
            leaf_size,
        }
    }

    /// Build VP-tree from atoms
    pub fn build(&mut self, mut atoms: Vec<AtomCoord>) {
        if atoms.is_empty() {
            return;
        }
        self.root = Some(Arc::new(RwLock::new(self.build_recursive(&mut atoms))));
    }

    fn build_recursive(&self, atoms: &mut [AtomCoord]) -> VpTreeNode {
        if atoms.len() <= self.leaf_size {
            return VpTreeNode::new_leaf(atoms.to_vec());
        }

        // Select vantage point (median by radial coordinate for geometric eviction alignment)
        atoms.sort_by_key(|a| a.h3.radial_coord());
        let mid = atoms.len() / 2;
        let vantage = atoms[mid];

        // Compute distances to vantage
        let mut distances: Vec<(i64, usize)> = atoms
            .iter()
            .enumerate()
            .map(|(i, a)| (vptree_distance(&a.h3, &vantage.h3), i))
            .collect();

        distances.sort_by_key(|d| d.0);
        let median_dist = distances[mid].0;

        // Partition
        let mut left_atoms = Vec::new();
        let mut right_atoms = Vec::new();

        for (dist, idx) in distances {
            if idx == mid {
                continue; // Skip vantage point itself
            }
            if dist <= median_dist {
                left_atoms.push(atoms[idx]);
            } else {
                right_atoms.push(atoms[idx]);
            }
        }

        let left = if left_atoms.is_empty() {
            None
        } else {
            Some(Arc::new(RwLock::new(self.build_recursive(&mut left_atoms))))
        };

        let right = if right_atoms.is_empty() {
            None
        } else {
            Some(Arc::new(RwLock::new(
                self.build_recursive(&mut right_atoms),
            )))
        };

        VpTreeNode::new_internal(vantage, median_dist, left, right)
    }

    /// k-NN search
    pub fn knn_search(&self, query: &AtomCoord, k: usize) -> Vec<(AtomCoord, i64)> {
        let mut results = Vec::new();
        if let Some(root) = &self.root {
            self.knn_recursive(root.clone(), query, k, &mut results);
        }
        results.sort_by_key(|(_, d)| *d);
        results.truncate(k);
        results
    }

    fn knn_recursive(
        &self,
        node: Arc<RwLock<VpTreeNode>>,
        query: &AtomCoord,
        k: usize,
        results: &mut Vec<(AtomCoord, i64)>,
    ) {
        let node_read = node.read();

        if node_read.is_leaf {
            // Linear scan leaf
            for atom in &node_read.leaf_atoms {
                let dist = vptree_distance(&query.h3, &atom.h3);
                Self::insert_result(results, atom.clone(), dist, k);
            }
            return;
        }

        // Distance to vantage
        let dist_to_vantage = vptree_distance(&query.h3, &node_read.vantage.h3);
        Self::insert_result(results, node_read.vantage.clone(), dist_to_vantage, k);

        // Determine which child to search first
        let search_left_first = dist_to_vantage <= node_read.threshold;

        let (first_child, second_child) = if search_left_first {
            (&node_read.left, &node_read.right)
        } else {
            (&node_read.right, &node_read.left)
        };

        // Search first child
        if let Some(child) = first_child {
            self.knn_recursive(child.clone(), query, k, results);
        }

        // Check if we need to search second child
        // Need to search if hypersphere intersects the splitting plane
        let mut need_second = false;
        if let Some(worst_dist) = results.last().map(|(_, d)| *d) {
            if results.len() < k || (dist_to_vantage - node_read.threshold).abs() <= worst_dist {
                need_second = true;
            }
        } else {
            need_second = true;
        }

        if need_second {
            if let Some(child) = second_child {
                self.knn_recursive(child.clone(), query, k, results);
            }
        }
    }

    fn insert_result(results: &mut Vec<(AtomCoord, i64)>, atom: AtomCoord, dist: i64, k: usize) {
        // Insert maintaining sorted order by distance (ascending)
        let pos = results
            .binary_search_by_key(&dist, |(_, d)| *d)
            .unwrap_or_else(|e| e);
        results.insert(pos, (atom, dist));
        if results.len() > k {
            results.pop();
        }
    }

    /// Insert new atom (for dynamic updates)
    pub fn insert(&mut self, atom: AtomCoord) {
        if self.root.is_none() {
            self.root = Some(Arc::new(RwLock::new(VpTreeNode::new_leaf(vec![atom]))));
            return;
        }

        self.insert_recursive(self.root.as_ref().unwrap().clone(), atom);
    }

    fn insert_recursive(&self, node: Arc<RwLock<VpTreeNode>>, atom: AtomCoord) {
        // First check if leaf without holding lock across recursion
        let is_leaf = node.read().is_leaf;

        if is_leaf {
            let mut node_write = node.write();
            node_write.leaf_atoms.push(atom);
            // Split if leaf gets too large
            if node_write.leaf_atoms.len() > self.leaf_size * 2 {
                let atoms = std::mem::take(&mut node_write.leaf_atoms);
                *node_write = self.build_recursive(&mut atoms.clone());
            }
            return;
        }

        // For internal nodes, get distance and child reference
        let (dist, threshold, has_left, has_right) = {
            let node_read = node.read();
            let dist = vptree_distance(&atom.h3, &node_read.vantage.h3);
            (
                dist,
                node_read.threshold,
                node_read.left.is_some(),
                node_read.right.is_some(),
            )
        };

        if dist <= threshold {
            if has_left {
                let left = node.read().left.as_ref().unwrap().clone();
                self.insert_recursive(left, atom);
            } else {
                let mut node_write = node.write();
                node_write.left = Some(Arc::new(RwLock::new(VpTreeNode::new_leaf(vec![atom]))));
            }
        } else {
            if has_right {
                let right = node.read().right.as_ref().unwrap().clone();
                self.insert_recursive(right, atom);
            } else {
                let mut node_write = node.write();
                node_write.right = Some(Arc::new(RwLock::new(VpTreeNode::new_leaf(vec![atom]))));
            }
        }
    }

    /// Evict atom with maximum radial coordinate (x0) among unpinned
    /// Returns evicted atom ID if found
    pub fn evict_max_radial(&mut self) -> Option<u64> {
        if self.root.is_none() {
            return None;
        }

        // Find candidate
        let mut max_radial = i64::MIN;
        let mut candidate_id = None;
        self.find_max_radial(
            self.root.as_ref().unwrap().clone(),
            &mut max_radial,
            &mut candidate_id,
        );

        if let Some(id) = candidate_id {
            self.remove_by_id(self.root.as_ref().unwrap().clone(), id);
            Some(id)
        } else {
            None
        }
    }

    fn find_max_radial(
        &self,
        node: Arc<RwLock<VpTreeNode>>,
        max_radial: &mut i64,
        candidate_id: &mut Option<u64>,
    ) {
        let node_read = node.read();

        if node_read.is_leaf {
            for atom in &node_read.leaf_atoms {
                if atom.pinned == 0 && atom.h3.radial_coord() > *max_radial {
                    *max_radial = atom.h3.radial_coord();
                    *candidate_id = Some(atom.atom_id);
                }
            }
            return;
        }

        if node_read.vantage.pinned == 0 && node_read.vantage.h3.radial_coord() > *max_radial {
            *max_radial = node_read.vantage.h3.radial_coord();
            *candidate_id = Some(node_read.vantage.atom_id);
        }

        if let Some(left) = &node_read.left {
            self.find_max_radial(left.clone(), max_radial, candidate_id);
        }
        if let Some(right) = &node_read.right {
            self.find_max_radial(right.clone(), max_radial, candidate_id);
        }
    }

    fn remove_by_id(&self, node: Arc<RwLock<VpTreeNode>>, id: u64) -> bool {
        let mut node_write = node.write();

        if node_write.is_leaf {
            let len_before = node_write.leaf_atoms.len();
            node_write.leaf_atoms.retain(|a| a.atom_id != id);
            return node_write.leaf_atoms.len() < len_before;
        }

        if node_write.vantage.atom_id == id {
            // For simplicity, mark as "removed" (in production, rebuild subtree)
            node_write.vantage.pinned = 2; // removed marker
            return true;
        }

        let mut found = false;
        if let Some(left) = &node_write.left {
            found |= self.remove_by_id(left.clone(), id);
        }
        if let Some(right) = &node_write.right {
            found |= self.remove_by_id(right.clone(), id);
        }
        found
    }

    /// Get approximate size
    pub fn size(&self) -> usize {
        self.count_nodes(self.root.as_ref())
    }

    fn count_nodes(&self, node: Option<&Arc<RwLock<VpTreeNode>>>) -> usize {
        if let Some(n) = node {
            let n_read = n.read();
            let mut count = if n_read.is_leaf {
                n_read.leaf_atoms.len()
            } else {
                1
            };
            count += self.count_nodes(n_read.left.as_ref());
            count += self.count_nodes(n_read.right.as_ref());
            count
        } else {
            0
        }
    }
}

/// Batch VP-tree builder for cutile-rs Tile kernels
/// This is the host-side orchestration; actual distance computations run on GPU
pub struct BatchVpTreeBuilder {
    atoms: Vec<AtomCoord>,
    leaf_size: usize,
}

impl BatchVpTreeBuilder {
    pub fn new(leaf_size: usize) -> Self {
        Self {
            atoms: Vec::new(),
            leaf_size,
        }
    }

    pub fn add_atom(&mut self, atom: AtomCoord) {
        self.atoms.push(atom);
    }

    pub fn build(self) -> VpTree {
        let mut tree = VpTree::new(self.leaf_size);
        tree.build(self.atoms);
        tree
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hyperbolic::{random_lorentz_vec, AtomCoord, ToroidalPhase};
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_vptree_build_and_search() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut atoms = Vec::new();

        for i in 0..1000 {
            let h3 = random_lorentz_vec(&mut rng, 10.0);
            let t2 = ToroidalPhase::from_lorentz(&h3);
            atoms.push(AtomCoord::new(h3, t2, i, false));
        }

        let mut tree = VpTree::new(32);
        tree.build(atoms);

        assert!(tree.size() > 0);

        // Search for nearest to first atom
        let query = tree.root.as_ref().unwrap().read().vantage.clone();
        let results = tree.knn_search(&query, 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].0.atom_id, query.atom_id);
        assert_eq!(results[0].1, 0); // distance to self = 0
    }

    #[test]
    fn test_vptree_evict() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut atoms = Vec::new();

        for i in 0..100 {
            let h3 = random_lorentz_vec(&mut rng, 100.0);
            let t2 = ToroidalPhase::from_lorentz(&h3);
            atoms.push(AtomCoord::new(h3, t2, i, false));
        }

        let mut tree = VpTree::new(16);
        tree.build(atoms);

        // Evict should return the atom with highest x0
        let evicted = tree.evict_max_radial();
        assert!(evicted.is_some());
    }
}

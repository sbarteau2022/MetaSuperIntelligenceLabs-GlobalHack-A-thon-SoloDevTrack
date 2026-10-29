//! Workload Configuration and Runner for FW-A1-CUDA
//!
//! Implements the mixed insert/knn_search/evict/picard_reduce workload
//! on H³×T² manifold with Whitehead 12-vertex structure.

use super::hyperbolic::{f64_to_q32_31, random_lorentz_vec, AtomCoord, LorentzVec, ToroidalPhase};
use super::picard::PicardReducer;
use super::toroidal::random_toroidal_phase;
use super::vptree::VpTree;
use parking_lot::RwLock;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use std::sync::Arc;
use std::time::Instant;

/// Workload configuration
#[derive(Debug, Clone)]
pub struct WorkloadConfig {
    pub num_vectors: usize,
    pub h3_dim: usize, // 3 (Lorentz coordinates)
    pub t2_dim: usize, // 2 (toroidal phase)
    pub fixed_point: &'static str,
    pub insert_count: usize,
    pub knn_count: usize,
    pub knn_k: usize,
    pub evict_count: usize,
    pub picard_count: usize,
    pub picard_iterations: u32,
    pub iterations: usize,
    pub warmup_iterations: usize,
    pub seed: u64,
    pub leaf_size: usize,
}

impl Default for WorkloadConfig {
    fn default() -> Self {
        Self {
            num_vectors: 1_000_000,
            h3_dim: 3,
            t2_dim: 2,
            fixed_point: "Q32.31",
            insert_count: 1_000_000,
            knn_count: 100_000,
            knn_k: 32,
            evict_count: 100_000,
            picard_count: 10_000,
            picard_iterations: 32,
            iterations: 1000,
            warmup_iterations: 100,
            seed: 42,
            leaf_size: 32,
        }
    }
}

/// Single iteration result
#[derive(Debug, Clone, Copy)]
pub struct WorkloadResult {
    pub insert_throughput: f64, // M vec/s
    pub knn_throughput: f64,    // M vec/s
    pub evict_throughput: f64,  // M vec/s
    pub picard_throughput: f64, // ops/s
    pub peak_memory_gb: f64,
    pub total_time_ms: f64,
}

/// Workload runner
pub struct WorkloadRunner {
    config: WorkloadConfig,
    tree: Arc<RwLock<VpTree>>,
    atoms: Vec<AtomCoord>,
    picard_reducer: PicardReducer,
    rng: StdRng,
    atom_counter: u64,
}

impl WorkloadRunner {
    pub fn new(config: WorkloadConfig) -> Self {
        let seed = config.seed;
        let tree = Arc::new(RwLock::new(VpTree::new(config.leaf_size)));
        let picard_reducer = PicardReducer::new().expect("Failed to create Picard reducer");

        Self {
            config,
            tree,
            atoms: Vec::new(),
            picard_reducer,
            rng: StdRng::seed_from_u64(seed),
            atom_counter: 0,
        }
    }

    /// Run a single benchmark iteration
    pub fn run_iteration(&mut self, iteration: usize) -> WorkloadResult {
        let iter_start = Instant::now();

        // Re-seed for reproducibility per iteration
        self.rng = StdRng::seed_from_u64(self.config.seed.wrapping_add(iteration as u64));

        // 1. Insert workload
        let insert_start = Instant::now();
        self.run_inserts();
        let insert_time = insert_start.elapsed().as_secs_f64();
        let insert_throughput = self.config.insert_count as f64 / insert_time / 1_000_000.0;

        // 2. k-NN search workload
        let knn_start = Instant::now();
        self.run_knn_search();
        let knn_time = knn_start.elapsed().as_secs_f64();
        let knn_throughput = self.config.knn_count as f64 / knn_time / 1_000_000.0;

        // 3. Evict workload
        let evict_start = Instant::now();
        self.run_evicts();
        let evict_time = evict_start.elapsed().as_secs_f64();
        let evict_throughput = self.config.evict_count as f64 / evict_time / 1_000_000.0;

        // 4. Picard reduce workload
        let picard_start = Instant::now();
        self.run_picard_reduce();
        let picard_time = picard_start.elapsed().as_secs_f64();
        let picard_throughput = self.config.picard_count as f64 / picard_time;

        let total_time = iter_start.elapsed().as_secs_f64() * 1000.0;

        // Estimate peak memory (simplified)
        let peak_memory_gb = self.estimate_memory_gb();

        WorkloadResult {
            insert_throughput,
            knn_throughput,
            evict_throughput,
            picard_throughput,
            peak_memory_gb,
            total_time_ms: total_time,
        }
    }

    fn run_inserts(&mut self) {
        let mut tree = self.tree.write();

        for _ in 0..self.config.insert_count {
            let h3 = random_lorentz_vec(&mut self.rng, 100.0);
            let t2 = ToroidalPhase::from_lorentz(&h3);
            let atom = AtomCoord::new(h3, t2, self.atom_counter, false);
            self.atom_counter += 1;

            tree.insert(atom);
        }
    }

    fn run_knn_search(&mut self) {
        let tree = self.tree.read();

        for _ in 0..self.config.knn_count {
            // Generate random query
            let h3 = random_lorentz_vec(&mut self.rng, 100.0);
            let t2 = ToroidalPhase::from_lorentz(&h3);
            let query = AtomCoord::new(h3, t2, u64::MAX, false);

            let _results = tree.knn_search(&query, self.config.knn_k);
        }
    }

    fn run_evicts(&mut self) {
        let mut tree = self.tree.write();

        for _ in 0..self.config.evict_count {
            let _ = tree.evict_max_radial();
        }
    }

    fn run_picard_reduce(&mut self) {
        // Collect atoms for Picard reduction
        let mut picard_atoms: Vec<LorentzVec> = Vec::with_capacity(self.config.picard_count);

        // Sample atoms from tree (simplified: just generate new ones with high x0)
        for _ in 0..self.config.picard_count {
            let h3 = random_lorentz_vec(&mut self.rng, 1000.0); // High x0 for Picard
            picard_atoms.push(h3);
        }

        // Run Picard reduction (CPU fallback for now)
        PicardReducer::reduce_cpu_fallback(&mut picard_atoms, self.config.picard_iterations);
    }

    fn estimate_memory_gb(&self) -> f64 {
        let tree = self.tree.read();
        let num_nodes = tree.size();

        // Rough estimate: AtomCoord ≈ 64 bytes, VP-tree overhead ~2x
        let atom_mem = num_nodes * 64;
        let overhead = atom_mem; // VP-tree pointers, etc.
        let total_bytes = atom_mem + overhead;

        total_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }
}

/// Generate initial dataset for benchmark
pub fn generate_initial_dataset(config: &WorkloadConfig) -> Vec<AtomCoord> {
    let mut rng = StdRng::seed_from_u64(config.seed);
    let mut atoms = Vec::with_capacity(config.num_vectors);

    for i in 0..config.num_vectors {
        let h3 = random_lorentz_vec(&mut rng, 100.0);
        let t2 = ToroidalPhase::from_lorentz(&h3);
        let pinned = i < 1000; // Pin first 1000 atoms
        atoms.push(AtomCoord::new(h3, t2, i as u64, pinned));
    }

    atoms
}

/// Build initial VP-tree from dataset
pub fn build_initial_tree(atoms: Vec<AtomCoord>, leaf_size: usize) -> VpTree {
    let mut tree = VpTree::new(leaf_size);
    tree.build(atoms);
    tree
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workload_config_default() {
        let config = WorkloadConfig::default();
        assert_eq!(config.num_vectors, 1_000_000);
        assert_eq!(config.insert_count, 1_000_000);
        assert_eq!(config.knn_count, 100_000);
        assert_eq!(config.knn_k, 32);
    }

    #[test]
    fn test_generate_dataset() {
        let config = WorkloadConfig {
            num_vectors: 1000,
            ..Default::default()
        };
        let atoms = generate_initial_dataset(&config);
        assert_eq!(atoms.len(), 1000);
        assert!(atoms[0].pinned == 1);
        assert!(atoms[999].pinned == 1);
    }

    #[test]
    fn test_workload_runner() {
        let config = WorkloadConfig {
            num_vectors: 1000,
            insert_count: 100,
            knn_count: 10,
            evict_count: 10,
            picard_count: 10,
            iterations: 1,
            warmup_iterations: 0,
            ..Default::default()
        };

        let mut runner = WorkloadRunner::new(config);
        let result = runner.run_iteration(0);

        assert!(result.insert_throughput > 0.0);
        assert!(result.knn_throughput > 0.0);
        assert!(result.evict_throughput > 0.0);
        assert!(result.picard_throughput > 0.0);
    }
}

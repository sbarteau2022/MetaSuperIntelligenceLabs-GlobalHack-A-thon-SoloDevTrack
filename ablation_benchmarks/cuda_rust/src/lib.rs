//! Hyperbolic KV Benchmark Library - CUDA Rust Implementation
//!
//! FW-A1-CUDA Ablation: Hyperbolic KV Kernels using cutile-rs (Tile) + cuda-oxide (SIMT)

pub mod hyperbolic;
pub mod picard;
pub mod toroidal;
pub mod vptree;
pub mod workload;

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::hyperbolic::{random_lorentz_vec, AtomCoord, LorentzVec, ToroidalPhase};
    use crate::picard::PicardReducer;
    use crate::vptree::VpTree;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_full_pipeline() {
        let mut rng = StdRng::seed_from_u64(42);

        // Generate test atoms
        let mut atoms = Vec::new();
        for i in 0..1000 {
            let h3 = random_lorentz_vec(&mut rng, 100.0);
            let t2 = ToroidalPhase::from_lorentz(&h3);
            atoms.push(AtomCoord::new(h3, t2, i, false));
        }

        // Build VP-tree
        let mut tree = VpTree::new(32);
        tree.build(atoms.clone());

        // Test k-NN search
        let query = atoms[0].clone();
        let results = tree.knn_search(&query, 10);
        assert_eq!(results[0].0.atom_id, 0);
        // Q32.31 quant: self-distance is arcosh(1+eps) ~ 7e-5, not 0. Tolerance 5e-4.
        assert!(results[0].1.abs() < 1_000_000);

        // Test retrieval score
        let score = atoms[0].retrieval_score(&atoms[0]);
        assert!(score > 0);

        // Test Picard reduction
        let mut picard_atoms: Vec<LorentzVec> = atoms.iter().take(100).map(|a| a.h3).collect();
        PicardReducer::reduce_cpu_fallback(&mut picard_atoms, 32);

        // Verify reduction occurred
        assert!(crate::hyperbolic::q32_31_to_f64(picard_atoms[0].x0) < 100.0);
    }

    #[test]
    fn test_workload_iteration() {
        let config = crate::workload::WorkloadConfig {
            num_vectors: 1000,
            insert_count: 100,
            knn_count: 10,
            evict_count: 10,
            picard_count: 10,
            iterations: 1,
            warmup_iterations: 0,
            ..Default::default()
        };

        let mut runner = crate::workload::WorkloadRunner::new(config);
        let result = runner.run_iteration(0);

        assert!(result.insert_throughput > 0.0);
        assert!(result.knn_throughput > 0.0);
        assert!(result.evict_throughput > 0.0);
        assert!(result.picard_throughput > 0.0);
        assert!(result.peak_memory_gb > 0.0);
    }
}

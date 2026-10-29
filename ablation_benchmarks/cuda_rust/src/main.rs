//! Hyperbolic KV Benchmark - CUDA Rust Implementation
//!
//! Implements the FW-A1-CUDA workload: mixed insert/knn_search/evict/picard_reduce
//! on H³×T² manifold with Whitehead 12-vertex structure (Picard fundamental domain Γ₁₂).
//!
//! Uses cutile-rs (Tile track) for VP-tree operations and cuda-oxide (SIMT track)
//! for Picard reduction warp reductions.

#![cfg_attr(feature = "simt-track", feature(asm_experimental_arch))]

mod hyperbolic;
mod vptree;
mod picard;
mod toroidal;
mod bench;
mod workload;

use std::sync::Arc;
use std::time::Instant;
use workload::{WorkloadConfig, WorkloadRunner, WorkloadResult};

fn main() {
    println!("=== Hyperbolic KV Benchmark (CUDA Rust) ===");
    println!("FW-A1-CUDA Ablation Workload");
    println!("Vectors: 1,000,000 | Dimensions: H³(3) + T²(2) | Fixed-point: Q32.31");
    println!();

    let config = WorkloadConfig::default();
    println!("Configuration: {:?}", config);
    println!();

    let mut runner = WorkloadRunner::new(config.clone());

    // Warmup
    println!("Running warmup ({} iterations)...", config.warmup_iterations);
    for i in 0..config.warmup_iterations {
        let _ = runner.run_iteration(i);
        if i % 20 == 0 {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).ok();
        }
    }
    println!("\nWarmup complete.\n");

    // Benchmark iterations
    println!("Running benchmark ({} iterations)...", config.iterations);
    let start = Instant::now();
    let mut results = Vec::with_capacity(config.iterations);

    for i in 0..config.iterations {
        let result = runner.run_iteration(i);
        results.push(result);

        if i % 100 == 0 {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).ok();
        }
    }
    let elapsed = start.elapsed();
    println!("\nBenchmark complete in {:.2}s", elapsed.as_secs_f64());
    println!();

    // Aggregate results
    let agg = aggregate_results(&results);
    print_results(&agg, elapsed);

    // Write raw data for analysis (binary format per FW-1)
    write_raw_data(&results, &agg);
}

fn aggregate_results(results: &[WorkloadResult]) -> AggregatedResults {
    let n = results.len() as f64;

    let insert_throughput: f64 = results.iter().map(|r| r.insert_throughput).sum::<f64>() / n;
    let knn_throughput: f64 = results.iter().map(|r| r.knn_throughput).sum::<f64>() / n;
    let evict_throughput: f64 = results.iter().map(|r| r.evict_throughput).sum::<f64>() / n;
    let picard_throughput: f64 = results.iter().map(|r| r.picard_throughput).sum::<f64>() / n;

    let peak_memory_gb: f64 = results.iter().map(|r| r.peak_memory_gb).sum::<f64>() / n;
    let total_time_ms: f64 = results.iter().map(|r| r.total_time_ms).sum::<f64>() / n;

    AggregatedResults {
        insert_throughput_mvs: insert_throughput,
        knn_throughput_mvs: knn_throughput,
        evict_throughput_mvs: evict_throughput,
        picard_throughput_kops: picard_throughput / 1000.0,
        peak_memory_gb,
        avg_total_time_ms: total_time_ms,
        iterations: results.len(),
    }
}

fn print_results(agg: &AggregatedResults, elapsed: std::time::Duration) {
    println!("=== AGGREGATED RESULTS ===");
    println!("Insert throughput:     {:.3} M vec/s", agg.insert_throughput_mvs);
    println!("k-NN search throughput: {:.3} M vec/s", agg.knn_throughput_mvs);
    println!("Evict throughput:      {:.3} M vec/s", agg.evict_throughput_mvs);
    println!("Picard reduce throughput: {:.3} K ops/s", agg.picard_throughput_kops);
    println!("Peak memory:           {:.3} GB", agg.peak_memory_gb);
    println!("Avg iteration time:    {:.3} ms", agg.avg_total_time_ms);
    println!("Total benchmark time:  {:.2} s", elapsed.as_secs_f64());
    println!("Iterations:            {}", agg.iterations);
    println!();

    // Judge criteria (from FW-A1-CUDA.yaml)
    println!("=== JUDGE CRITERIA (FW-A1-CUDA) ===");
    println!("Target: CUDA Rust throughput / C++ throughput ≥ 0.95 (parity)");
    println!("Target: CUDA Rust peak memory / C++ peak memory ≤ 1.05");
    println!();
    println!("NOTE: C++ baseline results required for comparison.");
    println!("Run C++ baseline (FW-A1) on same hardware for judge evaluation.");
}

fn write_raw_data(results: &[WorkloadResult], agg: &AggregatedResults) {
    use std::fs::File;
    use std::io::Write;

    // Write binary format (no JSON per FW-1)
    let mut file = File::create("ablation_results/FW-A1-CUDA/raw_data.bin").expect("create raw_data.bin");

    // Header: magic + version + count
    file.write_all(b"HKVB01").expect("magic");
    file.write_all(&(results.len() as u64).to_le_bytes()).expect("count");

    // Write each result
    for r in results {
        file.write_all(&r.insert_throughput.to_le_bytes()).expect("insert");
        file.write_all(&r.knn_throughput.to_le_bytes()).expect("knn");
        file.write_all(&r.evict_throughput.to_le_bytes()).expect("evict");
        file.write_all(&r.picard_throughput.to_le_bytes()).expect("picard");
        file.write_all(&r.peak_memory_gb.to_le_bytes()).expect("mem");
        file.write_all(&r.total_time_ms.to_le_bytes()).expect("time");
    }

    // Write aggregated
    file.write_all(&agg.insert_throughput_mvs.to_le_bytes()).expect("agg_insert");
    file.write_all(&agg.knn_throughput_mvs.to_le_bytes()).expect("agg_knn");
    file.write_all(&agg.evict_throughput_mvs.to_le_bytes()).expect("agg_evict");
    file.write_all(&agg.picard_throughput_kops.to_le_bytes()).expect("agg_picard");
    file.write_all(&agg.peak_memory_gb.to_le_bytes()).expect("agg_mem");
    file.write_all(&agg.avg_total_time_ms.to_le_bytes()).expect("agg_time");
    file.write_all(&(agg.iterations as u64).to_le_bytes()).expect("agg_iter");

    println!("Raw data written to ablation_results/FW-A1-CUDA/raw_data.bin");
}

#[derive(Debug)]
struct AggregatedResults {
    insert_throughput_mvs: f64,
    knn_throughput_mvs: f64,
    evict_throughput_mvs: f64,
    picard_throughput_kops: f64,
    peak_memory_gb: f64,
    avg_total_time_ms: f64,
    iterations: usize,
}

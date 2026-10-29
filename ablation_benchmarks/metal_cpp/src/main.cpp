// Hyperbolic KV Benchmark - C++ Main Entry Point
// FW-A1-CUDA Baseline Comparison

#include "hyperbolic_kv_bench.h"
#include <chrono>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <memory>
#include <random>
#include <vector>

struct WorkloadConfig {
  size_t num_vectors = 1'000'000;
  size_t insert_count = 1'000'000;
  size_t knn_count = 100'000;
  size_t knn_k = 32;
  size_t evict_count = 100'000;
  size_t picard_count = 10'000;
  uint32_t picard_iterations = 32;
  size_t iterations = 1000;
  size_t warmup_iterations = 100;
  uint64_t seed = 42;
  size_t leaf_size = 32;
};

struct WorkloadResult {
  double insert_throughput = 0.0; // M vec/s
  double knn_throughput = 0.0;    // M vec/s
  double evict_throughput = 0.0;  // M vec/s
  double picard_throughput = 0.0; // ops/s
  double peak_memory_gb = 0.0;
  double total_time_ms = 0.0;
};

class WorkloadRunner {
public:
  explicit WorkloadRunner(const WorkloadConfig &config)
      : config_(config), rng_(config.seed) {
    tree_ = std::make_unique<VpTree>(config.leaf_size);
  }

  WorkloadResult run_iteration(size_t iteration) {
    auto iter_start = std::chrono::high_resolution_clock::now();

    // Re-seed for reproducibility
    rng_.seed(config_.seed + iteration);

    // 1. Insert workload
    auto insert_start = std::chrono::high_resolution_clock::now();
    run_inserts();
    double insert_time =
        std::chrono::duration<double>(
            std::chrono::high_resolution_clock::now() - insert_start)
            .count();
    double insert_throughput = config_.insert_count / insert_time / 1'000'000.0;

    // 2. k-NN search workload
    auto knn_start = std::chrono::high_resolution_clock::now();
    run_knn_search();
    double knn_time = std::chrono::duration<double>(
                          std::chrono::high_resolution_clock::now() - knn_start)
                          .count();
    double knn_throughput = config_.knn_count / knn_time / 1'000'000.0;

    // 3. Evict workload
    auto evict_start = std::chrono::high_resolution_clock::now();
    run_evicts();
    double evict_time =
        std::chrono::duration<double>(
            std::chrono::high_resolution_clock::now() - evict_start)
            .count();
    double evict_throughput = config_.evict_count / evict_time / 1'000'000.0;

    // 4. Picard reduce workload
    auto picard_start = std::chrono::high_resolution_clock::now();
    run_picard_reduce();
    double picard_time =
        std::chrono::duration<double>(
            std::chrono::high_resolution_clock::now() - picard_start)
            .count();
    double picard_throughput = config_.picard_count / picard_time;

    double total_time =
        std::chrono::duration<double, std::milli>(
            std::chrono::high_resolution_clock::now() - iter_start)
            .count();
    double peak_memory_gb = estimate_memory_gb();

    return WorkloadResult{insert_throughput, knn_throughput, evict_throughput,
                          picard_throughput, peak_memory_gb, total_time};
  }

private:
  WorkloadConfig config_;
  std::mt19937_64 rng_;
  std::unique_ptr<VpTree> tree_;
  std::vector<AtomCoord> atoms_;
  uint64_t atom_counter_ = 0;

  void run_inserts() {
    for (size_t i = 0; i < config_.insert_count; ++i) {
      LorentzVec h3 = random_lorentz_vec(rng_, 100.0);
      ToroidalPhase t2 = ToroidalPhase::from_lorentz(h3);
      AtomCoord atom(h3, t2, atom_counter_++, false);
      tree_->insert(atom);
    }
  }

  void run_knn_search() {
    for (size_t i = 0; i < config_.knn_count; ++i) {
      LorentzVec h3 = random_lorentz_vec(rng_, 100.0);
      ToroidalPhase t2 = ToroidalPhase::from_lorentz(h3);
      AtomCoord query(h3, t2, UINT64_MAX, false);
      auto results = tree_->knn_search(query, config_.knn_k);
      (void)results; // suppress unused warning
    }
  }

  void run_evicts() {
    for (size_t i = 0; i < config_.evict_count; ++i) {
      tree_->evict_max_radial();
    }
  }

  void run_picard_reduce() {
    std::vector<LorentzVec> picard_atoms;
    picard_atoms.reserve(config_.picard_count);

    for (size_t i = 0; i < config_.picard_count; ++i) {
      picard_atoms.push_back(random_lorentz_vec(rng_, 10000.0));
    }

    for (auto &atom : picard_atoms) {
      for (uint32_t iter = 0; iter < config_.picard_iterations; ++iter) {
        double x0 = q32_31_to_f64(atom.x0);
        if (x0 <= 8.0) {
          picard_reduce::reduce_moderate(&atom);
          break;
        } else {
          picard_reduce::reduce_unbounded(&atom);
          if (q32_31_to_f64(atom.x0) <= 8.0)
            break;
        }
      }
    }
  }

  double estimate_memory_gb() const {
    size_t num_nodes = tree_->size();
    size_t atom_mem = num_nodes * sizeof(AtomCoord);
    size_t overhead = atom_mem; // tree pointers, etc.
    return static_cast<double>(atom_mem + overhead) /
           (1024.0 * 1024.0 * 1024.0);
  }
};

struct AggregatedResults {
  double insert_throughput_mvs = 0.0;
  double knn_throughput_mvs = 0.0;
  double evict_throughput_mvs = 0.0;
  double picard_throughput_kops = 0.0;
  double peak_memory_gb = 0.0;
  double avg_total_time_ms = 0.0;
  size_t iterations = 0;
};

AggregatedResults
aggregate_results(const std::vector<WorkloadResult> &results) {
  AggregatedResults agg;
  agg.iterations = results.size();
  double n = static_cast<double>(results.size());

  for (const auto &r : results) {
    agg.insert_throughput_mvs += r.insert_throughput;
    agg.knn_throughput_mvs += r.knn_throughput;
    agg.evict_throughput_mvs += r.evict_throughput;
    agg.picard_throughput_kops += r.picard_throughput / 1000.0;
    agg.peak_memory_gb += r.peak_memory_gb;
    agg.avg_total_time_ms += r.total_time_ms;
  }

  agg.insert_throughput_mvs /= n;
  agg.knn_throughput_mvs /= n;
  agg.evict_throughput_mvs /= n;
  agg.picard_throughput_kops /= n;
  agg.peak_memory_gb /= n;
  agg.avg_total_time_ms /= n;

  return agg;
}

void print_results(const AggregatedResults &agg, double elapsed_sec) {
  std::cout << "\n=== AGGREGATED RESULTS (C++ Baseline) ===\n";
  std::cout << std::fixed << std::setprecision(3);
  std::cout << "Insert throughput:      " << agg.insert_throughput_mvs
            << " M vec/s\n";
  std::cout << "k-NN search throughput: " << agg.knn_throughput_mvs
            << " M vec/s\n";
  std::cout << "Evict throughput:       " << agg.evict_throughput_mvs
            << " M vec/s\n";
  std::cout << "Picard reduce throughput: " << agg.picard_throughput_kops
            << " K ops/s\n";
  std::cout << "Peak memory:            " << agg.peak_memory_gb << " GB\n";
  std::cout << "Avg iteration time:     " << agg.avg_total_time_ms << " ms\n";
  std::cout << "Total benchmark time:   " << elapsed_sec << " s\n";
  std::cout << "Iterations:             " << agg.iterations << "\n";
  std::cout << "\n=== JUDGE CRITERIA (FW-A1-CUDA) ===\n";
  std::cout
      << "Target: CUDA Rust throughput / C++ throughput >= 0.95 (parity)\n";
  std::cout << "Target: CUDA Rust peak memory / C++ peak memory <= 1.05\n";
  std::cout << "\nThis C++ baseline provides the denominator for ratio "
               "calculations.\n";
}

void write_raw_data(const std::vector<WorkloadResult> &results,
                    const AggregatedResults &agg) {
  std::ofstream file("ablation_results/FW-A1-CUDA/cpp_raw_data.bin",
                     std::ios::binary);

  // Header
  file.write("HKVB01", 6);
  uint64_t count = results.size();
  file.write(reinterpret_cast<const char *>(&count), sizeof(count));

  // Results
  for (const auto &r : results) {
    file.write(reinterpret_cast<const char *>(&r.insert_throughput),
               sizeof(double));
    file.write(reinterpret_cast<const char *>(&r.knn_throughput),
               sizeof(double));
    file.write(reinterpret_cast<const char *>(&r.evict_throughput),
               sizeof(double));
    file.write(reinterpret_cast<const char *>(&r.picard_throughput),
               sizeof(double));
    file.write(reinterpret_cast<const char *>(&r.peak_memory_gb),
               sizeof(double));
    file.write(reinterpret_cast<const char *>(&r.total_time_ms),
               sizeof(double));
  }

  // Aggregated
  file.write(reinterpret_cast<const char *>(&agg.insert_throughput_mvs),
             sizeof(double));
  file.write(reinterpret_cast<const char *>(&agg.knn_throughput_mvs),
             sizeof(double));
  file.write(reinterpret_cast<const char *>(&agg.evict_throughput_mvs),
             sizeof(double));
  file.write(reinterpret_cast<const char *>(&agg.picard_throughput_kops),
             sizeof(double));
  file.write(reinterpret_cast<const char *>(&agg.peak_memory_gb),
             sizeof(double));
  file.write(reinterpret_cast<const char *>(&agg.avg_total_time_ms),
             sizeof(double));
  file.write(reinterpret_cast<const char *>(&agg.iterations), sizeof(size_t));

  std::cout << "C++ raw data written to "
               "ablation_results/FW-A1-CUDA/cpp_raw_data.bin\n";
}

int main() {
  std::cout << "=== Hyperbolic KV Benchmark (C++ Baseline) ===\n";
  std::cout << "FW-A1-CUDA Ablation - C++ Hand-rolled SIMD + PTX\n";
  std::cout << "Vectors: 1,000,000 | Dimensions: H³(3) + T²(2) | Fixed-point: "
               "Q32.31\n\n";

  WorkloadConfig config;
  std::cout << "Configuration:\n";
  std::cout << "  Insert: " << config.insert_count << "\n";
  std::cout << "  k-NN: " << config.knn_count << " queries, k=" << config.knn_k
            << "\n";
  std::cout << "  Evict: " << config.evict_count << "\n";
  std::cout << "  Picard: " << config.picard_count << " ops, "
            << config.picard_iterations << " iterations\n";
  std::cout << "  Iterations: " << config.iterations
            << " (warmup: " << config.warmup_iterations << ")\n\n";

  WorkloadRunner runner(config);

  // Warmup
  std::cout << "Running warmup (" << config.warmup_iterations
            << " iterations)...\n";
  for (size_t i = 0; i < config.warmup_iterations; ++i) {
    runner.run_iteration(i);
    if (i % 20 == 0) {
      std::cout << "." << std::flush;
    }
  }
  std::cout << "\nWarmup complete.\n\n";

  // Benchmark
  std::cout << "Running benchmark (" << config.iterations
            << " iterations)...\n";
  auto bench_start = std::chrono::high_resolution_clock::now();
  std::vector<WorkloadResult> results;
  results.reserve(config.iterations);

  for (size_t i = 0; i < config.iterations; ++i) {
    results.push_back(runner.run_iteration(i));
    if (i % 100 == 0) {
      std::cout << "." << std::flush;
    }
  }

  double elapsed = std::chrono::duration<double>(
                       std::chrono::high_resolution_clock::now() - bench_start)
                       .count();
  std::cout << "\nBenchmark complete in " << std::fixed << std::setprecision(2)
            << elapsed << "s\n\n";

  // Aggregate and print
  auto agg = aggregate_results(results);
  print_results(agg, elapsed);

  // Write raw data
  write_raw_data(results, agg);

  return 0;
}

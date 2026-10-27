# C21 — Performance Budgets, Capacity & Energy

**Status.** Scaffolded; detail pending. **Parent:** Master Design Document index.

## Purpose

Quantified SLOs and capacity/energy budgets that gate claims.

## Subcategories (broad topics)

- C21.1 Tokens/sec p50/p95/p99 on target hardware
- C21.2 TTFT thresholds and measurement
- C21.3 Peak memory ceiling per topology (Metal/CPU)
- C21.4 Joules/token and joules/recall
- C21.5 Concurrency/backpressure (max in-flight, permits)
- C21.6 Worst-arc (toroidal) reporting
- C21.7 Capacity planning for N lanes and long context

## Open questions / gaps

- Baselines on dev Mac not yet recorded.

## Contract catalog (frozen for design)

| Metric                       | Target shape            | Evidence requirement                             |
| ---------------------------- | ----------------------- | ------------------------------------------------ |
| Decode tok/s p50/p95/p99     | number                  | hardware, settings, workload, comparison (C09.3) |
| TTFT                         | ms                      | measured, not placeholder (F10)                  |
| Peak memory                  | GB ceiling per topology | hard ceiling; backpressure permits (NF1, C22.1)  |
| Joules/token, joules/recall  | number                  | energy accounting (NF9-11)                       |
| Worst-arc (toroidal) latency | min/max ratio           | cyclic reporting (C13.2)                         |
| Concurrency                  | max in-flight           | cap `min(16, max_connections)` (C12.2)           |

### Rule

- No "fast"/"efficient" claim without p99 on target hardware plus waste figure.

## Formal Performance & Energy Model

### Hardware-Normalized Metrics

All metrics normalized to **reference machine**: MacBook Pro M3 Max, 36GB Unified Memory, macOS 14.

```rust
struct HardwareSpec {
    gpu_tflops_fp16: f64,      // M3 Max: ~45
    memory_bandwidth_gb_s: f64, // M3 Max: ~400
    cpu_cores: u32,             // M3 Max: 16 (12P + 4E)
    unified_memory_gb: u32,     // 36
}

fn normalize_metric(raw: f64, target: &HardwareSpec, reference: &HardwareSpec) -> f64 {
    // Compute-bound: scale by TFLOPS
    // Memory-bound: scale by bandwidth
    // Mixed: geometric mean
    let compute_ratio = reference.gpu_tflops_fp16 / target.gpu_tflops_fp16;
    let memory_ratio = reference.memory_bandwidth_gb_s / target.memory_bandwidth_gb_s;
    let combined = (compute_ratio * memory_ratio).sqrt();
    raw * combined
}
```

### Energy Accounting (powermetrics Integration)

```rust
// Energy measurement via macOS powermetrics
struct EnergyAccountant {
    sampler: PowermetricsSampler,
    baseline_watts: f64,  // Idle power
}

impl EnergyAccountant {
    fn measure_tok_s(&mut self, duration: Duration) -> EnergyMetrics {
        let samples = self.sampler.sample(duration);
        let avg_watts = samples.iter().map(|s| s.gpu_watts + s.cpu_watts).sum::<f64>() / samples.len() as f64;
        let dynamic_watts = avg_watts - self.baseline_watts;

        EnergyMetrics {
            joules_per_token: dynamic_watts / self.tok_s,
            joules_per_recall: dynamic_watts / self.recalls_per_sec,
            peak_watts: samples.iter().map(|s| s.gpu_watts + s.cpu_watts).fold(0.0, f64::max),
            energy_efficiency: self.tok_s / dynamic_watts,  // tok/J
        }
    }
}
```

### Capacity Planning Model

**Memory budget per topology**:

| Topology                | VRAM Budget | CPU Budget | Max Context | Max Lanes |
| ----------------------- | ----------- | ---------- | ----------- | --------- |
| Standard Transformer    | 28 GB       | 8 GB       | 131k        | 4         |
| Hyperbolic Cache (ACFT) | 22 GB       | 6 GB       | 131k        | 8         |
| Hybrid (fallback)       | 30 GB       | 8 GB       | 131k        | 4         |

**Model**:
$$\text{Mem}_{\text{total}} = \underbrace{\text{ModelWeights}}_{\text{fixed}} + \underbrace{\text{KVCache}(\text{context}, \text{batch})}_{O(N^2) \text{ or } O(N \log N)} + \underbrace{\text{Atlas}}_{\text{geometry}} + \underbrace{\text{Overhead}}_{\text{system}}$$

**Hyperbolic savings**: $\text{KVCache}_{\text{hyperbolic}} \approx \frac{d_{\text{reason}}}{N} \cdot \text{KVCache}_{\text{standard}}$

### Backpressure as Control Theory

The **admission controller** is a **PI controller** on memory pressure:

```rust
struct AdmissionController {
    target_memory_gb: f64,      // e.g., 30 GB
    kp: f64,                    // Proportional gain
    ki: f64,                    // Integral gain
    integral_error: f64,
    max_inflight: u32,          // min(16, max_connections)
}

impl AdmissionController {
    fn admit(&mut self, current_memory_gb: f64) -> bool {
        let error = self.target_memory_gb - current_memory_gb;
        self.integral_error += error;

        let control = self.kp * error + self.ki * self.integral_error;
        let desired_inflight = (self.max_inflight as f64 * (1.0 + control)).clamp(1.0, self.max_inflight as f64);

        self.current_inflight < desired_inflight as u32
    }
}
```

### Toroidal Worst-Arc Reporting

```rust
struct ToroidalMetrics {
    // Per-phase (0 to 2π) latency
    phase_latencies: Vec<f64>,  // 360 bins (1° each)

    // Computed metrics
    mean_latency: f64,
    max_latency: f64,
    min_latency: f64,
    wrap_penalty: f64,           // Latency at phase boundary
    imbalance: f64,              // Max deviation from mean
    hotspot_arc: (f64, f64),     // (start_phase, end_phase) of worst arc
}

impl ToroidalMetrics {
    fn worst_arc(&self) -> (f64, f64, f64) {
        // Find contiguous arc with max average latency
        let n = self.phase_latencies.len();
        let mut max_avg = 0.0;
        let mut best_start = 0;
        let mut best_end = 0;

        for start in 0..n {
            for len in 1..=n/2 {  // Arcs up to half circle
                let end = (start + len) % n;
                let avg = if end > start {
                    self.phase_latencies[start..end].iter().sum::<f64>() / len as f64
                } else {
                    (self.phase_latencies[start..].iter().sum::<f64>()
                     + self.phase_latencies[..end].iter().sum::<f64>()) / len as f64
                };
                if avg > max_avg {
                    max_avg = avg;
                    best_start = start;
                    best_end = end;
                }
            }
        }

        (best_start as f64 * 2.0 * PI / n as f64,
         best_end as f64 * 2.0 * PI / n as f64,
         max_avg)
    }
}
```

### SLO Targets (Measurable)

| Metric           | Target  | Measurement          | Evidence                     |
| ---------------- | ------- | -------------------- | ---------------------------- |
| Decode tok/s p50 | ≥ 60    | A1 ablation workload | `throughput_qwen36.json`     |
| Decode tok/s p95 | ≥ 45    | A1 ablation workload | `throughput_qwen36.json`     |
| Decode tok/s p99 | ≥ 35    | A1 ablation workload | `throughput_qwen36.json`     |
| TTFT p50         | ≤ 50ms  | Streaming chat       | `ttft_qwen36.json`           |
| TTFT p99         | ≤ 150ms | Streaming chat       | `ttft_qwen36.json`           |
| Peak memory      | ≤ 30 GB | Pressure test        | `pressure_test_results.json` |
| Joules/token     | ≤ 0.5 J | powermetrics         | `energy_accounting.json`     |
| Joules/recall    | ≤ 2.0 J | powermetrics         | `energy_accounting.json`     |
| Worst-arc ratio  | ≤ 2.0x  | Toroidal bench       | `toroidal_metrics.json`      |
| Max in-flight    | 16      | Config               | `config.acft_max_inflight`   |

### Waste Figure (Required for Claims)

Every performance claim must include **waste figure**:

$$\text{Waste} = \frac{\text{Resources Consumed} - \text{Minimum Theoretical}}{\text{Resources Consumed}}$$

Example: "64 tok/s at 27 GB peak memory (waste: 12% vs theoretical minimum 23.8 GB)"

---

## Contributor Deepening — Muse (2026-10-26)

### C21.8 Normalization limits (depth: what hardware-normalization cannot do)

The geometric-mean normalization (compute + memory) is PROPOSED and honest for same-architecture comparisons (M3 Max vs M2 Ultra). Frozen limits [PROPOSED — limits are my proposal; the formula is frozen content]:

1. Normalization does NOT transfer across architectures (Apple Silicon vs NVIDIA H100): different memory hierarchies (unified vs discrete VRAM), different kernel availability (Metal vs CUDA), different power envelopes. The H100 column in C09.4 is therefore INDEPENDENT evidence, never normalized evidence — report raw numbers side by side, no combined metric.
2. Normalization does NOT transfer across model sizes or quantizations: 27B-4bit vs any other model is a different workload, not a different machine. Re-run, do not rescale.
3. Energy (joules/token) is NEVER normalized: watts are physical, and perf-per-watt is the quantity of interest for the solo-developer cost case (C06 frontier item 10). Report energy raw per machine with ambient temperature + battery-vs-mains state (macOS thermal throttles on battery — an unrecorded power state invalidates the comparison).
4. Each normalization application records its inputs (both spec sheets + raw numbers) in the claim evidence (C09.3) — a normalized number without its inputs is unverifiable and fails claim-lint.

### C21.9 SLO-target provenance (depth: the table above is a claim)

The SLO Targets table (tok/s ≥ 60 p50, TTFT ≤ 50ms p50, joules/token ≤ 0.5, worst-arc ≤ 2.0x, …) currently carries no provenance: are these measured baselines, aspirational goals, or placeholders? Frozen labeling [PROPOSED — labeling is my proposal; targets are frozen content]:

- Until A1 baselines land, every row in that table is PROPOSED (target), not VERIFIED (achievement). No badge, dashboard, or doc may present them as current performance.
- Each target needs a named rationale: which workload requires tok/s p99 ≥ 35? Which UX requires TTFT p99 ≤ 150ms? A target without a requiring workload is a wish — record the workload or drop the target (C03.2).
- The `throughput_qwen36.json` / `ttft_qwen36.json` evidence filenames are reserved: files with those names containing template/example numbers (cf. C12 receipt example, C05.13) MUST carry `"status": "TEMPLATE"` until a passing ablation populates them. Enforcement via claim-lint (C33/C15).

What was NOT checked here: powermetrics sampling overhead on the measured numbers (verify when baselines land); toroidal bench harness existence (still TBD per C13).

## Open questions / gaps

- Baselines on dev Mac not yet recorded.

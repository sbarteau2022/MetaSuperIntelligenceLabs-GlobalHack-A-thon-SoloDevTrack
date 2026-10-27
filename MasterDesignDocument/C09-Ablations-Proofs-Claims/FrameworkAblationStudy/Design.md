# C09-FW — Framework Ablation Study: Optimal Language/Framework Selection for Research-Grade Agentic Lab

**Status.** PROPOSED — pre-registration for ablation framework design.  
**Parent:** C09 Ablations, Proofs & Claims / Master Design Document.  
**Scope:** Prove optimal backend, middleware, frontend, and server framework stack under constraints: **no Python, no JSON**, custom hyperbolic KV (Lorentz H³×T²), custom kernels, custom inference engine, cross-platform (Mac/Linux/Windows local + Cloudflare Workers), millions of vectors, sub-ms latency.

---

## 1. Problem Statement & Hypothesis Space

### 1.1 Core Claim to Prove

> **There exists a unique (up to isomorphism) optimal framework stack** for building a research-grade agentic lab with:
> - Custom hyperbolic KV cache nested in H³×T²/Γ₁₂ (Whitehead 12-vertex)
> - Hand-rolled Metal/C++ kernels with mmap IPC
> - Custom ACFT inference engine with TwisterAttention
> - Agent harness: lanes, hooks, skills, tools, hole substrate, brigade
> - Full-stack: backend + middleware + frontend + server deployment
> - Constraints: **no Python, no JSON**, Apple Silicon + Cloudflare WASM + cross-platform

### 1.2 Hypothesis Formulation (Falsifiable)

| Hypothesis ID | Statement | Null (H₀) | Alternative (H₁) | Judge Metric |
|---------------|-----------|-----------|------------------|--------------|
| **FW-H1** | Rust is necessary for the agent harness runtime (lanes, hooks, skills, holes) | Rust replaceable by Zig/Go/C++ without regression | Rust's ownership + async + trait system is necessary for zero-cost abstractions in hot paths | Ablation: replace Rust → measure throughput regression, memory safety violations, compile-time guarantee loss |
| **FW-H2** | C++ (with custom Metal) is necessary for hyperbolic kernels (VP-tree, Lorentz ops, Picard reduction) | C++ replaceable by Rust MSL / Zig / Swift Metal | C++ template metaprogramming + SIMD intrinsics + mmap maturity is necessary for kernel fusion algebra | Ablation: port kernels → measure instruction count, register pressure, compile-time specialization |
| **FW-H3** | TypeScript (React + Vite) is necessary for IDE frontend (sub-100ms tab-open, route chunking, WCAG) | TS replaceable by Rust/WASM (Leptos/Dioxus) / Swift / Go WASM | TS ecosystem maturity for CodeMirror/LSP + Vite HMR + a11y tooling is necessary | Ablation: port frontend → measure LCP, bundle size, dev velocity, a11y compliance |
| **FW-H4** | Go (WASM) is necessary for Cloudflare Workers deployment (Durable Objects, <1ms cold start) | Go replaceable by Rust WASM / TypeScript Workers / Zig WASM | Go's WASM binary size + GC latency profile + DO integration is necessary | Ablation: deploy alternatives → measure cold start, memory, CPU, DO API ergonomics |
| **FW-H5** | FlatBuffers → custom mmap binary is necessary for serialization (no JSON, sub-µs decode) | FlatBuffers/Cap'n Proto/Protobuf replaceable without regression | Custom mmap + fixed-offset schema is necessary for zero-copy + deterministic layout | Ablation: bench serialization → measure decode latency, memory overhead, schema evolution |
| **FW-H6** | The **stack combination** (Rust+C+++TS+Go) is jointly necessary — no single-language alternative achieves Pareto frontier | Single-language (Rust-only, C++-only, Zig-only) achieves same Pareto | Cross-language boundaries (FFI, WASM, mmap) are necessary evils; each language owns its irreducible domain | Ablation: build single-lang prototype → measure total system metrics vs. multi-lang |

---

## 2. Ablation Matrix Design

### 2.1 Component × Framework Mapping

| System Component | Primary Language | Candidate Alternatives | Irreducible Property |
|------------------|------------------|------------------------|---------------------|
| **Hyperbolic KV Core** (VP-tree, Lorentz ops, Picard reduction, eviction) | **C++** | Rust MSL, Zig, Swift Metal | Template metaprogramming for kernel fusion algebra (C12.3); SIMD intrinsics; mmap maturity |
| **Inference Engine** (ACFT, TwisterAttention, scheduler, prefix cache) | **Rust** | C++, Zig | Async/await + zero-cost futures + trait-based kernel dispatch + borrow-checker for buffer lifetimes |
| **Agent Harness** (lanes, hooks, skills, tools, holes, brigade, autorunner) | **Rust** | Zig, Go, C++ | Algebraic effects via traits; session types for handoffs; compile-time tool allowlist enforcement |
| **Serialization / IPC** (mmap, flat binary, schema evolution) | **C++** (writer) + **Rust** (reader) | FlatBuffers, Cap'n Proto, custom | Zero-copy mmap + fixed Q32.31 layout + atomic refcount for cache entries |
| **Frontend / IDE** (panels, CodeMirror, LSP, Vite, a11y) | **TypeScript** | Rust/WASM (Leptos), Swift, Go/WASM | npm ecosystem for CodeMirror/LSP; Vite HMR; axe-core a11y; route-level chunking |
| **Cloudflare Workers** (Durable Objects, KV, AI binding) | **Go (WASM)** | Rust WASM, TS Workers, Zig WASM | WASM binary size (<200KB); GC pause profile; DO RPC ergonomics; wrangler compatibility |
| **Build / Orchestration** (Cargo + npm + Go + custom) | **polyglot** | Nix, Bazel, just | Hermetic reproducible builds (C10.5); cross-lang dependency graph |

### 2.2 Ablation Definitions (Pre-Registered)

Each ablation follows the **C09 protocol**: pre-registration hash → run → judge → record claim/failure.

| Ablation ID | Component Under Test | Baseline (Current Stack) | Treatment (Alternative) | Primary Judge | Secondary Judges |
|-------------|---------------------|--------------------------|-------------------------|---------------|------------------|
| **FW-A1** | Hyperbolic KV Kernels | C++ (clang, SIMD, mmap) | Rust (MLX Metal, safe SIMD) | Throughput (M vec/s) + peak memory (GB) | Instruction count (perf), compile time, kernel fusion verification |
| **FW-A2** | Hyperbolic KV Kernels | C++ | Zig (comptime SIMD, allocator) | Same as FW-A1 | Zig comptime kernel specialization vs C++ templates |
| **FW-A3** | Hyperbolic KV Kernels | C++ | Swift (MetalPerformanceShaders) | Same as FW-A1 | MPS graph optimization vs hand-rolled |
| **FW-A4** | Inference Engine (ACFT) | Rust (tokio, async-trait) | C++ (coroutines, folly) | Decode tok/s (p50/p99) + TTFT (ms) | Memory fragmentation, cancellation latency, backpressure correctness |
| **FW-A5** | Inference Engine (ACFT) | Rust | Zig (async, allocator) | Same as FW-A4 | Allocator overhead, comptime scheduler synthesis |
| **FW-A6** | Agent Harness (lanes/hooks/holes) | Rust (traits, session types) | Zig (comptime interfaces) | Lane spawn latency (µs) + tool dispatch overhead (ns) | Compile-time allowlist enforcement, handoff protocol verification |
| **FW-A7** | Agent Harness (lanes/hooks/holes) | Rust | Go (interfaces, generics) | Same as FW-A6 | GC impact on tail latency, interface dispatch cost |
| **FW-A8** | Serialization / IPC | Custom mmap (C++ writer, Rust reader) | FlatBuffers (Rust) | Decode latency (ns/op) + memory overhead (bytes/entry) | Schema evolution cost, zero-copy fidelity |
| **FW-A9** | Serialization / IPC | Custom mmap | Cap'n Proto (Rust) | Same as FW-A8 | Arena allocation vs mmap |
| **FW-A10** | Frontend (IDE panels) | TypeScript (React, Vite, CodeMirror) | Rust/WASM (Leptos, WASM-CodeMirror) | LCP (ms) + bundle size (KB gz) + TTFI (ms) | Dev velocity (HMR cycle), a11y score (axe-core), LSP integration |
| **FW-A11** | Frontend (IDE panels) | TypeScript | Swift (SwiftUI + WebKit) | Same as FW-A10 | Native integration, binary size |
| **FW-A12** | Cloudflare Workers | Go WASM (Durable Objects) | Rust WASM (workers-rs) | Cold start (ms) + memory (MB) + CPU (ms/req) | DO RPC latency, KV ops/sec, binary size |
| **FW-A13** | Cloudflare Workers | Go WASM | TypeScript (workers-ts) | Same as FW-A12 | Bundle size, JIT warmup, DO ergonomics |
| **FW-A14** | Full Stack Integration | Rust+C+++TS+Go (current) | Rust-only (WASM frontend, WASM workers) | End-to-end: tab-open → agent response (ms) | Total binary size, build time, cross-lang FFI overhead |
| **FW-A15** | Full Stack Integration | Rust+C+++TS+Go | Zig-only (WASM frontend, WASM workers, kernels) | Same as FW-A14 | Comptime maturity, ecosystem gaps |

---

## 3. Statistical Rigor & Evidence Standards

### 3.1 Sequential Testing (Per C09.5)

- **Alpha spending**: Lan-DeMets with ρ=2 (conservative)
- **Interim analyses**: Every 25% of planned samples
- **Futility stopping**: Conditional power < 0.2
- **Sample size**: Determined by power analysis (target 80% power, α=0.05, effect size d=0.8)

### 3.2 Necessity Testing (Bayesian Model Comparison)

Per C09.6, compute **Bayes Factor BF₀₁** comparing:
- H₀: Component is necessary (removal causes regression)
- H₁: Component is not necessary (no regression)

**Decision Rule**:
- BF₀₁ > 10 → Strong evidence for necessity → **REQUIRED**
- BF₀₁ < 0.1 → Strong evidence against necessity → **OPTIONAL**
- Otherwise → **INCONCLUSIVE** → more data needed

### 3.3 Effect Size Reporting (Per C09.7)

| Metric | Effect Size | CI Method | Meaningful Threshold |
|--------|-------------|-----------|---------------------|
| Throughput ratio | Cohen's d | Bootstrap (10k) | d > 0.8 = large |
| Latency reduction | % change | Percentile bootstrap | > 20% = significant |
| Memory reduction | % change | Percentile bootstrap | > 15% = significant |
| Binary size | Ratio | Log-normal | > 1.5x = worthwhile |
| Cold start | Ratio | Log-normal | > 2x = significant |
| Dev velocity | Cohen's d | Bootstrap | d > 0.5 = meaningful |

### 3.4 Hardware Normalization (Per C09.8)

All metrics normalized to reference machine:

```
norm_metric = metric_target / metric_ref × spec_ref / spec_target
```

**Reference**: MacBook Pro M3 Max, 36GB Unified Memory  
**Replica**: Mac Studio M2 Ultra, 96GB + Linux + NVIDIA H100 (optional)

---

## 4. Pre-Registration Templates

### 4.1 FW-A1: Hyperbolic KV Kernels (C++ vs Rust)

```yaml
# PreRegistrations/FW-A1.yaml
ablation_id: "FW-A1"
hypothesis: "C++ kernel implementation achieves ≥1.5x throughput at equal peak memory vs Rust MLX Metal for VP-tree + Lorentz ops + Picard reduction on H³×T²"
hardware:
  primary: "MacBook Pro M3 Max, 36GB"
  replica: "Mac Studio M2 Ultra, 96GB"
workload:
  name: "hyperbolic_kv_bench"
  vectors: 1_000_000
  dimensions: 256 (H³) + 2 (T² phase)
  ops: ["insert", "knn_search", "evict", "picard_reduce"]
  iterations: 1000
baseline:
  language: "C++20"
  compiler: "clang-17 -O3 -march=native"
  simd: "AVX2/NEON hand-rolled"
  ipc: "mmap + atomic refcount"
treatment:
  language: "Rust 1.82"
  metal: "mlx 0.21 + metal 0.28"
  simd: "std::simd + portable_simd"
  ipc: "mmap via memmap2"
judge:
  primary: "throughput_ratio >= 1.5 AND peak_memory_ratio <= 1.05"
  secondary:
    - "instruction_count_ratio <= 0.8"
    - "compile_time_minutes <= 30"
    - "kernel_fusion_verification: PASS"
effect_size_plan:
  throughput: "Cohen's d, bootstrap 10k"
  memory: "% change, percentile bootstrap"
  instructions: "ratio, log-normal"
null_handling:
  if_treatment_wins: "C++ not necessary for kernels → mark OPTIONAL"
  if_inconclusive: "Extend sample size; test hybrid (Rust caller + C++ kernels via FFI)"
pre_registration_hash: "sha256:TBD_BEFORE_RUN"
```

### 4.2 FW-A6: Agent Harness (Rust vs Zig)

```yaml
# PreRegistrations/FW-A6.yaml
ablation_id: "FW-A6"
hypothesis: "Rust trait-based agent harness achieves ≤2x lane spawn latency and ≤1.5x tool dispatch overhead vs Zig comptime interfaces, with compile-time allowlist enforcement"
hardware:
  primary: "MacBook Pro M3 Max, 36GB"
workload:
  name: "agent_harness_bench"
  lanes: 8
  tools_per_lane: 20
  handoffs: 1000
  iterations: 100
baseline:
  language: "Rust 1.82"
  dispatch: "trait objects + async-trait"
  session_types: "compile-time via typestate"
  allowlist: "const generics + trait bounds"
treatment:
  language: "Zig 0.13"
  dispatch: "comptime interfaces + async"
  session_types: "comptime protocol verification"
  allowlist: "comptime enum + switch"
judge:
  primary: "lane_spawn_latency_ratio <= 2.0 AND tool_dispatch_ratio <= 1.5"
  secondary:
    - "compile_time_allowlist_enforcement: PASS (Zig) vs PASS (Rust)"
    - "handoff_protocol_verification: PASS (both)"
    - "binary_size_ratio <= 1.2"
effect_size_plan:
  latency: "Cohen's d, bootstrap 10k"
  dispatch: "ratio, log-normal"
  binary: "ratio, log-normal"
null_handling:
  if_treatment_wins: "Zig viable for harness → mark Rust OPTIONAL for this component"
  if_inconclusive: "Test hybrid: Rust orchestration + Zig hot paths"
pre_registration_hash: "sha256:TBD_BEFORE_RUN"
```

### 4.3 FW-A10: Frontend (TypeScript vs Rust/WASM)

```yaml
# PreRegistrations/FW-A10.yaml
ablation_id: "FW-A10"
hypothesis: "TypeScript React frontend achieves ≤170KB gzipped LCP bundle, <100ms tab-open, WCAG 2.1 AA, with HMR <500ms"
hardware:
  primary: "MacBook Pro M3 Max, 36GB (dev)"
  target: "Chrome 120+, Safari 17+, Firefox 120+"
workload:
  name: "ide_frontend_bench"
  panels: 6 (Dashboard, Settings, Swarm, Runs, Holes, Memory, Skills)
  lsp: "rust-analyzer via vscode-languageserver"
  editor: "CodeMirror 6"
  routes: 7 (route-level chunking)
baseline:
  language: "TypeScript 5.4"
  framework: "React 18 + Vite 5"
  editor: "CodeMirror 6 + @codemirror/lang-rust"
  a11y: "axe-core + eslint-plugin-jsx-a11y"
  chunking: "route-level dynamic import"
treatment:
  language: "Rust 1.82 → WASM"
  framework: "Leptos 0.6 + Trunk"
  editor: "wasm-codemirror (port)"
  a11y: "manual ARIA + custom test"
  chunking: "wasm-split (experimental)"
judge:
  primary: "LCP_bundle_kb_gz <= 170 AND tab_open_ms <= 100 AND axe_score >= 0.95"
  secondary:
    - "HMR_cycle_ms <= 500"
    - "LSP_integration_latency_ms <= 50"
    - "dev_velocity_score (survey) >= 4/5"
effect_size_plan:
  bundle: "ratio, log-normal"
  latency: "Cohen's d, bootstrap"
  a11y: "risk difference, Wilson CI"
  dev_velocity: "ordinal, bootstrap"
null_handling:
  if_treatment_wins: "Rust/WASM viable for frontend → mark TS OPTIONAL"
  if_inconclusive: "Test hybrid: TS shell + Rust/WASM panels"
pre_registration_hash: "sha256:TBD_BEFORE_RUN"
```

### 4.4 FW-A12: Cloudflare Workers (Go vs Rust WASM)

```yaml
# PreRegistrations/FW-A12.yaml
ablation_id: "FW-A12"
hypothesis: "Go WASM achieves ≤2ms cold start, ≤10MB memory, ≤5ms CPU/req on Durable Objects with KV + AI binding"
hardware:
  primary: "Cloudflare Workers (production edge)"
  local_sim: "workerd local"
workload:
  name: "cf_workers_bench"
  do_classes: 4 (Session, Atlas, Lane, Hole)
  kv_ops: 10000 (read/write)
  ai_calls: 100 (workers-ai)
  concurrent: 100
baseline:
  language: "Go 1.23 → WASM (tinygo 0.32)"
  runtime: "workerd"
  do_rpc: "native Go DO bindings"
  kv: "workers KV binding"
  ai: "workers-ai binding"
treatment:
  language: "Rust 1.82 → WASM (wasm32-unknown-unknown)"
  runtime: "workerd"
  do_rpc: "workers-rs DO bindings"
  kv: "workers-rs KV binding"
  ai: "workers-rs AI binding"
judge:
  primary: "cold_start_ms <= 2 AND memory_mb <= 10 AND cpu_ms_per_req <= 5"
  secondary:
    - "binary_size_kb <= 200"
    - "do_rpc_latency_ms <= 1"
    - "kv_ops_per_sec >= 50000"
    - "build_time_seconds <= 60"
effect_size_plan:
  cold_start: "ratio, log-normal"
  memory: "ratio, log-normal"
  cpu: "ratio, log-normal"
  binary: "ratio, log-normal"
null_handling:
  if_treatment_wins: "Rust WASM viable for Workers → mark Go OPTIONAL"
  if_inconclusive: "Test hybrid: Go for DO + Rust for compute WASM"
pre_registration_hash: "sha256:TBD_BEFORE_RUN"
```

---

## 5. Cross-Language Boundary Analysis

### 5.1 FFI / WASM / mmap Boundaries

| Boundary | Current | Protocol | Overhead Target |
|----------|---------|----------|-----------------|
| **Rust ↔ C++** (kernels) | cxx-rs / bindgen | C ABI + opaque pointers | <50ns/call |
| **Rust ↔ C++** (serialization) | mmap shared memory | Fixed-offset Q32.31 | Zero-copy (pointer cast) |
| **Rust → Go WASM** (worker RPC) | JSON over HTTP (temp) | **Custom binary (no JSON)** | <100µs RPC |
| **TS ↔ Rust** (IDE backend) | Tauri IPC / HTTP | Cap'n Proto / custom binary | <1ms round-trip |
| **Go WASM ↔ DO** | Native DO RPC | Protobuf / custom | <1ms RPC |

### 5.2 Necessity of Each Boundary

Each cross-language boundary must pass **necessity ablation** (C13.1):

> **Remove boundary → measure regression**

If regression BF₀₁ > 10, boundary is **REQUIRED**. If BF₀₁ < 0.1, boundary is **OPTIONAL** (can consolidate).

---

## 6. Claims Register Schema (Machine-Readable)

```json
{
  "claim_id": "C09-FW-A1-throughput",
  "label": "PROPOSED",
  "statement": "C++ hyperbolic kernels achieve 1.6x throughput at 0.97x peak memory vs Rust MLX on M3 Max",
  "evidence": {
    "ablation_id": "FW-A1",
    "hardware": "MacBook Pro M3 Max, 36GB",
    "workload": "1M vectors, H³×T², mixed insert/search/evict",
    "baseline_throughput_mvs": 42.3,
    "treatment_throughput_mvs": 68.7,
    "baseline_peak_gb": 28.1,
    "treatment_peak_gb": 27.4,
    "effect_size": 1.42,
    "ci_95": [1.1, 1.8],
    "bayes_factor": 47.2,
    "replication": "Mac Studio M2 Ultra: 1.5x confirmed"
  },
  "pre_registration_hash": "sha256:...",
  "timestamp": "2026-10-27T00:00:00Z",
  "corrections": []
}
```

---

## 7. Execution Protocol

### 7.1 Prerequisites (Gate 0)

- [ ] All pre-registration YAML files written and hashed
- [ ] Hardware targets provisioned (primary + replica)
- [ ] Baseline implementations complete and benchmarked
- [ ] Treatment implementations complete (or stubbed with known gaps)
- [ ] CI pipeline for ablation runner configured (GitHub Actions self-hosted)

### 7.2 Run Order (Dependency-Aware)

```
Phase 1 (Independent):     FW-A1, FW-A2, FW-A3, FW-A8, FW-A9, FW-A12, FW-A13
Phase 2 (Depends on P1):   FW-A4, FW-A5, FW-A6, FW-A7, FW-A10, FW-A11
Phase 3 (Integration):     FW-A14, FW-A15
```

### 7.3 Artifact Collection

Each ablation produces:
- `ablation_results/FW-A{X}/raw_data.json` (measurements)
- `ablation_results/FW-A{X}/analysis.ipynb` (statistical analysis)
- `ablation_results/FW-A{X}/claim.json` (claims register entry)
- `ablation_results/FW-A{X}/failure.json` (if FAIL, per C09 failure ledger)

---

## 8. Open Questions / Gaps

1. **Zig maturity**: Zig 0.13 async/Metal story unproven at scale — may need to defer FW-A2/A5/A15
2. **Swift Metal**: MPS graph optimization vs hand-rolled — need Spike before FW-A3
3. **Rust/WASM frontend**: wasm-codemirror incomplete — FW-A10 treatment may be non-viable
4. **Cloudflare Go vs Rust WASM**: workers-rs DO bindings incomplete — FW-A12 treatment may lag
5. **Cross-hardware replication**: Linux/H100 access not confirmed — may run replica later
6. **Schema evolution**: Custom mmap schema migration strategy not designed — affects FW-A8/A9

---

## 9. Success Criteria (Go/No-Go for Framework Lock)

| Criterion | Threshold | Gate |
|-----------|-----------|------|
| All Phase 1 ablations: primary judge PASS | 100% | Gate 3 |
| At least 1 treatment INCONCLUSIVE per component | Acceptable | Gate 3 |
| Necessity BF₀₁ > 10 for ≥3 components | Required | Gate 4 |
| Cross-hardware replication: effect direction same, magnitude within 20% | Required | Gate 4 |
| Claims register: all VERIFIED claims have replication | Required | Gate 5 |
| No PROPOSED claims cited in design docs without ablation | Required | Gate 1 (lint) |

---

## 10. Contributor Deepening — Muse (2026-10-27)

### FW.1 Why This Is Not "Language War"

This ablation study is **not** about language preference. It is about **architectural necessity** under these constraints:

1. **Hyperbolic geometry** (H³×T²/Γ₁₂) requires fixed-point Q32.31 arithmetic with wide accumulators — C++ templates + SIMD intrinsics are the only proven path to zero-overhead kernel fusion algebra (C12.3). Rust MSL is promising but unverified for this geometry.

2. **Agent harness** requires algebraic effects (hooks, skills, tools) with compile-time effect policing — Rust's trait system + async-trait + typestate is the only mainstream language with this combination. Zig comptime *could* work but is unproven.

3. **IDE frontend** requires CodeMirror 6 + LSP + Vite HMR + axe-core — the npm ecosystem is the only one with all four production-ready. Leptos/WASM is promising but CodeMirror WASM port is incomplete.

4. **Cloudflare Workers** requires <2ms cold start + DO RPC + KV + AI — Go WASM (tinygo) is the only tested path. workers-rs is early.

5. **No Python, no JSON** — eliminates the default ML stack, forcing custom binary serialization (mmap) which C++ and Rust both handle well.

**The hypothesis**: The Pareto frontier is achieved *only* by the multi-language stack. Any single-language attempt sacrifices at least one irreducible property.

**This will be proven or falsified by the ablations above.**

---

## 11. Verification Checklist

| Ablation | Pre-reg Hash | Baseline Impl | Treatment Impl | Primary Judge | Replication | Claim Status |
|----------|--------------|---------------|----------------|---------------|-------------|--------------|
| FW-A1    | ☐            | ☐             | ☐              | ☐             | ☐           | PROPOSED     |
| FW-A2    | ☐            | ☐             | ☐              | ☐             | ☐           | PROPOSED     |
| FW-A3    | ☐            | ☐             | ☐              | ☐             | ☐           | PROPOSED     |
| FW-A4    | ☐            | ☐             | ☐              | ☐             | ☐           | PROPOSED     |
| FW-A5    | ☐            | ☐             | ☐              | ☐             | ☐           | PROPOSED     |
| FW-A6    | ☐            | ☐             | ☐              | ☐             | ☐           | PROPOSED     |
| FW-A7    | ☐            | ☐             | ☐              | ☐             | ☐           | PROPOSED     |
| FW-A8    | ☐            | ☐             | ☐              | ☐             | ☐           | PROPOSED     |
| FW-A9    | ☐            | ☐             | ☐              | ☐             | ☐           | PROPOSED     |
| FW-A10   | ☐            | ☐             | ☐              | ☐             | ☐           | PROPOSED     |
| FW-A11   | ☐            | ☐             | ☐              | ☐             | ☐           | PROPOSED     |
| FW-A12   | ☐            | ☐             | ☐              | ☐             | ☐           | PROPOSED     |
| FW-A13   | ☐            | ☐             | ☐              | ☐             | ☐           | PROPOSED     |
| FW-A14   | ☐            | ☐             | ☐              | ☐             | ☐           | PROPOSED     |
| FW-A15   | ☐            | ☐             | ☐              | ☐             | ☐           | PROPOSED     |

---

**Next Step**: File pre-registration hashes for FW-A1, FW-A6, FW-A10, FW-A12 (the four critical-path ablations) and begin baseline implementation verification.

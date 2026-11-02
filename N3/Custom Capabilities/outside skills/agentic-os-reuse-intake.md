---
title: "Agentic OS Research Intake and Reuse Map"
document_type: "research"
version: "1.0.0"
date: "2026-10-11T23:37:16Z"
implementation_date: "2026-10-11T23:37:16Z"
author: "OpenAI Codex"
reviewers: []
status: "draft"
related_files:
  - path: "docs/research/agentic-os-reuse-intake.md"
    lines: "1-217"
    role: "primary"
  - path: "docs/architecture/predictive-drafting-hybrid-server.md"
    lines: "1-521"
    role: "supporting"
  - path: "README.md"
    lines: "188-201"
    role: "supporting"
related_issues: []
related_documents:
  - "docs/architecture/predictive-drafting-hybrid-server.md"
  - "OPS_BOARD/Deep Research/14_decoding_and_speculation/INDEX.md"
  - "OPS_BOARD/Deep Research/docs/KV_CACHE_64K_SWARM_PLAN.md"
tags: ["research-intake", "erlang-otp", "rust", "swift", "metal", "mlx", "agentic-os"]
---

# Executive Summary

The local research library already contains most of the algorithms needed for
the proposed server and swarm. We should reuse open implementations for suffix
drafting and persistent Q4 agent caches, adapt workflow-aware scheduling ideas,
and use Erlang/OTP for fault-tolerant orchestration while Rust owns the hot
control path. Swift and MLX provide the Apple-native application and tensor
boundary; Metal work begins only after profiling proves an existing kernel is
the bottleneck.

# Question Investigated

Which local preprints, existing repositories, and primary vendor designs can
reduce original implementation work for the seven-tier hybrid server and
agentic operating system? Is Erlang a sound orchestration layer without harming
the latency-sensitive MLX serving path?

# Methodology

- Inventoried the local `OPS_BOARD/Deep Research` decoding, KV-cache, and swarm
  collections.
- Read the local index and first pages of the highest-relevance papers.
- Reviewed the prior local unified-server and 64K swarm design documents.
- Checked current primary documentation from Erlang/OTP, Apple, MLX, OpenAI,
  Anthropic, Microsoft, Snowflake, SGLang, and the `agent-memory` project.
- Classified each item as reuse, adapt, study, or reject.

No repository was downloaded, vendored, or executed during this intake. Paper
claims remain external results until reproduced on the Qwen3.6 checkpoint and
this Mac.

# Findings

## Recommended native architecture

```text
SwiftUI/macOS service shell
          |
Erlang/OTP agent OS
  supervisors | gen_statem workers | event/control plane
          |
Unix-domain socket with versioned messages
          |
Rust inference control daemon
  API | scheduler | automata | cache ledger | telemetry
          |
MLX/Metal data plane
  oMLX-compatible model | verification | KV/recurrent state
```

Erlang should supervise agents, goals, retries, leases, timers, and event
delivery. Rust should own admission, continuous batching, predictive automata,
cache transactions, and any operation on the token-decode critical path. Swift
should own the signed application, lifecycle, settings, and native monitoring.
MLX/Metal remains responsible for tensor execution.

Do not run model inference as a Native Implemented Function (NIF). Erlang's own
documentation classifies work that cannot finish in roughly a millisecond as a
dirty NIF. A model kernel, cache restore, or blocked SSD read belongs in a
separate supervised process. Rustler is appropriate only for short, bounded
operations if measurement proves message transport insufficient.

## Local preprints to use now

| Local paper | Tier | What to take | Boundary |
| --- | --- | --- | --- |
| `2411.04975` SuffixDecoding | 1 | Suffix-tree corpus, adaptive draft length, verification economics | Port concepts; evaluate ArcticInference code before reuse |
| `2610.01108` AgSpec | 1, 5 | Session/workspace/global corpora; per-agent offline cap plus online adaptation | New preprint; reproduce every gain locally |
| `2506.20675` Utility-Driven SD for MoE | 1, 2 | Abstain when verification cost exceeds expected accepted-token value | Use as policy, not a claimed speedup |
| `2605.02888` SpecKV | 1, 3 | Compression-aware draft-depth selection | Test against TurboQuant Q4 specifically |
| `2511.14102` MoE-SpeQ | 4 | Expert-prefetch roofline and overlap model | Reject its learned draft model; retain the cost model |
| `2607.24434` DraftExpert | 4 | Expansion-aware expert prefetch for end-device MoE | Study only; current model fits resident memory |
| `2608.30310` Tail-Replay | 3, 4 | Rebuild hybrid recurrent state while reusing full-attention KV | High-risk on Qwen GDN; exact checkpoint remains fallback |
| `2605.05219` Sparse Prefix Caching | 3 | Fixed recurrent-state boundaries | Candidate for pinned instruction boundaries |
| `2603.04428` Agent Memory Below the Prompt | 3, 4 | Q4 safetensors block pool, isolated agent caches, restore path | MIT code; adapt model/cache support and remove synchronous writes |
| `2608.25523` TOPAS | 5, 6 | Joint prefix residency and critical-path request admission | Reimplement policy for single-device constraints |
| `2606.16824` CacheWise | 5, 6 | Coding-workflow-aware cache retention | Compare against TOPAS in simulator |
| `2507.07400` KVFlow | 5 | Multi-agent prefix-cache placement | Reuse objective terms, not distributed-GPU assumptions |
| `2609.15011` Semantic TVM | 4, 6 | Trustworthy virtual-memory framing for agent state | Design reference until code/evidence audit |
| `2608.09225` and `2607.19957` | 3-6 | KV timing and cross-session attack cases | Mandatory adversarial tests |
| `2606.09613` AgentServeSim | 6 | Hardware-aware simulation before live scheduling | Inspect for reusable workload/event model |

## Existing code to gather, not rewrite

| Project | License/status | Reuse decision |
| --- | --- | --- |
| Snowflake `ArcticInference` | Apache-2.0 | Study and potentially port the suffix index, expansion/scoring policy, and tests; do not import vLLM/CUDA coupling |
| `yshk-mxim/agent-memory` | MIT | Highest-value intake: block-pool schema, Q4 safetensors persistence, compatibility manifests, tests, and benchmark structure |
| SGLang/RadixAttention | Apache-2.0 project | Adapt radix-prefix ownership and reference-counting concepts; do not port GPU-specific runtime wholesale |
| Apple `mlx-swift` | Apple-maintained Swift package | Use for native array/model experiments and the future Swift boundary; first prove exact Qwen hybrid/OptiQ compatibility |
| Rustler | MIT/Apache-2.0 | Optional short-call bridge only; never host inference or SSD I/O inside the BEAM |
| Erlang/OTP | Mature runtime | Use supervisors, `gen_statem`, monitors, timers, ETS, and release packaging for the active orchestration plane |

Before copying code, record the exact commit, license, notices, dependency tree,
tests, and security posture. Imported algorithms receive local property tests and
must satisfy the same gates as new code.

## Vendor patterns worth adopting

- Anthropic's multi-agent research system supports an orchestrator-worker shape
  and emphasizes evaluations, tool design, and operational controls. Copy the
  separation of lead-agent intent from focused worker context, not its cloud
  assumptions.
- OpenAI's Agents SDK separates manager-style agents-as-tools from handoffs.
  Our orchestrator should use workers as bounded tools; control does not transfer
  to a worker that could redefine the shared goal.
- Microsoft AutoGen Core demonstrates typed, event-driven actor messaging and
  agent lifecycle separation. Use its message-envelope ideas; Erlang already
  supplies the actor runtime.
- Erlang/OTP supervision trees directly support restart isolation and dynamic
  worker processes. Model lanes remain resources managed by Rust, not BEAM
  scheduler threads.
- Apple's Metal argument buffers, heaps, and residency sets can reduce CPU
  encoding and residency overhead. They are candidates only after Metal System
  Trace identifies driver/resource binding as material.
- MLX Swift is the preferred Apple-native experiment surface. It is not proof
  that the exact Qwen3.6 hybrid OptiQ checkpoint and cache patches are supported.

## Pieces still requiring original work

- Exact Qwen3.6 GatedDeltaNet plus full-attention cache transactions.
- TurboQuant Q4 serialization and restore compatibility across runtime changes.
- Lossless verification adapter between predictive automata and oMLX/MLX.
- Rust N=2/N=10 scheduler tuned for unified memory and current Metal kernels.
- Stable, versioned Erlang-to-Rust protocol with backpressure and cancellation.
- Snapshot/worktree capability leases and Git ownership enforcement.
- Deterministic orchestration event replay and human-approval boundaries.

# Recommendation

Proceed with the language split, but introduce Erlang at Tier 5 rather than at
the inference-server foundation. Tiers 0-4 should stabilize the Rust/MLX server
contract first. Tier 5 adds a small Erlang/OTP release that supervises logical
agents and talks to Rust over a Unix-domain socket. Tier 6 promotes that runtime
to active orchestration only after deterministic replay and simulated failures
pass.

The first intake sequence should be:

1. Vendor-review `agent-memory` without integrating it; extract its block format,
   compatibility keys, tests, and benchmark methods.
2. Vendor-review ArcticInference's suffix-decoding implementation; isolate data
   structures and scoring from vLLM/CUDA code.
3. Implement the common simulator using TOPAS, utility-driven drafting, and
   current local workload traces.
4. Define the Erlang/Rust protocol and failure semantics on paper.
5. Build one supervised fake Rust worker before connecting a model server.

# Rejected Shortcuts

- Do not translate oMLX or Rapid wholesale into Rust, Swift, or Erlang.
- Do not use BEAM NIFs for inference, cache restore, filesystem traversal, or
  Metal command completion.
- Do not adopt a learned draft model through MoE-SpeQ; retain only its prefetch
  economics.
- Do not infer Apple performance from CUDA/vLLM results.
- Do not permit shared mutable KV, global unsalted suffix corpora, or worker
  self-merges.
- Do not force MoE expert routing based on task labels. Router fingerprints are
  telemetry and optional scheduling hints only.

# Literature and Primary Sources

| Source | Use | Link |
| --- | --- | --- |
| Oliaro et al., “SuffixDecoding” | Weight-free adaptive suffix drafts | <https://arxiv.org/abs/2411.04975> |
| Lee et al., “AgSpec” | Coding-agent retrieval corpora and adaptive caps | <https://arxiv.org/abs/2610.01108> |
| Shkolnikov, “Agent Memory Below the Prompt” | Persistent isolated Q4 agent KV | <https://arxiv.org/abs/2603.04428> |
| Ni et al., “TOPAS” | Workflow-aware prefix residency and admission | <https://arxiv.org/abs/2608.25523> |
| Wang et al., “MoE-SpeQ” | Expert-prefetch/offload cost model | <https://arxiv.org/abs/2511.14102> |
| Snowflake ArcticInference | Apache-2.0 suffix-decoding reference | <https://github.com/snowflakedb/ArcticInference> |
| `agent-memory` | MIT Apple-Silicon Q4 KV reference | <https://github.com/yshk-mxim/agent-memory> |
| Erlang/OTP Design Principles | Supervision and behaviors | <https://www.erlang.org/docs/27/system/design_principles.html> |
| Erlang NIF documentation | Long-running NIF boundary | <https://www.erlang.org/docs/25/man/erl_nif.html> |
| MLX Swift | Apple-native MLX API | <https://github.com/ml-explore/mlx-swift> |
| Apple Metal argument buffers | CPU and residency overhead controls | <https://developer.apple.com/documentation/metal/improving-cpu-performance-by-using-argument-buffers> |
| Anthropic multi-agent research system | Orchestrator-worker operational lessons | <https://www.anthropic.com/engineering/multi-agent-research-system> |
| OpenAI orchestration and handoffs | Manager versus handoff boundary | <https://developers.openai.com/api/docs/guides/agents/orchestration> |
| Microsoft AutoGen Core | Event-driven agent-runtime reference | <https://microsoft.github.io/autogen/dev/user-guide/core-user-guide/index.html> |

# Time Invested

Approximately 35 minutes for local inventory, targeted paper inspection,
primary-source verification, architecture synthesis, and documentation.

# Changelog

| Version | Date | Author | Change |
| --- | --- | --- | --- |
| 1.0.0 | 2026-10-11 | OpenAI Codex | Initial local and external research intake |

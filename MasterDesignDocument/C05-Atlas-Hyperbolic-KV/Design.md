# C05 — Atlas / Hyperbolic Memory Map & KV Cache (Difference 1)

**Status.** PROPOSED (Difference 1 fully specified by user). **Parent:** Master Design Document index.

## Purpose

Replace flat O(N²) attention KV cache with a nested hyperbolic Poincaré dynamic phase-state space
nested inside the Atlas.

## Subcategories (broad topics)

- C05.1 Problem statement: O(N²) cost, O(N) memory growth, OOM ceiling
- C05.2 Atlas (Multi-Model Phase State Space) as container
- C05.3 Hyperbolic Memory Map / Poincaré ball geometry and capacity
- C05.4 Nested Hyperbolic Poincaré Dynamic Phase-State cache (plug-in mode)
- C05.5 Plug-in modes: (a) standard transformer vs (b) hyperbolic cache
- C05.6 ACFT — Active Inferencing Transformer and TwisterAttention
- C05.7 Fallback to exact attention for safety-critical spans
- C05.8 Telemetry/diagnostics as first-class (tok/s, cache hit, recall depth, OOM headroom)
- C05.9 Security/compliance as first-class (RBAC, audit, receipts)
- C05.10 Energy-aware serving (joules/token, joules/recall)

## Cross-model geometry note (found pass)

- Our design names the Poincaré ball. The verified monster-hybrid-server uses the Lorentz
  hyperboloid H^3 as canonical compute/storage with a verified isometry (4.4e-9 over 900 pairs)
  and Picard fundamental-domain reduction for unbounded radii. Canonical-form decision and
  interop contract are tracked in C35.

## Canonical decision (frozen for design)

- **Canonical geometry:** Lorentz hyperboloid `H^3` is the compute/storage form; Poincaré coordinates are a projection for UX/radial salience. Interop correctness (isometry within 4.4e-9) is a required verification gate (C35.10).
- **Fallback:** exact attention path always available via kill switch (C10.4).

## Mathematical Specification of the Nested Hyperbolic Cache

### 1. Canonical Geometry: Lorentz Model $H^3$

The **canonical compute/storage geometry** is the **future sheet of the 3-hyperboloid** in Minkowski space $\mathbb{R}^{3,1}$:

$$
H^3 = \{ x \in \mathbb{R}^4 : \langle x, x \rangle_{\mathcal{L}} = 1,\ x_0 > 0 \}
$$

with Lorentz inner product $\langle x, y \rangle_{\mathcal{L}} = x_0 y_0 - x_1 y_1 - x_2 y_2 - x_3 y_3$.

**Fixed-point arithmetic**: $Q_{32.31}$ with wide $i128$ accumulator for Minkowski products (FLAG-02 verified: no overflow at $x_0 \geq 65,536$).

### 2. Poincaré Ball Projection (UX/Salience View)

The **stereographic projection** $\pi: H^3 \to B^3$ is an isometry:

$$
\pi(x) = \frac{(x_1, x_2, x_3)}{x_0 + 1} \in B^3 = \{ p \in \mathbb{R}^3 : \|p\| < 1 \}
$$

$$
\pi^{-1}(p) = \left( \frac{1 + \|p\|^2}{1 - \|p\|^2}, \frac{2p}{1 - \|p\|^2} \right)
$$

**Verification gate**: $\sup_{x \in H^3} d_{H^3}(x, \pi^{-1}(\pi(x))) < 4.4 \times 10^{-9}$ over 900 test pairs (monster-hybrid-server verified).

### 3. Picard Fundamental Domain Reduction

For unbounded radii ($x_0 > 8$), the **Picard group $\Gamma = PSL(2, \mathbb{Z}[i])$** acts on the upper-half-space model $H^3 \cong \mathbb{C} \times \mathbb{R}^+$ via:

$$
\gamma \cdot (z, w) = \left( \frac{a z + b}{c z + d}, \frac{w}{|c z + d|^2} \right), \quad \gamma = \begin{pmatrix} a & b \\ c & d \end{pmatrix} \in \Gamma
$$

The reduction algorithm maps any $x \in H^3$ to the **fundamental domain** $\mathcal{F}_\Gamma$:

- Moderate radii ($x_0 \leq 8$): fixed-point iteration (32 steps max)
- Unbounded radii: exact $f64$ arithmetic with cancellation-free $x_0 + x_3$ computation
- Cusp folding: $w > 64 \mapsto 1 + \log(w - 1)$ preserving cusp ray

**Idempotence gate**: $\text{reduce}(\text{reduce}(x)) = \text{reduce}(x)$ for all test radii $\in \{25, 250, 5000, 50000\}$.

### 4. Product Manifold $H^3 \times T^2$ with Phase Coordinates

Each atom carries **toroidal phase** $\phi = (\theta_f, \theta_s) \in T^2$:

$$
\theta_f = \arg(x_1 + i x_2), \quad \theta_s = \arg(\sqrt{x_1^2 + x_2^2} + i x_3)
$$

**Joint retrieval** (5D) uses combined score:

$$
\text{score}(q, a) = \underbrace{\langle q_{H^3}, a_{H^3} \rangle_{\mathcal{L}}}_{\text{hyperbolic}} + \frac{1}{4} \underbrace{d_{T^2}^2(\phi_q, \phi_a)}_{\text{toroidal phase distance}}
$$

where $d_{T^2}^2(\phi, \psi) = (1 - \cos(\theta_f - \psi_f)) + (1 - \cos(\theta_s - \psi_s))$.

### 5. Geometric Memory Map as VP-Tree on $H^3$

The **GeometricMemoryMap** indexes atoms in a **vantage-point tree** with hyperbolic distance:

$$
d_{H^3}(x, y) = \operatorname{arcosh}(\langle x, y \rangle_{\mathcal{L}})
$$

**Eviction policy** (geometric forgetting): victim = $\arg\max_{a \notin \text{pinned}} x_0^{(a)}$ (max radial coordinate = closest to boundary = lowest salience). **No heuristics**—eviction is a consequence of geometry.

### 6. ACFT: Active Inferencing Transformer with TwisterAttention

The **Active Inferencing Transformer** replaces the forward pass with **task-scoped retrieval**:

$$
\text{TwisterAttention}(q, K, V) = \sum_{i \in \mathcal{N}_k(q)} \alpha_i V_i, \quad
\alpha_i = \frac{\exp(\langle q, k_i \rangle_{\mathcal{L}} / \tau)}{\sum_j \exp(\langle q, k_j \rangle_{\mathcal{L}} / \tau)}
$$

where $\mathcal{N}_k(q)$ is the **$k$-nearest neighbors in $H^3$** (VP-tree retrieval), not the full context. The **depth gate** $k$ is chosen by **variational free energy minimization**:

$$
k^* = \arg\min_k \left( \underbrace{C_{retrieve}(k)}_{O(\log k)} + \underbrace{C_{attend}(k)}_{O(k)} - \mathbb{E}[\log p(answer \mid context_k)] \right)
$$

### 7. Persistence: Atomic Snapshots + WAL

- **Snapshot** `.atoma_map`: serialized `HashMap<AtomId, Atom>` + VP-tree index, written to `.tmp` then atomic rename
- **WAL** `.atoma_map.wal`: append-only `AuditEntry` (Write/Evict + logical clock)
- **Recovery**: verify SHA-256 of snapshot; truncate torn WAL tails; replay from last valid clock
- **Concurrency**: 8-thread turn execution with snapshot isolation

### 8. Verification Gates (C35.10)

| Gate                     | Test                                | Threshold              |
| ------------------------ | ----------------------------------- | ---------------------- |
| Isometry                 | $\pi \circ \pi^{-1}$ round-trip     | $< 4.4 \times 10^{-9}$ |
| Picard idempotence       | $\text{reduce}^2 = \text{reduce}$   | 100%                   |
| Map-toroidal attention   | Phase coherence on retrieval        | 13/13 tests pass       |
| Full-chain qualification | E2E session hygiene + replay        | 13/13 pass             |
| Atomic vault survival    | Interrupted write + restart         | 100% recovery          |
| Concurrent turns         | 8 threads, no cross-session leakage | 100% isolation         |

## Open questions / gaps

- Measured throughput numbers TBD (C12/C21).
- Curvature/depth self-tuning experiment preregistration (Frontier §8.7).

---

## Contributor Deepening — Muse (2026-10-26)

### C05.11 Capacity theorem (what exponential volume actually buys)

The design claims exponential capacity without stating the bound. Frozen statement [PROPOSED — proof sketch, not measured]: in `H³`, `Vol(B_R) ~ π(sinh(2R) − 2R)`, so the number of atoms storable with pairwise separation ≥ δ grows as `~e^{2R}/δ³`. The binding constraint is therefore **not geometric capacity but the C21 memory ceiling**: each atom costs `KV_size + coord_size + VP-tree overhead` (C12 memory accounting), so `N_max = cache_budget / mem(entry)`. Geometry removes the _search_ bottleneck (`O(log N)` retrieval, §5); the _storage_ bottleneck stays physical and is owned by C21 admission control. Any claim of "exponential capacity" without the accompanying `N_max` at the configured cache budget is incomplete per C03.2.

### C05.12 Retrieval complexity honesty (when VP-tree degrades)

`O(log N)` expected retrieval holds under a **bounded expansion constant** `c`: every ball of radius `2r` contains at most `c` balls of radius `r`. Frozen rigor requirements:

1. The A1 preregistration must state the assumed `c` for the fixed prompt corpus, or label the complexity claim INFERRED rather than VERIFIED.
2. Degenerate case is documented, not hidden: adversarially clustered atoms (all within δ of each other) degrade VP-tree search toward `O(N)`. The A1 judge corpus must include a clustered-input stress case; if it does not, the throughput claim carries the restriction "non-adversarial distributions only" (C36 correction entry required when discovered).
3. Eviction (victim = max `x₀` unpinned, §5) is a **consequence of geometry only if** insertion assigns radial coordinate by salience. If any insertion path assigns coordinates by arrival order or hash, eviction is heuristic again — audit every insertion path in Gate 3 (property test: evicted atom is always the live minimum-salience atom).

### C05.13 Claim-label audit (existing numbers in this workspace)

The throughput qualification receipt example in C12 (`decode_tok_s` p50 68.2, `tok_s_ratio` 1.61, `"status": "PASS"`) is currently unlabeled. Until measured on the A1 workload/hardware, it MUST be read as a **PROPOSED template shape** (field schema VERIFIED as a schema [VERIFIED — fields match C21 contract catalog]; values PROPOSED, not results). No badge, dashboard, or doc may cite those values as outcomes. Enforcement: `claim-lint` (C33/C15 Gate 1-2) flags any `*_qwen36.json` receipt whose values lack a linked passing ablation hash. [INFERRED — lint rule follows from ADR-18/25; ends at those files.]

### C05.14 Tracked gap: systems proof from on-disk docs

The migrated source note ("MUST GET THE MAP AND THE SYSTEMS PROOF FROM THE DOCS ON DISC AND REDO") is preserved above unedited and formalized here as a tracked action [PROPOSED — action, not a claim]:

| Field       | Value                                                                                                                                                                  |
| ----------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Gap ID      | GAP-C05-01                                                                                                                                                             |
| Requirement | Locate the on-disk systems proof for the Hyperbolic Memory Map; transcribe its theorem statements into this file; mark each VERIFIED (matches disk source) or withdraw |
| Owner       | TBD (assign in C32)                                                                                                                                                    |
| Blocks      | A1 preregistration freeze (cannot preregister what the map guarantees until the proof is transcribed)                                                                  |
| Status      | OPEN                                                                                                                                                                   |

What was NOT checked here: the on-disk docs location (not pinned in this workspace); C35 interop gate details (verify at C35).

---

## Source detail migrated from `01-Aers-SpecialOperations-Agentic-Lab-Design-Overview.md`

## 5. Atlas / Hyperbolic Memory Map (detail)

- **Atlas** = the Multi-Model Phase State Space that stores memory as coordinates in a
  Hyperbolic Memory Map. It is the container; the KV cache plug-in nests inside it.
- **Poincaré ball** geometry provides exponential capacity for hierarchical data with
  sub-linear parameter growth vs. flat Euclidean attention. NOT JUST A POINTCARE-- MUST GET THE MAP AND THE SYSTEMS PROOF FROM THE DOCS ON DISC AND REDO

- **TwisterAttention** retrieves from the map on demand and completes the task with bounded
  depth-of-reasoning respect; no blanket full forward pass.
- **Fallback** to exact attention for safety-critical spans until the hyperbolic path is
  benchmarked and receipted.

---

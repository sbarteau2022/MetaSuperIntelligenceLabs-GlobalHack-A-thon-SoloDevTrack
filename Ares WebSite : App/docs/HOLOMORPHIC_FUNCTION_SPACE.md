---
title: Holomorphic Function Space — Math Lock
document_type: Math Lock (normative)
version: 2026-09-22
date: 2026-09-22
implementation_date: 2026-09-22
author: housepapers (external)
reviewers: []
status: NORMATIVE LOCK
related_documents:
  - ../COMMANDMENTS.md
  - ./SYSTEM_WORKFLOW_LOCK.md
tags: [math, holomorphic, forbidden-claims]
---

# Holomorphic function space — math lock (claim-safe)

**Status:** Normative lock for TorPhPoEnc / TwisterPhPE silicon work.  
**Date:** 2026-09-22 (America/Chicago)  
**Authority:** Paper freeze `paper/TwisterPhPE_Whitepaper_v4_1_MATH_FREEZE.md` + `AGENTS.md` claim boundary.

## What kernels do **not** operate on

Native Lorentz / bakeoff / `tit_bench` kernels do **not** operate on the Penrose twistor sheaf cohomology space

\[
H^{1}(\mathrm{PT},\,\mathcal{O}(-2h-2)).
\]

That identification is **not** part of the v4.1 normative core. Citing massless-helicity sheaf cohomology as the runtime function space of TwisterPhPE is forbidden.

## Normative space (frozen)

The normative feature space is the **finite-feature toroidal-phase / TwisterPhPE φ** construction:

- unwrapped phases on a torus chart,
- Adler/Kuramoto 2:1 structure where applicable,
- raised-cosine (or equivalent finite Fourier) coherence,
- exact finite φ feature map with causal recurrent aggregation (paper §§1–9, 11–12).

Silicon paths (Metal+CPU; no CUDA wheels) implement this finite-feature space, not holomorphic sheaf cohomology on projective twistor space.

## Active-inference / Lorentz / lemniscate path (roadmap)

Active-inference, Lorentz inner-product PE, lemniscate / saddle routers, and related Hamiltonian experiments are **custom deformations** on hyperbolic + toroidal charts and the **\(T^{\*}Q\) roadmap** (paper §10). They are:

- useful for relative deltas (e.g. Gaussian toroidal-phase bakeoff `|mse_hyp − mse_flat|`),
- **not** a proof that MSE validates \(H^{3}\times T^{2}\),
- **not** standard massless-helicity sheaf cohomology,
- **not** a TwisterPhPE validator (`tit_bench` is IR/structural, not Twister validation).

§10 cotangent / Hamiltonian / Lorentz remains **research/roadmap** only.

## Paper freeze — Penrose demotion

The math freeze explicitly demotes Penrose twistor theory from normative derivation to conceptual inspiration:

> Penrose's twistor program remains conceptual inspiration for exploring alternative geometric representations. TwisterPhPE is not an implementation or derivation of twistor theory.

> The core TwisterPhPE result does not depend on Hamiltonian mechanics, Lorentz geometry, Penrose twistor theory, variational free energy, or thermodynamic optimization.

See freeze changelog note demoting cotangent/symplectic, Penrose, and thermodynamic extensions to non-normative roadmap status (`FREEZE_v4_1`, whitepaper §10 / closing notes).

## Fix-claim reminders

| Allowed                                                | Forbidden                               |
| ------------------------------------------------------ | --------------------------------------- |
| Report `                                               | mse_hyp − mse_flat                      | ` for Gaussian / synthetic bakeoff | “MSE→0 proves \(H^{3}\times T^{2}\)” |
| Call synthetic path `gaussian_torphase_bakeoff` cousin | Treat bakeoff MSE as TwisterPhPE proof  |
| Document CORDIC / dominance as engineering             | Claim `tit_bench` validates TwisterPhPE |
| Cite §10 as roadmap                                    | Cite Lorentz/lemniscate as frozen core  |

## Implementation pointer (this wave)

- Vertex ABI: locked `GraphNode` in `native_decode/graph_node.rs` (see `TOPOLOGICAL_GRAPH_G.md`).
- Default geometry stack: DomTree depth → unprojected H³ Lorentz + T² phase + product geodesic.
- Sheaf-Laplacian remains an **alternative** research path — not the default this wave.

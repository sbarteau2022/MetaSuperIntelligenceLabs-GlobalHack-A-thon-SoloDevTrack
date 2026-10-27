# C01 — Scope, Goals, Non-Goals

**Status.** PROPOSED (contract-first). **Parent:** Master Design Document index.

## Purpose

Define what the enterprise track builds, what it explicitly does not build, and the deliverable shape,
so the sprint ships a coherent system instead of accumulated parts.

## Scope

- In scope: agent lanes, RAG + ingest + memory, workflow/hole orchestration, multi-agent comms,
  telemetry, recovery, governance, model serving, security, compliance.
- Out of scope for this scaffold pass: code implementation of the model server; numbers not yet
  measured; TEE/attestation (excluded on Apple Silicon stack).

## Subcategories (broad topics)

- C01.1 Problem statement and success definition
- C01.2 In-scope capabilities list (must map to architecture tree)
- C01.3 Explicit non-goals and exclusions
- C01.4 Deliverables and exit criteria
- C01.5 Assumptions and constraints (hardware, OS, local-only)

## Formal Problem Statement

**Theorem (Solo-Developer Enterprise Feasibility).** A single developer can ship an enterprise-grade agentic system iff the memory-computation coupling satisfies:

$$
\mathcal{C}_{mem} \cdot \mathcal{C}_{comp} \leq \kappa \cdot \frac{B_{VRAM}}{N_{lanes}}
$$

where $\mathcal{C}_{mem}$ is memory retrieval complexity, $\mathcal{C}_{comp}$ is compute complexity per token, $B_{VRAM}$ is GPU memory budget, $N_{lanes}$ is concurrent agent count, and $\kappa$ is a hardware-dependent constant. The hyperbolic cache achieves $\mathcal{C}_{mem} = O(\log N)$ and $\mathcal{C}_{comp} = O(d_{reason})$ where $d_{reason} \ll N$ is reasoning depth.

**Success Definition (Measurable).** The system ships when:

- $A1$ ablation passes: $\text{tok/s}_{hyperbolic} / \text{tok/s}_{standard} \geq 1.5$ at equal peak memory
- $A2$ ablation passes: 100% objective recall after 2 compactions
- $A4$ ablation passes: live weight copies = 1 under N=8 swarm
- $C15$ gates all pass on two distinct hardware targets

## Deliverables (Formal)

| Artifact                  | Type                                | Verification             |
| ------------------------- | ----------------------------------- | ------------------------ |
| `Design.md` per category  | Markdown + formal appendices        | Cross-reference complete |
| `PreRegistrations/`       | JSON + hashes                       | Pre-run hash recorded    |
| `VerificationCommands.md` | Executable CI                       | All gates green          |
| `BUILD-RECEIPT.md`        | Signed receipt                      | Git commit + lock hashes |
| `CLAIMS_REGISTER.md`      | VERIFIED/REPORTED/INFERRED/PROPOSED | Evidence-linked          |

## Explicit Non-Goals (with rationale)

| Non-Goal                    | Rationale                                         | Revisit Condition          |
| --------------------------- | ------------------------------------------------- | -------------------------- |
| TEE/attestation             | Apple Silicon lacks remote attestation            | Hardware support lands     |
| Distributed training        | Single-node OOM is the binding constraint         | Multi-node federation (P6) |
| Full model implementation   | Verified Monster server provides Lorentz/Poincaré | ACFT kernel fusion (D8-9)  |
| General-purpose LLM serving | Scope is agentic memory-computation               | If benchmark demands       |

## Assumptions & Constraints (Formal)

- **Hardware:** Apple Silicon (M-series), Unified Memory $\geq$ 36GB, Metal 3.0+
- **OS:** macOS 14+ (Seatbelt sandbox available)
- **Local-only:** No external API dependencies for core loop
- **Model:** Qwen3.8-27B-Uncensored-OrcaRouter-MLX-4bit (Rapid-MLX :8001)
- **Context:** 131,072 tokens max; 200-line default budget

## Open questions / gaps

- None yet; refine success metric thresholds in C21/C08.

## Verification hooks

- Verification gates (C15); claims require workload/hardware/baseline (C09).

---

## Source detail migrated from `01-Aers-SpecialOperations-Agentic-Lab-Design-Overview.md`

## 1. Scope and non-goals

- **In scope (local prototype → enterprise):** agent lanes, RAG + ingest + memory, workflow/hole
  orchestration, multi-agent comms, telemetry, recovery, governance, model serving, security,
  compliance.
- **Out of scope for this scaffold pass:** code implementation of the model server; numbers not
  measured yet; TEE/attestation (excluded on Apple Silicon stack).
- **Deliverable:** this design doc + implementation passes with explicit verification gates.

---

---

## Source preamble migrated from `01-Aers-SpecialOperations-Agentic-Lab-Design-Overview.md`

# Hackathon / Enterprise Design — Ares SpecOps Agentic Lab (from first principles)

**Status.** PROPOSED scaffold for the enterprise track; Difference 1 now fully specified by the user.
This is a _design_ document for the _next_ build, not a description of the current local-only app.

**Purpose.** Replace the ad-hoc, piecewise Ares app with a single enterprise-grade, first-principles
design scoped up front, so the sprint ships a coherent system instead of accumulated parts.
The current Ares app is the experimental local-only prototype that this design supersedes.

---

## Contributor Deepening — Muse (2026-10-26)

### C01.6 Falsifiability conditions (what would refute the feasibility theorem)

The feasibility theorem in Formal Problem Statement is PROPOSED [PROPOSED — no measurement yet exists]. It becomes testable only with refutation conditions:

1. If A1 treatment shows `tok/s_hyperbolic / tok/s_standard < 1.5` at equal peak memory on both hardware targets → success criterion 1 fails → scope reverts to standard-transformer-only via kill switch (ADR-05), hyperbolic cache demoted to optional/off-by-default with measured cost (C13.1).
2. If A2 recall < 100% after 2 compactions on the fixed objective corpus → criterion 2 fails → compaction parameters revised via appended correction entry (C36); ship blocked until re-pass.
3. If live weight copies > 1 under N=8 swarm → criterion 3 fails → Brigade (C06.12) stays PROPOSED experiment, never required.
4. If any C15 gate fails on either hardware target at Day 10 go/no-go → no-go regardless of other passes (ADR-21).

[INFERRED — refutation logic follows from ADR-21/22/05 combined with the C01 success definition; ends at the cited files, no new measurement claimed.]

### Requirement traceability (C01 → gates → ablations) [INFERRED from C09/C15/README gate table]

| #   | Success criterion                         | Gate (C15)                    | Ablation (C09)    | Owning category |
| --- | ----------------------------------------- | ----------------------------- | ----------------- | --------------- |
| 1   | tok/s ≥ 1.5x at equal peak memory         | Gate 8 (perf + build receipt) | A1                | C05 / C12       |
| 2   | 100% objective recall after 2 compactions | Gate 3 (unit + integration)   | A2                | C24 / C07       |
| 3   | Live weight copies = 1 under N=8          | Gate 5 (contract + property)  | A4                | C30 / C06.12    |
| 4   | All gates pass on two hardware targets    | Gates 1–8, Day 10             | A11–A35 as mapped | C15             |

### Scope-change control

Any in-scope / out-of-scope change after Day 1 requires an appended correction entry in C36 plus an updated traceability row above. Silent scope edits are forbidden (C03.3 append-only truth).

### Exit-criteria measurement protocol

Measured once at Day 10 go/no-go (C32), on primary + replica hardware (C09.4), recorded in `BUILD-RECEIPT.md` + `CLAIMS_REGISTER.md` with VERIFIED labels; null/negative results recorded as first-class entries in the workstream Failures file (ADR-17 five-file set). What was NOT checked here: actual gate-to-criterion mapping inside C15/Design.md (verify before freezing this table).

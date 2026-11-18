# SUBAGENT-PROTOCOLS.md — Swarm Law for PARTITIONed Delegation

Date: 2026-10-30. Status: PROPOSED.

Authority: `SUBAGENT-CONTRACT.md` (acceptance test); RULES.md rules 60a, 72–74; AGENTS.md §§7–9.
PARTITION-first: every delegation below assigns a vertical whole-slice (feature / surface / contract), never a horizontal layer — rule 60a. Siloed swarms (one mind per slice, disjoint write-sets) are a REQUIREMENT, not a preference.

## 1. ROLES table (contract §7 shape: exactly these three columns)

| role | responsibility | write-set |
| ---- | -------------- | --------- |
| Explorer | Maps one slice front-to-back (code, registers, contracts) and returns findings only. | Recon notes under `docs/` for its slice, or none (read-only preferred); disjoint per slice |
| Architect | Turns Explorer findings into frozen slice contracts (schemas, gates, write-sets) Coders build against. | `<slice>/CONTRACT.md` + delegation messages; disjoint per slice |
| Coder-N | Builds exactly one vertical slice front-to-back against frozen contracts, verifying per-edit. | One exclusive subtree per coder (e.g. `areas/forms/`); two coders MUST NOT share a file |
| Critic | Independently re-checks every slice (re-runs checkers, re-reads source) and alone judges pass/fail. | Critic findings log under `docs/` only; never Coder write-sets |
| Tool-Runner | Executes heavy toolchains (builds, typechecks, suites, audits) so Coders never self-certify. | Tool output logs only; zero source writes |

Forbidden actions per role (advisory — the table above is law): Explorer MUST NOT write source or contracts. Architect MUST NOT code slices. Coder-N MUST NOT write outside its write-set, MUST NOT touch another coder's slice, MUST NOT self-approve. Critic MUST NOT patch code — critic findings re-enter at VERIFY layer (b) through the owning Coder. Tool-Runner MUST NOT edit source.

## 2. ROUTING rule (contract §7 shape: one line per rule, WHEN / THEN / VIA)

- WHEN task touches ≤3 files in one slice THEN solo Coder VIA direct 5-field delegation (no fan-out).
- WHEN task touches >3 files or ≥2 slices THEN full fan-out (Explorer → Architect → N Coders → Critic) VIA frozen contracts on disk.
- WHEN two designs compete for one slice THEN A/B variant Coders VIA branched workspaces with disjoint write-sets and a dated justification recorded BEFORE fan-out.
- WHEN a Coder's slice fails VERIFY THEN the owning Coder VIA re-delegation to the failing layer (never a second Coder on the same write-set).
- WHEN Critic finds a defect THEN the owning Coder VIA the critic finding, re-entering at VERIFY layer (b) (critic MUST NOT patch).
- WHEN toolchain runtime exceeds interactive patience or one gate must hold identically across N slices THEN Tool-Runner VIA a standing delegation whose RETURN-SHAPE is pasted checker output.

## 3. PARTITION binding (rule 60a cited)

All routing above partitions vertically by whole slice; horizontal layer splits (one agent on CSS, another on JS of the same file) are FORBIDDEN and fail the contract gate. One mind owns each slice front-to-back; siloed swarms are REQUIRED.

## 4. CONTRACT-GATE definition (contract §7 shape: four items per gate)

- (a) Gate name: `spawn-gate`. (b) Artifacts on disk before fan-out: slice `CONTRACT.md` (reviewed, path named) + frozen delegation messages + a write-set map proving pairwise disjointness. (c) Enforcer: transcript review — every delegation carries BRIEF / RISK / BLAST-RADIUS / CONTRACT-FIRST and the 5-field template verbatim. (d) Veto: fan-out STOPS; the missing artifact is archived as a Failures entry; re-authorisation requires a new reviewed delegation. Contracts stay FROZEN during fan-out — any change = full re-review + re-delegation, never a mid-flight edit.

## 5. LOOP-EXIT condition (contract §7 shape: three items per loop)

- (a) Exit predicate (ALL true, pasted not summarised): gates green (discipline + audit exit 0) AND zero runtime errors on the slice's live path AND Critic pass recorded — else re-enter at the failing VERIFY layer. (b) Evaluator: Critic for judgement, Tool-Runner for checker layers — never the Coder being judged (no self-approval). (c) Veto archive: append findings to the review/Failures log + archive the attempt; never last-write-wins overwrite.

# Submission — Meta Super Intelligence Labs Global Hackathon (Solo Dev Track)

**Entry.** Ares SpecOps Agentic Lab — design-first agentic system for a solo developer on one Apple Silicon machine.
**Repo.** `/Users/stewartbarteau/Desktop/GitHub/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack` (verify with `git -C <path> status`).
**Date.** 2026-10-31. **Status.** Docs closeout; code frozen; nothing here is committed by this pass.

> Label key (repo standard §7): VERIFIED = measured/read from the source itself; REPORTED = a source says so, unchecked;
> INFERRED = reasoned from evidence; PROPOSED = a plan, not a fact.
> No published judging rubric was found in this repo (searched all `*.md` for judging/criteria/rubric language — no rubric file exists),
> so the mapping below is PROPOSED against the five dimensions judges typically score, each tied to a concrete artifact.

## 1. Judging-criteria mapping (PROPOSED dimensions → VERIFIED artifacts)

| Dimension (typical) | What to open | Evidence status |
|---|---|---|
| Innovation / technical depth | `MasterDesignDocument/C06-Frontier-Push/` (τ-aperture sweep A41-RUN-02 7/8 J1; hyperbolic-native retrain prereg A40; NN-G VP-tree + role-gate fix F1/F2/F3) | VERIFIED from run reports listed in §3 |
| Working demo (runs on hardware) | Demo runbook §2; receipts `FULL-SYSTEM-SMOKE-RECEIPT.txt` (6/6 stages exit 0), `SMOKE-RECEIPT-2026-10-29.txt` (157 green, 0 fail) | REPORTED (receipts); re-runnable via §2 |
| Rigor / preregistration discipline | `PreRegistrations/` (A1–A41 + NN-G v1/v2; no filed hash, no run — C11.3/ADR-22) | VERIFIED (hash tables + `Claims_Register.md` Registration Evidence) |
| Safety / honesty (failures kept) | `A39-RunCertification-2026-10-27.md` (0/80 all-fail, claim DENIED, report certified honest); `NN-G2-G7-RunReport-2026-10-30.md` (two measured FAILs, then v2 PASS) | VERIFIED from the reports themselves |
| Reproducibility / docs | `research-grade-harness/HARDWARE-GATE-RUNBOOK.md`; `PreRegistrations/README.md`; this file | VERIFIED (paths exist; commands in §2 verified against source) |

## 2. Demo runbook (every command grounded — path/binary/flag verified in-repo 2026-10-31)

Preconditions (from `FULL-SYSTEM-SMOKE-RECEIPT.txt`): `cargo 1.99.0 / rustc 1.99.0` (`cargo --version`);
model server omlx live at `http://127.0.0.1:8000` (receipt: Bearer dummy, 6 models listed);
binary present at `hypermind-nested-kv/target/debug/hypermind-mcp` (VERIFIED executable, 2026-10-30 build).

```sh
R=~/Desktop/GitHub/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack

# 0. Preconditions: toolchain, model server, debug binary
cargo --version
curl -s -H "Authorization: Bearer dummy" http://127.0.0.1:8000/v1/models
ls -la $R/hypermind-nested-kv/target/debug/hypermind-mcp

# 1. research-grade-harness suite, offline — receipt: 67 passed, 0 failed, 1 ignored
cd $R/research-grade-harness && cargo test --offline

# 2. hypermind-nested-kv suite, offline, all targets — receipt: 53 passed, 0 failed
cd $R/hypermind-nested-kv && cargo test --offline --all-targets

# 3. Live engine↔N³ round trip (ignored test; needs HYPERMIND_MCP_BIN + live :8000)
#    Test fn `live_n3_round_trip` in research-grade-harness/tests/engine_n3.rs (#[ignore] verified).
cd $R/research-grade-harness && \
  HYPERMIND_MCP_BIN=$R/hypermind-nested-kv/target/debug/hypermind-mcp \
  cargo test live_n3_round_trip -- --ignored

# 4. Hardware PPC gate, KEEP_BASE expected (saturation: all bundles 8/8, gain 0.0000)
#    Env `PPC_K` read in research-grade-harness/src/bin/hardware_ppc.rs:332; defaults
#    OMLX_URL=http://127.0.0.1:8000/v1/chat/completions, OMLX_MODEL=Qwen3.6-MoE-Combined.
cd $R/research-grade-harness && PPC_K=1 cargo run --bin hardware_ppc

# 5. Wire proof (delete old receipt/db first, per receipt) — receipt: events=10, integrity=ok, 6/6 accepts
#    Usage verified in hypermind-nested-kv/wire_proof.py:16 (--db/--mcp flags with same defaults).
cd $R/hypermind-nested-kv && rm -f observability/store/wire-proof.sqlite3 WIRE-PROOF-RECEIPT.txt && \
  HYPERMIND_ROOT=$(mktemp -d) python3 wire_proof.py

# 6. Pressure test — receipt form quoted verbatim
cd $R/hypermind-nested-kv && HYPERMIND_ROOT=/tmp/n3-smoke python3 pressure_test.py

# 7. SQLite observability suite — receipt: 54/54 (test file + .pytest_cache verified present)
cd $R/hypermind-nested-kv/observability && python3 -m pytest tests/ -q

# 8. (Optional) rgah→N³ stdio round trip + kill-server degraded drill
#    Env HYPERMIND_MCP / HYPERMIND_ROOT / WIRE_RECEIPT + `--kill-server` flag verified
#    in research-grade-harness/src/bin/rg_n3_wire.rs:126-140. Standing receipt: RG-N3-WIRE-RECEIPT.txt.
cd $R/research-grade-harness && \
  HYPERMIND_MCP=$R/hypermind-nested-kv/target/debug/hypermind-mcp \
  cargo run --bin rg_n3_wire
```

Expected outcome (REPORTED from the two smoke receipts): all stages exit 0; suite totals 157 green, 0 fail
(rgah 50/50, nkv 53/53, sqlite 54/54 per `SMOKE-RECEIPT-2026-10-29.txt`; full-system receipt records rgah 67 + hmnkv 53 = 120
across its stages a/b — both figures quoted with source, not reconciled here).
NOT proven by this runbook (stated in the receipts themselves): daemon mode, live write-through continuity,
multi-packet DAGs beyond seed→query, prompt-driven PPC separation (gate still abstains, gain 0.0000).

## 3. Video checklist (PROPOSED — no video asset exists in the repo as of 2026-10-31)

- [ ] 0:00–0:30 — Problem: solo dev needs team-grade discipline; show `README.md` entry point + C01–C35 map.
- [ ] 0:30–2:00 — Live run: execute runbook steps 1–4 on camera (offline suites → live round trip → PPC KEEP_BASE receipt).
- [ ] 2:00–3:00 — Honesty segment: open `A39-RunCertification-2026-10-27.md` (failed run, claim denied) + NN-G v1 FAIL → v2 PASS.
- [ ] 3:00–3:30 — Prereg discipline: `PreRegistrations/README.md` hash tables; "no filed hash, no run."
- [ ] Captions + repo URL + commit hash in description; keep under the hackathon's time limit (limit not in repo — confirm before recording).

## 4. Artifact index (top-level, docs scope)

`SUBMISSION.md` (this file) · `AUDIT-RECEIPT.md` (§2026-10-31 addendum) · `PreRegistrations/` (A1–A41 + NN-G) ·
`N^3-System-Audit.md` · `MasterN3DB-RAG-MultiModal.md` (renamed, space-free) · `AGENTS.md` (canonical-path correction).

Out of docs scope (frozen, untouched by this pass): `research-grade-harness/`, `hypermind-nested-kv/`,
`DistillationProtocol/`, `ablation_*`, git history.

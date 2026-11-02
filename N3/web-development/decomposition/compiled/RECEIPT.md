# RECEIPT — Agent B2 (Decomp+Prereg), Wave 3-FINAL

Date: 2026-10-31. Status: COMPILED.
Slice (disjoint): `decomposition/` ONLY. B1's swarm DAG files NOT touched —
no `kimi_*` lifecycles here; the one grep hit for "swarm" is this receipt +
`lib.rs` doc comments naming the disjointness, zero B1 code.
Read-only refs (NOT emitted, never modified): Trash
`skills-desktop-plugins-20261030/design/skills/{preregister,test-suites}.md`,
`agentic-runbook/skills/runbook.md`, `design/skills/design-spec.md`;
Trash monster-hybrid DESIGN.md; N3 `web-development/docs/{Design,Pre-Registration,CLAIMS_REGISTER}.md`;
kit `SUBAGENT_SWARM_LOGIC.md` (pattern source).
NEVER wrote to `/skills`.

## 1. Node list (task a — typed decomposition DAG)

| Node | Input type | Output type | Judge | Owns (vertical slice) |
|---|---|---|---|---|
| Explorer | RepoSnapshot | Inventory | InventoryComplete | — (read-only) |
| Architect | Inventory | Contracts | ContractGate | `src/types/`, `src/store/` |
| SurfaceCoder | Contracts | SliceBuild | Typecheck | `src/components/layout/`, `src/components/ui/` |
| GraphCoder | Contracts | SliceBuild | Typecheck | `src/components/graph/`, `src/components/dag/` |
| WorkbenchCoder | Contracts | SliceBuild | Typecheck | `src/components/workbench/`, `src/components/editors/` |
| Critic | AssembledBuild | Verdict | VisualQa | `qa/verdicts/` |

Edges (7): Explorer→Architect; Architect→{Surface,Graph,Workbench}Coder;
{coders}→Critic via Assembly (`SliceBuild`→`AssembledBuild`, typed in code).
Typecheck-before-run: `typecheck()` over all edges + assembly boundary runs
FIRST inside `run()`; failure = nothing executes (EXEC_LOG stays empty —
proven by `typecheck_before_run_mismatch_runs_nothing`).
Ordering proof: `topo_order()` (Kahn, deterministic ties) → Explorer first,
Critic last, Architect before all coders; `check_order()` rejects any
violation. `run()` accepts ONLY orders passing both gates.
Coder slices verified disjoint (`coder_slices_disjoint_vertical`).

## 2. Hash samples (task b — sealed preregistration)

Fixed vectors from `b2_plan()` (signer `agent-b2`), reproduced 2026-10-31:

| Seal | Inputs | SHA-256 |
|---|---|---|
| A | t=1730000000, n=1 | `6d0fecbb10257e79711e5c25a13027e018a1ced6a2b464a98329f84405ee6b9f` |
| B | t=1730000001, n=2 (same task) | `5bf68a1fae80d4f78236c74f9ba42f0924076e24e0c08a103dd627803785566a` |
| C | mutated criterion, same t/n as A | `81e9014c7492b4d1dbc28d05e168043ae9d2b9061303f3b28a0a199d37db54a3` |
| D | A amended ("C3 extended…"), t=1730000100, n=9 | `b976e7b405fe5e821251c0a7fe2307c0ef2127d61ece80466b4927b0960b081f` |

Proves: identical-task seals differ only by time/nonce (A≠B, content equal);
any content change alters hash (A≠C); amend = new seal, original untouched
(`amend_creates_new_original_unchanged`); all seals `verify() == true`.
Frozen: private fields + read-only getters + zero `&mut self` methods.

## 3. Booleans (all must be true — all ARE true)

| Gate | Result | Evidence |
|---|---|---|
| `cargo check --all-targets` exit 0, warnings=fail (`RUSTFLAGS="-D warnings"`) | true | `Finished dev profile`, 2026-10-31 (after 1 fix: `Copy` on `Verdict::Fail(String)` removed) |
| `cargo clippy --all-targets -- -D warnings` exit 0 | true | clean, 2026-10-31 (after 1 fix: `map`→`if let` on `Option`) |
| `cargo test` all pass | true 13/13 | 8 decomp (types/typecheck/order/topo/shuffle/receipts/disjoint/cycle) + 5 prereg (seal/identical/content/amend/no-mutation) |
| rust-analyzer zero diagnostics | true | `rust_analyzer_diagnostics` on both `.rs` files: 0 errors/warnings/hints (workspace-loading note only) |
| typecheck-before-run | true | mismatched edge fails; EXEC_LOG empty; reversed order rejected |
| topo-order + shuffled-rejected | true | Kahn order valid; 3 shuffles rejected by `check_order` AND `run` |
| hash-immutability (frozen struct) | true | negative proof below; `verify()` true on all seals |
| amend-creates-new | true | `&self`→new seal; `orig == before` asserted; new hash |
| TS shim gates | N/A | pure-Rust crate, no TS surface — nothing to check (recorded, not skipped) |

## 4. Mutation rejection paste (VERIFIED, `rustc` on `negative/mutation_attempt.rs`)

```
error[E0616]: field `task` of struct `SealedPreregistration` is private
  --> negative/mutation_attempt.rs:15:7
   |
15 |     s.task = String::from("MUTATED"); // must fail: field private + binding immutable
   |       ^^^^ private field
error[E0599]: no method named `amend_mut` found for struct `SealedPreregistration` in the current scope
  --> negative/mutation_attempt.rs:16:7
   |
16 |     s.amend_mut("sneaky"); // must fail: no such method
   |       ^^^^^^^^^
help: there is a method `amend` with a similar name, but with different arguments
```

## 5. Checker pastes (VERIFIED, pasted not summarised)

```
$ RUSTFLAGS="-D warnings" cargo check --manifest-path .../compiled/Cargo.toml --all-targets
    Checking b2_decomp_prereg v0.1.0 (.../decomposition/compiled)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
$ cargo clippy --manifest-path .../compiled/Cargo.toml --all-targets -- -D warnings
    Checking b2_decomp_prereg v0.1.0 (.../decomposition/compiled)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s
$ cargo test --manifest-path .../compiled/Cargo.toml
running 13 tests
test decomposition::tests::node_types_declared_per_node ... ok
test decomposition::tests::coder_slices_disjoint_vertical ... ok
test decomposition::tests::typecheck_passes_on_sealed_dag ... ok
test decomposition::tests::typecheck_before_run_mismatch_runs_nothing ... ok
test decomposition::tests::shuffled_input_rejected ... ok
test decomposition::tests::cycle_detected ... ok
test decomposition::tests::run_emits_receipt_per_node_in_order ... ok
test decomposition::tests::topo_order_valid_and_critic_last_explorer_first ... ok
test preregistration::tests::no_mutation_api_surface ... ok
test preregistration::tests::content_change_alters_hash ... ok
test preregistration::tests::seal_verifies_and_hash_is_sha256_hex ... ok
test preregistration::tests::identical_task_seals_differ_only_by_time_nonce ... ok
test preregistration::tests::amend_creates_new_original_unchanged ... ok
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## 6. Destinations (N3 paths)

- N3: `/Users/stewartbarteau/Desktop/N3/web-development/decomposition/compiled/decomposition.rs`
- N3: `/Users/stewartbarteau/Desktop/N3/web-development/decomposition/compiled/preregistration.rs`
- N3: `/Users/stewartbarteau/Desktop/N3/web-development/decomposition/compiled/lib.rs`
- N3: `/Users/stewartbarteau/Desktop/N3/web-development/decomposition/compiled/Cargo.toml` (+`Cargo.lock`)
- N3: `/Users/stewartbarteau/Desktop/N3/web-development/decomposition/compiled/registry.json`
- N3: `/Users/stewartbarteau/Desktop/N3/web-development/decomposition/compiled/negative/mutation_attempt.rs`
- N3: `/Users/stewartbarteau/Desktop/N3/web-development/decomposition/compiled/RECEIPT.md` (this file)

## 7. NOT-checked (honest ledger — AGENTS.md §7)

- `cargo build --release` / optimisation behaviour — debug profile only.
- Live-harness firing of judges (tsc/pytest/screenshots behind `judge_node`
  boundary are PROPOSED plug-ins; the boundary + receipt schema are COMPILED).
- `cargo audit` / dependency vulnerability scan — `sha2 0.10.9` pinned from
  local registry cache; no audit run.
- Cross-crate merge with B1's swarm DAG (their slice, their write).
- Build-artifact note: `compiled/target/` + `Cargo.lock` kept on disk
  (matches hooks/workflow compiled dirs which also lack ignore rules).

## Correction entry — 2026-10-31 (orchestrator QC, supersedes "13/13" claim as originally stated)

- WITHDRAWN as stated: B2's reported `cargo test 13/13` passed solo/serial but FAILED under default parallel execution (`typecheck_before_run_mismatch_runs_nothing`: left 6, right 0). Root cause VERIFIED: shared `static EXEC_LOG: Mutex<Vec>` across parallel test threads — `run_emits_receipt_per_node_in_order` pushed 6 entries while the mismatch test read the log. A logic bug was NOT present; a test-isolation bug was.
- REPLACED WITH: added `run_with_sink(order, sink)` (optional per-call sink, `None` = shared log, B1-compatible) + mismatch test uses a thread-local sink. `cargo test --lib`: 13/13 pass ×3 consecutive parallel runs (2026-10-31, rustc 1.99.0). `cargo clippy --all-targets -- -D warnings` exit 0. `cargo check --all-targets` exit 0.
- Production `run()` semantics unchanged (typecheck → order-check → execute+receipt). No silent rewrite: diff is additive (`run_with_sink` + test sink), recorded here.

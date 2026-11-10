# Idle frontier loop: model neutral system contract

Status: preregistered architecture, 2026-11-10. No autonomous improvement is certified yet.

## Local runtime correction

The local Apple Silicon lane uses a native Rust worker under a macOS sandbox profile. The supervisor pins Git HEAD, extracts a disposable archive under `results/frontier-loop/`, clears inherited credentials, denies network access, and grants writes only to that run directory. The worker must be compiled for macOS and must be an absolute executable. A mission preempts it through the existing frontier barrier. The Linux Docker image remains an optional cloud lane and is not the local model or Metal qualification runtime. The native lane has a separate smoke gate: verify the sandbox denies network and writes outside its run directory, then run a real repository gate. This is CPU gate discovery, not Metal/model qualification. Read access remains broad for build dependencies; do not run untrusted worker binaries.

## Placement

The backend owns an idle supervisor, not the model seat. It may trigger only when no governed mission holds a run permit, after an idle grace interval, and when no frontier job is already active. It survives a chat ending because it is a server process, not a session hook. Startup must not build an image or modify the repository without an explicit configured frontier worker image. A stopped backend means the loop is stopped, and status must say so.
While the worker is discovery-only, the supervisor runs a given commit at most once per backend process after a valid receipt. Repeating an unchanged snapshot cannot reveal a new source target and would consume energy for no new evidence. A new commit reopens discovery. This is a process-lifetime deduplication rule, not persistent history or a substitute for future iteration on an admitted target.
Mission dispatch and frontier launch share a Tokio read/write barrier. A mission requests frontier cancellation before waiting for a read guard; a worker holds the write guard only while its container is active. The supervisor polls cancellation and removes its own container before releasing the guard. No mission may start its governed work while a frontier container still owns the barrier. Cancellation is a separate, non-success receipt state.

The worker is an executable with a model neutral contract. The active model can propose a target, but proposal text cannot itself choose the gate or mark success. The governor ranks candidates from actual failed compile and library-test gates, real task outcomes, and telemetry. A deterministic tie key is the candidate's canonical identifier. No Qwen path, endpoint, tokenizer, or checkpoint is compiled into the supervisor. Model I/O, when added, uses the binary seat contract. A library-test failure is a target-discovery signal, not a hardware or task-quality verdict.

## Optional binary proposer, selection only

A custom worker image may set `ARES_FRONTIER_PROPOSER_BIN` to an absolute executable inside that image. The worker sends a bounded `FRQ1` frame over stdin: 40 ASCII lowercase source-commit hex bytes, an 8-bit candidate count, then for each failed gate an 8-bit target-name length and bytes, 64 lowercase output-digest hex bytes, a 16-bit little-endian excerpt length, and the raw bounded failure excerpt. The proposer returns one `FRS1` frame on stdout: the same 40-byte source, one admitted target name, a 64-byte lowercase model-identity claim, a 16-bit little-endian rationale length, and UTF-8 rationale bytes. Each frame begins with its four-byte magic. Candidate count, names, excerpts, rationale, total output, and runtime are bounded by the worker. Extra bytes, stale source, an unknown target, invalid identity, or a failed process refuse the proposal.

The proposer chooses among gate-failure candidates only. The worker owns candidate eligibility, and the backend independently requires the selected target's gate row to carry `reason=gate-failure`, a nonzero exit, and the same 64-hex output digest named by the decision. The response identity is a claim until separately attested by a native model seat. If no proposer is configured, selection remains deterministic by canonical target ID. This protocol does not edit source, set a success bar, or certify self-improvement.

## One iteration

1. Snapshot the committed Git HEAD; dirty worktree files are excluded by `git archive`. Inspect registered component and end-to-end gates, relevant receipts, and architecture dependencies. Choose one target with a measured baseline and expected system-level gain. If there is no evidenced target, record `no_target` and sleep.
   The backend pins the exact commit before launching Docker and archives that commit, even if HEAD advances while the job runs. Every receipt line must carry that pinned source identity; an unmatched or duplicated target decision is rejected before persistence.
2. Before edits, persist a goal receipt naming target, baseline command/data, quality floor, resource metric, exact threshold, drastic bar, and expiry. Neither a worker nor a model may alter the bar after measurements begin.
3. Stream `git archive HEAD` into a bounded, disposable Docker worker. There is no host mount, live database credential, or Docker socket inside the container. The current worker uses a read-only image filesystem plus an anonymous `/work` volume. The container has CPU and memory limits; the volume does not yet have a disk quota. Run the baseline and the failing gate on real data. The worker can iterate inside that container only. Each attempt records source digest, command, output digest, duration, and failure classification. The container is removed after the job; no separate Lima disk remains.
   Gate stdout and stderr must be capped while the process runs, not after `wait_with_output`. An over-limit child is killed and its gate is marked unavailable; truncated output cannot be classified as a source failure.
   The backend must also bound Docker stdout and stderr while reading. If either exceeds the receipt cap, it terminates the job, removes its named container, and records failure without persisting partial output.
   A failed gate receipt must carry a bounded, byte-exact tail excerpt alongside its full-output SHA-256. A killed process or unavailable dependency/platform is an environment result, not an optimization target. The excerpt helps diagnosis; the full output digest is the identity of the evidence that was observed.
4. The same quality gates run on baseline and candidate. Measure task quality, latency/throughput, peak memory, and actual energy when available; label time-derived cost as proxy. A speed improvement that violates correctness, safety, model-boundary, or system-level homogeneity is rejected. Apply the same evidence and scrutiny to every candidate family (isotropic suppression).
5. Export a patch plus full receipts; never auto-merge into the live tree. The backend may schedule another attempt only after recording the decision. A gate pass below the preregistered drastic bar means continue; a failed gate requires a reflection before another attempt. A promotion needs a separate governed review and real post-merge gate.

## First implementation gate

Expose truthful active-mission count and an idle predicate from the backend. A foreground mission must prevent any frontier trigger; a second frontier job must be refused. A stopped Docker daemon, missing image, or insufficient Colima memory must show `unavailable`, not a success claim. The Rust worker now discovers targets from portable real compile gates and emits bounded receipts. `core_lm`'s Apple framework gate is explicitly unavailable in Linux Docker; GlobalMind's library gate is portable, while its hardware tests are not. Model proposal, implementation iteration, full quality/energy gates, Apple hardware qualification, and promotion remain separate gates.

## Falsifiers

- Any Qwen-specific path in the supervisor, a proposal that can set its own gate, or a fabricated measurement fails the model-neutral requirement.
- Running while a mission is active, after a failed authority check, or against the host's live database fails isolation.
- Replaying a result without immutable source and command digests, or claiming a drastic gain from a unit test alone, fails qualification.

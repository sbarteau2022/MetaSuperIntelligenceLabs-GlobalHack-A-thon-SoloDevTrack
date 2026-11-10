# Idle frontier loop: model neutral system contract

Status: preregistered architecture, 2026-11-10. No autonomous improvement is certified yet.

## Placement

The backend owns an idle supervisor, not the model seat. It may trigger only when no governed mission holds a run permit, after an idle grace interval, and when no frontier job is already active. It survives a chat ending because it is a server process, not a session hook. Startup must not build an image or modify the repository without an explicit configured frontier worker image. A stopped backend means the loop is stopped, and status must say so.
Mission dispatch and frontier launch share a Tokio read/write barrier. A mission requests frontier cancellation before waiting for a read guard; a worker holds the write guard only while its container is active. The supervisor polls cancellation and removes its own container before releasing the guard. No mission may start its governed work while a frontier container still owns the barrier. Cancellation is a separate, non-success receipt state.

The worker is an executable with a model neutral contract. The active model can propose a target, but proposal text cannot itself choose the gate or mark success. The governor ranks candidates from actual failed compile and library-test gates, real task outcomes, and telemetry. A deterministic tie key is the candidate's canonical identifier. No Qwen path, endpoint, tokenizer, or checkpoint is compiled into the supervisor. Model I/O, when added, uses the binary seat contract. A library-test failure is a target-discovery signal, not a hardware or task-quality verdict.

## One iteration

1. Snapshot the committed Git HEAD; dirty worktree files are excluded by `git archive`. Inspect registered component and end-to-end gates, relevant receipts, and architecture dependencies. Choose one target with a measured baseline and expected system-level gain. If there is no evidenced target, record `no_target` and sleep.
   The backend pins the exact commit before launching Docker and archives that commit, even if HEAD advances while the job runs. Every receipt line must carry that pinned source identity; an unmatched or duplicated target decision is rejected before persistence.
2. Before edits, persist a goal receipt naming target, baseline command/data, quality floor, resource metric, exact threshold, drastic bar, and expiry. Neither a worker nor a model may alter the bar after measurements begin.
3. Stream `git archive HEAD` into a bounded, disposable Docker worker. There is no host mount, live database credential, or Docker socket inside the container. Use a read-only image filesystem and a capped writable tmpfs for the source snapshot. Run the baseline and the failing gate on real data. The worker can iterate inside that container only. Each attempt records source digest, command, output digest, duration, and failure classification. The container is removed after the job; no Lima disk remains.
   Gate stdout and stderr must be capped while the process runs, not after `wait_with_output`. An over-limit child is killed and its gate is marked unavailable; truncated output cannot be classified as a source failure.
4. The same quality gates run on baseline and candidate. Measure task quality, latency/throughput, peak memory, and actual energy when available; label time-derived cost as proxy. A speed improvement that violates correctness, safety, model-boundary, or system-level homogeneity is rejected. Apply the same evidence and scrutiny to every candidate family (isotropic suppression).
5. Export a patch plus full receipts; never auto-merge into the live tree. The backend may schedule another attempt only after recording the decision. A gate pass below the preregistered drastic bar means continue; a failed gate requires a reflection before another attempt. A promotion needs a separate governed review and real post-merge gate.

## First implementation gate

Expose truthful active-mission count and an idle predicate from the backend. A foreground mission must prevent any frontier trigger; a second frontier job must be refused. A stopped Docker daemon, missing image, or insufficient Colima memory must show `unavailable`, not a success claim. The Rust worker now discovers targets from portable real compile gates and emits bounded receipts. `core_lm`'s Apple framework gate is explicitly unavailable in Linux Docker; GlobalMind's library gate is portable, while its hardware tests are not. Model proposal, implementation iteration, full quality/energy gates, Apple hardware qualification, and promotion remain separate gates.

## Falsifiers

- Any Qwen-specific path in the supervisor, a proposal that can set its own gate, or a fabricated measurement fails the model-neutral requirement.
- Running while a mission is active, after a failed authority check, or against the host's live database fails isolation.
- Replaying a result without immutable source and command digests, or claiming a drastic gain from a unit test alone, fails qualification.

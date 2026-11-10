# Frontier worker

This Rust worker is the model-neutral discovery stage of the idle frontier
loop. It receives a committed Git snapshot over the Docker container's stdin,
checks registered Rust components against their actual source, and emits
bounded gate and target-decision receipts. It does not edit code, claim a
drastic improvement, or promote anything to the live checkout.
The Linux container runs backend `cargo check --tests`, backend library tests,
and GlobalMind library `cargo check --lib`. It marks `core_lm` Apple-framework gates unavailable;
those require a separate macOS hardware run and can never be inferred from
this Docker receipt. A `target=none` decision covers portable gates only.
Each gate's stdout and stderr are bounded while the process runs. An output
overflow kills that gate and records it as unavailable rather than selecting
it as a source defect. Backend library-test failures can become discovery
targets; those tests do not certify model-forward quality.
Failed gate receipts include a byte-exact hex tail of the observed output and
the digest of the full bounded output. Dependency, Apple-framework, and
resource failures are classified as unavailable; they are not code targets.

An image may set `ARES_FRONTIER_PROPOSER_BIN` to a native Linux executable.
The worker sends a bounded binary `FRQ1` frame over stdin and validates a
binary `FRS1` selection frame from stdout. See
[the idle frontier design](<../Ares WebSite : App/backend/IDLE-FRONTIER-DESIGN.md>)
for the wire fields.
Only a target with a recorded gate failure may be selected. The returned model
identity is labeled a claim; this path does not attest weights or apply code.
An invalid or timed-out proposer yields `target=none` with an explicit reason.
Without a proposer, the worker chooses the first failing target by canonical
ID. No Python or JSON enters the proposer boundary.

Build the image from a committed canonical repo snapshot using the
`frontier-worker/Dockerfile`. The backend supervisor expects the image to
contain `/usr/local/bin/frontier-worker`. The worker runs without network,
host mounts, or store credentials. An unavailable offline dependency is an
environment fault, not a source optimization target.

The next stage must preregister a quality floor and drastic bar, supply a
model-neutral proposer/implementer inside the container, iterate against real
gates, and export its patch and receipts for governed review.

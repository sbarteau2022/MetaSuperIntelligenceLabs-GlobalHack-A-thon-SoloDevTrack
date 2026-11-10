# Isolated Docker discovery receipt — 2026-11-10

Source commit: `26dfa6ddf073f986fecb4037b1aa20d9f7b5633c`.
Worker image: `ares-frontier-worker:26dfa6d`, inspected image ID
`sha256:35c53694215567e72c51f2e4065be46078689b33215283ddf4f13ce065c028be`.

The host streamed `git archive` of that exact commit through `colima ssh --
docker run` into a disposable, read-only, network-disabled container with no
host mount or database credentials. It used a temporary anonymous `/work`
volume, `--memory 768m`, `--cpus 2`, and one Cargo build job. The image held
prefetched crates for offline checks. The worker ran the real committed source;
its exact bounded stdout is in `manual-discovery-2026-11-10.receipt.tsv`.

Observed result: backend `cargo check --tests --offline` exit 0;
GlobalMind `cargo check --lib --offline` exit 0; `core_lm` marked unavailable
because its Apple framework cannot link in Linux. No portable failing gate was
admitted as an improvement target. The earlier false target caused by a
read-only `/tmp` linker failure was corrected by setting `TMPDIR=/work/tmp`;
missing offline crates were corrected by fetching all registered manifests in
the image. This is target-discovery evidence, not autonomous code improvement.

After the run, `docker volume ls -q | wc -l` returned `6` (same count observed
before the run), and `docker ps -a --filter name=ares-frontier` returned no
container. The three pre-existing Colima services remained up. A subsequent
`MemAvailable` probe returned `516036 KiB`, below the supervisor's
`786432 KiB` Colima admission threshold, so the live idle supervisor would
refuse another job on this configuration.

Open gates: model-neutral proposal and implementation adapter, preregistered
quality/performance bar, iteration loop and patch export, macOS hardware
qualification, and a full unattended supervisor run with status and receipt
persistence. No live backend was restarted to claim those gates.

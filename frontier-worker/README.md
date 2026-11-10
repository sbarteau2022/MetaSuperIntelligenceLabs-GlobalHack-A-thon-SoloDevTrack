# Frontier worker

This Rust worker is the model-neutral discovery stage of the idle frontier
loop. It receives a committed Git snapshot over the Docker container's stdin,
checks registered Rust components against their actual source, and emits
bounded gate and target-decision receipts. It does not edit code, claim a
drastic improvement, or promote anything to the live checkout.

Build the image from a committed canonical repo snapshot using the
`frontier-worker/Dockerfile`. The backend supervisor expects the image to
contain `/usr/local/bin/frontier-worker`. The worker runs without network,
host mounts, or store credentials. An unavailable offline dependency is an
environment fault, not a source optimization target.

The next stage must preregister a quality floor and drastic bar, supply a
model-neutral proposer/implementer inside the container, iterate against real
gates, and export its patch and receipts for governed review.

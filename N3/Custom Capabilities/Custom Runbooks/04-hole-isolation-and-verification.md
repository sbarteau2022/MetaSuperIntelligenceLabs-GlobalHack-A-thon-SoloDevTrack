---
name: hole-isolation-and-verification
description: Execute untrusted edits and test suites inside seatbelt-isolated hole cavities with SHA-256 Merkle verification and approval gates.
depends_on: [runbook-plan-and-judges, two-strike-tactical-escalation]
load_mode: lazy
---

# Isolated Execution Cavities ("Holes") & Merkle Merge Gates

Decouple the reasoning brain from the execution hands: never mutate the primary project root directly during multi-agent swarm runs.

## Hole Lifecycle

1. **Create Isolated Cavity (`POST /holes`)**:
   - Clones the project tree into an isolated working directory and records `source_fingerprint` and `source_merkle_root` (RFC 6962 domain-separated SHA-256).
2. **Seatbelt Execution (`POST /holes/run`)**:
   - Runs build and test commands inside the hole under macOS `/usr/bin/sandbox-exec` (`deny network*`, writes confined strictly to `hole_root` and `/private/tmp`).
3. **Inspect Unified Diffs (`GET /holes/diff`)**:
   - Review exact `+added / -removed` line deltas across modified files before requesting merge.
4. **Two-Step Merge Gate (`POST /holes/merge-request` → `POST /holes/approve` → `POST /holes/merge`)**:
   - Requires `tested_pass` (exit code `0`), explicit approval (`approved: true`), and verifies that `source_merkle_root` has not drifted since hole creation (optimistic concurrency guard).

# Condensed Doctrine (01-06, applied to runbooks 07-12)

Each doctrine below is condensed to its binding rule. Full doctrine files 01-06 remain the source; this page is the applied form, not a replacement.

**Plan-and-judges (condensed):** Work is DAG packets, never an unbounded loop. PLAN lane decomposes into atomic packets with contracts, depends_on edges, declared context/tools, and deterministic judges. CODE lane claims one ready packet at a time. No self-graded completion — a packet passes only when all declared judges pass.

**Two-strike (condensed):** Never retry identically. Strike 1: pause, read the receipt, change approach so retry_signature changes, or the retry is rejected. Strike 2: mandatory pause and escalation, packet locks, dependents skip. A 3rd attempt needs orchestrator guidance + a preregistered plan.

**KV-cache (condensed):** Stable first, volatile last — keep identity and tool schemas at the prefix root, volatile context at the tail, so the radix prefix stays warm. Checkpoint at ~60% context, before 88% auto-compaction, with a 5W summary.

**Hole isolation (condensed):** Never mutate the primary root directly on untrusted work. Execute in a seatbelt hole cavity, record source fingerprint + Merkle root, merge only through the Merkle gate with approval.

**Rust/Tokio (condensed):** Bounded concurrency behind semaphores, blocking I/O on spawn_blocking, explicit timeouts and byte caps on every external boundary. State in an append-only ledger, no FFI hops in core contracts.

**Agent experience (condensed):** Every token must earn its keep. Start with minimal context, load detail on demand, return bounded outputs with stable ids. No unpruned log dumps.

# queues: at-least-once is the contract, idempotency is the defence
**Done means these pass (broker-agnostic):** consumer idempotency test (deliver the same message twice, assert same effect), redelivery test (kill a handler mid-flight, assert the message is reprocessed, not lost), DLQ inspection (poison message lands in the DLQ with headers/payload intact and the alert fires). See also: postgres.md (rule 4 — `ON CONFLICT` idempotent writes) and tokio.md (rules 4-5 — deadlines and cancel-safety for handlers).
## Rules
1. Consumers are idempotent: processing the same message twice has the same effect as once; enforce with dedupe keys + unique constraints (`ON CONFLICT DO NOTHING`, see postgres.md rule 4), never with "it will only arrive once" optimism.
2. Assume at-least-once delivery; never assume exactly-once — no broker here guarantees it end-to-end, so every side effect must tolerate repeats.
3. Every queue gets a dead-letter queue (DLQ) with an alert on arrival; redelivery counts are bounded (e.g. max 5) before routing to the DLQ.
4. Visibility timeout (SQS) / ack deadline / prefetch-nack window exceeds max handler time with margin (at least 2x p99); renew the lease for long handlers, never let a live handler lose its claim.
5. Retry with exponential backoff plus jitter; no hot retry loops — a failing consumer backs off, it does not spin against the broker.
6. Message schemas are versioned (explicit `v` field / subject version); consumers ignore unknown fields, reject unknown versions to the DLQ, never crash the loop.
7. Enforce a payload size cap (broker default or smaller, e.g. 256KB); large blobs go to object storage with a pointer in the message — oversized payloads kill throughput.
8. Ordering is claimed only where the broker guarantees it (SQS FIFO group, Kafka partition key, NATS JetStream subject ordering, RabbitMQ single-consumer classic queue) — everywhere else, design for unordered redelivery.
9. Handlers are cancel-safe: a dropped future / killed worker must leave the message un-acked and redeliverable, never half-applied and acked (see tokio.md rule 5).
10. Ack only after the effect is durable; an ack before commit is an invisible failure — the message is gone and the work never happened.
## Pitfalls
Poison messages without a DLQ spin forever and starve the queue; ack-on-receive (auto-ack) loses work on every crash; clock-based dedupe windows expire and let duplicates through after the window; unordered redelivery breaks read-modify-write consumers that assumed sequence; oversized payloads saturate broker memory and collapse throughput for all queues; NATS (NEEDS-VERIFY: JetStream retention/ack-policy names per deployment), RabbitMQ (NEEDS-VERIFY: quorum vs classic queue semantics per version), SQS (NEEDS-VERIFY: standard vs FIFO limits per region) — broker specifics unverified here, no broker install required for this contract-first skill.
## Versions
Pinned: broker-agnostic contract (no broker install required). Floor: none — contract applies to any broker.
Rule: read the project's pin (broker type + version in infra config / docker image) before writing consumer code; follow the deployed broker's documented delivery, ordering, and DLQ semantics. When a broker version is pinned by a project using this skill, record it here; re-verify broker-specific notes against that broker's release notes when the pin changes or the check date is over 12 months old.
Sources: NATS JetStream docs, RabbitMQ quorum-queue docs, SQS developer guide (all NEEDS-VERIFY against deployed versions).
Date: 2026-10-30
Status: active

Canonical source: `plugins/web-development/middleware/languages/queues/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/queues.md` (reference; do not author).

# Q32.31 bounded resource policy — component receipt

Scope: policy selection only. The prior ratio reward is not equivalent to a
free-energy difference objective (see `report.tex`). This implementation
does not claim thermodynamic measurement or live executor control.

Design: `research-grade-harness/ACTIVE-INFERENCE-RESOURCE-POLICY-DESIGN.md`.
All comparisons are signed Q32.31 raw integer comparisons. Quality and gain
are constrained to [0,1]; cost and remaining budget to nonnegative values.
The selector validates every candidate before choosing the cheapest feasible
action. Equal-cost ties resolve by greater gain, greater quality, then ID.

Verification on 2026-11-02:

- `cargo test --test bounded_resource_policy`: 3 passed. Covers quality and
  budget filtering, reversibility, invalid values and units, duplicate IDs,
  zero-cost boundary, and an independent small-grid lexicographic oracle.
- `cargo test --all-targets`: exit 0 (live Surreal bridge remains ignored by
  the default suite).
- `cargo clippy --all-targets -- -D warnings`: exit 0.
- `cargo fmt --check`: exit 0 after formatting.

Open gate: calibrate quality and cost bounds from real receipts, meter the
declared unit during execution, and cancel an overrun before calling this a
live active-inference controller.

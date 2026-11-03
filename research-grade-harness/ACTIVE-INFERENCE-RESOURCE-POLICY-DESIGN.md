# Bounded resource policy — preregistered 2026-11-02

Status: component proposal. This does not claim a variational free-energy theorem or metered heat. The prior ratio reward and FEP difference objective can disagree; see `../results/forward-recall-math/report.tex`. The existing `PolicyRanker::rank` is retained as historical behavior while a separate fail-closed selector is implemented.

## Domain and objective

Each invocation declares one resource unit and a Q32.31 remaining allocation. A candidate has a stable ID, a lower-bound judged quality as Q32.31, an information-gain estimate as Q32.31, an upper-bound Q32.31 cost in that same resource unit, and reversibility. Admissible candidates satisfy quality lower bound >= floor, cost <= remaining budget, unit equality, and an optional reversibility requirement. Select the admissible candidate with the lexicographically smallest tuple `(cost, -information_gain, -quality_lower_bound, id)`. If the set is empty, return a typed failure; do not raise the budget, synthesize quality, or call a model. Zero cost is valid only if a caller can substantiate it; this component does not certify forecasts or metering.

## Proof obligation and falsifiers

For finite candidates, a total lexicographic order has a minimum. Filtering first makes the selected action budget- and quality-admissible. Minimizing the first coordinate makes it minimum-cost among admissible actions; subsequent coordinates only break equal-cost ties. A negative result is mandatory if the quality floor blocks every candidate. Counterexamples to catch: cheapest action below quality floor; wrong unit with zero cost; equal cost requiring deterministic information/quality/ID order; no candidates; quality outside [0,1] in Q32.31; duplicate IDs. A small independent brute-force reference enumerates Q32.31 candidate tuples on bounded raw-integer grids and checks the selector's result.

## Limits

This is predicted-resource policy over one declared unit, not thermodynamic measurement. It becomes a live active-inference controller only when candidate quality bounds and cost bounds come from receipted calibration, the executor meters the same unit, and an overrun cancels work. No Landauer value is used as a spendable budget. The user-requested no Python/JSON model I/O boundary remains intact.

All policy magnitudes use signed Q32.31 (`raw / 2^31`) and raw-integer comparisons. Quality and information gain must lie in [0,1]; cost and budget must be nonnegative. No float conversion or mixed denomination enters selection. Values too large for Q32.31 must be represented as a different declared unit and calibrated before use.

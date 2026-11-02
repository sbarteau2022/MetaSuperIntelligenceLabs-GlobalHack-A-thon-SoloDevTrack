# Pre-Registration.md — Predictions Before the Next Run

Date: 2026-10-30. Status: PROPOSED. Written BEFORE any harness executes these hooks.

## Predictions

1. `audit.py` will stay green on our own tree (exit 0) as long as every new file is written through the skill checklists. First red flag expected: a literal hex in a new CSS file (rule `no-hardcoded-color`, warn-level).
2. First deny expected from `pre_tool_guard.py`: a `.env` write or a hardcoded password in a backend template (rules `no-env-commit`, `no-hardcoded-password`).
3. The stop-gate will veto at least one early completion in the next 5 substantive tasks, most likely on `no-todo-stub` or `no-console-log` warnings.
4. The contract gate (§6.3) will be the most-skipped rule under time pressure — watch for parallel coder work starting before contracts land.
5. `openapi.yaml` will fail a strict parser on first `yaml.safe_load` (eye-checked only). Probability INFERRED moderate — indentation was hand-verified, anchors/aliases unused.

## Judges (how each prediction is settled)

- Predictions 1–3: checker output (`audit.py` exit code + JSON; hook stdout payloads). Paste the output, don't summarise.
- Prediction 4: review the task transcript for contract-before-fanout ordering.
- Prediction 5: `python3 -c "import yaml; yaml.safe_load(open('templates/api/openapi.yaml'))"` — REPORTED result decides.

## What would falsify this design

- A task completed faster WITHOUT the calibration line and with equal quality (kills §7).
- A deny-rule firing on legitimate vendored/reference content twice (rule scoping is wrong, not the content).
- Two consecutive stop-gate vetoes on the SAME archived issues (archive logic broken).

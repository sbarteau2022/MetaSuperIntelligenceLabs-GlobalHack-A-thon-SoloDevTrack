# MIGRATION AUDIT — 2026-10-31 (Agent C6, FINAL round, auditor)

- Auditor: Agent C6. Mode: READ-ONLY except this file. Nothing else written, moved, or deleted.
- Scope: every migrated basename (`tool.ts` / `tool.rs` / `SKILL.md` / hook predicates / subagent `.rs` / swarm / decomp / prereg) across three zones: `/Users/stewartbarteau/Desktop/skills` (workspace — contains only `.agents/skills/tool-write/SKILL.md`), `~/.Trash/skills-desktop-*` (archive), `~/Desktop/N3` (live).
- Method: `find` inventory + `diff -rq` + `shasum -a 256` pairwise + RECEIPT cross-check. Pasted outputs below are VERIFIED (measured by this audit). Claims from RECEIPTs are labeled REPORTED. Inferences labeled INFERRED.
- Label key per AGENTS.md section 7: VERIFIED / REPORTED / INFERRED / PROPOSED.

## 1. DUPLICATES — per basename, all locations, sha256, disposition

### D1. Twin trees: `N3/kimi-frontend-kit` vs `N3/web-development/kimi-frontend-kit`

VERIFIED `diff -rq` output (70 lines total): 14 SKILL.md `differ`, 56 files `Only in web-development/kimi-frontend-kit`, ZERO files `Only in N3/kimi-frontend-kit`, everything else identical.

- File counts VERIFIED: `N3/kimi-frontend-kit` = 225 files; `N3/web-development/kimi-frontend-kit` = 281 files. Delta = 56 = exactly 14 RECEIPT.md + 14 tool.json + 14 tool.rs + 14 tool.ts.
- Shared byte-identical files = 225 - 14 = 211 (VERIFIED by elimination from diff output; spot-verified: `hooks/hooks.json` e11c784917ce28bd4f2dc2f04c9a12e5d00ba207e96a021ff91f32ffd227f8c3 both trees; `README.md` 540bb4ba2d04629e9237a111ee8da79ba5e6eca76a4506f34fd11b4b4b47fdd0 both; `rules/rules.md` 8aa5ee110fac9d600df6a5cd05a28724004f2e22a9f09912b9b797aacce81e3d both).
- The 14 SKILL.md pairs all DIFFER (VERIFIED per-file sha, e.g. brutalist Trash/N3-kimi `a3c761afe46c60d52eb3f9945073b8ca4c8d0c2c55adafaed25b12e6ed98afdc` vs web-dev `107904e35af8afdde1c281e7a7baa695b03376e9b8a6178359acd3066b137fe2`). Content direction VERIFIED (brutalist diff): Trash/N3-kimi = full prose discipline doc; web-dev = rewritten interface (`Purpose / When to use / Out of scope / Type signature / Compiled gate`) + RECEIPT + tool triple. Intentional interface evolution, NOT a copy error.
- A2's REPORTED claim ("identical SKILL.mds") is WITHDRAWN by this audit with correction dated 2026-10-31: the SKILL.mds differ; what is identical is the 211 non-interface files.
- Disposition: canonical = `N3/web-development/kimi-frontend-kit` (KEEP — strict superset: all 211 shared files + 14 gated interfaces + 56 port artifacts + RECEIPTs). `N3/kimi-frontend-kit` = DELETE-DEFER — VERIFIED to carry zero unique bytes (no `Only in` files), but deletion is OWNER-DECISION because it is currently the only N3-side pristine snapshot; Trash holds identical originals so recovery is assured either way. Consequence of keeping: 211 live-live duplicate files (~9.8MB kit weight per CLAIMS_REGISTER) and two SKILL.md versions that will drift.

### D2. Trash `skills-desktop-web-development-20261030/kimi-frontend-kit/skills/*/SKILL.md` (14 files) vs both N3 trees

VERIFIED all 14: Trash hash == `N3/kimi-frontend-kit` hash (byte-identical, e.g. brutalist `a3c761afe46c...98afdc` both; full 14-pair table in section 5). Trash copies vs web-dev copies: all 14 DIFFER (interface divergence, section D1).
- Disposition: KEEP as archived (Trash) — these are the declared pre-migration sources. Deletion authority is OWNER-ONLY (section 4, item T10). Consequence of deletion: loss of pristine pre-migration snapshot; `N3/kimi-frontend-kit` currently covers that role byte-identically.

### D3. Trash top-level area `SKILL.md` (10) vs `N3/web-development/*/SKILL.md` (10)

VERIFIED all 10 pairs DIFFER (migration = interface rewrite, not copy). Full sha table in section 5. RECEIPT table-row hashes VERIFIED MATCH Trash for 8/10 (api-integrations, devops-deployment, frontend-design-language, frontend-dev, middleware, security-auth, testing-qa, ui-ux-flow). For backend-systems and database-data the RECEIPT table-row hash VERIFIED == the N3 DEST hash, not the Trash source hash (table cells quote the trimmed target).
- Zone-B RECEIPT label "Trash source ... — VERIFIED" for backend-systems/database-data is therefore INACCURATE as a source-hash claim (dated correction 2026-10-31, this audit). It does not affect the compiled tools (their gates are self-contained) but it voids the audit-trail premise for those two rows. Owner/critic re-verification owed.
- Disposition: both sides KEEP (archived source + live interface — different content, no true duplicate). Deletion of Trash originals: OWNER-DECISION (section 4, T10). Consequence of keeping: none beyond disk; these are declared source-refs.

### D4. Hook predicates `.py` vs compiled `.rs` (hook basename audit)

VERIFIED locations (excluding `__pycache__`/`target`):
- `pre_tool_guard.py` + `session_init.py`: ONLY in `N3/web-development/hooks/scripts/` (`cbbc7f68395a...` / `4bd70854ccf2...`). NOT in Trash (Trash `hooks/scripts/` holds only `_engine.py`, `post_edit_verify.py`, `stop_completion_gate.py` — VERIFIED listing). NOT in `/skills`. Single live copies: NO duplicate.
- `post_edit_verify.rs` + `stop_completion_gate.rs`: TWO copies each — Trash `hooks/.agent4-rs/` and `N3/web-development/hooks/compiled/`, VERIFIED byte-identical (`f1abe556a22c...` both; `d1bc14dc4454...` both). Trash `.agent4-rs/` is a build-staging dir (Agent 4 working copy), not a migration source; N3 copy is canonical.
- `pre_tool_guard.rs` + `session_init.rs`: ONLY in N3 compiled. No Trash counterpart. Single copies.
- Disposition: KEEP all. Trash `.agent4-rs/*.rs` = archived build-staging copies, OWNER-DECISION on deletion with the rest of Trash (T10); no live-live duplication. Consequence of deleting Trash copies: none for the build (N3 canonical verified identical).

### D5. Subagent / swarm / decomp / prereg / loop basenames

VERIFIED locations (excluding `target/`, `__pycache__`):
- `subagent_definitions.json` + `SUBAGENT_SWARM_LOGIC.md`: FOUR identical copies each — Trash kimi `subagents/`, `N3/kimi-frontend-kit/subagents/`, `N3/web-development/kimi-frontend-kit/subagents/`, `N3/staging-2026-10-30/subagents/` (VERIFIED `bd455657f95736278bf0ee67c11b7a6a1dfdef95bb4d35d4f2a93734a1101bfb` / `0314a6921ae09928efc1859821ea46476b2f1789cfacd5b899ce93aa2b990284` all four). One Trash archive + two N3 live + one N3 staging. Live-live duplication between the two N3 copies is real (same consequence as D1, resolved by the D1 canonical decision).
- `ok_computer_loop.rs`: TWO copies, VERIFIED identical (`66448b45c461af7b...`) — Trash `kimi-frontend-kit/workflows/compiled/` + N3 `web-development/workflows/compiled/`. Note: workflows RECEIPT claims a source `.md` was DELETED post-receipt from `/skills` (REPORTED, pre-existing `/skills` path outside audit reach to re-verify; no `.md` duplicate remains in N3 — VERIFIED).
- `swarm.rs` / `swarm.json` / `subagents.json` (compiled registry): ONLY in `N3/web-development/{swarm,subagents}/compiled/`. `decomposition.rs` / `preregistration.rs`: ONLY in `N3/web-development/decomposition/compiled/`. No Trash counterparts found (VERIFIED find). Single copies.
- cognitive-map slice2 (`N3/cognitive-map/slice2/`): VERIFIED contains zero migrated basenames (no `SKILL.md`/`tool.*` anywhere under `N3/cognitive-map`). Slice2 is source-ref/integration context only — intentionally kept, no action. Its content was not hashed (out of basename scope by design).
- Disposition: `subagent_definitions.json` / `SUBAGENT_SWARM_LOGIC.md` — KEEP Trash archive (OWNER-DECISION on deletion, T10); N3 live copies follow D1 canonical; `N3/staging-2026-10-30` copies — OWNER-DECISION (whole staging dir is pre-migration scaffolding, 708 files, untouched by this audit). All single-copy compiled `.rs`/`.json` — KEEP. Consequence of the current 4x `subagent_definitions.json`: edit-drift risk; canonical-home rule should name one.

### D6. `tool.ts` / `tool.rs` / `tool.json` port triples

VERIFIED: port triples exist in exactly one live location each — 13 areas under `N3/skills/*/`, 10 areas under `N3/web-development/*/compiled/`, 14 skills under `N3/web-development/kimi-frontend-kit/skills/*/`, 3 under `N3/plugins/mcp-tools/*/`. No basename collision across zones (each triple is namespaced by its area dir), no second live copy of any triple, nothing under `/skills` (VERIFIED: workspace holds only `.agents/skills/tool-write/SKILL.md`). Trash holds the `.py`/`.md` sources, never `.ts`/`.rs` ports. NO true duplicates in this class.
- C1 `carve-specialist` triple (`N3/plugins/mcp-tools/carve-specialist/`): single copy, hashes match RECEIPT (REPORTED hashes assumed; N3-side re-hash NOT performed — listed NOT-checked). Trash sources (`carve-specialist.py` / `.json`) VERIFIED present with hashes EXACTLY matching C1 RECEIPT (`70518e67...35593bf2` / `99e66702...ad89cc98`).
- Disposition: KEEP all triples. No action.

### D7. `/skills` workspace

VERIFIED: `/Users/stewartbarteau/Desktop/skills` contains only `.agents/skills/tool-write/SKILL.md` (single file). No migrated basenames, no duplicates, no writes made by this audit.

## 2. TWIN-TREES quantified (`diff -rq`)

VERIFIED `diff -rq N3/kimi-frontend-kit N3/web-development/kimi-frontend-kit` = 70 lines: 14 `differ` (all SKILL.md), 56 `Only in web-development/kimi-frontend-kit` (14 RECEIPT.md + 14 tool.json + 14 tool.rs + 14 tool.ts), 0 `Only in N3/kimi-frontend-kit`. 211 remaining files identical (spot-verified section D1).
- Correction to A2 (dated 2026-10-31): SKILL.mds are NOT identical; they are 14/14 rewritten interfaces. Identical content = the 211 support files.
- Canonical recommendation (PROPOSED, owner decides): `N3/web-development/kimi-frontend-kit` canonical (strict superset + gates + RECEIPTs). `N3/kimi-frontend-kit` DELETE-DEFER — zero unique bytes VERIFIED, but keep until owner rules, since it is the only N3-side pristine snapshot of the 14 originals (Trash holds identical originals, so recovery is assured). Consequence of deferral: ~9.8MB live-live duplication + two SKILL.md versions per skill that will drift.
- Pasted evidence: full 70-line diff captured in audit transcript (tool output, 2026-10-31); per-file SKILL.md sha table in section 5.

## 3. GAPS — per-item disposition (no bulk anything)

- G1 `carve-specialist` (C1 in flight at audit start): CLOSED during audit. VERIFIED `N3/plugins/mcp-tools/carve-specialist/` contains `tool.ts`, `tool.rs`, `tool.json`, `RECEIPT.md` (dated 2026-10-31, C1 FINAL). Trash sources present with EXACTLY the RECEIPT-claimed hashes. N3-side re-hash NOT performed (NOT-checked). Disposition: KEEP. Consequence: hyperbolic-active-mind's A2-era "removed/non-ported" note (section `hyperbolic-active-mind/RECEIPT.md:16`) is now STALE — carve-specialist IS ported; that RECEIPT line needs a dated amendment by its owner, not by this audit.
- G2 `no-div-button` (C3 in flight): OPEN, no receipt landed. VERIFIED: rule exists in `hooks/rules-table.json` (deny, scope tsx/jsx/html/vue/svelte, pattern `<div[^>]*role\s*=\s*["']button["']`) and in Python `_engine.py` scan path; ABSENT from compiled `pre_tool_guard.rs` (8 deny rules: no-div-button missing; `hook.json` says "8 rules"). So the live `.py` gate covers it, the compiled `.rs` gate does NOT. Also uncompiled: all 15 `rules-languages.json` rules (4 deny: py-exec-eval, py-sql-concat, rs-unwrap, yaml-secret) live only in the Python path. Disposition: OWNER-DECISION (port `no-div-button` into `pre_tool_guard.rs` as 9th deny rule + amend `hook.json` "8 rules" text, or formally scope compiled gates to `rules-table.json`-deny-minus-div-button with a dated waiver). Consequence of inaction: compiled pre-guard allows div-as-button while the `.py` gate denies it — enforcement depends on which gate runs.
- G3 `no-debugger` rule coverage: COVERED in both paths (VERIFIED: `rules-table.json` deny + `pre_tool_guard.rs` `id: "no-debugger"` + `find_match` arm). No decoder-ring gap. The task-list phrasing is answered: rule coverage VERIFIED present. Disposition: KEEP, no action.
- G4 `rules-languages.json` 15 extra rules vs compiled ports: GAP CONFIRMED and widened. VERIFIED: 15 rules total (11 warn + 4 deny: py-exec-eval, py-sql-concat, rs-unwrap, yaml-secret); ZERO are compiled into any `.rs` (compiled gates = `rules-table.json` only: 8 deny + 9 warn). Python `_engine.py` loads the union (VERIFIED: `RULES_LANGUAGES` + merge). So compiled parity = `rules-table.json` only; language rules are Python-only. Disposition per rule severity — deny (4): OWNER-DECISION to port or waive (same form as G2; py-exec-eval/py-sql-concat are injection-class, rs-unwrap panics, yaml-secret leaks — waiving needs a dated reason each); warn (11): OWNER-DECISION, lower urgency (post-write advisory only). Consequence of inaction: any consumer of the compiled gates alone silently loses 15 rules, 4 of them deny-class.
- G5 `discipline.py` D4 fail (Agent 3 flagged: hook `.py` paths migrated): RESOLVED — claim is STALE. VERIFIED 2026-10-31: full `discipline.py` run shows ZERO D4 lines (all HOOK_FILES present: config, schema, both tables, audit.py, audit.sh, `_engine.py`, all 5 scripts); 22 fails are all D2 (10: section-order/conventions on area SKILL.md) + D9 (12: missing languages/plugin.json + 11 language skills), 10 warns D3 (no Date lines). D4 passes because the `.py` hook files were never deleted — the RECEIPT "DELETED post-receipt" note (hooks/compiled/RECEIPT.md section 6) refers to a stale stash-restore story contradicted by the live tree; deletion did not happen. Disposition: KEEP `.py` files (they are the gates that actually enforce all 33 rules); amend the stale RECEIPT deletion note (owner of that RECEIPT, dated amendment). Consequence: none for enforcement; audit-trail hygiene only. New finding from the same run: D2/D9/D3 fails are pre-existing and out of C6 scope — recorded, not fixed.
- G6 `hooks.json` still pointing at `.py`: CONFIRMED, and CORRECT. VERIFIED `N3/web-development/hooks/hooks.json` commands are `python3 skills/web-development/hooks/scripts/*.py`, and the `.py` files EXIST (all 5 + `_engine.py`). The compiled RECEIPT itself records harness rewiring as OUT OF SCOPE (no harness contract for compiled-Rust invocation on disk). Disposition: KEEP `.py` references (they resolve); compiled `.rs` invocation contract is a separate OWNER-DECISION (needs a harness contract first; inventing one would violate RULES.md rule 75 per the RECEIPT). Consequence of rewiring without a contract: silent gate bypass.
- G7 Trash deletion authority (all agents deferred): compiled approved-deletion list below from RECEIPTs. Note the list is a COMPILATION, not an approval — every entry defers to owner per the RECEIPTs themselves.
  - T1–T10. Ten Trash area SKILL.md (`api-integrations`, `backend-systems`, `database-data`, `devops-deployment`, `frontend-design-language`, `frontend-dev`, `middleware`, `security-auth`, `testing-qa`, `ui-ux-flow` under `~/.Trash/skills-desktop-web-development-20261030/`): all 10 area RECEIPTs section 5 defer Trash-original deletion to OWNER decision ("see approved-deletion list in wave report"); no wave report found on disk (VERIFIED find, no `*WAVE*`/`*CLOSURE*`/`*migration*report*` at N3 depth ≤3). Disposition: OWNER-DECISION, per-item (10 separate decisions). Consequence of deletion: loss of pre-migration sources; mitigated only where N3 holds identical bytes (kimi 14 — yes; area interfaces — no, they are rewrites).
  - T11. Trash `atoma-brain/tools/carve-specialist.py` + `.json`: C1 RECEIPT: "Trash-source deletion NOT performed — owner decision pending". Disposition: OWNER-DECISION. Consequence: source/provenance loss; C1 hashes recorded in RECEIPT.
  - T12. Dream-consolidate / checkpoint-status originals: Wave-1 Agent 2 RECEIPT claims "Deletion of originals: DONE ... No duplicate basename remains under /skills" (REPORTED, pre-existing `/skills` unreachable to re-verify; Trash plugins dir holds no dream/checkpoint `.py` — VERIFIED absent from Trash listing). Disposition: NO ACTION (already done per REPORTED receipt; nothing to delete).
  - T13. Workflows `KIMI_OK_COMPUTER_LOOP.md`: RECEIPT claims deleted post-receipt (REPORTED); VERIFIED absent from Trash and N3 (find clean). Disposition: NO ACTION.
  - T14. Trash `hooks/.agent4-rs/*.rs` (post_edit_verify, stop_completion_gate — byte-identical staging copies): no RECEIPT mentions them. Disposition: OWNER-DECISION with T1–T10 batch (same Trash dir). Consequence: none (N3 canonical verified identical).
  - T15. Everything else in `~/.Trash/skills-desktop-*` (855-file web-dev tree, 65-file plugins tree, 434-file core_lm, harness/immune/orchestrator/vm/system dirs, top-level runbook/pillar `.md`s): NO RECEIPT proposes deletion. Disposition: OWNER-DECISION, explicitly NOT approved by this audit. Consequence of bulk deletion: unrecoverable provenance loss — no bulk anything.

## 4. RECEIPT-hash integrity spot-check (new finding, dated 2026-10-31)

- A2 zone-C kimi RECEIPTs (13/14 sampled): recorded source-hash strings are TRUNCATED relative to real sha256 in 12 of 13 checked (e.g. brutalist RECEIPT has 61 hex chars, Trash file hashes to 64: `...5adafaed25b12e6ed98afdc` vs `...5a...`; only cortex-react matches at full 64). The 12 truncated strings are PREFIX-CONSISTENT with the true hashes (INFERRED copy/paste truncation, not wrong-file — first N chars match). Zone-B area RECEIPTs use a different (table-cell) format and 8/10 MATCH fully; backend-systems/database-data cells quote the N3 DEST hash instead of the Trash source (section D3 correction). C1 carve-specialist source hashes VERIFIED full-64 EXACT match. Hyperbolic-active-mind 64-char MATCH.
- Disposition: REPORTED-to-owner; re-verification of the 12 truncated A2 hashes owed by receipt owners (dated amendment, one line each). Enforcement impact: none (compiled tools self-contained). Audit-trail impact: prefix-only hashes are not proofs.
- Pasted evidence: `MISMATCH brutalist-frontend len=61 rec=a3c761afe46c60d5... cur=a3c761afe46c60d5...` (audit transcript); full per-area table: `MATCH cortex-react`, lengths 37–63 across the other 12.

## 5. Hash + find evidence (VERIFIED, pasted)

### 5a. Twin-tree SKILL.md (Trash == N3/kimi, both != web-dev)
| skill | Trash == N3/kimi sha256 | web-dev sha256 (differ) |
|---|---|---|
| brutalist-frontend | `a3c761afe46c60d52eb3f9945073b8ca4c8d0c2c55adafaed25b12e6ed98afdc` | `107904e35af8afdde1c281e7a7baa695b03376e9b8a6178359acd3066b137fe2` |
| cortex-react | `7ed463b4c6f4e70231267c0997ed956d81e158e9b64f24cf69f6524ea47b12d6` | web-dev differs (per-file sha in transcript) |
| cortex-typescript | `71c37d1f7a9fadeb889731d998e11cfc6476e1616effecd1b7780bce4be7c211` | differs |
| design-system | `8aa5ff93320559a6e5289d37b6b06b3307737dbfb4a6f3fc44135c705613c511` | differs |
| design-taste-frontend | `aa194351b246b8b4799099d4ed7b033d29eab6e6e3d58d8d2172978be7b3ec89` | differs |
| frontend-design | `d91970639e9f5c37682ac7ab60094d35f1c7c1f38d731bd56396563aee10c1d3` | differs |
| frontier-stack-2026 | `048b23ff89fa434bf8758e3d28edc388c324ae7da89f27fb194615fa7c2c0bf5` | differs |
| high-end-visual-design | `e1e32f5e2d420872c6c7332b53d5ff7721946766b78c4822b424c2d512c8fdbc` | differs |
| ide-dockview-editors | `cf845b265bde0056881391f351f04fabf41d6a4e8aabd92408e316f3fe7bce27` | differs |
| kimi-frontend-orchestrator | `da6717413f36a2d3a85a6d3d6de7da8c4546a854d9c73a38d90bdac891f55e8c` | differs |
| minimalist-frontend | `206830251b0515c1926e572c03378590fa15d0a7557734b668a35db358694274` | differs |
| ui-styling | `625e302c60ee8f2e22d56cbe638c4a739e2657f566dae42689503ad517c30f1e` | differs |
| ui-ux-pro-max | `ea087c341bfb5b23195c7302027268ede86da802554c18a5c4896a6017b439f9` | differs |
| webgl-r3f-graphs-dags | `2f8e12181cabfcc71daef61f6e1383977bddc9c246881c690c7b8cda85a52666` | differs |
All 14 Trash==N3/kimi pairs VERIFIED byte-identical by `shasum -a 256` (full two-path output in transcript).

### 5b. Area SKILL.md (Trash source vs N3 dest — all DIFFER, interface rewrite)
| area | Trash source sha256 | N3 dest sha256 |
|---|---|---|
| api-integrations | `94d21f13f5ba00de92f4bd3f578ef60d5d34d2956ff013c5446344f0f1b2530e` | `f97701fa7541a6c10c4bfff3f3448377fdae589f52bda7370eba4329262d7d66` |
| backend-systems | `cbf0f4f2410ffb9400d82f777e286706d4d1c6d3eb4412ecc62241a82a50918d` | `af98974c4b9af7d5666957604bdf312afbd45bda6a2ca76ce358fe0e1c46ab39` |
| database-data | `6da13048969f88a6f85c56f152c827b10fdf64d215402677116dc4aca3509833` | `061d87b407b7b650cbc79bd7f4f72314c04487d7f929e054efea9b00d4f5e7b8` |
| devops-deployment | `06a039adac18447af89f3331b5f86d1482ff0d22f304e4d42a37602c0676b8f7` | `a28a120b12841297f58e4128c1d5f75eb11dae4c6f373894004090cbf448493f` |
| frontend-design-language | `2e72200e411b9d6b2ffdcad82bbd81721c48cdc92edce6afb71458fa6bcaa019` | `06ffe84da0657808398f60ed41f885c5e65a3f05a4170f91e70b83c78355ce0d` |
| frontend-dev | `fadedfe19207e4f80cfc4ce9508b402513a0ecd9f6c2f21432ec1a8faacdcbf2` | `4d44475a64050661cdbbebce2232534d4d356115e853a773b4627cd1bccc356e` |
| middleware | `38b4656533361131fd1ac6b170e3b9ae2723eaed44f49bf095e8dda9f9cc412a` | `9bf6405a970f93727a2e5c85fe8555abcc74f4b80ae9ad51c730f9ec4491b22a` |
| security-auth | `277ea252a5e281e822974f12ac3bf3dd888846086e490bf4521c18ede8690c98` | `84031bbabb081b779ad3638cf75395fc65cc95c902f4980161cfdd79c5b758fe` |
| testing-qa | `72ee11680fbc83f96990c2d3573f758b51f4ac0db9f44392bc930a91e856da21d851beae16230baff` | `aebe9fa8c24d47177709d6db9f44392bc930a91e856da21d851beae16230baff` |
| ui-ux-flow | `20837ddd409788fd0a58dad4dc3f48fcd39a1d9555177244290947d3ae7377e8` | `b247b76d5830314be53c264fc7e33443baeb47f29a7dffc747a487c5b9bd0545` |

### 5c. Hook / subagent / loop / carve identity proofs
- `post_edit_verify.rs`: Trash `.agent4-rs/` == N3 compiled, `f1abe556a22cda3527ae6588da59fc0aed5a664aee8464b5f3df4397dbe545c9` both.
- `stop_completion_gate.rs`: Trash `.agent4-rs/` == N3 compiled, `d1bc14dc445474ae69ac4ebc732a006a07bb570a70416b041e5ef411da6f6fe0` both.
- `subagent_definitions.json` ×4 identical: `bd455657f95736278bf0ee67c11b7a6a1dfdef95bb4d35d4f2a93734a1101bfb` (Trash kimi, N3/kimi, web-dev/kimi, staging).
- `SUBAGENT_SWARM_LOGIC.md` ×4 identical: `0314a6921ae09928efc1859821ea46476b2f1789cfacd5b899ce93aa2b990284` (same four).
- `ok_computer_loop.rs` ×2 identical: `66448b45c461af7b63f3f6eb9a5ac76a87b458bc04fc445286a6f53672c5d328` (Trash kimi workflows/compiled, N3 web-dev workflows/compiled).
- `carve-specialist.py` Trash: `70518e67742542b6c8e7d2fe5f396012a8ed0a8fd36e4ca9b8a0275a35593bf2` == C1 RECEIPT claim (MATCH). `.json`: `99e6670252eccd7dae77575bde16ec0216a585972f0942a01a4996ad89cc98` (MATCH).
- Shared twin-tree files: `hooks/hooks.json` `e11c784917ce28bd4f2dc2f04c9a12e5d00ba207e96a021ff91f32ffd227f8c3` both; `README.md` `540bb4ba2d04629e9237a111ee8da79ba5e6eca76a4506f34fd11b4b4b47fdd0`-class (full in transcript); `rules/rules.md` `8aa5ee110fac9d600df6a5cd05a28724004f2e22a9f09912b9b797aacce81e3d` both.
- `find` for every `pre_tool_guard*`/`session_init*` basename (minus pycache/target) returns exactly the 4 N3 files; no Trash copies. `find` for single-copy compiled `.rs`/`subagents.json`/`swarm.json`/`decomposition.rs`/`preregistration.rs` returns exactly one location each. cognitive-map `find` for `SKILL.md`/`tool.*` returns empty.

## 6. Summary counts

- Duplicates: TRUE live-live duplicates = 211 identical support files + 4 identical subagent/logic files (2 basenames × 2 N3 copies) + 0 identical SKILL.md (14/14 differ) + 0 port-triple collisions. Archived (Trash, KEEP): all Trash sources + `.agent4-rs` staging pair + staging-2026-10-30 copies. Source-refs (intentionally kept): cognitive-map slice2 (zero migrated basenames VERIFIED) + Trash pre-migration originals.
- Gaps: 7 items — CLOSED 2 (G1 carve DONE; G5 D4 stale-claim resolved), COVERED-NO-ACTION 1 (G3 no-debugger), OPEN 2 (G2 no-div-button compiled gap; G4 15 language rules Python-only, incl. 4 deny), CORRECT-AS-IS 1 (G6 hooks.json → .py), COMPILED-NOT-APPROVED 1 (G7 Trash deletion list T1–T15).
- Owner decisions: D1 twin-tree deletion (defer/keep); G2 port-or-waive no-div-button; G4 port-or-waive 15 language rules (4 deny urgent); G6 compiled-invocation contract; G7 T1–T11/T14/T15 per-item Trash deletions; section 4 receipt amendments (backend/database rows, 12 truncated A2 hashes, stale hyperbolic carve note, stale hooks deletion note).
- NOT checked: live `:8088`/network/MCP round-trips; N3-side re-hash of C1 carve triple and A2 tool triples (REPORTED hashes assumed); `staging-2026-10-30` 708-file content (only subagent pair hashed); `deepseek-harness-20261030` Trash content (out of scope); pre-existing `/skills` paths named in Wave-1 RECEIPTs (workspace holds one file only); D2/D9/D3 discipline fails (pre-existing, recorded section G5); repo-wide filename-reference grep for deleted loop `.md`.

## Directive entry — 2026-10-31 (owner, supersedes any prior deletion timing)

- Archives in ~/.Trash/skills-desktop-*-20261030 STAY through submission. No T1–T15 deletions before Nov 4.
- After Nov 4: T1–T15 as a reviewed list, one pass, per-item disposition, sources last.
- Space policy before then: if space ever needed, delete rebuildables first (target/, node_modules) — never skill/source archives.
- Status 2026-10-31: 16 Trash archive entries present, /skills holds only .agents/, disk 391Gi avail — no pressure, no deletions performed.

# TPE-vs-RoPE End-to-End Run Report (2026-11-01)

## Corpus (VERIFIED)
REAL Qwen3-Embedding-8B DWQ `embed_tokens` lookups, 4 seqs N=15..19, dim 4096,
head slice = first 64 dims, RMSNorm (eps 1e-6 per model config) pre-arm.
Fixture: `analysis/tpe_e2e/frozen_hidden.json` (sha256 sidecar, verified OK at
bench startup: `2e98caa8...42bad2ca`).

## Prereg (condensed — full text in session)
Arms: EXACT (causal softmax) / TPE (TpeState carrier + rank-5 scan) / ROPE
(rotate-half, theta=1e6). Judges: J1 max rel TPE/EX <1.0; J2 mean TPE/RO >1e-6;
J3 report RO/EX; J4 prefix-causality <1e-9.

## Result (VERIFIED — `cargo run --offline --bin tpe_e2e`, exit 1)
| seq | N  | J1 max  | J2 mean | J3 mean  | J4 max | verdict |
|-----|----|---------|---------|----------|--------|---------|
| 0   | 15 | 0.754   | 0.499   | 0.00249  | 0.0    | PASS    |
| 1   | 19 | 2.00e9  | 1.05e8  | 0.00272  | 0.0    | MISS    |
| 2   | 17 | 0.587   | 0.457   | 0.00208  | 0.0    | PASS    |
| 3   | 17 | 0.731   | 0.473   | 0.00224  | 0.0    | PASS    |

OVERALL: MISS (recorded, no tuning).

## Seq-1 t=4 mechanism (VERIFIED, two independent fibers + vault mirror)
Phase-reversal denom pin at ORDINARY radii: query c1=-1.000 vs accumulated
zc1[19]=+10.458 (undecayed, gamma=1.0) overwhelms z0=7.5 -> raw denom -0.216
-> floor 1e-9 -> |y| = 1.6e10. r_iso max 1.98 (saturation hypothesis DENIED).
Vault kernel ITSELF pins under byte-faithful semantics (f32/f64, gamma
1.0/0.9995) — transcription exonerated. Evolve exonerated (spike identical
pre/post drive rescope; single-wrap correct in (-3pi,3pi)).

## Corrections log
1. 2026-11-01: norm-ratio prediction ("scan ~100x smaller") FALSIFIED by
   measurement (ratio was 1.91, opposite sign). Fix (RMSNorm) stood on
   architecture grounds, not the ratio.
2. 2026-11-01: NEAR-ZERO guard tested components not norm — fired spuriously
   on |y_exact|=8.0 rows. Replaced with L2-norm guard; spurious lines gone.
3. 2026-11-01: arg-order suspicion FALSIFIED — max_rel(&tpe,&exact) already
   correct orientation; blowup is numerator-side.
4. 2026-11-01: heterogeneous-r_iso saturation mechanism DENIED by r_iso
   table (max 1.98, zero pairs near 4.0).

## Open (REPORTED, out of scope this turn)
- Vault `shell_expansion` successor kernel (monster-hybrid iteration) NOT
  tested against the pin — different kernel, unknown behavior.
- Submission claim stays: "kernels verified in bench crate; end-to-end shows
  3/4 with one honest vault-faithful denom pin (recorded miss)."

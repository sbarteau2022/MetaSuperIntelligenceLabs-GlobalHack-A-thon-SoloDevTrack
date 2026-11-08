//! The meta loop: the automaton's iterate(plan → kernel → verify) cycle.
//!
//! The model never asserts a number. Each round it proposes independent
//! computational ROUTES to the target quantity; every route is executed on the
//! deterministic math kernel (via `MathKernelLane`); the receipts are folded
//! into held branches under the one free-energy functional
//! (`superposition::fold_kernel_receipts`); and `superposition::deliberate`
//! either COLLAPSES onto a route confirmed by independent agreement, or
//! mandates ITERATION. On iteration the loop asks the caller for more/refined
//! routes, up to a cycle budget, with every round's cost descending the
//! session's free-energy bound.
//!
//! Confirmation = two independent routes producing the same kernel
//! `output_hash`. Independence is in how each route DERIVES its inputs; the
//! canonical value is what must coincide. One route is never enough — a lone
//! hash is unverified and forces another round (H3: dynamic iteration).

use crate::fep::{FreeEnergyTerms, SessionBound};
use crate::lanes::{ContributorLane, LaneVerdict, MathKernelLane, TaskPacket};
use crate::mathkernel::KernelOp;
use crate::superposition::{
    deliberate, fold_kernel_receipts, BranchSource, DeliberationBranch, DeliberationOutcome,
};

/// One independent computational route to the target quantity.
pub struct Route {
    pub id: usize,
    pub source: BranchSource,
    pub op: KernelOp,
    /// Prior complexity (nats) — how far this route moves from the prior.
    pub prior_complexity: f64,
}

pub struct MetaConfig {
    pub cycle_budget: u32,
    pub suppress_delta: f64,
    pub confirm_bonus: f64,
    pub unverified_penalty: f64,
    /// Base inaccuracy every route starts with before verification folds in.
    pub base_inaccuracy: f64,
}

impl Default for MetaConfig {
    fn default() -> Self {
        Self {
            cycle_budget: 6,
            suppress_delta: 2.0,
            confirm_bonus: 3.0,
            unverified_penalty: 5.0,
            base_inaccuracy: 3.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MetaOutcome {
    /// Independent agreement reached: `hash` confirmed by `agreeing` routes.
    Verified { hash: String, agreeing: usize, bound_total: f64, cycles: u32 },
    /// Budget exhausted with no independent agreement.
    Exhausted { reason: String, bound_total: f64, cycles: u32 },
}

/// Run one route on the kernel lane, returning (id, Some(hash)) or (id, None)
/// when the kernel refused (ill-posed / singular).
fn execute_route(r: &Route) -> (usize, Option<String>) {
    let lane = MathKernelLane { op: r.op.clone() };
    let pkt = TaskPacket {
        packet_id: format!("route-{}", r.id),
        lane_id: "math-kernel".into(),
        task: "kernel-op".into(),
        context_json: String::new(),
        budget_tokens: 0,
    };
    match lane.run(&pkt).verdict {
        LaneVerdict::Done { output_hash } => (r.id, Some(output_hash)),
        _ => (r.id, None),
    }
}

fn to_branch(r: &Route, cfg: &MetaConfig) -> DeliberationBranch {
    DeliberationBranch {
        branch_id: r.id,
        source: r.source.clone(),
        terms: FreeEnergyTerms::new(r.prior_complexity, cfg.base_inaccuracy, 0.0),
        prediction_hash: String::new(),
        prediction_ready: false,
    }
}

/// Drive the loop. `propose_more(cycle, &receipts)` supplies additional routes
/// after a non-confirming round (it may inspect the receipts so far); return an
/// empty vec to signal "no further routes" → the loop stops as exhausted.
pub fn run_meta_loop<F>(
    initial: Vec<Route>,
    mut propose_more: F,
    cfg: &MetaConfig,
) -> MetaOutcome
where
    F: FnMut(u32, &[(usize, Option<String>)]) -> Vec<Route>,
{
    let mut bound = SessionBound::new();
    let mut routes = initial;
    let mut cycle = 0u32;

    while cycle < cfg.cycle_budget {
        cycle += 1;
        if routes.is_empty() {
            return MetaOutcome::Exhausted {
                reason: "no routes to execute".into(),
                bound_total: bound.total(),
                cycles: cycle,
            };
        }

        // 1. Kernel-execute every route (deterministic oracle).
        let receipts: Vec<(usize, Option<String>)> = routes.iter().map(execute_route).collect();

        // 2. Fold receipts into held branches under the free-energy functional.
        let mut branches: Vec<DeliberationBranch> = routes.iter().map(|r| to_branch(r, cfg)).collect();
        let agreement = fold_kernel_receipts(
            &mut branches,
            &receipts,
            cfg.confirm_bonus,
            cfg.unverified_penalty,
        );

        // 3. Deliberate: collapse onto a prediction-ready minimum, or iterate.
        let (outcome, _state) = deliberate(
            &[],
            &[],
            branches,
            cfg.suppress_delta,
            &mut bound,
            &format!("meta cycle {cycle}"),
        );

        if let DeliberationOutcome::Collapsed { .. } = outcome {
            if let Some(hash) = agreement.verified_hash {
                return MetaOutcome::Verified {
                    hash,
                    agreeing: agreement.multiplicity,
                    bound_total: bound.total(),
                    cycles: cycle,
                };
            }
        }

        // 4. Not confirmed → ask for more/refined routes for the next round.
        let more = propose_more(cycle, &receipts);
        if more.is_empty() {
            return MetaOutcome::Exhausted {
                reason: "iteration proposed no further routes".into(),
                bound_total: bound.total(),
                cycles: cycle,
            };
        }
        routes = more;
    }

    MetaOutcome::Exhausted {
        reason: "cycle budget exhausted without independent agreement".into(),
        bound_total: bound.total(),
        cycles: cycle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mathkernel::Metric;

    fn route(id: usize, src: BranchSource, op: KernelOp) -> Route {
        Route { id, source: src, op, prior_complexity: 0.5 }
    }

    #[test]
    fn confirms_when_two_routes_agree_first_round() {
        // Routes 1 & 2 reach the SAME residue by independent derivations
        // (bases 7 and 20, both ≡ 7 mod 13) — exact-canonical, so they
        // hash-agree; route 3 diverges. (Exact ops agree bit-for-bit;
        // FD-numeric ops like curvature need tolerance agreement, a separate
        // path — so the exact-op case is what the hash-equality test covers.)
        let initial = vec![
            route(1, BranchSource::FreshDerivation,
                  KernelOp::ModPow { base: 7, exp: 100, modulus: 13 }),
            route(2, BranchSource::NeighborAtom,
                  KernelOp::ModPow { base: 20, exp: 100, modulus: 13 }),
            route(3, BranchSource::CrossBasinBridge,
                  KernelOp::ModPow { base: 5, exp: 100, modulus: 13 }),
        ];
        let out = run_meta_loop(initial, |_c, _r| vec![], &MetaConfig::default());
        match out {
            MetaOutcome::Verified { agreeing, cycles, .. } => {
                assert_eq!(agreeing, 2);
                assert_eq!(cycles, 1);
            }
            other => panic!("expected Verified, got {:?}", other),
        }
    }

    #[test]
    fn iterates_then_confirms() {
        // Round 1: a single route (no agreement possible) → iterate.
        // Round 2: a second independent route reaches the same value → confirm.
        let initial = vec![route(1, BranchSource::FreshDerivation,
            KernelOp::ModPow { base: 7, exp: 100, modulus: 13 })];
        let out = run_meta_loop(
            initial,
            |cycle, _receipts| {
                if cycle == 1 {
                    // independent route to the same residue (7^100 ≡ 7^(100 mod 12) ... = 9)
                    vec![
                        route(1, BranchSource::FreshDerivation,
                            KernelOp::ModPow { base: 7, exp: 100, modulus: 13 }),
                        route(2, BranchSource::NeighborAtom,
                            KernelOp::ModPow { base: 20, exp: 100, modulus: 13 }), // 20≡7 mod13
                    ]
                } else {
                    vec![]
                }
            },
            &MetaConfig::default(),
        );
        match out {
            MetaOutcome::Verified { agreeing, cycles, hash, .. } => {
                assert_eq!(agreeing, 2);
                assert_eq!(cycles, 2);
                assert!(hash.len() == 64);
            }
            other => panic!("expected Verified after iteration, got {:?}", other),
        }
    }

    #[test]
    fn exhausts_without_agreement() {
        // One lone route, no further proposals → never confirmed.
        let initial = vec![route(1, BranchSource::FreshDerivation,
            KernelOp::Gcd { a: 48, b: 18 })];
        let out = run_meta_loop(initial, |_c, _r| vec![], &MetaConfig::default());
        assert!(matches!(out, MetaOutcome::Exhausted { .. }));
    }
}

// ---------- model-facing route DSL (no JSON; one ROUTE per line) ----------
//
// Grammar (the model emits these; it must NOT state the answer):
//   ROUTE modpow base=<u64> exp=<u64> mod=<u64>
//   ROUTE gcd a=<u64> b=<u64>
//   ROUTE lcm xs=<u64,u64,...>
//   ROUTE scalar_curvature metric=sphere2 r=<f64> point=<f64,f64>
//   ROUTE scalar_curvature metric=poincareh2 point=<f64,f64>
//   ROUTE scalar_curvature metric=flat dim=<usize> point=<f64,...>
//   ROUTE christoffel metric=<sphere2 r=..|poincareh2|flat dim=..> point=<..>
//
// Each ROUTE must reach the SAME target by an INDEPENDENT derivation — that
// independence is what makes cross-route hash agreement a real verification.

use crate::mathkernel::Metric;

fn kv<'a>(tokens: &'a [&'a str], key: &str) -> Option<&'a str> {
    tokens.iter().find_map(|t| t.strip_prefix(&format!("{key}=")))
}
fn f64s(s: &str) -> Vec<f64> { s.split(',').filter_map(|x| x.trim().parse().ok()).collect() }
fn u64s(s: &str) -> Vec<u64> { s.split(',').filter_map(|x| x.trim().parse().ok()).collect() }

fn parse_metric(tokens: &[&str]) -> Option<Metric> {
    match kv(tokens, "metric")? {
        "sphere2" => Some(Metric::Sphere2 { r: kv(tokens, "r")?.parse().ok()? }),
        "sphere3" => Some(Metric::Sphere3 { r: kv(tokens, "r")?.parse().ok()? }),
        "poincareh2" => Some(Metric::PoincareH2),
        "hyperbolich3" => Some(Metric::HyperbolicH3),
        "flat" => Some(Metric::FlatEuclidean { dim: kv(tokens, "dim").and_then(|d| d.parse().ok()).unwrap_or(2) }),
        _ => None,
    }
}

/// Parse a bracketed row-major matrix like [[1,2],[3,4]] (whitespace ignored).
fn parse_matrix(s: &str) -> Option<Vec<Vec<f64>>> {
    let t: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    let inner = t.strip_prefix("[[")?.strip_suffix("]]")?;
    let rows: Vec<Vec<f64>> = inner.split("],[")
        .map(|row| row.split(',').filter_map(|x| x.parse().ok()).collect())
        .collect();
    if rows.is_empty() || rows.iter().any(|r| r.is_empty()) { None } else { Some(rows) }
}

fn parse_op(line: &str) -> Option<KernelOp> {
    let t: Vec<&str> = line.split_whitespace().collect();
    let i = t.iter().position(|x| *x == "ROUTE")?;
    let op = *t.get(i + 1)?;
    let a = &t[i + 2..];
    match op {
        "modpow" => Some(KernelOp::ModPow {
            base: kv(a, "base")?.parse().ok()?,
            exp: kv(a, "exp")?.parse().ok()?,
            modulus: kv(a, "mod")?.parse().ok()?,
        }),
        "gcd" => Some(KernelOp::Gcd { a: kv(a, "a")?.parse().ok()?, b: kv(a, "b")?.parse().ok()? }),
        "lcm" => Some(KernelOp::Lcm { xs: u64s(kv(a, "xs")?) }),
        "scalar_curvature" => Some(KernelOp::ScalarCurvature { metric: parse_metric(a)?, point: f64s(kv(a, "point")?) }),
        "christoffel" => Some(KernelOp::Christoffel { metric: parse_metric(a)?, point: f64s(kv(a, "point")?) }),
        "determinant" => Some(KernelOp::Determinant { a: parse_matrix(kv(a, "a")?)? }),
        "eigenvalues" => Some(KernelOp::SymEigenvalues { a: parse_matrix(kv(a, "a")?)? }),
        "rank" => Some(KernelOp::Rank { a: parse_matrix(kv(a, "a")?)?, tol: kv(a, "tol").and_then(|t| t.parse().ok()).unwrap_or(1e-9) }),
        "is_prime" => Some(KernelOp::IsPrime { n: kv(a, "n")?.parse().ok()? }),
        "factorize" => Some(KernelOp::Factorize { n: kv(a, "n")?.parse().ok()? }),
        "modinv" => Some(KernelOp::ModInv { a: kv(a, "a")?.parse().ok()?, modulus: kv(a, "mod")?.parse().ok()? }),
        "totient" => Some(KernelOp::Totient { n: kv(a, "n")?.parse().ok()? }),
        _ => None,
    }
}

/// Parse all ROUTE lines from model output into routes, ids starting at
/// `start_id`, cycling branch sources so independent routes stay distinguishable.
pub fn parse_routes(model_output: &str, start_id: usize) -> Vec<Route> {
    let sources = [
        BranchSource::FreshDerivation,
        BranchSource::NeighborAtom,
        BranchSource::CrossBasinBridge,
        BranchSource::DirectRecall,
    ];
    model_output
        .lines()
        .filter(|l| l.contains("ROUTE "))
        .filter_map(parse_op)
        .enumerate()
        .map(|(k, op)| Route {
            id: start_id + k,
            source: sources[k % sources.len()].clone(),
            op,
            prior_complexity: 0.5,
        })
        .collect()
}

#[cfg(test)]
mod dsl_tests {
    use super::*;
    #[test]
    fn parses_mixed_routes() {
        let out = "thinking...\n\
            ROUTE modpow base=7 exp=100 mod=13\n\
            ROUTE scalar_curvature metric=sphere2 r=2 point=1.0,0.0\n\
            ROUTE christoffel metric=poincareh2 point=0.0,1.0\n\
            some trailing prose";
        let routes = parse_routes(out, 1);
        assert_eq!(routes.len(), 3);
        assert!(matches!(routes[0].op, KernelOp::ModPow { base: 7, exp: 100, modulus: 13 }));
        assert!(matches!(routes[1].op, KernelOp::ScalarCurvature { .. }));
        assert!(matches!(routes[2].op, KernelOp::Christoffel { .. }));
    }
    #[test]
    fn parses_expanded_ops() {
        let out = "ROUTE determinant a=[[1,2],[3,4]]\n\
                   ROUTE is_prime n=97\n\
                   ROUTE scalar_curvature metric=sphere3 r=2 point=1.0,1.0,0.5\n\
                   ROUTE totient n=36";
        let routes = parse_routes(out, 1);
        assert_eq!(routes.len(), 4);
        assert!(matches!(routes[0].op, KernelOp::Determinant { .. }));
        assert!(matches!(routes[1].op, KernelOp::IsPrime { n: 97 }));
        assert!(matches!(routes[2].op, KernelOp::ScalarCurvature { metric: crate::mathkernel::Metric::Sphere3 { .. }, .. }));
        assert!(matches!(routes[3].op, KernelOp::Totient { n: 36 }));
    }

    #[test]
    fn ignores_malformed() {
        assert_eq!(parse_routes("ROUTE bogus x=1\nnot a route", 0).len(), 0);
    }
}

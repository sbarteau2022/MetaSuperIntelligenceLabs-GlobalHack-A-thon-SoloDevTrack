//! meta_solve: the model proposes computational ROUTES; the meta loop executes
//! them on the deterministic kernel and verifies by cross-route hash agreement.
//! The model never states the answer — it only names independent derivations.
//!
//! Env: DSPARK_URL, DSPARK_MODEL, DSPARK_API_KEY (same as the harness bins).
//! Usage: meta_solve "<problem / quantity to verify>"

use rgah::meta_loop::{parse_routes, run_meta_loop, MetaConfig, MetaOutcome, Route};
use rgah::lanes::ModelLane;

const ROUTE_GRAMMAR: &str = "\
You propose INDEPENDENT computational routes to verify a quantity. You do NOT \
state the answer. Emit 2 to 4 lines, each EXACTLY one route, nothing after:\n\
  ROUTE modpow base=<int> exp=<int> mod=<int>\n\
  ROUTE gcd a=<int> b=<int>\n\
  ROUTE lcm xs=<int,int,...>\n\
  ROUTE scalar_curvature metric=sphere2 r=<num> point=<num,num>\n\
  ROUTE scalar_curvature metric=poincareh2 point=<num,num>\n\
  ROUTE scalar_curvature metric=flat dim=<int> point=<num,...>\n\
  ROUTE christoffel metric=sphere2 r=<num> point=<num,num>\n\
  ROUTE scalar_curvature metric=sphere3 r=<num> point=<num,num,num>\n\
  ROUTE scalar_curvature metric=hyperbolich3 point=<num,num,num>\n\
  ROUTE determinant a=[[..],[..]]   |   ROUTE eigenvalues a=[[..],[..]]\n\
  ROUTE rank a=[[..],[..]] tol=<num>\n\
  ROUTE is_prime n=<int>  |  ROUTE factorize n=<int>  |  ROUTE modinv a=<int> mod=<int>  |  ROUTE totient n=<int>\n\
Each route must reach the SAME target by a DIFFERENT derivation (e.g. a \
congruent base mod m, a different chart/parametrization). At least two routes \
are required so they can be cross-checked. Output only ROUTE lines. Example:\nROUTE modpow base=7 exp=100 mod=13\nROUTE modpow base=20 exp=100 mod=13";

fn ask(system: &str, user: &str) -> Vec<Route> {
    // Route proposal goes through the model bound into the harness as a lane
    // (thinking off: proposing routes is formatting, the kernel does the math).
    let lane = ModelLane::new(system, false);
    match lane.complete(user) {
        Ok((content, _)) => {
            let routes = parse_routes(&content, 1);
            eprintln!("  model proposed {} parseable route(s)", routes.len());
            routes
        }
        Err(e) => { eprintln!("model call failed: {e}"); vec![] }
    }
}

fn main() {
    let problem = std::env::args().nth(1).unwrap_or_else(|| {
        "Verify the value of 7^100 mod 13.".into()
    });
    eprintln!("problem: {problem}");

    let initial = ask(ROUTE_GRAMMAR, &problem);

    // Unbounded model-driven iteration: re-query every non-agreeing round,
    // bounded only by MetaConfig.cycle_budget. Each round asks for NEW
    // independent routes, seeded with the distinct hashes seen so far so the
    // model is pushed toward a different derivation rather than repeating.
    let out = run_meta_loop(
        initial,
        |cycle, receipts| {
            let mut seen: Vec<String> = receipts.iter().filter_map(|(_, h)| h.clone()).collect();
            seen.sort(); seen.dedup();
            let fb = format!(
                "Round {cycle}: no two routes agreed yet ({} distinct results so far). \
                 Propose at least two NEW independent routes to the SAME target — different \
                 congruent bases or a different chart/parametrization — so two coincide. \
                 Avoid repeating previous routes. Only ROUTE lines.",
                seen.len()
            );
            ask(ROUTE_GRAMMAR, &format!("{problem}\n\n{fb}"))
        },
        &MetaConfig::default(),
    );

    match out {
        MetaOutcome::Verified { hash, agreeing, bound_total, cycles } => {
            println!("VERIFIED  agreeing_routes={agreeing}  cycles={cycles}  \
                      bound={bound_total:.4}\n  output_hash={hash}");
        }
        MetaOutcome::Exhausted { reason, bound_total, cycles } => {
            println!("NO CONFIDENT RESULT  cycles={cycles}  bound={bound_total:.4}\n  reason={reason}");
        }
    }
}

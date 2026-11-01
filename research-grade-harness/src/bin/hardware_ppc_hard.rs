// Hardware PPC hardening probe (ADDENDUM-2026-10-31): does a HARDER task
// battery break the 8/8 saturation seen in hardware_ppc (v2) and the adopt
// RUN-01/02/03 runs (gain 0.0000 every time)?
//
// Focused change vs hardware_ppc.rs (which is FROZEN by its receipts and
// untouched): only the task battery is harder (multi-step with nearer
// distractor traps) and the candidates carry genuinely different prompt
// mechanisms (unit-carry, equation-first) instead of single-sentence
// wording tweaks. Judge (last_number_eq), std-only HTTP, prompt rendering,
// and the regularized_adopt + regulator stack are identical logic via the
// shared ppc_probe instrument.
//
// Env: OMLX_URL (default http://127.0.0.1:8000/v1/chat/completions),
//      OMLX_MODEL (default Qwen3.6-MoE-Combined), OMLX_API_KEY (default
//      dummy), PPC_RECEIPT (default /var/tmp/ppc-hardening-receipt.txt).
use rgah::energy::EnergyBudget;
use rgah::evolution::regularized_adopt;
use rgah::ppc_probe::{
    HELDOUT_TASKS, PpcTask, bundle_system_prompt, bundle_text, chat_once, last_number_eq, split_url,
};
use rgah::rsi::Bundle;

// HARD battery (8 tasks): every intermediate result is a plausible wrong
// LAST number, and the final answer requires carrying a conversion or a
// two-stage remainder. All targets python3-verified before the run:
//   hard-01: 240-60=180, 180/3=60, 180-60=120 (traps 60, 180)
//   hard-02: 500*0.85=425, +40=465 (traps 75, 425)
//   hard-03: 150+45-70=125 (trap 195)
//   hard-04: 203=16*12+11 (trap quotient 16)
//   hard-05: 80*0.75=60, 60*1.08=64.8 (trap 60)
//   hard-06: (90+150)/3=80 (traps 90, 75)
//   hard-07: 500-213+96-158=225 (traps 287, 383)
//   hard-08: 130-47=83, 83-29=54 (trap 83, stated in prompt)
const HARD_TASKS: &[PpcTask] = &[
    PpcTask {
        id: "hard-01",
        prompt: "A bakery bakes 240 loaves, sells 5 dozen in the morning and a third of the remainder in the afternoon. How many loaves remain?",
        target: "120",
        held_out: false,
    },
    PpcTask {
        id: "hard-02",
        prompt: "A water tank holds 500 liters, drains 15 percent, then refills 40 liters. How many liters are in the tank?",
        target: "465",
        held_out: false,
    },
    PpcTask {
        id: "hard-03",
        prompt: "A gym has 150 members, gains 45 in January, then loses 70 in February. How many members remain?",
        target: "125",
        held_out: false,
    },
    PpcTask {
        id: "hard-04",
        prompt: "A farmer packs 203 eggs into cartons holding 12 each. How many eggs are left unpacked?",
        target: "11",
        held_out: false,
    },
    PpcTask {
        id: "hard-05",
        prompt: "A shirt costs 80 dollars, is marked down 25 percent, then the sale price is taxed 8 percent. What is the final price in dollars?",
        target: "64.8",
        held_out: false,
    },
    PpcTask {
        id: "hard-06",
        prompt: "A train travels 90 km in the first hour and 150 km over the next 2 hours. What is its average speed in km/h across the whole trip?",
        target: "80",
        held_out: false,
    },
    PpcTask {
        id: "hard-07",
        prompt: "Lena has 500 dollars, spends 213 on rent, earns 96 babysitting, then spends 158 on groceries. How much money is left?",
        target: "225",
        held_out: false,
    },
    PpcTask {
        id: "hard-08",
        prompt: "A depot holds 130 crates. 47 ship out Monday, leaving 83. Then 29 more ship Tuesday. First confirm Monday's count of 83, then give Tuesday's remainder. How many remain after Tuesday?",
        target: "54",
        held_out: false,
    },
];

fn unit_bundle(base: &Bundle) -> Bundle {
    let mut c = base.components.clone();
    c[0] = format!(
        "{} + unit-carry discipline: carry the unit (loaves, liters, dollars) on every intermediate number and restate it in the final answer.",
        base.components[0]
    );
    Bundle {
        version: 1,
        components: c,
    }
}

fn equation_bundle(base: &Bundle) -> Bundle {
    let mut c = base.components.clone();
    c[0] = format!(
        "{} + equation-first discipline: write the single arithmetic equation for the whole problem first, then compute it step by step.",
        base.components[0]
    );
    Bundle {
        version: 1,
        components: c,
    }
}

fn junk_bundle(base: &Bundle) -> Bundle {
    let mut c = base.components.clone();
    c[0] = format!("{} + improve invariant", base.components[0]);
    Bundle {
        version: 1,
        components: c,
    }
}

fn probe(
    b: &Bundle,
    host: &str,
    port: u16,
    path: &str,
    model: &str,
    key: &str,
) -> (usize, usize, u64) {
    let sys = bundle_system_prompt(b);
    let mut passes = 0usize;
    let mut total = 0usize;
    let mut tokens = 0u64;
    for t in HARD_TASKS {
        total += 1;
        match chat_once(host, port, path, model, key, &sys, t.prompt) {
            Ok((text, tok)) => {
                tokens += tok;
                let (ok, why) = last_number_eq(&text, t.target);
                if ok {
                    passes += 1;
                }
                eprintln!(
                    "  probe task={} target={} pass={} why={}",
                    t.id, t.target, ok, why
                );
            }
            Err(e) => eprintln!("  probe task={} ERROR {}", t.id, e),
        }
    }
    (passes, total, tokens)
}

fn main() -> std::process::ExitCode {
    let url = std::env::var("OMLX_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8000/v1/chat/completions".into());
    let model = std::env::var("OMLX_MODEL").unwrap_or_else(|_| "Qwen3.6-MoE-Combined".into());
    let key = std::env::var("OMLX_API_KEY").unwrap_or_else(|_| "dummy".into());
    let (host, port, path) = match split_url(&url) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("bad OMLX_URL: {}", e);
            return std::process::ExitCode::FAILURE;
        }
    };
    if let Err(e) = chat_once(
        &host,
        port,
        &path,
        &model,
        &key,
        "You are a probe.",
        "Reply with: FINAL 1",
    ) {
        eprintln!("PREFLIGHT FAIL: model server unreachable: {}", e);
        return std::process::ExitCode::FAILURE;
    }
    let base = Bundle::base();
    let mut budget = EnergyBudget::new(50.0);
    let cands: Vec<(&str, Bundle)> = vec![
        ("UNIT", unit_bundle(&base)),
        ("EQUATION", equation_bundle(&base)),
        ("JUNK", junk_bundle(&base)),
    ];
    eprintln!("probing base (hard battery, 8 tasks) ...");
    let (base_passes, base_total, base_tokens) = probe(&base, &host, port, &path, &model, &key);
    let base_q = base_passes as f64 / base_total as f64;
    budget.charge(base_tokens as f64 * 0.0005, base_tokens * 4);
    let mut out = String::new();
    out.push_str("hardware_ppc_hardening_receipt\n");
    out.push_str("parent_runbook=research-grade-harness/HARDWARE-GATE-RUNBOOK.md\n");
    out.push_str(&format!("model={}\n", model));
    out.push_str(&format!("server={}\n", url));
    out.push_str("real_probe=true\n");
    out.push_str(&format!(
        "base passes={}/{} quality={:.4} tokens={}\n",
        base_passes, base_total, base_q, base_tokens
    ));
    let mut adopted_name = "BASE";
    let mut adopted = base.clone();
    let mut adopted_q = base_q;
    for (name, b) in &cands {
        eprintln!("probing {} ...", name);
        let (passes, total, tokens) = probe(b, &host, port, &path, &model, &key);
        let q = passes as f64 / total as f64;
        budget.charge(tokens as f64 * 0.0005, tokens * 4);
        let (adopt, reasons) = regularized_adopt(q, base_q, b, 1.0, 1.0, 0.02);
        let rs = reasons.join("; ");
        out.push_str(&format!("candidate name={} fp={} passes={}/{} quality={:.4} tokens={} adopt={} reasons={} instruction={}\n", name, b.fingerprint(), passes, total, q, tokens, adopt, rs, bundle_text(b)));
        out.push_str(&format!(
            "regulators: leakage={} noise_band={} security={} pareto={}\n",
            !rs.contains("leakage"),
            !rs.contains("noise band"),
            !rs.contains("security floor"),
            !rs.contains("pareto")
        ));
        if adopt && q > adopted_q {
            adopted_name = name;
            adopted = b.clone();
            adopted_q = q;
        }
    }
    out.push_str(&format!(
        "adopted={} kept_base={} quality_gain={:.4}\n",
        adopted_name,
        adopted_name == "BASE",
        adopted_q - base_q
    ));
    let sys = bundle_system_prompt(&adopted);
    let mut held_passes = 0usize;
    let mut held_tokens = 0u64;
    for t in HELDOUT_TASKS {
        match chat_once(&host, port, &path, &model, &key, &sys, t.prompt) {
            Ok((text, tok)) => {
                held_tokens += tok;
                let (ok, why) = last_number_eq(&text, t.target);
                if ok {
                    held_passes += 1;
                }
                out.push_str(&format!(
                    "heldout task={} target={} pass={} why={}\n",
                    t.id, t.target, ok, why
                ));
            }
            Err(e) => out.push_str(&format!("heldout task={} ERROR {}\n", t.id, e)),
        }
    }
    out.push_str(&format!(
        "heldout passes={}/{} tokens={}\n",
        held_passes,
        HELDOUT_TASKS.len(),
        held_tokens
    ));
    let path = std::env::var("PPC_RECEIPT")
        .unwrap_or_else(|_| "/var/tmp/ppc-hardening-receipt.txt".into());
    match std::fs::File::create(&path).and_then(|mut f| {
        use std::io::Write;
        f.write_all(out.as_bytes()).map(|_| ())
    }) {
        Ok(()) => {
            println!("{}", out);
            eprintln!("receipt written to {}", path);
            std::process::ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("receipt write FAIL {}: {}", path, e);
            std::process::ExitCode::FAILURE
        }
    }
}

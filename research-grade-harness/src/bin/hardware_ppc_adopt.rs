// PPC adopt-demo: instructional-difference candidates (PPC-ADOPT-RUN-01).
//
// Proves the fixed PPC gate fires BOTH ways live: ADOPT a genuine win,
// REJECT a genuine regression, ABSTAIN among ties. Uses the shared
// ppc_probe instrument (task set v2, judges, HTTP, prompt rendering —
// byte-identical logic to hardware_ppc.rs, which is frozen by its receipts
// and untouched). Adoption rule = the tested regularized_adopt +
// regulator stack, pairwise per candidate vs base (same code path as
// ProposeProbeCompose::run step 4).
//
// Candidates (PPC-ADOPT-RUN-01-Manifest.md §2):
// - WIN: base + show-work-then-verify discipline (instructional mechanism
//   targeting trap tasks). Predicted: passes >= base.
// - LOSE: base with the FINAL-line instruction weakened (removes the
//   machine-checkable format contract). Predicted: passes <= base.
// - JUNK: base component 0 + " + improve invariant" (control — reproduces
//   the abstention baseline: ties base).
//
// Env: OMLX_URL (default http://127.0.0.1:8002/v1/chat/completions),
//      OMLX_MODEL (default Qwen3.6-MoE-Combined), OMLX_API_KEY (default
//      dummy), PPC_RECEIPT (default research-grade-harness/HARDWARE-PPC-ADOPT-RECEIPT.txt).
use rgah::energy::EnergyBudget;
use rgah::evolution::regularized_adopt;
use rgah::ppc_probe::{
    ADOPT_RUN02_BOUNDARY, ADOPT_RUN02_DISCRIMINATOR, ADOPT_RUN02_GUARDS_EXTRA, HELDOUT_TASKS,
    PROBE_TASKS, PpcTask, bundle_system_prompt, bundle_text, chat_once, last_number_eq, split_url,
};
use rgah::rsi::Bundle;

fn win_bundle(base: &Bundle) -> Bundle {
    let mut c = base.components.clone();
    c[0] = format!(
        "{} + show-work discipline: Show each calculation step on its own line, then double-check the last step before answering.",
        base.components[0]
    );
    Bundle {
        version: 1,
        components: c,
    }
}

fn lose_bundle(base: &Bundle) -> Bundle {
    // Weakened FINAL line: the system prompt template still appends the
    // strict FINAL instruction AFTER components, so this tests whether a
    // contradictory looseness inside the components degrades discipline.
    // (bundle_system_prompt always ends with the strict FINAL line; the
    // LOSE candidate injects "a plain answer is fine" upstream of it.)
    let mut c = base.components.clone();
    c[0] = format!(
        "{}. A plain answer is fine; formatting does not matter.",
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

fn stable_bundle(base: &Bundle) -> Bundle {
    // PPC-ADOPT-RUN-02 WIN candidate (pressure-proven): recompute discipline.
    // h-06 BASE 0/15 vs STABLE 15/15 across 3 sessions (2026-10-30).
    let mut c = base.components.clone();
    c[0] = format!(
        "{} Before answering, recompute the total a second time from the original numbers and use the recomputed value.",
        base.components[0]
    );
    Bundle {
        version: 1,
        components: c,
    }
}

/// Probe any task slice (RUN-02 battery composes discriminator + guards +
/// boundary from shared consts; default battery = v2 PROBE_TASKS).
/// RUN-03: PPC_REPEATS (env, default 1) repeats each task; passes count per
/// repeat, quality is a pass RATE. Default 1 = RUN-01/RUN-02 behavior.
fn probe_tasks(
    b: &Bundle,
    host: &str,
    port: u16,
    path: &str,
    model: &str,
    key: &str,
    tasks: &[PpcTask],
) -> (usize, usize, u64, Vec<String>) {
    let sys = bundle_system_prompt(b);
    // RUN-03: in-run repeats (PPC_REPEATS env, default 1). Each task probed
    // N times; passes/tokens accumulate; quality is a rate. Default 1
    // preserves RUN-01/RUN-02 single-pass behavior exactly.
    let repeats: usize = std::env::var("PPC_REPEATS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    let mut passes = 0usize;
    let mut total = 0usize;
    let mut tokens = 0u64;
    let mut rows: Vec<String> = vec![];
    for t in tasks {
        for rep in 0..repeats.max(1) {
            total += 1;
            match chat_once(host, port, path, model, key, &sys, t.prompt) {
                Ok((text, tok)) => {
                    tokens += tok;
                    let (ok, why) = last_number_eq(&text, t.target);
                    if ok {
                        passes += 1;
                    }
                    rows.push(format!(
                        "probe task={} rep={} target={} pass={} why={}",
                        t.id, rep, t.target, ok, why
                    ));
                }
                Err(e) => rows.push(format!("probe task={} rep={} ERROR {}", t.id, rep, e)),
            }
        }
    }
    for r in &rows {
        eprintln!("  {}", r);
    }
    (passes, total, tokens, rows)
}

fn main() -> std::process::ExitCode {
    let url = std::env::var("OMLX_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8002/v1/chat/completions".into());
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
    // Battery select (PPC-ADOPT-RUN-02-Manifest §4): PPC_BATTERY=run02 runs
    // the discriminator + guards + boundary battery with the STABLE candidate;
    // default (unset/anything else) = RUN-01 battery byte-identical.
    let battery = std::env::var("PPC_BATTERY").unwrap_or_else(|_| "run01".into());
    let is_run02 = battery == "run02";
    // Composed RUN-02 task list: discriminator + v2 probes + extra guards +
    // boundary (diagnostic). Owned vec because it spans const slices.
    let run02_tasks: Vec<PpcTask> = ADOPT_RUN02_DISCRIMINATOR
        .iter()
        .chain(PROBE_TASKS.iter())
        .chain(ADOPT_RUN02_GUARDS_EXTRA.iter())
        .chain(ADOPT_RUN02_BOUNDARY.iter())
        .cloned()
        .collect();
    let probe_battery: &[PpcTask] = if is_run02 { &run02_tasks } else { PROBE_TASKS };
    let cands: Vec<(&str, Bundle)> = if is_run02 {
        vec![
            ("STABLE", stable_bundle(&base)),
            ("LOSE", lose_bundle(&base)),
            ("JUNK", junk_bundle(&base)),
        ]
    } else {
        vec![
            ("WIN", win_bundle(&base)),
            ("LOSE", lose_bundle(&base)),
            ("JUNK", junk_bundle(&base)),
        ]
    };
    let mut budget = EnergyBudget::new(50.0);
    // Base probe first (KEEP_BASE competes on real signal, same as run()).
    eprintln!("probing base (battery={}) ...", battery);
    let (base_passes, base_total, base_tokens, _) =
        probe_tasks(&base, &host, port, &path, &model, &key, probe_battery);
    let base_q = base_passes as f64 / base_total as f64;
    budget.charge(base_tokens as f64 * 0.0005, base_tokens * 4);

    let mut out = String::new();
    out.push_str("hardware_ppc_adopt_receipt\n");
    out.push_str("runbook=research-grade-harness/HARDWARE-GATE-RUNBOOK.md\n");
    // Manifest pointer: PPC_MANIFEST env overrides (RUN-03+); default derives
    // from the battery (run01->01, run02->02). A wrong pointer voids traceability
    // (C09.3) — RUN-03 attempt 1 was voided for exactly this (said RUN-02).
    let manifest_ptr = std::env::var("PPC_MANIFEST").unwrap_or_else(|_| {
        format!(
            "MasterDesignDocument/C09-Ablations-Proofs-Claims/PPC-ADOPT-RUN-{:02}-Manifest.md",
            if is_run02 { 2 } else { 1 }
        )
    });
    out.push_str(&format!("manifest={}\n", manifest_ptr));
    out.push_str(&format!(
        "battery={} tasks={}\n",
        battery,
        probe_battery.len()
    ));
    out.push_str(&format!("model={}\n", model));
    out.push_str(&format!("server={}\n", url));
    out.push_str("real_probe=true\n");
    out.push_str(&format!(
        "base passes={}/{} quality={:.4} tokens={}\n",
        base_passes, base_total, base_q, base_tokens
    ));

    // Pairwise adopt vs base through the tested regulator stack.
    let mut adopted_name = "BASE";
    let mut adopted = base.clone();
    let mut adopted_q = base_q;
    let mut adopted_tokens = base_tokens;
    for (name, b) in &cands {
        eprintln!("probing {} ...", name);
        let (passes, total, tokens, _) =
            probe_tasks(b, &host, port, &path, &model, &key, probe_battery);
        let q = passes as f64 / total as f64;
        budget.charge(tokens as f64 * 0.0005, tokens * 4);
        let (adopt, reasons) = regularized_adopt(q, base_q, b, 1.0, 1.0, 0.02);
        let rs = reasons.join("; ");
        out.push_str(&format!(
            "candidate name={} fp={} passes={}/{} quality={:.4} tokens={} adopt={} reasons={} instruction={}\n",
            name,
            b.fingerprint(),
            passes,
            total,
            q,
            tokens,
            adopt,
            rs,
            bundle_text(b),
        ));
        out.push_str("regulators:\n");
        out.push_str(&format!(
            "regulator leakage_critic passed={}\n",
            !rs.contains("leakage")
        ));
        out.push_str(&format!(
            "regulator noise_band_0.02 passed={}\n",
            !rs.contains("noise band")
        ));
        out.push_str(&format!(
            "regulator security_floor passed={}\n",
            !rs.contains("security floor")
        ));
        out.push_str(&format!(
            "regulator pareto_frontier passed={}\n",
            !rs.contains("pareto")
        ));
        // Adopt the best adopted candidate (strictly better than base and
        // better than any previously adopted candidate).
        if adopt && q > adopted_q {
            adopted_name = name;
            adopted = b.clone();
            adopted_q = q;
            adopted_tokens = tokens;
        }
    }
    let kept_base = adopted_name == "BASE";
    out.push_str(&format!(
        "adopted={} kept_base={} quality_gain={:.4}\n",
        adopted_name,
        kept_base,
        adopted_q - base_q
    ));

    // Held-out ONCE on the adopted bundle (or base if kept).
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
        "heldout passes={}/{} tokens={} adopted_tokens={}\n",
        held_passes,
        HELDOUT_TASKS.len(),
        held_tokens,
        adopted_tokens
    ));

    let path = std::env::var("PPC_RECEIPT")
        .unwrap_or_else(|_| "research-grade-harness/HARDWARE-PPC-ADOPT-RECEIPT.txt".into());
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

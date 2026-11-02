use clap::{Parser, Subcommand};
#[cfg(feature = "synthetic-scaffold")]
use rgah::bench;
use rgah::mission::{
    MissionInput, MissionReceipt, dry_run, geometry_mission, run_mission, run_mission_with_prior,
};
use rgah::receipt::parse_prior_receipt_text;
use rgah::{FrozenPolicyKernel, registry};
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "rgah", about = "Research-grade agentic harness")]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Pressure-test FrozenPolicyKernel mutation refusals.
    FpkCheck,
    /// Run built-in throughput benchmark (MadMind/Harness2/Cordis/Rgah).
    Bench {
        #[arg(long, default_value = "json")]
        format: String,
    },
    /// Print design bars.
    Bars,
    /// Validate the built-in geometry mission. No run, no side effect. Go / no-go check.
    DryRun,
    /// Run the built-in geometry mission to a plain-text receipt. Exit 1 if any packet escalates or fails.
    ///
    /// Default (no `--live`): fail-closed local orchestration — no executor,
    /// so every packet escalates and the exit is 1. This proves the gate,
    /// it does not execute work.
    ///
    /// With `--live`: packets execute through the standalone map/KV plugin
    /// binary (`HYPERBOLIC_MAP_PLUGIN_BIN`, root under `--live-root` or a
    /// fresh temp dir). Receipts carry the chained evidence hash plus the
    /// exact N3 `receipt_sha256` digests bound at execution.
    Run {
        /// Receipt output file. Defaults to stdout when omitted.
        #[arg(long)]
        receipt: Option<PathBuf>,
        /// Energy budget in joules. Default 1000.
        #[arg(long, default_value = "1000")]
        budget: f64,
        /// Execute live through the map/KV plugin instead of fail-closed.
        #[arg(long, default_value = "false")]
        live: bool,
        /// Plugin working root (defaults to a fresh temp dir per run).
        #[arg(long)]
        live_root: Option<PathBuf>,
        /// Capability manifest for the live gate (defaults to a manifest
        /// admitting the geometry mission's map tools).
        #[arg(long)]
        manifest: Option<PathBuf>,
    },
    /// Resume the built-in mission from a prior receipt file.
    ///
    /// The prior file MUST verify as a chain (`prev=`/`chain=` links from
    /// GENESIS, recomputed hashes match). Forged plaintext PASS lines,
    /// truncated files, and edited outcomes fail verification: the whole
    /// prior file is rejected (accepted=0) and the resume exits non-zero
    /// without marking any packet passed.
    ///
    /// With `--live`, remaining packets execute through the plugin binary;
    /// without it, remaining packets escalate (fail-closed).
    Resume {
        /// Prior receipt file (output of `rgah run`). Only chain-verified
        /// PASS lines are trusted; anything else is ignored and counted.
        #[arg(long)]
        prior: PathBuf,
        /// Receipt output file. Defaults to stdout when omitted.
        #[arg(long)]
        receipt: Option<PathBuf>,
        /// Energy budget in joules. Default 1000.
        #[arg(long, default_value = "1000")]
        budget: f64,
        /// Execute remaining packets live through the map/KV plugin.
        #[arg(long, default_value = "false")]
        live: bool,
        /// Plugin working root (defaults to a fresh temp dir per run).
        #[arg(long)]
        live_root: Option<PathBuf>,
        /// Capability manifest for the live gate.
        #[arg(long)]
        manifest: Option<PathBuf>,
    },
    /// Print the native-name registry: MCP, tools, skills, hooks, runbooks, workflows, policy.
    Registry,
}

/// Full chained emit format (P0-1/P0-2 closure): every line carries the
/// `prev=`/`chain=` links, the issue timestamp, and the bound N3 digests,
/// so a later `resume` can verify the file instead of trusting it.
/// `n3=-` marks executor-less (fail-closed) receipts with no bound digests.
fn receipt_text(out: &MissionReceipt) -> String {
    let mut text = format!("mission={}\n", out.mission_id);
    text.push_str(&format!("passed={}\n", out.passed.join(",")));
    text.push_str(&format!("failed={}\n", out.failed.join(",")));
    text.push_str(&format!("skipped={}\n", out.skipped.join(",")));
    text.push_str(&format!("escalated={}\n", out.escalated.join(",")));
    text.push_str(&format!("energy_spent_j={:.4}\n", out.energy_spent_j));
    text.push_str(&format!(
        "energy_remaining_j={:.4}\n",
        out.energy_remaining_j
    ));
    for r in &out.receipts.entries {
        let n3 = if r.n3_receipts.is_empty() {
            "-".to_string()
        } else {
            r.n3_receipts.join(",")
        };
        text.push_str(&format!(
            "receipt packet={} attempt={} outcome={} judges={} energy_j={:?} erased_bits={} ts={} prev={} chain={} n3={}\n",
            r.packet_id,
            r.attempt,
            r.outcome,
            r.judge_ids.join(","),
            r.energy_joules,
            r.erased_bits,
            r.timestamp_ms,
            r.prev_hash,
            r.evidence_hash,
            n3
        ));
    }
    text
}

fn emit(receipt: Option<PathBuf>, out: &MissionReceipt) -> Result<(), String> {
    let text = receipt_text(out);
    match receipt {
        Some(path) => {
            fs::write(&path, text).map_err(|e| format!("write {}: {}", path.display(), e))
        }
        None => {
            print!("{}", text);
            Ok(())
        }
    }
}

fn report(out: &MissionReceipt) -> ExitCode {
    println!(
        "mission={} passed={} escalated={} failed={} energy_spent={:.4}",
        out.mission_id,
        out.passed.len(),
        out.escalated.len(),
        out.failed.len(),
        out.energy_spent_j
    );
    if out.passed_all() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.cmd {
        Commands::FpkCheck => {
            let k = FrozenPolicyKernel::default();
            for t in ["judge", "safety", "egress", "receipt", "other"] {
                match k.check_mutation(t) {
                    Ok(()) => println!("ALLOW {}", t),
                    Err(e) => println!("REFUSE {} -> {}", t, e),
                }
            }
            ExitCode::SUCCESS
        }
        Commands::Bench { format: _ } => {
            #[cfg(feature = "synthetic-scaffold")]
            {
                eprintln!("SCAFFOLD ONLY: synthetic benchmark output is not acceptance evidence");
                let res = bench::run_all();
                print!("{}", res.summary());
                ExitCode::SUCCESS
            }
            #[cfg(not(feature = "synthetic-scaffold"))]
            {
                eprintln!(
                    "NO-GO: synthetic benchmark disabled; use a pre-registered live-model run"
                );
                ExitCode::FAILURE
            }
        }
        Commands::Bars => {
            println!("Quality: Δpass >= +3pp (k=1), +8pp (k=3)");
            println!("Waste: tokens/passed-judge reduction >=10%");
            println!("Landauer: cost/passed-judge reduction >=15%");
            println!("Coverage: >=70% exact declared context");
            println!("Safety: zero kernel violations");
            ExitCode::SUCCESS
        }
        Commands::DryRun => {
            let input: MissionInput = geometry_mission();
            match dry_run(&input.plan) {
                Ok(()) => {
                    println!(
                        "GO mission={} packets={}",
                        input.plan.id,
                        input.plan.packets.len()
                    );
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("NO-GO {}: {}", input.plan.id, e);
                    ExitCode::FAILURE
                }
            }
        }
        Commands::Run {
            receipt, budget, ..
        } => {
            let input: MissionInput = geometry_mission();
            if let Err(e) = dry_run(&input.plan) {
                eprintln!("NO-GO {}: {}", input.plan.id, e);
                return ExitCode::FAILURE;
            }
            let out = run_mission(input, budget);
            if let Err(e) = emit(receipt, &out) {
                eprintln!("{}", e);
                return ExitCode::FAILURE;
            }
            report(&out)
        }
        Commands::Resume {
            prior,
            receipt,
            budget,
            ..
        } => match fs::read_to_string(&prior) {
            Ok(text) => {
                let plan_ids: Vec<String> = geometry_mission()
                    .plan
                    .packets
                    .iter()
                    .map(|p| p.id.clone())
                    .collect();
                let (prior_log, stats) = parse_prior_receipt_text(&text, &plan_ids);
                eprintln!(
                    "resume prior={} accepted={} ignored={} chain_broken={} chain_ok={}",
                    prior.display(),
                    stats.accepted,
                    stats.ignored,
                    stats.chain_broken,
                    stats.chain_ok,
                );
                // P0-1 closure: a prior file that fails chain verification
                // is rejected outright — exit non-zero, mark nothing passed.
                if !stats.chain_ok {
                    eprintln!("NO-GO prior chain unverified (forged/truncated/edited): accepted=0");
                    return ExitCode::FAILURE;
                }
                let input: MissionInput = geometry_mission();
                if let Err(e) = dry_run(&input.plan) {
                    eprintln!("NO-GO {}: {}", input.plan.id, e);
                    return ExitCode::FAILURE;
                }
                let out = run_mission_with_prior(input, budget, Some(prior_log));
                if let Err(e) = emit(receipt, &out) {
                    eprintln!("{}", e);
                    return ExitCode::FAILURE;
                }
                report(&out)
            }
            Err(e) => {
                eprintln!("NO-GO read {}: {}", prior.display(), e);
                ExitCode::FAILURE
            }
        },
        Commands::Registry => {
            for entry in registry::registry() {
                println!(
                    "{} | {} | {:?} | {} | live={}",
                    entry.native, entry.current, entry.kind, entry.role, entry.live
                );
            }
            ExitCode::SUCCESS
        }
    }
}

use clap::{Parser, Subcommand};
use rgah::mission::{MissionInput, MissionReceipt, dry_run, geometry_mission, run_mission, run_mission_with_prior};
use rgah::receipt::{Receipt, ReceiptLog};
use rgah::{FrozenPolicyKernel, bench, registry};
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
    Run {
        /// Receipt output file. Defaults to stdout when omitted.
        #[arg(long)]
        receipt: Option<PathBuf>,
        /// Energy budget in joules. Default 1000.
        #[arg(long, default_value = "1000")]
        budget: f64,
    },
    /// Resume the built-in mission from a prior receipt file.
    /// Prior file: one passed packet id per line. Passed packets are not re-run.
    Resume {
        /// Prior receipt file, one passed packet id per line.
        #[arg(long)]
        prior: PathBuf,
        /// Receipt output file. Defaults to stdout when omitted.
        #[arg(long)]
        receipt: Option<PathBuf>,
        /// Energy budget in joules. Default 1000.
        #[arg(long, default_value = "1000")]
        budget: f64,
    },
    /// Print the native-name registry: MCP, tools, skills, hooks, runbooks, workflows, policy.
    Registry,
}

fn receipt_text(out: &MissionReceipt) -> String {
    let mut text = format!("mission={}\n", out.mission_id);
    text.push_str(&format!("passed={}\n", out.passed.join(",")));
    text.push_str(&format!("failed={}\n", out.failed.join(",")));
    text.push_str(&format!("skipped={}\n", out.skipped.join(",")));
    text.push_str(&format!("escalated={}\n", out.escalated.join(",")));
    text.push_str(&format!("energy_spent_j={:.4}\n", out.energy_spent_j));
    text.push_str(&format!("energy_remaining_j={:.4}\n", out.energy_remaining_j));
    for r in &out.receipts.entries {
        text.push_str(&format!(
            "receipt packet={} attempt={} outcome={} judges={} energy_j={:.4} erased_bits={}\n",
            r.packet_id,
            r.attempt,
            r.outcome,
            r.judge_ids.join(","),
            r.energy_joules,
            r.erased_bits
        ));
    }
    text
}

fn emit(receipt: Option<PathBuf>, out: &MissionReceipt) -> Result<(), String> {
    let text = receipt_text(out);
    match receipt {
        Some(path) => fs::write(&path, text).map_err(|e| format!("write {}: {}", path.display(), e)),
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
            let res = bench::run_all();
            print!("{}", res.summary());
            ExitCode::SUCCESS
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
                    println!("GO mission={} packets={}", input.plan.id, input.plan.packets.len());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("NO-GO {}: {}", input.plan.id, e);
                    ExitCode::FAILURE
                }
            }
        }
        Commands::Run { receipt, budget } => {
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
        Commands::Resume { prior, receipt, budget } => {
            match fs::read_to_string(&prior) {
                Ok(text) => {
                    let mut prior_log = ReceiptLog::default();
                    let plan_ids: Vec<String> = geometry_mission()
                        .plan
                        .packets
                        .iter()
                        .map(|p| p.id.clone())
                        .collect();
                    for line in text.lines() {
                        let id = line.trim();
                        if id.is_empty() {
                            continue;
                        }
                        // Prior files written by this CLI list receipts as
                        // "receipt packet=<id> ... outcome=PASS ...".
                        // Accept both that form and a bare id per line, but
                        // only if the id is a packet in this mission. Summary
                        // lines like passed=... are not packet ids and are skipped.
                        let packet_id = if let Some(rest) = id.strip_prefix("receipt packet=") {
                            rest.split_whitespace().next().unwrap_or(rest)
                        } else {
                            id
                        };
                        if !plan_ids.iter().any(|p| p == packet_id) {
                            continue;
                        }
                        prior_log.append(Receipt {
                            packet_id: packet_id.to_string(),
                            attempt: 1,
                            outcome: "PASS".into(),
                            judge_ids: vec![],
                            evidence_hash: String::new(),
                            energy_joules: 0.0,
                            erased_bits: 0,
                            timestamp_ms: 0,
                        });
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
            }
        }
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

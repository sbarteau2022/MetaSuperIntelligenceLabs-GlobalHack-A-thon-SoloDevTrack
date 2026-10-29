use clap::{Parser, Subcommand};
use rgah::{bench, FrozenPolicyKernel};
use std::path::PathBuf;

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
}

fn main() {
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
        }
        Commands::Bench { format: _ } => {
            let res = bench::run_all();
            print!("{}", res.summary());
        }
        Commands::Bars => {
            println!("Quality: Δpass >= +3pp (k=1), +8pp (k=3)");
            println!("Waste: tokens/passed-judge reduction >=10%");
            println!("Landauer: cost/passed-judge reduction >=15%");
            println!("Coverage: >=70% exact declared context");
            println!("Safety: zero kernel violations");
        }
    }
    let _ = PathBuf::new();
}

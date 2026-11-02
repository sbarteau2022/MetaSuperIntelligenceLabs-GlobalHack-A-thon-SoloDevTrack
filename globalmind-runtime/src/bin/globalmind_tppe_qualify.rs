//! Matched real-forward positional-encoder ablation (v3).
//!
//! Every arm runs at the same Layer-3 Q/K boundary on the same frozen
//! weights and token tape. Quality is real next-token NLL over the
//! 248,320-token vocabulary with paired per-token uncertainty against
//! the native-RoPE control. No zero-cost or zero-error control is
//! invented, and no arm is a rotary hybrid: TPPE arms rotate by the
//! sequential state's T2 phase with radial scale only.
//!
//! Preregistered component comparison rule (not encoder certification):
//! - tapes with fewer than 100 predictions are smoke-only: decision OPEN;
//! - an arm is provisionally BENCHED when its paired ΔNLL 95% CI lies entirely above
//!   +0.01 nats/token versus native RoPE;
//! - an arm is provisionally PROMOTED only when its paired ΔNLL 95% CI lies entirely
//!   at or below 0 and it is not slower than RoPE beyond CI overlap
//!   (non-inferior quality plus no measured resource regression);
//! - the radial-law winner is the TPPE arm whose paired ΔNLL against
//!   the other TPPE arm excludes 0 on the winning side; otherwise the
//!   radial decision is a TIE and no law is selected.
//!
//! Even with 100 predictions, this Layer-3-only runner cannot verify corpus
//! provenance or full-model quality. It never emits a certification decision.

use globalmind_runtime::Manifest;
use globalmind_runtime::RadialLaw;
use globalmind_runtime::trace::{
    PositionEncoderArm, decode_token_tape, default_tppe_config, paired_nll_stats,
    qualify_position_encoders,
};

const MIN_DECISION_PREDICTIONS: u32 = 100;
const BENCH_MARGIN_NATS: f64 = 0.01;

fn main() {
    if let Err(error) = run() {
        eprintln!("globalmind-tppe-qualify: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !(2..=3).contains(&args.len()) {
        return Err("usage: globalmind-tppe-qualify <manifest> <token-tape.gmtt> [repeats]".into());
    }
    let repeats = args
        .get(2)
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|_| "repeats must be an integer".to_string())
        })
        .transpose()?
        .unwrap_or(3);
    if repeats < 3 {
        return Err("qualification requires at least 3 counterbalanced repeats".into());
    }
    let verified = Manifest::load(&args[0])?.verify_artifacts()?;
    let tape_bytes =
        std::fs::read(&args[1]).map_err(|error| format!("read token tape {}: {error}", args[1]))?;
    let (tokens, token_tape_digest) = decode_token_tape(&tape_bytes)?;
    let rows = qualify_position_encoders(&verified, &tokens, repeats)?;

    println!("qualification=matched_real_qk_forward_v4_one_turn_budget");
    println!(
        "radial_budget_w_ref=1 linear_log_start={:.17} linear_log_span={:.17} log_start={:.17} log_span={:.17}",
        default_tppe_config(RadialLaw::LinearLogV1).radial_start,
        default_tppe_config(RadialLaw::LinearLogV1).radial_span,
        default_tppe_config(RadialLaw::LogV1).radial_start,
        default_tppe_config(RadialLaw::LogV1).radial_span,
    );
    println!("input_digest={}", verified.input_digest);
    println!("token_tape_digest={token_tape_digest}");
    println!("tokens={} repeats={repeats}", tokens.len());
    for row in &rows {
        println!(
            "arm={} mean_next_token_nll={:.9} top1_hits={}/{} mean_wall_us={:.3} wall_stddev_us={:.3} encoder_state_bytes={} mean_abs_logit_delta_from_rope={:.9} max_abs_logit_delta_from_rope={:.9} logits_digest={} paired_nll_delta_vs_rope={:.9} paired_nll_delta_ci95={:.9}",
            row.arm.as_str(),
            row.mean_next_token_nll,
            row.top1_hits,
            row.predictions,
            row.mean_wall_us,
            row.wall_stddev_us,
            row.encoder_state_bytes,
            row.mean_abs_logit_delta_from_rope,
            row.max_abs_logit_delta_from_rope,
            row.logits_digest,
            row.paired_nll_delta_mean,
            row.paired_nll_delta_ci95,
        );
    }

    let predictions = rows.first().map(|row| row.predictions).unwrap_or(0);
    if predictions < MIN_DECISION_PREDICTIONS {
        // Smoke tapes cannot support a quality promotion or bench.
        println!("decision=OPEN_REQUIRES_HELD_OUT_TASK_TAPE");
        println!("promoted=none");
        println!("benched=none");
        println!("radial_winner=none");
        return Ok(());
    }

    let rope = rows
        .iter()
        .find(|row| row.arm == PositionEncoderArm::NativeRope)
        .ok_or_else(|| "native RoPE reference row is missing".to_string())?;
    let mut promoted: Vec<&str> = Vec::new();
    let mut benched: Vec<&str> = Vec::new();
    for row in &rows {
        if row.arm == PositionEncoderArm::NativeRope {
            continue;
        }
        let ci_low = row.paired_nll_delta_mean - row.paired_nll_delta_ci95;
        let ci_high = row.paired_nll_delta_mean + row.paired_nll_delta_ci95;
        if ci_low > BENCH_MARGIN_NATS {
            benched.push(row.arm.as_str());
        } else if ci_high <= 0.0 && row.mean_wall_us <= rope.mean_wall_us + rope.wall_stddev_us {
            promoted.push(row.arm.as_str());
        }
    }

    let linear = rows
        .iter()
        .find(|row| row.arm == PositionEncoderArm::TppeLinearLogV1)
        .ok_or_else(|| "linear_log_v1 row is missing".to_string())?;
    let log = rows
        .iter()
        .find(|row| row.arm == PositionEncoderArm::TppeLogV1)
        .ok_or_else(|| "log_v1 row is missing".to_string())?;
    let (radial_delta, radial_ci95) = paired_nll_stats(&linear.per_token_nll, &log.per_token_nll)?;
    println!(
        "radial_paired_delta_linear_minus_log={:.9} radial_paired_ci95={:.9}",
        radial_delta, radial_ci95
    );
    let radial_winner = if radial_delta + radial_ci95 < 0.0 {
        "linear_log_v1"
    } else if radial_delta - radial_ci95 > 0.0 {
        "log_v1"
    } else {
        "none_tie"
    };

    println!("decision=OPEN_PARTIAL_MODEL_COMPONENT_COMPARISON");
    println!(
        "component_promoted={}",
        if promoted.is_empty() {
            "none".to_string()
        } else {
            promoted.join(",")
        }
    );
    println!(
        "component_benched={}",
        if benched.is_empty() {
            "none".to_string()
        } else {
            benched.join(",")
        }
    );
    println!("component_radial_winner={radial_winner}");
    println!("promoted=none");
    println!("benched=none");
    println!("radial_winner=none");
    Ok(())
}

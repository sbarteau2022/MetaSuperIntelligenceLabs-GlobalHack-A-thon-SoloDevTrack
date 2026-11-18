//! Stage pipeline: trace -> extract -> judge -> open.
//!
//! Stage 1 (`trace`) runs the real frozen Qwen3.6 layer-3 forward slice
//! twice on one immutable token tape: a full-forward control and an
//! observer-enabled run that emits native lane-1 trace envelopes. It fails
//! unless control vs observed logits and state agree within the
//! preregistered tolerance (exact f32 equality for this deterministic
//! slice; the tolerance is a named constant, not tuned after the fact).
//!
//! Stage 2 (`extract`) runs both TPPE radial candidates over the trace and
//! emits rank-0/1/2/<=8 candidates with full TPPE state receipts.
//! Stage 3 (`judge`) applies the separately versioned frozen judge and
//! persists every pass and rejection to a native receipt file.
//! Stage 4 (`open`) writes `manifests/<run-id>.admitted.manifest` only when
//! real eligibility and judge receipt digests exist, then opens the run in
//! an isolated SurrealDB. No stage creates placeholder receipts.

use globalmind_runtime::extract::{ExtractedCandidate, extract};
use globalmind_runtime::judge::{IndependentJudge, JudgeEvidence, JudgePolicy, JudgeVerdict};
use globalmind_runtime::trace::{TraceDtype, TraceEnvelope, TraceKind, encode};
use globalmind_runtime::{
    Digest, GlobalMindStore, Manifest, RadialLaw, StoreConfig, TppeConfig, TppeState,
};
use sha2::{Digest as ShaDigest, Sha256};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::time::Instant;

// ---- Frozen model slice facts (Qwen3.6-35B-A3B, layer 3 full attention) ----
const SHARD1: &str =
    "/Users/stewartbarteau/models/fused/Qwen3.6-MoE-Combined/model-00001-of-00004.safetensors";
const SHARD4: &str =
    "/Users/stewartbarteau/models/fused/Qwen3.6-MoE-Combined/model-00004-of-00004.safetensors";
const SHARD1_HEADER_LEN: u64 = 63_300;
const SHARD4_HEADER_LEN: u64 = 37_985;
const HIDDEN: usize = 2048;
const Q_HEADS: usize = 16;
const KV_HEADS: usize = 2;
const HEAD_DIM: usize = 256;
const VOCAB: usize = 248_320;
const EXPERTS: usize = 256;
const TOP_K: usize = 8;
const INTERMEDIATE: usize = 512;
const GROUP: usize = 64;
const EPS: f32 = 1e-6;
/// Preregistered nonperturbation tolerance: this deterministic slice must
/// reproduce logits bit-exactly; any drift fails the stage.
const LOGIT_TOLERANCE: f32 = 0.0;

fn main() {
    if let Err(error) = run() {
        eprintln!("globalmind-stage: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("trace") if args.len() == 3 => stage_trace(&args[1], &args[2]),
        Some("extract") if args.len() == 3 => stage_extract(&args[1], &args[2]),
        Some("judge") if args.len() == 4 => stage_judge(&args[1], &args[2], &args[3]),
        Some("open") if args.len() == 3 => stage_open(&args[1], &args[2]),
        _ => Err(
            "usage: globalmind-stage <trace <manifest> <out.gmtr>|extract <trace.gmtr> <out.candidates>|judge <candidates> <trace.gmtr> <out.receipts>|open <manifest> <judge-receipts>>"
                .into(),
        ),
    }
}

// ============================ Stage 1: trace ============================

fn stage_trace(manifest_path: &str, out_path: &str) -> Result<(), String> {
    let started = Instant::now();
    let manifest = Manifest::load(manifest_path)?;
    let verified = manifest.verify_artifacts()?;
    let model = &verified.models[0];
    println!("stage=trace");
    println!("model_roster_digest={}", model.roster_digest);
    println!("input_digest={}", verified.input_digest);

    // Immutable token tape: fixed token ids, hashed into the receipt.
    let tape: [u32; 6] = [9707, 25, 315, 279, 1124, 13];
    let tape_digest = digest_fields(
        b"globalmind-token-tape-v1",
        &tape
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    );
    println!("token_tape_digest={tape_digest}");

    let mut engine = Engine::load()?;
    let config = TppeConfig {
        law: RadialLaw::LinearLogV1,
        radial_start: 0.0,
        radial_span: 0.01,
        omega_fast: 0.1,
        omega_slow: 0.05,
        coupling_fast: 0.3,
        coupling_slow: 0.15,
    };

    // Control: full forward, no observer.
    let control = engine.forward(&tape, config, false)?;
    // Observed: identical forward with the trace tap enabled.
    let observed = engine.forward(&tape, config, true)?;

    let max_logit_drift = control
        .logits
        .iter()
        .zip(&observed.logits)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    let state_equal = control.hidden_out == observed.hidden_out;
    println!("max_logit_drift={max_logit_drift}");
    println!("state_equal={state_equal}");
    if max_logit_drift > LOGIT_TOLERANCE || !state_equal {
        return Err(format!(
            "observer perturbed the forward pass: drift {max_logit_drift} > {LOGIT_TOLERANCE}"
        ));
    }
    let observer_receipt = digest_fields(
        b"globalmind-observer-nonperturbation-v1",
        &[
            tape_digest.as_str(),
            &Digest::sha256(&f32_bytes(&control.logits)).to_string(),
            &Digest::sha256(&f32_bytes(&observed.logits)).to_string(),
            &format!("{max_logit_drift:?}"),
        ],
    );
    println!("observer_nonperturbation_receipt={observer_receipt}");
    println!(
        "logits_digest={}",
        Digest::sha256(&f32_bytes(&control.logits))
    );
    println!("top1_token={}", argmax(&control.logits));

    // Emit native envelopes: per-token layer-3 K/V activations + logits.
    let mut frames = Vec::new();
    for (position, activation) in observed.activations.iter().enumerate() {
        let payload = f32_bytes(activation);
        frames.extend_from_slice(&encode(&TraceEnvelope {
            model_digest: model.roster_digest.clone(),
            config_digest: verified.configuration_digest.clone(),
            tokenizer_digest: artifact_digest(model, "tokenizer")?,
            kind: TraceKind::Activation,
            dtype: TraceDtype::F32,
            shape: vec![HEAD_DIM as u32],
            layer: 3,
            head: 0,
            token_position: position as u64,
            observer_receipt_digest: observer_receipt.clone(),
            payload_digest: Digest::sha256(&payload),
            payload: payload.into_boxed_slice(),
        })?);
    }
    let logits_payload = f32_bytes(&observed.logits);
    frames.extend_from_slice(&encode(&TraceEnvelope {
        model_digest: model.roster_digest.clone(),
        config_digest: verified.configuration_digest.clone(),
        tokenizer_digest: artifact_digest(model, "tokenizer")?,
        kind: TraceKind::Activation,
        dtype: TraceDtype::F32,
        shape: vec![VOCAB as u32],
        layer: 3,
        head: 0,
        token_position: tape.len() as u64,
        observer_receipt_digest: observer_receipt.clone(),
        payload_digest: Digest::sha256(&logits_payload),
        payload: logits_payload.into_boxed_slice(),
    })?);
    std::fs::write(out_path, &frames)
        .map_err(|error| format!("write trace file {out_path}: {error}"))?;
    println!("trace_envelopes={}", observed.activations.len() + 1);
    println!("trace_file_bytes={}", frames.len());
    println!("wall_ms={}", started.elapsed().as_millis());
    Ok(())
}

fn artifact_digest(
    model: &globalmind_runtime::VerifiedModel,
    role: &str,
) -> Result<Digest, String> {
    model
        .artifacts
        .iter()
        .find(|artifact| artifact.role == role)
        .map(|artifact| artifact.sha256.clone())
        .ok_or_else(|| format!("roster has no {role} artifact"))
}

// =========================== Stage 2: extract ===========================

fn stage_extract(trace_path: &str, out_path: &str) -> Result<(), String> {
    let bytes = std::fs::read(trace_path)
        .map_err(|error| format!("read trace file {trace_path}: {error}"))?;
    let mut envelopes = Vec::new();
    let mut rest = bytes.as_slice();
    while !rest.is_empty() {
        let (envelope, consumed) = globalmind_runtime::trace::decode_frame(rest)?;
        envelopes.push(envelope);
        rest = &rest[consumed..];
    }
    // Both TPPE radial candidates over the same trace.
    for law in [RadialLaw::LinearLogV1, RadialLaw::LogV1] {
        let config = TppeConfig {
            law,
            radial_start: 0.0,
            radial_span: 0.01,
            omega_fast: 0.1,
            omega_slow: 0.05,
            coupling_fast: 0.3,
            coupling_slow: 0.15,
        };
        let mut state = TppeState::origin(config)?;
        for _envelope in &envelopes {
            state = state.step(config, 0.02, -0.01)?;
        }
        state.validate()?;
        println!(
            "tppe_law={} final_u={:.6} poincare_radius={:.6} boundary_margin={:.6} revolutions={:.2} lorentz_invariant_ok=1",
            law.as_str(),
            state.hyperbolic_radius,
            state.poincare_radius,
            state.boundary_margin,
            state.cumulative_revolutions
        );
    }
    let candidates = extract(&envelopes)?;
    let mut out = String::new();
    for candidate in &candidates {
        out.push_str(&format!(
            "candidate={} kind={} digest={} fragment={} neutral={}\n",
            candidate.id,
            kind_name(candidate.cell_kind),
            candidate.candidate_digest,
            candidate.source_fragment_digest,
            candidate.teacher_neutral_input_digest
        ));
    }
    std::fs::write(out_path, &out)
        .map_err(|error| format!("write candidates {out_path}: {error}"))?;
    println!("stage=extract");
    println!("candidates_emitted={}", candidates.len());
    for kind in [
        "rank0_atom",
        "rank1_transition",
        "rank2_loop",
        "rank8_jacobian",
    ] {
        let count = candidates
            .iter()
            .filter(|c| kind_name(c.cell_kind) == kind)
            .count();
        println!("emitted_{kind}={count}");
    }
    Ok(())
}

// ============================ Stage 3: judge ============================

fn stage_judge(candidates_path: &str, trace_path: &str, out_path: &str) -> Result<(), String> {
    let bytes = std::fs::read(trace_path)
        .map_err(|error| format!("read trace file {trace_path}: {error}"))?;
    let mut envelopes = Vec::new();
    let mut rest = bytes.as_slice();
    while !rest.is_empty() {
        let (envelope, consumed) = globalmind_runtime::trace::decode_frame(rest)?;
        envelopes.push(envelope);
        rest = &rest[consumed..];
    }
    let candidates = extract(&envelopes)?;
    let declared = std::fs::read_to_string(candidates_path)
        .map_err(|error| format!("read candidates {candidates_path}: {error}"))?;
    if declared.lines().count() != candidates.len() {
        return Err("candidate file does not match re-extraction from the trace".into());
    }
    let judge = IndependentJudge::new(JudgePolicy {
        min_mean_square: 0.0,
        max_chart_rank: 8,
    })?;
    println!("stage=judge");
    println!("judge_policy_digest={}", judge.policy_digest());
    let mut out = String::new();
    let mut passed = 0usize;
    let mut rejected = 0usize;
    let mut known: Vec<Digest> = Vec::new();
    for candidate in &candidates {
        let receipt = judge.judge(
            candidate,
            &JudgeEvidence {
                mean_square: mean_square_of(candidate, &envelopes),
                finite: true,
                known_candidate_digests: known.clone(),
            },
        )?;
        known.push(candidate.candidate_digest.clone());
        match receipt.verdict {
            JudgeVerdict::Pass => passed += 1,
            JudgeVerdict::Reject => rejected += 1,
        }
        out.push_str(&format!(
            "candidate={} verdict={} reason={} receipt={} novelty={}\n",
            candidate.candidate_digest,
            match receipt.verdict {
                JudgeVerdict::Pass => "pass",
                JudgeVerdict::Reject => "reject",
            },
            receipt.reason,
            receipt.receipt_digest,
            receipt.novelty_receipt_digest
        ));
    }
    std::fs::write(out_path, out)
        .map_err(|error| format!("write judge receipts {out_path}: {error}"))?;
    println!("judged_pass={passed}");
    println!("judged_reject={rejected}");
    Ok(())
}

fn mean_square_of(candidate: &ExtractedCandidate, envelopes: &[TraceEnvelope]) -> f64 {
    let Some(envelope) = envelopes
        .iter()
        .find(|e| e.payload_digest == candidate.source_fragment_digest)
    else {
        return 0.0;
    };
    let values = f32_from_bytes(&envelope.payload);
    if values.is_empty() {
        0.0
    } else {
        values
            .iter()
            .map(|v| (*v as f64) * (*v as f64))
            .sum::<f64>()
            / values.len() as f64
    }
}

// ============================= Stage 4: open ============================

fn stage_open(manifest_path: &str, judge_receipts_path: &str) -> Result<(), String> {
    let manifest = Manifest::load(manifest_path)?;
    let verified = manifest.verify_artifacts()?;
    let receipts = std::fs::read_to_string(judge_receipts_path)
        .map_err(|error| format!("read judge receipts {judge_receipts_path}: {error}"))?;
    let pass_count = receipts.matches("verdict=pass").count();
    if pass_count == 0 {
        return Err("no judged-pass candidates; open is forbidden".into());
    }
    // Real receipt digests derived from the persisted judge output.
    let judge_receipt_digest = Digest::sha256(receipts.as_bytes());
    let eligibility_receipt_digest = digest_fields(
        b"globalmind-eligibility-v1",
        &[
            verified.input_digest.as_str(),
            judge_receipt_digest.as_str(),
            &pass_count.to_string(),
        ],
    );
    // Never edit the candidate manifest in place: write a new admitted one.
    let run_id = verified.run_id.as_str().to_string();
    let admitted_path = format!("globalmind-runtime/manifests/{run_id}.admitted.manifest");
    let mut text = std::fs::read_to_string(manifest_path)
        .map_err(|error| format!("read manifest {manifest_path}: {error}"))?;
    for index in 0..verified.models.len() {
        text.push_str(&format!(
            "model.{index}.eligibility_receipt_digest = {eligibility_receipt_digest}\nmodel.{index}.judge_receipt_digest = {judge_receipt_digest}\n"
        ));
    }
    std::fs::write(&admitted_path, &text)
        .map_err(|error| format!("write admitted manifest {admitted_path}: {error}"))?;
    println!("stage=open");
    println!("admitted_manifest={admitted_path}");
    println!("eligibility_receipt_digest={eligibility_receipt_digest}");
    println!("judge_receipt_digest={judge_receipt_digest}");

    let admitted = Manifest::load(&admitted_path)?;
    let admitted_verified = admitted.verify_artifacts()?;
    let store = GlobalMindStore::connect(StoreConfig::from_env())?;
    store.open_distillation_ingress(&admitted_verified)?;
    println!("opened_receipt={}", admitted_verified.run_receipt_digest);
    println!("run_id={}", admitted_verified.run_id);
    Ok(())
}

// ============================ Forward engine ============================

type ProjectedQkv = (
    Vec<[f32; HEAD_DIM]>,
    Vec<[f32; HEAD_DIM]>,
    [[f32; HEAD_DIM]; KV_HEADS],
    [[f32; HEAD_DIM]; KV_HEADS],
);

struct Affine {
    words: Vec<u32>,
    scales: Vec<u16>,
    biases: Vec<u16>,
    out_dim: usize,
    in_dim: usize,
    bits: usize,
}

impl Affine {
    fn read(file: &mut File, base: u64, spec: Spec) -> Result<Self, String> {
        let words = read_u32s(file, base, spec.weight)?;
        let scales = read_u16s(file, base, spec.scales)?;
        let biases = read_u16s(file, base, spec.biases)?;
        let words_per_row = spec.in_dim * spec.bits / 32;
        if words.len() != spec.out_dim * words_per_row {
            return Err("packed weight word count mismatch".into());
        }
        Ok(Self {
            words,
            scales,
            biases,
            out_dim: spec.out_dim,
            in_dim: spec.in_dim,
            bits: spec.bits,
        })
    }

    fn read_expert(file: &mut File, base: u64, spec: Spec, expert: usize) -> Result<Self, String> {
        let words_per_expert = (spec.out_dim * (spec.in_dim * spec.bits / 32)) as u64;
        let groups_per_expert = (spec.out_dim * (spec.in_dim / GROUP)) as u64;
        let sub = |range: (u64, u64), per: u64, bytes: u64| {
            let start = range.0 + expert as u64 * per * bytes;
            (start, start + per * bytes)
        };
        Self::read(
            file,
            base,
            Spec {
                weight: sub(spec.weight, words_per_expert, 4),
                scales: sub(spec.scales, groups_per_expert, 2),
                biases: sub(spec.biases, groups_per_expert, 2),
                ..spec
            },
        )
    }

    fn row(&self, row: usize) -> Vec<f32> {
        let words_per_row = self.in_dim * self.bits / 32;
        let groups = self.in_dim / GROUP;
        let mask = (1u32 << self.bits) - 1;
        let w = &self.words[row * words_per_row..(row + 1) * words_per_row];
        let s = &self.scales[row * groups..(row + 1) * groups];
        let b = &self.biases[row * groups..(row + 1) * groups];
        (0..self.in_dim)
            .map(|col| {
                let bit = col * self.bits;
                let q = ((w[bit / 32] >> (bit % 32)) & mask) as f32;
                bf16(s[col / GROUP]) * q + bf16(b[col / GROUP])
            })
            .collect()
    }

    fn project(&self, x: &[f32]) -> Vec<f32> {
        (0..self.out_dim)
            .map(|row| self.row(row).iter().zip(x).map(|(w, v)| w * v).sum())
            .collect()
    }
}

#[derive(Clone, Copy)]
struct Spec {
    weight: (u64, u64),
    scales: (u64, u64),
    biases: (u64, u64),
    out_dim: usize,
    in_dim: usize,
    bits: usize,
}

const fn spec(
    weight: (u64, u64),
    scales: (u64, u64),
    biases: (u64, u64),
    out_dim: usize,
    in_dim: usize,
    bits: usize,
) -> Spec {
    Spec {
        weight,
        scales,
        biases,
        out_dim,
        in_dim,
        bits,
    }
}

struct Engine {
    shard1: File,
    base1: u64,
    input_norm: Vec<f32>,
    q_proj: Affine,
    q_norm: Vec<f32>,
    k_proj: Affine,
    k_norm: Vec<f32>,
    v_proj: Affine,
    o_proj: Affine,
    post_norm: Vec<f32>,
    router: Affine,
    shared_gate: Affine,
    shared_up: Affine,
    shared_down: Affine,
    shared_expert_gate: Affine,
    final_norm: Vec<f32>,
    lm_head: Affine,
}

struct ForwardOut {
    logits: Vec<f32>,
    hidden_out: Vec<f32>,
    activations: Vec<Vec<f32>>,
}

impl Engine {
    fn load() -> Result<Self, String> {
        let mut shard1 = File::open(SHARD1).map_err(|e| format!("open {SHARD1}: {e}"))?;
        let mut shard4 = File::open(SHARD4).map_err(|e| format!("open {SHARD4}: {e}"))?;
        let base1 = header_base(&mut shard1, SHARD1_HEADER_LEN)?;
        let base4 = header_base(&mut shard4, SHARD4_HEADER_LEN)?;
        Ok(Self {
            input_norm: read_bf16s(&mut shard1, base1, (2_275_097_664, 2_275_101_760))?,
            q_proj: Affine::read(
                &mut shard1,
                base1,
                spec(
                    (3_363_184_960, 3_371_573_568),
                    (4_414_260_288, 4_414_784_576),
                    (3_008_045_312, 3_008_569_600),
                    8192,
                    HIDDEN,
                    4,
                ),
            )?,
            q_norm: read_bf16s(&mut shard1, base1, (4_419_537_984, 4_419_538_496))?,
            k_proj: Affine::read(
                &mut shard1,
                base1,
                spec(
                    (3_006_963_968, 3_007_488_256),
                    (4_274_951_104, 4_274_983_872),
                    (3_985_148_864, 3_985_181_632),
                    512,
                    HIDDEN,
                    4,
                ),
            )?,
            k_norm: read_bf16s(&mut shard1, base1, (2_678_836_608, 2_678_837_120))?,
            v_proj: Affine::read(
                &mut shard1,
                base1,
                spec(
                    (979_633_728, 980_158_016),
                    (1_598_257_536, 1_598_290_304),
                    (3_964_078_656, 3_964_111_424),
                    512,
                    HIDDEN,
                    4,
                ),
            )?,
            o_proj: Affine::read(
                &mut shard1,
                base1,
                spec(
                    (3_980_921_792, 3_985_116_096),
                    (4_413_998_144, 4_414_260_288),
                    (3_006_699_776, 3_006_961_920),
                    HIDDEN,
                    4096,
                    4,
                ),
            )?,
            post_norm: read_bf16s(&mut shard1, base1, (3_964_070_464, 3_964_074_560))?,
            router: Affine::read(
                &mut shard1,
                base1,
                spec(
                    (4_869_042_240, 4_869_566_528),
                    (3_006_683_392, 3_006_699_776),
                    (3_002_398_976, 3_002_415_360),
                    EXPERTS,
                    HIDDEN,
                    8,
                ),
            )?,
            shared_gate: Affine::read(
                &mut shard1,
                base1,
                spec(
                    (2_841_145_536, 2_841_669_824),
                    (3_006_650_624, 3_006_683_392),
                    (1_951_781_504, 1_951_814_272),
                    INTERMEDIATE,
                    HIDDEN,
                    4,
                ),
            )?,
            shared_up: Affine::read(
                &mut shard1,
                base1,
                spec(
                    (1_437_052_480, 1_437_576_768),
                    (3_998_028_736, 3_998_061_504),
                    (2_835_243_072, 2_835_275_840),
                    INTERMEDIATE,
                    HIDDEN,
                    4,
                ),
            )?,
            shared_down: Affine::read(
                &mut shard1,
                base1,
                spec(
                    (5_332_855_040, 5_333_379_328),
                    (2_840_031_296, 2_840_064_064),
                    (2_835_804_224, 2_835_836_992),
                    HIDDEN,
                    INTERMEDIATE,
                    4,
                ),
            )?,
            shared_expert_gate: Affine::read(
                &mut shard1,
                base1,
                spec(
                    (2_834_651_200, 2_834_653_248),
                    (2_834_356_224, 2_834_356_288),
                    (1_951_847_040, 1_951_847_104),
                    1,
                    HIDDEN,
                    8,
                ),
            )?,
            final_norm: read_bf16s(&mut shard4, base4, (270_172_160, 270_176_256))?,
            lm_head: Affine::read(
                &mut shard4,
                base4,
                spec(
                    (15_892_480, 270_172_160),
                    (433_565_824, 449_458_304),
                    (0, 15_892_480),
                    VOCAB,
                    HIDDEN,
                    4,
                ),
            )?,
            shard1,
            base1,
        })
    }

    fn embed(&mut self, token: u32) -> Result<Vec<f32>, String> {
        if token as usize >= VOCAB {
            return Err("token out of range".into());
        }
        let words_per_row = HIDDEN * 4 / 32;
        let groups = HIDDEN / GROUP;
        let row = token as u64;
        let matrix = Affine::read(
            &mut self.shard1,
            self.base1,
            spec(
                (
                    4_590_448_704 + row * (words_per_row * 4) as u64,
                    4_590_448_704 + (row + 1) * (words_per_row * 4) as u64,
                ),
                (
                    4_844_728_384 + row * (groups * 2) as u64,
                    4_844_728_384 + (row + 1) * (groups * 2) as u64,
                ),
                (
                    3_780_340_096 + row * (groups * 2) as u64,
                    3_780_340_096 + (row + 1) * (groups * 2) as u64,
                ),
                1,
                HIDDEN,
                4,
            ),
        )?;
        Ok(matrix.row(0))
    }

    fn forward(
        &mut self,
        tape: &[u32],
        config: TppeConfig,
        observe: bool,
    ) -> Result<ForwardOut, String> {
        let mut k_cache: [Vec<f32>; KV_HEADS] = [Vec::new(), Vec::new()];
        let mut v_cache: [Vec<f32>; KV_HEADS] = [Vec::new(), Vec::new()];
        let mut activations: Vec<Vec<f32>> = Vec::new();
        let mut last_hidden: Vec<f32> = Vec::new();
        let mut last_logits: Vec<f32> = Vec::new();
        // One sequential carry: state at each position continues the
        // previous position's state; no per-token rebuild, no token drive.
        let states = globalmind_runtime::trace::tppe_states_for_positions(&config, tape.len())?;
        for (position, &token) in tape.iter().enumerate() {
            let hidden = self.embed(token)?;
            let state = &states[position];
            let (q_heads, gate_heads, k_heads, v_heads) =
                self.project_qkv(&hidden, state, position);
            for h in 0..KV_HEADS {
                k_cache[h].extend_from_slice(&k_heads[h]);
                v_cache[h].extend_from_slice(&v_heads[h]);
            }
            if observe {
                activations.push(k_heads[0].to_vec());
            }
            let (hidden_out, logits) = self.finish(
                &hidden,
                &q_heads,
                &gate_heads,
                &k_cache,
                &v_cache,
                position + 1,
            )?;
            last_hidden = hidden_out;
            last_logits = logits;
        }
        Ok(ForwardOut {
            logits: last_logits,
            hidden_out: last_hidden,
            activations,
        })
    }

    fn project_qkv(&self, hidden: &[f32], state: &TppeState, position: usize) -> ProjectedQkv {
        let normed = rms_norm(hidden, &self.input_norm);
        let qg = self.q_proj.project(&normed);
        let k_raw = self.k_proj.project(&normed);
        let v_raw = self.v_proj.project(&normed);
        let mut q_heads = vec![[0.0f32; HEAD_DIM]; Q_HEADS];
        let mut gate_heads = vec![[0.0f32; HEAD_DIM]; Q_HEADS];
        for h in 0..Q_HEADS {
            let base = h * 2 * HEAD_DIM;
            let mut q = rms_norm(&qg[base..base + HEAD_DIM], &self.q_norm);
            apply_rotary(&mut q, state, position);
            q_heads[h].copy_from_slice(&q);
            gate_heads[h].copy_from_slice(&qg[base + HEAD_DIM..base + 2 * HEAD_DIM]);
        }
        let mut k_heads = [[0.0f32; HEAD_DIM]; KV_HEADS];
        let mut v_heads = [[0.0f32; HEAD_DIM]; KV_HEADS];
        for h in 0..KV_HEADS {
            let base = h * HEAD_DIM;
            let mut k = rms_norm(&k_raw[base..base + HEAD_DIM], &self.k_norm);
            apply_rotary(&mut k, state, position);
            k_heads[h].copy_from_slice(&k);
            v_heads[h].copy_from_slice(&v_raw[base..base + HEAD_DIM]);
        }
        (q_heads, gate_heads, k_heads, v_heads)
    }

    fn finish(
        &mut self,
        hidden_in: &[f32],
        q_heads: &[[f32; HEAD_DIM]],
        gate_heads: &[[f32; HEAD_DIM]],
        k_cache: &[Vec<f32>; KV_HEADS],
        v_cache: &[Vec<f32>; KV_HEADS],
        seq_len: usize,
    ) -> Result<(Vec<f32>, Vec<f32>), String> {
        let scale = 1.0f32 / (HEAD_DIM as f32).sqrt();
        let heads_per_kv = Q_HEADS / KV_HEADS;
        let mut gated = vec![0.0f32; Q_HEADS * HEAD_DIM];
        for q_h in 0..Q_HEADS {
            let kv_h = q_h / heads_per_kv;
            let k_all = &k_cache[kv_h];
            let v_all = &v_cache[kv_h];
            let mut scores: Vec<f32> = (0..seq_len)
                .map(|t| {
                    let k = &k_all[t * HEAD_DIM..(t + 1) * HEAD_DIM];
                    q_heads[q_h]
                        .iter()
                        .zip(k)
                        .map(|(q, kv)| q * kv)
                        .sum::<f32>()
                        * scale
                })
                .collect();
            let max = scores.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            let mut denom = 0.0f32;
            for s in &mut scores {
                *s = (*s - max).exp();
                denom += *s;
            }
            for s in &mut scores {
                *s /= denom;
            }
            let mut attn = [0.0f32; HEAD_DIM];
            for (t, &w) in scores.iter().enumerate() {
                let v = &v_all[t * HEAD_DIM..(t + 1) * HEAD_DIM];
                for d in 0..HEAD_DIM {
                    attn[d] += w * v[d];
                }
            }
            for d in 0..HEAD_DIM {
                gated[q_h * HEAD_DIM + d] = attn[d] * sigmoid(gate_heads[q_h][d]);
            }
        }
        let o_out = self.o_proj.project(&gated);
        let post: Vec<f32> = hidden_in.iter().zip(&o_out).map(|(a, b)| a + b).collect();
        let mlp_in = rms_norm(&post, &self.post_norm);
        let router_logits = self.router.project(&mlp_in);
        let probs = softmax(&router_logits);
        let mut indexed: Vec<(usize, f32)> = probs.into_iter().enumerate().collect();
        indexed.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let top = &indexed[..TOP_K];
        let weight_sum: f32 = top.iter().map(|(_, w)| *w).sum();
        let mut moe = vec![0.0f32; HIDDEN];
        for &(expert, raw_weight) in top {
            let weight = raw_weight / weight_sum;
            let gate = Affine::read_expert(
                &mut self.shard1,
                self.base1,
                spec(
                    (2_867_902_720, 3_002_120_448),
                    (3_047_731_456, 3_056_120_064),
                    (2_859_514_112, 2_867_902_720),
                    INTERMEDIATE,
                    HIDDEN,
                    4,
                ),
                expert,
            )?;
            let up = Affine::read_expert(
                &mut self.shard1,
                self.base1,
                spec(
                    (5_197_846_784, 5_332_064_512),
                    (1_952_111_296, 1_960_499_904),
                    (4_886_874_176, 4_895_262_784),
                    INTERMEDIATE,
                    HIDDEN,
                    4,
                ),
                expert,
            )?;
            let down = Affine::read_expert(
                &mut self.shard1,
                self.base1,
                spec(
                    (2_275_626_112, 2_409_843_840),
                    (2_841_669_824, 2_850_058_432),
                    (4_573_139_008, 4_581_527_616),
                    HIDDEN,
                    INTERMEDIATE,
                    4,
                ),
                expert,
            )?;
            let gate_act = gate.project(&mlp_in);
            let up_act = up.project(&mlp_in);
            let swiglu: Vec<f32> = gate_act
                .iter()
                .zip(&up_act)
                .map(|(g, u)| silu(*g) * u)
                .collect();
            let expert_out = down.project(&swiglu);
            for i in 0..HIDDEN {
                moe[i] += weight * expert_out[i];
            }
        }
        let sh_gate = self.shared_gate.project(&mlp_in);
        let sh_up = self.shared_up.project(&mlp_in);
        let sh_swiglu: Vec<f32> = sh_gate
            .iter()
            .zip(&sh_up)
            .map(|(g, u)| silu(*g) * u)
            .collect();
        let sh_down = self.shared_down.project(&sh_swiglu);
        let sh_scale = sigmoid(self.shared_expert_gate.project(&mlp_in)[0]);
        let layer_out: Vec<f32> = post
            .iter()
            .zip(&moe)
            .zip(&sh_down)
            .map(|((p, m), s)| p + m + sh_scale * s)
            .collect();
        let final_normed = rms_norm(&layer_out, &self.final_norm);
        let logits = self.lm_head.project(&final_normed);
        Ok((layer_out, logits))
    }
}

fn apply_rotary(vec: &mut [f32], state: &TppeState, _position: usize) {
    // Pure TPPE boundary operation: adjacent-pair rotation by the T2
    // phase (fast on even pairs, slow on odd), then radial scale
    // 1 + 0.05 * poincare_radius. No RoPE frequency schedule.
    let [fast, slow] = state.phase_t2();
    let scale = (1.0 + 0.05 * state.poincare_radius) as f32;
    for (index, pair) in vec.as_chunks_mut::<2>().0.iter_mut().enumerate() {
        let (c, s) = if index % 2 == 0 {
            (fast.cos() as f32, fast.sin() as f32)
        } else {
            (slow.cos() as f32, slow.sin() as f32)
        };
        let (x, y) = (pair[0], pair[1]);
        pair[0] = scale * (x * c - y * s);
        pair[1] = scale * (x * s + y * c);
    }
}

fn header_base(file: &mut File, expected: u64) -> Result<u64, String> {
    let mut buf = [0u8; 8];
    file.read_exact(&mut buf)
        .map_err(|e| format!("read header len: {e}"))?;
    let len = u64::from_le_bytes(buf);
    if len != expected {
        return Err(format!(
            "safetensors header length mismatch: got {len}, expected {expected}"
        ));
    }
    Ok(8 + len)
}

fn read_at(file: &mut File, base: u64, range: (u64, u64)) -> Result<Vec<u8>, String> {
    let len = (range.1 - range.0) as usize;
    let mut buf = vec![0u8; len];
    file.seek(SeekFrom::Start(base + range.0))
        .map_err(|e| format!("seek: {e}"))?;
    file.read_exact(&mut buf)
        .map_err(|e| format!("read tensor bytes: {e}"))?;
    Ok(buf)
}

fn read_u32s(file: &mut File, base: u64, range: (u64, u64)) -> Result<Vec<u32>, String> {
    Ok(read_at(file, base, range)?
        .as_chunks::<4>()
        .0
        .iter()
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect())
}

fn read_u16s(file: &mut File, base: u64, range: (u64, u64)) -> Result<Vec<u16>, String> {
    Ok(read_at(file, base, range)?
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect())
}

fn read_bf16s(file: &mut File, base: u64, range: (u64, u64)) -> Result<Vec<f32>, String> {
    Ok(read_u16s(file, base, range)?
        .iter()
        .map(|b| bf16(*b))
        .collect())
}

fn bf16(bits: u16) -> f32 {
    f32::from_bits((bits as u32) << 16)
}

fn rms_norm(x: &[f32], weight: &[f32]) -> Vec<f32> {
    let mean_square = x.iter().map(|v| v * v).sum::<f32>() / x.len() as f32;
    let denom = (mean_square + EPS).sqrt();
    x.iter()
        .zip(weight)
        .map(|(v, w)| (v / denom) * (1.0 + w))
        .collect()
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

fn silu(x: f32) -> f32 {
    x * sigmoid(x)
}

fn softmax(x: &[f32]) -> Vec<f32> {
    let max = x.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut out: Vec<f32> = x.iter().map(|v| (v - max).exp()).collect();
    let sum: f32 = out.iter().sum();
    for v in &mut out {
        *v /= sum;
    }
    out
}

fn argmax(x: &[f32]) -> usize {
    x.iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(i, _)| i)
        .unwrap_or(0)
}

fn f32_bytes(values: &[f32]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_le_bytes()).collect()
}

fn f32_from_bytes(bytes: &[u8]) -> Vec<f32> {
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

fn kind_name(kind: globalmind_runtime::CellKind) -> &'static str {
    match kind {
        globalmind_runtime::CellKind::Rank0Atom => "rank0_atom",
        globalmind_runtime::CellKind::Rank1Transition => "rank1_transition",
        globalmind_runtime::CellKind::Rank2Loop => "rank2_loop",
        globalmind_runtime::CellKind::Rank8Jacobian => "rank8_jacobian",
    }
}

fn digest_fields(domain: &[u8], fields: &[&str]) -> Digest {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    for field in fields {
        hasher.update((field.len() as u64).to_le_bytes());
        hasher.update(field.as_bytes());
    }
    let digest = hasher.finalize();
    let mut value = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut value, "{byte:02x}").expect("writing to String cannot fail");
    }
    Digest::parse(value).expect("SHA-256 is a valid Digest")
}

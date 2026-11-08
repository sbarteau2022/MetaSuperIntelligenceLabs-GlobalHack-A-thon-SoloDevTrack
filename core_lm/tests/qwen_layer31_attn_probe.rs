//! Intermediate-level oracle for layer 31's full attention at position 0. Compares our q/gate
//! projection, value projection, and gated attention output (before o_proj) against the reference's
//! intermediates on the same input. The first intermediate that exceeds the 1% gate is the fault.
//! Real weights only: ignored by default.
use core_lm::qwen_forward::QwenModel;
use std::path::PathBuf;

const LAYER: usize = 31;
const HIDDEN: usize = 2048;
const GATE: f32 = 1e-2;

fn model_dir() -> PathBuf {
    std::env::var("QWEN_MODEL_DIR").map(PathBuf::from).unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/stewartbarteau".into());
        PathBuf::from(home).join("models/fused/Qwen3.6-MoE-Combined")
    })
}

fn read_f32(rel: &str) -> Vec<f32> {
    let path = format!("{}/tests/data/{}", env!("CARGO_MANIFEST_DIR"), rel);
    std::fs::read(&path).expect(&path).chunks_exact(4).map(|b| f32::from_le_bytes(b.try_into().unwrap())).collect()
}

fn rel_l2(ours: &[f32], exp: &[f32]) -> f32 {
    let num = ours.iter().zip(exp).map(|(a, b)| (a - b) * (a - b)).sum::<f32>().sqrt();
    let den = exp.iter().map(|b| b * b).sum::<f32>().sqrt();
    num / den
}

#[test]
#[ignore = "real Qwen3.6 weights; run with --ignored"]
fn layer31_attention_intermediates_match_reference() {
    let refs = read_f32("ref_hidden_760.bin");
    let x = &refs[LAYER * HIDDEN..(LAYER + 1) * HIDDEN];
    let mut model = QwenModel::load(&model_dir()).expect("load Qwen");
    model.reset_memory();
    model.attn_probe = Some(Vec::new());
    model.attention_block(LAYER, x, 0).expect("attention_block");
    let probe = model.attn_probe.clone().expect("probe");
    let names = ["qg", "v", "gated"];
    let refs_int = [read_f32("ref_l31_qg.bin"), read_f32("ref_l31_v.bin"), read_f32("ref_l31_gated.bin")];
    let mut first_bad: Option<&str> = None;
    for (i, name) in names.iter().enumerate() {
        let rel = rel_l2(&probe[i], &refs_int[i]);
        println!("INTERMEDIATE {name} rel_l2 {rel:.6}");
        if rel > GATE && first_bad.is_none() {
            first_bad = Some(name);
        }
    }
    assert!(first_bad.is_none(), "first intermediate over the gate: {:?}", first_bad);
}

/// o_proj alone, fed the reference gated attention output. If this is over the gate, the
/// output projection (or its quantized weights) is the fault; if not, the residual is.
#[test]
#[ignore = "real Qwen3.6 weights; run with --ignored"]
fn layer31_o_proj_matches_reference_given_reference_gated() {
    let refs = read_f32("ref_hidden_760.bin");
    let x = &refs[LAYER * HIDDEN..(LAYER + 1) * HIDDEN];
    let h_ref = read_f32("ref_h31.bin");
    let attn_ref: Vec<f32> = h_ref.iter().zip(x).map(|(h, x)| h - x).collect();
    let gated_ref = read_f32("ref_l31_gated.bin");
    let mut model = QwenModel::load(&model_dir()).expect("load Qwen");
    let y = model
        .weights
        .matvec_weight(&format!("language_model.model.layers.{LAYER}.self_attn.o_proj.weight"), HIDDEN, 4096, &gated_ref)
        .expect("o_proj");
    let rel = rel_l2(&y, &attn_ref);
    let attn_norm = attn_ref.iter().map(|v| v * v).sum::<f32>().sqrt();
    let x_norm = x.iter().map(|v| v * v).sum::<f32>().sqrt();
    println!("O_PROJ rel_l2 {rel:.6} attn_ref_norm {attn_norm:.6} input_norm {x_norm:.6}");
    assert!(rel <= GATE, "o_proj rel error {rel:.6} exceeds gate {GATE}");
}

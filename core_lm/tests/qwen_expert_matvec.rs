//! Parity gate: the per-expert quantized matvec must equal the full-dequantize reference
//! on the real Qwen3.6 switch_mlp weights, for a routed expert. Uses the real model directory
//! (QWEN_MODEL_DIR, default ~/models/fused/Qwen3.6-MoE-Combined), so it is ignored by default.

use core_lm::qwen_forward::QwenModel;
use std::path::PathBuf;

fn model_dir() -> PathBuf {
    std::env::var("QWEN_MODEL_DIR").map(PathBuf::from).unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/stewartbarteau".into());
        PathBuf::from(home).join("models/fused/Qwen3.6-MoE-Combined")
    })
}

#[test]
#[ignore = "loads real Qwen3.6 weights (multi-GB reads)"]
fn expert_matvec_matches_full_dequant_for_a_routed_expert() {
    let mut model = QwenModel::load(&model_dir()).expect("load Qwen");
    let h = model.config.hidden_size;
    let inter = model.config.moe_intermediate_size;
    let n_exp = model.config.num_experts;
    let expert = 5usize;
    let name = "language_model.model.layers.0.mlp.switch_mlp.gate_proj.weight";
    let x: Vec<f32> = (0..h).map(|i| ((i * 37 % 101) as f32 - 50.0) / 50.0).collect();

    let fast = model
        .weights
        .expert_matvec(name, n_exp, inter, h, expert, &x)
        .expect("expert_matvec");

    let full = model.weights.get_weight(name).expect("full dequant reference");
    let off = expert * inter * h;
    let reference: Vec<f32> = (0..inter)
        .map(|r| (0..h).map(|c| full[off + r * h + c] * x[c]).sum())
        .collect();

    let scale = reference.iter().fold(0f32, |m, v| m.max(v.abs())).max(1e-6);
    let max_err = fast
        .iter()
        .zip(&reference)
        .fold(0f32, |m, (a, b)| m.max((a - b).abs()));
    assert_eq!(fast.len(), inter);
    assert!(max_err / scale < 1e-3, "relative error {} exceeds 1e-3", max_err / scale);
}

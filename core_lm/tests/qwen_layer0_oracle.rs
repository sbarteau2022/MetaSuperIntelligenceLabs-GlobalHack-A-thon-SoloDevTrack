//! Layer oracle gate. Reference: oMLX's bundled mlx_lm Qwen3.5-MoE (see docs/engine-audit), run on
//! this machine on the same checkpoint, token 760 ("The" ... first token of the prompt), position 0.
//! Values are the L2 norm of the residual stream after the embedding and after each layer.
//! Reference is bf16 end to end, so the tolerance is 2% relative per layer, not 1e-3.
//! Real weights only: ignored by default (QWEN_MODEL_DIR, default ~/models/fused/Qwen3.6-MoE-Combined).

use core_lm::qwen_forward::QwenModel;
use std::path::PathBuf;

const TOKEN: u32 = 760;
const REF_NORMS: [f32; 41] = [
    // embedding, then after each layer 0..39 (oMLX mlx_lm reference, bf16)
    0.447266,
    1.117188,
    1.593750,
    3.078125,
    2.140625,
    2.187500,
    2.609375,
    13.062500,
    8.000000,
    7.343750,
    7.250000,
    41.250000,
    41.500000,
    41.750000,
    41.750000,
    42.000000,
    42.000000,
    42.000000,
    42.000000,
    42.250000,
    42.250000,
    42.250000,
    42.250000,
    42.500000,
    42.500000,
    42.250000,
    42.250000,
    42.500000,
    42.250000,
    42.250000,
    42.000000,
    42.250000,
    7.093750,
    6.500000,
    7.531250,
    19.125000,
    10.625000,
    10.687500,
    10.125000,
    16.875000,
    22.125000,
];
const REL_TOL: f32 = 2e-2;

fn model_dir() -> PathBuf {
    std::env::var("QWEN_MODEL_DIR").map(PathBuf::from).unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/stewartbarteau".into());
        PathBuf::from(home).join("models/fused/Qwen3.6-MoE-Combined")
    })
}

#[test]
#[ignore = "superseded by tests/qwen_layer_local.rs: cumulative bf16 drift, not a layer defect (fails at layer 29, 2.15%)"]
fn residual_norms_match_reference_for_first_layers() {
    let mut model = QwenModel::load(&model_dir()).expect("load Qwen");
    model.layer_trace = Some(Vec::new());
    model.forward(TOKEN, 0).expect("forward");
    let trace = model.layer_trace.clone().expect("trace");
    assert_eq!(trace.len(), 41, "expected embedding + 40 layers, got {}", trace.len());
    for (i, &r) in REF_NORMS.iter().enumerate() {
        let ours = trace[i];
        let rel = (ours - r).abs() / r;
        println!("NORM {i:2} ours {ours:12.6} ref {r:12.6} rel {rel:.5}");
        assert!(
            rel <= REL_TOL,
            "norm index {i}: ours {ours:.6} vs reference {r:.6} (relative error {rel:.4} > {REL_TOL})"
        );
    }
}

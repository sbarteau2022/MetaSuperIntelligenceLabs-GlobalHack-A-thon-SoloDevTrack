//! Greedy continuation parity against the reference on real weights. The reference is oMLX's
//! bundled mlx_lm Qwen3.5-MoE (bf16); for the prompt below, its greedy ids are recorded here.
//! This exercises multi-token attention over the exact KV cache, which the layer oracle does not.
//! Ignored by default (loads real weights); run with --ignored.

use core_lm::ops::argmax;
use core_lm::qwen_forward::QwenModel;
use std::path::PathBuf;

const PROMPT: [u32; 5] = [760, 6511, 314, 9338, 369]; // "The capital of France is"
const REF_GREEDY: [u32; 8] = [11751, 11, 264, 3177, 34756, 364, 1141, 25438]; // " Paris, a city renowned for its iconic"

fn model_dir() -> PathBuf {
    std::env::var("QWEN_MODEL_DIR").map(PathBuf::from).unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/stewartbarteau".into());
        PathBuf::from(home).join("models/fused/Qwen3.6-MoE-Combined")
    })
}

#[test]
#[ignore = "real Qwen3.6 weights; run with --ignored"]
fn greedy_continuation_matches_reference() {
    let mut model = QwenModel::load(&model_dir()).expect("load Qwen");
    model.reset_memory();
    let mut logits = Vec::new();
    for (pos, &id) in PROMPT.iter().enumerate() {
        logits = model.forward(id, pos).expect("prefill");
    }
    let mut ours = Vec::new();
    let mut pos = PROMPT.len();
    for _ in 0..REF_GREEDY.len() {
        let next = argmax(&logits).expect("argmax") as u32;
        ours.push(next);
        logits = model.forward(next, pos).expect("decode");
        pos += 1;
    }
    assert_eq!(ours, REF_GREEDY.to_vec(), "greedy ids diverge from the reference");
}

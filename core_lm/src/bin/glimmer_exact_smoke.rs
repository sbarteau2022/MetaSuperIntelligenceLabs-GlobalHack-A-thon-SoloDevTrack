//! Frozen-weight full-layer exact K/V continuation gate. No Surreal or mind claim.
use core_lm::exact_kv::ExactKvCache;
use core_lm::glimmer_forward::GlimmerModel;
use core_lm::tokenizer::BpeTokenizer;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model_dir = PathBuf::from("/Users/stewartbarteau/models/Muse-Glimmer-q4km-gs128-MLX");
    let tokenizer = BpeTokenizer::from_json_file(&model_dir.join("tokenizer.json"))?;
    let mut model = GlimmerModel::load(&model_dir)?;
    model.weights.retain_cache = false;
    let mut tokens = tokenizer.encode("The capital of France is")?;
    tokens.insert(0, model.config.bos_token_id);
    if let Ok(path) = std::env::var("GLIMMER_EMBED_RAW_OUT") {
        let row = model.weights.get_weight_row(
            "model.embed_tokens.weight",
            tokens[0] as usize,
            model.config.hidden_size,
        )?;
        std::fs::write(path, row.iter().flat_map(|value| value.to_le_bytes()).collect::<Vec<_>>())?;
    }
    if let Ok(path) = std::env::var("GLIMMER_EMBED_OUT") {
        let embedding = model.embed_token(tokens[0])?;
        std::fs::write(path, embedding.iter().flat_map(|value| value.to_le_bytes()).collect::<Vec<_>>())?;
    }
    if let Ok(path) = std::env::var("GLIMMER_QPROJ0_OUT") {
        let embedding = model.embed_token(tokens[0])?;
        let norm = model.weights.get_weight("model.layers.0.input_layernorm.weight")?;
        if let Ok(weight_path) = std::env::var("GLIMMER_NORM0_WEIGHT_OUT") {
            std::fs::write(
                weight_path,
                norm.iter().flat_map(|value| value.to_le_bytes()).collect::<Vec<_>>(),
            )?;
        }
        let input = core_lm::ops::rmsnorm(&embedding, &norm, model.config.rms_norm_eps)?;
        if let Ok(input_path) = std::env::var("GLIMMER_PRENORM0_OUT") {
            std::fs::write(
                input_path,
                input.iter().flat_map(|value| value.to_le_bytes()).collect::<Vec<_>>(),
            )?;
        }
        let q = model.weights.matvec_weight(
            "model.layers.0.self_attn.q_proj.weight",
            2 * model.config.num_attention_heads * model.config.head_dim,
            model.config.hidden_size,
            &input,
        )?;
        std::fs::write(path, q.iter().flat_map(|value| value.to_le_bytes()).collect::<Vec<_>>())?;
    }
    if tokens.len() < 2 {
        return Err("real prompt yielded no prefix".into());
    }
    let continuation = *tokens.last().ok_or("missing continuation token")?;
    let prefix_len = tokens.len() - 1;
    let mut cache = ExactKvCache::new(
        model.config.num_hidden_layers,
        model.config.num_attention_heads,
        model.config.num_key_value_heads,
        model.config.head_dim,
    )?;
    for (position, &token) in tokens[..prefix_len].iter().enumerate() {
        model.forward_with_backend(token, position, &mut cache)?;
    }
    cache.verify_complete_prefix(prefix_len)?;
    let mut restored = cache.clone();
    let uninterrupted = model.forward_with_backend(continuation, prefix_len, &mut cache)?;
    let resumed = model.forward_with_backend(continuation, prefix_len, &mut restored)?;
    cache.verify_complete_prefix(tokens.len())?;
    restored.verify_complete_prefix(tokens.len())?;
    if uninterrupted.len() != resumed.len()
        || !uninterrupted
            .iter()
            .zip(&resumed)
            .all(|(a, b)| a.to_bits() == b.to_bits())
    {
        return Err("full Glimmer exact K/V continuation differs".into());
    }
    let top = resumed
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .ok_or("no logits")?;
    if let Ok(path) = std::env::var("GLIMMER_LOGITS_OUT") {
        let bytes = resumed
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect::<Vec<_>>();
        std::fs::write(path, bytes)?;
    }
    println!(
        "PASS real Glimmer 52-layer exact K/V in-memory continuation: prefix_tokens={} logits={} argmax={} logit={:.4}",
        prefix_len,
        resumed.len(),
        top.0,
        top.1
    );
    Ok(())
}

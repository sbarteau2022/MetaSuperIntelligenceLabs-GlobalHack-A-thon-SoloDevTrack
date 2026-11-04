//! Glimmer model loader for safetensors with affine Q4 quantization.
//!
//! Loads the Muse-Glimmer-q4km-gs128-MLX model from 4 sharded safetensors files,
//! handling the affine quantization format with scales and biases per group.

use crate::{
    Digest, GlimmerLayerConfig,
};
use safetensors::SafeTensors;
use sha2::{Digest as ShaDigest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Glimmer model architecture constants
pub const GLIMMER_HIDDEN_SIZE: usize = 6656;
pub const GLIMMER_NUM_LAYERS: usize = 52;
pub const GLIMMER_NUM_Q_HEADS: usize = 32;
pub const GLIMMER_NUM_KV_HEADS: usize = 2;
pub const GLIMMER_HEAD_DIM: usize = 128;
pub const GLIMMER_VOCAB_SIZE: usize = 202048;
pub const GLIMMER_INTERMEDIATE_SIZE: usize = 19968;
pub const GLIMMER_QUANT_GROUP_SIZE: usize = 32;
pub const GLIMMER_RMS_NORM_EPS: f32 = 1e-5;
pub const GLIMMER_ROPE_THETA: f64 = 500_000.0;
pub const GLIMMER_PARTIAL_ROTARY_FACTOR: f32 = 0.25;
pub const GLIMMER_SLIDING_WINDOW: usize = 2048;

/// Affine quantized weight matrix
#[derive(Debug, Clone)]
pub struct AffineQuantizedMatrix {
    pub weight: Vec<u8>,      // Packed int4 weights
    pub scales: Vec<f32>,     // BF16 scales per group
    pub biases: Vec<f32>,     // BF16 biases per group
    pub out_dim: usize,
    pub in_dim: usize,
    pub bits: usize,
    pub group_size: usize,
}

/// Dequantized weight matrix (for computation)
#[derive(Debug, Clone)]
pub struct DequantizedMatrix {
    pub weight: Vec<f32>,     // [out_dim, in_dim]
    pub out_dim: usize,
    pub in_dim: usize,
}

/// RMS normalization weight
#[derive(Debug, Clone)]
pub struct RmsNorm {
    pub weight: Vec<f32>,     // [hidden_size]
}

/// Embedding weight (quantized)
#[derive(Debug, Clone)]
pub struct EmbeddingWeight {
    pub weight: AffineQuantizedMatrix,
}

/// Output projection weight (quantized)
#[derive(Debug, Clone)]
pub struct OutputProjection {
    pub weight: AffineQuantizedMatrix,
}

/// Complete Glimmer model weights
#[derive(Debug)]
pub struct GlimmerModel {
    pub embedding: EmbeddingWeight,
    pub layers: Vec<GlimmerLayerWeights>,
    pub final_norm: RmsNorm,
    pub output_projection: OutputProjection,
    pub layer_configs: Vec<GlimmerLayerConfig>,
    pub config_digest: Digest,
}

/// Single layer weights for Glimmer
#[derive(Debug, Clone)]
pub struct GlimmerLayerWeights {
    pub input_layernorm: RmsNorm,
    pub q_proj: AffineQuantizedMatrix,
    pub q_norm: RmsNorm,
    pub k_proj: AffineQuantizedMatrix,
    pub k_norm: RmsNorm,
    pub v_proj: AffineQuantizedMatrix,
    pub o_proj: AffineQuantizedMatrix,
    pub post_attn_norm: RmsNorm,
    pub gate_proj: AffineQuantizedMatrix,
    pub up_proj: AffineQuantizedMatrix,
    pub down_proj: AffineQuantizedMatrix,
    pub post_ffn_norm: RmsNorm,
}

/// KV Cache for a single layer
#[derive(Debug, Clone)]
pub struct LayerKVCache {
    pub k_cache: [Vec<f32>; GLIMMER_NUM_KV_HEADS],  // [seq_len * head_dim] per head
    pub v_cache: [Vec<f32>; GLIMMER_NUM_KV_HEADS],
}

/// Complete KV cache for all layers
#[derive(Debug, Clone)]
pub struct FullKVCache {
    pub layers: Vec<LayerKVCache>,  // One per layer
}

/// Forward pass output
#[derive(Debug, Clone)]
pub struct ForwardOutput {
    pub logits: Vec<f32>,           // Last token logits
    pub hidden_states: Vec<Vec<f32>>,  // Hidden states for all tokens
    pub kv_cache: FullKVCache,      // Captured KV cache
}

/// Load Glimmer model from safetensors directory
pub fn load_glimmer_model(model_dir: &Path) -> Result<GlimmerModel, String> {
    // Load config.json to get layer_types
    let config_path = model_dir.join("config.json");
    let config_json = fs::read_to_string(&config_path)
        .map_err(|e| format!("read config.json: {}", e))?;
    let config: serde_json::Value = serde_json::from_str(&config_json)
        .map_err(|e| format!("parse config.json: {}", e))?;

    // This candidate allocates a dense f32 copy of every matrix and assumes
    // uniform Q4 layers. The frozen checkpoint has mixed-bit projections.
    // Refuse it before reading four large shards or emitting invalid logits.
    if config["quantization"].as_object().is_some_and(|q| {
        q.keys().any(|key| key.starts_with("model.layers."))
    }) {
        return Err("candidate Glimmer loader does not support the checkpoint's mixed-bit quantization; use the native mapped-weight executor".into());
    }

    let layer_types: Vec<String> = config["layer_types"]
        .as_array()
        .ok_or("layer_types not found in config")?
        .iter()
        .map(|v| v.as_str().unwrap_or("").to_string())
        .collect();

    if layer_types.len() != GLIMMER_NUM_LAYERS {
        return Err(format!("expected {} layers, got {}", GLIMMER_NUM_LAYERS, layer_types.len()));
    }

    let layer_type_strs: Vec<&str> = layer_types.iter().map(|s| s.as_str()).collect();
    let layer_configs = crate::build_glimmer_layer_configs(&layer_type_strs)?;

    // Load all 4 safetensors shards - keep data alive
    let shard_paths: Vec<PathBuf> = (1..=4)
        .map(|i| model_dir.join(format!("model-{:05}-of-00004.safetensors", i)))
        .collect();

    for path in &shard_paths {
        if !path.exists() {
            return Err(format!("shard not found: {}", path.display()));
        }
    }

    // Read all shard data first, then deserialize
    let shard_data: Vec<Vec<u8>> = shard_paths
        .iter()
        .map(|path| fs::read(path).map_err(|e| format!("read {}: {}", path.display(), e)))
        .collect::<Result<Vec<_>, _>>()?;

    let shards: Vec<SafeTensors> = shard_data
        .iter()
        .map(|data| SafeTensors::deserialize(data).map_err(|e| format!("deserialize: {}", e)))
        .collect::<Result<Vec<_>, _>>()?;

    // Build tensor name -> (shard_idx, tensor_view) map
    let mut tensor_map = HashMap::new();
    for (shard_idx, shard) in shards.iter().enumerate() {
        for name in shard.names() {
            tensor_map.insert(name.to_string(), shard_idx);
        }
    }

    // Load embedding
    let embedding = load_embedding(&shards, &tensor_map)?;

    // Load layers
    let mut layers = Vec::with_capacity(GLIMMER_NUM_LAYERS);
    for layer_idx in 0..GLIMMER_NUM_LAYERS {
        let layer = load_layer(&shards, &tensor_map, layer_idx)?;
        layers.push(layer);
    }

    // Load final norm
    let final_norm = load_rms_norm(&shards, &tensor_map, "model.norm.weight")?;

    // Load output projection
    let output_projection = load_output_projection(&shards, &tensor_map)?;

    // Compute config digest
    let config_digest = compute_config_digest(&config_json)?;

    Ok(GlimmerModel {
        embedding,
        layers,
        final_norm,
        output_projection,
        layer_configs,
        config_digest,
    })
}

/// Load embedding weight
fn load_embedding(
    shards: &[SafeTensors],
    tensor_map: &HashMap<String, usize>,
) -> Result<EmbeddingWeight, String> {
    let weight = load_affine_quantized_matrix(
        shards,
        tensor_map,
        "model.embed_tokens.weight",
        "model.embed_tokens.scales",
        "model.embed_tokens.biases",
        GLIMMER_VOCAB_SIZE,
        GLIMMER_HIDDEN_SIZE,
        4, // Q4
        GLIMMER_QUANT_GROUP_SIZE,
    )?;
    Ok(EmbeddingWeight { weight })
}

/// Load output projection (lm_head)
fn load_output_projection(
    shards: &[SafeTensors],
    tensor_map: &HashMap<String, usize>,
) -> Result<OutputProjection, String> {
    // lm_head uses Q5 quantization per config
    let weight = load_affine_quantized_matrix(
        shards,
        tensor_map,
        "lm_head.weight",
        "lm_head.scales",
        "lm_head.biases",
        GLIMMER_VOCAB_SIZE,
        GLIMMER_HIDDEN_SIZE,
        5, // Q5
        128, // group_size 128 for lm_head
    )?;
    Ok(OutputProjection { weight })
}

/// Load a single layer's weights
fn load_layer(
    shards: &[SafeTensors],
    tensor_map: &HashMap<String, usize>,
    layer_idx: usize,
) -> Result<GlimmerLayerWeights, String> {
    let prefix = format!("model.layers.{}", layer_idx);

    let input_layernorm = load_rms_norm(shards, tensor_map, &format!("{}.input_layernorm.weight", prefix))?;
    let q_proj = load_affine_quantized_matrix(
        shards, tensor_map,
        &format!("{}.self_attn.q_proj.weight", prefix),
        &format!("{}.self_attn.q_proj.scales", prefix),
        &format!("{}.self_attn.q_proj.biases", prefix),
        GLIMMER_HIDDEN_SIZE * 2, // q_proj outputs q and gate concatenated
        GLIMMER_HIDDEN_SIZE,
        4,
        GLIMMER_QUANT_GROUP_SIZE,
    )?;
    let q_norm = load_rms_norm(shards, tensor_map, &format!("{}.self_attn.q_norm.weight", prefix))?;
    let k_proj = load_affine_quantized_matrix(
        shards, tensor_map,
        &format!("{}.self_attn.k_proj.weight", prefix),
        &format!("{}.self_attn.k_proj.scales", prefix),
        &format!("{}.self_attn.k_proj.biases", prefix),
        GLIMMER_NUM_KV_HEADS * GLIMMER_HEAD_DIM,
        GLIMMER_HIDDEN_SIZE,
        4,
        GLIMMER_QUANT_GROUP_SIZE,
    )?;
    let k_norm = load_rms_norm(shards, tensor_map, &format!("{}.self_attn.k_norm.weight", prefix))?;
    let v_proj = load_affine_quantized_matrix(
        shards, tensor_map,
        &format!("{}.self_attn.v_proj.weight", prefix),
        &format!("{}.self_attn.v_proj.scales", prefix),
        &format!("{}.self_attn.v_proj.biases", prefix),
        GLIMMER_NUM_KV_HEADS * GLIMMER_HEAD_DIM,
        GLIMMER_HIDDEN_SIZE,
        4,
        GLIMMER_QUANT_GROUP_SIZE,
    )?;
    let o_proj = load_affine_quantized_matrix(
        shards, tensor_map,
        &format!("{}.self_attn.o_proj.weight", prefix),
        &format!("{}.self_attn.o_proj.scales", prefix),
        &format!("{}.self_attn.o_proj.biases", prefix),
        GLIMMER_HIDDEN_SIZE,
        GLIMMER_NUM_Q_HEADS * GLIMMER_HEAD_DIM,
        4,
        GLIMMER_QUANT_GROUP_SIZE,
    )?;
    let post_attn_norm = load_rms_norm(shards, tensor_map, &format!("{}.post_attn_norm.weight", prefix))?;
    let gate_proj = load_affine_quantized_matrix(
        shards, tensor_map,
        &format!("{}.mlp.gate_proj.weight", prefix),
        &format!("{}.mlp.gate_proj.scales", prefix),
        &format!("{}.mlp.gate_proj.biases", prefix),
        GLIMMER_INTERMEDIATE_SIZE,
        GLIMMER_HIDDEN_SIZE,
        4,
        GLIMMER_QUANT_GROUP_SIZE,
    )?;
    let up_proj = load_affine_quantized_matrix(
        shards, tensor_map,
        &format!("{}.mlp.up_proj.weight", prefix),
        &format!("{}.mlp.up_proj.scales", prefix),
        &format!("{}.mlp.up_proj.biases", prefix),
        GLIMMER_INTERMEDIATE_SIZE,
        GLIMMER_HIDDEN_SIZE,
        4,
        GLIMMER_QUANT_GROUP_SIZE,
    )?;
    let down_proj = load_affine_quantized_matrix(
        shards, tensor_map,
        &format!("{}.mlp.down_proj.weight", prefix),
        &format!("{}.mlp.down_proj.scales", prefix),
        &format!("{}.mlp.down_proj.biases", prefix),
        GLIMMER_HIDDEN_SIZE,
        GLIMMER_INTERMEDIATE_SIZE,
        4,
        GLIMMER_QUANT_GROUP_SIZE,
    )?;
    let post_ffn_norm = load_rms_norm(shards, tensor_map, &format!("{}.post_ffn_norm.weight", prefix))?;

    Ok(GlimmerLayerWeights {
        input_layernorm,
        q_proj,
        q_norm,
        k_proj,
        k_norm,
        v_proj,
        o_proj,
        post_attn_norm,
        gate_proj,
        up_proj,
        down_proj,
        post_ffn_norm,
    })
}

/// Load RMS norm weight
fn load_rms_norm(
    shards: &[SafeTensors],
    tensor_map: &HashMap<String, usize>,
    name: &str,
) -> Result<RmsNorm, String> {
    let shard_idx = *tensor_map.get(name).ok_or_else(|| format!("tensor not found: {}", name))?;
    let tensor = shards[shard_idx].tensor(name)
        .map_err(|e| format!("tensor {}: {}", name, e))?;

    let data = tensor.data();
    if data.len() % 2 != 0 {
        return Err(format!("RMS norm weight {} has odd byte length", name));
    }

    let weight: Vec<f32> = data
        .chunks_exact(2)
        .map(|chunk| {
            let bits = u16::from_le_bytes([chunk[0], chunk[1]]);
            f32::from_bits((bits as u32) << 16)
        })
        .collect();

    Ok(RmsNorm { weight })
}

/// Load affine quantized matrix from safetensors
fn load_affine_quantized_matrix(
    shards: &[SafeTensors],
    tensor_map: &HashMap<String, usize>,
    weight_name: &str,
    scales_name: &str,
    biases_name: &str,
    out_dim: usize,
    in_dim: usize,
    bits: usize,
    group_size: usize,
) -> Result<AffineQuantizedMatrix, String> {
    let weight_shard = *tensor_map.get(weight_name).ok_or_else(|| format!("weight not found: {}", weight_name))?;
    let scales_shard = *tensor_map.get(scales_name).ok_or_else(|| format!("scales not found: {}", scales_name))?;
    let biases_shard = *tensor_map.get(biases_name).ok_or_else(|| format!("biases not found: {}", biases_name))?;

    let weight_tensor = shards[weight_shard].tensor(weight_name)
        .map_err(|e| format!("weight tensor {}: {}", weight_name, e))?;
    let scales_tensor = shards[scales_shard].tensor(scales_name)
        .map_err(|e| format!("scales tensor {}: {}", scales_name, e))?;
    let biases_tensor = shards[biases_shard].tensor(biases_name)
        .map_err(|e| format!("biases tensor {}: {}", biases_name, e))?;

    let weight_data = weight_tensor.data().to_vec();
    let scales_data = read_bf16_tensor(&scales_tensor)?;
    let biases_data = read_bf16_tensor(&biases_tensor)?;

    let groups_per_row = in_dim / group_size;
    let expected_scales = out_dim * groups_per_row;
    if scales_data.len() != expected_scales {
        return Err(format!(
            "scales length mismatch for {}: expected {}, got {}",
            weight_name, expected_scales, scales_data.len()
        ));
    }
    if biases_data.len() != expected_scales {
        return Err(format!(
            "biases length mismatch for {}: expected {}, got {}",
            weight_name, expected_scales, biases_data.len()
        ));
    }

    Ok(AffineQuantizedMatrix {
        weight: weight_data,
        scales: scales_data,
        biases: biases_data,
        out_dim,
        in_dim,
        bits,
        group_size,
    })
}

/// Read BF16 tensor data as f32
fn read_bf16_tensor(tensor: &safetensors::tensor::TensorView) -> Result<Vec<f32>, String> {
    let data = tensor.data();
    if data.len() % 2 != 0 {
        return Err("BF16 tensor has odd byte length".into());
    }
    Ok(data
        .chunks_exact(2)
        .map(|chunk| {
            let bits = u16::from_le_bytes([chunk[0], chunk[1]]);
            f32::from_bits((bits as u32) << 16)
        })
        .collect())
}

/// Dequantize affine quantized matrix to f32
pub fn dequantize_matrix(matrix: &AffineQuantizedMatrix) -> DequantizedMatrix {
    let mut weight = vec![0.0f32; matrix.out_dim * matrix.in_dim];
    let groups_per_row = matrix.in_dim / matrix.group_size;
    let mask = (1u32 << matrix.bits) - 1;
    let words_per_row = (matrix.in_dim * matrix.bits) / 32;

    for row in 0..matrix.out_dim {
        let w_row = &matrix.weight[row * words_per_row * 4..(row + 1) * words_per_row * 4];
        let s_row = &matrix.scales[row * groups_per_row..(row + 1) * groups_per_row];
        let b_row = &matrix.biases[row * groups_per_row..(row + 1) * groups_per_row];

        for col in 0..matrix.in_dim {
            let bit = col * matrix.bits;
            let word_idx = bit / 32;
            let shift = bit % 32;
            let word = u32::from_le_bytes([
                w_row[word_idx * 4],
                w_row[word_idx * 4 + 1],
                w_row[word_idx * 4 + 2],
                w_row[word_idx * 4 + 3],
            ]);
            let q = ((word >> shift) & mask) as f32;
            let g = col / matrix.group_size;
            let scale = s_row[g];
            let bias = b_row[g];
            weight[row * matrix.in_dim + col] = scale * q + bias;
        }
    }

    DequantizedMatrix {
        weight,
        out_dim: matrix.out_dim,
        in_dim: matrix.in_dim,
    }
}

/// Project vector through dequantized matrix: y = x @ W^T
pub fn project_dequantized(x: &[f32], matrix: &DequantizedMatrix) -> Vec<f32> {
    assert_eq!(x.len(), matrix.in_dim);
    let mut out = vec![0.0f32; matrix.out_dim];
    for row in 0..matrix.out_dim {
        let mut sum = 0.0f32;
        let w_row = &matrix.weight[row * matrix.in_dim..(row + 1) * matrix.in_dim];
        for col in 0..matrix.in_dim {
            sum += x[col] * w_row[col];
        }
        out[row] = sum;
    }
    out
}

/// Apply RMS normalization
pub fn rms_norm(x: &mut [f32], weight: &[f32], eps: f32) {
    assert_eq!(x.len(), weight.len());
    let mut sum_sq = 0.0f32;
    for &v in x.iter() {
        sum_sq += v * v;
    }
    let inv_rms = 1.0f32 / ((sum_sq / x.len() as f32) + eps).sqrt();
    for i in 0..x.len() {
        x[i] = x[i] * inv_rms * (1.0f32 + weight[i]);
    }
}

/// Compute config digest for verification
fn compute_config_digest(config_json: &str) -> Result<Digest, String> {
    let mut h = Sha256::new();
    h.update(b"glimmer-config-v1\0");
    h.update(config_json.as_bytes());
    Ok(Digest::parse(&format!("{:x}", h.finalize())).unwrap())
}

/// SiLU activation
fn silu(x: f32) -> f32 {
    let sig = 1.0f32 / (1.0f32 + (-x).exp());
    x * sig
}

impl GlimmerModel {
    /// Get the model config digest
    pub fn config_digest(&self) -> Digest {
        self.config_digest.clone()
    }

    /// Embed token IDs
    pub fn embed_tokens(&self, token_ids: &[u32]) -> Result<Vec<Vec<f32>>, String> {
        let dequantized = dequantize_matrix(&self.embedding.weight);
        let mut embeddings = Vec::with_capacity(token_ids.len());
        for &token_id in token_ids {
            let row_start = (token_id as usize) * dequantized.in_dim;
            let row_end = row_start + dequantized.in_dim;
            if row_end > dequantized.weight.len() {
                return Err(format!("token_id {} out of range", token_id));
            }
            embeddings.push(dequantized.weight[row_start..row_end].to_vec());
        }
        Ok(embeddings)
    }

    /// Run full forward pass for a sequence of tokens, capturing KV cache
    pub fn forward_with_kv_cache(&self, token_ids: &[u32]) -> Result<ForwardOutput, String> {
        // Embed tokens
        let mut hidden = self.embed_tokens(token_ids)?;

        // Precompute RoPE frequencies for max sequence length
        let max_seq_len = token_ids.len();
        let rope_config = crate::RopeConfig::default();
        let rope_freqs = crate::position_transform::RopeFrequencies::new(&rope_config, max_seq_len);

        // Initialize KV cache for all layers
        let mut kv_cache = FullKVCache {
            layers: (0..GLIMMER_NUM_LAYERS).map(|_| LayerKVCache {
                k_cache: [Vec::new(), Vec::new()],
                v_cache: [Vec::new(), Vec::new()],
            }).collect(),
        };

        // Process through all layers
        for (layer_idx, layer_weights) in self.layers.iter().enumerate() {
            let layer_config = &self.layer_configs[layer_idx];
            hidden = self.forward_layer(
                &hidden,
                layer_weights,
                layer_config,
                &rope_freqs,
                token_ids.len(),
                &mut kv_cache.layers[layer_idx],
            )?;
        }

        // Final norm
        for h in &mut hidden {
            rms_norm(h, &self.final_norm.weight, GLIMMER_RMS_NORM_EPS);
        }

        // Output projection (lm_head) - only for last token
        let dequantized_lm_head = dequantize_matrix(&self.output_projection.weight);
        let last_hidden = &hidden[hidden.len() - 1];
        let logits = project_dequantized(last_hidden, &dequantized_lm_head);

        // Apply logit cap (soft cap)
        let cap_temp = 20.0f32;
        let multiplier = 0.19611613513818404f32;
        let mut final_logits = vec![0.0f32; GLIMMER_VOCAB_SIZE];
        for (i, logit) in logits.iter().enumerate() {
            if i < GLIMMER_VOCAB_SIZE {
                final_logits[i] = cap_temp * (logit / cap_temp).tanh() * multiplier;
            }
        }

        Ok(ForwardOutput {
            logits: final_logits,
            hidden_states: hidden,
            kv_cache,
        })
    }

    /// Run forward pass with pre-filled KV cache (for restore test)
    pub fn forward_with_injected_kv(
        &self,
        token_ids: &[u32],
        injected_kv: &FullKVCache,
    ) -> Result<ForwardOutput, String> {
        // Embed tokens
        let mut hidden = self.embed_tokens(token_ids)?;

        // Precompute RoPE frequencies for max sequence length
        let prefix_len = injected_kv.layers[0].k_cache[0].len() / GLIMMER_HEAD_DIM;
        let max_seq_len = token_ids.len() + prefix_len;
        let rope_config = crate::RopeConfig::default();
        let rope_freqs = crate::position_transform::RopeFrequencies::new(&rope_config, max_seq_len);

        // Initialize KV cache with injected values
        let mut kv_cache = injected_kv.clone();

        // Process through all layers
        for (layer_idx, layer_weights) in self.layers.iter().enumerate() {
            let layer_config = &self.layer_configs[layer_idx];
            hidden = self.forward_layer(
                &hidden,
                layer_weights,
                layer_config,
                &rope_freqs,
                token_ids.len(),
                &mut kv_cache.layers[layer_idx],
            )?;
        }

        // Final norm
        for h in &mut hidden {
            rms_norm(h, &self.final_norm.weight, GLIMMER_RMS_NORM_EPS);
        }

        // Output projection (lm_head) - only for last token
        let dequantized_lm_head = dequantize_matrix(&self.output_projection.weight);
        let last_hidden = &hidden[hidden.len() - 1];
        let logits = project_dequantized(last_hidden, &dequantized_lm_head);

        // Apply logit cap (soft cap)
        let cap_temp = 20.0f32;
        let multiplier = 0.19611613513818404f32;
        let mut final_logits = vec![0.0f32; GLIMMER_VOCAB_SIZE];
        for (i, logit) in logits.iter().enumerate() {
            if i < GLIMMER_VOCAB_SIZE {
                final_logits[i] = cap_temp * (logit / cap_temp).tanh() * multiplier;
            }
        }

        Ok(ForwardOutput {
            logits: final_logits,
            hidden_states: hidden,
            kv_cache,
        })
    }

    /// Forward pass through a single layer with KV cache
    fn forward_layer(
        &self,
        hidden: &[Vec<f32>],
        layer: &GlimmerLayerWeights,
        layer_config: &GlimmerLayerConfig,
        rope_freqs: &crate::position_transform::RopeFrequencies,
        seq_len: usize,
        kv_cache: &mut LayerKVCache,
    ) -> Result<Vec<Vec<f32>>, String> {
        let num_q_heads = GLIMMER_NUM_Q_HEADS;
        let num_kv_heads = GLIMMER_NUM_KV_HEADS;
        let head_dim = GLIMMER_HEAD_DIM;
        let hidden_size = GLIMMER_HIDDEN_SIZE;
        let intermediate_size = GLIMMER_INTERMEDIATE_SIZE;
        let heads_per_kv = num_q_heads / num_kv_heads;

        // Dequantize layer weights
        let q_proj_deq = dequantize_matrix(&layer.q_proj);
        let k_proj_deq = dequantize_matrix(&layer.k_proj);
        let v_proj_deq = dequantize_matrix(&layer.v_proj);
        let o_proj_deq = dequantize_matrix(&layer.o_proj);
        let gate_proj_deq = dequantize_matrix(&layer.gate_proj);
        let up_proj_deq = dequantize_matrix(&layer.up_proj);
        let down_proj_deq = dequantize_matrix(&layer.down_proj);

        let mut new_hidden = Vec::with_capacity(seq_len);

        for (pos, h) in hidden.iter().enumerate() {
            // Input layernorm
            let mut normed = h.clone();
            rms_norm(&mut normed, &layer.input_layernorm.weight, GLIMMER_RMS_NORM_EPS);

            // Q, K, V projections
            let qg = project_dequantized(&normed, &q_proj_deq); // [q_dim + gate_dim]
            let k = project_dequantized(&normed, &k_proj_deq);
            let v = project_dequantized(&normed, &v_proj_deq);

            // Split q and gate
            let q_dim = num_q_heads * head_dim;
            let q_raw = &qg[..q_dim];
            let gate_raw = &qg[q_dim..];

            // Reshape Q/K/V to heads
            let mut q_heads = vec![[0.0f32; 128]; num_q_heads];
            let mut k_heads = vec![[0.0f32; 128]; num_kv_heads];
            let mut v_heads = vec![[0.0f32; 128]; num_kv_heads];
            let mut gate_heads = vec![[0.0f32; 128]; num_q_heads];

            for q_h in 0..num_q_heads {
                let start = q_h * head_dim;
                q_heads[q_h].copy_from_slice(&q_raw[start..start + head_dim]);
                gate_heads[q_h].copy_from_slice(&gate_raw[start..start + head_dim]);
            }

            for kv_h in 0..num_kv_heads {
                let start = kv_h * head_dim;
                k_heads[kv_h].copy_from_slice(&k[start..start + head_dim]);
                v_heads[kv_h].copy_from_slice(&v[start..start + head_dim]);
            }

            // Q/K normalization
            for q_h in 0..num_q_heads {
                rms_norm(&mut q_heads[q_h], &layer.q_norm.weight, GLIMMER_RMS_NORM_EPS);
            }
            for kv_h in 0..num_kv_heads {
                rms_norm(&mut k_heads[kv_h], &layer.k_norm.weight, GLIMMER_RMS_NORM_EPS);
            }

            // Apply position transform (RoPE/NoPE)
            crate::apply_position_transform_layer(
                &mut q_heads,
                &mut k_heads,
                pos,
                layer_config,
                rope_freqs,
            );

            // Update KV cache with new K/V for this position
            for kv_h in 0..num_kv_heads {
                kv_cache.k_cache[kv_h].extend_from_slice(&k_heads[kv_h]);
                kv_cache.v_cache[kv_h].extend_from_slice(&v_heads[kv_h]);
            }

            // Attention computation using KV cache
            let mut attn_output = vec![0.0f32; hidden_size];
            let _current_seq_len = kv_cache.k_cache[0].len() / head_dim;
            let scale = 1.0f32 / (head_dim as f32).sqrt();

            for q_h in 0..num_q_heads {
                let kv_h = q_h / heads_per_kv;
                let k_cache = &kv_cache.k_cache[kv_h];
                let v_cache = &kv_cache.v_cache[kv_h];
                let q_vec = &q_heads[q_h];

                let attn_result = crate::exact_attention::attend_one_head_f32(
                    q_vec, k_cache, v_cache, head_dim, 0, scale,
                )?;

                // Apply gate
                for d in 0..head_dim {
                    let gate_val = 1.0f32 / (1.0f32 + (-gate_heads[q_h][d]).exp());
                    attn_output[q_h * head_dim + d] = attn_result[d] * gate_val;
                }
            }

            // O projection
            let o_out = project_dequantized(&attn_output, &o_proj_deq);

            // Residual connection
            let mut post_attn = vec![0.0f32; hidden_size];
            for i in 0..hidden_size {
                post_attn[i] = h[i] + o_out[i];
            }

            // Post-attention norm
            let mut mlp_in = post_attn.clone();
            rms_norm(&mut mlp_in, &layer.post_attn_norm.weight, GLIMMER_RMS_NORM_EPS);

            // MLP: gate_proj, up_proj, down_proj with SiLU
            let gate_act = project_dequantized(&mlp_in, &gate_proj_deq);
            let up_act = project_dequantized(&mlp_in, &up_proj_deq);

            let mut swiglu = vec![0.0f32; intermediate_size];
            for i in 0..intermediate_size {
                swiglu[i] = silu(gate_act[i]) * up_act[i];
            }

            let mut down_out = project_dequantized(&swiglu, &down_proj_deq);

            // Post-FFN norm
            rms_norm(&mut down_out, &layer.post_ffn_norm.weight, GLIMMER_RMS_NORM_EPS);

            // Residual connection
            let mut layer_out = vec![0.0f32; hidden_size];
            for i in 0..hidden_size {
                layer_out[i] = post_attn[i] + down_out[i];
            }

            new_hidden.push(layer_out);
        }

        Ok(new_hidden)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_model_constants() {
        assert_eq!(GLIMMER_HIDDEN_SIZE, 6656);
        assert_eq!(GLIMMER_NUM_LAYERS, 52);
        assert_eq!(GLIMMER_NUM_Q_HEADS, 32);
        assert_eq!(GLIMMER_NUM_KV_HEADS, 2);
        assert_eq!(GLIMMER_HEAD_DIM, 128);
        assert_eq!(GLIMMER_VOCAB_SIZE, 202048);
    }

    #[test]
    fn test_dequantize_q4() {
        // For Q4: out_dim=16, in_dim=64, group_size=32
        // groups_per_row = 64/32 = 2
        // words_per_row = (64 * 4) / 32 = 8
        // weight bytes per row = 8 * 4 = 32
        // total weight bytes = 16 * 32 = 512
        let matrix = AffineQuantizedMatrix {
            weight: vec![0xFF; 512], // 4-bit packed: 16 rows * 32 bytes per row
            scales: vec![1.0; 32],   // 16 rows * 2 groups per row
            biases: vec![0.0; 32],
            out_dim: 16,
            in_dim: 64,
            bits: 4,
            group_size: 32,
        };
        let dequantized = dequantize_matrix(&matrix);
        assert_eq!(dequantized.out_dim, 16);
        assert_eq!(dequantized.in_dim, 64);
        assert_eq!(dequantized.weight.len(), 16 * 64);
    }

    #[test]
    fn test_rms_norm() {
        let mut x = vec![1.0, 2.0, 3.0, 4.0];
        let weight = vec![0.1, 0.2, 0.3, 0.4];
        rms_norm(&mut x, &weight, 1e-5);
        // Just verify it runs without panic
        assert_eq!(x.len(), 4);
    }

    #[test]
    fn test_project_dequantized() {
        let matrix = DequantizedMatrix {
            weight: vec![1.0; 4 * 8],
            out_dim: 4,
            in_dim: 8,
        };
        let x = vec![1.0; 8];
        let out = project_dequantized(&x, &matrix);
        assert_eq!(out.len(), 4);
        for v in out {
            assert_eq!(v, 8.0); // 1*1*8 = 8
        }
    }
}

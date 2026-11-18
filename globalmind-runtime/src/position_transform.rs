//! Position transform implementations for Glimmer: RoPE on sliding layers, NoPE on full layers.
//!
//! This module implements the trained position transforms used by Glimmer:
//! - RoPE (Rotary Position Embedding) for sliding attention layers
//! - NoPE (No Position Embedding) for full attention layers
//!
//! The transform is applied post-Q/K projection and normalization, before attention.

use crate::{Digest};
use sha2::{Digest as ShaDigest, Sha256};

/// Glimmer layer type determining position transform
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerAttentionType {
    Sliding, // RoPE
    Full,    // NoPE
}

impl LayerAttentionType {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "sliding_attention" => Ok(Self::Sliding),
            "full_attention" => Ok(Self::Full),
            _ => Err(format!("unknown layer type: {}", s)),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sliding => "sliding_attention",
            Self::Full => "full_attention",
        }
    }
}

/// RoPE configuration for Glimmer
#[derive(Debug, Clone)]
pub struct RopeConfig {
    pub head_dim: usize,
    pub theta: f64,
    pub partial_rotary_factor: f32,
    pub sliding_window: usize,
}

impl Default for RopeConfig {
    fn default() -> Self {
        Self {
            head_dim: 128,
            theta: 500_000.0,
            partial_rotary_factor: 0.25,
            sliding_window: 2048,
        }
    }
}

/// Precomputed RoPE frequencies for efficient application
#[derive(Debug, Clone)]
pub struct RopeFrequencies {
    pub cos: Vec<f32>,
    pub sin: Vec<f32>,
    pub rotary_dim: usize,
}

impl RopeFrequencies {
    /// Precompute RoPE frequencies up to max_seq_len
    pub fn new(config: &RopeConfig, max_seq_len: usize) -> Self {
        let rotary_dim = (config.head_dim as f32 * config.partial_rotary_factor) as usize;
        let half_rotary = rotary_dim / 2;
        let mut cos = vec![0.0f32; max_seq_len * half_rotary];
        let mut sin = vec![0.0f32; max_seq_len * half_rotary];

        for pos in 0..max_seq_len {
            for i in 0..half_rotary {
                let freq = config.theta.powf(-((2 * i) as f64) / config.head_dim as f64);
                let angle = pos as f64 * freq;
                cos[pos * half_rotary + i] = angle.cos() as f32;
                sin[pos * half_rotary + i] = angle.sin() as f32;
            }
        }

        Self {
            cos,
            sin,
            rotary_dim,
        }
    }

    /// Apply RoPE in-place to query or key vector
    /// Only the first `rotary_dim` dimensions are rotated
    pub fn apply_in_place(&self, vec: &mut [f32], position: usize) {
        let half_rotary = self.rotary_dim / 2;
        if position >= self.cos.len() / half_rotary {
            return; // Position out of precomputed range
        }
        let base = position * half_rotary;
        for i in 0..half_rotary {
            let x0 = vec[i];
            let x1 = vec[i + half_rotary];
            let c = self.cos[base + i];
            let s = self.sin[base + i];
            vec[i] = x0 * c - x1 * s;
            vec[i + half_rotary] = x1 * c + x0 * s;
        }
    }

    /// Apply RoPE with sliding window - only attend to recent tokens
    pub fn apply_with_window(&self, vec: &mut [f32], position: usize, window_start: usize) {
        // For sliding attention, the effective position is relative to window
        let effective_pos = position.saturating_sub(window_start);
        self.apply_in_place(vec, effective_pos);
    }
}

/// NoPE (No Position Embedding) - identity transform for full attention layers
#[derive(Debug, Clone, Default)]
pub struct NoPeTransform;

impl NoPeTransform {
    pub fn apply_in_place(&self, _vec: &mut [f32], _position: usize) {
        // No position transform for full attention
    }
}

/// Unified position transform that selects RoPE or NoPE per layer
#[derive(Debug, Clone)]
pub struct PositionTransform {
    pub rope: RopeFrequencies,
    pub nope: NoPeTransform,
    pub config: RopeConfig,
}

impl PositionTransform {
    pub fn new(config: RopeConfig, max_seq_len: usize) -> Self {
        let rope = RopeFrequencies::new(&config, max_seq_len);
        Self {
            rope,
            nope: NoPeTransform,
            config,
        }
    }

    /// Apply the appropriate position transform for the layer type
    pub fn apply_in_place(
        &self,
        vec: &mut [f32],
        position: usize,
        layer_type: LayerAttentionType,
    ) {
        match layer_type {
            LayerAttentionType::Sliding => {
                self.rope.apply_in_place(vec, position);
            }
            LayerAttentionType::Full => {
                self.nope.apply_in_place(vec, position);
            }
        }
    }

    /// Apply with sliding window for sliding layers
    pub fn apply_with_window(
        &self,
        vec: &mut [f32],
        position: usize,
        layer_type: LayerAttentionType,
        window_start: usize,
    ) {
        match layer_type {
            LayerAttentionType::Sliding => {
                self.rope.apply_with_window(vec, position, window_start);
            }
            LayerAttentionType::Full => {
                self.nope.apply_in_place(vec, position);
            }
        }
    }

    /// Get the digest of the position transform configuration
    pub fn config_digest(&self) -> Digest {
        let mut h = Sha256::new();
        h.update(b"glimmer-position-transform-v1\0");
        h.update(self.config.head_dim.to_le_bytes());
        h.update(self.config.theta.to_le_bytes());
        h.update(self.config.partial_rotary_factor.to_le_bytes());
        h.update(self.config.sliding_window.to_le_bytes());
        Digest::parse(&format!("{:x}", h.finalize())).expect("SHA-256 is valid")
    }
}

/// Glimmer layer configuration including attention type
#[derive(Debug, Clone)]
pub struct GlimmerLayerConfig {
    pub layer_idx: u32,
    pub attention_type: LayerAttentionType,
    pub head_dim: usize,
    pub num_q_heads: usize,
    pub num_kv_heads: usize,
    pub hidden_size: usize,
    pub sliding_window: usize,
    pub rope_theta: f64,
    pub partial_rotary_factor: f32,
}

impl Default for GlimmerLayerConfig {
    fn default() -> Self {
        Self {
            layer_idx: 0,
            attention_type: LayerAttentionType::Sliding,
            head_dim: 128,
            num_q_heads: 32,
            num_kv_heads: 2,
            hidden_size: 6656,
            sliding_window: 2048,
            rope_theta: 500_000.0,
            partial_rotary_factor: 0.25,
        }
    }
}

/// Build all 52 Glimmer layer configurations from config.json layer_types
pub fn build_glimmer_layer_configs(
    layer_types: &[&str],
) -> Result<Vec<GlimmerLayerConfig>, String> {
    if layer_types.len() != 52 {
        return Err("Glimmer must have exactly 52 layers".into());
    }

    let mut configs = Vec::with_capacity(52);
    for (i, layer_type) in layer_types.iter().enumerate() {
        let attention_type = LayerAttentionType::from_str(layer_type)?;
        configs.push(GlimmerLayerConfig {
            layer_idx: i as u32,
            attention_type,
            head_dim: 128,
            num_q_heads: 32,
            num_kv_heads: 2,
            hidden_size: 6656,
            sliding_window: 2048,
            rope_theta: 500_000.0,
            partial_rotary_factor: 0.25,
        });
    }
    Ok(configs)
}

/// Apply position transform to all Q/K heads for a layer
pub fn apply_position_transform_layer(
    q_heads: &mut [[f32; 128]],
    k_heads: &mut [[f32; 128]],
    position: usize,
    layer_config: &GlimmerLayerConfig,
    rope_freqs: &RopeFrequencies,
) {
    let heads_per_kv = layer_config.num_q_heads / layer_config.num_kv_heads;

    for q_h in 0..layer_config.num_q_heads {
        let kv_h = q_h / heads_per_kv;
        if layer_config.attention_type == LayerAttentionType::Sliding {
            rope_freqs.apply_in_place(&mut q_heads[q_h], position);
            rope_freqs.apply_in_place(&mut k_heads[kv_h], position);
        }
        // Full attention: no position transform (NoPE)
    }
}

/// Apply position transform with sliding window
pub fn apply_position_transform_layer_windowed(
    q_heads: &mut [[f32; 128]],
    k_heads: &mut [[f32; 128]],
    position: usize,
    window_start: usize,
    layer_config: &GlimmerLayerConfig,
    rope_freqs: &RopeFrequencies,
) {
    let heads_per_kv = layer_config.num_q_heads / layer_config.num_kv_heads;

    for q_h in 0..layer_config.num_q_heads {
        let kv_h = q_h / heads_per_kv;
        if layer_config.attention_type == LayerAttentionType::Sliding {
            rope_freqs.apply_with_window(&mut q_heads[q_h], position, window_start);
            rope_freqs.apply_with_window(&mut k_heads[kv_h], position, window_start);
        }
        // Full attention: no position transform
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rope_config_default() {
        let config = RopeConfig::default();
        assert_eq!(config.head_dim, 128);
        assert_eq!(config.theta, 500_000.0);
        assert_eq!(config.partial_rotary_factor, 0.25);
        assert_eq!(config.sliding_window, 2048);
    }

    #[test]
    fn rope_frequencies_precompute() {
        let config = RopeConfig::default();
        let freqs = RopeFrequencies::new(&config, 100);
        assert_eq!(freqs.rotary_dim, 32); // 128 * 0.25 = 32
        assert_eq!(freqs.cos.len(), 100 * 16); // max_seq_len * half_rotary
        assert_eq!(freqs.sin.len(), 100 * 16);
    }

    #[test]
    fn rope_apply_in_place() {
        let config = RopeConfig::default();
        let freqs = RopeFrequencies::new(&config, 10);
        let mut vec = vec![1.0f32; 128];
        freqs.apply_in_place(&mut vec, 0);
        // At position 0, cos=1, sin=0, so vector unchanged
        assert_eq!(vec[0], 1.0);
        assert_eq!(vec[16], 1.0);
    }

    #[test]
    fn rope_apply_at_position() {
        let config = RopeConfig::default();
        let freqs = RopeFrequencies::new(&config, 10);
        let mut vec = vec![1.0f32; 128];
        freqs.apply_in_place(&mut vec, 5);
        // At position 5, vector should be rotated
        assert_ne!(vec[0], 1.0);
        assert_ne!(vec[16], 1.0);
    }

    #[test]
    fn nope_no_transform() {
        let nope = NoPeTransform;
        let mut vec = vec![1.0f32; 128];
        nope.apply_in_place(&mut vec, 100);
        assert_eq!(vec[0], 1.0);
        assert_eq!(vec[64], 1.0);
    }

    #[test]
    fn position_transform_selects_correctly() {
        let config = RopeConfig::default();
        let transform = PositionTransform::new(config, 10);
        let mut vec_sliding = vec![1.0f32; 128];
        let mut vec_full = vec![1.0f32; 128];

        transform.apply_in_place(&mut vec_sliding, 5, LayerAttentionType::Sliding);
        transform.apply_in_place(&mut vec_full, 5, LayerAttentionType::Full);

        // Sliding should be rotated
        assert_ne!(vec_sliding[0], 1.0);
        // Full should be unchanged
        assert_eq!(vec_full[0], 1.0);
    }

    #[test]
    fn layer_config_from_str() {
        assert_eq!(
            LayerAttentionType::from_str("sliding_attention"),
            Ok(LayerAttentionType::Sliding)
        );
        assert_eq!(
            LayerAttentionType::from_str("full_attention"),
            Ok(LayerAttentionType::Full)
        );
        assert!(LayerAttentionType::from_str("unknown").is_err());
    }

    #[test]
    fn build_glimmer_layer_configs_valid() {
        let layer_types: Vec<&str> = (0..52)
            .map(|i| {
                if (i + 1) % 4 == 0 {
                    "full_attention"
                } else {
                    "sliding_attention"
                }
            })
            .collect();
        let configs = build_glimmer_layer_configs(&layer_types).unwrap();
        assert_eq!(configs.len(), 52);
        let sliding = configs.iter().filter(|c| c.attention_type == LayerAttentionType::Sliding).count();
        let full = configs.iter().filter(|c| c.attention_type == LayerAttentionType::Full).count();
        assert_eq!(sliding, 39);
        assert_eq!(full, 13);
    }

    #[test]
    fn build_glimmer_layer_configs_invalid_length() {
        let layer_types = vec!["sliding_attention"; 51];
        assert!(build_glimmer_layer_configs(&layer_types).is_err());
    }

    #[test]
    fn config_digest_deterministic() {
        let config = RopeConfig::default();
        let transform1 = PositionTransform::new(config.clone(), 100);
        let transform2 = PositionTransform::new(config, 100);
        assert_eq!(transform1.config_digest(), transform2.config_digest());
    }
}

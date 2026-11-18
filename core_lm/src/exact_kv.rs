//! Exact transformer K/V backend for the native Glimmer attention seam.
//! This is model-bound tensor state, separate from teacher-neutral mind atoms.

use crate::attention_backend::{AttentionBackend, AttentionInput};

#[derive(Clone, Debug)]
struct LayerCache {
    keys: Vec<Vec<f32>>,
    values: Vec<Vec<f32>>,
    tokens: usize,
}

/// Owned cache for exact continuation. Shape is fixed at construction.
#[derive(Clone, Debug)]
pub struct ExactKvCache {
    layers: Vec<LayerCache>,
    query_heads: usize,
    kv_heads: usize,
    head_dim: usize,
}

impl ExactKvCache {
    pub fn new(
        layers: usize,
        query_heads: usize,
        kv_heads: usize,
        head_dim: usize,
    ) -> Result<Self, String> {
        if layers == 0
            || kv_heads == 0
            || query_heads == 0
            || head_dim == 0
            || !query_heads.is_multiple_of(kv_heads)
            || layers > 4096
            || head_dim > 4096
        {
            return Err("invalid exact K/V shape".into());
        }
        Ok(Self {
            layers: (0..layers)
                .map(|_| LayerCache {
                    keys: vec![Vec::new(); kv_heads],
                    values: vec![Vec::new(); kv_heads],
                    tokens: 0,
                })
                .collect(),
            query_heads,
            kv_heads,
            head_dim,
        })
    }

    pub fn token_count(&self, layer: usize) -> Option<usize> {
        self.layers.get(layer).map(|state| state.tokens)
    }

    pub fn head_page(&self, layer: usize, head: usize) -> Option<(&[f32], &[f32])> {
        let state = self.layers.get(layer)?;
        Some((state.keys.get(head)?, state.values.get(head)?))
    }

    /// Reconstruct only from a complete, layer-major/head-major tensor set.
    pub fn from_head_pages(
        layers: usize,
        query_heads: usize,
        kv_heads: usize,
        head_dim: usize,
        tokens: usize,
        pages: Vec<(Vec<f32>, Vec<f32>)>,
    ) -> Result<Self, String> {
        let mut cache = Self::new(layers, query_heads, kv_heads, head_dim)?;
        if tokens == 0 || tokens > 1_000_000 || layers.checked_mul(kv_heads) != Some(pages.len()) {
            return Err("incomplete exact K/V page set".into());
        }
        let expected = tokens
            .checked_mul(head_dim)
            .ok_or("exact K/V shape overflow")?;
        for (index, (keys, values)) in pages.into_iter().enumerate() {
            if keys.len() != expected
                || values.len() != expected
                || !keys.iter().chain(&values).all(|value| value.is_finite())
            {
                return Err("invalid exact K/V tensor page".into());
            }
            let layer = index / kv_heads;
            let head = index % kv_heads;
            cache.layers[layer].keys[head] = keys;
            cache.layers[layer].values[head] = values;
            cache.layers[layer].tokens = tokens;
        }
        cache.verify_complete_prefix(tokens)?;
        Ok(cache)
    }

    pub fn verify_complete_prefix(&self, expected_tokens: usize) -> Result<(), String> {
        if self.layers.iter().any(|layer| {
            layer.tokens != expected_tokens
                || (0..self.kv_heads).any(|head| {
                    layer.keys[head].len() != expected_tokens * self.head_dim
                        || layer.values[head].len() != expected_tokens * self.head_dim
                })
        }) {
            return Err("incomplete exact K/V prefix".into());
        }
        Ok(())
    }
}

impl AttentionBackend for ExactKvCache {
    fn attend(&mut self, input: AttentionInput<'_>) -> Result<Vec<f32>, String> {
        input.validate()?;
        if input.query_heads != self.query_heads
            || input.kv_heads != self.kv_heads
            || input.head_dim != self.head_dim
        {
            return Err("exact K/V shape changed".into());
        }
        let layer = self
            .layers
            .get_mut(input.layer)
            .ok_or("unknown exact K/V layer")?;
        if layer.tokens != input.position {
            return Err("exact K/V position gap or duplicate".into());
        }
        if input.position >= 1_000_000 {
            return Err("exact K/V token limit exceeded".into());
        }
        for head in 0..self.kv_heads {
            let start = head * self.head_dim;
            let end = start + self.head_dim;
            layer.keys[head].extend_from_slice(&input.key[start..end]);
            layer.values[head].extend_from_slice(&input.value[start..end]);
        }
        layer.tokens += 1;

        let mut output = vec![0.0_f32; self.query_heads * self.head_dim];
        let heads_per_kv = self.query_heads / self.kv_heads;
        for query_head in 0..self.query_heads {
            let kv_head = query_head / heads_per_kv;
            let q_start = query_head * self.head_dim;
            let query = &input.query[q_start..q_start + self.head_dim];
            let keys = &layer.keys[kv_head];
            let values = &layer.values[kv_head];
            let eligible = layer.tokens - input.window_start;
            let mut scores = Vec::with_capacity(eligible);
            let mut max_score = f32::NEG_INFINITY;
            for token in input.window_start..layer.tokens {
                let start = token * self.head_dim;
                let key = &keys[start..start + self.head_dim];
                let mut dot = 0.0_f32;
                for i in 0..self.head_dim {
                    dot += query[i] * key[i];
                }
                let score = (dot as f64 * input.dot_scale) as f32;
                if !score.is_finite() {
                    return Err("exact K/V score overflow".into());
                }
                max_score = max_score.max(score);
                scores.push(score);
            }
            let mut denom = 0.0_f32;
            for score in &mut scores {
                *score = (*score - max_score).exp();
                denom += *score;
            }
            if !denom.is_finite() || denom <= 0.0 {
                return Err("exact K/V softmax failure".into());
            }
            for (index, score) in scores.iter().enumerate() {
                let start = (input.window_start + index) * self.head_dim;
                for i in 0..self.head_dim {
                    output[q_start + i] += (*score / denom) * values[start + i];
                }
            }
        }
        if output.iter().any(|value| !value.is_finite()) {
            return Err("exact K/V output overflow".into());
        }
        Ok(output)
    }
}

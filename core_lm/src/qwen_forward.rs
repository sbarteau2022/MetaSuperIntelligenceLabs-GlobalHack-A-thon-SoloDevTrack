//! Qwen3.5-MoE forward pass with geometric memory.
//!
//! Each full_attention layer owns a GeometricMemoryMap.
//! Keys are projected to LorentzVector; values stored as payload.
//! There are NO k_cache / v_cache tensors in this module.

use crate::cache::geometric::GeometricMemoryMap;
use crate::model_config::{LayerType, ModelConfig};
use crate::ops::{rmsnorm, silu, softmax, top_k_indices};
use crate::qwen_model::{ModelError, WeightManager};
use crate::{ForwardCompliancePermit, ImmuneForwardGate};
use std::path::Path;

pub struct LinearAttnLayerState {
    /// Last (kernel-1) qkv inputs per conv channel, oldest first: [conv_dim][kernel-1].
    pub conv: Vec<f32>,
    /// Gated DeltaNet recurrent state per value head: [value_heads][Dv][Dk], fp32.
    pub s: Vec<f32>,
}

impl LinearAttnLayerState {
    pub fn new(conv_dim: usize, kernel: usize, value_heads: usize, dv: usize, dk: usize) -> Self {
        Self {
            conv: vec![0.0; conv_dim * (kernel - 1)],
            s: vec![0.0; value_heads * dv * dk],
        }
    }
}

/// Fresh per-layer linear-attention state, sized from the model config.
fn new_linear_state(config: &ModelConfig) -> LinearAttnLayerState {
    let key_dim = config.linear_num_key_heads * config.linear_key_head_dim;
    let value_dim = config.linear_num_value_heads * config.linear_value_head_dim;
    LinearAttnLayerState::new(
        2 * key_dim + value_dim,
        config.linear_conv_kernel_dim,
        config.linear_num_value_heads,
        config.linear_value_head_dim,
        config.linear_key_head_dim,
    )
}

pub struct QwenModel {
    pub config: ModelConfig,
    pub weights: WeightManager,
    pub geo_maps: Vec<GeometricMemoryMap>,
    pub linear_states: Vec<LinearAttnLayerState>,
    pub immune_gate: ImmuneForwardGate,
    /// When `Some`, each forward call records the L2 norm of the residual stream after the
    /// embedding and after every layer. Used by the layer-oracle gate; `None` costs nothing.
    pub layer_trace: Option<Vec<f32>>,
    /// Exact per-layer KV cache for full-attention layers (index = layer).
    pub kv: Vec<KvLayerCache>,
    /// Diagnostic probe for full attention: when Some, records [qg, v, gated] per call.
    pub attn_probe: Option<Vec<Vec<f32>>>,
}

impl QwenModel {
    pub fn load(model_dir: &Path) -> Result<Self, ModelError> {
        let config = ModelConfig::from_json_file(&model_dir.join("config.json"))?;
        let weights =
            WeightManager::load(model_dir, &model_dir.join("model.safetensors.index.json"))?;
        let num_full = config.full_attention_layers().len();
        let nkv = config.num_key_value_heads.max(1);
        let mut geo_maps = Vec::with_capacity(num_full * nkv);
        for _ in 0..(num_full * nkv) {
            geo_maps.push(GeometricMemoryMap::new(1_000_000));
        }
        let mut linear_states = Vec::with_capacity(config.num_hidden_layers);
        for _ in 0..config.num_hidden_layers {
            linear_states.push(new_linear_state(&config));
        }
        let kv: Vec<KvLayerCache> = (0..config.num_hidden_layers).map(|_| KvLayerCache::default()).collect();
        let fp = (config.hidden_size as u64) << 32 | (config.num_hidden_layers as u64);
        Ok(Self {
            config,
            weights,
            geo_maps,
            linear_states,
            immune_gate: ImmuneForwardGate::new(fp),
            layer_trace: None,
            kv,
            attn_probe: None,
        })
    }

    fn p(&self, l: usize) -> String {
        format!("language_model.model.layers.{}", l)
    }

    fn embed(&mut self, token_id: u32) -> Result<Vec<f32>, ModelError> {
        let h = self.config.hidden_size;
        self.weights.get_weight_row(
            "language_model.model.embed_tokens.weight",
            token_id as usize,
            h,
        )
    }

    /// Exact full softmax attention, matching the reference Qwen3NextAttention: q_proj emits
    /// [query | gate] per head; per-head RMSNorm on q and k; non-traditional (half-split) RoPE over
    /// the first head_dim*partial_rotary_factor dims; exact causal softmax over an exact KV cache with
    /// GQA head repetition; then output * sigmoid(gate) through o_proj.
    fn full_attention(&mut self, layer: usize, x: &[f32], pos: usize) -> Result<Vec<f32>, ModelError> {
        let h = self.config.hidden_size;
        let nh = self.config.num_attention_heads;
        let nkv = self.config.num_key_value_heads.max(1);
        let hd = self.config.head_dim;
        let eps = self.config.rms_norm_eps;
        let theta = self.config.rope_theta;
        let rot = (hd as f32 * self.config.partial_rotary_factor) as usize;
        let p = self.p(layer);

        let ln_w = self.weights.get_f32(&format!("{}.input_layernorm.weight", p))?;
        let xn = rmsnorm(x, &ln_w, eps)?;
        let qg = self.weights.matvec_weight(&format!("{}.self_attn.q_proj.weight", p), nh * hd * 2, h, &xn)?;
        let mut k = self.weights.matvec_weight(&format!("{}.self_attn.k_proj.weight", p), nkv * hd, h, &xn)?;
        let v = self.weights.matvec_weight(&format!("{}.self_attn.v_proj.weight", p), nkv * hd, h, &xn)?;
        if let Some(pr) = self.attn_probe.as_mut() {
            pr.clear();
            pr.push(qg.clone());
            pr.push(v.clone());
        }
        let q_norm_w = self.weights.get_f32(&format!("{}.self_attn.q_norm.weight", p))?;
        let k_norm_w = self.weights.get_f32(&format!("{}.self_attn.k_norm.weight", p))?;

        // q_proj output is laid out per head as [query hd | gate hd].
        let mut q = vec![0f32; nh * hd];
        let mut gate = vec![0f32; nh * hd];
        for hh in 0..nh {
            let base = hh * 2 * hd;
            q[hh * hd..(hh + 1) * hd].copy_from_slice(&qg[base..base + hd]);
            gate[hh * hd..(hh + 1) * hd].copy_from_slice(&qg[base + hd..base + 2 * hd]);
        }
        for hh in 0..nh {
            let normed = rmsnorm(&q[hh * hd..(hh + 1) * hd], &q_norm_w, eps)?;
            q[hh * hd..(hh + 1) * hd].copy_from_slice(&normed);
        }
        for kh in 0..nkv {
            let normed = rmsnorm(&k[kh * hd..(kh + 1) * hd], &k_norm_w, eps)?;
            k[kh * hd..(kh + 1) * hd].copy_from_slice(&normed);
        }
        rope_half(&mut q, hd, rot, pos, theta);
        rope_half(&mut k, hd, rot, pos, theta);

        // Exact KV cache: positions must arrive contiguously from 0 within one sequence.
        let cache = &mut self.kv[layer];
        if cache.k.len() != pos * nkv * hd {
            return Err(ModelError::MissingWeight(format!("non-contiguous position {pos} for layer {layer}")));
        }
        cache.k.extend_from_slice(&k);
        cache.v.extend_from_slice(&v);
        let t_len = pos + 1;

        let repeat = (nh / nkv).max(1);
        let scale = 1.0 / (hd as f32).sqrt();
        let mut out = vec![0f32; nh * hd];
        for hh in 0..nh {
            let kvh = (hh / repeat).min(nkv - 1);
            let qh = &q[hh * hd..(hh + 1) * hd];
            let mut scores = Vec::with_capacity(t_len);
            for ti in 0..t_len {
                let kt = &cache.k[(ti * nkv + kvh) * hd..(ti * nkv + kvh + 1) * hd];
                scores.push(qh.iter().zip(kt).map(|(a, b)| a * b).sum::<f32>() * scale);
            }
            let m = scores.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
            let mut denom = 0f32;
            for s in scores.iter_mut() {
                *s = (*s - m).exp();
                denom += *s;
            }
            let oh = &mut out[hh * hd..(hh + 1) * hd];
            for ti in 0..t_len {
                let w = scores[ti] / denom;
                let vt = &cache.v[(ti * nkv + kvh) * hd..(ti * nkv + kvh + 1) * hd];
                for d in 0..hd {
                    oh[d] += w * vt[d];
                }
            }
        }
        for i in 0..nh * hd {
            out[i] *= 1.0 / (1.0 + (-gate[i]).exp());
        }
        if let Some(pr) = self.attn_probe.as_mut() {
            pr.push(out.clone());
        }
        let o = self.weights.matvec_weight(&format!("{}.self_attn.o_proj.weight", p), h, nh * hd, &out)?;
        Ok(x.iter().zip(o.iter()).map(|(a, b)| a + b).collect())
    }

    /// One decoder layer: attention (full or linear) with residual, then the MoE block with residual.
    /// Public so layer-local oracles can drive a single layer from a reference hidden state.
    pub fn run_layer(&mut self, layer: usize, x: &[f32], pos: usize) -> Result<Vec<f32>, ModelError> {
        let attn = self.attention_block(layer, x, pos)?;
        self.moe_block(layer, &attn)
    }

    /// Attention block with its residual (full or linear). Public for the split oracle.
    pub fn attention_block(&mut self, layer: usize, x: &[f32], pos: usize) -> Result<Vec<f32>, ModelError> {
        if matches!(self.config.layer_types[layer], LayerType::FullAttention) {
            self.full_attention(layer, x, pos)
        } else {
            self.linear_attention(layer, x)
        }
    }

    /// MoE block with its residual. Public for the split oracle.
    pub fn moe_block(&mut self, layer: usize, x: &[f32]) -> Result<Vec<f32>, ModelError> {
        self.moe_mlp(layer, x)
    }

    fn moe_mlp(&mut self, layer: usize, x: &[f32]) -> Result<Vec<f32>, ModelError> {
        let h = self.config.hidden_size;
        let inter = self.config.moe_intermediate_size;
        let shared_inter = self.config.shared_expert_intermediate_size;
        let num_experts = self.config.num_experts;
        let top_k = self.config.num_experts_per_tok;
        let eps = self.config.rms_norm_eps;
        let p = self.p(layer);

        let xn = rmsnorm(
            x,
            &self
                .weights
                .get_f32(&format!("{}.post_attention_layernorm.weight", p))?,
            eps,
        )?;

        let gate_logits = self.weights.matvec_weight(
            &format!("{}.mlp.gate.weight", p),
            num_experts,
            h,
            &xn,
        )?;
        let gate_probs = softmax(&gate_logits)?;
        let top_experts = top_k_indices(&gate_probs, top_k);

        // Only the top-k routed experts are read, as quantized row slices. The full
        // [num_experts, rows, cols] tensors are never dequantized per token.
        let gate_name = format!("{}.mlp.switch_mlp.gate_proj.weight", p);
        let up_name = format!("{}.mlp.switch_mlp.up_proj.weight", p);
        let down_name = format!("{}.mlp.switch_mlp.down_proj.weight", p);

        let mut out = vec![0.0f32; h];

        // norm_topk_prob (Qwen3.5 default True): routed weights are renormalised over the top-k.
        let top_sum: f32 = top_experts.iter().map(|&e| gate_probs[e]).sum();
        for &ei in &top_experts {
            let prob = gate_probs[ei] / top_sum;
            let gate = self.weights.expert_matvec(&gate_name, num_experts, inter, h, ei, &xn)?;
            let up = self.weights.expert_matvec(&up_name, num_experts, inter, h, ei, &xn)?;
            let gated = silu(&gate);
            let hidden: Vec<f32> = gated.iter().zip(up.iter()).map(|(g, u)| g * u).collect();
            let down = self.weights.expert_matvec(&down_name, num_experts, h, inter, ei, &hidden)?;
            for i in 0..h {
                out[i] += prob * down[i];
            }
        }

        let sp = format!("{}.mlp.shared_expert", p);
        let s_gate = self.weights.matvec_weight(
            &format!("{}.gate_proj.weight", sp),
            shared_inter,
            h,
            &xn,
        )?;
        let s_up = self.weights.matvec_weight(
            &format!("{}.up_proj.weight", sp),
            shared_inter,
            h,
            &xn,
        )?;
        let s_gated = silu(&s_gate);
        let s_hidden: Vec<f32> = s_gated
            .iter()
            .zip(s_up.iter())
            .map(|(g, u)| g * u)
            .collect();
        let s_down = self.weights.matvec_weight(
            &format!("{}.down_proj.weight", sp),
            h,
            shared_inter,
            &s_hidden,
        )?;
        // Shared expert is gated per token by sigmoid(shared_expert_gate(x)) (8-bit gate).
        let sg = self.weights.matvec_weight(
            &format!("{}.mlp.shared_expert_gate.weight", p),
            1,
            h,
            &xn,
        )?[0];
        let shared_gate = 1.0 / (1.0 + (-sg).exp());
        for i in 0..h {
            out[i] += shared_gate * s_down[i];
        }

        Ok(x.iter().zip(out.iter()).map(|(a, b)| a + b).collect())
    }

    /// Gated DeltaNet linear attention, exact to the reference (mlx_lm qwen3_5 / gated_delta):
    /// depthwise causal conv(k=4) + SiLU, L2-normalised q/k with 1/Dk readout scale,
    /// beta = sigmoid(b), g = exp(-exp(A_log) * softplus(a + dt_bias)), per-value-head delta rule
    /// on an fp32 [Dv, Dk] state, RMSNorm(y) * SiLU(z) gate, then out_proj and the residual.
    fn linear_attention(&mut self, layer: usize, x: &[f32]) -> Result<Vec<f32>, ModelError> {
        let h = self.config.hidden_size;
        let eps = self.config.rms_norm_eps;
        let p = self.p(layer);
        let hk = self.config.linear_num_key_heads;
        let hv = self.config.linear_num_value_heads;
        let dk = self.config.linear_key_head_dim;
        let dv = self.config.linear_value_head_dim;
        let ksz = self.config.linear_conv_kernel_dim;
        let key_dim = hk * dk;
        let value_dim = hv * dv;
        let conv_dim = 2 * key_dim + value_dim;

        let ln_w = self.weights.get_f32(&format!("{}.input_layernorm.weight", p))?;
        let xn = rmsnorm(x, &ln_w, eps)?;
        let qkv = self.weights.matvec_weight(&format!("{}.linear_attn.in_proj_qkv.weight", p), conv_dim, h, &xn)?;
        let z = self.weights.matvec_weight(&format!("{}.linear_attn.in_proj_z.weight", p), value_dim, h, &xn)?;
        let b = self.weights.matvec_weight(&format!("{}.linear_attn.in_proj_b.weight", p), hv, h, &xn)?;
        let a = self.weights.matvec_weight(&format!("{}.linear_attn.in_proj_a.weight", p), hv, h, &xn)?;
        // Stored sanitized as [conv_dim, ksz, 1] (checked against the index header).
        let conv_w = self.weights.get_f32(&format!("{}.linear_attn.conv1d.weight", p))?;
        let a_log = self.weights.get_f32(&format!("{}.linear_attn.A_log", p))?;
        let dt_bias = self.weights.get_f32(&format!("{}.linear_attn.dt_bias", p))?;
        let norm_w = self.weights.get_f32(&format!("{}.linear_attn.norm.weight", p))?;
        if conv_w.len() != conv_dim * ksz || a_log.len() != hv || dt_bias.len() != hv || norm_w.len() != dv {
            return Err(ModelError::MissingWeight(format!("{p}: linear-attention shape mismatch")));
        }
        let state = self
            .linear_states
            .get_mut(layer)
            .ok_or_else(|| ModelError::MissingWeight(format!("linear state {layer}")))?;

        // Causal depthwise conv over [h0, h1, h2, current] per channel, then SiLU.
        let hist_len = ksz - 1;
        let mut conv = vec![0f32; conv_dim];
        for c in 0..conv_dim {
            let w = &conv_w[c * ksz..(c + 1) * ksz];
            let hist = &mut state.conv[c * hist_len..(c + 1) * hist_len];
            let mut acc = w[ksz - 1] * qkv[c];
            for j in 0..hist_len {
                acc += w[j] * hist[j];
            }
            conv[c] = silu_scalar(acc);
            hist.rotate_left(1);
            hist[hist_len - 1] = qkv[c];
        }

        // q/k heads: L2-normalise with the reference eps; q carries the 1/Dk readout scale.
        let inv_scale = (dk as f32).powf(-0.5);
        let rms_eps = eps * inv_scale * inv_scale;
        let mut q = vec![0f32; key_dim];
        let mut k = vec![0f32; key_dim];
        for kh in 0..hk {
            let qs = &conv[kh * dk..(kh + 1) * dk];
            let ks = &conv[key_dim + kh * dk..key_dim + (kh + 1) * dk];
            let (qn, kn) = (rms_inv(qs, rms_eps), rms_inv(ks, rms_eps));
            for i in 0..dk {
                q[kh * dk + i] = inv_scale * inv_scale * qs[i] * qn;
                k[kh * dk + i] = inv_scale * ks[i] * kn;
            }
        }
        let v_all = &conv[2 * key_dim..];

        // Delta-rule recurrence per value head, fp32 state S[dv][dk].
        let heads_per_k = hv / hk;
        let mut y = vec![0f32; value_dim];
        for vh in 0..hv {
            let kh = vh / heads_per_k;
            let qh = &q[kh * dk..(kh + 1) * dk];
            let kvec = &k[kh * dk..(kh + 1) * dk];
            let vvec = &v_all[vh * dv..(vh + 1) * dv];
            let beta = sigmoid(b[vh]);
            let g = (-(a_log[vh].exp()) * softplus(a[vh] + dt_bias[vh])).exp();
            let s = &mut state.s[vh * dv * dk..(vh + 1) * dv * dk];
            for i in 0..dv {
                let row = &mut s[i * dk..(i + 1) * dk];
                for sv in row.iter_mut() {
                    *sv *= g;
                }
                let kv_mem: f32 = row.iter().zip(kvec).map(|(sv, kk)| sv * kk).sum();
                let delta = (vvec[i] - kv_mem) * beta;
                for (sv, kk) in row.iter_mut().zip(kvec) {
                    *sv += kk * delta;
                }
                y[vh * dv + i] = row.iter().zip(qh).map(|(sv, qq)| sv * qq).sum();
            }
        }

        // Gated RMSNorm (per value head, shared weight), then out_proj and the residual.
        let mut out = vec![0f32; value_dim];
        for vh in 0..hv {
            let yh = &y[vh * dv..(vh + 1) * dv];
            let rn = rms_inv(yh, eps);
            for i in 0..dv {
                let normed = yh[i] * rn * norm_w[i];
                out[vh * dv + i] = silu_scalar(z[vh * dv + i]) * normed;
            }
        }
        let attn = self.weights.matvec_weight(&format!("{}.linear_attn.out_proj.weight", p), h, value_dim, &out)?;
        Ok(x.iter().zip(attn.iter()).map(|(a, b)| a + b).collect())
    }

    pub fn forward_with_permit(
        &mut self,
        token_id: u32,
        pos: usize,
        permit: &ForwardCompliancePermit,
    ) -> Result<Vec<f32>, ModelError> {
        self.immune_gate
            .verify_permit_and_loss(permit, None)
            .map_err(ModelError::MissingWeight)?;

        let mut x = self.embed(token_id)?;
        let nrm = |v: &[f32]| v.iter().map(|a| a * a).sum::<f32>().sqrt();
        if let Some(t) = self.layer_trace.as_mut() {
            t.clear();
            t.push(nrm(&x));
        }

        for layer in 0..self.config.num_hidden_layers {
            x = self.run_layer(layer, &x, pos)?;
            if let Some(t) = self.layer_trace.as_mut() {
                t.push(nrm(&x));
            }
        }

        let norm_w = self.weights.get_f32("language_model.model.norm.weight")?;
        let xn = rmsnorm(&x, &norm_w, self.config.rms_norm_eps)?;
        let logits = self.weights.matvec_weight(
            "language_model.lm_head.weight",
            self.config.vocab_size,
            self.config.hidden_size,
            &xn,
        )?;
        Ok(logits)
    }

    pub fn forward(&mut self, token_id: u32, pos: usize) -> Result<Vec<f32>, ModelError> {
        let fp = (self.config.hidden_size as u64) << 32 | (self.config.num_hidden_layers as u64);
        let permit = self
            .immune_gate
            .authorize_forward_pass(token_id, pos, fp)
            .map_err(ModelError::MissingWeight)?;
        self.forward_with_permit(token_id, pos, &permit)
    }

    pub fn reset_memory(&mut self) {
        for geo in self.geo_maps.iter_mut() {
            *geo = GeometricMemoryMap::new(1_000_000);
        }
        for st in self.linear_states.iter_mut() {
            *st = new_linear_state(&self.config);
        }
        for kv in self.kv.iter_mut() {
            kv.k.clear();
            kv.v.clear();
        }
    }
}

fn silu_scalar(x: f32) -> f32 {
    x / (1.0 + (-x).exp())
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

/// Stable softplus: identity above 20 (exp would overflow near f32 max, and softplus(x) == x to f32 precision there).
fn softplus(x: f32) -> f32 {
    if x > 20.0 { x } else { (1.0 + x.exp()).ln() }
}

/// 1 / sqrt(mean(x^2) + eps), the reference rms_norm denominator.
fn rms_inv(x: &[f32], eps: f32) -> f32 {
    let ms = x.iter().map(|v| v * v).sum::<f32>() / x.len() as f32;
    1.0 / (ms + eps).sqrt()
}

/// Exact KV cache for one full-attention layer: k and v are [position][kv_head][head_dim], appended per token.
#[derive(Default)]
pub struct KvLayerCache {
    pub k: Vec<f32>,
    pub v: Vec<f32>,
}

/// Non-traditional (half-split) RoPE over the first `rot` dims of each head, matching mlx rope(traditional=False):
/// pairs (i, i + rot/2) with frequency theta^(-2i/rot).
fn rope_half(x: &mut [f32], hd: usize, rot: usize, pos: usize, theta: f32) {
    let half = rot / 2;
    for base in (0..x.len()).step_by(hd) {
        for i in 0..half {
            let freq = theta.powf(-(2.0 * i as f32) / rot as f32);
            let (s, c) = (pos as f32 * freq).sin_cos();
            let x0 = x[base + i];
            let x1 = x[base + i + half];
            x[base + i] = x0 * c - x1 * s;
            x[base + i + half] = x0 * s + x1 * c;
        }
    }
}

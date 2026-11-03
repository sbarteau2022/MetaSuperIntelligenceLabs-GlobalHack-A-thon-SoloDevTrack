use globalmind_runtime::{
    Digest, ExactKvIdentity, GlobalMindStore, Manifest, RadialLaw, RecordId, StoreConfig,
    TensorDtype, TensorLayout, TensorPage, TensorPayload, TensorSession, TppeConfig, TppeState,
};
use serde_json::Value;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const MODEL_DIR: &str = "/Users/stewartbarteau/models/fused/Qwen3.6-MoE-Combined";
const SHARD1_FILE: &str =
    "/Users/stewartbarteau/models/fused/Qwen3.6-MoE-Combined/model-00001-of-00004.safetensors";
const SHARD4_FILE: &str =
    "/Users/stewartbarteau/models/fused/Qwen3.6-MoE-Combined/model-00004-of-00004.safetensors";
const SHARD1_HEADER_LEN: u64 = 63_300;
const SHARD4_HEADER_LEN: u64 = 37_985;

const MODEL_ID: &str = "stamsam/Qwen3.6-35B-A3B-Claude-4.7-Opus-Reasoning-Distilled-MLX-oQ4-MTP";
const MODEL_REVISION: &str = "local-frozen-2026-10-22";
const TPE_VERSION: &str = "tppe-layer3-candidate-v2";

const HIDDEN_SIZE: usize = 2048;
const NUM_LAYERS: u32 = 40;
const Q_HEADS: usize = 16;
const KV_HEADS: usize = 2;
const HEAD_DIM: usize = 256;
const VOCAB_SIZE: usize = 248_320;
const MOE_EXPERTS: usize = 256;
const MOE_TOP_K: usize = 8;
const MOE_INTERMEDIATE: usize = 512;
const QUANT_GROUP: usize = 64;
const RMS_NORM_EPS: f32 = 1e-6;
const TARGET_FULL_ATTN_LAYER: u32 = 3;

#[derive(Clone, Copy, Debug)]
struct TensorSliceSpec {
    start: u64,
    end: u64,
}

impl TensorSliceSpec {
    const fn new(start: u64, end: u64) -> Self {
        Self { start, end }
    }

    const fn byte_len(self) -> usize {
        (self.end - self.start) as usize
    }
}

#[derive(Clone, Copy, Debug)]
struct AffineQuantSpec {
    weight: TensorSliceSpec,
    scales: TensorSliceSpec,
    biases: TensorSliceSpec,
    out_dim: usize,
    in_dim: usize,
    bits: usize,
}

// Exact binary slice offsets in model-00001-of-00004.safetensors (after 8 + SHARD1_HEADER_LEN)
const EMBED_TOKENS_SPEC: AffineQuantSpec = AffineQuantSpec {
    weight: TensorSliceSpec::new(4_590_448_704, 4_844_728_384),
    scales: TensorSliceSpec::new(4_844_728_384, 4_860_620_864),
    biases: TensorSliceSpec::new(3_780_340_096, 3_796_232_576),
    out_dim: VOCAB_SIZE,
    in_dim: HIDDEN_SIZE,
    bits: 4,
};

const L3_INPUT_NORM_SPEC: TensorSliceSpec = TensorSliceSpec::new(2_275_097_664, 2_275_101_760);

const L3_Q_PROJ_SPEC: AffineQuantSpec = AffineQuantSpec {
    weight: TensorSliceSpec::new(3_363_184_960, 3_371_573_568),
    scales: TensorSliceSpec::new(4_414_260_288, 4_414_784_576),
    biases: TensorSliceSpec::new(3_008_045_312, 3_008_569_600),
    out_dim: 8192,
    in_dim: HIDDEN_SIZE,
    bits: 4,
};

const L3_Q_NORM_SPEC: TensorSliceSpec = TensorSliceSpec::new(4_419_537_984, 4_419_538_496);

const L3_K_PROJ_SPEC: AffineQuantSpec = AffineQuantSpec {
    weight: TensorSliceSpec::new(3_006_963_968, 3_007_488_256),
    scales: TensorSliceSpec::new(4_274_951_104, 4_274_983_872),
    biases: TensorSliceSpec::new(3_985_148_864, 3_985_181_632),
    out_dim: 512,
    in_dim: HIDDEN_SIZE,
    bits: 4,
};

const L3_K_NORM_SPEC: TensorSliceSpec = TensorSliceSpec::new(2_678_836_608, 2_678_837_120);

const L3_V_PROJ_SPEC: AffineQuantSpec = AffineQuantSpec {
    weight: TensorSliceSpec::new(979_633_728, 980_158_016),
    scales: TensorSliceSpec::new(1_598_257_536, 1_598_290_304),
    biases: TensorSliceSpec::new(3_964_078_656, 3_964_111_424),
    out_dim: 512,
    in_dim: HIDDEN_SIZE,
    bits: 4,
};

const L3_O_PROJ_SPEC: AffineQuantSpec = AffineQuantSpec {
    weight: TensorSliceSpec::new(3_980_921_792, 3_985_116_096),
    scales: TensorSliceSpec::new(4_413_998_144, 4_414_260_288),
    biases: TensorSliceSpec::new(3_006_699_776, 3_006_961_920),
    out_dim: HIDDEN_SIZE,
    in_dim: 4096,
    bits: 4,
};

const L3_POST_ATTN_NORM_SPEC: TensorSliceSpec = TensorSliceSpec::new(3_964_070_464, 3_964_074_560);

const L3_MLP_GATE_SPEC: AffineQuantSpec = AffineQuantSpec {
    weight: TensorSliceSpec::new(4_869_042_240, 4_869_566_528),
    scales: TensorSliceSpec::new(3_006_683_392, 3_006_699_776),
    biases: TensorSliceSpec::new(3_002_398_976, 3_002_415_360),
    out_dim: MOE_EXPERTS,
    in_dim: HIDDEN_SIZE,
    bits: 8,
};

const L3_SHARED_GATE_PROJ_SPEC: AffineQuantSpec = AffineQuantSpec {
    weight: TensorSliceSpec::new(2_841_145_536, 2_841_669_824),
    scales: TensorSliceSpec::new(3_006_650_624, 3_006_683_392),
    biases: TensorSliceSpec::new(1_951_781_504, 1_951_814_272),
    out_dim: MOE_INTERMEDIATE,
    in_dim: HIDDEN_SIZE,
    bits: 4,
};

const L3_SHARED_UP_PROJ_SPEC: AffineQuantSpec = AffineQuantSpec {
    weight: TensorSliceSpec::new(1_437_052_480, 1_437_576_768),
    scales: TensorSliceSpec::new(3_998_028_736, 3_998_061_504),
    biases: TensorSliceSpec::new(2_835_243_072, 2_835_275_840),
    out_dim: MOE_INTERMEDIATE,
    in_dim: HIDDEN_SIZE,
    bits: 4,
};

const L3_SHARED_DOWN_PROJ_SPEC: AffineQuantSpec = AffineQuantSpec {
    weight: TensorSliceSpec::new(5_332_855_040, 5_333_379_328),
    scales: TensorSliceSpec::new(2_840_031_296, 2_840_064_064),
    biases: TensorSliceSpec::new(2_835_804_224, 2_835_836_992),
    out_dim: HIDDEN_SIZE,
    in_dim: MOE_INTERMEDIATE,
    bits: 4,
};

const L3_SHARED_EXPERT_GATE_SPEC: AffineQuantSpec = AffineQuantSpec {
    weight: TensorSliceSpec::new(2_834_651_200, 2_834_653_248),
    scales: TensorSliceSpec::new(2_834_356_224, 2_834_356_288),
    biases: TensorSliceSpec::new(1_951_847_040, 1_951_847_104),
    out_dim: 1,
    in_dim: HIDDEN_SIZE,
    bits: 8,
};

const L3_SWITCH_GATE_PROJ_SPEC: AffineQuantSpec = AffineQuantSpec {
    weight: TensorSliceSpec::new(2_867_902_720, 3_002_120_448),
    scales: TensorSliceSpec::new(3_047_731_456, 3_056_120_064),
    biases: TensorSliceSpec::new(2_859_514_112, 2_867_902_720),
    out_dim: MOE_INTERMEDIATE,
    in_dim: HIDDEN_SIZE,
    bits: 4,
};

const L3_SWITCH_UP_PROJ_SPEC: AffineQuantSpec = AffineQuantSpec {
    weight: TensorSliceSpec::new(5_197_846_784, 5_332_064_512),
    scales: TensorSliceSpec::new(1_952_111_296, 1_960_499_904),
    biases: TensorSliceSpec::new(4_886_874_176, 4_895_262_784),
    out_dim: MOE_INTERMEDIATE,
    in_dim: HIDDEN_SIZE,
    bits: 4,
};

const L3_SWITCH_DOWN_PROJ_SPEC: AffineQuantSpec = AffineQuantSpec {
    weight: TensorSliceSpec::new(2_275_626_112, 2_409_843_840),
    scales: TensorSliceSpec::new(2_841_669_824, 2_850_058_432),
    biases: TensorSliceSpec::new(4_573_139_008, 4_581_527_616),
    out_dim: HIDDEN_SIZE,
    in_dim: MOE_INTERMEDIATE,
    bits: 4,
};

// Exact binary slice offsets in model-00004-of-00004.safetensors (after 8 + SHARD4_HEADER_LEN)
const FINAL_NORM_SPEC: TensorSliceSpec = TensorSliceSpec::new(270_172_160, 270_176_256);

const LM_HEAD_SPEC: AffineQuantSpec = AffineQuantSpec {
    weight: TensorSliceSpec::new(15_892_480, 270_172_160),
    scales: TensorSliceSpec::new(433_565_824, 449_458_304),
    biases: TensorSliceSpec::new(0, 15_892_480),
    out_dim: VOCAB_SIZE,
    in_dim: HIDDEN_SIZE,
    bits: 4,
};

struct LoadedAffineMatrix {
    weight_words: Vec<u32>,
    scales_bf16: Vec<u16>,
    biases_bf16: Vec<u16>,
    out_dim: usize,
    in_dim: usize,
    bits: usize,
}

impl LoadedAffineMatrix {
    fn read_from(file: &mut File, data_base: u64, spec: AffineQuantSpec) -> Result<Self, String> {
        let weight_words = read_u32_vec(file, data_base, spec.weight)?;
        let scales_bf16 = read_u16_vec(file, data_base, spec.scales)?;
        let biases_bf16 = read_u16_vec(file, data_base, spec.biases)?;
        let words_per_row = (spec.in_dim * spec.bits) / 32;
        let groups_per_row = spec.in_dim / QUANT_GROUP;
        if weight_words.len() != spec.out_dim * words_per_row {
            return Err(format!(
                "packed weight word count mismatch: got {}, expected {}",
                weight_words.len(),
                spec.out_dim * words_per_row
            ));
        }
        if scales_bf16.len() != spec.out_dim * groups_per_row
            || biases_bf16.len() != spec.out_dim * groups_per_row
        {
            return Err("scales/biases length mismatch".into());
        }
        Ok(Self {
            weight_words,
            scales_bf16,
            biases_bf16,
            out_dim: spec.out_dim,
            in_dim: spec.in_dim,
            bits: spec.bits,
        })
    }

    fn read_expert_from(
        file: &mut File,
        data_base: u64,
        spec: AffineQuantSpec,
        expert_idx: usize,
    ) -> Result<Self, String> {
        let words_per_expert = spec.out_dim * ((spec.in_dim * spec.bits) / 32);
        let groups_per_expert = spec.out_dim * (spec.in_dim / QUANT_GROUP);
        let weight_byte_len = (words_per_expert * 4) as u64;
        let sb_byte_len = (groups_per_expert * 2) as u64;
        let w_start = spec.weight.start + (expert_idx as u64) * weight_byte_len;
        let s_start = spec.scales.start + (expert_idx as u64) * sb_byte_len;
        let b_start = spec.biases.start + (expert_idx as u64) * sb_byte_len;
        let expert_spec = AffineQuantSpec {
            weight: TensorSliceSpec::new(w_start, w_start + weight_byte_len),
            scales: TensorSliceSpec::new(s_start, s_start + sb_byte_len),
            biases: TensorSliceSpec::new(b_start, b_start + sb_byte_len),
            out_dim: spec.out_dim,
            in_dim: spec.in_dim,
            bits: spec.bits,
        };
        Self::read_from(file, data_base, expert_spec)
    }

    fn dequantize_row(&self, row: usize) -> Vec<f32> {
        let words_per_row = (self.in_dim * self.bits) / 32;
        let groups_per_row = self.in_dim / QUANT_GROUP;
        let mask = (1u32 << self.bits) - 1;
        let w_row = &self.weight_words[row * words_per_row..(row + 1) * words_per_row];
        let s_row = &self.scales_bf16[row * groups_per_row..(row + 1) * groups_per_row];
        let b_row = &self.biases_bf16[row * groups_per_row..(row + 1) * groups_per_row];
        let mut out = vec![0.0f32; self.in_dim];
        for (col, slot) in out.iter_mut().enumerate() {
            let bit = col * self.bits;
            let word = w_row[bit / 32];
            let shift = bit % 32;
            let q = ((word >> shift) & mask) as f32;
            let g = col / QUANT_GROUP;
            let scale = bf16_to_f32(s_row[g]);
            let bias = bf16_to_f32(b_row[g]);
            *slot = scale * q + bias;
        }
        out
    }

    fn project_vec(&self, x: &[f32]) -> Vec<f32> {
        assert_eq!(x.len(), self.in_dim);
        let words_per_row = (self.in_dim * self.bits) / 32;
        let groups_per_row = self.in_dim / QUANT_GROUP;
        let mut out = vec![0.0f32; self.out_dim];
        if self.bits == 4 {
            for (row, out_slot) in out.iter_mut().enumerate() {
                let w_row = &self.weight_words[row * words_per_row..(row + 1) * words_per_row];
                let s_row = &self.scales_bf16[row * groups_per_row..(row + 1) * groups_per_row];
                let b_row = &self.biases_bf16[row * groups_per_row..(row + 1) * groups_per_row];
                let mut sum = 0.0f32;
                for g in 0..groups_per_row {
                    let scale = bf16_to_f32(s_row[g]);
                    let bias = bf16_to_f32(b_row[g]);
                    let w_group = &w_row[g * 8..g * 8 + 8];
                    let x_group = &x[g * 64..g * 64 + 64];
                    for w_idx in 0..8 {
                        let word = w_group[w_idx];
                        let xb = &x_group[w_idx * 8..w_idx * 8 + 8];
                        sum += (scale * ((word & 0xF) as f32) + bias) * xb[0];
                        sum += (scale * (((word >> 4) & 0xF) as f32) + bias) * xb[1];
                        sum += (scale * (((word >> 8) & 0xF) as f32) + bias) * xb[2];
                        sum += (scale * (((word >> 12) & 0xF) as f32) + bias) * xb[3];
                        sum += (scale * (((word >> 16) & 0xF) as f32) + bias) * xb[4];
                        sum += (scale * (((word >> 20) & 0xF) as f32) + bias) * xb[5];
                        sum += (scale * (((word >> 24) & 0xF) as f32) + bias) * xb[6];
                        sum += (scale * (((word >> 28) & 0xF) as f32) + bias) * xb[7];
                    }
                }
                *out_slot = sum;
            }
        } else if self.bits == 8 {
            for (row, out_slot) in out.iter_mut().enumerate() {
                let w_row = &self.weight_words[row * words_per_row..(row + 1) * words_per_row];
                let s_row = &self.scales_bf16[row * groups_per_row..(row + 1) * groups_per_row];
                let b_row = &self.biases_bf16[row * groups_per_row..(row + 1) * groups_per_row];
                let mut sum = 0.0f32;
                for g in 0..groups_per_row {
                    let scale = bf16_to_f32(s_row[g]);
                    let bias = bf16_to_f32(b_row[g]);
                    let w_group = &w_row[g * 16..g * 16 + 16];
                    let x_group = &x[g * 64..g * 64 + 64];
                    for w_idx in 0..16 {
                        let word = w_group[w_idx];
                        let xb = &x_group[w_idx * 4..w_idx * 4 + 4];
                        sum += (scale * ((word & 0xFF) as f32) + bias) * xb[0];
                        sum += (scale * (((word >> 8) & 0xFF) as f32) + bias) * xb[1];
                        sum += (scale * (((word >> 16) & 0xFF) as f32) + bias) * xb[2];
                        sum += (scale * (((word >> 24) & 0xFF) as f32) + bias) * xb[3];
                    }
                }
                *out_slot = sum;
            }
        } else {
            unreachable!("unsupported quantization bit width");
        }
        out
    }
}

struct Qwen36Layer3Weights {
    shard1_file: File,
    shard1_data_base: u64,
    input_norm: Vec<f32>,
    q_proj: LoadedAffineMatrix,
    q_norm: Vec<f32>,
    k_proj: LoadedAffineMatrix,
    k_norm: Vec<f32>,
    v_proj: LoadedAffineMatrix,
    o_proj: LoadedAffineMatrix,
    post_attn_norm: Vec<f32>,
    mlp_gate: LoadedAffineMatrix,
    shared_gate_proj: LoadedAffineMatrix,
    shared_up_proj: LoadedAffineMatrix,
    shared_down_proj: LoadedAffineMatrix,
    shared_expert_gate: LoadedAffineMatrix,
    final_norm: Vec<f32>,
    lm_head: LoadedAffineMatrix,
}

#[derive(Clone, Debug)]
struct ForwardTrace {
    prefix_k_by_head: [Vec<f32>; KV_HEADS],
    prefix_v_by_head: [Vec<f32>; KV_HEADS],
    decode_hidden_out: Vec<f32>,
    decode_logits: Vec<f32>,
    phase_state_digest: Digest,
}

type ProjectedTokenQkv = (
    Vec<[f32; HEAD_DIM]>,
    Vec<[f32; HEAD_DIM]>,
    [[f32; HEAD_DIM]; KV_HEADS],
    [[f32; HEAD_DIM]; KV_HEADS],
);

type IdentityMutator = (&'static str, fn(&mut ExactKvIdentityQuery));
type PageMutator = (&'static str, fn(&mut TensorPage));

impl Qwen36Layer3Weights {
    fn load() -> Result<Self, String> {
        let mut shard1 = File::open(SHARD1_FILE).map_err(|e| format!("open {SHARD1_FILE}: {e}"))?;
        let mut header_len_bytes = [0u8; 8];
        shard1
            .read_exact(&mut header_len_bytes)
            .map_err(|e| format!("read shard1 header len: {e}"))?;
        let shard1_header_len = u64::from_le_bytes(header_len_bytes);
        if shard1_header_len != SHARD1_HEADER_LEN {
            return Err(format!(
                "shard1 header length mismatch: got {shard1_header_len}, expected {SHARD1_HEADER_LEN}"
            ));
        }
        let shard1_data_base = 8 + shard1_header_len;

        let mut shard4 = File::open(SHARD4_FILE).map_err(|e| format!("open {SHARD4_FILE}: {e}"))?;
        shard4
            .read_exact(&mut header_len_bytes)
            .map_err(|e| format!("read shard4 header len: {e}"))?;
        let shard4_header_len = u64::from_le_bytes(header_len_bytes);
        if shard4_header_len != SHARD4_HEADER_LEN {
            return Err(format!(
                "shard4 header length mismatch: got {shard4_header_len}, expected {SHARD4_HEADER_LEN}"
            ));
        }
        let shard4_data_base = 8 + shard4_header_len;

        let input_norm = read_bf16_as_f32(&mut shard1, shard1_data_base, L3_INPUT_NORM_SPEC)?;
        let q_proj = LoadedAffineMatrix::read_from(&mut shard1, shard1_data_base, L3_Q_PROJ_SPEC)?;
        let q_norm = read_bf16_as_f32(&mut shard1, shard1_data_base, L3_Q_NORM_SPEC)?;
        let k_proj = LoadedAffineMatrix::read_from(&mut shard1, shard1_data_base, L3_K_PROJ_SPEC)?;
        let k_norm = read_bf16_as_f32(&mut shard1, shard1_data_base, L3_K_NORM_SPEC)?;
        let v_proj = LoadedAffineMatrix::read_from(&mut shard1, shard1_data_base, L3_V_PROJ_SPEC)?;
        let o_proj = LoadedAffineMatrix::read_from(&mut shard1, shard1_data_base, L3_O_PROJ_SPEC)?;
        let post_attn_norm =
            read_bf16_as_f32(&mut shard1, shard1_data_base, L3_POST_ATTN_NORM_SPEC)?;
        let mlp_gate =
            LoadedAffineMatrix::read_from(&mut shard1, shard1_data_base, L3_MLP_GATE_SPEC)?;
        let shared_gate_proj =
            LoadedAffineMatrix::read_from(&mut shard1, shard1_data_base, L3_SHARED_GATE_PROJ_SPEC)?;
        let shared_up_proj =
            LoadedAffineMatrix::read_from(&mut shard1, shard1_data_base, L3_SHARED_UP_PROJ_SPEC)?;
        let shared_down_proj =
            LoadedAffineMatrix::read_from(&mut shard1, shard1_data_base, L3_SHARED_DOWN_PROJ_SPEC)?;
        let shared_expert_gate = LoadedAffineMatrix::read_from(
            &mut shard1,
            shard1_data_base,
            L3_SHARED_EXPERT_GATE_SPEC,
        )?;

        let final_norm = read_bf16_as_f32(&mut shard4, shard4_data_base, FINAL_NORM_SPEC)?;
        let lm_head = LoadedAffineMatrix::read_from(&mut shard4, shard4_data_base, LM_HEAD_SPEC)?;

        Ok(Self {
            shard1_file: shard1,
            shard1_data_base,
            input_norm,
            q_proj,
            q_norm,
            k_proj,
            k_norm,
            v_proj,
            o_proj,
            post_attn_norm,
            mlp_gate,
            shared_gate_proj,
            shared_up_proj,
            shared_down_proj,
            shared_expert_gate,
            final_norm,
            lm_head,
        })
    }

    fn embed_token(&mut self, token_id: u32) -> Result<Vec<f32>, String> {
        let row = token_id as usize;
        if row >= VOCAB_SIZE {
            return Err(format!("token_id {token_id} out of range"));
        }
        let words_per_row = (HIDDEN_SIZE * EMBED_TOKENS_SPEC.bits) / 32;
        let groups_per_row = HIDDEN_SIZE / QUANT_GROUP;
        let w_start = EMBED_TOKENS_SPEC.weight.start + (row * words_per_row * 4) as u64;
        let s_start = EMBED_TOKENS_SPEC.scales.start + (row * groups_per_row * 2) as u64;
        let b_start = EMBED_TOKENS_SPEC.biases.start + (row * groups_per_row * 2) as u64;
        let row_spec = AffineQuantSpec {
            weight: TensorSliceSpec::new(w_start, w_start + (words_per_row * 4) as u64),
            scales: TensorSliceSpec::new(s_start, s_start + (groups_per_row * 2) as u64),
            biases: TensorSliceSpec::new(b_start, b_start + (groups_per_row * 2) as u64),
            out_dim: 1,
            in_dim: HIDDEN_SIZE,
            bits: EMBED_TOKENS_SPEC.bits,
        };
        let row_mat =
            LoadedAffineMatrix::read_from(&mut self.shard1_file, self.shard1_data_base, row_spec)?;
        Ok(row_mat.dequantize_row(0))
    }

    fn project_q_gate_kv_for_token(
        &self,
        hidden: &[f32],
        tppe_state: &TppeState,
        token_pos: usize,
    ) -> ProjectedTokenQkv {
        let normed = rms_norm_shifted(hidden, &self.input_norm);
        let qg_raw = self.q_proj.project_vec(&normed);
        let k_raw = self.k_proj.project_vec(&normed);
        let v_raw = self.v_proj.project_vec(&normed);

        let mut q_heads = vec![[0.0f32; HEAD_DIM]; Q_HEADS];
        let mut gate_heads = vec![[0.0f32; HEAD_DIM]; Q_HEADS];
        for h in 0..Q_HEADS {
            let base = h * (2 * HEAD_DIM);
            let q_slice = &qg_raw[base..base + HEAD_DIM];
            let g_slice = &qg_raw[base + HEAD_DIM..base + 2 * HEAD_DIM];
            let mut q_normed = rms_norm_shifted(q_slice, &self.q_norm);
            apply_tppe_rotary_in_place(&mut q_normed, tppe_state, token_pos);
            q_heads[h].copy_from_slice(&q_normed);
            gate_heads[h].copy_from_slice(g_slice);
        }

        let mut k_heads = [[0.0f32; HEAD_DIM]; KV_HEADS];
        let mut v_heads = [[0.0f32; HEAD_DIM]; KV_HEADS];
        for kv_h in 0..KV_HEADS {
            let base = kv_h * HEAD_DIM;
            let mut k_normed = rms_norm_shifted(&k_raw[base..base + HEAD_DIM], &self.k_norm);
            apply_tppe_rotary_in_place(&mut k_normed, tppe_state, token_pos);
            k_heads[kv_h].copy_from_slice(&k_normed);
            v_heads[kv_h].copy_from_slice(&v_raw[base..base + HEAD_DIM]);
        }

        (q_heads, gate_heads, k_heads, v_heads)
    }

    fn finish_decode_step_from_kv(
        &mut self,
        decode_hidden_in: &[f32],
        q_heads: &[[f32; HEAD_DIM]],
        gate_heads: &[[f32; HEAD_DIM]],
        full_k_by_head: &[Vec<f32>; KV_HEADS],
        full_v_by_head: &[Vec<f32>; KV_HEADS],
        seq_len: usize,
    ) -> Result<(Vec<f32>, Vec<f32>), String> {
        let scale = 1.0f32 / (HEAD_DIM as f32).sqrt();
        let heads_per_kv = Q_HEADS / KV_HEADS;
        let mut gated_concat = vec![0.0f32; Q_HEADS * HEAD_DIM];

        for q_h in 0..Q_HEADS {
            let kv_h = q_h / heads_per_kv;
            let k_cache = &full_k_by_head[kv_h];
            let v_cache = &full_v_by_head[kv_h];
            assert_eq!(k_cache.len(), seq_len * HEAD_DIM);
            assert_eq!(v_cache.len(), seq_len * HEAD_DIM);

            let q_vec = &q_heads[q_h];
            let mut scores = vec![0.0f32; seq_len];
            let mut max_score = f32::NEG_INFINITY;
            for (t, score_slot) in scores.iter_mut().enumerate() {
                let k_vec = &k_cache[t * HEAD_DIM..(t + 1) * HEAD_DIM];
                let mut dot = 0.0f32;
                for d in 0..HEAD_DIM {
                    dot += q_vec[d] * k_vec[d];
                }
                let s = dot * scale;
                if s > max_score {
                    max_score = s;
                }
                *score_slot = s;
            }
            let mut denom = 0.0f32;
            for s in &mut scores {
                *s = (*s - max_score).exp();
                denom += *s;
            }
            for s in &mut scores {
                *s /= denom;
            }

            let mut attn_out = [0.0f32; HEAD_DIM];
            for (t, &weight) in scores.iter().enumerate() {
                let v_vec = &v_cache[t * HEAD_DIM..(t + 1) * HEAD_DIM];
                for d in 0..HEAD_DIM {
                    attn_out[d] += weight * v_vec[d];
                }
            }

            let gate_vec = &gate_heads[q_h];
            let out_slice = &mut gated_concat[q_h * HEAD_DIM..(q_h + 1) * HEAD_DIM];
            for d in 0..HEAD_DIM {
                out_slice[d] = attn_out[d] * sigmoid(gate_vec[d]);
            }
        }

        let o_out = self.o_proj.project_vec(&gated_concat);
        let mut post_attn_hidden = vec![0.0f32; HIDDEN_SIZE];
        for i in 0..HIDDEN_SIZE {
            post_attn_hidden[i] = decode_hidden_in[i] + o_out[i];
        }

        // MoE block on layer 3
        let mlp_in = rms_norm_shifted(&post_attn_hidden, &self.post_attn_norm);
        let router_logits = self.mlp_gate.project_vec(&mlp_in);
        let router_probs = softmax(&router_logits);

        // Top-8 routed experts (norm_topk_prob = true)
        let mut indexed: Vec<(usize, f32)> = router_probs.into_iter().enumerate().collect();
        indexed.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let top_k = &indexed[..MOE_TOP_K];
        let weight_sum: f32 = top_k.iter().map(|(_, w)| *w).sum();

        let mut moe_accum = vec![0.0f32; HIDDEN_SIZE];
        for &(expert_idx, raw_weight) in top_k {
            let norm_weight = raw_weight / weight_sum;
            let gate_mat = LoadedAffineMatrix::read_expert_from(
                &mut self.shard1_file,
                self.shard1_data_base,
                L3_SWITCH_GATE_PROJ_SPEC,
                expert_idx,
            )?;
            let up_mat = LoadedAffineMatrix::read_expert_from(
                &mut self.shard1_file,
                self.shard1_data_base,
                L3_SWITCH_UP_PROJ_SPEC,
                expert_idx,
            )?;
            let down_mat = LoadedAffineMatrix::read_expert_from(
                &mut self.shard1_file,
                self.shard1_data_base,
                L3_SWITCH_DOWN_PROJ_SPEC,
                expert_idx,
            )?;
            let gate_act = gate_mat.project_vec(&mlp_in);
            let up_act = up_mat.project_vec(&mlp_in);
            let mut swiglu = vec![0.0f32; MOE_INTERMEDIATE];
            for i in 0..MOE_INTERMEDIATE {
                swiglu[i] = silu(gate_act[i]) * up_act[i];
            }
            let expert_out = down_mat.project_vec(&swiglu);
            for i in 0..HIDDEN_SIZE {
                moe_accum[i] += norm_weight * expert_out[i];
            }
        }

        // Shared expert
        let sh_gate = self.shared_gate_proj.project_vec(&mlp_in);
        let sh_up = self.shared_up_proj.project_vec(&mlp_in);
        let mut sh_swiglu = vec![0.0f32; MOE_INTERMEDIATE];
        for i in 0..MOE_INTERMEDIATE {
            sh_swiglu[i] = silu(sh_gate[i]) * sh_up[i];
        }
        let sh_down = self.shared_down_proj.project_vec(&sh_swiglu);
        let sh_scale = sigmoid(self.shared_expert_gate.project_vec(&mlp_in)[0]);

        let mut layer_out = vec![0.0f32; HIDDEN_SIZE];
        for i in 0..HIDDEN_SIZE {
            layer_out[i] = post_attn_hidden[i] + moe_accum[i] + sh_scale * sh_down[i];
        }

        let final_normed = rms_norm_shifted(&layer_out, &self.final_norm);
        let logits = self.lm_head.project_vec(&final_normed);
        Ok((layer_out, logits))
    }

    fn run_uninterrupted_forward(
        &mut self,
        prefix_tokens: &[u32],
        decode_token: u32,
        tppe_config: &TppeConfig,
    ) -> Result<ForwardTrace, String> {
        let mut prefix_k_by_head = [Vec::new(), Vec::new()];
        let mut prefix_v_by_head = [Vec::new(), Vec::new()];
        let mut full_k_by_head = [Vec::new(), Vec::new()];
        let mut full_v_by_head = [Vec::new(), Vec::new()];

        let mut phase_transcript = Vec::new();
        for (pos, &tok) in prefix_tokens.iter().enumerate() {
            let hidden = self.embed_token(tok)?;
            let state = tppe_state_for_position(tppe_config, pos, tok)?;
            append_phase_state_bytes(&mut phase_transcript, &state);
            let (_q, _g, k_heads, v_heads) = self.project_q_gate_kv_for_token(&hidden, &state, pos);
            for kv_h in 0..KV_HEADS {
                prefix_k_by_head[kv_h].extend_from_slice(&k_heads[kv_h]);
                prefix_v_by_head[kv_h].extend_from_slice(&v_heads[kv_h]);
                full_k_by_head[kv_h].extend_from_slice(&k_heads[kv_h]);
                full_v_by_head[kv_h].extend_from_slice(&v_heads[kv_h]);
            }
        }
        let phase_state_digest = Digest::sha256(&phase_transcript);

        let decode_pos = prefix_tokens.len();
        let decode_hidden = self.embed_token(decode_token)?;
        let decode_state = tppe_state_for_position(tppe_config, decode_pos, decode_token)?;
        let (q_heads, gate_heads, k_new, v_new) =
            self.project_q_gate_kv_for_token(&decode_hidden, &decode_state, decode_pos);
        for kv_h in 0..KV_HEADS {
            full_k_by_head[kv_h].extend_from_slice(&k_new[kv_h]);
            full_v_by_head[kv_h].extend_from_slice(&v_new[kv_h]);
        }

        let (decode_hidden_out, decode_logits) = self.finish_decode_step_from_kv(
            &decode_hidden,
            &q_heads,
            &gate_heads,
            &full_k_by_head,
            &full_v_by_head,
            prefix_tokens.len() + 1,
        )?;

        Ok(ForwardTrace {
            prefix_k_by_head,
            prefix_v_by_head,
            decode_hidden_out,
            decode_logits,
            phase_state_digest,
        })
    }

    fn run_injected_decode(
        &mut self,
        injected_payloads: &[TensorPayload; KV_HEADS],
        prefix_len: usize,
        decode_token: u32,
        tppe_config: &TppeConfig,
    ) -> Result<(Vec<f32>, Vec<f32>), String> {
        let mut full_k_by_head = [
            f32_vec_from_le_bytes(&injected_payloads[0].key)?,
            f32_vec_from_le_bytes(&injected_payloads[1].key)?,
        ];
        let mut full_v_by_head = [
            f32_vec_from_le_bytes(&injected_payloads[0].value)?,
            f32_vec_from_le_bytes(&injected_payloads[1].value)?,
        ];
        for kv_h in 0..KV_HEADS {
            if full_k_by_head[kv_h].len() != prefix_len * HEAD_DIM
                || full_v_by_head[kv_h].len() != prefix_len * HEAD_DIM
            {
                return Err("injected KV payload shape mismatch".into());
            }
        }

        let decode_hidden = self.embed_token(decode_token)?;
        let decode_state = tppe_state_for_position(tppe_config, prefix_len, decode_token)?;
        let (q_heads, gate_heads, k_new, v_new) =
            self.project_q_gate_kv_for_token(&decode_hidden, &decode_state, prefix_len);
        for kv_h in 0..KV_HEADS {
            full_k_by_head[kv_h].extend_from_slice(&k_new[kv_h]);
            full_v_by_head[kv_h].extend_from_slice(&v_new[kv_h]);
        }

        self.finish_decode_step_from_kv(
            &decode_hidden,
            &q_heads,
            &gate_heads,
            &full_k_by_head,
            &full_v_by_head,
            prefix_len + 1,
        )
    }
}

fn tppe_state_for_position(
    config: &TppeConfig,
    position: usize,
    _token_id: u32,
) -> Result<TppeState, String> {
    // Sequential positional states from the shared runtime helper:
    // state is a function of position alone, never of a token ID.
    let states = globalmind_runtime::trace::tppe_states_for_positions(config, position + 1)?;
    Ok(states[position])
}

fn append_phase_state_bytes(buf: &mut Vec<u8>, state: &TppeState) {
    let t2 = state.phase_t2();
    buf.extend_from_slice(&t2[0].to_le_bytes());
    buf.extend_from_slice(&t2[1].to_le_bytes());
    buf.extend_from_slice(&state.hyperbolic_radius.to_le_bytes());
    buf.extend_from_slice(&state.poincare_radius.to_le_bytes());
    buf.extend_from_slice(&state.cumulative_revolutions.to_le_bytes());
    buf.extend_from_slice(&state.boundary_margin.to_le_bytes());
}

fn apply_tppe_rotary_in_place(vec: &mut [f32], state: &TppeState, _token_pos: usize) {
    // Candidate v2 pure TPPE: T2 phase rotation on T^2 (fast phase on
    // even pairs, slow phase on odd) plus radial scale
    // `1 + 0.05 * poincare_radius`. No position * inv_freq product, no
    // `0.01 * phase` hybrid term (those are the retired reference-v1
    // hybrid, audited invalid 2026-11-02). Matches the runtime's
    // apply_position_encoder_in_place for the TPPE arms.
    let rope_pairs = 32usize;
    let radial_scale = (1.0 + 0.05 * state.poincare_radius) as f32;
    let t2 = state.phase_t2();
    for pair in 0..rope_pairs {
        let angle = t2[pair % 2] as f32;
        let (sin_a, cos_a) = angle.sin_cos();
        let x0 = vec[pair];
        let x1 = vec[pair + rope_pairs];
        vec[pair] = radial_scale * (x0 * cos_a - x1 * sin_a);
        vec[pair + rope_pairs] = radial_scale * (x1 * cos_a + x0 * sin_a);
    }
}

fn bf16_to_f32(bits: u16) -> f32 {
    f32::from_bits((bits as u32) << 16)
}

fn rms_norm_shifted(x: &[f32], weight: &[f32]) -> Vec<f32> {
    assert_eq!(x.len(), weight.len());
    let mut sum_sq = 0.0f32;
    for &v in x {
        sum_sq += v * v;
    }
    let inv_rms = 1.0f32 / (sum_sq / (x.len() as f32) + RMS_NORM_EPS).sqrt();
    let mut out = vec![0.0f32; x.len()];
    for i in 0..x.len() {
        out[i] = x[i] * inv_rms * (1.0f32 + weight[i]);
    }
    out
}

fn sigmoid(x: f32) -> f32 {
    1.0f32 / (1.0f32 + (-x).exp())
}

fn silu(x: f32) -> f32 {
    x * sigmoid(x)
}

fn softmax(x: &[f32]) -> Vec<f32> {
    let mut max_v = f32::NEG_INFINITY;
    for &v in x {
        if v > max_v {
            max_v = v;
        }
    }
    let mut out = vec![0.0f32; x.len()];
    let mut denom = 0.0f32;
    for (i, &v) in x.iter().enumerate() {
        let e = (v - max_v).exp();
        out[i] = e;
        denom += e;
    }
    for v in &mut out {
        *v /= denom;
    }
    out
}

fn read_raw_bytes(
    file: &mut File,
    data_base: u64,
    spec: TensorSliceSpec,
) -> Result<Vec<u8>, String> {
    file.seek(SeekFrom::Start(data_base + spec.start))
        .map_err(|e| format!("seek to {}: {e}", spec.start))?;
    let mut buf = vec![0u8; spec.byte_len()];
    file.read_exact(&mut buf)
        .map_err(|e| format!("read {} bytes at {}: {e}", spec.byte_len(), spec.start))?;
    Ok(buf)
}

fn read_u32_vec(
    file: &mut File,
    data_base: u64,
    spec: TensorSliceSpec,
) -> Result<Vec<u32>, String> {
    let bytes = read_raw_bytes(file, data_base, spec)?;
    if !bytes.len().is_multiple_of(4) {
        return Err("u32 slice byte length not divisible by 4".into());
    }
    Ok(bytes
        .as_chunks::<4>()
        .0
        .iter()
        .copied()
        .map(u32::from_le_bytes)
        .collect())
}

fn read_u16_vec(
    file: &mut File,
    data_base: u64,
    spec: TensorSliceSpec,
) -> Result<Vec<u16>, String> {
    let bytes = read_raw_bytes(file, data_base, spec)?;
    if !bytes.len().is_multiple_of(2) {
        return Err("u16 slice byte length not divisible by 2".into());
    }
    Ok(bytes
        .as_chunks::<2>()
        .0
        .iter()
        .copied()
        .map(u16::from_le_bytes)
        .collect())
}

fn read_bf16_as_f32(
    file: &mut File,
    data_base: u64,
    spec: TensorSliceSpec,
) -> Result<Vec<f32>, String> {
    Ok(read_u16_vec(file, data_base, spec)?
        .into_iter()
        .map(bf16_to_f32)
        .collect())
}

fn f32_slice_to_le_bytes(values: &[f32]) -> Box<[u8]> {
    let mut bytes = Vec::with_capacity(values.len() * 4);
    for &v in values {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    bytes.into_boxed_slice()
}

fn f32_vec_from_le_bytes(bytes: &[u8]) -> Result<Vec<f32>, String> {
    if !bytes.len().is_multiple_of(4) {
        return Err("f32 byte buffer length is not a multiple of 4".into());
    }
    Ok(bytes
        .as_chunks::<4>()
        .0
        .iter()
        .copied()
        .map(f32::from_le_bytes)
        .collect())
}

struct IsolatedSurreal {
    port: u16,
    db_dir: PathBuf,
    namespace: String,
    database: String,
    username: String,
    password: String,
    child: Option<Child>,
}

impl IsolatedSurreal {
    fn start_with_schema() -> Result<(Self, Duration), String> {
        let t0 = Instant::now();
        let listener =
            TcpListener::bind("127.0.0.1:0").map_err(|e| format!("bind free port: {e}"))?;
        let port = listener
            .local_addr()
            .map_err(|e| format!("local_addr: {e}"))?
            .port();
        drop(listener);
        assert_ne!(
            port, 8000,
            "never touch quarantined live SurrealDB port 8000"
        );

        let manifest_dir = runtime_manifest_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos();
        let db_dir = manifest_dir
            .join("target")
            .join(format!("tensor-kv-surreal-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&db_dir).map_err(|e| format!("create db_dir: {e}"))?;

        let mut instance = Self {
            port,
            db_dir,
            namespace: "globalmind_tensor_kv_iso".into(),
            database: "tensor_kv_qual_001".into(),
            username: "root".into(),
            password: format!("ephemeral_lane_b_{nonce}"),
            child: None,
        };
        instance.spawn_process()?;
        instance.apply_schema_chain(&manifest_dir)?;
        Ok((instance, t0.elapsed()))
    }

    fn store_config(&self) -> StoreConfig {
        StoreConfig {
            endpoint: format!("http://127.0.0.1:{}", self.port),
            namespace: self.namespace.clone(),
            database: self.database.clone(),
            username: Some(self.username.clone()),
            password: Some(self.password.clone()),
            allow_quarantined_core: false,
        }
    }

    fn connect_store(&self) -> Result<GlobalMindStore, String> {
        GlobalMindStore::connect(self.store_config())
    }

    fn pid(&self) -> Option<u32> {
        self.child.as_ref().map(Child::id)
    }

    fn spawn_process(&mut self) -> Result<(), String> {
        let storage_uri = format!("surrealkv://{}", self.db_dir.display());
        let bind_addr = format!("127.0.0.1:{}", self.port);
        let child = Command::new("/opt/homebrew/bin/surreal")
            .args([
                "start",
                "--no-banner",
                "--log",
                "none",
                "--bind",
                &bind_addr,
                "--user",
                &self.username,
                "--pass",
                &self.password,
                &storage_uri,
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("spawn surreal: {e}"))?;
        self.child = Some(child);

        let deadline = Instant::now() + Duration::from_secs(15);
        while Instant::now() < deadline {
            if self.raw_sql("INFO FOR KV;").is_ok() {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(40));
        }
        Err("timed out waiting for isolated SurrealDB to accept connections".into())
    }

    fn restart(&mut self) -> Result<Duration, String> {
        let t0 = Instant::now();
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        // Prove server is genuinely down before respawning
        let addr = format!("127.0.0.1:{}", self.port);
        if TcpStream::connect(&addr).is_ok() {
            return Err("isolated SurrealDB still accepting connections after stop".into());
        }
        self.spawn_process()?;
        Ok(t0.elapsed())
    }

    fn apply_schema_chain(&self, manifest_dir: &Path) -> Result<(), String> {
        let repo_root = manifest_dir
            .parent()
            .ok_or_else(|| "missing repo root".to_string())?;
        let schema_dir = repo_root.join("hypermind-nested-kv").join("schema");
        for file_name in [
            "0001_hypermind_graph.surql",
            "0003_nested_hyperbolic_kv.surql",
            "0004_transformer_kv_tensor_pages.surql",
            "0005_universal_atom_lineage.surql",
            "0006_immutable_atom_address.surql",
            "0007_globalmind_distillation_trajectories.surql",
            "0008_superposition_holding.surql",
            "0009_immutable_tensor_kv.surql",
            "0010_native_distillation_ingress.surql",
            "0011_atomic_trajectory_cells.surql",
        ] {
            let path = schema_dir.join(file_name);
            let sql =
                fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
            self.raw_sql(&sql)
                .map_err(|e| format!("apply {file_name}: {e}"))?;
        }
        self.raw_sql(
            "UPSERT workspace:globalmind CONTENT { slug: 'globalmind', created_at: time::now() } RETURN NONE;",
        )?;
        Ok(())
    }

    fn raw_sql(&self, query: &str) -> Result<Vec<Value>, String> {
        let signin_body = format!(
            "{{\"user\":{},\"pass\":{}}}",
            serde_json::to_string(&self.username).unwrap(),
            serde_json::to_string(&self.password).unwrap()
        );
        let signin_resp =
            http_post_loopback(self.port, "/signin", "application/json", &signin_body, None)?;
        let signin_json: Value =
            serde_json::from_str(&signin_resp).map_err(|e| format!("signin json: {e}"))?;
        let token = signin_json
            .get("token")
            .and_then(Value::as_str)
            .ok_or_else(|| "missing signin token".to_string())?;

        let full_sql = format!("USE NS {} DB {}; {query}", self.namespace, self.database);
        let resp = http_post_loopback(self.port, "/sql", "text/plain", &full_sql, Some(token))?;
        let statements: Value =
            serde_json::from_str(&resp).map_err(|e| format!("sql json: {e}"))?;
        let arr = statements
            .as_array()
            .ok_or_else(|| "SurrealDB response not array".to_string())?;
        let mut results = Vec::new();
        for (idx, stmt) in arr.iter().enumerate() {
            if idx == 0 {
                continue;
            }
            if stmt.get("status").and_then(Value::as_str) == Some("ERR") {
                return Err(format!(
                    "SurrealDB statement {idx} failed: {}",
                    stmt.get("result").cloned().unwrap_or(Value::Null)
                ));
            }
            results.push(stmt.get("result").cloned().unwrap_or(Value::Null));
        }
        Ok(results)
    }
}

impl Drop for IsolatedSurreal {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        let _ = fs::remove_dir_all(&self.db_dir);
    }
}

fn http_post_loopback(
    port: u16,
    path: &str,
    content_type: &str,
    body: &str,
    token: Option<&str>,
) -> Result<String, String> {
    let addr = format!("127.0.0.1:{port}");
    let mut stream = TcpStream::connect(&addr).map_err(|e| format!("connect {addr}: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(15)))
        .map_err(|e| e.to_string())?;
    let auth = token
        .map(|t| format!("\r\nAuthorization: Bearer {t}"))
        .unwrap_or_default();
    let req = format!(
        "POST {path} HTTP/1.1\r\nHost: {addr}\r\nContent-Type: {content_type}\r\nAccept: application/json\r\nContent-Length: {}{auth}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream
        .write_all(req.as_bytes())
        .map_err(|e| format!("write: {e}"))?;
    let mut resp = Vec::new();
    stream
        .read_to_end(&mut resp)
        .map_err(|e| format!("read: {e}"))?;
    let text = String::from_utf8(resp).map_err(|e| format!("utf8: {e}"))?;
    if !text.starts_with("HTTP/1.1 200") {
        return Err(format!(
            "HTTP error: {}",
            text.lines().next().unwrap_or("unknown")
        ));
    }
    let sep = text
        .find("\r\n\r\n")
        .ok_or_else(|| "missing HTTP body separator".to_string())?;
    Ok(text[sep + 4..].to_string())
}

#[derive(Clone, Debug)]
struct ExactKvIdentityQuery {
    workspace: RecordId,
    model_id: String,
    model_revision: String,
    model_digest: Digest,
    request_digest: Digest,
    prefix_digest: Digest,
    tensor_contract_digest: Digest,
    tppe_version: String,
    tppe_configuration_digest: Digest,
    address_contract_digest: Digest,
    geometry_space: RecordId,
    layer: u32,
    kv_head: u32,
    token_start: u64,
    token_end: u64,
    tensor_shape: [u32; 2],
    dtype: TensorDtype,
    layout: TensorLayout,
    address: RecordId,
    phase_state_digest: Digest,
}

fn runtime_exact_identity(
    q: &ExactKvIdentityQuery,
    session: &RecordId,
    page: &RecordId,
    key_digest: &Digest,
    value_digest: &Digest,
) -> ExactKvIdentity {
    ExactKvIdentity {
        workspace: q.workspace.clone(),
        session: session.clone(),
        page: page.clone(),
        model_id: q.model_id.clone(),
        model_revision: q.model_revision.clone(),
        model_digest: q.model_digest.clone(),
        request_digest: q.request_digest.clone(),
        prefix_digest: q.prefix_digest.clone(),
        tppe_version: q.tppe_version.clone(),
        tppe_configuration_digest: q.tppe_configuration_digest.clone(),
        address_contract_digest: q.address_contract_digest.clone(),
        tensor_contract_digest: q.tensor_contract_digest.clone(),
        geometry_space: q.geometry_space.clone(),
        dtype: q.dtype,
        layer_count: NUM_LAYERS,
        kv_head_count: KV_HEADS as u32,
        head_dim: HEAD_DIM as u32,
        layer: q.layer,
        kv_head: q.kv_head,
        token_start: q.token_start,
        token_end: q.token_end,
        tensor_shape: q.tensor_shape,
        layout: q.layout,
        address: q.address.clone(),
        phase_state_digest: q.phase_state_digest.clone(),
        revision: 1,
        key_digest: key_digest.clone(),
        value_digest: value_digest.clone(),
    }
}

fn dtype_str(dtype: TensorDtype) -> &'static str {
    match dtype {
        TensorDtype::F16 => "f16",
        TensorDtype::Bf16 => "bf16",
        TensorDtype::F32 => "f32",
        TensorDtype::Q8_0 => "q8_0",
        TensorDtype::Q4_0 => "q4_0",
    }
}

fn layout_str(layout: TensorLayout) -> &'static str {
    match layout {
        TensorLayout::TokenMajor => "token_major",
        TensorLayout::HeadDimMajor => "head_dim_major",
    }
}

fn lookup_tensor_page_strict(
    surreal: &IsolatedSurreal,
    store: &GlobalMindStore,
    q: &ExactKvIdentityQuery,
) -> Result<TensorPayload, String> {
    let sql = format!(
        "SELECT id FROM transformer_kv_session WHERE active = true \
         AND workspace = workspace:`{workspace}` \
         AND model_id = {model_id} \
         AND model_revision = {model_revision} \
         AND model_digest = {model_digest} \
         AND request_digest = {request_digest} \
         AND prefix_digest = {prefix_digest} \
         AND tensor_contract_digest = {tensor_contract_digest} \
         AND toroidal_phase_encoder_version = {tppe_version} \
         AND tppe_configuration_digest = {tppe_configuration_digest} \
         AND address_contract_digest = {address_contract_digest} \
         AND geometry_space = geometry_space:`{geometry_space}` \
         AND dtype = {dtype} LIMIT 1;",
        workspace = q.workspace.as_str(),
        model_id = serde_json::to_string(&q.model_id).unwrap(),
        model_revision = serde_json::to_string(&q.model_revision).unwrap(),
        model_digest = serde_json::to_string(q.model_digest.as_str()).unwrap(),
        request_digest = serde_json::to_string(q.request_digest.as_str()).unwrap(),
        prefix_digest = serde_json::to_string(q.prefix_digest.as_str()).unwrap(),
        tensor_contract_digest = serde_json::to_string(q.tensor_contract_digest.as_str()).unwrap(),
        tppe_version = serde_json::to_string(&q.tppe_version).unwrap(),
        tppe_configuration_digest =
            serde_json::to_string(q.tppe_configuration_digest.as_str()).unwrap(),
        address_contract_digest =
            serde_json::to_string(q.address_contract_digest.as_str()).unwrap(),
        geometry_space = q.geometry_space.as_str(),
        dtype = serde_json::to_string(dtype_str(q.dtype)).unwrap(),
    );
    let session_res = surreal.raw_sql(&sql)?;
    let session_id_full = session_res
        .first()
        .and_then(Value::as_array)
        .and_then(|rows| rows.first())
        .and_then(|row| row.get("id"))
        .and_then(Value::as_str)
        .ok_or_else(|| "cache miss: no matching active TensorSession".to_string())?;

    let page_sql = format!(
        "SELECT id FROM transformer_kv_page WHERE active = true \
         AND workspace = workspace:`{workspace}` \
         AND session = {session_id_full} \
         AND layer = {layer} \
         AND kv_head = {kv_head} \
         AND token_start = {token_start} \
         AND token_end = {token_end} \
         AND tensor_shape = [{s0}, {s1}] \
         AND dtype = {dtype} \
         AND layout = {layout} \
         AND address = atom_address:`{address}` \
         AND phase_state_digest = {phase_state_digest} LIMIT 1;",
        workspace = q.workspace.as_str(),
        layer = q.layer,
        kv_head = q.kv_head,
        token_start = q.token_start,
        token_end = q.token_end,
        s0 = q.tensor_shape[0],
        s1 = q.tensor_shape[1],
        dtype = serde_json::to_string(dtype_str(q.dtype)).unwrap(),
        layout = serde_json::to_string(layout_str(q.layout)).unwrap(),
        address = q.address.as_str(),
        phase_state_digest = serde_json::to_string(q.phase_state_digest.as_str()).unwrap(),
    );
    let page_res = surreal.raw_sql(&page_sql)?;
    let page_id_full = page_res
        .first()
        .and_then(Value::as_array)
        .and_then(|rows| rows.first())
        .and_then(|row| row.get("id"))
        .and_then(Value::as_str)
        .ok_or_else(|| "cache miss: no matching active TensorPage".to_string())?;

    let raw_id = page_id_full
        .strip_prefix("transformer_kv_page:")
        .unwrap_or(page_id_full)
        .trim_matches('`');
    let record_id = RecordId::parse(raw_id)?;
    store.fetch_tensor_payload(&record_id)
}

fn flip_first_hex_nibble(digest: &Digest) -> Digest {
    let mut s = digest.as_str().to_string();
    let replacement = if s.starts_with('0') { "1" } else { "0" };
    s.replace_range(0..1, replacement);
    Digest::parse(s).expect("flipped digest must remain valid hex")
}

fn process_rss_bytes(pid: u32) -> u64 {
    let output = Command::new("ps")
        .args(["-o", "rss=", "-p", &pid.to_string()])
        .output();
    match output {
        Ok(out) => {
            String::from_utf8_lossy(&out.stdout)
                .trim()
                .parse::<u64>()
                .unwrap_or(0)
                * 1024
        }
        Err(_) => 0,
    }
}

fn runtime_manifest_dir() -> PathBuf {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if crate_dir.join("manifests").is_dir() {
        crate_dir
    } else {
        crate_dir.join("../globalmind-runtime")
    }
}

#[test]
#[cfg_attr(
    feature = "live-forward-model",
    ignore = "explicit real Qwen Layer-3 harness smoke"
)]
fn tensor_kv_qualification_end_to_end() -> Result<(), String> {
    let suite_start = Instant::now();
    let manifest_dir = runtime_manifest_dir();
    let repo_root = manifest_dir
        .parent()
        .ok_or_else(|| "missing repo root".to_string())?;

    // Verify contract files and model manifest on disk
    let manifest_path = manifest_dir
        .join("manifests")
        .join("qwen36-layer3-candidate-v2.manifest");
    let manifest_path_str = manifest_path
        .to_str()
        .ok_or_else(|| "invalid manifest path".to_string())?;
    let t_manifest_start = Instant::now();
    eprintln!("Q1.5 phase: verify complete model roster (streaming SHA-256)");
    let parsed_manifest = Manifest::load(manifest_path_str)?;
    let verified_manifest = parsed_manifest.verify_artifacts()?;
    let manifest_verify_dur = t_manifest_start.elapsed();
    eprintln!("Q1.5 phase: roster verified in {manifest_verify_dur:?}");
    let model_entry = verified_manifest
        .models
        .first()
        .ok_or_else(|| "manifest has no models".to_string())?;
    let model_digest = model_entry.roster_digest.clone();

    let tppe_contract_bytes = fs::read(
        manifest_dir
            .join("contracts")
            .join("tppe-layer3-candidate-v2.contract"),
    )
    .map_err(|e| format!("read tppe contract: {e}"))?;
    let tppe_contract_file_digest = Digest::sha256(&tppe_contract_bytes);

    let tppe_config = TppeConfig {
        law: RadialLaw::LinearLogV1,
        radial_start: 0.05,
        radial_span: 0.12,
        omega_fast: 0.19,
        omega_slow: 0.07,
        coupling_fast: 0.08,
        coupling_slow: 0.04,
    }
    .validate()?;
    let tppe_configuration_digest = verified_manifest.tppe_configuration_digest.clone();

    let schema_0006_bytes = fs::read(
        repo_root
            .join("hypermind-nested-kv")
            .join("schema")
            .join("0006_immutable_atom_address.surql"),
    )
    .map_err(|e| format!("read 0006 schema: {e}"))?;
    let address_contract_digest = Digest::sha256(&schema_0006_bytes);

    let schema_0004_bytes = fs::read(
        repo_root
            .join("hypermind-nested-kv")
            .join("schema")
            .join("0004_transformer_kv_tensor_pages.surql"),
    )
    .map_err(|e| format!("read 0004 schema: {e}"))?;
    let schema_0009_bytes = fs::read(
        repo_root
            .join("hypermind-nested-kv")
            .join("schema")
            .join("0009_immutable_tensor_kv.surql"),
    )
    .map_err(|e| format!("read 0009 schema: {e}"))?;
    let mut tensor_contract_buf = Vec::new();
    tensor_contract_buf.extend_from_slice(&schema_0004_bytes);
    tensor_contract_buf.extend_from_slice(&schema_0009_bytes);
    let tensor_contract_digest = Digest::sha256(&tensor_contract_buf);

    // Load real Qwen3.6 Layer 3 + Final Norm + LM Head weights directly from binary safetensors
    let t_load_start = Instant::now();
    eprintln!("Q1.5 phase: load Layer-3 model weights");
    let mut model = Qwen36Layer3Weights::load()?;
    let model_load_dur = t_load_start.elapsed();
    eprintln!("Q1.5 phase: model weights loaded in {model_load_dur:?}");
    let rss_after_model_load = process_rss_bytes(std::process::id());

    // Real token tape (4 prefix tokens + 1 decode continuation token)
    let prefix_tokens: [u32; 4] = [151_644, 872, 198, 12_801];
    let decode_token: u32 = 3_582;
    let mut prefix_bytes = Vec::with_capacity(prefix_tokens.len() * 4);
    for &t in &prefix_tokens {
        prefix_bytes.extend_from_slice(&t.to_le_bytes());
    }
    let prefix_digest = Digest::sha256(&prefix_bytes);
    let mut request_bytes = prefix_bytes.clone();
    request_bytes.extend_from_slice(&decode_token.to_le_bytes());
    let request_digest = Digest::sha256(&request_bytes);

    // Run uninterrupted real forward pass
    let t_fwd_start = Instant::now();
    eprintln!("Q1.5 phase: uninterrupted Layer-3 forward");
    let trace = model.run_uninterrupted_forward(&prefix_tokens, decode_token, &tppe_config)?;
    let uninterrupted_fwd_dur = t_fwd_start.elapsed();
    eprintln!("Q1.5 phase: forward completed in {uninterrupted_fwd_dur:?}");

    // Start isolated SurrealDB 3.3.5 instance and verify schema contract
    eprintln!("Q1.5 phase: boot isolated SurrealDB and apply schema");
    let (mut surreal, surreal_boot_dur) = IsolatedSurreal::start_with_schema()?;
    eprintln!("Q1.5 phase: isolated SurrealDB ready in {surreal_boot_dur:?}");
    let store = surreal.connect_store()?;
    store.verify_contract()?;
    let surreal_pid_before_restart = surreal.pid().unwrap_or(0);
    let surreal_rss_before_restart = process_rss_bytes(surreal_pid_before_restart);

    // =========================================================================
    // GATE 1: Create exact TensorSession with model, prefix, tensor, TPPE, and
    //         address contract digests.
    // =========================================================================
    let workspace_id = RecordId::parse("globalmind")?;
    let session_id = RecordId::parse("kv_session_qwen36_layer3_qual_001")?;
    let geometry_space_id = RecordId::parse("h3_t2_r12_v1")?;

    let session = TensorSession {
        id: session_id.clone(),
        workspace: workspace_id.clone(),
        model_id: MODEL_ID.into(),
        model_revision: MODEL_REVISION.into(),
        model_digest: model_digest.clone(),
        request_digest: request_digest.clone(),
        prefix_digest: prefix_digest.clone(),
        dtype: TensorDtype::F32,
        layer_count: NUM_LAYERS,
        kv_head_count: KV_HEADS as u32,
        head_dim: HEAD_DIM as u32,
        geometry_space: geometry_space_id.clone(),
        tppe_version: TPE_VERSION.into(),
        tppe_configuration_digest: tppe_configuration_digest.clone(),
        address_contract_digest: address_contract_digest.clone(),
        tensor_contract_digest: tensor_contract_digest.clone(),
    };
    let t_gate1 = Instant::now();
    store.create_tensor_session(&session)?;
    let gate1_dur = t_gate1.elapsed();

    // =========================================================================
    // GATE 2: Write actual TensorPage records with correct shape/dtype/layout
    //         and SHA-256 digests.
    // =========================================================================
    let address_id = RecordId::parse("addr_qwen36_l3_prefix4_001")?;
    let mut page_ids = Vec::with_capacity(KV_HEADS);
    let mut expected_key_bytes = Vec::with_capacity(KV_HEADS);
    let mut expected_val_bytes = Vec::with_capacity(KV_HEADS);
    let mut expected_key_digests = Vec::with_capacity(KV_HEADS);
    let mut expected_val_digests = Vec::with_capacity(KV_HEADS);
    let mut gate2_write_durs = Vec::with_capacity(KV_HEADS);

    for kv_h in 0..KV_HEADS {
        let key_bytes = f32_slice_to_le_bytes(&trace.prefix_k_by_head[kv_h]);
        let val_bytes = f32_slice_to_le_bytes(&trace.prefix_v_by_head[kv_h]);
        assert_eq!(key_bytes.len(), prefix_tokens.len() * HEAD_DIM * 4);
        assert_eq!(val_bytes.len(), prefix_tokens.len() * HEAD_DIM * 4);

        let key_digest = Digest::sha256(&key_bytes);
        let val_digest = Digest::sha256(&val_bytes);
        let mut receipt_preimage = Vec::new();
        receipt_preimage.extend_from_slice(session_id.as_str().as_bytes());
        receipt_preimage.extend_from_slice(&(TARGET_FULL_ATTN_LAYER).to_le_bytes());
        receipt_preimage.extend_from_slice(&(kv_h as u32).to_le_bytes());
        receipt_preimage.extend_from_slice(key_digest.as_str().as_bytes());
        receipt_preimage.extend_from_slice(val_digest.as_str().as_bytes());
        receipt_preimage.extend_from_slice(trace.phase_state_digest.as_str().as_bytes());
        let receipt_digest = Digest::sha256(&receipt_preimage);

        let page_id = RecordId::parse(format!("kv_page_qwen36_l3_h{kv_h}_0_4_r1"))?;
        let page = TensorPage {
            id: page_id.clone(),
            workspace: workspace_id.clone(),
            session: session_id.clone(),
            layer: TARGET_FULL_ATTN_LAYER,
            kv_head: kv_h as u32,
            token_start: 0,
            token_end: prefix_tokens.len() as u64,
            tensor_shape: [prefix_tokens.len() as u32, HEAD_DIM as u32],
            dtype: TensorDtype::F32,
            layout: TensorLayout::TokenMajor,
            key_tensor: key_bytes.clone(),
            value_tensor: val_bytes.clone(),
            key_digest: key_digest.clone(),
            value_digest: val_digest.clone(),
            address: address_id.clone(),
            trajectory_atoms: Vec::new(),
            well: None,
            trajectory: None,
            phase_state_digest: trace.phase_state_digest.clone(),
            receipt_digest,
            revision: 1,
        };
        let t_write = Instant::now();
        store.put_tensor_page(&page)?;
        gate2_write_durs.push(t_write.elapsed());

        page_ids.push(page_id);
        expected_key_bytes.push(key_bytes);
        expected_val_bytes.push(val_bytes);
        expected_key_digests.push(key_digest);
        expected_val_digests.push(val_digest);
    }

    #[cfg(feature = "live-forward-model")]
    surreal.raw_sql(&format!(
        "CREATE atom:`kv_qual_prefix` CONTENT {{ workspace: workspace:`globalmind`, external_id: 'kv_qual_prefix', content_kind: 'text', content_text: 'frozen Qwen prefix', content_uri: NONE, content_digest: '{}', domain: 'forward-smoke', rank: 0, salience: 1.0, provenance: 'real-layer3-qualification', taint: 'none', pinned: false, current_address: NONE }} RETURN NONE; \
         CREATE atom_address:`{}` CONTENT {{ workspace: workspace:`globalmind`, atom: atom:`kv_qual_prefix`, h3_lorentz: [1.0, 0.0, 0.0, 0.0], t2_phase: [0.0, 0.0], geometry_model: 'lorentz-t2', geometry_version: 'v1', revision: 1, active: true }} RETURN NONE;",
        prefix_digest.as_str(), address_id.as_str()
    ))?;

    // =========================================================================
    // GATE 3: Restart SurrealDB / runtime.
    // =========================================================================
    drop(store);
    let gate3_restart_dur = surreal.restart()?;
    let store_after_restart = surreal.connect_store()?;
    store_after_restart.verify_contract()?;
    let surreal_pid_after_restart = surreal.pid().unwrap_or(0);
    let surreal_rss_after_restart = process_rss_bytes(surreal_pid_after_restart);

    // =========================================================================
    // GATE 4: Fetch bytes and prove byte-for-byte digest equality.
    // =========================================================================
    let mut fetched_payloads = Vec::with_capacity(KV_HEADS);
    let mut gate4_fetch_durs = Vec::with_capacity(KV_HEADS);
    for kv_h in 0..KV_HEADS {
        let t_fetch = Instant::now();
        let payload = store_after_restart.fetch_tensor_payload(&page_ids[kv_h])?;
        gate4_fetch_durs.push(t_fetch.elapsed());

        let fetched_key_digest = Digest::sha256(&payload.key);
        let fetched_val_digest = Digest::sha256(&payload.value);
        assert_eq!(
            fetched_key_digest, expected_key_digests[kv_h],
            "key SHA-256 mismatch after SurrealDB restart for head {kv_h}"
        );
        assert_eq!(
            fetched_val_digest, expected_val_digests[kv_h],
            "value SHA-256 mismatch after SurrealDB restart for head {kv_h}"
        );
        assert_eq!(
            payload.key.as_ref(),
            expected_key_bytes[kv_h].as_ref(),
            "key byte-for-byte mismatch after SurrealDB restart for head {kv_h}"
        );
        assert_eq!(
            payload.value.as_ref(),
            expected_val_bytes[kv_h].as_ref(),
            "value byte-for-byte mismatch after SurrealDB restart for head {kv_h}"
        );
        fetched_payloads.push(payload);
    }
    let fetched_pair: [TensorPayload; KV_HEADS] = fetched_payloads
        .try_into()
        .map_err(|_| "expected 2 KV head payloads".to_string())?;

    // =========================================================================
    // GATE 5: Inject fetched page into the same model and prove decode/logit parity.
    // =========================================================================
    let t_inject_start = Instant::now();
    let (injected_hidden_out, injected_logits) = model.run_injected_decode(
        &fetched_pair,
        prefix_tokens.len(),
        decode_token,
        &tppe_config,
    )?;
    let gate5_inject_dur = t_inject_start.elapsed();

    assert_eq!(injected_hidden_out.len(), trace.decode_hidden_out.len());
    assert_eq!(injected_logits.len(), trace.decode_logits.len());

    let mut max_hidden_abs_diff = 0.0f32;
    for (a, b) in trace.decode_hidden_out.iter().zip(&injected_hidden_out) {
        let diff = (a - b).abs();
        if diff > max_hidden_abs_diff {
            max_hidden_abs_diff = diff;
        }
        assert_eq!(
            a.to_bits(),
            b.to_bits(),
            "layer 3 decode output float bits diverged"
        );
    }

    let mut max_logit_abs_diff = 0.0f32;
    for (a, b) in trace.decode_logits.iter().zip(&injected_logits) {
        let diff = (a - b).abs();
        if diff > max_logit_abs_diff {
            max_logit_abs_diff = diff;
        }
        assert_eq!(
            a.to_bits(),
            b.to_bits(),
            "lm_head logit float bits diverged"
        );
    }

    let ref_logits_bytes = f32_slice_to_le_bytes(&trace.decode_logits);
    let inj_logits_bytes = f32_slice_to_le_bytes(&injected_logits);
    let ref_logits_digest = Digest::sha256(&ref_logits_bytes);
    let inj_logits_digest = Digest::sha256(&inj_logits_bytes);
    assert_eq!(ref_logits_digest, inj_logits_digest);

    let (ref_top1_token, ref_top1_logit) = argmax_f32(&trace.decode_logits);
    let (inj_top1_token, inj_top1_logit) = argmax_f32(&injected_logits);
    assert_eq!(ref_top1_token, inj_top1_token);
    assert_eq!(ref_top1_logit.to_bits(), inj_top1_logit.to_bits());

    // Negative sensitivity control: perturb 1 float in head 0 key cache and verify logits change
    let mut perturbed_pair = fetched_pair.clone();
    let mut perturbed_k0 = f32_vec_from_le_bytes(&perturbed_pair[0].key)?;
    perturbed_k0[0] += 0.25;
    perturbed_pair[0].key = f32_slice_to_le_bytes(&perturbed_k0);
    let (_perturbed_hidden, perturbed_logits) = model.run_injected_decode(
        &perturbed_pair,
        prefix_tokens.len(),
        decode_token,
        &tppe_config,
    )?;
    let perturbed_logits_digest = Digest::sha256(&f32_slice_to_le_bytes(&perturbed_logits));
    assert_ne!(
        perturbed_logits_digest, ref_logits_digest,
        "decode logits must be causally sensitive to injected KV tensor bytes"
    );

    // =========================================================================
    // GATE 6: Change one identity field at a time and prove a cache miss/failure.
    // =========================================================================
    let base_query = ExactKvIdentityQuery {
        workspace: workspace_id.clone(),
        model_id: MODEL_ID.into(),
        model_revision: MODEL_REVISION.into(),
        model_digest: model_digest.clone(),
        request_digest: request_digest.clone(),
        prefix_digest: prefix_digest.clone(),
        tensor_contract_digest: tensor_contract_digest.clone(),
        tppe_version: TPE_VERSION.into(),
        tppe_configuration_digest: tppe_configuration_digest.clone(),
        address_contract_digest: address_contract_digest.clone(),
        geometry_space: geometry_space_id.clone(),
        layer: TARGET_FULL_ATTN_LAYER,
        kv_head: 0,
        token_start: 0,
        token_end: prefix_tokens.len() as u64,
        tensor_shape: [prefix_tokens.len() as u32, HEAD_DIM as u32],
        dtype: TensorDtype::F32,
        layout: TensorLayout::TokenMajor,
        address: address_id.clone(),
        phase_state_digest: trace.phase_state_digest.clone(),
    };

    // Baseline exact identity query must HIT
    let baseline_hit = lookup_tensor_page_strict(&surreal, &store_after_restart, &base_query)?;
    let base_runtime_query = runtime_exact_identity(
        &base_query,
        &session_id,
        &page_ids[0],
        &expected_key_digests[0],
        &expected_val_digests[0],
    );
    let runtime_hit = store_after_restart.fetch_tensor_payload_exact(&base_runtime_query)?;
    assert_eq!(
        runtime_hit, baseline_hit,
        "runtime exact lookup must match qualification witness"
    );
    assert_eq!(
        Digest::sha256(&baseline_hit.key),
        expected_key_digests[0],
        "baseline exact identity query must hit head 0"
    );

    #[cfg(feature = "live-forward-model")]
    {
        use globalmind_runtime::retrieval::{RetrievalConfig, RetrievalMetric};
        use rgah::globalmind_bridge::{GlobalMindBridge, McpSurrealTarget};

        let retrieval = RetrievalConfig {
            endpoint: format!("http://127.0.0.1:{}", surreal.port),
            namespace: surreal.namespace.clone(),
            database: surreal.database.clone(),
            username: surreal.username.clone(),
            password: surreal.password.clone(),
            workspace: workspace_id.clone(),
            candidate_budget: 8,
            metric: RetrievalMetric::Hyperbolic,
        };
        let mcp_target = McpSurrealTarget {
            mode: "on".into(),
            endpoint: retrieval.endpoint.clone(),
            namespace: retrieval.namespace.clone(),
            database: retrieval.database.clone(),
        };
        let bridge = GlobalMindBridge::connect_at_mcp_boundary(retrieval, &mcp_target)?;
        let mut bridge_payloads = Vec::with_capacity(KV_HEADS);
        for head in 0..KV_HEADS {
            let mut query = base_query.clone();
            query.kv_head = head as u32;
            let identity = runtime_exact_identity(
                &query,
                &session_id,
                &page_ids[head],
                &expected_key_digests[head],
                &expected_val_digests[head],
            );
            let prepared = bridge.prepare_forward_recall(
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 0.0],
                "qwen-layer3-real-forward",
                1,
                &identity,
            )?;
            if !prepared.neighborhood_gmrc.starts_with(b"GMRC") {
                return Err("bridge returned no binary GlobalMind neighborhood".into());
            }
            bridge_payloads.push(prepared.tensor);
        }
        let bridge_pair: [TensorPayload; KV_HEADS] = bridge_payloads
            .try_into()
            .map_err(|_| "bridge returned wrong KV head count".to_string())?;
        let (bridge_hidden, bridge_logits) = model.run_injected_decode(
            &bridge_pair,
            prefix_tokens.len(),
            decode_token,
            &tppe_config,
        )?;
        if bridge_hidden.len() != trace.decode_hidden_out.len()
            || bridge_logits.len() != trace.decode_logits.len()
            || bridge_hidden
                .iter()
                .zip(&trace.decode_hidden_out)
                .any(|(a, b)| a.to_bits() != b.to_bits())
            || bridge_logits
                .iter()
                .zip(&trace.decode_logits)
                .any(|(a, b)| a.to_bits() != b.to_bits())
        {
            return Err(
                "bridge-injected Layer-3 forward diverged from uninterrupted control".into(),
            );
        }
        let mut drifted = base_runtime_query.clone();
        drifted.model_digest = Digest::sha256(b"different-model");
        if bridge
            .prepare_forward_recall(
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 0.0],
                "qwen-layer3-real-forward",
                1,
                &drifted,
            )
            .is_ok()
        {
            return Err("bridge admitted a changed model digest".into());
        }
        eprintln!(
            "REAL HARNESS BRIDGE FORWARD PASS: Layer-3 hidden and 248320 logits bit-exact; both K/V heads consumed"
        );
    }

    let t_gate6 = Instant::now();
    let mut single_field_miss_receipts: Vec<(String, String)> = Vec::new();

    let mutations: [IdentityMutator; 20] = [
        ("workspace", |q| {
            q.workspace = RecordId::parse("other_workspace").unwrap()
        }),
        ("model_id", |q| q.model_id = "other/Model-35B".into()),
        ("model_revision", |q| {
            q.model_revision = "local-frozen-9999-99-99".into()
        }),
        ("model_digest", |q| {
            q.model_digest = flip_first_hex_nibble(&q.model_digest)
        }),
        ("request_digest", |q| {
            q.request_digest = flip_first_hex_nibble(&q.request_digest)
        }),
        ("prefix_digest", |q| {
            q.prefix_digest = flip_first_hex_nibble(&q.prefix_digest)
        }),
        ("tensor_contract_digest", |q| {
            q.tensor_contract_digest = flip_first_hex_nibble(&q.tensor_contract_digest)
        }),
        ("tppe_version", |q| {
            q.tppe_version = "tppe-reference-v999".into()
        }),
        ("tppe_configuration_digest", |q| {
            q.tppe_configuration_digest = flip_first_hex_nibble(&q.tppe_configuration_digest)
        }),
        ("address_contract_digest", |q| {
            q.address_contract_digest = flip_first_hex_nibble(&q.address_contract_digest)
        }),
        ("geometry_space", |q| {
            q.geometry_space = RecordId::parse("h3_t2_r12_v999").unwrap()
        }),
        ("layer", |q| q.layer = TARGET_FULL_ATTN_LAYER + 1),
        ("kv_head", |q| q.kv_head = 99),
        ("token_start", |q| q.token_start = 1),
        ("token_end", |q| q.token_end += 1),
        ("tensor_shape", |q| {
            q.tensor_shape = [q.tensor_shape[0], (HEAD_DIM as u32) / 2]
        }),
        ("dtype", |q| q.dtype = TensorDtype::Bf16),
        ("layout", |q| q.layout = TensorLayout::HeadDimMajor),
        ("address", |q| {
            q.address = RecordId::parse("addr_wrong_001").unwrap()
        }),
        ("phase_state_digest", |q| {
            q.phase_state_digest = flip_first_hex_nibble(&q.phase_state_digest)
        }),
    ];

    for (field_name, mutate_fn) in mutations {
        let mut mutated = base_query.clone();
        mutate_fn(&mut mutated);
        let runtime_mutated = runtime_exact_identity(
            &mutated,
            &session_id,
            &page_ids[0],
            &expected_key_digests[0],
            &expected_val_digests[0],
        );
        if store_after_restart
            .fetch_tensor_payload_exact(&runtime_mutated)
            .is_ok()
        {
            return Err(format!(
                "runtime exact lookup accepted mutated identity field `{field_name}`"
            ));
        }
        match lookup_tensor_page_strict(&surreal, &store_after_restart, &mutated) {
            Ok(_) => {
                return Err(format!(
                    "Gate 6 violation: mutating identity field `{field_name}` unexpectedly hit cache"
                ));
            }
            Err(reason) => {
                single_field_miss_receipts.push((field_name.to_string(), reason));
            }
        }
    }

    // Write-time & immutability fail-closed checks on GlobalMindStore + SurrealDB 3.3.5
    let valid_template = TensorPage {
        id: RecordId::parse("kv_page_negative_probe")?,
        workspace: workspace_id.clone(),
        session: session_id.clone(),
        layer: TARGET_FULL_ATTN_LAYER,
        kv_head: 0,
        token_start: 0,
        token_end: prefix_tokens.len() as u64,
        tensor_shape: [prefix_tokens.len() as u32, HEAD_DIM as u32],
        dtype: TensorDtype::F32,
        layout: TensorLayout::TokenMajor,
        key_tensor: expected_key_bytes[0].clone(),
        value_tensor: expected_val_bytes[0].clone(),
        key_digest: expected_key_digests[0].clone(),
        value_digest: expected_val_digests[0].clone(),
        address: address_id.clone(),
        trajectory_atoms: Vec::new(),
        well: None,
        trajectory: None,
        phase_state_digest: trace.phase_state_digest.clone(),
        receipt_digest: expected_key_digests[0].clone(),
        revision: 2,
    };

    let mut write_rejection_receipts: Vec<(String, String)> = Vec::new();
    let write_mutations: [PageMutator; 9] = [
        ("write_session_missing", |p| {
            p.session = RecordId::parse("missing_session_999").unwrap()
        }),
        ("write_workspace_mismatch", |p| {
            p.workspace = RecordId::parse("wrong_workspace").unwrap()
        }),
        ("write_dtype_mismatch", |p| {
            p.dtype = TensorDtype::Bf16;
            p.tensor_shape = [p.tensor_shape[0], (HEAD_DIM * 2) as u32];
        }),
        ("write_layer_out_of_bounds", |p| p.layer = NUM_LAYERS),
        ("write_kv_head_out_of_bounds", |p| {
            p.kv_head = KV_HEADS as u32
        }),
        ("write_key_digest_mismatch", |p| {
            p.key_digest = flip_first_hex_nibble(&p.key_digest)
        }),
        ("write_value_digest_mismatch", |p| {
            p.value_digest = flip_first_hex_nibble(&p.value_digest)
        }),
        ("write_shape_byte_mismatch", |p| {
            p.tensor_shape = [p.tensor_shape[0], (HEAD_DIM as u32) - 1]
        }),
        ("write_duplicate_coordinate_revision", |p| {
            p.id = RecordId::parse("kv_page_dup_coord_probe").unwrap();
            p.revision = 1;
        }),
    ];

    for (name, mut_fn) in write_mutations {
        let mut bad_page = valid_template.clone();
        mut_fn(&mut bad_page);
        match store_after_restart.put_tensor_page(&bad_page) {
            Ok(()) => {
                return Err(format!(
                    "Gate 6 write violation: invalid page `{name}` was admitted"
                ));
            }
            Err(err) => {
                write_rejection_receipts.push((name.to_string(), err));
            }
        }
    }

    // Immutability enforcement check on READONLY fields in SurrealDB 3.3.5
    let readonly_session_err = surreal
        .raw_sql(&format!(
            "UPDATE transformer_kv_session:`{}` SET model_digest = '{}';",
            session_id.as_str(),
            flip_first_hex_nibble(&model_digest).as_str()
        ))
        .expect_err("READONLY session.model_digest update must fail");
    write_rejection_receipts.push(("readonly_session_model_digest".into(), readonly_session_err));

    let readonly_page_err = surreal
        .raw_sql(&format!(
            "UPDATE transformer_kv_page:`{}` SET key_digest = '{}';",
            page_ids[0].as_str(),
            flip_first_hex_nibble(&expected_key_digests[0]).as_str()
        ))
        .expect_err("READONLY page.key_digest update must fail");
    write_rejection_receipts.push(("readonly_page_key_digest".into(), readonly_page_err));

    let gate6_dur = t_gate6.elapsed();
    let total_suite_dur = suite_start.elapsed();
    let final_test_rss_bytes = process_rss_bytes(std::process::id());

    // =========================================================================
    // GATE 7: Write plain-text qualification report with real hardware receipts.
    // =========================================================================
    let report_dir = repo_root.join("results").join("tensor-kv");
    fs::create_dir_all(&report_dir).map_err(|e| format!("create report dir: {e}"))?;
    let report_path = if cfg!(feature = "live-forward-model") {
        report_dir.join("tensor-kv-harness-bridge-report.txt")
    } else {
        report_dir.join("tensor-kv-qualification-report.txt")
    };

    let mut report = String::new();
    use std::fmt::Write as _;
    writeln!(
        &mut report,
        "GLOBALMIND LANE B — REAL TENSOR K/V QUALIFICATION REPORT"
    )
    .unwrap();
    writeln!(
        &mut report,
        "========================================================"
    )
    .unwrap();
    writeln!(&mut report, "status: PASS (all 7 required gates passed)").unwrap();
    #[cfg(feature = "live-forward-model")]
    writeln!(&mut report, "harness_bridge_forward: PASS (real Layer-3; both exact K/V heads consumed; hidden and 248320 logits bit-exact)").unwrap();
    #[cfg(feature = "live-forward-model")]
    writeln!(&mut report, "mcp_authority_preflight: PASS (declared MCP target equals isolated GlobalMind bridge target; MCP child transport not exercised)").unwrap();
    writeln!(&mut report, "canonical_workspace: {}", repo_root.display()).unwrap();
    writeln!(&mut report, "model_directory: {MODEL_DIR}").unwrap();
    writeln!(&mut report, "model_id: {MODEL_ID}").unwrap();
    writeln!(&mut report, "model_revision: {MODEL_REVISION}").unwrap();
    writeln!(&mut report, "model_roster_digest: {model_digest}").unwrap();
    writeln!(
        &mut report,
        "manifest_input_digest: {}",
        verified_manifest.input_digest
    )
    .unwrap();
    writeln!(
        &mut report,
        "tppe_contract_file_sha256: {tppe_contract_file_digest}"
    )
    .unwrap();
    writeln!(&mut report, "tppe_version: {TPE_VERSION}").unwrap();
    writeln!(
        &mut report,
        "tppe_configuration_digest: {tppe_configuration_digest}"
    )
    .unwrap();
    writeln!(
        &mut report,
        "address_contract_digest: {address_contract_digest}"
    )
    .unwrap();
    writeln!(
        &mut report,
        "tensor_contract_digest: {tensor_contract_digest}"
    )
    .unwrap();
    writeln!(&mut report, "prefix_tokens: {:?}", prefix_tokens).unwrap();
    writeln!(&mut report, "decode_token: {decode_token}").unwrap();
    writeln!(&mut report, "prefix_digest: {prefix_digest}").unwrap();
    writeln!(&mut report, "request_digest: {request_digest}").unwrap();
    writeln!(
        &mut report,
        "phase_state_digest: {}",
        trace.phase_state_digest
    )
    .unwrap();
    writeln!(&mut report).unwrap();
    writeln!(&mut report, "ISOLATED SURREALDB 3.3.5 INSTANCE").unwrap();
    writeln!(&mut report, "---------------------------------").unwrap();
    writeln!(&mut report, "endpoint: http://127.0.0.1:{}", surreal.port).unwrap();
    writeln!(&mut report, "namespace: {}", surreal.namespace).unwrap();
    writeln!(&mut report, "database: {}", surreal.database).unwrap();
    writeln!(
        &mut report,
        "quarantined_core_touched: false (port 8000 untouched)"
    )
    .unwrap();
    writeln!(
        &mut report,
        "pid_before_restart: {surreal_pid_before_restart}"
    )
    .unwrap();
    writeln!(
        &mut report,
        "pid_after_restart: {surreal_pid_after_restart}"
    )
    .unwrap();
    writeln!(&mut report).unwrap();
    writeln!(&mut report, "GATE RECEIPTS").unwrap();
    writeln!(&mut report, "-------------").unwrap();
    writeln!(
        &mut report,
        "Gate 1 (Create TensorSession): PASS | session_id={} | latency_ms={:.3}",
        session_id,
        gate1_dur.as_secs_f64() * 1000.0
    )
    .unwrap();
    for kv_h in 0..KV_HEADS {
        writeln!(
            &mut report,
            "Gate 2 (Write TensorPage head={kv_h}): PASS | page_id={} | shape=[{}, {}] | dtype=f32 | layout=token_major | key_bytes={} | value_bytes={} | key_sha256={} | value_sha256={} | latency_ms={:.3}",
            page_ids[kv_h],
            prefix_tokens.len(),
            HEAD_DIM,
            expected_key_bytes[kv_h].len(),
            expected_val_bytes[kv_h].len(),
            expected_key_digests[kv_h],
            expected_val_digests[kv_h],
            gate2_write_durs[kv_h].as_secs_f64() * 1000.0
        )
        .unwrap();
    }
    writeln!(
        &mut report,
        "Gate 3 (Restart SurrealDB & Reverify Contract): PASS | pid_before={} -> pid_after={} | restart_latency_ms={:.3}",
        surreal_pid_before_restart,
        surreal_pid_after_restart,
        gate3_restart_dur.as_secs_f64() * 1000.0
    )
    .unwrap();
    for (kv_h, fetch_dur) in gate4_fetch_durs.iter().enumerate().take(KV_HEADS) {
        writeln!(
            &mut report,
            "Gate 4 (Fetch & Byte-for-Byte Digest Equality head={kv_h}): PASS | key_sha256_match=true | value_sha256_match=true | byte_equal=true | fetch_latency_ms={:.3}",
            fetch_dur.as_secs_f64() * 1000.0
        )
        .unwrap();
    }
    writeln!(
        &mut report,
        "Gate 5 (Inject Fetched KV & Prove Decode/Logit Parity): PASS | layer=3 | vocab_size={VOCAB_SIZE} | max_hidden_abs_diff={max_hidden_abs_diff:.8} | max_logit_abs_diff={max_logit_abs_diff:.8} | reference_logits_sha256={ref_logits_digest} | injected_logits_sha256={inj_logits_digest} | perturbed_control_logits_sha256={perturbed_logits_digest} | top1_token_id={ref_top1_token} | top1_logit={ref_top1_logit:.6} | inject_decode_latency_ms={:.3}",
        gate5_inject_dur.as_secs_f64() * 1000.0
    )
    .unwrap();
    writeln!(
        &mut report,
        "Gate 6 (Single-Field Identity Mutations & Write/Readonly Rejections): PASS | lookup_identity_misses={}/{} | write_and_readonly_rejections={}/{} | latency_ms={:.3}",
        single_field_miss_receipts.len(),
        single_field_miss_receipts.len(),
        write_rejection_receipts.len(),
        write_rejection_receipts.len(),
        gate6_dur.as_secs_f64() * 1000.0
    )
    .unwrap();
    for (field, err) in &single_field_miss_receipts {
        writeln!(&mut report, "  - lookup_mutation[{field}]: MISS ({err})").unwrap();
    }
    for (check, err) in &write_rejection_receipts {
        writeln!(
            &mut report,
            "  - write_or_readonly_check[{check}]: REJECTED ({err})"
        )
        .unwrap();
    }
    writeln!(&mut report).unwrap();
    writeln!(
        &mut report,
        "GATE 7 — REAL HARDWARE LATENCY & MEMORY MEASUREMENTS"
    )
    .unwrap();
    writeln!(
        &mut report,
        "----------------------------------------------------"
    )
    .unwrap();
    writeln!(
        &mut report,
        "manifest_rehash_verify_ms: {:.3}",
        manifest_verify_dur.as_secs_f64() * 1000.0
    )
    .unwrap();
    writeln!(
        &mut report,
        "model_weights_binary_load_ms: {:.3}",
        model_load_dur.as_secs_f64() * 1000.0
    )
    .unwrap();
    writeln!(
        &mut report,
        "uninterrupted_forward_pass_ms: {:.3}",
        uninterrupted_fwd_dur.as_secs_f64() * 1000.0
    )
    .unwrap();
    writeln!(
        &mut report,
        "surreal_isolated_boot_and_schema_ms: {:.3}",
        surreal_boot_dur.as_secs_f64() * 1000.0
    )
    .unwrap();
    writeln!(
        &mut report,
        "session_create_ms: {:.3}",
        gate1_dur.as_secs_f64() * 1000.0
    )
    .unwrap();
    writeln!(
        &mut report,
        "page_write_head0_ms: {:.3}",
        gate2_write_durs[0].as_secs_f64() * 1000.0
    )
    .unwrap();
    writeln!(
        &mut report,
        "page_write_head1_ms: {:.3}",
        gate2_write_durs[1].as_secs_f64() * 1000.0
    )
    .unwrap();
    writeln!(
        &mut report,
        "surreal_restart_ms: {:.3}",
        gate3_restart_dur.as_secs_f64() * 1000.0
    )
    .unwrap();
    writeln!(
        &mut report,
        "page_fetch_head0_ms: {:.3}",
        gate4_fetch_durs[0].as_secs_f64() * 1000.0
    )
    .unwrap();
    writeln!(
        &mut report,
        "page_fetch_head1_ms: {:.3}",
        gate4_fetch_durs[1].as_secs_f64() * 1000.0
    )
    .unwrap();
    writeln!(
        &mut report,
        "injected_decode_forward_pass_ms: {:.3}",
        gate5_inject_dur.as_secs_f64() * 1000.0
    )
    .unwrap();
    writeln!(
        &mut report,
        "gate6_identity_and_immutability_checks_ms: {:.3}",
        gate6_dur.as_secs_f64() * 1000.0
    )
    .unwrap();
    writeln!(
        &mut report,
        "total_qualification_suite_ms: {:.3}",
        total_suite_dur.as_secs_f64() * 1000.0
    )
    .unwrap();
    writeln!(
        &mut report,
        "test_process_rss_after_model_load_bytes: {rss_after_model_load}"
    )
    .unwrap();
    writeln!(
        &mut report,
        "test_process_rss_final_bytes: {final_test_rss_bytes}"
    )
    .unwrap();
    writeln!(
        &mut report,
        "surreal_process_rss_before_restart_bytes: {surreal_rss_before_restart}"
    )
    .unwrap();
    writeln!(
        &mut report,
        "surreal_process_rss_after_restart_bytes: {surreal_rss_after_restart}"
    )
    .unwrap();

    fs::write(&report_path, &report).map_err(|e| format!("write report: {e}"))?;
    println!("{report}");
    Ok(())
}

fn argmax_f32(values: &[f32]) -> (usize, f32) {
    let mut best_idx = 0usize;
    let mut best_val = f32::NEG_INFINITY;
    for (idx, &v) in values.iter().enumerate() {
        if v > best_val {
            best_val = v;
            best_idx = idx;
        }
    }
    (best_idx, best_val)
}

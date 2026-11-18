//! Universal decoder graph admission (candidate, 2026-11-03).
//!
//! This module owns the model-neutral operator graph surface: supported
//! operator names, typed graph admission, and frozen-artifact config/tokenizer
//! ingestion. It deliberately contains NO Glimmer-only forward path. Glimmer
//! is admitted only as a graph profile using the operator families below;
//! unknown operators fail closed before any weight allocation or execution.
//!
//! Honest status: this is DECLARE + GATE scaffolding. No full forward exists
//! yet, and no receipt may claim one from this module alone.

use serde_json::Value;
use sha2::{Digest as ShaDigest, Sha256};
use std::collections::BTreeSet;
use std::fs;

/// Operator families named in UNIVERSAL-DECODER-DESIGN.md. A graph node's
/// op must be one of these exact names, or admission is rejected.
pub const SUPPORTED_OPS: &[&str] = &[
    "embedding_lookup",
    "affine_mixed_bit",
    "normalization",
    "gated_mlp",
    "qk_normalization",
    "position_transform", // trained RoPE or NoPE; TPPE is separate
    "causal_attention",   // sliding (RoPE) or full (NoPE)
    "residual",
    "final_projection",
    "logit_cap",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphNode {
    pub id: String,
    pub op: String,
    pub input_shapes: Vec<Vec<u64>>,
    pub output_shape: Vec<u64>,
    pub tensor_bindings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct OperatorGraph {
    pub nodes: Vec<GraphNode>,
    pub execution_contract_digest: String,
}

#[derive(Debug, Clone)]
pub struct AdmissionError(pub String);

impl std::fmt::Display for AdmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for AdmissionError {}

/// Fail closed: every node op must be in SUPPORTED_OPS.
pub fn admit_graph(nodes: &[GraphNode]) -> Result<(), AdmissionError> {
    if nodes.is_empty() || nodes.len() > 8192 {
        return Err(AdmissionError("graph node count outside 1..=8192".into()));
    }
    let allowed: BTreeSet<&str> = SUPPORTED_OPS.iter().copied().collect();
    let mut ids = BTreeSet::new();
    for n in nodes {
        if !allowed.contains(n.op.as_str()) {
            return Err(AdmissionError(format!("unknown operator: {}", n.op)));
        }
        if n.id.is_empty()
            || n.id.len() > 128
            || !n.id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err(AdmissionError("invalid node id".into()));
        }
        if !ids.insert(n.id.as_str()) {
            return Err(AdmissionError("duplicate node id".into()));
        }
        let valid_shape = |shape: &Vec<u64>| {
            !shape.is_empty()
                && shape.len() <= 6
                && shape.iter().all(|d| (1..=1_000_000).contains(d))
        };
        if !valid_shape(&n.output_shape) || !n.input_shapes.iter().all(valid_shape) {
            return Err(AdmissionError("invalid tensor shape".into()));
        }
        let mut bindings = BTreeSet::new();
        for binding in &n.tensor_bindings {
            if binding.is_empty()
                || binding.len() > 256
                || binding.contains('\0')
                || !bindings.insert(binding)
            {
                return Err(AdmissionError("invalid or duplicate tensor binding".into()));
            }
        }
    }
    Ok(())
}

/// Attest execution contract digest over node ops + shapes + bindings.
/// This is a model-neutral digest: no model name/version is baked in.
pub fn attest_graph(nodes: &[GraphNode]) -> String {
    fn hash_text(h: &mut Sha256, value: &str) {
        h.update((value.len() as u64).to_le_bytes());
        h.update(value.as_bytes());
    }
    let mut h = Sha256::new();
    h.update(b"globalmind-operator-graph-v2\0");
    h.update((nodes.len() as u64).to_le_bytes());
    for n in nodes {
        hash_text(&mut h, &n.id);
        hash_text(&mut h, &n.op);
        h.update((n.input_shapes.len() as u64).to_le_bytes());
        for s in &n.input_shapes {
            h.update((s.len() as u64).to_le_bytes());
            for d in s {
                h.update(d.to_le_bytes());
            }
        }
        h.update((n.output_shape.len() as u64).to_le_bytes());
        for d in &n.output_shape {
            h.update(d.to_le_bytes());
        }
        h.update((n.tensor_bindings.len() as u64).to_le_bytes());
        for t in &n.tensor_bindings {
            hash_text(&mut h, t);
        }
    }
    format!("{:x}", h.finalize())
}

/// A claimed execution-contract digest is 64 lowercase hex chars.
fn is_lower_hex64(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Admit a full operator graph, including its claimed digest.
/// This closes the last gap in `admit_graph` alone: a graph whose
/// `execution_contract_digest` field is forged, empty, or non-hex
/// must fail here, even if every node individually admits.
pub fn admit_operator_graph(graph: &OperatorGraph) -> Result<(), AdmissionError> {
    admit_graph(&graph.nodes)?;
    if !is_lower_hex64(&graph.execution_contract_digest) {
        return Err(AdmissionError(
            "execution-contract digest is not 64 lowercase hex".into(),
        ));
    }
    if attest_graph(&graph.nodes) != graph.execution_contract_digest {
        return Err(AdmissionError(
            "execution-contract digest mismatch".into(),
        ));
    }
    Ok(())
}

/// Build the Glimmer layer-sequence operator graph as an admission
/// profile, not an execution schedule. One node per operator family
/// per layer, with real shapes (hidden 6656, vocab 202048, batch 1)
/// and real safetensors tensor bindings taken from
/// model.safetensors.index.json. Ops with no dedicated weight in
/// the index (qk_normalization, position_transform, residual,
/// logit_cap) carry no bindings rather than an invented one.
/// Layer attention type comes from the config `layer_types` entry:
/// "sliding_attention" or "full_attention", 52 entries, in order.
/// Returns Err on any other layer type or on a length other than 52.
pub fn glimmer_layer_graph(layer_types: &[&str]) -> Result<OperatorGraph, AdmissionError> {
    if layer_types.len() != 52 {
        return Err(AdmissionError(
            "glimmer layer_types length must be 52".into(),
        ));
    }
    let hidden = vec![1u64, 6656];
    let logits = vec![1u64, 202048];
    let mut nodes: Vec<GraphNode> = Vec::new();
    let node = |id: String, op: &str, input: Vec<Vec<u64>>, output: Vec<u64>, bindings: Vec<String>| GraphNode {
        id,
        op: op.to_string(),
        input_shapes: input,
        output_shape: output,
        tensor_bindings: bindings,
    };
    nodes.push(node(
        "embed".to_string(),
        "embedding_lookup",
        vec![vec![1]],
        hidden.clone(),
        vec!["model.embed_tokens.weight".to_string()],
    ));
    for (i, layer_type) in layer_types.iter().enumerate() {
        let attn_id = match *layer_type {
            "sliding_attention" => format!("layer{i}_attention_sliding"),
            "full_attention" => format!("layer{i}_attention_full"),
            other => {
                return Err(AdmissionError(format!(
                    "unknown glimmer layer type: {other}"
                )))
            }
        };
        let base = format!("model.layers.{i}");
        nodes.push(node(
            format!("layer{i}_input_norm"),
            "normalization",
            vec![hidden.clone()],
            hidden.clone(),
            vec![format!("{base}.input_layernorm.weight")],
        ));
        nodes.push(node(
            format!("layer{i}_qk_norm"),
            "qk_normalization",
            vec![hidden.clone()],
            hidden.clone(),
            vec![],
        ));
        nodes.push(node(
            format!("layer{i}_position"),
            "position_transform",
            vec![hidden.clone()],
            hidden.clone(),
            vec![],
        ));
        nodes.push(node(
            attn_id,
            "causal_attention",
            vec![hidden.clone()],
            hidden.clone(),
            vec![
                format!("{base}.self_attn.q_proj.weight"),
                format!("{base}.self_attn.k_proj.weight"),
                format!("{base}.self_attn.v_proj.weight"),
                format!("{base}.self_attn.o_proj.weight"),
            ],
        ));
        nodes.push(node(
            format!("layer{i}_post_norm"),
            "normalization",
            vec![hidden.clone()],
            hidden.clone(),
            vec![format!("{base}.post_attn_norm.weight")],
        ));
        nodes.push(node(
            format!("layer{i}_mlp"),
            "gated_mlp",
            vec![hidden.clone()],
            hidden.clone(),
            vec![
                format!("{base}.mlp.gate_proj.weight"),
                format!("{base}.mlp.up_proj.weight"),
                format!("{base}.mlp.down_proj.weight"),
            ],
        ));
        nodes.push(node(
            format!("layer{i}_residual"),
            "residual",
            vec![hidden.clone()],
            hidden.clone(),
            vec![],
        ));
    }
    nodes.push(node(
        "final_norm".to_string(),
        "normalization",
        vec![hidden.clone()],
        hidden.clone(),
        vec!["model.norm.weight".to_string()],
    ));
    nodes.push(node(
        "lm_head".to_string(),
        "final_projection",
        vec![hidden.clone()],
        logits.clone(),
        vec!["lm_head.weight".to_string()],
    ));
    nodes.push(node(
        "logit_cap".to_string(),
        "logit_cap",
        vec![logits.clone()],
        logits,
        vec![],
    ));
    let digest = attest_graph(&nodes);
    let graph = OperatorGraph {
        nodes,
        execution_contract_digest: digest,
    };
    admit_operator_graph(&graph)?;
    Ok(graph)
}

/// Ingest and validate frozen Glimmer config. Returns the parsed JSON only
/// after all architecture fields match the frozen contract. Does NOT allocate
/// weights or execute.
pub fn ingest_glimmer_config(path: &str) -> Result<Value, String> {
    let bytes = fs::read(path).map_err(|e| format!("read config: {e}"))?;
    if bytes.len() > 64 * 1024 {
        return Err("config exceeds 64 KiB".into());
    }
    let v: Value = serde_json::from_slice(&bytes).map_err(|e| format!("config json: {e}"))?;
    // Required architecture fields (frozen 2026-11-02).
    let checks = [
        ("model_type", Value::String("muse_glimmer".into())),
        ("num_hidden_layers", Value::from(52u64)),
        ("hidden_size", Value::from(6656u64)),
        ("vocab_size", Value::from(202048u64)),
        ("num_attention_heads", Value::from(32u64)),
        ("num_key_value_heads", Value::from(2u64)),
        ("head_dim", Value::from(128u64)),
    ];
    for (key, expected) in checks {
        if v.get(key) != Some(&expected) {
            return Err(format!("architecture mismatch: {key}"));
        }
    }
    if v.get("layer_types")
        .and_then(|x| x.as_array())
        .map(|a| a.len())
        != Some(52)
    {
        return Err("layer_types length mismatch".into());
    }
    if v.get("quantization")
        .and_then(|q| q.get("mode"))
        .and_then(|m| m.as_str())
        != Some("affine")
    {
        return Err("quantization mode mismatch".into());
    }
    Ok(v)
}

/// Ingest tokenizer artifact: return its SHA-256. Caller compares to frozen.
pub fn digest_file(path: &str) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("read file: {e}"))?;
    let mut h = Sha256::new();
    h.update(&bytes);
    Ok(format!("{:x}", h.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_op_rejected() {
        let node = GraphNode {
            id: "n0".into(),
            op: "magic_op".into(),
            input_shapes: vec![],
            output_shape: vec![],
            tensor_bindings: vec![],
        };
        assert!(admit_graph(&[node]).is_err());
    }

    #[test]
    fn empty_id_rejected() {
        let node = GraphNode {
            id: "".into(),
            op: "residual".into(),
            input_shapes: vec![],
            output_shape: vec![],
            tensor_bindings: vec![],
        };
        assert!(admit_graph(&[node]).is_err());
    }

    #[test]
    fn supported_ops_admit() {
        for op in SUPPORTED_OPS {
            let node = GraphNode {
                id: "n".into(),
                op: (*op).into(),
                input_shapes: vec![],
                output_shape: vec![1],
                tensor_bindings: vec![],
            };
            assert!(admit_graph(&[node]).is_ok(), "op {} should admit", op);
        }
    }

    #[test]
    fn ingest_missing_config_fails() {
        assert!(ingest_glimmer_config("/no/such/config.json").is_err());
    }

    #[test]
    fn shape_partition_changes_graph_digest() {
        let base = GraphNode {
            id: "n0".into(),
            op: "residual".into(),
            input_shapes: vec![vec![1, 2]],
            output_shape: vec![2],
            tensor_bindings: vec![],
        };
        let mut partitioned = base.clone();
        partitioned.input_shapes = vec![vec![1], vec![2]];
        assert_ne!(attest_graph(&[base]), attest_graph(&[partitioned]));
    }

    fn glimmer_layer_types() -> Vec<&'static str> {
        // Config pattern, verified against ~/models/Muse-Glimmer-q4km-gs128-MLX/config.json:
        // sliding, sliding, sliding, full, repeating, 52 layers total,
        // 39 sliding and 13 full.
        let mut v = Vec::new();
        for i in 0..52 {
            if (i + 1) % 4 == 0 {
                v.push("full_attention");
            } else {
                v.push("sliding_attention");
            }
        }
        v
    }

    #[test]
    fn operator_graph_digest_verified_and_forged_rejected() {
        let graph = glimmer_layer_graph(&glimmer_layer_types()).expect("glimmer graph admits");
        assert_eq!(graph.nodes.len(), 368);
        assert!(admit_operator_graph(&graph).is_ok());
        let mut forged = graph.clone();
        forged.execution_contract_digest = "0".repeat(64);
        assert!(admit_operator_graph(&forged).is_err());
        let mut bad_hex = graph.clone();
        bad_hex.execution_contract_digest = "not-a-digest".to_string();
        assert!(admit_operator_graph(&bad_hex).is_err());
        let mut empty_digest = graph.clone();
        empty_digest.execution_contract_digest = String::new();
        assert!(admit_operator_graph(&empty_digest).is_err());
    }

    #[test]
    fn glimmer_graph_layer_types_and_digest_sensitivity() {
        let graph = glimmer_layer_graph(&glimmer_layer_types()).expect("glimmer graph admits");
        let sliding = graph
            .nodes
            .iter()
            .filter(|n| n.id.contains("attention_sliding"))
            .count();
        let full = graph
            .nodes
            .iter()
            .filter(|n| n.id.contains("attention_full"))
            .count();
        assert_eq!(sliding, 39);
        assert_eq!(full, 13);
        assert_eq!(graph.nodes.first().unwrap().id, "embed");
        assert_eq!(graph.nodes.last().unwrap().id, "logit_cap");
        let mut flipped = glimmer_layer_types();
        flipped[0] = "full_attention";
        let other = glimmer_layer_graph(&flipped).expect("flipped graph admits");
        assert_ne!(
            graph.execution_contract_digest, other.execution_contract_digest
        );
        assert!(glimmer_layer_graph(&["sliding_attention"]).is_err());
        assert!(glimmer_layer_graph(&vec!["unknown_type"; 52]).is_err());
    }

    #[test]
    fn duplicate_id_and_zero_shape_rejected() {
        let node = GraphNode {
            id: "n0".into(),
            op: "residual".into(),
            input_shapes: vec![],
            output_shape: vec![1],
            tensor_bindings: vec![],
        };
        assert!(admit_graph(&[node.clone(), node.clone()]).is_err());
        let mut bad = node;
        bad.output_shape = vec![0];
        assert!(admit_graph(&[bad]).is_err());
    }
}

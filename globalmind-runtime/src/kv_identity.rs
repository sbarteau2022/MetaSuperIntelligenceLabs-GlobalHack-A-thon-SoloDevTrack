//! Exact, read-only transformer K/V lookup for model-bound forward reuse.

use crate::store::{record, sq};
use crate::{Digest, GlobalMindStore, RecordId, TensorDtype, TensorLayout, TensorPayload};
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct ExactKvIdentity {
    pub workspace: RecordId,
    pub session: RecordId,
    pub page: RecordId,
    pub model_id: String,
    pub model_revision: String,
    pub model_digest: Digest,
    pub request_digest: Digest,
    pub prefix_digest: Digest,
    pub tppe_version: String,
    pub tppe_configuration_digest: Digest,
    pub address_contract_digest: Digest,
    pub tensor_contract_digest: Digest,
    pub geometry_space: RecordId,
    pub dtype: TensorDtype,
    pub layer_count: u32,
    pub kv_head_count: u32,
    pub head_dim: u32,
    pub layer: u32,
    pub kv_head: u32,
    pub token_start: u64,
    pub token_end: u64,
    pub tensor_shape: [u32; 2],
    pub layout: TensorLayout,
    pub address: RecordId,
    pub phase_state_digest: Digest,
    pub revision: u32,
    pub key_digest: Digest,
    pub value_digest: Digest,
}

impl ExactKvIdentity {
    fn validate(&self) -> Result<(), String> {
        if self.model_id.trim().is_empty()
            || self.model_revision.trim().is_empty()
            || self.tppe_version.trim().is_empty()
            || self.layer_count == 0
            || self.kv_head_count == 0
            || self.head_dim == 0
            || self.layer >= self.layer_count
            || self.kv_head >= self.kv_head_count
            || self.token_end <= self.token_start
            || self.revision == 0
            || self.tensor_shape[0] as u64 != self.token_end - self.token_start
            || self.tensor_shape[1] != self.head_dim
        {
            return Err("invalid exact tensor K/V identity".into());
        }
        Ok(())
    }
}

fn exactly_one_id(rows: &Value, expected: &str) -> Result<(), String> {
    let rows = rows
        .as_array()
        .ok_or_else(|| format!("{expected} lookup returned no array"))?;
    if rows.len() != 1 {
        return Err(format!("cache miss or ambiguous {expected} identity"));
    }
    if rows[0]["id"].as_str() != Some(expected) {
        return Err(format!("{expected} identity changed during lookup"));
    }
    Ok(())
}

impl GlobalMindStore {
    /// Return K/V only when every model, request, TPPE, coordinate and payload
    /// digest field matches the caller's immutable witness.
    pub fn fetch_tensor_payload_exact(&self, q: &ExactKvIdentity) -> Result<TensorPayload, String> {
        q.validate()?;
        let session = record("transformer_kv_session", &q.session);
        let page = record("transformer_kv_page", &q.page);
        let session_rows = self.one(&format!(
            "SELECT id FROM {session} WHERE active = true AND workspace = {workspace} \
             AND model_id = {model_id} AND model_revision = {model_revision} \
             AND model_digest = {model_digest} AND request_digest = {request_digest} \
             AND prefix_digest = {prefix_digest} AND toroidal_phase_encoder_version = {tppe_version} \
             AND tppe_configuration_digest = {tppe_config} AND address_contract_digest = {address_contract} \
             AND tensor_contract_digest = {tensor_contract} AND geometry_space = {geometry_space} \
             AND dtype = {dtype} AND layer_count = {layer_count} AND kv_head_count = {kv_head_count} \
             AND head_dim = {head_dim} LIMIT 2;",
            workspace = record("workspace", &q.workspace),
            model_id = sq(&q.model_id),
            model_revision = sq(&q.model_revision),
            model_digest = sq(q.model_digest.as_str()),
            request_digest = sq(q.request_digest.as_str()),
            prefix_digest = sq(q.prefix_digest.as_str()),
            tppe_version = sq(&q.tppe_version),
            tppe_config = sq(q.tppe_configuration_digest.as_str()),
            address_contract = sq(q.address_contract_digest.as_str()),
            tensor_contract = sq(q.tensor_contract_digest.as_str()),
            geometry_space = record("geometry_space", &q.geometry_space),
            dtype = sq(q.dtype.as_str()),
            layer_count = q.layer_count,
            kv_head_count = q.kv_head_count,
            head_dim = q.head_dim,
        ))?;
        exactly_one_id(
            &session_rows,
            &format!("transformer_kv_session:{}", q.session),
        )?;

        let page_rows = self.one(&format!(
            "SELECT id FROM {page} WHERE active = true AND workspace = {workspace} AND session = {session} \
             AND layer = {layer} AND kv_head = {kv_head} AND token_start = {token_start} \
             AND token_end = {token_end} AND tensor_shape = [{shape0}, {shape1}] \
             AND dtype = {dtype} AND layout = {layout} AND address = {address} \
             AND phase_state_digest = {phase_state} AND revision = {revision} \
             AND key_digest = {key_digest} AND value_digest = {value_digest} LIMIT 2;",
            workspace = record("workspace", &q.workspace),
            layer = q.layer,
            kv_head = q.kv_head,
            token_start = q.token_start,
            token_end = q.token_end,
            shape0 = q.tensor_shape[0],
            shape1 = q.tensor_shape[1],
            dtype = sq(q.dtype.as_str()),
            layout = sq(q.layout.as_str()),
            address = record("atom_address", &q.address),
            phase_state = sq(q.phase_state_digest.as_str()),
            revision = q.revision,
            key_digest = sq(q.key_digest.as_str()),
            value_digest = sq(q.value_digest.as_str()),
        ))?;
        exactly_one_id(&page_rows, &format!("transformer_kv_page:{}", q.page))?;
        let payload = self.fetch_tensor_payload(&q.page)?;
        if Digest::sha256(&payload.key) != q.key_digest
            || Digest::sha256(&payload.value) != q.value_digest
        {
            return Err("tensor K/V payload changed after identity lookup".into());
        }
        Ok(payload)
    }
}

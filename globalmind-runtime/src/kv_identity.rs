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

    /// Reassemble one complete per-layer/head K/V interval from exact pages.
    /// Input pages must be in token order. A missing, overlapping, or foreign
    /// page is a cache miss; no partial tensor is returned to the decoder.
    pub fn fetch_contiguous_tensor_span_exact(
        &self,
        pages: &[ExactKvIdentity],
        token_start: u64,
        token_end: u64,
    ) -> Result<TensorPayload, String> {
        const MAX_SPAN_BYTES: usize = 256 * 1024 * 1024;
        if pages.is_empty() || pages.len() > 1024 || token_end <= token_start {
            return Err("invalid exact K/V span page count or interval".into());
        }
        let first = &pages[0];
        let scalar_bytes = first
            .dtype
            .scalar_bytes()
            .ok_or("quantized K/V span assembly requires an explicit block layout")?;
        let dim = usize::try_from(first.head_dim).map_err(|_| "head dimension overflow")?;
        let total_tokens =
            usize::try_from(token_end - token_start).map_err(|_| "K/V token span overflow")?;
        let total_bytes = total_tokens
            .checked_mul(dim)
            .and_then(|n| n.checked_mul(scalar_bytes))
            .ok_or("K/V span byte count overflow")?;
        if total_bytes > MAX_SPAN_BYTES {
            return Err("K/V span exceeds 256 MiB per tensor".into());
        }

        let mut cursor = token_start;
        let mut payloads = Vec::with_capacity(pages.len());
        for page in pages {
            if page.token_start != cursor || page.token_end > token_end {
                return Err("K/V span has a gap, overlap, or out-of-range page".into());
            }
            if page.workspace != first.workspace
                || page.session != first.session
                || page.model_id != first.model_id
                || page.model_revision != first.model_revision
                || page.model_digest != first.model_digest
                || page.request_digest != first.request_digest
                || page.prefix_digest != first.prefix_digest
                || page.tppe_version != first.tppe_version
                || page.tppe_configuration_digest != first.tppe_configuration_digest
                || page.address_contract_digest != first.address_contract_digest
                || page.tensor_contract_digest != first.tensor_contract_digest
                || page.geometry_space != first.geometry_space
                || page.dtype != first.dtype
                || page.layer_count != first.layer_count
                || page.kv_head_count != first.kv_head_count
                || page.head_dim != first.head_dim
                || page.layer != first.layer
                || page.kv_head != first.kv_head
                || page.layout != first.layout
            {
                return Err("K/V span mixes incompatible page identities".into());
            }
            let payload = self.fetch_tensor_payload_exact(page)?;
            let page_tokens = usize::try_from(page.token_end - page.token_start)
                .map_err(|_| "K/V page token count overflow")?;
            let page_bytes = page_tokens
                .checked_mul(dim)
                .and_then(|n| n.checked_mul(scalar_bytes))
                .ok_or("K/V page byte count overflow")?;
            if payload.key.len() != page_bytes || payload.value.len() != page_bytes {
                return Err("K/V page byte count does not match its shape".into());
            }
            payloads.push(payload);
            cursor = page.token_end;
        }
        if cursor != token_end {
            return Err("K/V span does not cover the requested token interval".into());
        }

        let mut key = Vec::with_capacity(total_bytes);
        let mut value = Vec::with_capacity(total_bytes);
        match first.layout {
            TensorLayout::TokenMajor => {
                for payload in &payloads {
                    key.extend_from_slice(&payload.key);
                    value.extend_from_slice(&payload.value);
                }
            }
            TensorLayout::HeadDimMajor => {
                for coordinate in 0..dim {
                    for payload in &payloads {
                        let page_tokens = payload.key.len() / (dim * scalar_bytes);
                        let start = coordinate * page_tokens * scalar_bytes;
                        let end = start + page_tokens * scalar_bytes;
                        key.extend_from_slice(&payload.key[start..end]);
                        value.extend_from_slice(&payload.value[start..end]);
                    }
                }
            }
        }
        Ok(TensorPayload {
            key: key.into_boxed_slice(),
            value: value.into_boxed_slice(),
        })
    }

    /// Hydrate one complete attention layer, with every KV head present.
    /// Pages are ordered by head then token interval; missing heads fail closed.
    pub fn fetch_complete_tensor_layer_exact(
        &self,
        pages: &[ExactKvIdentity],
        layer: u32,
        token_start: u64,
        token_end: u64,
    ) -> Result<Vec<TensorPayload>, String> {
        if pages.is_empty() || pages.len() > 4096 {
            return Err("invalid complete-layer page count".into());
        }
        let first = &pages[0];
        if layer >= first.layer_count || first.kv_head_count == 0 {
            return Err("complete-layer target outside session".into());
        }
        for page in pages {
            if page.workspace != first.workspace
                || page.session != first.session
                || page.model_id != first.model_id
                || page.model_revision != first.model_revision
                || page.model_digest != first.model_digest
                || page.request_digest != first.request_digest
                || page.prefix_digest != first.prefix_digest
                || page.tppe_version != first.tppe_version
                || page.tppe_configuration_digest != first.tppe_configuration_digest
                || page.address_contract_digest != first.address_contract_digest
                || page.tensor_contract_digest != first.tensor_contract_digest
                || page.geometry_space != first.geometry_space
                || page.dtype != first.dtype
                || page.layout != first.layout
                || page.layer_count != first.layer_count
                || page.kv_head_count != first.kv_head_count
                || page.head_dim != first.head_dim
                || page.layer != layer
            {
                return Err("complete K/V layer mixes incompatible identities".into());
            }
        }
        let mut cursor = 0;
        let mut heads = Vec::with_capacity(first.kv_head_count as usize);
        for head in 0..first.kv_head_count {
            let start = cursor;
            while cursor < pages.len() && pages[cursor].kv_head == head {
                cursor += 1;
            }
            if cursor == start {
                return Err("complete K/V layer is missing a head".into());
            }
            heads.push(self.fetch_contiguous_tensor_span_exact(
                &pages[start..cursor],
                token_start,
                token_end,
            )?);
        }
        if cursor != pages.len() {
            return Err("complete K/V layer contains extra or misordered pages".into());
        }
        Ok(heads)
    }
}

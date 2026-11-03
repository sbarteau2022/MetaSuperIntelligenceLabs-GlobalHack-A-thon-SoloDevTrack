//! Read-only, typed bridge to the one GlobalMind SurrealDB substrate.
//!
//! Model-facing recall is the native GMRC binary format. This bridge never
//! ingests atoms, creates a mind map, or exposes tensor K/V by page ID alone.

use globalmind_runtime::retrieval::{RetrievalConfig, encode_bundle, retrieve};
use globalmind_runtime::{ExactKvIdentity, GlobalMindStore, StoreConfig, TensorPayload};

#[derive(Debug)]
pub struct GlobalMindBridge {
    retrieval: RetrievalConfig,
    store: GlobalMindStore,
}

/// Native inputs for one read-only forward preparation. Tensor bytes remain
/// model-bound; the recalled neighborhood may contain shared atoms.
pub struct ForwardRecall {
    pub neighborhood_gmrc: Vec<u8>,
    pub tensor: TensorPayload,
}

impl GlobalMindBridge {
    pub fn connect(retrieval: RetrievalConfig) -> Result<Self, String> {
        retrieval.validate()?;
        let store = GlobalMindStore::connect(StoreConfig {
            endpoint: retrieval.endpoint.clone(),
            namespace: retrieval.namespace.clone(),
            database: retrieval.database.clone(),
            username: Some(retrieval.username.clone()),
            password: Some(retrieval.password.clone()),
            allow_quarantined_core: false,
        })?;
        store.verify_contract()?;
        Ok(Self { retrieval, store })
    }

    /// Retrieve a receipted neighborhood and encode it for the model boundary.
    pub fn recall_binary(
        &self,
        query_h3: [f64; 4],
        query_t2: [f64; 2],
        query_provenance: &str,
        k: u32,
    ) -> Result<Vec<u8>, String> {
        let bundle = retrieve(&self.retrieval, query_h3, query_t2, query_provenance, k)?;
        encode_bundle(&bundle)
    }

    /// Fetch immutable model-bound K/V only through the complete identity.
    pub fn fetch_tensor_exact(&self, identity: &ExactKvIdentity) -> Result<TensorPayload, String> {
        self.store.fetch_tensor_payload_exact(identity)
    }

    /// Bind graph recall and tensor K/V to one fail-closed harness operation.
    /// No tensor bytes are returned unless the complete identity matches.
    pub fn prepare_forward_recall(
        &self,
        query_h3: [f64; 4],
        query_t2: [f64; 2],
        query_provenance: &str,
        k: u32,
        identity: &ExactKvIdentity,
    ) -> Result<ForwardRecall, String> {
        let neighborhood_gmrc = self.recall_binary(query_h3, query_t2, query_provenance, k)?;
        let tensor = self.fetch_tensor_exact(identity)?;
        Ok(ForwardRecall {
            neighborhood_gmrc,
            tensor,
        })
    }
}

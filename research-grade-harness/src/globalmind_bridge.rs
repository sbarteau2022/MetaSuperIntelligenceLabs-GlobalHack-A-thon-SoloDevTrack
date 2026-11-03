//! Read-only, typed bridge to the one GlobalMind SurrealDB substrate.
//!
//! Model-facing recall is the native GMRC binary format. This bridge never
//! ingests atoms, creates a mind map, or exposes tensor K/V by page ID alone.

use globalmind_runtime::retrieval::{RetrievalConfig, decode_bundle, encode_bundle, retrieve};
use globalmind_runtime::{ExactKvIdentity, GlobalMindStore, StoreConfig, TensorPayload};

/// Declared authority of the MCP endpoint that sits beside this bridge.
/// This is a config preflight, not proof that an MCP child is healthy.
#[derive(Clone, Debug)]
pub struct McpSurrealTarget {
    pub mode: String,
    pub endpoint: String,
    pub namespace: String,
    pub database: String,
}

impl McpSurrealTarget {
    pub fn from_env() -> Self {
        Self {
            mode: std::env::var("HYPERMIND_SURREAL").unwrap_or_default(),
            endpoint: std::env::var("HYPERMIND_SURREAL_URL").unwrap_or_default(),
            namespace: std::env::var("HYPERMIND_SURREAL_NS").unwrap_or_default(),
            database: std::env::var("HYPERMIND_SURREAL_DB").unwrap_or_default(),
        }
    }

    pub fn verify_same_mind(&self, retrieval: &RetrievalConfig) -> Result<(), String> {
        if self.mode != "on" || self.database == "core" {
            return Err(
                "MCP must declare SurrealDB authority mode on outside quarantined core".into(),
            );
        }
        if self.endpoint.is_empty()
            || self.namespace.is_empty()
            || self.database.is_empty()
            || self.endpoint != retrieval.endpoint
            || self.namespace != retrieval.namespace
            || self.database != retrieval.database
        {
            return Err("MCP and harness bridge target different GlobalMind authorities".into());
        }
        Ok(())
    }
}

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
    /// Admit a forward smoke only when MCP and typed bridge name one authority.
    pub fn connect_at_mcp_boundary(
        retrieval: RetrievalConfig,
        mcp: &McpSurrealTarget,
    ) -> Result<Self, String> {
        mcp.verify_same_mind(&retrieval)?;
        Self::connect(retrieval)
    }

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
        let bundle = decode_bundle(&neighborhood_gmrc)?;
        if !bundle
            .members
            .iter()
            .any(|member| member.address == identity.address)
        {
            return Err("tensor K/V address is absent from the recalled neighborhood".into());
        }
        let tensor = self.fetch_tensor_exact(identity)?;
        Ok(ForwardRecall {
            neighborhood_gmrc,
            tensor,
        })
    }
}

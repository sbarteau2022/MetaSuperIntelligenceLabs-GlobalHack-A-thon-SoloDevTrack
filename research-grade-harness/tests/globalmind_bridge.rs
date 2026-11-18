use globalmind_runtime::RecordId;
use globalmind_runtime::retrieval::{RetrievalConfig, RetrievalMetric};
use rgah::GlobalMindBridge;

#[test]
fn bridge_refuses_quarantined_core_even_with_valid_retrieval_config() {
    let config = RetrievalConfig {
        endpoint: "http://127.0.0.1:8000".into(),
        namespace: "hypermind".into(),
        database: "core".into(),
        username: "test".into(),
        password: "test".into(),
        workspace: RecordId::parse("globalmind").unwrap(),
        candidate_budget: 4096,
        metric: RetrievalMetric::Hyperbolic,
    };
    assert!(
        GlobalMindBridge::connect(config)
            .unwrap_err()
            .contains("quarantined")
    );
}

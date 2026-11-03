use ares_backend::harness::qdrant_memory::{research_rank1, QdrantMemory};
use serde_json::{json, Value};

/// Run explicitly against the local populated research library.
#[tokio::test]
#[ignore = "requires live Qdrant research_library_v1"]
async fn stored_qwen_vector_retrieves_its_own_point_on_both_ranks() {
    let base = std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://127.0.0.1:6333".into());
    let client = reqwest::Client::new();
    let response: Value = client
        .post(format!(
            "{base}/collections/research_library_v1/points/scroll"
        ))
        .json(
            &json!({ "limit": 1, "with_payload": true, "with_vector": true,
            "filter": { "must": [{ "key": "silo", "match": { "value": "specops" } }] } }),
        )
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    let point = &response["result"]["points"][0];
    let id = point["id"].as_str().expect("stored point ID");
    let rank0: Vec<f32> = point["vector"]["text_rank0"]
        .as_array()
        .expect("stored rank0")
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect();
    let rank1: Vec<f32> = point["vector"]["text_rank1"]
        .as_array()
        .expect("stored rank1")
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect();
    let computed = research_rank1(&rank0).unwrap();
    let max_error = computed
        .iter()
        .zip(&rank1)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0_f32, f32::max);
    assert!(
        max_error < 2e-6,
        "rank rotation differs from ingest: {max_error}"
    );
    let memory = QdrantMemory::new(base).unwrap();
    let hits = memory
        .query_research_rank0(&rank0, 8, Some("specops"))
        .await
        .unwrap();
    assert_eq!(hits.first().map(|h| h.id.as_str()), Some(id));
    assert!(hits[0].score.unwrap_or_default() > 0.0);
}

use ares_backend::harness::qdrant_memory::{QdrantMemory, MEMORY_ENCODER};

#[tokio::test]
#[ignore = "writes one true cutover event to live local Qdrant"]
async fn tppe_memory_append_tail_and_query_survive_new_client() {
    let memory = QdrantMemory::from_env().unwrap();
    memory.ensure_collection().await.unwrap();
    let session = "qdrant-tppe-cutover-20261102";
    let text = "Qdrant session_memory_tppe_v1 was created locally with a 384D cosine vector and six payload indexes on 2026-11-02.";
    let entry = memory
        .append_memory(
            "session_summary",
            text,
            None,
            Some(session),
            None,
            None,
            None,
            None,
            &[],
        )
        .await
        .unwrap();
    assert_eq!(entry.text, text);
    let restarted_client = QdrantMemory::from_env().unwrap();
    let tail = restarted_client
        .tail_recent(4, None, Some(session))
        .await
        .unwrap();
    assert!(tail.iter().any(|item| item.id == entry.id));
    let hits = restarted_client
        .query_memory(text, 4, None, Some(session), None)
        .await
        .unwrap();
    assert!(
        hits.iter().any(|item| item.id == entry.id),
        "TPPE memory query missed the stored event"
    );
    assert_eq!(
        MEMORY_ENCODER,
        "tppe-linear-log-v1+candidate-v2+memory-dual-v1"
    );
}

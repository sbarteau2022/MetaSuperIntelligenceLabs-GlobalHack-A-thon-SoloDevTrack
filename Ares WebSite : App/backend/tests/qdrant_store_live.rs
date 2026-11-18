use ares_backend::harness::qdrant_memory::{QdrantMemory, MEMORY_COLLECTION};

#[tokio::test]
#[ignore = "requires live local Qdrant and research library"]
async fn tppe_memory_and_research_collections_have_separate_verified_contracts() {
    let memory = QdrantMemory::from_env().unwrap();
    assert_eq!(memory.collection(), MEMORY_COLLECTION);
    assert!(
        !memory.ensure_collection().await.unwrap(),
        "TPPE collection must already exist"
    );
    let research_points = memory.verify_research_collection().await.unwrap();
    assert!(
        research_points > 0,
        "research library must contain real points"
    );
}

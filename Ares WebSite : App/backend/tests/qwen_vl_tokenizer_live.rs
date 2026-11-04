use ares_backend::harness::qwen_vl_native::QwenVlTokenizer;
use std::path::Path;

#[test]
#[ignore = "requires the pinned local Qwen3 VL checkpoint"]
fn ingest_token_sequence_matches_mlx_oracle() {
    let directory = Path::new("/Users/stewartbarteau/models/Qwen3-VL-Embedding-2B-6bit-RUN");
    let tokenizer = QwenVlTokenizer::load(directory).unwrap();
    assert_eq!(
        tokenizer.encode("hyperbolic memory").unwrap(),
        [68192, 65, 7762, 4938, 151643]
    );
    assert!(tokenizer.encode("").is_err());
    assert!(tokenizer.encode(&"a".repeat(3201)).is_err());
}

use ares_backend::harness::glimmer_tokenizer::GlimmerTokenizer;
use std::path::Path;

#[test]
#[ignore = "requires the pinned Glimmer checkpoint on WD Drive"]
fn glimmer_shared_seat_uses_pinned_tokenizer() {
    let directory = Path::new("/Volumes/WD Drive/models/Muse-Glimmer-q4km-gs128-MLX");
    let tokenizer = GlimmerTokenizer::load(directory).unwrap();
    assert_eq!(
        tokenizer.encode("hyperbolic memory").unwrap(),
        [200000, 51611, 149191, 7927]
    );
    assert!(tokenizer.encode("").is_err());
    assert!(tokenizer.encode(&"a".repeat(3201)).is_err());
    assert!(GlimmerTokenizer::load(Path::new("/no/such/glimmer/checkpoint")).is_err());
}

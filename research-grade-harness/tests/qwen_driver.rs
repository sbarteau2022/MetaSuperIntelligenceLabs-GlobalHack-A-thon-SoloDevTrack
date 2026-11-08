use rgah::qwen_driver::QwenDriver;

/// Live gate: real Qwen3.6-MoE behind `serve_qwen` (RGAH_QWEN_ADDR, e.g. 127.0.0.1:8003).
/// Every reply is receipt-checked inside the driver; a receipt mismatch fails here.
#[test]
#[ignore = "requires serve_qwen (RGAH_QWEN_ADDR)"]
fn qwen_tokenizes_chat_markers_as_single_tokens_and_completes_deterministically() {
    let addr = std::env::var("RGAH_QWEN_ADDR").expect("RGAH_QWEN_ADDR");
    let d = QwenDriver::new(&addr);

    // Chat markers must be atomic special tokens, not character sequences.
    let im_start = d.tokenize("<|im_start|>").expect("tokenize marker");
    assert_eq!(im_start.len(), 1, "<|im_start|> split into {im_start:?}");
    let im_end = d.tokenize("<|im_end|>").expect("tokenize marker");
    assert_eq!(im_end.len(), 1, "<|im_end|> split into {im_end:?}");

    let prompt = "<|im_start|>user\nSay the word ready.<|im_end|>\n<|im_start|>assistant\n<think>\n\n</think>\n\n";
    let a = d.complete(prompt, 16).expect("complete 1");
    let b = d.complete(prompt, 16).expect("complete 2");
    assert!(!a.text.is_empty(), "empty completion");
    assert_eq!(a.text, b.text, "identical requests must give identical text");
    assert_eq!(a.composite_hex, b.composite_hex, "receipts must match for identical requests");
    println!("COMPLETION {:?} receipt={}", a.text, a.composite_hex);
}

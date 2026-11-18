//! Live-only test for the native MTP automaton executor.
//!
//! Ignored by default: it requires the native MTP server running on
//! loopback (default http://127.0.0.1:8010). Run explicitly with:
//!   cargo test --test native_mtp_live -- --ignored
//! It proves the backend's AMST frame parser against real server
//! frames, and that the executor returns a 64-hex binding digest.
//! It is a wiring probe, not a model-quality test: it asserts output
//! is non-empty and the receipt is a digest, nothing about what the
//! model says.

use ares_backend::harness::native_mtp::NativeMtpExecutor;
use ares_backend::harness::runtime::Executor;

#[test]
#[ignore]
fn native_mtp_executor_returns_output_and_digest() {
    let mut executor = NativeMtpExecutor::from_env().expect("native-mtp config");
    let out = executor
        .call("map.probe", "{\"probe\": \"wiring\"}")
        .expect("native-mtp call");
    assert!(!out.inner_text.is_empty(), "empty output");
    assert_eq!(out.receipt_sha256.len(), 64, "receipt not 64-hex");
    assert!(
        out.receipt_sha256.bytes().all(|b| b.is_ascii_hexdigit()),
        "receipt not hex"
    );
    let seat = executor.seat_identity().expect("seat identity");
    assert_eq!(seat.len(), 64, "seat identity not 64-hex");
}

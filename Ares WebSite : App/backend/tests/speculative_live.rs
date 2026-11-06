//! Hardware gate for the automaton-governed speculative loop on the frozen
//! Muse Glimmer: the speculative arm must produce EXACTLY the control arm's
//! greedy tokens, with the model's own forward as the only judge.
//! Needs ARES_GLIMMER_NATIVE_URL (isolated server) and ARES_GLIMMER_WEIGHTS_SHA256.
//! Run: cargo test --release --test speculative_live -- --ignored --nocapture

use ares_backend::fixed::Q32_31;
use ares_backend::harness::automaton::StrategyState;
use ares_backend::harness::native_glimmer_executor::{
    decode_token_response, encode_token_request, ArtifactPin,
};
use ares_backend::harness::speculative::{
    run_control, run_speculative, run_speculative_held, HeldProposer, HttpVerify,
    PromptLookupDrafter, SpecConfig,
};

const PROMPT: &str = "The capital of France is";
const ATOM: &str =
    "The capital of France is Paris. Paris lies on the Seine and is home to the Louvre.";
const OFF_TOPIC: &str =
    "Quarterly revenue grew while the committee postponed the budget vote again.";

fn tokenize(url: &str, text: &str, bos: bool) -> Vec<u32> {
    let req = encode_token_request(text, bos).unwrap();
    let resp = reqwest::blocking::Client::new()
        .post(format!("{url}/v1/tokenize-native"))
        .body(req.clone())
        .send()
        .unwrap()
        .bytes()
        .unwrap();
    decode_token_response(&req, &resp).unwrap().token_ids
}

fn state(pin: &ArtifactPin) -> StrategyState {
    StrategyState {
        strategy_id: "glimmer-drafter".into(),
        policy_version: "policy-v1".into(),
        prefix_identity: pin.composite.iter().map(|b| format!("{b:02x}")).collect(),
        map_identity: "atom-text-v1".into(),
        confidence: Q32_31::from_f64(0.6),
        confidence_floor: Q32_31::from_f64(0.2),
        calibration_count: 0,
        outcome_history_digest: "genesis".into(),
    }
}

#[test]
#[ignore = "isolated frozen-Glimmer server with /v1/verify-exact"]
fn speculative_equals_control_on_real_glimmer() {
    let url = std::env::var("ARES_GLIMMER_NATIVE_URL").unwrap();
    assert!(!url.contains(":8011") && !url.contains(":8000"));
    let pin = ArtifactPin::from_hex(
        &std::env::var("ARES_GLIMMER_WEIGHTS_SHA256").unwrap(),
        "c9dbee66967b58f31a7c27f723c3760da3526ccd0427578e8905b0abb0031c4d",
        "3ce829f4654d2f5f5f43acca424fe17d72e74015d375d385a86b7fff24d4a031",
    )
    .unwrap();
    let prompt = tokenize(&url, PROMPT, true);
    let cfg = SpecConfig {
        max_new_tokens: 12,
        eos_ids: vec![],
        now_ms: 1,
        probe_every: 2,
        probe_width: 1,
    };

    let mut backend = HttpVerify::new(&url, pin).unwrap();
    let control = run_control(&mut backend, &prompt, &cfg).unwrap();
    println!(
        "control: tokens={:?} rounds={} elapsed={:?}",
        control.tokens, control.forward_rounds, control.elapsed
    );

    for (label, text) in [("on-topic atom", ATOM), ("off-topic atom", OFF_TOPIC)] {
        let corpus = tokenize(&url, text, false);
        let drafter = PromptLookupDrafter::new(corpus, 8);
        let mut st = state(&pin);
        let start = st.confidence;
        let spec = run_speculative(&mut backend, &drafter, &mut st, &prompt, &cfg).unwrap();
        assert_eq!(
            spec.tokens, control.tokens,
            "{label}: output must equal control"
        );
        let drafted: usize = spec.rounds.iter().map(|r| r.draft_len).sum();
        let accepted: usize = spec.rounds.iter().map(|r| r.accepted).sum();
        let rolled: usize = spec.rounds.iter().map(|r| r.rolled_back).sum();
        println!(
            "{label}: rounds={} (control {}) drafted={drafted} accepted={accepted} rolled_back={rolled} \
             confidence {:.4}->{:.4} calibrations={} elapsed={:?} head={}",
            spec.forward_rounds,
            control.forward_rounds,
            start.to_f64(),
            st.confidence.to_f64(),
            st.calibration_count,
            spec.elapsed,
            spec.rounds.last().unwrap().chain_hash
        );
        if label == "on-topic atom" {
            assert!(accepted > 0, "on-topic atom should be partly accepted");
            assert!(spec.forward_rounds < control.forward_rounds);
        } else if drafted > 0 {
            assert!(
                st.confidence < start,
                "rejected drafts must lower confidence"
            );
        }
    }
}

/// Hardware gate for the held loop: two real strategies are held every round,
/// re-ranked by earned confidence, and the model's own forward is still the
/// only judge, so the output must equal the control arm's greedy tokens. The
/// off-topic strategy starts HIGHER (0.9) than the on-topic one (0.6), so any
/// re-ranking has to be earned from verified rejections on the real model.
/// Run: cargo test --release --test speculative_live held_loop -- --ignored --nocapture
#[test]
#[ignore = "isolated frozen-Glimmer server with /v1/verify-exact"]
fn held_loop_reranks_two_real_strategies_and_equals_control() {
    let url = std::env::var("ARES_GLIMMER_NATIVE_URL").unwrap();
    assert!(!url.contains(":8011") && !url.contains(":8000"));
    let pin = ArtifactPin::from_hex(
        &std::env::var("ARES_GLIMMER_WEIGHTS_SHA256").unwrap(),
        "c9dbee66967b58f31a7c27f723c3760da3526ccd0427578e8905b0abb0031c4d",
        "3ce829f4654d2f5f5f43acca424fe17d72e74015d375d385a86b7fff24d4a031",
    )
    .unwrap();
    let prompt = tokenize(&url, PROMPT, true);
    let cfg = SpecConfig {
        max_new_tokens: 12,
        eos_ids: vec![],
        now_ms: 1,
        probe_every: 2,
        probe_width: 1,
    };
    let mut backend = HttpVerify::new(&url, pin).unwrap();
    let control = run_control(&mut backend, &prompt, &cfg).unwrap();

    let on = PromptLookupDrafter::new(tokenize(&url, ATOM, false), 8);
    let off = PromptLookupDrafter::new(tokenize(&url, OFF_TOPIC, false), 8);
    let mut s_on = state(&pin);
    s_on.strategy_id = "atom-on-topic".into();
    let mut s_off = state(&pin);
    s_off.strategy_id = "atom-off-topic".into();
    s_off.confidence = Q32_31::from_f64(0.9);
    let (start_on, start_off) = (s_on.confidence, s_off.confidence);

    let spec = {
        let mut held = [
            HeldProposer {
                drafter: &off,
                state: &mut s_off,
            },
            HeldProposer {
                drafter: &on,
                state: &mut s_on,
            },
        ];
        run_speculative_held(&mut backend, &mut held, &prompt, &cfg).unwrap()
    };
    assert_eq!(spec.tokens, control.tokens, "output must equal control");
    for (n, r) in spec.rounds.iter().enumerate() {
        println!(
            "round {n:2}: ran={:<15} held={} probe={} draft={} accepted={} conf_after={:.4}",
            r.strategy_id.as_deref().unwrap_or("-"),
            r.held,
            r.probe,
            r.draft_len,
            r.accepted,
            r.confidence_after.to_f64()
        );
    }
    println!(
        "on-topic {:.4}->{:.4} (n={}), off-topic {:.4}->{:.4} (n={}), rounds {} vs control {}, elapsed {:?}",
        start_on.to_f64(),
        s_on.confidence.to_f64(),
        s_on.calibration_count,
        start_off.to_f64(),
        s_off.confidence.to_f64(),
        s_off.calibration_count,
        spec.forward_rounds,
        control.forward_rounds,
        spec.elapsed
    );
    assert!(
        spec.rounds
            .iter()
            .any(|r| r.strategy_id.as_deref() == Some("atom-on-topic")),
        "the honest strategy must get to run once the liar loses rank"
    );
    if s_off.calibration_count > 0 {
        assert!(
            s_off.confidence < start_off,
            "verified rejections must lower the off-topic strategy"
        );
    }
}

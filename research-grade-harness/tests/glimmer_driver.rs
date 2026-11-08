use rgah::glimmer_driver::{Action, GlimmerDriver, parse_action};

#[test]
fn parses_each_action_verb() {
    assert_eq!(
        parse_action("ACTION search :: reactor vent rate"),
        Ok(Action::Search("reactor vent rate".into()))
    );
    assert_eq!(
        parse_action("some preamble\nACTION read :: doc-3\n"),
        Ok(Action::Read("doc-3".into()))
    );
    assert_eq!(parse_action("ACTION done :: 47 minutes"), Ok(Action::Done("47 minutes".into())));
}

#[test]
fn rejects_unknown_verb_and_missing_line() {
    assert!(parse_action("ACTION delete :: everything").is_err());
    assert!(parse_action("I think the answer is 47").is_err());
}

#[test]
fn rejects_empty_prompt_before_any_network_call() {
    let d = GlimmerDriver::new("127.0.0.1:1");
    assert!(d.complete("", 8).is_err());
}

/// Live gate: real Muse Glimmer server. Set RGAH_GLIMMER_ADDR (host:port) and run with --ignored.
#[test]
#[ignore = "requires a running Muse Glimmer server (RGAH_GLIMMER_ADDR)"]
fn live_glimmer_returns_parseable_action_with_receipt() {
    let addr = std::env::var("RGAH_GLIMMER_ADDR").expect("RGAH_GLIMMER_ADDR");
    let d = GlimmerDriver::new(&addr);
    let prompt = "Reply with exactly one line of the form: ACTION <search|read|done> :: <argument>\nTask: how often does the reactor vent coolant?\n";
    let out = d.complete(prompt, 40).expect("live completion");
    assert!(!out.text.is_empty());
    assert!(out.composite_hex.len() == 64, "composite digest must be 32 bytes");
    // The model may not follow the grammar on every try; the driver must surface that as an error, not guess.
    match parse_action(&out.text) {
        Ok(_) => {}
        Err(e) => assert!(e.contains("no ACTION line"), "unexpected parse error: {e}"),
    }
}

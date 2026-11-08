//! Model-agnostic action protocol: the `ACTION <verb> :: <arg>` line a model
//! emits for one agent step. No model or transport is named here — any
//! executor (the bound ModelLane, a test stub, anything) can produce the text
//! this parses. Extracted from the old Muse-specific driver so the agent step
//! is model-agnostic.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Search(String),
    Read(String),
    Done(String),
}

/// Reads the first `ACTION <verb> :: <arg>` line. Unknown verbs and missing lines are errors.
pub fn parse_action(text: &str) -> Result<Action, String> {
    for line in text.lines() {
        let Some(rest) = line.trim().strip_prefix("ACTION ") else { continue };
        let Some((verb, arg)) = rest.split_once(" :: ") else { continue };
        let arg = arg.trim().to_string();
        return match verb.trim() {
            "search" => Ok(Action::Search(arg)),
            "read" => Ok(Action::Read(arg)),
            "done" => Ok(Action::Done(arg)),
            other => Err(format!("unknown ACTION verb: {other}")),
        };
    }
    Err("no ACTION line in model output".into())
}

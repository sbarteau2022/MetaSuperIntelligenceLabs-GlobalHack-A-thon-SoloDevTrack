//! One daemon cycle as a model step: build the prompt (task, grammar, evidence so far),
//! read the model's action, run any retrieval it asked for, and turn that into the
//! verifier signal the loop controller consumes.

use crate::action::{Action, parse_action};
use crate::loop_ctl::CycleSignal;

/// Evidence gathered across cycles. Each entry is a retrieval query and what came back.
#[derive(Debug, Clone, Default)]
pub struct Evidence {
    items: Vec<String>,
}

impl Evidence {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, query: &str, result: &str) {
        self.items.push(format!("{query}: {result}"));
    }

    pub fn render(&self) -> String {
        self.items.iter().map(|i| format!("- {i}\n")).collect()
    }
}

/// Source of evidence for `search` and `read` actions. The real implementation is the
/// N3 retrieval tool; tests may supply a fixed source for plumbing checks only.
pub trait Retriever {
    fn lookup(&mut self, query: &str) -> Result<String, String>;
}

/// Prompt for one cycle. The grammar line is fixed so the output can be checked by `parse_action`.
pub fn step_prompt(task: &str, cycle: u32, evidence: &Evidence) -> String {
    let mut user = format!(
        "Reply with exactly one line of the form: ACTION <search|read|done> :: <argument>\n\
         Task: {task}\n\
         This is step {cycle} of the work. Use done only when the task is answered.\n"
    );
    if !evidence.items.is_empty() {
        user.push_str("Evidence so far:\n");
        user.push_str(&evidence.render());
    }
    // Qwen3.6 ChatML with thinking disabled: user turn, then an empty think block, then the
    // answer. The model continues from `ACTION `, so its first tokens are the action line.
    format!("<|im_start|>user\n{user}<|im_end|>\n<|im_start|>assistant\n<think>\n\n</think>\n\nACTION ")
}

/// Rebuilds the action line from the model's continuation: the text after the prompt's
/// `ACTION ` prefix, cut at the first newline so trailing prose cannot reach the parser.
pub fn action_text(continuation: &str) -> String {
    format!("ACTION {}", continuation.lines().next().unwrap_or("").trim())
}

/// Maps the model's output to a verifier signal without retrieval.
/// Only `done` passes; `search` and `read` are intermediate; unparseable output is a grammar error.
pub fn signal_from_output(text: &str) -> CycleSignal {
    match parse_action(text) {
        Ok(Action::Done(_)) => signal(true, None),
        Ok(Action::Search(_)) | Ok(Action::Read(_)) => signal(false, None),
        Err(_) => signal(false, Some("grammar")),
    }
}

/// Runs one cycle: parses the action, performs any retrieval it asks for into `evidence`,
/// and returns the verifier signal. Retrieval failures propagate as errors.
pub fn run_step(
    text: &str,
    retriever: &mut dyn Retriever,
    evidence: &mut Evidence,
) -> Result<CycleSignal, String> {
    match parse_action(text) {
        Ok(Action::Search(q)) | Ok(Action::Read(q)) => {
            let result = retriever.lookup(&q)?;
            evidence.push(&q, &result);
            Ok(signal(false, None))
        }
        Ok(Action::Done(_)) => Ok(signal(true, None)),
        Err(_) => Ok(signal(false, Some("grammar"))),
    }
}

fn signal(verifier_pass: bool, error_class: Option<&str>) -> CycleSignal {
    CycleSignal {
        verifier_pass,
        stalled: false,
        budget_used_frac: 0.0,
        error_class: error_class.map(str::to_string),
    }
}

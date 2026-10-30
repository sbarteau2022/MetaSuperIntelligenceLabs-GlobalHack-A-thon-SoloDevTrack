//! Structured egress: grammar validation (post-hoc) + receipt.
//!
//! 2026-10-30 (English-only pin, Stewart directive): `Grammar::Json` DELETED.
//! A JSON wellformedness gate at egress contradicts the repo JSON ban
//! (model I/O boundary: no JSON). Only the Qwen3Xml grammar check remains.

#[derive(Debug, Clone, Copy)]
pub enum Grammar {
    Qwen3Xml,
}

#[derive(Debug, Clone)]
pub struct EgressReceipt {
    pub grammar: Grammar,
    pub wellformed: bool,
}

pub fn validate(grammar: Grammar, text: &str) -> EgressReceipt {
    let wellformed = match grammar {
        Grammar::Qwen3Xml => validate_qwen3_xml(text),
    };
    EgressReceipt {
        grammar,
        wellformed,
    }
}

fn validate_qwen3_xml(text: &str) -> bool {
    // Minimal: balanced tags + no stray `<` outside allowed.
    let mut stack: Vec<String> = Vec::new();
    let mut i = 0;
    let bytes = text.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'<' {
            // find closing >
            let end = match text[i..].find('>') {
                Some(e) => i + e + 1,
                None => return false,
            };
            let tag = &text[i + 1..end - 1];
            if tag.starts_with('/') {
                let name = tag.trim_start_matches('/');
                if stack.pop().as_deref() != Some(name) {
                    return false;
                }
            } else if tag.ends_with('/') {
                // self-closing ok
            } else {
                // tag name
                let name = tag.split_whitespace().next().unwrap_or("");
                if name.is_empty() {
                    return false;
                }
                stack.push(name.to_string());
            }
            i = end;
        } else {
            i += 1;
        }
    }
    stack.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    // 2026-10-30: Json-arm tests deleted with Grammar::Json (JSON ban).
    #[test]
    fn xml_valid() {
        assert!(validate(Grammar::Qwen3Xml, "<think>x</think>").wellformed);
    }
    #[test]
    fn xml_invalid() {
        assert!(!validate(Grammar::Qwen3Xml, "<a><b></a>").wellformed);
    }
}

//! Map-owned text splitting.
//!
//! This is NOT atomadb (the RAG). The RAG stores and retrieves; it does
//! not own the map's text layer. The map's judge, coordinates, and dedup
//! previously called `atomadb::tokenize`, which made map behaviour depend
//! on a RAG helper. That dependency is removed: the map owns this splitter.
//!
//! Behaviour is deliberately identical to the old splitter (lowercase,
//! split on non-alphanumeric, drop empties) so existing coordinates,
//! judge outcomes, and dedup decisions do not change. Replacing this
//! with a model tokenizer is a separate, coordinate-changing decision
//! and must not be done silently here.

/// Split text into lowercase alphanumeric terms, dropping empties.
pub fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitter_matches_old_atomadb_behaviour() {
        assert_eq!(
            tokenize("Hello, World! Don't stop."),
            vec!["hello", "world", "don", "t", "stop"]
        );
        assert_eq!(tokenize(""), Vec::<String>::new());
    }
}

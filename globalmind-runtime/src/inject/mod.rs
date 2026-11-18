//! Recall-into-forward injection for Queue 2 ablations 1-2 (new files only).
//!
//! [`tokenize`] encodes text with the rostered Qwen3.6 BPE exactly on its
//! enforced domain; [`ablate`] seeds memory from real traces, retrieves
//! neighborhoods, injects recalled text as tape tokens, and scores both
//! arms of the ablation on held-out suffixes.
pub mod ablate;
pub mod tokenize;

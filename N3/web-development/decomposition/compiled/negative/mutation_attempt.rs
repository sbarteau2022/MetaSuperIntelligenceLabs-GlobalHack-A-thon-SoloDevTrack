//! NEGATIVE PROOF (excluded from the build — no mod/extern reference).
//! Attempts to mutate a SealedPreregistration. Must FAIL to compile:
//!   error[E0594]: cannot assign to `s.task`, which is not declared as mutable
//!   error[E0616]: field `task` is private  (via accessor path, if attempted)
//!   error[E0599]: no method named `amend_mut` found
//! Paste of the actual rustc rejection lives in RECEIPT.md §4.

fn main() {
    let s = b2_decomp_prereg::preregistration::SealedPreregistration::seal(
        b2_decomp_prereg::preregistration::b2_plan(),
        "agent-b2",
        1_730_000_000,
        1,
    );
    s.task = String::from("MUTATED"); // must fail: field private + binding immutable
    s.amend_mut("sneaky"); // must fail: no such method
}

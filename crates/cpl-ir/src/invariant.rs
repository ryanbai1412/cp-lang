//! Lowering's one stop for a fact it trusts that its types don't prove yet
//! (handbook/guidelines/compiler-changes-checklist.md#compiler-changes-r16).
//! Every caller names the fact; each call goes when a type carries it.

pub(crate) fn broken_invariant(what: &str) -> ! {
    panic!("cpl-ir: broken invariant: {what}")
}

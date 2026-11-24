//! The crate's one stop for a fact the code guarantees but its types don't
//! carry yet (handbook/guidelines/compiler-changes-checklist.md R16). Each
//! call names the fact with a literal; `invariant_sites` counts them.

pub(crate) fn broken_invariant(what: &str) -> ! {
    panic!("cpl-codegen: broken invariant: {what}")
}

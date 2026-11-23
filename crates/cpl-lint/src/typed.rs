//! What a lint found on syntax alone, checked against the checker's facts.
//! `compound_self_ref` (L0210) needs the place's type: on an optional place
//! `x = x + e` keeps `none` and `x += e` is an error (decision 19.5), so it
//! stays only where the type is known and not optional. `unproven_list_index`
//! (L0414) is dropped where the checker proves the index by its type.

use crate::facts::Facts;
use crate::{COMPOUND_SELF_REF, UNPROVEN_LIST_INDEX};
use cpl_check::{Analysis, Ty};
use cpl_diag::{Diagnostic, Span};

const TYPED: [cpl_diag::Code; 2] = [COMPOUND_SELF_REF.code, UNPROVEN_LIST_INDEX.code];

/// `diags` less the findings `analysis`'s facts rule out.
pub fn refine_with_analysis(diags: &mut Vec<Diagnostic>, analysis: &Analysis) {
    if diags.iter().any(|d| TYPED.contains(&d.code)) {
        refine_with_facts(diags, &Facts::from_analysis(analysis));
    }
}

/// `diags` less the findings the facts of `src` rule out; `src` is checked
/// only when a finding needs it.
pub(crate) fn refine(diags: &mut Vec<Diagnostic>, src: &str) {
    if !diags.iter().any(|d| TYPED.contains(&d.code)) {
        return;
    }
    match Facts::check("<lint>.cp", src) {
        Some(facts) => refine_with_facts(diags, &facts),
        None => diags.retain(|d| d.code != COMPOUND_SELF_REF.code),
    }
}

pub(crate) fn refine_with_facts(diags: &mut Vec<Diagnostic>, facts: &Facts) {
    diags.retain(|d| match d.code {
        c if c == COMPOUND_SELF_REF.code => plain_place(d, facts),
        c if c == UNPROVEN_LIST_INDEX.code => facts.typed_index(d.span).is_none(),
        _ => true,
    });
}

/// Whether the place `diag` rewrites, from its start to where its fix
/// inserts `op=`, has a known type that is not optional.
fn plain_place(diag: &Diagnostic, facts: &Facts) -> bool {
    let Some(end) = diag
        .fixes
        .first()
        .and_then(|f| f.edits.first())
        .map(|(span, _)| span.start)
    else {
        return false;
    };
    facts
        .place_type(Span::new(diag.span.start, end))
        .is_some_and(|ty| !ty.has_vague() && !matches!(ty, Ty::Optional(_)))
}

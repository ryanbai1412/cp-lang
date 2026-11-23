//! What a lint found on syntax alone, checked against the types the checker
//! gives each place. `compound_self_ref` (L0210) is the one rule that needs
//! a type: on an optional place `x = x + e` keeps `none` and `x += e` is an
//! error (decision 19.5), so it stays only where the place's type is known
//! and not optional.

use crate::COMPOUND_SELF_REF;
use cpl_check::{Analysis, Ty};
use cpl_diag::{Diagnostic, Span};

/// `diags` less the findings `analysis`'s types rule out.
pub fn refine_with_analysis(diags: &mut Vec<Diagnostic>, analysis: &Analysis) {
    if !diags.iter().any(|d| d.code == COMPOUND_SELF_REF.code) {
        return;
    }
    let typed: Vec<(Span, &Ty)> = analysis
        .items
        .iter()
        .flat_map(|item| {
            let hovers = item.check.hovers.iter().map(|h| (h.span, &h.ty));
            let shapes = item.check.shapes.iter().map(|(s, t)| (*s, t));
            hovers
                .chain(shapes)
                .map(move |(s, t)| (s.shift(item.offset), t))
        })
        .collect();
    diags.retain(|d| d.code != COMPOUND_SELF_REF.code || plain_place(d, &typed));
}

/// `diags` less the findings the types of `src` rule out; `src` is checked
/// only when a finding needs it.
pub(crate) fn refine(diags: &mut Vec<Diagnostic>, src: &str) {
    if !diags.iter().any(|d| d.code == COMPOUND_SELF_REF.code) {
        return;
    }
    let mut db = cpl_check::Database::new();
    db.set_file("<lint>.cp", src);
    match db.analyze("<lint>.cp") {
        Some(analysis) => refine_with_analysis(diags, &analysis),
        None => diags.retain(|d| d.code != COMPOUND_SELF_REF.code),
    }
}

/// Whether the place `diag` rewrites, from its start to where its fix
/// inserts `op=`, has a known type that is not optional: the widest typed
/// span that ends there, a name, an index or a field.
fn plain_place(diag: &Diagnostic, typed: &[(Span, &Ty)]) -> bool {
    let Some(end) = diag
        .fixes
        .first()
        .and_then(|f| f.edits.first())
        .map(|(span, _)| span.start)
    else {
        return false;
    };
    typed
        .iter()
        .filter(|(s, _)| s.end == end && s.start >= diag.span.start)
        .min_by_key(|(s, _)| s.start)
        .is_some_and(|(_, ty)| !ty.has_vague() && !matches!(ty, Ty::Optional(_)))
}

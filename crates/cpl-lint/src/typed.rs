//! What a lint found on syntax alone, checked against the checker's facts.
//! `compound_self_ref` (L0210) needs the place's type: on an optional place
//! `x = x + e` keeps `none` and `x += e` is an error (decision 19.5), so it
//! stays only where the type is known and not optional. `unproven_list_index`
//! (L0414) is dropped where the checker proves the index by its type;
//! `stale_entry_skip` (L0553) stays only where the popped value is a heap.

use crate::facts::Facts;
use crate::{
    COMPOUND_SELF_REF, COPY_COMPREHENSION, FLOAT_EQUALITY, FLOAT_SQRT_FOR_INT, LOOP_REDUCTION,
    MANUAL_BIT_TEST, UNPROVEN_LIST_INDEX,
};
use cpl_diag::{Diagnostic, Span};
use cpl_facade::today::{Analysis, Ty};

/// `diags` less the findings `analysis`'s facts rule out.
pub fn refine_with_analysis(diags: &mut Vec<Diagnostic>, analysis: &Analysis) {
    refine_with_facts(diags, analysis.facts());
}

/// Lints a checked file the way `cpl lint` does: no lints on a file with
/// syntax errors, and the findings its facts rule out are dropped. The
/// editor and the playground call this on the analysis they already have.
pub fn lint_analysis(analysis: &Analysis, config: &crate::Config) -> Vec<Diagnostic> {
    if analysis.has_syntax_errors {
        return Vec::new();
    }
    lints_of(analysis, analysis.facts(), &Default::default(), config)
}

/// [`lint_analysis`] with `analysis`'s facts and the bounds proof, which a
/// [`crate::LintRun`] shares with the editor's hovers.
pub(crate) fn lints_of(
    analysis: &Analysis,
    facts: &Facts,
    proven: &std::sync::OnceLock<crate::rules::Proven>,
    config: &crate::Config,
) -> Vec<Diagnostic> {
    if analysis.has_syntax_errors {
        return Vec::new();
    }
    let items: Vec<_> = analysis
        .items
        .iter()
        .map(|i| (i.offset, &i.syntax.parsed))
        .collect();
    let library = crate::library::collect(analysis);
    let mut diags = crate::lint_parsed_library(
        &analysis.text,
        &items,
        Some(facts),
        config,
        &library,
        proven,
    );
    refine_with_facts(&mut diags, facts);
    crate::recheck::recheck(&mut diags, &analysis.text);
    crate::drop_repeats(&mut diags, &analysis.diagnostics);
    diags
}

pub(crate) fn refine_with_facts(diags: &mut Vec<Diagnostic>, facts: &Facts) {
    diags.retain(|d| keep(d, facts));
    for d in diags.iter_mut() {
        count_spelling(d, Some(facts));
        if !fix_checks(d, facts) {
            d.fixes.clear();
        }
    }
}

/// `loop_reduction` (L0504) writes a count as `xs.count(x => ..)`
/// (decision 53) where the checker types the walked value as a `Seq`, and
/// labels it with the generator to write anywhere else: a `Map` or a
/// product domain walks in `for` but has no `Seq` methods.
fn count_spelling(d: &mut Diagnostic, facts: Option<&Facts>) {
    if d.code != LOOP_REDUCTION.code || d.labels.is_empty() {
        return;
    }
    let label = d.labels.remove(0);
    let seq = facts
        .and_then(|f| f.place_type(label.span))
        .is_some_and(Ty::is_seq);
    if seq {
        return;
    }
    let Some(method) = d
        .help
        .as_deref()
        .and_then(|h| h.strip_prefix("write `"))
        .and_then(|h| h.split_once("`;"))
        .map(|(m, _)| m.to_string())
    else {
        return;
    };
    if let Some(help) = &mut d.help {
        *help = help.replacen(&method, &label.message, 1);
    }
    for (_, text) in d.fixes.iter_mut().flat_map(|f| &mut f.edits) {
        *text = text.replace(&method, &label.message);
    }
}

/// Whether the fix of the kept finding `d` means what the code it replaces does.
fn fix_checks(d: &Diagnostic, facts: &Facts) -> bool {
    match d.code {
        // `collect` copies a list or an array into the `List` the
        // comprehension builds; a `Str` or a generator may collect into
        // whatever the context asks for. A range literal, stepped or not, has
        // no label and collects into a `List` of its elements.
        c if c == COPY_COMPREHENSION.code => d.labels.first().is_none_or(|l| {
            matches!(facts.place_type(l.span), Some(Ty::Array { .. }))
                || facts.place_type(l.span).is_some_and(|t| t.is_list())
        }),
        _ => true,
    }
}

/// Whether `facts` leave the finding `d` standing.
fn keep(d: &Diagnostic, facts: &Facts) -> bool {
    let label = || d.labels.first().map(|l| facts.place_type(l.span));
    // A lint whose record names the types it is `on` keeps a finding only
    // where the checker types its first label as one of them.
    let on = crate::LINTS
        .iter()
        .find(|l| l.code == d.code)
        .map_or(&[][..], |l| l.on);
    if !on.is_empty() {
        return if d.code == UNPROVEN_LIST_INDEX.code {
            // A label names a list only where the checker's type for it is one.
            facts.typed_index(d.span).is_none()
                && label().is_none_or(|ty| ty.is_some_and(|t| t.is_one_of(on)))
        } else {
            label().is_some_and(|ty| ty.is_some_and(|t| t.is_one_of(on)))
        };
    }
    match d.code {
        c if c == COMPOUND_SELF_REF.code => plain_place(d, facts),
        c if c == FLOAT_SQRT_FOR_INT.code => {
            matches!(label(), Some(Some(Ty::Int | Ty::Range { .. })))
        }
        c if c == MANUAL_BIT_TEST.code => {
            matches!(label(), Some(Some(Ty::Int | Ty::Range { .. })))
        }
        // Kept unless the checker types the operand read as a float as an
        // integer (std/geometry.cp: `Pt.x` is an `Int`, `Vec.x` an `F64`).
        c if c == FLOAT_EQUALITY.code => !matches!(label(), Some(Some(ty)) if ty.is_int()),
        // `collect` is a `Seq` method: a `SortedSet` walks in `for` but has
        // none. Domains, generic `S: Seq` and generators are `Seq` too.
        c if c == COPY_COMPREHENSION.code => {
            !matches!(label(), Some(Some(ty @ Ty::Named { .. })) if !ty.is_seq())
        }
        _ => true,
    }
}

/// Whether the place `diag` rewrites, its first label or else from its
/// start to where its fix inserts `op=`, has a known type that is not
/// optional.
fn plain_place(diag: &Diagnostic, facts: &Facts) -> bool {
    plain_type(place_type(diag, facts))
}

/// `min=` and `max=` agree with `<` only on scalars: on a list or a tuple
/// they work elementwise, while `<` compares lexicographically.
fn place_type<'f>(diag: &Diagnostic, facts: &'f Facts) -> Option<&'f Ty> {
    if let Some(label) = diag.labels.first() {
        return facts.place_type(label.span);
    }
    let end = diag.fixes.first()?.edits.first()?.0.start;
    facts.place_type(Span::new(diag.span.start, end))
}

fn plain_type(ty: Option<&Ty>) -> bool {
    ty.is_some_and(|ty| !ty.has_vague() && !matches!(ty, Ty::Optional(_)))
}

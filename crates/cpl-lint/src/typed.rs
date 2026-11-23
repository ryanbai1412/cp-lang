//! What a lint found on syntax alone, checked against the checker's facts.
//! `compound_self_ref` (L0210) needs the place's type: on an optional place
//! `x = x + e` keeps `none` and `x += e` is an error (decision 19.5), so it
//! stays only where the type is known and not optional. `unproven_list_index`
//! (L0414) is dropped where the checker proves the index by its type;
//! `stale_entry_skip` (L0553) stays only where the popped value is a heap.

use crate::facts::Facts;
use crate::{
    CEIL_DIV_FORMULA, CHMIN_CHMAX_IF, COMPOUND_SELF_REF, CONTAINS_THEN_ADD, COPY_COMPREHENSION,
    COUNTER_BY_MAP, FLOAT_SQRT_FOR_INT, LAST_VIA_LEN, MANUAL_BIT_TEST, MANUAL_STARTS_WITH,
    MAP_AS_MULTISET, NEGATED_HEAP, NEG_MOD_FIXUP, PEEK_THEN_POP, POP_MIN_LOOP, PUSH_POP_PAIR,
    REVERSE_INDEX, SORT_FOR_EXTREME, STALE_ENTRY_SKIP, UNPROVEN_LIST_INDEX,
};
use cpl_check::{Analysis, Ty};
use cpl_diag::{Diagnostic, Span};

const TYPED: [cpl_diag::Code; 20] = [
    MANUAL_BIT_TEST.code,
    MANUAL_STARTS_WITH.code,
    LAST_VIA_LEN.code,
    FLOAT_SQRT_FOR_INT.code,
    REVERSE_INDEX.code,
    NEG_MOD_FIXUP.code,
    CEIL_DIV_FORMULA.code,
    PEEK_THEN_POP.code,
    COMPOUND_SELF_REF.code,
    CHMIN_CHMAX_IF.code,
    UNPROVEN_LIST_INDEX.code,
    STALE_ENTRY_SKIP.code,
    MAP_AS_MULTISET.code,
    NEGATED_HEAP.code,
    SORT_FOR_EXTREME.code,
    PUSH_POP_PAIR.code,
    POP_MIN_LOOP.code,
    COUNTER_BY_MAP.code,
    CONTAINS_THEN_ADD.code,
    COPY_COMPREHENSION.code,
];

/// `diags` less the findings `analysis`'s facts rule out.
pub fn refine_with_analysis(diags: &mut Vec<Diagnostic>, analysis: &Analysis) {
    if diags.iter().any(|d| TYPED.contains(&d.code)) {
        refine_with_facts(diags, &Facts::from_analysis(analysis));
    }
}

/// Lints a checked file the way `cpl lint` does: no lints on a file with
/// syntax errors, and the findings its facts rule out are dropped. The
/// editor and the playground call this on the analysis they already have.
pub fn lint_analysis(analysis: &Analysis, config: &crate::Config) -> Vec<Diagnostic> {
    if analysis.has_syntax_errors {
        return Vec::new();
    }
    let items: Vec<_> = analysis
        .items
        .iter()
        .map(|i| (i.offset, &i.syntax.parsed))
        .collect();
    let mut diags = crate::lint_parsed(&analysis.text, &items, config);
    refine_with_analysis(&mut diags, analysis);
    diags
}

/// `diags` less the findings the facts of `src` rule out; `src` is checked
/// only when a finding needs it.
pub(crate) fn refine(diags: &mut Vec<Diagnostic>, src: &str) {
    if !diags.iter().any(|d| TYPED.contains(&d.code)) {
        return;
    }
    match Facts::check("<lint>.cp", src) {
        Some(facts) => refine_with_facts(diags, &facts),
        None => diags.retain(|d| d.code == UNPROVEN_LIST_INDEX.code || !TYPED.contains(&d.code)),
    }
}

pub(crate) fn refine_with_facts(diags: &mut Vec<Diagnostic>, facts: &Facts) {
    diags.retain(|d| keep(d, facts));
    for d in diags.iter_mut() {
        if !fix_checks(d, facts) {
            d.fixes.clear();
        }
    }
}

/// Whether the fix of the kept finding `d` means what the code it replaces does.
fn fix_checks(d: &Diagnostic, facts: &Facts) -> bool {
    match d.code {
        // `collect` copies a list or an array into the `List` the
        // comprehension builds; a `Str` or a generator may collect into
        // whatever the context asks for. A range literal has no label: in a
        // generic fn `cpl run` has no `Range.collect` yet (std `sa_order`).
        c if c == COPY_COMPREHENSION.code => d.labels.first().is_some_and(|l| {
            matches!(facts.place_type(l.span), Some(Ty::Array { .. }))
                || matches!(facts.place_type(l.span), Some(Ty::Named { name, .. }) if name == "List")
        }),
        _ => true,
    }
}

/// Whether `facts` leave the finding `d` standing.
fn keep(d: &Diagnostic, facts: &Facts) -> bool {
    let label = || d.labels.first().map(|l| facts.place_type(l.span));
    match d.code {
        c if c == COMPOUND_SELF_REF.code || c == CHMIN_CHMAX_IF.code => plain_place(d, facts),
        c if c == PEEK_THEN_POP.code || c == REVERSE_INDEX.code => {
            matches!(label(), Some(Some(Ty::Named { name, .. })) if name == "List")
        }
        c if c == CEIL_DIV_FORMULA.code => match label() {
            None => true,
            Some(ty) => matches!(ty, Some(Ty::Range { lo: Some(lo), .. }) if *lo >= 1),
        },
        c if c == FLOAT_SQRT_FOR_INT.code => {
            matches!(label(), Some(Some(Ty::Int | Ty::Range { .. })))
        }
        c if c == NEG_MOD_FIXUP.code => match label() {
            None => true,
            Some(ty) => matches!(ty, Some(Ty::Int | Ty::I128 | Ty::Range { .. })),
        },
        c if c == MANUAL_BIT_TEST.code => {
            matches!(label(), Some(Some(Ty::Int | Ty::Range { .. })))
        }
        c if c == LAST_VIA_LEN.code => {
            matches!(label(), Some(Some(Ty::Str | Ty::Array { .. })))
                || matches!(label(), Some(Some(Ty::Named { name, .. })) if name == "List")
        }
        // Slices take keys and `starts_with` positions: they agree on `Str` and `List`.
        c if c == MANUAL_STARTS_WITH.code => {
            matches!(label(), Some(Some(Ty::Str)))
                || matches!(label(), Some(Some(Ty::Named { name, .. })) if name == "List")
        }
        c if c == STALE_ENTRY_SKIP.code => matches!(label(), Some(Some(Ty::Named { name, .. }))
            if ["Heap", "MaxHeap", "MinMaxHeap", "ErasableHeap", "MeldHeap"].contains(&name.as_str())),
        c if c == UNPROVEN_LIST_INDEX.code => facts.typed_index(d.span).is_none(),
        c if c == MAP_AS_MULTISET.code => matches!(label(), Some(Some(Ty::Named { name, args }))
            if name == "Map" && matches!(args.as_slice(), [_, Ty::Int | Ty::Range { .. }])),
        c if c == SORT_FOR_EXTREME.code => {
            matches!(label(), Some(Some(Ty::Array { .. })))
                || matches!(label(), Some(Some(Ty::Named { name, .. })) if name == "List")
        }
        c if c == COUNTER_BY_MAP.code => matches!(label(), Some(Some(Ty::Named { name, args }))
            if (name == "Map" || name == "HashMap") && matches!(args.as_slice(), [_, Ty::Int | Ty::Range { .. }])),
        // `collect` is a `Seq` method: a `SortedSet` walks in `for` but has
        // none. Domains, generic `S: Seq` and generators are `Seq` too.
        c if c == COPY_COMPREHENSION.code => {
            !matches!(label(), Some(Some(ty @ Ty::Named { .. })) if !ty.is_seq())
        }
        c if c == CONTAINS_THEN_ADD.code => {
            matches!(label(), Some(Some(Ty::Named { name, args }))
                if (name == "Map" || name == "HashMap") && matches!(args.as_slice(), [_, Ty::Int | Ty::Range { .. }]))
                || matches!(label(), Some(Some(Ty::Named { name, .. })) if name == "Counter")
        }
        c if c == POP_MIN_LOOP.code => {
            matches!(label(), Some(Some(Ty::Named { name, .. })) if name == "SortedBag" || name == "SortedSet")
        }
        c if c == PUSH_POP_PAIR.code => {
            matches!(label(), Some(Some(Ty::Named { name, .. })) if name == "Heap" || name == "MaxHeap")
        }
        c if c == NEGATED_HEAP.code => {
            matches!(label(), Some(Some(Ty::Named { name, .. })) if name == "Heap")
        }
        _ => true,
    }
}

/// Whether the place `diag` rewrites, its first label or else from its
/// start to where its fix inserts `op=`, has a known type that is not
/// optional.
fn plain_place(diag: &Diagnostic, facts: &Facts) -> bool {
    if let Some(label) = diag.labels.first() {
        return plain_type(facts.place_type(label.span));
    }
    let Some(end) = diag
        .fixes
        .first()
        .and_then(|f| f.edits.first())
        .map(|(span, _)| span.start)
    else {
        return false;
    };
    plain_type(facts.place_type(Span::new(diag.span.start, end)))
}

fn plain_type(ty: Option<&Ty>) -> bool {
    ty.is_some_and(|ty| !ty.has_vague() && !matches!(ty, Ty::Optional(_)))
}

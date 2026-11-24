//! What a lint found on syntax alone, checked against the checker's facts.
//! `compound_self_ref` (L0210) needs the place's type: on an optional place
//! `x = x + e` keeps `none` and `x += e` is an error (decision 19.5), so it
//! stays only where the type is known and not optional. `unproven_list_index`
//! (L0414) is dropped where the checker proves the index by its type;
//! `stale_entry_skip` (L0553) stays only where the popped value is a heap.

use crate::facts::Facts;
use crate::{
    CEIL_DIV_FORMULA, CHMIN_CHMAX_IF, COMPOUND_SELF_REF, CONTAINS_THEN_ADD, COPY_COMPREHENSION,
    COUNTER_BY_MAP, FLOAT_EQUALITY, FLOAT_SQRT_FOR_INT, LAST_VIA_LEN, LOOP_REDUCTION,
    MANUAL_BIT_TEST, MANUAL_STARTS_WITH, MAP_AS_MULTISET, NEGATED_HEAP, NEG_MOD_FIXUP,
    PEEK_THEN_POP, POP_MIN_LOOP, PUSH_POP_PAIR, REVERSE_INDEX, SORT_FOR_EXTREME, STALE_ENTRY_SKIP,
    UNPROVEN_LIST_INDEX,
};
use cpl_check::{Analysis, Ty};
use cpl_diag::{Diagnostic, Span};

const TYPED: [cpl_diag::Code; 22] = [
    LOOP_REDUCTION.code,
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
    FLOAT_EQUALITY.code,
];

/// `diags` less the findings `analysis`'s facts rule out.
pub fn refine_with_analysis(diags: &mut Vec<Diagnostic>, analysis: &Analysis) {
    if diags.iter().any(|d| TYPED.contains(&d.code)) {
        refine_with_facts(diags, analysis.facts());
    }
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
/// [`crate::Session`] shares with the editor's hovers.
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
    if diags.iter().any(|d| TYPED.contains(&d.code)) {
        refine_with_facts(&mut diags, facts);
    }
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
        c if c == CHMIN_CHMAX_IF.code => scalar_place(d, facts),
        c if c == COMPOUND_SELF_REF.code => plain_place(d, facts),
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
        // A label names a list only the checker's type for it shows is one.
        c if c == UNPROVEN_LIST_INDEX.code => {
            facts.typed_index(d.span).is_none()
                && label()
                    .is_none_or(|ty| matches!(ty, Some(Ty::Named { name, .. }) if name == "List"))
        }
        c if c == MAP_AS_MULTISET.code => matches!(label(), Some(Some(Ty::Named { name, args }))
            if name == "Map" && matches!(args.as_slice(), [_, Ty::Int | Ty::Range { .. }])),
        c if c == SORT_FOR_EXTREME.code => {
            matches!(label(), Some(Some(Ty::Array { .. })))
                || matches!(label(), Some(Some(Ty::Named { name, .. })) if name == "List")
        }
        c if c == COUNTER_BY_MAP.code => matches!(label(), Some(Some(Ty::Named { name, args }))
            if (name == "Map" || name == "HashMap") && matches!(args.as_slice(), [_, Ty::Int | Ty::Range { .. }])),
        // Kept unless the checker types the operand read as a float as an
        // integer (std/geometry.cp: `Pt.x` is an `Int`, `Vec.x` an `F64`).
        c if c == FLOAT_EQUALITY.code => !matches!(label(), Some(Some(ty)) if ty.is_int()),
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
    plain_type(place_type(diag, facts))
}

/// `min=` and `max=` agree with `<` only on scalars: on a list or a tuple
/// they work elementwise, while `<` compares lexicographically.
fn scalar_place(diag: &Diagnostic, facts: &Facts) -> bool {
    place_type(diag, facts).is_some_and(|ty| ty.is_numeric() || matches!(ty, Ty::Char | Ty::Pos(_)))
}

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

/// `text` with the lint fixes `cpl lint --fix` would make under `config`,
/// and how many; each round lints the analysis `db` has, so a clean file is
/// checked once. `cpl fix` and the playground's `fix` share it. A round is
/// re-checked: a fix that would add a checker error is not applied.
pub fn apply_lint_fixes(
    db: &mut cpl_check::Database,
    path: &str,
    mut text: String,
    config: &crate::Config,
) -> (String, usize) {
    let mut applied = 0;
    for _ in 0..8 {
        db.set_file(path, text.as_str());
        let analysis = db.analyze(path).expect("the file was just set");
        let findings = lint_analysis(&analysis, config);
        let errors = error_codes(&analysis.diagnostics);
        let (next, n) = safe_fixes(db, path, &text, &findings, &errors);
        if n == 0 {
            break;
        }
        applied += n;
        text = next;
    }
    db.set_file(path, text.as_str());
    (text, applied)
}

/// The checker errors in `diagnostics`, as sorted codes.
fn error_codes(diagnostics: &[cpl_diag::Diagnostic]) -> Vec<&'static str> {
    let mut codes: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.severity == cpl_diag::Severity::Error)
        .map(|d| d.code.0)
        .collect();
    codes.sort_unstable();
    codes
}

/// Whether `text` as `path` has no checker error beyond `errors`.
fn adds_no_error(db: &mut cpl_check::Database, path: &str, text: &str, errors: &[&str]) -> bool {
    db.set_file(path, text);
    let Some(analysis) = db.analyze(path) else {
        return false;
    };
    let mut left = errors.to_vec();
    for code in error_codes(&analysis.diagnostics) {
        match left.iter().position(|c| *c == code) {
            Some(at) => {
                left.remove(at);
            }
            None => return false,
        }
    }
    true
}

/// `findings`' fixes applied to `text`, keeping only fixes after which the
/// checker reports no error that `errors` lacks: all at once when that is
/// safe, else each one that is safe alone, else the first such.
pub(crate) fn safe_fixes(
    db: &mut cpl_check::Database,
    path: &str,
    text: &str,
    findings: &[cpl_diag::Diagnostic],
    errors: &[&str],
) -> (String, usize) {
    let (all, n) = crate::apply_fixes(text, findings);
    if n == 0 || adds_no_error(db, path, &all, errors) {
        return (all, n);
    }
    let safe: Vec<_> = findings
        .iter()
        .filter(|f| !f.fixes.is_empty())
        .filter(|f| {
            let (alone, n) = crate::apply_fixes(text, std::slice::from_ref(*f));
            n > 0 && adds_no_error(db, path, &alone, errors)
        })
        .cloned()
        .collect();
    let (kept, n) = crate::apply_fixes(text, &safe);
    if n == 0 || adds_no_error(db, path, &kept, errors) {
        return (kept, n);
    }
    crate::apply_fixes(text, &safe[..1])
}

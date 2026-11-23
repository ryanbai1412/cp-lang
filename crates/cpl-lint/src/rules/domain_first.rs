//! `domain_empty_first_last` (L0304) and `unwrap_on_possibly_empty`
//! (L0305): `D.first!` on a domain, or `a.max()!` / `a.first!` on an array
//! over one, or `a[l..r].first!` on a slice, where the declared bounds let
//! it be empty, so the unwrap traps on that input.

use super::bounds::Bounds;
use crate::walk::{unparen, Visitor};
use crate::{File, Finding, Lint, DOMAIN_EMPTY_FIRST_LAST, UNWRAP_ON_POSSIBLY_EMPTY};
use cpl_diag::{Diagnostic, Span};
use cpl_syntax::ast::{Expr, ExprKind, Pattern, PatternKind, StmtKind, Type, TypeKind};

/// What empties a domain or an array: its declared index set, the end
/// that does it and the value it takes then.
struct Empty {
    name: String,
    shown: String,
    end: String,
    at: i128,
    domain: bool,
}

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let bounds = Bounds::of(file);
    // The end that can empty `lo..hi` (`lo..=hi`), with the value it takes then.
    let empties = |lo: &Expr, hi: &Expr, inclusive: bool| -> Option<(Expr, i128)> {
        let at = super::proven::hazards::can_be_empty(lo, hi, inclusive, &bounds)?;
        matches!(unparen(hi).kind, ExprKind::Name(_)).then(|| (hi.clone(), at))
    };
    let mut empty: Vec<Empty> = Vec::new();
    for (base, stmt) in file.top_level() {
        let text = |span: cpl_diag::Span| span.shift(base).text(file.src).to_string();
        match &stmt.kind {
            StmtKind::Domain { name, def } => {
                if let ExprKind::Range {
                    lo: Some(lo),
                    hi: Some(hi),
                    inclusive,
                    step: None,
                } = &def.kind
                {
                    if let Some((end, at)) = empties(lo, hi, *inclusive) {
                        empty.push(Empty {
                            name: name.text.clone(),
                            shown: format!("{} = {}", name.text, text(def.span)),
                            end: text(end.span),
                            at,
                            domain: true,
                        });
                    }
                }
            }
            StmtKind::Let {
                pat, ty: Some(ty), ..
            } => {
                let mut arrays = Vec::new();
                named_types(pat, ty, &mut arrays);
                for (name, ty) in arrays {
                    let TypeKind::Array { index, .. } = &ty.kind else {
                        continue;
                    };
                    let [first] = index.as_slice() else {
                        continue;
                    };
                    let found = match &first.kind {
                        TypeKind::Range {
                            lo: Some(lo),
                            hi: Some(hi),
                            inclusive,
                        } => empties(lo, hi, *inclusive).map(|(e, at)| (text(e.span), at)),
                        TypeKind::Named { name: d, .. } => empty
                            .iter()
                            .find(|e| e.domain && e.name == d.text)
                            .map(|e| (e.end.clone(), e.at)),
                        _ => None,
                    };
                    if let Some((end, at)) = found {
                        empty.push(Empty {
                            name: name.to_string(),
                            shown: format!("{name}: {}", text(ty.span)),
                            end,
                            at,
                            domain: false,
                        });
                    }
                }
            }
            _ => {}
        }
    }
    // A file that tests the end or the length anywhere may guard the
    // unwrap: leave it alone.
    let guarded = |e: &Empty| {
        let (end, name) = (&e.end, &e.name);
        [
            format!("{end} > 0"),
            format!("{end} >= 1"),
            format!("{end} == 0"),
            format!("{end} != 0"),
            format!("{name}.len"),
            format!("{name}.is_empty"),
        ]
        .iter()
        .any(|g| file.src.contains(g.as_str()))
    };
    empty.retain(|e| !guarded(e));
    let mut slices: Vec<Finding> = super::proven::hazards::proven_empty_slices(file)
        .into_iter()
        .filter(|(at, _)| !super::wide_shift::in_test_fn(file, at.0))
        .map(|((start, end), (lo, hi))| {
            let shown = Span::new(start, end).text(file.src);
            let read = &shown[..shown.len() - 1];
            let range = Span::new(lo, hi).text(file.src);
            let diag = Diagnostic::warning(
                UNWRAP_ON_POSSIBLY_EMPTY.code,
                Span::new(start, end),
                format!("the declared bounds let `{range}` be empty, and then `{shown}` traps"),
            )
            .with_help(format!(
                "`{read}` is `None` on an empty slice: handle it with `if let v = {read}:` or `{read} ?? fallback`, or declare the ends so `{range}` is never empty"
            ));
            Finding {
                lint: &UNWRAP_ON_POSSIBLY_EMPTY,
                diag,
            }
        })
        .collect();
    if empty.is_empty() {
        return slices;
    }
    struct Unwraps<'a> {
        src: &'a str,
        base: u32,
        empty: &'a [Empty],
        out: Vec<Finding>,
    }
    impl Visitor for Unwraps<'_> {
        fn expr(&mut self, expr: &Expr) {
            let ExprKind::Unwrap(inner) = &expr.kind else {
                return;
            };
            // Every member of a sequence or domain read with no argument
            // and unwrapped (`first`, `last`, `min()`, `argmax()`, ..) is
            // `None` exactly when it is empty.
            let recv = match &unparen(inner).kind {
                ExprKind::Field { base, .. } => base,
                ExprKind::Call { callee, args } if args.is_empty() => {
                    let ExprKind::Field { base, .. } = &unparen(callee).kind else {
                        return;
                    };
                    base
                }
                _ => return,
            };
            let ExprKind::Name(target) = &unparen(recv).kind else {
                return;
            };
            let Some(e) = self.empty.iter().find(|e| e.name == *target) else {
                return;
            };
            let lint: &'static Lint = if e.domain {
                &DOMAIN_EMPTY_FIRST_LAST
            } else {
                &UNWRAP_ON_POSSIBLY_EMPTY
            };
            let span = expr.span.shift(self.base);
            let shown = span.text(self.src);
            let read = &shown[..shown.len() - 1];
            let (name, end, at) = (&e.name, &e.end, e.at);
            let what = if e.domain { "domain" } else { "list" };
            let diag = Diagnostic::warning(
                lint.code,
                span,
                format!(
                    "`{}` is empty when `{end}` is {at}, and then `{shown}` traps",
                    e.shown
                ),
            )
            .with_help(format!(
                "`{read}` is `None` on an empty {what}: handle it with `if let v = {read}:` or `{read} ?? fallback`, or declare `{end}` so `{name}` is never empty"
            ));
            self.out.push(Finding { lint, diag });
        }
    }
    let mut v = Unwraps {
        src: file.src,
        base: 0,
        empty: &empty,
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out.append(&mut slices);
    v.out
}

/// The names a `let` pattern binds, each with its declared type.
fn named_types<'a>(pat: &'a Pattern, ty: &'a Type, out: &mut Vec<(&'a str, &'a Type)>) {
    match (&pat.kind, &ty.kind) {
        (PatternKind::Name(name), _) => out.push((name, ty)),
        (PatternKind::Tuple(pats), TypeKind::Tuple(tys)) if pats.len() == tys.len() => {
            for (p, t) in pats.iter().zip(tys) {
                named_types(p, t, out);
            }
        }
        _ => {}
    }
}

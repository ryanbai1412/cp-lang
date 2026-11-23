//! Lent views: which kind an expression is, whether it is read-only, and
//! the keys it keeps (R07.4.5, decision 388): a slice, run, window or chunk
//! keeps its source's keys, and a `.rev` view walks out of key order.

use super::lends::{show, unparen};
use super::*;
use cpl_syntax::ast::{self, Expr, ExprKind};

/// A view that keeps its source's keys: what `w[0]` would name.
#[derive(Clone)]
pub(crate) struct Keyed {
    /// `lo..hi` when both ends are literals.
    pub(crate) keys: Option<(i128, i128)>,
    /// The source, as written.
    pub(crate) source: String,
    /// `.rev`: walks out of key order, so it has no `[]`.
    pub(crate) rev: bool,
}

impl Checker<'_> {
    /// The kind of lent view an expression is, if it is one.
    pub(crate) fn view_kind(&self, e: &Expr) -> Option<&'static str> {
        let e = unparen(e);
        if let Some(kind) = self.lends.view_exprs.get(&e.span) {
            return Some(kind);
        }
        match &e.kind {
            ExprKind::Name(name) => match self.local(name) {
                Some(local) => self.lends.views.get(&local.span).map(|lent| lent.kind),
                None => self.lends.global_views.get(name).map(|lent| lent.kind),
            },
            _ => None,
        }
    }

    /// True if `e` is a read-only lent view (`Str`/`List` slice or `.rev`).
    pub(crate) fn is_readonly_view(&self, e: &Expr) -> bool {
        let e = unparen(e);
        if self.lends.view_readonly.contains(&e.span) {
            return true;
        }
        match &e.kind {
            ExprKind::Name(name) => match self.local(name) {
                Some(local) => self
                    .lends
                    .views
                    .get(&local.span)
                    .is_some_and(|lent| lent.read_only),
                None => self
                    .lends
                    .global_views
                    .get(name)
                    .is_some_and(|lent| lent.read_only),
            },
            _ => false,
        }
    }

    /// R07.4.5: `w[x]` on a view takes a key of its source. `rev` has no
    /// `[]`; a literal not proven one of `w.keys` is meant as a position.
    pub(super) fn keyed_view_index(&mut self, base: &Expr, indices: &[Expr]) {
        let [index] = indices else {
            return;
        };
        let w = self.quote(base.span);
        let rev = self.keyed_view(base).is_some_and(|keyed| keyed.rev);
        if rev || self.view_kind(base) == Some("reverse view") {
            let message =
                format!("`{w}` walks backwards, out of key order, so it can't be indexed");
            if let Some(diag) = self.error(codes::NOT_INDEXABLE, base.span, message) {
                diag.help = Some(format!(
                    "write `{w}.nth(k)` for the `k`th element it walks, or index the source by its key"
                ));
            }
            return;
        }
        let ExprKind::Int(v) = crate::check::lends::unparen(index).kind else {
            return;
        };
        let Some(keyed) = self.keyed_view(base) else {
            return;
        };
        if let Ty::Array { index, .. } = self.quietly(|c| c.infer(base)) {
            if let [Ty::Domain(d) | Ty::Pos(d)] = index.as_slice() {
                let message = format!(
                    "`{w}` is a slice of a `{d}` table, so it is indexed by `{d}`, not by the raw `Int` {v}"
                );
                if let Some(diag) = self.error(codes::WRONG_DOMAIN, indices[0].span, message) {
                    diag.help = Some(format!(
                        "index it with a `{d}`: `{w}[{d}({v})]`, or use `{w}.nth({v})!`"
                    ));
                }
                return;
            }
        }
        if keyed.keys.is_some_and(|(lo, hi)| lo <= v && v < hi) {
            let message = format!(
                "`{w}[{v}]` reads source key {v}: a slice keeps its source's keys, so this is not its element {v}"
            );
            self.sink.push(
                Diagnostic::warning(codes::LITERAL_SLICE_INDEX, index.span, message).with_help(
                    format!(
                        "write `{w}.nth({v})` (or `{w}.first!`) for the element by position, or `{w}.rebase` to count from 0"
                    ),
                ),
            );
            return;
        }
        let message = format!(
            "`{w}[{v}]` reads key {v}, which isn't known to be one of `{w}.keys`: a view keeps the keys of `{}`",
            keyed.source
        );
        if let Some(diag) = self.error(codes::NOT_INDEXABLE, index.span, message) {
            diag.help = Some(format!(
                "write `{w}.nth({v})` for its element {v}, or `{}[{v}]` for the source's key {v}",
                keyed.source
            ));
        }
    }

    /// `e` as a view keyed by its source's keys: a `List`/`Str` slice, or
    /// a run, window or chunk of one (R07.4.5).
    pub(crate) fn keyed_view(&mut self, e: &Expr) -> Option<Keyed> {
        let e = unparen(e);
        match &e.kind {
            ExprKind::Name(name) => match self.local(name) {
                Some(local) => self.lends.keyed.get(&local.span).cloned(),
                None => self.lends.global_keyed.get(name).cloned(),
            },
            ExprKind::Index { base, indices } => {
                if let Some(source) = self.pieces_source(base) {
                    return Some(Keyed {
                        keys: None,
                        source,
                        rev: false,
                    });
                }
                if self.view_kind(e) != Some("slice") {
                    return None;
                }
                let [range] = indices.as_slice() else {
                    return None;
                };
                let ExprKind::Range {
                    lo,
                    hi,
                    inclusive,
                    step: None,
                } = &unparen(range).kind
                else {
                    return None;
                };
                let int = |b: &Option<Box<Expr>>| match b.as_deref().map(|b| &unparen(b).kind) {
                    Some(ExprKind::Int(v)) => Some(*v),
                    _ => None,
                };
                // An open end is the source's own bound when its type fixes it: `[1..=4]T`.
                let (src_lo, src_hi) = match self.quietly(|c| c.infer(base)) {
                    Ty::Array { index, .. } => match index.as_slice() {
                        [Ty::Range { lo, hi, .. }] => (*lo, hi.map(|h| h + 1)),
                        _ => (None, None),
                    },
                    _ => (None, None),
                };
                let lo_key = if lo.is_some() { int(lo) } else { src_lo };
                let hi_key = if hi.is_some() {
                    int(hi).map(|h| h + *inclusive as i128)
                } else {
                    src_hi
                };
                let keys = lo_key.zip(hi_key);
                Some(Keyed {
                    keys,
                    source: show(base),
                    rev: false,
                })
            }
            _ => None,
        }
    }

    /// `keyed_view`, or a `.rev` view, which keeps keys but has no `[]`.
    pub(crate) fn keyed_or_rev(&mut self, e: &Expr) -> Option<Keyed> {
        if self.view_kind(e) == Some("reverse view") {
            return Some(Keyed {
                keys: None,
                source: String::new(),
                rev: true,
            });
        }
        self.keyed_view(e)
    }

    /// The receiver of `xs.runs()`, `xs.windows(k)` or `xs.chunks(k)` on a
    /// `List` or `Str`: each piece keeps its keys.
    pub(crate) fn pieces_source(&mut self, e: &Expr) -> Option<String> {
        let ExprKind::Call { callee, args } = &unparen(e).kind else {
            return None;
        };
        // `xs.runs()`, or the same call written `runs(xs)`.
        let (base, name) = match (&callee.kind, args.first()) {
            (ExprKind::Field { base, name }, _) => (&**base, name.text.as_str()),
            (ExprKind::Name(name), Some(first)) => (&first.value, name.as_str()),
            _ => return None,
        };
        if !matches!(name, "runs" | "windows" | "chunks") {
            return None;
        }
        let ty = self.quietly(|c| c.infer(base));
        let seq = ty == Ty::Str || matches!(&ty, Ty::Named { name, .. } if name == "List");
        seq.then(|| show(base))
    }

    /// `for r in xs.runs()` or `for (k, r) in xs.runs().items`: `r` keeps
    /// `xs`'s keys.
    pub(crate) fn note_pieces(&mut self, pat: &ast::Pattern, iter: &Expr) {
        let mut iter = unparen(iter);
        let mut pat = pat;
        if let ExprKind::Field { base, name } = &iter.kind {
            if name.text != "items" {
                return;
            }
            let ast::PatternKind::Tuple(parts) = &pat.kind else {
                return;
            };
            let [_, piece] = parts.as_slice() else {
                return;
            };
            iter = unparen(base);
            pat = piece;
        }
        let ast::PatternKind::Name(name) = &pat.kind else {
            return;
        };
        let Some(source) = self.pieces_source(iter) else {
            return;
        };
        if let Some(decl) = self.local(name).map(|local| local.span) {
            self.lends.keyed.insert(
                decl,
                Keyed {
                    keys: None,
                    source,
                    rev: false,
                },
            );
        }
    }
}

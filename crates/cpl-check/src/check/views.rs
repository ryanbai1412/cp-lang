//! Lent views: which kind an expression is, whether it is read-only, and
//! the keys it keeps (R07.4.5, decision 388): a slice, run, window or chunk
//! keeps its source's keys, and a `.rev` view walks out of key order.

use super::lends::{show, unparen};
use super::*;
use cpl_syntax::ast::{self, Expr, ExprKind};

/// A view that keeps its source's keys: what `w[0]` would name.
#[derive(Clone)]
pub(crate) struct Keyed {
    /// `lo..hi` when both ends are literals; an open end is `i128::MAX`.
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
    pub(super) fn keyed_view_index(&mut self, whole: Span, base: &Expr, indices: &[Expr]) {
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
            let (by_position, by_key) = (format!("{w}.nth({v})"), format!("{}[{v}]", keyed.source));
            diag.help = Some(format!(
                "write `{by_position}` for its element {v}, or `{by_key}` for the source's key {v}"
            ));
            diag.fix(format!("write `{by_position}`"), whole, by_position);
            diag.fix(format!("write `{by_key}`"), whole, by_key);
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
                // `s[2..]` runs to the source's end: every key from 2 on is its own.
                let hi_key = if hi.is_some() {
                    int(hi).map(|h| h + *inclusive as i128)
                } else {
                    src_hi.or(Some(i128::MAX))
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

    /// `.rev` itself, or a member `Rev` declares (`a.rev.take(k)`): not
    /// passed on to the source the view reverses.
    pub(super) fn rev_owns(&mut self, name: &str) -> bool {
        name == "rev"
            || self
                .members_of("Rev")
                .is_some_and(|m| m.members.iter().any(|(n, _)| n == name))
    }

    /// A method whose `self` must be `Indexed`, called on a `.rev`: a
    /// reversed view keeps its source's keys, so it isn't (R07.3.5).
    pub(super) fn rev_not_indexed(&mut self, base_ty: &Ty, name: &ast::Name) -> bool {
        let def = self.method_def(base_ty, &name.text);
        let Some(Def::Fn {
            ty: Ty::Fn { params, .. },
            bounds,
            ..
        }) = def.as_deref()
        else {
            return false;
        };
        let indexed = match params.first() {
            Some(crate::Param {
                name: this,
                ty: Ty::Param(p),
                ..
            }) if this == "self" => bounds
                .iter()
                .any(|(b, traits)| b == p && traits.iter().any(|t| t == "Indexed")),
            _ => false,
        };
        if indexed {
            let message = format!(
                "`{base_ty}` does not implement `Indexed` required by `{}`",
                name.text
            );
            if let Some(diag) = self.error(codes::MISSING_IMPL, name.span, message) {
                diag.help =
                    Some("a reversed view isn't Indexed; copy it with `List(a.rev)`".into());
            }
        }
        indexed
    }

    /// A member of a `Slice<S>` or a `.rev` the view doesn't declare: the
    /// source's, unless the `.rev` would need to be `Indexed` for it.
    pub(super) fn member_through_view(
        &mut self,
        expr: &Expr,
        name: &ast::Name,
        base_ty: &Ty,
    ) -> Option<Ty> {
        if let Some(cut) = Self::cut_member(base_ty, &name.text) {
            return Some(cut);
        }
        if let Some(source) = self.slice_source(base_ty, &name.text) {
            return Some(self.member_ty(expr, name, source));
        }
        let source = base_ty.rev_source()?.clone();
        if self.rev_owns(&name.text) {
            return None;
        }
        if self.rev_not_indexed(base_ty, name) {
            return Some(Ty::Error);
        }
        Some(self.member_ty(expr, name, source))
    }

    /// The sequence a `Slice<S>` cuts, for a member `Slice` doesn't declare:
    /// a slice has its source's members (R07.4.5).
    pub(super) fn slice_source(&mut self, ty: &Ty, name: &str) -> Option<Ty> {
        let Ty::Named { name: slice, args } = ty else {
            return None;
        };
        if slice != "Slice" || args.len() != 1 {
            return None;
        }
        let own = self
            .members_of("Slice")
            .is_some_and(|m| m.members.iter().any(|(n, _)| n == name));
        (!own).then(|| args[0].clone())
    }
}

/// Where a lent value would be kept: a variable being declared here, or
/// the variable at the root of a place.
pub(crate) enum Keep<'e> {
    New(Span),
    In(&'e Expr),
    /// A `let`: it lives no longer than any named source in its block, so
    /// only a temporary source can die before it.
    Let,
}

impl Keep<'_> {
    pub(crate) fn binding(mutable: bool, at: Span) -> Self {
        if mutable {
            Keep::New(at)
        } else {
            Keep::Let
        }
    }
}

impl Checker<'_> {
    /// Reports the lent `e` kept in `keep` unless that storage dies no later
    /// than `e`'s source: it is declared in the source's block or one nested
    /// in it, after the source (R05.7.8). A global source outlives all
    /// storage but a global declared before it; a global storage outlives
    /// every local source.
    pub(crate) fn escape_unless_kept(&mut self, e: &Expr, what: &str, keep: Keep<'_>) {
        let is_let = matches!(keep, Keep::Let);
        let global_storage = match keep {
            Keep::In(place) => super::lends::root_name(unparen(place))
                .filter(|r| self.local(r).is_none())
                .map(str::to_string),
            _ => None,
        };
        let storage = match keep {
            Keep::Let => None,
            Keep::New(at) => Some((self.scopes.len() - 1, at)),
            Keep::In(place) => super::lends::root_name(unparen(place))
                .and_then(|r| self.local(r).map(|l| (self.var_depth(r), l.span))),
        };
        if let Some(temp) = self.temp_source(e) {
            let (src, temp) = (self.quote(e.span), self.quote(temp.span));
            let message = format!(
                "{what} would keep `{src}`, a view of the temporary `{temp}`, which lives only to the end of this statement"
            );
            if let Some(diag) = self.error(codes::LENT_ESCAPES, e.span, message) {
                diag.help = Some(format!(
                    "name the source first (`let s = {temp}`), or keep a copy: `List({src})`"
                ));
            }
            return;
        }
        if is_let {
            return;
        }
        let kept = super::lends::root_name(unparen(e)).is_some_and(|root| match self.local(root) {
            // Top-level storage outlives a top-level source declared after
            // it: the file body is a block like any other (R05.7.10).
            None => global_storage.as_deref().is_none_or(|kept_in| {
                match (self.env.decl_order(root), self.env.decl_order(kept_in)) {
                    (Some(source), Some(kept_in)) => source <= kept_in,
                    _ => true,
                }
            }),
            Some(source) => storage.is_some_and(|(depth, at)| {
                self.var_depth(root) <= depth && source.span.start <= at.start
            }),
        });
        if !kept {
            self.no_escape(e, what);
        }
    }
}

/// The bound a generic fn's later `Indexed` parameter has when its body
/// reads it only by walk position (`.len`, `.nth(k)`, `.is_empty`): it
/// also takes the `.rev` of an `Indexed`, so `zip(a, a.rev)` pairs mirrors
/// (R07.6.4). The first parameter keys the result and stays `Indexed`.
/// Only the checker writes it: no program can name it.
pub(crate) const INDEXED_BY_WALK: &str = "Indexed (or its .rev)";

pub(crate) fn by_walk_bounds(
    f: &ast::FnDecl,
    mut bounds: Vec<(String, Vec<String>)>,
) -> Vec<(String, Vec<String>)> {
    for (param, traits) in &mut bounds {
        let typed: Vec<(usize, &ast::Param)> = f
            .params
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                matches!(p.ty.as_ref().map(|t| &t.kind),
                    Some(ast::TypeKind::Named { name, args }) if args.is_empty() && name.text == *param)
            })
            .collect();
        let by_walk = !typed.is_empty()
            && typed
                .iter()
                .all(|(i, p)| *i > 0 && read_by_walk(&f.body, &p.name.text));
        if by_walk {
            for t in traits.iter_mut().filter(|t| *t == "Indexed") {
                *t = INDEXED_BY_WALK.to_string();
            }
        }
    }
    bounds
}

/// Every use of the name `param` in `body` is `.len`, `.nth` or `.is_empty`.
fn read_by_walk(body: &ast::FnBody, param: &str) -> bool {
    struct Uses<'p> {
        param: &'p str,
        all: usize,
        by_walk: usize,
    }
    impl cpl_syntax::visit::Visitor for Uses<'_> {
        fn expr(&mut self, e: &Expr) {
            match &e.kind {
                ExprKind::Name(n) if n == self.param => self.all += 1,
                ExprKind::Field { base, name }
                    if matches!(&base.kind, ExprKind::Name(n) if n == self.param)
                        && matches!(name.text.as_str(), "len" | "nth" | "is_empty") =>
                {
                    self.by_walk += 1
                }
                _ => {}
            }
        }
    }
    let mut uses = Uses {
        param,
        all: 0,
        by_walk: 0,
    };
    match body {
        ast::FnBody::Block(b) => cpl_syntax::visit::walk_block(b, &mut uses),
        ast::FnBody::Expr(e) => cpl_syntax::visit::walk_expr(e, &mut uses),
        ast::FnBody::None => return false,
    }
    uses.all > 0 && uses.all == uses.by_walk
}

pub(crate) fn binding_kw(mutable: bool) -> &'static str {
    if mutable {
        "var"
    } else {
        "let"
    }
}

impl Checker<'_> {
    /// The unnamed temporary a view is cut from, if any (`[3, 1, 2].sorted()`
    /// in `[3, 1, 2].sorted()[0..1]`). It dies at the end of its statement,
    /// so nothing may keep the view (R07.5.10, R05.7.8).
    fn temp_source<'e>(&mut self, e: &'e Expr) -> Option<&'e Expr> {
        let mut e = unparen(e);
        self.view_kind(e)?;
        loop {
            e = match &e.kind {
                ExprKind::Paren(inner) | ExprKind::Unwrap(inner) => inner,
                ExprKind::Index { base, .. } | ExprKind::Field { base, .. } => base,
                ExprKind::Call { callee, .. }
                    if self.view_kind(e).is_some()
                        || self.lends.proj_calls.contains_key(&e.span) =>
                {
                    match &unparen(callee).kind {
                        ExprKind::Field { base, .. } => base,
                        _ => return None,
                    }
                }
                ExprKind::Call { callee, .. } => return self.owned_call(e, callee).then_some(e),
                _ => return None,
            };
        }
    }

    /// Whether a call builds an owned sequence: its result is a `List`,
    /// `Str` or array, and the fn it calls isn't declared `lends`.
    fn owned_call(&mut self, call: &Expr, callee: &Expr) -> bool {
        if let ExprKind::Name(n) = &unparen(callee).kind {
            if matches!(self.global(n).as_deref(), Some(Def::Fn { lends: true, .. })) {
                return false;
            }
        }
        let ty = self.quietly(|c| c.infer(call));
        matches!(&ty, Ty::Str | Ty::Array { .. })
            || matches!(&ty, Ty::Named { name, .. } if name == "List")
    }
}

impl Checker<'_> {
    /// A closure argument borrows its captures for the call (R12.4.11), so
    /// it can't capture what argument `k` lends `mut`.
    pub(crate) fn no_capture_of_mut_arg(
        &mut self,
        args: &[ast::Arg],
        k: usize,
        root: &str,
        name: &str,
    ) {
        if !args[k].mutable {
            return;
        }
        for (j, other) in args.iter().enumerate() {
            if j == k || !self.captures(&other.value).iter().any(|c| c == root) {
                continue;
            }
            let message = format!(
                "`{root}` can't be passed `mut` to `{name}`: `{}` captures it and uses it during the same call",
                self.quote(other.value.span),
            );
            if let Some(diag) = self.error(codes::USED_WHILE_LENT, args[k].value.span, message) {
                diag.help = Some(format!(
                    "pass `{root}` to the closure as a parameter instead of capturing it"
                ));
            }
        }
    }

    /// What a closure argument captures by reference: a lambda's own
    /// captures, or those of the closure a name holds.
    fn captures(&mut self, e: &Expr) -> Vec<String> {
        if let ExprKind::Name(f) = &unparen(e).kind {
            return self
                .lends
                .holds
                .iter()
                .filter(|h| h.is_capture && &h.name == f)
                .map(|h| h.root.clone())
                .collect();
        }
        self.lambda_ref_captures(e)
    }

    /// Variables captured by reference by a lambda: local `var`s, local heap
    /// values, parameters, or top-level heap/`var` values (R05.10.1).
    pub(crate) fn lambda_ref_captures(&mut self, expr: &Expr) -> Vec<String> {
        let ExprKind::Lambda { params, body, .. } = &unparen(expr).kind else {
            return Vec::new();
        };
        let mut bound = crate::fx::HashSet::default();
        for p in params {
            super::pattern::pattern_types(p, &Ty::Unknown, &mut |n, _| {
                bound.insert(n.to_string());
            });
        }
        let mut caps = Vec::new();
        for n in super::lends::names_in(body) {
            if bound.contains(&n) || caps.contains(&n) {
                continue;
            }
            let by_ref = if let Some(local) = self.local(&n) {
                let ty = local.ty.clone();
                let span = local.span;
                local.mutable
                    || local.keyword == "param"
                    || self.is_heap_type(&ty)
                    || self.lends.views.contains_key(&span)
            } else if let Some(def) = self.global(&n) {
                match &*def {
                    Def::Value { ty, mutable, .. } => {
                        *mutable
                            || self.is_heap_type(ty)
                            || self.lends.global_views.contains_key(&n)
                    }
                    _ => false,
                }
            } else {
                false
            };
            if by_ref {
                caps.push(n);
            }
        }
        caps
    }
}

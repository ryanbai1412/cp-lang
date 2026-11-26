//! Index syntax: `s[k]` reads through `Indexed`, `s[r]` calls the `@index` fn that cuts a view
//! (R07.4.5, R07.4.17).
use super::*;

impl Cx<'_> {
    /// `recv.name(args)` with already-typed arguments: used for `get` and `set` behind indexing.
    /// `s[i]` is `s.at(i)` of its `Indexed` impl (R12.4.5), read in place; a method of
    /// the type's own named `at` is not indexing.
    pub(super) fn elem(&mut self, s: TExpr, i: TExpr, span: Span) -> TExpr {
        let at = INDEX_READ;
        let rt = self.shallow(&s.ty);
        let read = self.d.impl_member_by(&rt, at, |f| f.region.is_none());
        let call = match (&rt, read) {
            (Ty::Param(p), _) => self.bound_call(*p, s, at, vec![TArg::Value(i)], span),
            (_, Some(decls::Member::Method(m, ts, ss))) => {
                self.call_member(s, (m, ts, ss), vec![TArg::Value(i)], span, false)
            }
            _ => {
                let shown = self.show(&rt);
                self.err(
                    span,
                    format!("`{shown}` is not indexed: it has no `Indexed` impl"),
                );
                return TExpr {
                    kind: TK::Error,
                    ty: Ty::Error,
                    span,
                };
            }
        };
        TExpr {
            ty: call.ty.clone(),
            kind: TK::Elem(Box::new(call)),
            span,
        }
    }
    /// Whether `t` is the type the `@index` fns take as a key: a `Range` (R07.4.17).
    pub(super) fn is_range(&mut self, t: &Ty) -> bool {
        let d = self.d;
        let key = |s: StructId| {
            (d.index_fns.iter()).any(|f| {
                matches!(d.f(*f).params.first().map(|p| &p.ty), Some(Ty::Struct(k, ..)) if *k == s)
            })
        };
        matches!(self.shallow(t), Ty::Struct(s, ..) if key(s))
    }
    /// R07.4.5: a literal key into a view made by `s[r]` reads a key, so it must be proven one:
    /// inside the literal range the view was cut by. Anything else is E0305, as `w[0]` is meant
    /// as a position.
    pub(super) fn literal_key(&mut self, base: &ast::Expr, key: &ast::Expr, ty: &Ty) {
        let ExprKind::Int(v) = unparen(key).kind else {
            return;
        };
        let Ty::Struct(sid, ..) = self.shallow(ty) else {
            return;
        };
        let d = self.d;
        if !(d.index_fns.iter()).any(|f| matches!(d.f(*f).ret, Ty::Struct(s, ..) if s == sid)) {
            return;
        }
        let keys = match &unparen(base).kind {
            ExprKind::Name(n) => {
                (self.lookup(n)).and_then(|(var, _)| self.view_keys.get(&var).copied())
            }
            _ => literal_cut(base),
        };
        if keys.is_some_and(|(lo, hi)| lo <= v && v < hi) {
            return;
        }
        self.err(
            key.span,
            format!(
                "{}: `[{v}]` reads key {v}, which is not proven one of the view's keys: write `.nth({v})` for its element {v}, or index the source for its key {v}",
                cpl_diag::codes::NOT_INDEXABLE.0
            ),
        );
    }
    /// The `@index` fn `s[i]` calls: when `i` is a `Range` and `s` is not keyed by ranges (R07.4.17).
    pub(super) fn index_fn(&mut self, s: &Ty, i: &Ty) -> Option<FnId> {
        if !self.is_range(i) {
            return None;
        }
        let key = match self.shallow(s) {
            Ty::Param(p) => (self.bound_traits(p).into_iter())
                .find(|t| self.d.traits[t.tr.0].methods.contains_key(INDEX_READ))
                .and_then(|t| t.args.first().cloned()),
            rt => match self
                .d
                .impl_member_by(&rt, INDEX_READ, |f| f.region.is_none())
            {
                Some(decls::Member::Method(m, ts, _)) => Some(
                    Subst {
                        tys: &ts,
                        sets: &[],
                    }
                    .ty(&self.d.f(m).params[0].ty),
                ),
                _ => None,
            },
        };
        if key.is_some_and(|k| self.is_range(&k)) {
            return None;
        }
        let d = self.d;
        d.index_fns
            .iter()
            .copied()
            .find(|f| d.f(*f).params.len() == 1)
    }
    /// `s[lo..]`, `s[..hi]`, `s[..=hi]` and `s[..]`: the `@index` fn that takes the two ends, each
    /// optional, and whether `hi` is in (R07.4.6).
    pub(super) fn open_slice(
        &mut self,
        s: TExpr,
        (lo, hi, through): (Option<&ast::Expr>, Option<&ast::Expr>, bool),
        span: Span,
    ) -> TExpr {
        let d = self.d;
        let Some(f) = d
            .index_fns
            .iter()
            .copied()
            .find(|f| d.f(*f).params.len() == 3)
        else {
            self.err(
                span,
                "slice: a slice with an end left out needs core's `@index` fn of the ends",
            );
            return TExpr {
                kind: TK::Error,
                ty: Ty::Error,
                span,
            };
        };
        let end = |e: Option<&ast::Expr>| ast::Arg {
            name: None,
            mutable: false,
            value: e.cloned().unwrap_or(ast::Expr {
                kind: ExprKind::None,
                span,
            }),
        };
        let args = [
            end(lo),
            end(hi),
            ast::Arg {
                name: None,
                mutable: false,
                value: ast::Expr {
                    kind: ExprKind::Bool(through),
                    span,
                },
            },
        ];
        let f = self.fnref(f, Vec::new(), Vec::new());
        if let Some(st) = &d.f(f.decl).self_ty {
            let st = Subst {
                tys: &f.targs,
                sets: &f.sets,
            }
            .ty(st);
            self.unify(&s.ty, &st);
        }
        self.call_fn(f, Some(TArg::Value(s)), &args, span, None)
    }
    /// `s[r]` as a call of the `@index` fn `f`, whose `self` and key it fits to `s` and `r`.
    pub(super) fn index_call(&mut self, f: FnId, s: TExpr, r: TExpr, span: Span) -> TExpr {
        let f = self.fnref(f, Vec::new(), Vec::new());
        let fd = self.d.f(f.decl);
        let sub = Subst {
            tys: &f.targs,
            sets: &f.sets,
        };
        if let Some(st) = &fd.self_ty {
            let st = sub.ty(st);
            self.unify(&s.ty, &st);
        }
        let pty = sub.ty(&fd.params[0].ty);
        let r = self.coerce(r, &pty);
        self.oblige(&f, span);
        self.solve(false);
        let ty = self.ret_ty(&f);
        TExpr {
            kind: TK::Call {
                f,
                recv: Some(Box::new(TArg::Value(s))),
                args: vec![TArg::Value(r)],
            },
            ty,
            span,
        }
    }
}

/// What `s[i]` calls on an `Indexed` receiver: the trait's read (R12.4.5).
const INDEX_READ: &str = "at";

pub(super) fn unparen(mut e: &ast::Expr) -> &ast::Expr {
    while let ExprKind::Paren(inner) = &e.kind {
        e = inner;
    }
    e
}

/// The keys `lo..hi` of `s[lo..hi]` or `s[lo..=hi]` cut with literal or left-out ends.
pub(super) fn literal_cut(e: &ast::Expr) -> Option<(i128, i128)> {
    let ExprKind::Index { indices, .. } = &unparen(e).kind else {
        return None;
    };
    let [r] = indices.as_slice() else {
        return None;
    };
    let ExprKind::Range {
        lo,
        hi,
        inclusive,
        step: None,
    } = &unparen(r).kind
    else {
        return None;
    };
    // An end left out is unbounded on that side: only the written end limits a literal key.
    let lit = |e: &Option<Box<ast::Expr>>, open: i128| match e.as_deref().map(|e| &unparen(e).kind)
    {
        None => Some(open),
        Some(ExprKind::Int(v)) => Some(*v),
        Some(_) => None,
    };
    let (a, b) = (lit(lo, i128::MIN)?, lit(hi, i128::MAX)?);
    Some((a, if *inclusive && hi.is_some() { b + 1 } else { b }))
}

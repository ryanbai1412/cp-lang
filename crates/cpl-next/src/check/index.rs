//! Index syntax: `s[k]` reads through `Indexed`, `s[r]` calls the `@index` fn that cuts a view
//! (R07.4.5, R07.4.17).
use super::*;
use std::collections::HashMap;

/// What runs while a cell is lent: given the cell's name, it adds its statements.
pub(super) type InCell<'a, C> = Box<dyn FnOnce(&mut C, ast::Expr, &mut Vec<TStmt>) + 'a>;

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
            _ if self.mentions_error(&rt) => return Self::error_node(span),
            _ => {
                let shown = self.show(&rt);
                self.err(
                    span,
                    format!(
                        "{}: `{shown}` is not indexed: it has no `Indexed` impl",
                        cpl_diag::codes::NOT_INDEXABLE.0
                    ),
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
        // A view: what an `@index` fn cuts from a source of another type. A range's sub-range is
        // a range, keyed by its values (R07.2.15).
        let view = |f: &FnId| {
            matches!(d.f(*f).ret, Ty::Struct(s, ..) if s == sid)
                && !matches!(d.f(*f).self_ty, Some(Ty::Struct(s, ..)) if s == sid)
        };
        if !d.index_fns.iter().any(view) {
            return;
        }
        let keys = match &unparen(base).kind {
            ExprKind::Name(n) => {
                (self.lookup(n)).and_then(|(var, _)| self.view_keys.get(&var).copied())
            }
            _ => literal_cut(base),
        };
        // Only a view whose keys are known here can prove a literal outside them; any other
        // view's keys are checked as it runs (R07.4.5).
        if keys.is_none_or(|(lo, hi)| lo <= v && v < hi) {
            return;
        }
        self.err(
            key.span,
            format!(
                "{}: `[{v}]` reads key {v}, which is not one of the view's keys: write `.nth({v})` for its element {v}, or index the source for its key {v}",
                cpl_diag::codes::NOT_INDEXABLE.0
            ),
        );
    }
    /// The `@index` fn `s[i]` calls: when `i` is a `Range` and `s` is not keyed by ranges (R07.4.17).
    pub(super) fn index_fn(&mut self, s: &Ty, i: &Ty) -> Option<FnId> {
        if !self.is_range(i) {
            return None;
        }
        if self.key_of(s).is_some_and(|k| self.is_range(&k)) {
            return None;
        }
        self.index_fn_for(s, 1)
    }
    /// The range type over `k` that the `@index` fns take, where a literal range indexes a
    /// source keyed by `k`, so a range of cuts there is the piece between them (R04.4.4).
    pub(super) fn range_over(&self, k: Ty) -> Option<Ty> {
        let d = self.d;
        (d.index_fns.iter()).find_map(|f| match d.f(*f).params.first().map(|p| &p.ty) {
            Some(Ty::Struct(r, ..)) => Some(Ty::Struct(*r, vec![k.clone()], Vec::new())),
            _ => None,
        })
    }
    /// The type of the cell `s[k]` reads, through `Indexed`.
    pub(super) fn cell_of(&mut self, s: &Ty) -> Option<Ty> {
        let rt = self.shallow(s);
        match self.read_sig(&rt)?.1 {
            Ty::Ref(t) => Some(*t),
            t => Some(t),
        }
    }
    /// The key and cell types of `rt`'s `Indexed` read.
    fn read_sig(&self, rt: &Ty) -> Option<(Ty, Ty)> {
        match self
            .d
            .impl_member_by(rt, INDEX_READ, |f| f.region.is_none())
        {
            Some(decls::Member::Method(m, ts, _)) => {
                let sub = Subst {
                    tys: &ts,
                    sets: &[],
                };
                let f = self.d.f(m);
                Some((sub.ty(&f.params[0].ty), sub.ty(&f.ret)))
            }
            _ => None,
        }
    }
    /// The key type `s[k]` reads at, through `Indexed`.
    pub(super) fn key_of(&mut self, s: &Ty) -> Option<Ty> {
        match self.shallow(s) {
            Ty::Param(p) => (self.bound_traits(p).into_iter())
                .find(|t| self.d.traits[t.tr.0].methods.contains_key(INDEX_READ))
                .and_then(|t| t.args.first().cloned()),
            rt => self.read_sig(&rt).map(|(k, _)| k),
        }
    }
    /// The `@index` fn taking `arity` arguments that `s[..]` calls: one declared for `s`'s own
    /// type, as a range's sub-range is (R07.2.15), before the one for any indexed `S` (R07.4.6).
    fn index_fn_for(&mut self, s: &Ty, arity: usize) -> Option<FnId> {
        let own = match self.shallow(s) {
            Ty::Struct(sid, ..) => Some(sid),
            _ => None,
        };
        let d = self.d;
        let mut any = None;
        for &f in d
            .index_fns
            .iter()
            .filter(|f| d.f(**f).params.len() == arity)
        {
            match &d.f(f).self_ty {
                Some(Ty::Struct(sid, ..)) => {
                    if Some(*sid) == own {
                        return Some(f);
                    }
                }
                _ => {
                    any.get_or_insert(f);
                }
            }
        }
        any
    }
    /// Runs `inner` while `base[idx]` is lent as a place, through the `at` of `base`'s
    /// `IndexedMut` that lends one (R12.4.6). `inner` gets the lent cell as a name; `g[i][j]`
    /// lends `g[i]` first, then its cell `j` inside that.
    pub(super) fn through_cell(
        &mut self,
        (base, idx): (&ast::Expr, &ast::Expr),
        span: Span,
        out: &mut Vec<TStmt>,
        inner: InCell<'_, Self>,
    ) {
        if let ExprKind::Index { base: b2, indices } = &unparen(base).kind {
            if indices.len() == 1 {
                let i2 = indices[0].clone();
                let idx = idx.clone();
                return self.through_cell(
                    (b2, &i2),
                    span,
                    out,
                    Box::new(move |this, cell, out| {
                        this.through_cell((&cell, &idx), span, out, inner)
                    }),
                );
            }
        }
        let b = self.place(base);
        let i = self.expr(idx, Some(&Ty::Int));
        let i = self.index_arg(&b.ty, i);
        let rt = self.shallow(&b.ty);
        let lends = |f: &decls::FnDecl| matches!(f.region, Some((true, _)));
        let Some(decls::Member::Method(m, ts, ss)) = self.d.impl_member_by(&rt, INDEX_READ, lends)
        else {
            if !matches!(rt, Ty::Error) {
                let why = if matches!(rt, Ty::Param(_)) {
                    "slice: writing a cell of a type parameter".to_string()
                } else {
                    let shown = self.show(&rt);
                    format!(
                        "{}: `{shown}` has no `IndexedMut` impl, so `[]` on it is not a place",
                        cpl_diag::codes::NOT_INDEXABLE.0
                    )
                };
                self.err(span, why);
            }
            return;
        };
        let f = self.fnref(m, ts, ss);
        self.oblige(&f, span);
        self.solve(false);
        let fd = self.d.f(f.decl);
        let sub = Subst {
            tys: &f.targs,
            sets: &f.sets,
        };
        let i = self.coerce(i, &sub.ty(&fd.params[0].ty));
        let elem = sub.ty(&fd.region.as_ref().map_or(Ty::Error, |(_, t)| t.clone()));
        self.scopes.push(HashMap::new());
        let name = format!("$cell{}", out.len());
        let var = self.bind(&name, elem);
        if let VarRef::Local(l) = var {
            self.locals[l].mutable = true;
        }
        let mut body = Vec::new();
        inner(
            self,
            ast::Expr {
                kind: ExprKind::Name(name),
                span,
            },
            &mut body,
        );
        self.scopes.pop();
        out.push(TStmt::Region {
            callee: Callee::Fn(f),
            recv: Some(Box::new(TArg::Mut(b))),
            args: vec![TArg::Value(i)],
            var,
            lend: true,
            body: TExpr {
                kind: TK::Block(body, None),
                ty: Ty::Unit,
                span,
            },
        });
    }
    /// `g[i].m(args)` as a statement, `m` a method that writes its receiver: the indices, then
    /// the arguments, are read once; then `g`'s `IndexedMut` lends cell `i` and `m` runs on it
    /// (R12.4.6). False, with nothing done, for any other statement.
    pub(super) fn cell_method(&mut self, e: &ast::Expr, out: &mut Vec<TStmt>) -> bool {
        let ExprKind::Call { callee, args } = &e.kind else {
            return false;
        };
        let ExprKind::Field { base, name } = &callee.kind else {
            return false;
        };
        let base = unparen(base);
        if !matches!(&base.kind, ExprKind::Index { indices, .. } if indices.len() == 1) {
            return false;
        }
        let seen = self.errors.len();
        let probe = self.expr(base, None);
        self.errors.truncate(seen);
        let rt = self.shallow(&probe.ty);
        let writes = match self.d.lookup_member(&rt, &name.text) {
            Some(decls::Member::Method(m, ..)) => {
                self.d.f(m).self_mode == SelfMode::Mut || self.d.writes_cache(m)
            }
            _ => false,
        };
        if !writes {
            return false;
        }
        let target = self.hoist_indices(base, out);
        let args: Vec<ast::Arg> = (args.iter())
            .map(|a| match a.value.kind {
                ExprKind::Lambda { .. } => a.clone(),
                _ if a.mutable => a.clone(),
                _ => ast::Arg {
                    value: self.hoist(&a.value, None, "$arg", out),
                    ..a.clone()
                },
            })
            .collect();
        let ExprKind::Index { base: b, indices } = &target.kind else {
            return false;
        };
        let (name, span, callee_span) = (name.clone(), e.span, callee.span);
        self.through_cell(
            (b, &indices[0]),
            target.span,
            out,
            Box::new(move |this, cell, out| {
                let call = ast::Expr {
                    kind: ExprKind::Call {
                        callee: Box::new(ast::Expr {
                            kind: ExprKind::Field {
                                base: Box::new(cell),
                                name,
                            },
                            span: callee_span,
                        }),
                        args,
                    },
                    span,
                };
                let v = this.expr(&call, None);
                out.push(TStmt::Expr(v));
            }),
        );
        true
    }
    /// `target` with each of its indices, outer first, read once into a temporary before any
    /// cell is lent, so `p[r] = p[p[r]]` reads `p` before writing it (R12.4.6).
    pub(super) fn hoist_indices(&mut self, target: &ast::Expr, out: &mut Vec<TStmt>) -> ast::Expr {
        let ExprKind::Index { base, indices } = &target.kind else {
            return target.clone();
        };
        let base = match &unparen(base).kind {
            ExprKind::Index { .. } => self.hoist_indices(unparen(base), out),
            _ => (**base).clone(),
        };
        let indices = (indices.iter())
            .map(|i| self.hoist(i, Some(&Ty::Int), "$idx", out))
            .collect();
        ast::Expr {
            kind: ExprKind::Index {
                base: Box::new(base),
                indices,
            },
            span: target.span,
        }
    }
    /// `e` read once into a temporary, as a name for it.
    pub(super) fn hoist(
        &mut self,
        e: &ast::Expr,
        want: Option<&Ty>,
        what: &str,
        out: &mut Vec<TStmt>,
    ) -> ast::Expr {
        // A name or a literal reads the same later; anything else runs now.
        if matches!(
            unparen(e).kind,
            ExprKind::Name(_)
                | ExprKind::Int(_)
                | ExprKind::Bool(_)
                | ExprKind::Char(_)
                | ExprKind::Str(_)
        ) {
            return e.clone();
        }
        let v = self.expr(e, want);
        let name = format!("{what}{}", out.len());
        let t = self.bind(&name, v.ty.clone());
        out.push(TStmt::Let(t, v));
        ast::Expr {
            kind: ExprKind::Name(name),
            span: e.span,
        }
    }
    /// `s[lo..]`, `s[..hi]`, `s[..=hi]` and `s[..]`: the `@index` fn that takes the two ends, each
    /// optional, and whether `hi` is in (R07.4.6).
    pub(super) fn open_slice(
        &mut self,
        s: TExpr,
        (lo, hi, through): (Option<&ast::Expr>, Option<&ast::Expr>, bool),
        span: Span,
    ) -> TExpr {
        let key = self.key_of(&s.ty).map(|k| self.shallow(&k));
        let lo = lo.map(|e| self.expr(e, key.as_ref()));
        let hi = hi.map(|e| self.expr(e, key.as_ref()));
        // An end that is a cut of the keys: the piece of positions up to or from it (R04.4.8).
        if let Some(k) = key
            .filter(|k| (lo.iter().chain(&hi)).any(|e| self.cut_domain(&e.ty).as_ref() == Some(k)))
        {
            let f = self.index_fn_for(&s.ty, 1);
            let piece = self.cut_piece(lo.clone(), hi.clone(), through, k, span);
            if let (Some(f), Some(r)) = (f, piece) {
                return self.index_call(f, s, vec![Some(r)], span);
            }
        }
        let Some(f) = self.index_fn_for(&s.ty, 3) else {
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
        let through = TExpr {
            kind: TK::Bool(through),
            ty: Ty::Bool,
            span,
        };
        self.index_call(f, s, vec![lo, hi, Some(through)], span)
    }
    /// `s[..]` as a call of the `@index` fn `f`, whose `self` and params it fits to `s` and
    /// `args`; a left-out argument is `None`.
    pub(super) fn index_call(
        &mut self,
        f: FnId,
        s: TExpr,
        args: Vec<Option<TExpr>>,
        span: Span,
    ) -> TExpr {
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
        // `self`'s impl settles the key type before an argument's literal could.
        self.oblige(&f, span);
        self.solve(false);
        let mut out = Vec::new();
        for (a, p) in args.into_iter().zip(&fd.params) {
            let pty = sub.ty(&p.ty);
            let a = match a {
                Some(a) => self.coerce(a, &pty),
                None => {
                    let none = ast::Expr {
                        kind: ExprKind::None,
                        span,
                    };
                    self.expr(&none, Some(&pty))
                }
            };
            out.push(TArg::Value(a));
        }
        self.solve(false);
        let ty = self.ret_ty(&f);
        TExpr {
            kind: TK::Call {
                f,
                recv: Some(Box::new(TArg::Value(s))),
                args: out,
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

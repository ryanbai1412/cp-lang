//! Index syntax: `s[k]` reads through `Indexed`, `s[r]` calls the `@index` fn that cuts a view
//! (R07.4.5, R07.4.17).
use super::*;
use cpl_next_ir::unbuilt::Unbuilt;
use std::collections::HashMap;

/// What runs while a cell is lent: given the cell's name, it adds its statements.
pub(super) type InCell<'a, C> = Box<dyn FnOnce(&mut C, ast::Expr, &mut Vec<TStmt>) + 'a>;

impl Cx<'_> {
    /// `recv.name(args)` with already-typed arguments: used for `get` and `set` behind indexing.
    /// `s[i]` is `s.at(i)` of its `Indexed` impl (R12.4.5), read in place; a method of
    /// the type's own named `at` is not indexing.
    pub(super) fn elem(&mut self, s: TExpr, i: TExpr, span: Span) -> TExpr {
        let at = self.index_read();
        let rt = self.recv_ty(&s.ty);
        let read = self
            .impls()
            .impl_member_by(&rt, &at, |f| f.region.is_none());
        let call = match (&rt, read) {
            (Ty::Param(p), _) => self.bound_call(*p, s, &at, vec![TArg::Value(i)], span),
            (_, Some(decls::Member::Method(m, ts, ss))) => {
                self.call_member(s, (m, ts, ss), vec![TArg::Value(i)], span, false)
            }
            _ if self.mentions_error(&rt) => return Self::error_node(span),
            _ => {
                let shown = self.show(&rt);
                self.err(
                    span,
                    Msg::verdict(
                        cpl_diag::codes::NOT_INDEXABLE,
                        format!("`{shown}` is not indexed: it has no `Indexed` impl"),
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
        matches!(self.shallow(t), Ty::Struct(s, ..) if Some(s) == self.range_struct())
    }
    /// The `Range` struct: the key of the `@index` fn that cuts any indexed `S` (R07.4.17).
    pub(super) fn range_struct(&self) -> Option<StructId> {
        let d = self.d;
        (d.index_fns.iter()).find_map(|f| match (&d.f(*f).self_ty, &d.f(*f).params[..]) {
            (Some(Ty::Param(_)), [p]) => match p.ty {
                Ty::Struct(r, ..) => Some(r),
                _ => None,
            },
            _ => None,
        })
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
            Msg::verdict(cpl_diag::codes::NOT_INDEXABLE, format!("`[{v}]` reads key {v}, which is not one of the view's keys: write `.nth({v})` for its element {v}, or index the source for its key {v}")),
        );
    }
    /// An `Int` used to index an array is checked against the array's domain. The proof that removes the check is the facts layer's.
    /// A declared domain, or a type that implements core's `Domain`.
    pub(super) fn is_domain(&self, t: &Ty) -> bool {
        match t {
            Ty::Dom(_) => true,
            Ty::Struct(..) => {
                (self.d.roles().domain).is_some_and(|tr| self.impls().has_impl(t, tr))
            }
            _ => false,
        }
    }
    pub(super) fn index_arg(&mut self, base: &Ty, i: TExpr) -> TExpr {
        let array = matches!(self.shallow(base), Ty::Struct(a, ..) if Some(a) == self.array_id());
        let i = match self.key_of(base) {
            Some(k) => self.fit_key(&k, array, i),
            None => i,
        };
        // A position of one domain never indexes an array over another (R04.3.1).
        if let Some(k) = self.key_of(base) {
            let (k, t) = (self.zonk(&k), self.zonk(&i.ty));
            if k != t && self.is_domain(&k) && self.is_domain(&t) {
                let (ks, ts) = (self.show(&k), self.show(&t));
                self.err(
                    i.span,
                    Msg::verdict(
                        cpl_diag::codes::WRONG_DOMAIN,
                        format!("a `{ts}` indexes a `[{ks}]` array: index it with a `{ks}`"),
                    ),
                );
                return TExpr {
                    kind: TK::Error,
                    ty: Ty::Error,
                    span: i.span,
                };
            }
        }
        // Keys never wrap (R07.2.17).
        if let TK::Neg(k) = &i.kind {
            if let TK::Int(k) = k.kind {
                self.err(
                    i.span,
                    Msg::verdict(
                        cpl_diag::codes::NEGATIVE_LITERAL_INDEX,
                        format!("`-{k}` is never a key: keys do not wrap; count from `.len`"),
                    ),
                );
            }
        }
        i
    }
    /// `i` fitted to the key `k`: an `Int` becomes a position of a domain key, by the rules
    /// below, and a tuple written in place, as in `g[1, j]`, is fitted part by part to the
    /// product's factors (R04.2.10).
    fn fit_key(&mut self, k: &Ty, array: bool, mut i: TExpr) -> TExpr {
        let dty = self.shallow(k);
        if let (Ty::Struct(t, fs, _), Ty::Struct(u, _, sets)) = (&dty, self.shallow(&i.ty)) {
            let tuple = u == *t && self.d.tuple(fs.len()).as_ref() == Some(t);
            if let TK::Struct(ps) = &mut i.kind {
                if tuple && ps.len() == fs.len() {
                    let ps: Vec<TExpr> = (std::mem::take(ps).into_iter().zip(fs))
                        .map(|(p, f)| self.fit_key(f, array, p))
                        .collect();
                    let ty = Ty::Struct(u, ps.iter().map(|p| p.ty.clone()).collect(), sets);
                    return TExpr {
                        kind: TK::Struct(ps),
                        ty,
                        span: i.span,
                    };
                }
            }
        }
        if self.shallow(&i.ty) != Ty::Int
            || !matches!(
                dty,
                Ty::Dom(_) | Ty::Param(_) | Ty::Range(..) | Ty::RangeTo(..)
            )
        {
            return i;
        }
        let span = i.span;
        let literal = matches!(i.kind, TK::Int(_) | TK::Neg(_));
        // A literal into a named domain is a position written as a number: outside it, that
        // is the domain's trap (R04.4.25); any other `Int` is a key out of range (R0101).
        let mut kind = CheckKind::Key;
        // A named domain's positions are never raw `Int`s; a literal converts into an
        // array (R02.8.1, R04.8.8), never into a view of one (R07.4.5).
        if let Ty::Dom(d) = &dty {
            let named = !self.domains[*d].name.starts_with('$');
            if named && literal {
                kind = CheckKind::Domain;
            }
            if named && !(array && literal) {
                let name = self.domains[*d].name.clone();
                let what = if array {
                    format!("a `[{name}]` array")
                } else {
                    format!("a view keyed by `{name}`")
                };
                self.err(
                    span,
                    Msg::verdict(
                        cpl_diag::codes::WRONG_DOMAIN,
                        format!("an `Int` indexes {what}: write `{name}(i)`"),
                    ),
                );
            }
        }
        self.position(dty, i, kind)
    }
    /// The `@index` fn `s[i]` calls: when `i` is a `Range` and `s` is not keyed by ranges (R07.4.17).
    pub(super) fn index_fn(&mut self, s: &Ty, i: &Ty) -> Option<FnId> {
        if !self.is_range(i) {
            return self.view_fn(s, i);
        }
        if self.key_of(s).is_some_and(|k| self.is_range(&k)) {
            return None;
        }
        self.index_fn_for(s, 1)
    }
    /// The `@index` fn `s[i]` calls for an `i` that is not one of `s`'s keys: one whose `self`
    /// and key take `s` and `i`, as a grid's row `g[r]` is (R04b.1.12).
    fn view_fn(&mut self, s: &Ty, i: &Ty) -> Option<FnId> {
        if self.key_of(s).is_none_or(|k| self.fits(&k, i)) {
            return None;
        }
        let d = self.d;
        (d.index_fns.iter().copied()).find(|&f| {
            let fd = d.f(f);
            let mut binds = HashMap::new();
            fd.params.len() == 1
                && (fd.self_ty.as_ref()).is_some_and(|t| self.shape_fits(s, t, &mut binds))
                && self.shape_fits(i, &fd.params[0].ty, &mut binds)
        })
    }
    /// Whether `have` could be the declared `decl`, each of the declaration's params standing for
    /// one type, the first it meets in `binds`; nothing is solved.
    fn shape_fits(&self, have: &Ty, decl: &Ty, binds: &mut HashMap<usize, Ty>) -> bool {
        match (self.shallow(have), decl) {
            (h, Ty::Param(p)) => match binds.get(p) {
                Some(b) => self.fits(&h, b),
                None => {
                    binds.insert(*p, h);
                    true
                }
            },
            (Ty::Struct(x, xs, _), Ty::Struct(y, ys, _)) => {
                x == *y
                    && xs.len() == ys.len()
                    && (xs.iter().zip(ys)).all(|(a, b)| self.shape_fits(a, b, binds))
            }
            (h, t) => self.fits(&h, t),
        }
    }
    /// The key `s[(.., c)]` wants: its product key with each range part a range over that
    /// part's factor (R04.4.11).
    pub(super) fn ranged_key(&mut self, s: &Ty, parts: &[ast::Expr]) -> Option<Ty> {
        let ranged = |p: &ast::Expr| matches!(unparen(p).kind, ExprKind::Range { .. });
        if !parts.iter().any(ranged) {
            return None;
        }
        let k = self.key_of(s)?;
        let Ty::Struct(t, fs, sets) = self.shallow(&k) else {
            return None;
        };
        if self.d.tuple(parts.len()) != Some(t) || fs.len() != parts.len() {
            return None;
        }
        let fs = (parts.iter().zip(fs))
            .map(|(p, f)| {
                if ranged(p) {
                    self.range_over(f)
                } else {
                    Some(f)
                }
            })
            .collect::<Option<Vec<_>>>()?;
        Some(Ty::Struct(t, fs, sets))
    }
    /// `D`, when a bare `..` stands where a `Range<D>` is wanted: all of `D` (R04.4.11).
    pub(super) fn whole_of(&self, want: Option<&Ty>) -> Option<Ty> {
        let w = self.shallow(want?);
        let Ty::Struct(_, a, _) = &w else {
            return None;
        };
        let [d] = &a[..] else {
            return None;
        };
        (self.range_over(d.clone()).as_ref() == Some(&w)).then(|| d.clone())
    }
    /// The range type over `k` that the `@index` fns take, where a literal range indexes a
    /// source keyed by `k`, so a range of cuts there is the piece between them (R04.4.4).
    pub(super) fn range_over(&self, k: Ty) -> Option<Ty> {
        (self.range_struct()).map(|r| Ty::Struct(r, vec![k], Vec::new()))
    }
    /// The type of the cell `s[k]` reads, through `Indexed`.
    pub(super) fn cell_of(&mut self, s: &Ty) -> Option<Ty> {
        let rt = self.recv_ty(s);
        match self.read_sig(&rt)?.1 {
            Ty::Ref(t) => Some(*t),
            t => Some(t),
        }
    }
    /// What `s[i]` calls: the method of std's `@operator("[]")` trait (R12.4.5).
    fn index_read(&self) -> String {
        let tr = self.d.operator_traits("[]").next();
        tr.and_then(|tr| self.d.operator_method(tr).map(|(n, _)| n.to_string()))
            .unwrap_or_default()
    }
    /// The key and cell types of `rt`'s `Indexed` read.
    fn read_sig(&self, rt: &Ty) -> Option<(Ty, Ty)> {
        match self
            .impls()
            .impl_member_by(rt, &self.index_read(), |f| f.region.is_none())
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
                .find(|t| {
                    self.d
                        .member(&self.d.t(t.tr).methods, &self.index_read())
                        .is_some()
                })
                .and_then(|t| t.args.first().cloned()),
            _ => self.read_sig(&self.recv_ty(s)).map(|(k, _)| k),
        }
    }
    /// The `@index` fn taking `arity` range ends that `s[..]` calls: one declared for `s`'s own
    /// type, as a range's sub-range is (R07.2.15), before the one for any indexed `S` (R07.4.6).
    fn index_fn_for(&mut self, s: &Ty, arity: usize) -> Option<FnId> {
        let own = match self.shallow(s) {
            Ty::Struct(sid, ..) => Some(sid),
            _ => None,
        };
        let d = self.d;
        let range = self.range_struct();
        let ends = |f: &FnId| {
            let ps = &d.f(*f).params;
            let first = match ps.first().map(|p| &p.ty) {
                Some(Ty::Struct(k, ..)) => Some(*k),
                _ => None,
            };
            ps.len() == arity && first.is_some() && (arity > 1 || first == range)
        };
        let mut any = None;
        for &f in d.index_fns.iter().filter(|f| ends(f)) {
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
    /// `s[k] = v` as `s.set(k, v)` when `s`'s type has an impl of core's `@operator("[]=")`
    /// trait, `IndexSet`: a write that needs no place (spec 12, W8).
    pub(super) fn index_set(
        &mut self,
        base: &ast::Expr,
        k: &ast::Expr,
        v: &ast::Expr,
        span: Span,
    ) -> Option<ast::Stmt> {
        let tr = self.d.operator_traits("[]=").next()?;
        let name = self.d.operator_method(tr)?.0;
        let seen = self.errors.len();
        let b = self.expr_as_typed(base, None);
        self.errors.truncate(seen);
        let rt = self.shallow(&b.ty);
        let d = &self.d;
        let of_tr = |f: &decls::FnDecl| matches!(f.owner, Some(decls::Owner::Impl(i)) if d.imp(i).trait_.tr == tr);
        self.impls().impl_member_by(&rt, name, of_tr)?;
        let callee = ast::Expr {
            kind: ExprKind::Field {
                base: Box::new(base.clone()),
                name: ast::Name {
                    text: name.to_string(),
                    span,
                },
            },
            span,
        };
        let args = [k, v].map(|e| ast::Arg {
            name: None,
            mutable: false,
            value: e.clone(),
        });
        let call = ExprKind::Call {
            callee: Box::new(callee),
            args: args.to_vec(),
        };
        Some(ast::Stmt {
            kind: StmtKind::Expr(ast::Expr { kind: call, span }),
            span,
            decorators: Vec::new(),
            public: false,
        })
    }
    /// `t[i][j] = v` with `t` a table is E0503: its cell is `t[i, j]` (R01.8.5).
    fn table_point_again(
        &mut self,
        t: &ast::Expr,
        i: &ast::Expr,
        j: &ast::Expr,
        span: Span,
    ) -> bool {
        let point = |e: &ast::Expr| !matches!(unparen(e).kind, ExprKind::Range { .. });
        if !point(i) || !point(j) {
            return false;
        }
        let (seen, owed) = (self.errors.len(), self.b.obligations.len());
        let probe = self.expr(t, None);
        self.errors.truncate(seen);
        self.b.obligations.truncate(owed);
        if !self.is_table(&probe.ty) {
            return false;
        }
        let first = format!("{}[{}]", self.text(t.span), self.text(i.span));
        let text = cpl_diag::msgs::check::REMOVED_SYNTAX_INDEXES_TWICE_0_3
            .say(&[&first, &self.text(j.span)]);
        let help = "all indices go in one pair of brackets: `g[r, c]`, never `g[r][c]`";
        let code = cpl_diag::codes::REMOVED_SYNTAX;
        self.err_help(span, Msg::verdict(code, text), Some(help.to_string()));
        self.expr(j, None);
        true
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
                if self.table_point_again(b2, &indices[0], idx, span) {
                    return;
                }
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
        // A cell of a place that was diagnosed is the error: its index is not fitted to it.
        if self.mentions_error(&b.ty) {
            self.expr(idx, None);
            return;
        }
        let i = self.expr(idx, Some(&Ty::Int));
        let i = self.index_arg(&b.ty, i);
        let rt = self.shallow(&b.ty);
        let lends = |f: &decls::FnDecl| matches!(f.region, Some((true, _)));
        // A type parameter's cell is lent by the method of its bound; anything else's by its impl.
        let read = &self.index_read();
        let d = self.d;
        let bound = match rt {
            Ty::Param(p) => (self.bound_traits(p).into_iter()).find_map(|tr| {
                let m = d.member(&d.t(tr.tr).methods, read)?;
                lends(d.f(m)).then_some((tr, m))
            }),
            _ => None,
        };
        let (callee, decl, tys, sets) = if let Some((tr, m)) = bound {
            self.reach(m);
            let mut tys = vec![b.ty.clone()];
            tys.extend(tr.args.iter().cloned());
            (Callee::Trait(tr, read.to_string()), m, tys, Vec::new())
        } else {
            let Some(decls::Member::Method(m, ts, ss)) =
                self.impls().impl_member_by(&rt, read, lends)
            else {
                if !matches!(rt, Ty::Error) {
                    let shown = self.show(&rt);
                    let why = Msg::verdict(
                        cpl_diag::codes::NOT_INDEXABLE,
                        format!("`{shown}` has no `IndexedMut` impl, so `[]` on it is not a place"),
                    );
                    self.err(span, why);
                }
                return;
            };
            let f = self.fnref(m, ts, ss);
            self.oblige(&f, span);
            self.solve(false);
            let (decl, tys, sets) = (f.decl, f.targs.clone(), f.sets.clone());
            (Callee::Fn(f), decl, tys, sets)
        };
        let fd = d.f(decl);
        let sub = Subst {
            tys: &tys,
            sets: &sets,
        };
        let i = self.coerce(i, &sub.ty(&fd.params[0].ty));
        let elem = sub.ty(&fd.region.as_ref().map_or(Ty::Error, |(_, t)| t.clone()));
        self.b.scopes.push(HashMap::new());
        let name = format!("$cell{}", out.len());
        let var = self.bind(&name, elem);
        if let VarRef::Local(l) = var {
            self.b.locals[l].mutable = true;
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
        self.b.scopes.pop();
        out.push(TStmt::Region {
            callee,
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
        let writes = match self.impls().lookup_member(&rt, &name.text) {
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
                    value: self.hoist(&a.value, (None, false), "$arg", out),
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
        // A tuple written in place is fitted to a named array's key before it is held (R04.2.10).
        let keyed = match &unparen(&base).kind {
            ExprKind::Name(n) => self.lookup(n).map(|(_, t)| t),
            _ => None,
        };
        let indices = (indices.iter())
            .map(|i| match (&keyed, &unparen(i).kind) {
                (Some(b), ExprKind::Tuple(_)) => {
                    let v = self.expr(i, Some(&Ty::Int));
                    let v = self.index_arg(b, v);
                    self.hold(v, "$idx", i.span, out)
                }
                _ => self.hoist(i, (Some(&Ty::Int), false), "$idx", out),
            })
            .collect();
        ast::Expr {
            kind: ExprKind::Index {
                base: Box::new(base),
                indices,
            },
            span: target.span,
        }
    }
    /// `e` read once into a temporary, as a name for it; with `copy`, a copy of its own, so it
    /// borrows no place a later write changes.
    pub(super) fn hoist(
        &mut self,
        e: &ast::Expr,
        (want, copy): (Option<&Ty>, bool),
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
        let v = match copy {
            true => {
                let (ty, span) = (v.ty.clone(), v.span);
                let kind = TK::Owned(Box::new(v));
                TExpr { kind, ty, span }
            }
            false => v,
        };
        self.hold(v, what, e.span, out)
    }
    /// `a[x1, ..., xn]` on an array keyed by a struct `K`: `a[K(x1, ..., xn)]`, K's ordinary
    /// constructor call, held once, when it takes `n` arguments, and E0303 when it does not
    /// (decision 409, R04.2.12). `Ok(None)` leaves the indices a tuple, as a product's key or a
    /// record's takes them (decision 159).
    pub(super) fn key_ctor(
        &mut self,
        e: &ast::Expr,
        out: &mut Vec<TStmt>,
    ) -> Result<Option<ast::Expr>, ()> {
        let ExprKind::Index { base, indices } = &e.kind else {
            return Ok(None);
        };
        let [first, .., last] = &indices[..] else {
            return Ok(None);
        };
        let ExprKind::Name(n) = &unparen(base).kind else {
            return Ok(None);
        };
        let Some((_, t)) = self.lookup(n) else {
            return Ok(None);
        };
        let Some(k) = self.key_of(&t) else {
            return Ok(None);
        };
        let Ty::Struct(sid, targs, sets) = self.shallow(&k) else {
            return Ok(None);
        };
        let sd = self.d.s(sid);
        if sd.record || self.d.tuple(sd.fields.len()) == Some(sid) {
            return Ok(None);
        }
        let span = Span::new(first.span.start, last.span.end);
        let takes = sd
            .ctor
            .map_or(sd.fields.len(), |c| self.d.f(c).params.len());
        if takes != indices.len() {
            let (ks, n) = (self.show(&k), indices.len());
            self.err_help(
                span,
                Msg::verdict(
                    cpl_diag::codes::WRONG_ARG_COUNT,
                    format!(
                        "{n} indices into a `[{ks}]` array, whose key `{ks}` is built from {takes}"
                    ),
                ),
                Some(format!("index it with one key: `[{ks}(...)]`")),
            );
            return Err(());
        }
        let args: Vec<ast::Arg> = (indices.iter())
            .map(|x| ast::Arg {
                name: None,
                mutable: false,
                value: x.clone(),
            })
            .collect();
        let key = match sd.ctor {
            Some(new) => {
                let f = self.fnref(new, targs, Vec::new());
                self.call_fn(f, None, &args, span, Some(&k))
            }
            None => self.construct(sid, targs, sets, &args, span, Some(&k)),
        };
        if self.is_error(&key.ty) {
            return Err(());
        }
        let key = self.hold(key, "$key", span, out);
        Ok(Some(ast::Expr {
            kind: ExprKind::Index {
                base: base.clone(),
                indices: vec![key],
            },
            span: e.span,
        }))
    }
    /// `v` held in a new temporary, as a name for it.
    pub(super) fn hold(
        &mut self,
        v: TExpr,
        what: &str,
        span: Span,
        out: &mut Vec<TStmt>,
    ) -> ast::Expr {
        let name = format!("{what}{}", out.len());
        let t = self.bind(&name, v.ty.clone());
        out.push(TStmt::Let(t, v));
        ast::Expr {
            kind: ExprKind::Name(name),
            span,
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
        // `s[..hi]` with a literal `hi` ends at a cut, so `hi` may be the keys' end (R04.4.25).
        if let (None, Some(h), false, Some(k @ Ty::Dom(_))) = (lo, hi, through, &key) {
            if matches!(unparen(h).kind, ExprKind::Int(_)) {
                let f = self.index_fn_for(&s.ty, 1);
                if let (Some(f), Some(rt)) = (f, self.range_over(k.clone())) {
                    let r = self.range_value(None, Some(h), false, None, Some(&rt), span);
                    return self.index_call(f, s, vec![Some(r)], span);
                }
            }
        }
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
            self.err(span, Unbuilt::SliceOpenEnd);
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

/// `a[x, y]` is `a[(x, y)]`: the parts of a product key index it one by one (R04.2.11).
pub(super) fn packed(e: &ast::Expr) -> Option<ast::Expr> {
    let ExprKind::Index { base, indices } = &e.kind else {
        return None;
    };
    let [first, .., last] = &indices[..] else {
        return None;
    };
    let key = ast::Expr {
        kind: ExprKind::Tuple(indices.clone()),
        span: Span::new(first.span.start, last.span.end),
    };
    Some(ast::Expr {
        kind: ExprKind::Index {
            base: base.clone(),
            indices: vec![key],
        },
        span: e.span,
    })
}

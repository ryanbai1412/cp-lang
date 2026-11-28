//! Index syntax: `s[k]` reads through `Indexed`, `s[r]` calls the `@index` fn that cuts a view
//! (R07.4.5, R07.4.17).
use super::call::CallArg;
use super::member::Called;
use super::*;
use cpl_next_ir::unbuilt::Unbuilt;
use std::collections::HashMap;

/// What runs while a cell is lent: given the cell's name, it adds its statements.
pub(super) type InCell<'a, C> = Box<dyn FnOnce(&mut C, TExpr, &mut Vec<TStmt>) + 'a>;

impl Cx<'_> {
    /// `recv.name(args)` with already-typed arguments: used for `get` and `set` behind indexing.
    /// `s[i]` is `s.at(i)` of its `Indexed` impl (R12.4.5), read in place; a method of
    /// the type's own named `at` is not indexing.
    pub(super) fn elem(&mut self, s: TExpr, i: TExpr, span: Span) -> TExpr {
        let at = self.index_read();
        let rt = self.recv_ty(&s.ty);
        let read = self.d.sym(&at).and_then(|sym| {
            self.impls()
                .impl_member_by(&rt, sym, |f| f.region.is_none())
        });
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
    pub(super) fn literal_key(&mut self, base: &next::Expr, key: &next::Expr, ty: &Ty) {
        let next::Expr::Literal(_) = unparen(key.clone()) else {
            return;
        };
        let Some(v) = decls::literal_int(key) else {
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
        let keys = match unparen(base.clone()) {
            next::Expr::Name(n) => (n.ident())
                .and_then(|n| self.lookup(n.text()))
                .and_then(|(var, _)| self.view_keys.get(&var).copied()),
            b => literal_cut(&b),
        };
        // Only a view whose keys are known here can prove a literal outside them; any other
        // view's keys are checked as it runs (R07.4.5).
        let v = i128::from(v);
        if keys.is_none_or(|(lo, hi)| lo <= v && v < hi) {
            return;
        }
        let at = self.at(key);
        self.err(
            at,
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
            let named = !self.domains[*d].anonymous;
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
    pub(super) fn ranged_key(&mut self, s: &Ty, parts: &[next::Expr]) -> Option<Ty> {
        let ranged = |p: &next::Expr| matches!(unparen(p.clone()), next::Expr::Range(_));
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
            .d
            .sym(&self.index_read())
            .and_then(|sym| self.impls().impl_member_by(rt, sym, |f| f.region.is_none()))
        {
            Some(decls::Member::Method(m, ts, _)) => {
                let sub = Subst {
                    tys: &ts,
                    sets: &[],
                };
                let f = self.items.f(m);
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
                        .member(&self.items.t(t.tr).methods, &self.index_read())
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
    /// `recv[k] = v` as a call of the `[]=` method of `recv`'s impl, when it has one; `k` and `v`
    /// are checked against its parameters.
    pub(super) fn index_set(
        &mut self,
        recv: TExpr,
        k: &next::Expr,
        v: &next::Expr,
        at: Span,
    ) -> Option<TExpr> {
        let (m, ts, ss) = self.set_method(&recv.ty)?;
        let args = [k, v].map(|e| CallArg::plain(e));
        let f = self.fnref(m, ts, ss);
        let recv = self.recv_arg(m, recv, at);
        Some(self.call_method(f, recv, false, Some(&args), at, None))
    }
    /// `recv[k] = v` through `[]=`, as [`Self::index_set`], with `k` and `v` already read.
    pub(super) fn index_set_held(
        &mut self,
        recv: TExpr,
        k: TExpr,
        v: TExpr,
        at: Span,
    ) -> Option<TExpr> {
        let m = self.set_method(&recv.ty)?;
        let args = vec![TArg::Value(k), TArg::Value(v)];
        Some(self.call_member(recv, m, args, at, false))
    }
    /// The `[]=` method of `t`'s impl of the trait that declares the operator.
    fn set_method(&mut self, t: &Ty) -> Option<(FnId, Vec<Ty>, Vec<SetTy>)> {
        let tr = self.d.operator_traits("[]=").next()?;
        let name = self.d.operator_method(tr)?.0;
        let rt = self.recv_ty(t);
        let d = &self.d;
        let of_tr = |f: &decls::FnDecl| matches!(f.owner, Some(decls::Owner::Impl(i)) if d.imp(i).trait_.tr == tr);
        let found = (self.d.sym(name)).and_then(|sym| self.impls().impl_member_by(&rt, sym, of_tr));
        match found {
            Some(decls::Member::Method(m, ts, ss)) => Some((m, ts, ss)),
            _ => None,
        }
    }
    /// `t[i][j] = v` with `t` a table is E0503: its cell is `t[i, j]` (R01.8.5).
    fn table_point_again(
        &mut self,
        t: &next::Expr,
        i: &next::Expr,
        j: &next::Expr,
        span: Span,
    ) -> bool {
        let point = |e: &next::Expr| !matches!(unparen(e.clone()), next::Expr::Range(_));
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
        let first = format!("{}[{}]", self.text(self.at(t)), self.text(self.at(i)));
        let text = cpl_diag::msgs::check::REMOVED_SYNTAX_INDEXES_TWICE_0_3
            .say(&[&first, &self.text(self.at(j))]);
        let help = "all indices go in one pair of brackets: `g[r, c]`, never `g[r][c]`";
        let code = cpl_diag::codes::REMOVED_SYNTAX;
        self.err_help(span, Msg::verdict(code, text), Some(help.to_string()));
        self.expr(j, None);
        true
    }
    /// Runs `inner` while `root[k1][k2]..` is lent as a place: each level's cell through the
    /// `at` of its container's `IndexedMut` that lends one (R12.4.6), inside the level before.
    /// `inner` gets the innermost cell; `keys` come from [`Self::hoist_indices`].
    pub(super) fn through_cell(
        &mut self,
        root: TExpr,
        keys: Vec<Vec<TExpr>>,
        span: Span,
        out: &mut Vec<TStmt>,
        inner: InCell<'_, Self>,
    ) {
        let mut levels = keys.into_iter();
        let Some(first) = levels.next() else {
            return inner(self, root, out);
        };
        let rest: Vec<Vec<TExpr>> = levels.collect();
        let Some(key) = self.level_key(&root.ty, first, span) else {
            return;
        };
        self.lend_cell(
            root,
            key,
            span,
            out,
            Box::new(move |this, cell, out| this.through_cell(cell, rest, span, out, inner)),
        );
    }
    /// `root[k1]..[kn] = value` with `=`: the last container's `[]=` write when it has one
    /// (spec 12, W8), run while the levels before are lent; otherwise its `IndexedMut` lends the
    /// last cell too and `assign` writes it.
    pub(super) fn cell_set(
        &mut self,
        root: TExpr,
        mut keys: Vec<Vec<TExpr>>,
        value: TExpr,
        span: Span,
        out: &mut Vec<TStmt>,
        assign: InCell<'_, Self>,
    ) {
        let Some(last) = keys.pop() else {
            return;
        };
        let set = move |this: &mut Self, c: TExpr, out: &mut Vec<TStmt>| {
            let Some(k) = this.level_key(&c.ty, last, span) else {
                return;
            };
            if let Some(call) = this.index_set_held(c.clone(), k.clone(), value, span) {
                return out.push(TStmt::Expr(call));
            }
            this.lend_cell(c, k, span, out, assign)
        };
        self.through_cell(root, keys, span, out, Box::new(set));
    }
    /// One level's key from its parts: `a[x1, ..., xn]` is `a[K(x1, ..., xn)]` for an array
    /// keyed by a struct `K` that takes `n` arguments (decision 409, R04.2.12), and the tuple of
    /// them otherwise, as a product's key takes them (R04.2.11, decision 159).
    pub(super) fn level_key(
        &mut self,
        base: &Ty,
        mut parts: Vec<TExpr>,
        span: Span,
    ) -> Option<TExpr> {
        if parts.len() == 1 {
            return parts.pop();
        }
        let args: Vec<CallArg> = (parts.iter().cloned())
            .map(|p| {
                let at = p.span;
                CallArg::held(None, p, at)
            })
            .collect();
        if let Some(key) = self.ctor_key(base, &args, span) {
            return key;
        }
        let d = self.d.tuple(parts.len())?;
        let tys = parts.iter().map(|p| p.ty.clone()).collect();
        let sets = (0..self.items.s(d).n_sets)
            .map(|_| self.new_set())
            .collect();
        Some(TExpr {
            kind: TK::Struct(parts),
            ty: Ty::Struct(d, tys, sets),
            span,
        })
    }
    /// `s[x1, ..., xn]` read: the key [`Self::level_key`] makes, each part checked against
    /// the key's part (`..` parts a range over it, R04.4.11).
    pub(super) fn index_parts(
        &mut self,
        s: &Ty,
        parts: &[next::Expr],
        span: Span,
    ) -> Option<TExpr> {
        let args: Vec<CallArg> = parts.iter().map(|x| CallArg::plain(x)).collect();
        if let Some(key) = self.ctor_key(s, &args, span) {
            return key;
        }
        let want = match self.ranged_key(s, parts) {
            Some(Ty::Struct(_, fs, _)) => fs,
            _ => Vec::new(),
        };
        let checked = (parts.iter().enumerate())
            .map(|(i, x)| self.expr(x, Some(want.get(i).unwrap_or(&Ty::Int))))
            .collect();
        self.level_key(&Ty::Error, checked, span)
    }
    /// The key `K(args)` of an array keyed by a struct `K` that is not a tuple or record; E0303
    /// when `K` takes another number of arguments. `None` for any other key; `Some(None)` when
    /// the key is an error, already reported.
    fn ctor_key(&mut self, s: &Ty, args: &[CallArg], span: Span) -> Option<Option<TExpr>> {
        let k = self.key_of(s)?;
        let Ty::Struct(sid, targs, sets) = self.shallow(&k) else {
            return None;
        };
        let sd = self.items.s(sid);
        if sd.record || self.d.tuple(sd.fields.len()) == Some(sid) {
            return None;
        }
        let takes = (sd.ctor).map_or(sd.fields.len(), |c| self.items.f(c).params.len());
        if takes != args.len() {
            let (ks, n) = (self.show(&k), args.len());
            let msg = format!(
                "{n} indices into a `[{ks}]` array, whose key `{ks}` is built from {takes}"
            );
            self.err_help(
                span,
                Msg::verdict(cpl_diag::codes::WRONG_ARG_COUNT, msg),
                Some(format!("index it with one key: `[{ks}(...)]`")),
            );
            return Some(None);
        }
        let key = match sd.ctor {
            Some(new) => {
                let f = self.fnref(new, targs, Vec::new());
                self.call_fn(f, None, args, span, Some(&k))
            }
            None => self.construct(sid, targs, sets, args, span, Some(&k)),
        };
        Some((!self.is_error(&key.ty)).then_some(key))
    }
    /// Runs `inner` while cell `key` of `b` is lent, through the `at` of `b`'s `IndexedMut`
    /// (R12.4.6).
    fn lend_cell(
        &mut self,
        b: TExpr,
        key: TExpr,
        span: Span,
        out: &mut Vec<TStmt>,
        inner: InCell<'_, Self>,
    ) {
        // A cell of a place that was diagnosed is the error: its index is not fitted to it.
        if self.mentions_error(&b.ty) {
            return;
        }
        let i = self.index_arg(&b.ty, key);
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
            let Some(decls::Member::Method(m, ts, ss)) = self
                .d
                .sym(read)
                .and_then(|sym| self.impls().impl_member_by(&rt, sym, lends))
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
        let var = self.temp(&name, elem.clone());
        if let VarRef::Local(l) = var {
            self.b.locals[l].mutable = true;
        }
        let mut body = Vec::new();
        let cell = TExpr {
            kind: TK::Var(var),
            ty: elem,
            span,
        };
        inner(self, cell, &mut body);
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
    pub(super) fn cell_method(&mut self, e: &next::Expr, out: &mut Vec<TStmt>) -> bool {
        let next::Expr::Call(c) = e else {
            return false;
        };
        let Some(next::Expr::Field(callee)) = c.callee() else {
            return false;
        };
        let (Some(base), Some(name)) = (callee.base(), callee.name()) else {
            return false;
        };
        let next::Expr::Index(ix) = unparen(base) else {
            return false;
        };
        if ix.indices().next().is_none() {
            return false;
        }
        let seen = self.errors.len();
        let probe = self.expr(&next::Expr::Index(ix.clone()), None);
        self.errors.truncate(seen);
        let rt = self.shallow(&probe.ty);
        let writes = match self.member_named(&rt, self.d.sym(name.text())) {
            Some(decls::Member::Method(m, ..)) => {
                self.items.f(m).self_mode == SelfMode::Mut || self.d.writes_cache(m)
            }
            _ => false,
        };
        if !writes {
            return false;
        }
        let Some((root, keys)) = self.hoist_indices(&ix, out) else {
            return true;
        };
        let written: Vec<next::Arg> = c.args().map(|l| l.args().collect()).unwrap_or_default();
        let mut args = Vec::new();
        for a in &written {
            let a = CallArg::of(a);
            let arg = match a.written() {
                Some(next::Expr::Lambda(_)) | None => a,
                Some(_) if a.mutable => a,
                Some(v) => {
                    let v = v.clone();
                    let held = self.hoist(&v, (None, false), "$arg", out);
                    CallArg::held(a.label, held, a.at)
                }
            };
            args.push(arg);
        }
        let (name, name_at, span) = (name.text().to_string(), name.span(), self.at(e));
        self.through_cell(
            root,
            keys,
            self.at(&ix),
            out,
            Box::new(move |this, cell, out| {
                let v = this.method_on(
                    cell,
                    Called {
                        text: &name,
                        sym: this.d.sym(&name),
                        span: name_at,
                    },
                    &args,
                    span,
                );
                out.push(TStmt::Expr(v));
            }),
        );
        true
    }
    /// `recv.m(args)` on a checked receiver whose type declares `m`.
    fn method_on(&mut self, recv: TExpr, name: Called, args: &[CallArg], span: Span) -> TExpr {
        let rt = self.recv_ty(&recv.ty);
        let found = self.member_named(&rt, name.sym);
        let module = self.b.module;
        let hidden = (found.as_ref()).and_then(|x| self.d.hidden_member(x, &rt, name.text, module));
        if let Some((msg, help)) = hidden {
            self.err_help(name.span, msg, Some(help));
            return Self::error_node(span);
        }
        let Some(decls::Member::Method(m, ts, ss)) = found else {
            return Self::error_node(span);
        };
        let f = self.fnref(m, ts, ss);
        let recv = self.recv_arg(m, recv, span);
        self.call_method(f, recv, false, Some(args), span, None)
    }
    /// The root of `target`, checked as a place, and each level's keys, outer first, each read
    /// once into a temporary before any cell is lent, so `p[r] = p[p[r]]` reads `p` before
    /// writing it (R12.4.6). `None` when `target` is diagnosed and nothing more is to be done.
    pub(super) fn hoist_indices(
        &mut self,
        target: &next::IndexExpr,
        out: &mut Vec<TStmt>,
    ) -> Option<(TExpr, Vec<Vec<TExpr>>)> {
        let span = self.at(target);
        let base = unparen(target.base()?);
        let idx: Vec<next::Expr> = target.indices().collect();
        let (root, mut keys) = match &base {
            next::Expr::Index(inner) => {
                let i2: Vec<next::Expr> = inner.indices().collect();
                if let (Some(t), [i2], [j]) = (inner.base(), &i2[..], &idx[..]) {
                    if self.table_point_again(&t, i2, j, span) {
                        return None;
                    }
                }
                self.hoist_indices(inner, out)?
            }
            b => (self.place(b), Vec::new()),
        };
        // The root's own key is made and fitted to it before it is held: a tuple written in
        // place (R04.2.10), or the parts of a struct key (decision 409).
        let root_level = keys.is_empty() && !self.mentions_error(&root.ty);
        let level = match &idx[..] {
            [i] if root_level && matches!(unparen(i.clone()), next::Expr::Tuple(_)) => {
                let v = self.expr(i, Some(&Ty::Int));
                let v = self.index_arg(&root.ty, v);
                vec![self.hold(v, "$idx", out)]
            }
            [_, _, ..] if root_level => {
                let k = self.index_parts(&root.ty, &idx, span)?;
                let k = self.index_arg(&root.ty, k);
                vec![self.hold(k, "$key", out)]
            }
            _ => (idx.iter())
                .map(|i| self.hoist(i, (Some(&Ty::Int), false), "$idx", out))
                .collect(),
        };
        keys.push(level);
        Some((root, keys))
    }
    /// `e` read once into a temporary; with `copy`, a copy of its own, so it borrows no place a
    /// later write changes. A name or a literal reads the same later, so it is read where used.
    pub(super) fn hoist(
        &mut self,
        e: &next::Expr,
        (want, copy): (Option<&Ty>, bool),
        what: &str,
        out: &mut Vec<TStmt>,
    ) -> TExpr {
        let v = self.expr(e, want);
        if matches!(
            unparen(e.clone()),
            next::Expr::Name(_) | next::Expr::Literal(_)
        ) {
            return v;
        }
        let v = match copy {
            true => {
                let (ty, span) = (v.ty.clone(), v.span);
                let kind = TK::Owned(Box::new(v));
                TExpr { kind, ty, span }
            }
            false => v,
        };
        self.hold(v, what, out)
    }
    /// A binding the checker makes for its own use: no suggestion or fix names it.
    pub(super) fn temp(&mut self, role: &str, ty: Ty) -> VarRef {
        let v = self.bind(role, ty);
        match v {
            VarRef::Local(l) => self.b.locals[l].synthetic = true,
            VarRef::Global(g) => self.globals[g].synthetic = true,
        }
        v
    }
    /// `v` held in a new temporary, read as that temporary.
    pub(super) fn hold(&mut self, v: TExpr, what: &str, out: &mut Vec<TStmt>) -> TExpr {
        let name = format!("{what}{}", out.len());
        let (ty, span) = (v.ty.clone(), v.span);
        let t = self.temp(&name, ty.clone());
        out.push(TStmt::Let(t, v));
        TExpr {
            kind: TK::Var(t),
            ty,
            span,
        }
    }
    /// `s[lo..]`, `s[..hi]`, `s[..=hi]` and `s[..]`: the `@index` fn that takes the two ends, each
    /// optional, and whether `hi` is in (R07.4.6).
    pub(super) fn open_slice(
        &mut self,
        s: TExpr,
        (lo, hi, through): (Option<&next::Expr>, Option<&next::Expr>, bool),
        span: Span,
    ) -> TExpr {
        let key = self.key_of(&s.ty).map(|k| self.shallow(&k));
        // `s[..hi]` with a literal `hi` ends at a cut, so `hi` may be the keys' end (R04.4.25).
        if let (None, Some(h), false, Some(k @ Ty::Dom(_))) = (lo, hi, through, &key) {
            if matches!(unparen(h.clone()), next::Expr::Literal(_))
                && decls::literal_int(h).is_some()
            {
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
        let fd = self.items.f(f.decl);
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
                    let mk = |kind, ty| TExpr { kind, ty, span };
                    let inner = self.optional_of(&pty).unwrap_or(Ty::Error);
                    let no = mk(TK::Bool(false), Ty::Bool);
                    mk(TK::Struct(vec![no, mk(TK::Unset, inner)]), pty)
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

pub(super) fn unparen(mut e: next::Expr) -> next::Expr {
    while let next::Expr::Paren(p) = &e {
        match p.inner() {
            Some(inner) => e = inner,
            None => break,
        }
    }
    e
}

/// The keys `lo..hi` of `s[lo..hi]` or `s[lo..=hi]` cut with literal or left-out ends.
pub(super) fn literal_cut(e: &next::Expr) -> Option<(i128, i128)> {
    let next::Expr::Index(ix) = unparen(e.clone()) else {
        return None;
    };
    let keys: Vec<next::Expr> = ix.indices().collect();
    let [r] = &keys[..] else {
        return None;
    };
    let next::Expr::Range(re) = unparen(r.clone()) else {
        return None;
    };
    if re.step().is_some() || re.walk().is_some() {
        return None;
    }
    // An end left out is unbounded on that side: only the written end limits a literal key.
    let lit = |e: Option<next::Expr>, open: i128| match e.map(unparen) {
        None => Some(open),
        Some(e) => decls::literal_int(&e).map(i128::from),
    };
    let hi = re.hi();
    let closed = re.is_inclusive() && hi.is_some();
    let (a, b) = (lit(re.lo(), i128::MIN)?, lit(hi, i128::MAX)?);
    Some((a, if closed { b + 1 } else { b }))
}

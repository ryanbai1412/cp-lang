//! Type and function-set variables: unification, coercion, display.

use super::member::Called;
use super::*;
use cpl_next_decls::decls::Role;
use cpl_next_ir::unbuilt::Unbuilt;
use cpl_next_syntax::grammar::Decorator;

impl<'a> Cx<'a> {
    /// What `f` returns, with the `_`s its body fixed (R02.9.3): a body not checked yet is
    /// checked now, for its result. A call inside that body's own check, through a cycle of
    /// such fns, reads the result as declared.
    pub(super) fn decl_ret(&mut self, f: FnId) -> Ty {
        let decl = self.items.f(f);
        let checkable =
            !matches!(decl.body, cpl_next_decls::decls::Body::None) && decl.intrinsic.is_none();
        let fresh = !self.fns.contains_key(&f) && !self.ret_busy.contains(&f);
        if self.facts.ret_hole(f) && !self.found_ret.contains_key(&f) && checkable && fresh {
            self.ret_busy.push(f);
            let outer =
                std::mem::replace(&mut self.b, Body::new(self.top_doms.clone(), self.program));
            let copy = std::mem::take(&mut self.copy);
            let kept = std::mem::replace(&mut self.checks_kept, self.checks);
            let tf = self.check_fn(f, Vec::new());
            (self.copy, self.b, self.checks_kept) = (copy, outer, kept);
            self.ret_busy.pop();
            self.fns.insert(f, tf);
        }
        (self.found_ret.get(&f).cloned()).unwrap_or_else(|| self.items.f(f).ret.clone())
    }
    /// `t` with each `_` a variable its body solves.
    pub(super) fn open_holes(&mut self, t: &Ty) -> Ty {
        match t {
            Ty::Error => self.new_ty(),
            Ty::Struct(s, ts, ss) => {
                let ts = ts.iter().map(|t| self.open_holes(t)).collect();
                Ty::Struct(*s, ts, ss.clone())
            }
            _ => t.clone(),
        }
    }
    pub(super) fn new_ty(&mut self) -> Ty {
        self.tys.push(None);
        Ty::Var(self.tys.len() - 1)
    }
    pub(super) fn new_set(&mut self) -> SetTy {
        self.sets.push(SetVar::Root(Vec::new()));
        SetTy::Var(self.sets.len() - 1)
    }
    pub(super) fn root(&self, mut v: usize) -> usize {
        while let SetVar::Link(n) = &self.sets[v] {
            v = *n;
        }
        v
    }
    pub(super) fn shallow(&self, t: &Ty) -> Ty {
        let mut t = t.clone();
        while let Ty::Var(v) = &t {
            match &self.tys[*v] {
                Some(n) => t = n.clone(),
                None => break,
            }
        }
        self.listed(t)
    }
    /// `[D]T` is `List<T>` once `D` is known to have no count (R04.8.11).
    fn listed(&self, t: Ty) -> Ty {
        match (&t, self.role_structs.role_struct(Role::List)) {
            (Ty::Struct(a, ts, _), Some(list))
                if Some(*a) == self.role_structs.role_struct(Role::Array) && ts.len() == 2 =>
            {
                let (walk, dom) = (
                    self.rows.role_trait(Role::Discrete),
                    self.rows.role_trait(Role::Positions),
                );
                if self.impls().uncounted(&self.shallow(&ts[0]), walk, dom) {
                    return Ty::Struct(list, vec![ts[1].clone()], Vec::new());
                }
                t
            }
            _ => t,
        }
    }
    pub(super) fn shallow_set(&self, s: &SetTy) -> SetTy {
        match s {
            SetTy::Var(v) => {
                let r = self.root(*v);
                match &self.sets[r] {
                    SetVar::Bound(b) => b.clone(),
                    _ => SetTy::Var(r),
                }
            }
            other => other.clone(),
        }
    }
    /// Replaces the placeholder set of a fn type written in a body by a fresh variable.
    pub(super) fn fresh_sets(&mut self, t: &Ty) -> Ty {
        match t {
            Ty::Fn(ps, r, s) => {
                let s = if *s == SetTy::Var(usize::MAX) {
                    self.new_set()
                } else {
                    s.clone()
                };
                Ty::Fn(
                    ps.iter().map(|p| self.fresh_sets(p)).collect(),
                    Box::new(self.fresh_sets(r)),
                    s,
                )
            }
            Ty::Raw(e) => Ty::Raw(Box::new(self.fresh_sets(e))),
            Ty::RangeTo(e, m, lo, k) => Ty::RangeTo(Box::new(self.fresh_sets(e)), *m, *lo, *k),
            Ty::Mut(e) => Ty::Mut(Box::new(self.fresh_sets(e))),
            Ty::Struct(d, ts, ss) => {
                let mut ss: Vec<SetTy> = ss
                    .iter()
                    .map(|s| {
                        if *s == SetTy::Var(usize::MAX) {
                            self.new_set()
                        } else {
                            s.clone()
                        }
                    })
                    .collect();
                while ss.len() < self.items.s(*d).n_sets {
                    ss.push(self.new_set());
                }
                Ty::Struct(*d, ts.iter().map(|p| self.fresh_sets(p)).collect(), ss)
            }
            other => other.clone(),
        }
    }
    pub(super) fn zonk(&self, t: &Ty) -> Ty {
        match self.shallow(t) {
            Ty::Var(_) => Ty::Error,
            Ty::Raw(e) => Ty::Raw(Box::new(self.zonk(&e))),
            Ty::RangeTo(e, m, lo, k) => Ty::RangeTo(Box::new(self.zonk(&e)), m, lo, k),
            Ty::Mut(e) => Ty::Mut(Box::new(self.zonk(&e))),
            Ty::Struct(d, ts, ss) => {
                let mut named = self.named.borrow_mut();
                if named.0.insert(d) {
                    named.1.push(d);
                }
                drop(named);
                Ty::Struct(
                    d,
                    ts.iter().map(|t| self.zonk(t)).collect(),
                    ss.iter().map(|s| self.zonk_set(s)).collect(),
                )
            }
            Ty::Fn(ps, r, s) => Ty::Fn(
                ps.iter().map(|t| self.zonk(t)).collect(),
                Box::new(self.zonk(&r)),
                self.zonk_set(&s),
            ),
            Ty::Clo(c) => Ty::Clo(Box::new(self.zonk_clo(&c))),
            other => other,
        }
    }
    /// `t` with its solved variables followed, its unknown ones and its sets left as they are.
    pub(super) fn followed(&self, t: &Ty) -> Ty {
        match self.shallow(t) {
            Ty::Raw(e) => Ty::Raw(Box::new(self.followed(&e))),
            Ty::RangeTo(e, m, lo, k) => Ty::RangeTo(Box::new(self.followed(&e)), m, lo, k),
            Ty::Mut(e) => Ty::Mut(Box::new(self.followed(&e))),
            Ty::Ref(e) => Ty::Ref(Box::new(self.followed(&e))),
            Ty::Struct(d, ts, ss) => {
                Ty::Struct(d, ts.iter().map(|t| self.followed(t)).collect(), ss)
            }
            Ty::Fn(ps, r, s) => Ty::Fn(
                ps.iter().map(|t| self.followed(t)).collect(),
                Box::new(self.followed(&r)),
                s,
            ),
            other => other,
        }
    }
    pub(super) fn zonk_fnref(&self, f: &FnRef) -> FnRef {
        f.map(|t| self.zonk(t), |s| self.zonk_set(s))
    }
    pub(super) fn zonk_clo(&self, c: &CloRef) -> CloRef {
        match c {
            CloRef::Item(f) => CloRef::Item(self.zonk_fnref(f)),
            CloRef::Lambda(id, ts, ss) => CloRef::Lambda(
                *id,
                ts.iter().map(|t| self.zonk(t)).collect(),
                ss.iter().map(|s| self.zonk_set(s)).collect(),
            ),
        }
    }
    pub(super) fn zonk_set(&self, s: &SetTy) -> SetTy {
        // A variable solved to a set that names it again is a cycle, like one met in its own closures.
        if let SetTy::Var(v) = s {
            let r = self.root(*v);
            if let SetVar::Bound(b) = &self.sets[r] {
                if self.zonking.borrow().contains(&r) {
                    self.cyclic.set(true);
                    return SetTy::Closed(Vec::new());
                }
                self.zonking.borrow_mut().push(r);
                let out = self.zonk_set(&b.clone());
                self.zonking.borrow_mut().retain(|x| *x != r);
                return out;
            }
        }
        match self.shallow_set(s) {
            SetTy::Var(r) => {
                if self.zonking.borrow().contains(&r) {
                    self.cyclic.set(true);
                    return SetTy::Closed(Vec::new());
                }
                // Everything this variable includes, through "includes that variable" edges, is one group.
                let mark = self.zonking.borrow().len();
                let mut group = vec![r];
                let mut at = 0;
                let mut closures = Vec::new();
                let mut subs = Vec::new();
                while at < group.len() {
                    if let SetVar::Root(ms) = &self.sets[group[at]] {
                        for m in ms {
                            match m {
                                Member::Clo(c) => closures.push(c.clone()),
                                Member::Sub(s) => match self.shallow_set(s) {
                                    SetTy::Var(v) => {
                                        if !group.contains(&v) {
                                            group.push(v);
                                        }
                                    }
                                    other => subs.push(other),
                                },
                            }
                        }
                    }
                    at += 1;
                }
                // Only this variable is "being written out": meeting it again inside one of its own closures is a real cycle.
                self.zonking.borrow_mut().push(r);
                let mut out: Vec<Member> = closures
                    .iter()
                    .map(|c| Member::Clo(self.zonk_clo(c)))
                    .collect();
                out.extend(subs.iter().map(|s| Member::Sub(self.zonk_set(s))));
                out.sort();
                out.dedup();
                self.zonking.borrow_mut().truncate(mark);
                SetTy::Closed(out)
            }
            SetTy::Closed(ms) => SetTy::Closed(ms),
            p => p,
        }
    }
    pub(super) fn unify_set(&mut self, a: &SetTy, b: &SetTy) -> bool {
        let (a, b) = (self.shallow_set(a), self.shallow_set(b));
        match (a, b) {
            (SetTy::Var(x), SetTy::Var(y)) => {
                if x != y {
                    // Both are roots: `shallow_set` followed the links.
                    let was = std::mem::replace(&mut self.sets[x], SetVar::Link(y));
                    if let (SetVar::Root(ms), SetVar::Root(into)) = (was, &mut self.sets[y]) {
                        into.extend(ms);
                    }
                }
                true
            }
            (SetTy::Var(x), other) | (other, SetTy::Var(x)) => {
                // A variable with members cannot become someone else's set: the members would be lost.
                let empty = matches!(&self.sets[x], SetVar::Root(ms) if ms.is_empty());
                if empty {
                    self.sets[x] = SetVar::Bound(other);
                } else if let (SetTy::Param(j), Some(g)) = (&other, self.b.decl) {
                    // The fn's own set parameter, which each call passes: the variable's members
                    // are what the fn adds to it.
                    let j = *j;
                    let was = std::mem::replace(&mut self.sets[x], SetVar::Bound(other));
                    if let SetVar::Root(ms) = was {
                        self.param_adds.entry((g, j)).or_default().extend(ms);
                    }
                } else if let SetTy::Global(g, k) = other {
                    // A global's own set: the variable's members are what the program puts in it.
                    let was = std::mem::replace(&mut self.sets[x], SetVar::Bound(other));
                    if let SetVar::Root(ms) = was {
                        self.global_adds.entry((g, k)).or_default().extend(ms);
                    }
                } else if let SetVar::Root(ms) = &mut self.sets[x] {
                    ms.push(Member::Sub(other));
                }
                true
            }
            (SetTy::Global(g, k), SetTy::Global(h, j)) if (g, k) != (h, j) => {
                let adds = &mut self.global_adds;
                adds.entry((g, k))
                    .or_default()
                    .push(Member::Sub(SetTy::Global(h, j)));
                adds.entry((h, j))
                    .or_default()
                    .push(Member::Sub(SetTy::Global(g, k)));
                true
            }
            // Two of the fn's own set parameters made one: each holds what its caller passes
            // for the other.
            (SetTy::Param(i), SetTy::Param(j)) if i != j => match self.b.decl {
                Some(g) => {
                    let adds = &mut self.param_adds;
                    adds.entry((g, i))
                        .or_default()
                        .push(Member::Sub(SetTy::Param(j)));
                    adds.entry((g, j))
                        .or_default()
                        .push(Member::Sub(SetTy::Param(i)));
                    true
                }
                None => false,
            },
            (a, b) => a == b,
        }
    }
    fn occurs(&self, v: usize, t: &Ty) -> bool {
        match self.shallow(t) {
            Ty::Var(x) => x == v,
            Ty::Raw(e) | Ty::Ref(e) | Ty::Mut(e) | Ty::RangeTo(e, ..) => self.occurs(v, &e),
            Ty::Struct(_, ts, _) => ts.iter().any(|t| self.occurs(v, t)),
            Ty::Fn(ps, r, _) => ps.iter().any(|t| self.occurs(v, t)) || self.occurs(v, &r),
            Ty::Clo(c) => match &*c {
                CloRef::Item(f) => f.targs.iter().any(|t| self.occurs(v, t)),
                CloRef::Lambda(_, ts, _) => ts.iter().any(|t| self.occurs(v, t)),
            },
            _ => false,
        }
    }
    pub(super) fn unify(&mut self, a: &Ty, b: &Ty) -> bool {
        let (a, b) = (self.shallow(a), self.shallow(b));
        match (&a, &b) {
            (Ty::Var(x), Ty::Var(y)) if x == y => true,
            // An unknown solved by the error node is that error: what it reaches says nothing more.
            (Ty::Var(x), Ty::Error) | (Ty::Error, Ty::Var(x)) => {
                self.tys[*x] = Some(Ty::Error);
                true
            }
            (Ty::Error, _) | (_, Ty::Error) => true,
            // A variable is never solved to a type holding itself: no type is infinite.
            (Ty::Var(x), _) => {
                let ok = !self.occurs(*x, &b);
                if ok {
                    self.tys[*x] = Some(b);
                }
                ok
            }
            (_, Ty::Var(y)) => {
                let ok = !self.occurs(*y, &a);
                if ok {
                    self.tys[*y] = Some(a);
                }
                ok
            }
            (Ty::Dom(x), Ty::Dom(y))
                if self
                    .fresh_eq
                    .iter()
                    .any(|&p| p == (*x, *y) || p == (*y, *x)) =>
            {
                true
            }
            (Ty::Raw(x), Ty::Raw(y)) | (Ty::Mut(x), Ty::Mut(y)) => self.unify(x, y),
            (Ty::RangeTo(x, m, a, k), Ty::RangeTo(y, n, b, j)) => {
                m == n && a == b && k == j && self.unify(x, y)
            }
            (Ty::Struct(d1, t1, s1), Ty::Struct(d2, t2, s2)) => {
                d1 == d2
                    && t1.len() == t2.len()
                    && t1.iter().zip(t2).all(|(x, y)| self.unify(x, y))
                    && s1.iter().zip(s2).all(|(x, y)| self.unify_set(x, y))
            }
            (Ty::Fn(p1, r1, s1), Ty::Fn(p2, r2, s2)) => {
                p1.len() == p2.len()
                    && p1.iter().zip(p2).all(|(x, y)| self.unify(x, y))
                    && self.unify(r1, r2)
                    && self.unify_set(s1, s2)
            }
            _ => a == b,
        }
    }
    pub(super) fn show(&self, t: &Ty) -> String {
        show_ty(
            self.items,
            self.role_structs.role_struct(Role::Option),
            &self.domains,
            &self.b.generics,
            &|t| self.shallow(t),
            t,
        )
    }
    /// The signature of a closure, in the caller's terms.
    pub(super) fn clo_sig(&mut self, c: &CloRef) -> (Vec<Ty>, Ty) {
        match c {
            CloRef::Item(f) => {
                let fd = self.items.f(f.decl);
                let sub = Subst {
                    tys: &f.targs,
                    sets: &f.sets,
                };
                let ps = (fd.params.iter())
                    .map(|p| match p.mutable {
                        true => Ty::Mut(Box::new(sub.ty(&p.ty))),
                        false => sub.ty(&p.ty),
                    })
                    .collect();
                let r = self.ret_ty(f);
                (ps, r)
            }
            CloRef::Lambda(id, ..) => (
                self.lambdas[id].params.clone(),
                self.lambdas[id].ret.clone(),
            ),
        }
    }
    /// The result type of a call: its function types hold the sets the call passes for them.
    pub(super) fn ret_ty(&mut self, f: &FnRef) -> Ty {
        let ret = self.decl_ret(f.decl);
        self.result_of(f, &ret)
    }
    /// `t`, a type in `f`'s result (what it returns or yields), as one call of `f` gives it: its
    /// k-th result slot is the set the call passes after `f`'s own sets.
    pub(super) fn result_of(&mut self, f: &FnRef, t: &Ty) -> Ty {
        let sub = Subst {
            tys: &f.targs,
            sets: &f.sets,
        };
        let n = self.items.f(f.decl).n_sets;
        let at = |k| f.sets.get(n + k).cloned();
        let t = map_slots(&sub.ty(t), &|k| at(k).unwrap_or(SetTy::Closed(Vec::new())));
        self.fresh_sets(&t)
    }
    /// What trait fn `m` returns at `Self` and the trait's arguments `tys`: its fns are the ones
    /// the impl the type picks returns.
    pub(super) fn trait_ret(&mut self, m: FnId, tys: Vec<Ty>) -> Ty {
        let sets = self.trait_sets(m);
        self.trait_ret_in(m, tys, sets)
    }
    /// Fresh sets for each of trait fn `m`'s own and result sets, as a call of it passes.
    pub(super) fn trait_sets(&mut self, m: FnId) -> Vec<SetTy> {
        let fd = self.items.f(m);
        (0..fd.n_sets + fd.n_ret_sets)
            .map(|_| self.new_set())
            .collect()
    }
    /// `m`'s result under `tys` and the call's `sets`, the ones its arguments are put in.
    pub(super) fn trait_ret_in(&mut self, m: FnId, tys: Vec<Ty>, sets: Vec<SetTy>) -> Ty {
        let fd = self.items.f(m);
        let f = FnRef {
            decl: m,
            targs: tys,
            sets,
            ctx: 0,
        };
        if fd.n_ret_sets > 0 {
            self.set_call(&f);
        }
        let ret = self.decl_ret(m);
        self.result_of(&f, &ret)
    }
    /// `T` when `t` is `?T`.
    fn optional_id(&self) -> Option<StructId> {
        self.role_structs.role_struct(Role::Option)
    }
    /// `?t`.
    pub(super) fn optional_ty(&self, t: Ty) -> Ty {
        match self.optional_id() {
            Some(id) => Ty::Struct(id, vec![t], Vec::new()),
            None => Ty::Error,
        }
    }
    pub(super) fn optional_of(&self, t: &Ty) -> Option<Ty> {
        match self.shallow(t) {
            Ty::Struct(d, ts, _) if Some(d) == self.optional_id() => Some(ts[0].clone()),
            _ => None,
        }
    }
    /// `v` as a present value of the optional type `ty`.
    pub(super) fn some(&mut self, v: TExpr, ty: Ty) -> TExpr {
        let span = v.span;
        let yes = TExpr {
            kind: TK::Bool(true),
            ty: Ty::Bool,
            span,
        };
        TExpr {
            kind: TK::Struct(vec![yes, v]),
            ty,
            span,
        }
    }
    /// The optional `v`, held in a new variable, as its `some` flag and its value.
    /// A place is read where it is: nothing is moved out of it.
    pub(super) fn opened(&mut self, v: TExpr, inner: Ty) -> (Vec<TStmt>, TExpr, TExpr) {
        let span = v.span;
        let (hold, read) = self.held(v);
        let flag = TExpr {
            kind: TK::Field(Box::new(read.clone()), 0),
            ty: Ty::Bool,
            span,
        };
        let val = TExpr {
            kind: TK::Field(Box::new(read), 1),
            ty: inner,
            span,
        };
        (hold, flag, val)
    }
    /// `v` as a place: itself when it is one, else a new variable that holds it.
    pub(super) fn held(&mut self, v: TExpr) -> (Vec<TStmt>, TExpr) {
        fn place(e: &TExpr) -> bool {
            match &e.kind {
                TK::Var(_) => true,
                TK::Field(b, _) => place(b),
                _ => false,
            }
        }
        let span = v.span;
        let (hold, read) = if place(&v) {
            (Vec::new(), v)
        } else {
            let tmp = self.temp("$opt", v.ty.clone());
            let read = TExpr {
                kind: TK::Var(tmp),
                ty: v.ty.clone(),
                span,
            };
            (vec![TStmt::Let(tmp, v)], read)
        };
        (hold, read)
    }
    /// A value flows into a slot of type `want`. The closed list of conversions lives here.
    /// Whether `a` and `b` could be one type, an unknown fitting anything; nothing is solved.
    pub(super) fn fits(&self, a: &Ty, b: &Ty) -> bool {
        match (self.shallow(a), self.shallow(b)) {
            (Ty::Var(_) | Ty::Error, _) | (_, Ty::Var(_) | Ty::Error) => true,
            (Ty::Struct(x, xs, _), Ty::Struct(y, ys, _)) => {
                x == y && xs.len() == ys.len() && xs.iter().zip(&ys).all(|(p, q)| self.fits(p, q))
            }
            (Ty::Raw(p), Ty::Raw(q)) | (Ty::Ref(p), Ty::Ref(q)) | (Ty::Mut(p), Ty::Mut(q)) => {
                self.fits(&p, &q)
            }
            (r, Ty::Int) | (Ty::Int, r) if r.is_int_range() => true,
            (p, q) => p == q,
        }
    }
    /// Core's `@coerce` fn from a struct of `have`'s kind to one of `want`'s.
    pub(super) fn coercion(&self, have: &Ty, want: &Ty) -> Option<FnId> {
        let (Ty::Struct(h, ..), Ty::Struct(w, ..)) = (have, want) else {
            return None;
        };
        let kind = |t: Option<&Ty>| match t {
            Some(Ty::Struct(s, ..)) => Some(*s),
            _ => None,
        };
        (self.marks.marked_fns(Decorator::Coerce).iter().copied()).find(|f| {
            let fd = self.items.f(*f);
            h != w
                && kind(fd.params.first().map(|p| &p.ty)) == Some(*h)
                && kind(Some(&fd.ret)) == Some(*w)
        })
    }
    /// The element count of `e` when it is a list literal, as the checker builds one.
    fn literal_len(&self, e: &TExpr) -> Option<usize> {
        let TK::Block(stmts, Some(read)) = &e.kind else {
            return None;
        };
        let local = match &read.kind {
            TK::Var(VarRef::Local(l)) => self.b.locals.get(*l),
            TK::Var(VarRef::Global(g)) => self.globals.get(*g),
            _ => None,
        }?;
        let pushes = stmts.iter().filter(|s| matches!(s, TStmt::Expr(_))).count();
        (local.name == "$list").then_some(pushes)
    }
    fn coerce_by(&mut self, decl: FnId, e: TExpr, want: &Ty) -> TExpr {
        let span = e.span;
        // A literal whose length differs from a constant count is wrong before it runs (R04.8.10).
        if let (Some(n), Ty::Struct(a, ts, _)) = (self.literal_len(&e), want) {
            let array = Some(*a) == self.role_structs.role_struct(Role::Array);
            let count = ts.iter().find_map(|t| match self.shallow(t) {
                Ty::Dom(i) => self.domains[i].count,
                Ty::Range(lo, hi) if array => Some(hi - lo + 1),
                _ => None,
            });
            if let Some(c) = count.filter(|c| *c != n as i64) {
                let w = self.show(want);
                self.err(
                    span,
                    Msg::verdict(
                        cpl_diag::codes::TYPE_MISMATCH,
                        format!("a list of {n} elements where `{w}` has {c} cells"),
                    ),
                );
            }
        }
        let f = self.fnref(decl, Vec::new(), Vec::new());
        self.oblige(&f, span);
        let ty = self.ret_ty(&f);
        let param = {
            let sub = Subst {
                tys: &f.targs,
                sets: &f.sets,
            };
            sub.ty(&self.items.f(decl).params[0].ty)
        };
        let n = self.literal_len(&e);
        let arg = self.coerce(e, &param);
        if !self.unify(&ty, want) {
            self.err(
                span,
                Msg::verdict(
                    cpl_diag::codes::TYPE_MISMATCH,
                    format!("expected `{}`, found `{}`", self.show(want), self.show(&ty)),
                ),
            );
        }
        if let Some(n) = n {
            let fd = self.items.f(decl);
            let domain = self.rows.role_trait(Role::Positions);
            for (i, t) in f.targs.iter().enumerate() {
                let positions = fd
                    .bounds
                    .iter()
                    .any(|(b, r)| *b == i && Some(r.tr) == domain);
                if positions && matches!(self.shallow(t), Ty::Var(_)) {
                    self.b.list_doms.push((t.clone(), n, span));
                }
            }
        }
        TExpr {
            kind: TK::Call {
                f,
                recv: None,
                args: vec![TArg::Value(arg)],
            },
            ty,
            span,
        }
    }
    /// `have` and `want` are one container whose cells `want` narrows to a range of what `have`
    /// holds: storing it unchecked would put an out-of-range value in a cell (R04.4.24).
    fn narrows_cells(&self, have: &Ty, want: &Ty) -> bool {
        let (Ty::Struct(h, hs, _), Ty::Struct(w, ws, _)) = (self.shallow(have), self.shallow(want))
        else {
            return false;
        };
        // An array's key is its domain, never a cell: `[0..=3]T` does not narrow to `[0..=2]T`.
        let keyed = Some(h) == self.role_structs.role_struct(Role::Array);
        if keyed && (hs.first().map(|t| self.shallow(t)) != ws.first().map(|t| self.shallow(t))) {
            return false;
        }
        let cell = |a: &Ty, b: &Ty| match (self.shallow(a), self.shallow(b)) {
            (Ty::Int, Ty::Range(..)) => Some(true),
            (Ty::Range(a, b), Ty::Range(c, d)) => Some(a < c || b > d),
            (a, b) => (a == b).then_some(false),
        };
        h == w
            && hs.len() == ws.len()
            && (hs.iter().zip(&ws).map(|(a, b)| cell(a, b)))
                .try_fold(false, |any, c| Some(any || c?))
                == Some(true)
    }
    /// `t` with each range type in it, alone or a tuple's part, widened to its `Int`; `None` when
    /// one is a type argument of anything else. A range type is never a type argument: its value
    /// is an `Int`, checked into the range as it is stored (R03.1.4).
    pub(super) fn int_parts(&self, t: &Ty) -> Option<Ty> {
        match self.shallow(t) {
            t if t.is_int_range() => Some(Ty::Int),
            Ty::Struct(s, ts, sets) => {
                let parts = (ts.iter().map(|t| self.int_parts(t))).collect::<Option<Vec<Ty>>>()?;
                (parts == ts || self.is_tuple(s)).then_some(Ty::Struct(s, parts, sets))
            }
            t => Some(t),
        }
    }
    fn is_tuple(&self, s: StructId) -> bool {
        self.shapes.tuple(self.items.s(s).fields.len()) == Some(s)
    }
    /// Whether `t` is an array keyed by more than one axis, `[I, J]T` (R01.8.2).
    pub(super) fn is_table(&self, t: &Ty) -> bool {
        match self.shallow(t) {
            Ty::Struct(a, args, _) if Some(a) == self.role_structs.role_struct(Role::Array) => {
                matches!(args.first().map(|k| self.shallow(k)), Some(Ty::Struct(k, ..)) if self.is_tuple(k))
            }
            _ => false,
        }
    }
    /// A range type with ends known only as the program runs, given for the type parameter fn
    /// `d` gives back, is its `Int`, checked into the range as it is stored (R03.1.4): no
    /// instance is its own. One given for an optional it gives back is `None` outside the range
    /// (R10.2.4). A range with constant ends is a type argument like any type, so `read<1..=3>`
    /// runs the range type's own `Read`. The range type the call's value is held to, and
    /// whether that value is optional.
    pub(super) fn ranged_targ(&self, d: FnId, given: &mut [Ty]) -> Option<(usize, Ty, bool)> {
        let fd = self.items.f(d);
        let (i, opt) = (made_param(fd).map(|i| (i, false)))
            .or_else(|| self.made_optional(fd).map(|i| (i, true)))?;
        let t = self.shallow(given.get(i)?);
        if !self.runtime_ranged(&t) {
            return None;
        }
        let w = self.int_parts(&t).filter(|w| *w != t)?;
        given[i] = w;
        Some((i, t, opt))
    }
    /// Whether `t` has a range part whose ends are known only as the program runs.
    pub(super) fn runtime_ranged(&self, t: &Ty) -> bool {
        match self.shallow(t) {
            Ty::RangeOf(_) => true,
            Ty::Struct(_, ts, _) => ts.iter().any(|t| self.runtime_ranged(t)),
            _ => false,
        }
    }
    /// A call's value held to the range type `ranged_targ` gave. A fn that gives back its range
    /// `T` keeps the range as its instance's argument, so `T.MIN` in it is the range's.
    pub(super) fn held_to(&mut self, v: TExpr, ranged: Option<(usize, Ty, bool)>) -> TExpr {
        match ranged {
            Some((_, r, true)) => self.narrowed(v, &r),
            Some((i, r, false)) => {
                let mut v = v;
                if let TK::Call { f, .. } = &mut v.kind {
                    if let Some(t) = f.targs.get_mut(i) {
                        *t = r.clone();
                    }
                }
                self.coerce(v, &r)
            }
            None => v,
        }
    }
    /// The type parameter a fn gives back as an optional and takes in no parameter, as
    /// `read<T>(from: s) -> ?T` does.
    pub(super) fn made_optional(&self, fd: &decls::FnDecl) -> Option<usize> {
        match self.optional_of(&fd.ret)? {
            Ty::Param(i) if !takes(fd, i) => Some(i),
            _ => None,
        }
    }
    /// The optional `v`, of `want`'s `Int` parts, as an optional `want`: `None` too when a part
    /// lies outside its range (R10.2.4).
    pub(super) fn narrowed(&mut self, v: TExpr, want: &Ty) -> TExpr {
        let span = v.span;
        let mk = |kind, ty| TExpr { kind, ty, span };
        let opt = self.optional_ty(want.clone());
        let inner = self.optional_of(&v.ty).unwrap_or(Ty::Error);
        let (hold, flag, val) = self.opened(v, inner);
        let fits = self.within(&val, want);
        let ok = Self::both(Some(flag.clone()), fits).unwrap_or(flag);
        let held = self.coerce(val, want);
        let yes = self.some(held, opt.clone());
        let unset = vec![mk(TK::Bool(false), Ty::Bool), mk(TK::Unset, want.clone())];
        let no = mk(TK::Struct(unset), opt.clone());
        let pick = mk(
            TK::If(Box::new(ok), Box::new(yes), Box::new(no)),
            opt.clone(),
        );
        mk(TK::Block(hold, Some(Box::new(pick))), opt)
    }
    /// A store of `e` into a range whose least value lies in `least` and greatest in `most`,
    /// decided when checked: a constant outside every value the range can take is E0902, and a
    /// value inside every one needs no check (R03.1.10). `None` leaves it to run time.
    fn proved_store(
        &mut self,
        e: &TExpr,
        least: Option<(i64, i64)>,
        most: Option<(i64, i64)>,
        want: &Ty,
        span: Span,
    ) -> Option<TExpr> {
        let (i, j) = known_range(&self.b.let_ranges, &|v| self.var_ty(v), e)?;
        let outside = least.is_some_and(|l| i < l.0) || most.is_some_and(|m| i > m.1);
        if i == j && outside && self.constant(e) {
            self.const_out_of_range(e, i, want, span);
        } else if !(least.is_some_and(|l| l.1 <= i) && most.is_some_and(|m| j <= m.0)) {
            return None;
        }
        Some(TExpr {
            ty: want.clone(),
            ..e.clone()
        })
    }
    /// Whether `e` is a constant (R04.4.24): a literal, a `const` or a module-level `let`,
    /// or negation, `+`, `-` and `*` of those. A local never is, whatever it is known to hold.
    fn constant(&self, e: &TExpr) -> bool {
        match &e.kind {
            TK::Int(_) => true,
            TK::Neg(x) => self.constant(x),
            TK::Bin(BinOp::Add | BinOp::Sub | BinOp::Mul, l, r) => {
                self.constant(l) && self.constant(r)
            }
            TK::Var(VarRef::Global(g)) => {
                let v = &self.globals[*g];
                v.top && !v.mutable && !v.synthetic
            }
            _ => false,
        }
    }
    /// E0902 for a constant `i` (or a `let` that keeps it) stored where `want` cannot hold it.
    fn const_out_of_range(&mut self, e: &TExpr, i: i64, want: &Ty, span: Span) {
        let what = match &e.kind {
            TK::Var(v) => format!("`{}`", self.var_name(*v)),
            _ => i.to_string(),
        };
        let range = match want {
            Ty::RangeOf(r) => self.text(self.ranges[*r].at).to_string(),
            _ => self.show(want),
        };
        self.err(
            span,
            Msg::verdict(
                cpl_diag::codes::RANGE_STORE_CONST,
                format!("{what} is out of range for `{range}`"),
            ),
        );
    }
    /// A range type's end as a run-time value: a constant, or the global of the top-level `let`
    /// it names (R02.10.3), made before any body is checked; `less` is one below it, for an
    /// excluded end.
    fn range_end(&mut self, end: &cpl_next_ir::ty::End, less: bool, span: Span) -> TExpr {
        use cpl_next_ir::ty::End;
        let mk = |kind| TExpr {
            kind,
            ty: Ty::Int,
            span,
        };
        let at = match end {
            End::Lit(k) => return mk(TK::Int(if less { k.saturating_sub(1) } else { *k })),
            End::Global { id, .. } => mk(TK::Var(VarRef::Global(self.top_slots[id.0]))),
            End::Of(_) => {
                self.err(span, Unbuilt::RangeTypeOutsideBody);
                return Self::error_node(span);
            }
        };
        match less {
            true => mk(TK::Bin(BinOp::Sub, Box::new(at), Box::new(mk(TK::Int(1))))),
            false => at,
        }
    }
    /// Whether `v`, of `want`'s `Int` parts, lies in each range in `want`; `None` when `want`
    /// has none.
    fn within(&mut self, v: &TExpr, want: &Ty) -> Option<TExpr> {
        let span = v.span;
        let mk = |kind, ty| TExpr { kind, ty, span };
        let (lo, hi) = match self.shallow(want) {
            Ty::Range(lo, hi) => (
                Some(mk(TK::Int(lo), Ty::Int)),
                Some(mk(TK::Int(hi), Ty::Int)),
            ),
            Ty::RangeOf(r) => {
                let end = |e: Option<VarRef>| e.map(|e| mk(TK::Var(e), Ty::Int));
                (end(self.ranges[r].lo), end(self.ranges[r].hi))
            }
            Ty::RangeEnds { lo, hi, inclusive } if ends_at_global(&lo, &hi) => (
                Some(self.range_end(&lo, false, span)),
                Some(self.range_end(&hi, !inclusive, span)),
            ),
            Ty::Struct(..) => {
                let ((_, have), (_, wants)) = (self.field_tys(&v.ty)?, self.field_tys(want)?);
                let mut all = None;
                for (k, (h, w)) in have.into_iter().zip(wants).enumerate() {
                    let part = mk(TK::Field(Box::new(v.clone()), k), h);
                    let fits = self.within(&part, &w);
                    all = Self::both(all, fits);
                }
                return all;
            }
            _ => return None,
        };
        let cmp = |op, end: TExpr| mk(TK::Cmp(op, Box::new(v.clone()), Box::new(end)), Ty::Bool);
        Self::both(lo.map(|e| cmp(CmpOp::Ge, e)), hi.map(|e| cmp(CmpOp::Le, e)))
    }
    /// A tuple held to `want` part by part, each part coerced as it is stored (R03.1.4).
    fn held_parts(&mut self, e: TExpr, want: &Ty) -> TExpr {
        let span = e.span;
        let (Some((_, have)), Some((_, wants))) = (self.field_tys(&e.ty), self.field_tys(want))
        else {
            return e;
        };
        let input = self.is_input(&e);
        let ety = e.ty.clone();
        self.b.scopes.push(HashMap::new());
        let tmp = self.temp("$parts", ety.clone());
        if input {
            self.b.input_parts.insert(tmp);
        }
        let mut parts = Vec::new();
        for (i, (h, w)) in have.into_iter().zip(&wants).enumerate() {
            let var = TExpr {
                kind: TK::Var(tmp),
                ty: ety.clone(),
                span,
            };
            let part = TExpr {
                kind: TK::Field(Box::new(var), i),
                ty: h,
                span,
            };
            parts.push(self.coerce(part, w));
        }
        self.b.scopes.pop();
        let whole = TExpr {
            kind: TK::Struct(parts),
            ty: want.clone(),
            span,
        };
        TExpr {
            kind: TK::Block(vec![TStmt::Let(tmp, e)], Some(Box::new(whole))),
            ty: want.clone(),
            span,
        }
    }
    /// `J` where a result's fresh `C` is wanted, each in the same place of one struct: `Range<J>`
    /// for a `domain C` slot, `Rooted<V, J, A>` for a `Rooted<V, _, A>`.
    fn fresh_pairs(&self, have: &Ty, want: &Ty) -> Vec<(usize, usize)> {
        let (Ty::Struct(s1, a, _), Ty::Struct(s2, b, _)) = (have, want) else {
            return Vec::new();
        };
        if s1 != s2 {
            return Vec::new();
        }
        (a.iter().zip(b))
            .filter_map(|(x, y)| match (self.shallow(x), self.shallow(y)) {
                (Ty::Dom(j), Ty::Dom(c))
                    if j != c
                        && self.doms.fresh_dom(c)
                        && crate::pattern::holds_dom(&self.b.ret, c) =>
                {
                    Some((j, c))
                }
                _ => None,
            })
            .collect()
    }
    /// Returning `J` as the result's fresh `C`: `C` takes `J`'s ends, and in this body `J`
    /// stands for `C`, so `[J]T` is the result's `[C]T` (R02.9.3).
    fn return_fresh(&mut self, pairs: Vec<(usize, usize)>, e: TExpr, want: Ty) -> TExpr {
        let span = e.span;
        let var = |r| TExpr {
            kind: TK::Var(r),
            ty: Ty::Int,
            span,
        };
        // The value first: a call's result sets `J`'s ends as it returns.
        let (mut pre, e) = self.held(e);
        for (j, c) in pairs {
            self.fresh_eq.push((j, c));
            let (from, to) = (&self.domains[j], &self.domains[c]);
            pre.push(TStmt::Assign(var(to.lo), var(from.lo)));
            pre.push(TStmt::Assign(var(to.len), var(from.len)));
        }
        crate::expr::block(pre, TExpr { ty: want, ..e })
    }
    pub(super) fn coerce(&mut self, e: TExpr, want: &Ty) -> TExpr {
        let (have, want_s) = (self.shallow(&e.ty), self.shallow(want));
        let span = e.span;
        let pairs = self.fresh_pairs(&have, &want_s);
        // The rest of the result is the value's: `A` of `Rooted<V, _, _>` is what the body made.
        let as_fresh = (pairs.iter()).fold(self.zonk(&have), |t, &(j, c)| {
            crate::pattern::retag(&t, j, c)
        });
        if !pairs.is_empty() && self.unify(&as_fresh, &want_s) {
            return self.return_fresh(pairs, e, want_s);
        }
        // `T` converts to `?T` (R02.4.2).
        if let Some(inner) = self.optional_of(&want_s) {
            if self.optional_of(&have).is_none() && !matches!(have, Ty::Var(_) | Ty::Error) {
                let v = self.coerce(e, &inner);
                return self.some(v, want_s);
            }
        }
        if let Some(f) = self.coercion(&have, &want_s) {
            return self.coerce_by(f, e, &want_s);
        }
        match (&have, &want_s) {
            // An anonymous range's position is its `Int` (R04.3.4).
            (Ty::Dom(d), Ty::Int) if self.domains[*d].anonymous => self.position_int(e),
            // An integer literal is that position of an integer domain (R04.2.18).
            (Ty::Int, Ty::Dom(_)) if matches!(e.kind, TK::Int(_) | TK::Neg(_)) => {
                self.position(want_s.clone(), e, CheckKind::Domain)
            }
            // An `Int` where an `I128` is expected widens to it (R02.1).
            (h, Ty::I128) if *h == Ty::Int || h.is_int_range() => TExpr {
                kind: TK::ToI128(Box::new(e)),
                ty: Ty::I128,
                span,
            },
            // An integer literal in a `U64` slot is that `U64`: its bits (R02.1).
            (Ty::Int, Ty::U64) if matches!(e.kind, TK::Int(_)) => TExpr {
                kind: TK::ToU64(Box::new(e)),
                ty: Ty::U64,
                span,
            },
            // An integer literal where an `F64` is expected is that `F64`, if it holds it (R01.4.2).
            (Ty::Int, Ty::F64) if let TK::Int(i) = e.kind => {
                if i as f64 as i128 != i128::from(i) {
                    self.err(
                        span,
                        Msg::verdict(
                            cpl_diag::codes::TYPE_MISMATCH,
                            format!("`F64` cannot hold {i} exactly: write `F64({i})`"),
                        ),
                    );
                }
                TExpr {
                    kind: TK::Float((i as f64).to_bits()),
                    ty: Ty::F64,
                    span,
                }
            }
            // A tuple's parts are held to their types one by one (R03.1.4).
            (Ty::Struct(a, hs, _), Ty::Struct(b, ws, _))
                if a == b
                    && hs != ws
                    && self.is_tuple(*a)
                    && self.int_parts(&want_s).is_some_and(|w| w != want_s) =>
            {
                self.held_parts(e, &want_s)
            }
            // Loads widen, stores check (R03.1.4).
            (r, Ty::Int) if r.is_int_range() => TExpr { ty: Ty::Int, ..e },
            (Ty::RangeOf(a), Ty::RangeOf(b)) if a == b => e,
            (h, Ty::RangeTo(d, m, lo, k)) if *h == Ty::Int || h.is_int_range() => {
                let int = |i| TExpr {
                    kind: TK::Int(i),
                    ty: Ty::Int,
                    span,
                };
                let (lo, k, m) = (*lo, *k, *m);
                let text = self.items.f(m).name.clone();
                let name = Called {
                    text: &text,
                    sym: self.syms.get(&text),
                    span,
                };
                let Some(end) = self.domain_fn_of((**d).clone(), m, name, None, span) else {
                    let what = format!("`{}` has no `{}`", self.show(d), text);
                    self.err(span, Msg::verdict(cpl_diag::codes::UNKNOWN_FIELD, what));
                    return TExpr { ty: want_s, ..e };
                };
                let end = match k {
                    0 => end,
                    k => TExpr {
                        kind: TK::Bin(BinOp::Add, Box::new(end), Box::new(int(k))),
                        ty: Ty::Int,
                        span,
                    },
                };
                let v = TExpr { ty: Ty::Int, ..e };
                let v = self.checked_between(v, Some(int(lo)), Some(end), span);
                TExpr { ty: want_s, ..v }
            }
            (h, Ty::RangeOf(r)) if *h == Ty::Int || h.is_int_range() => {
                let end = |v: Option<VarRef>| {
                    v.map(|v| TExpr {
                        kind: TK::Var(v),
                        ty: Ty::Int,
                        span,
                    })
                };
                // Each end's slot keeps what it can be, so a store is proved as with fixed ends.
                let held = |v: Option<VarRef>, open: i64| match v {
                    Some(v) => self.b.let_ranges.get(&v).copied(),
                    None => Some((open, open)),
                };
                let (lo, hi) = (self.ranges[*r].lo, self.ranges[*r].hi);
                let (least, most) = (held(lo, i64::MIN), held(hi, i64::MAX));
                if let Some(done) = self.proved_store(&e, least, most, &want_s, span) {
                    return done;
                }
                let (lo, hi) = (end(lo), end(hi));
                let v = TExpr { ty: Ty::Int, ..e };
                let v = self.checked_between(v, lo, hi, span);
                TExpr { ty: want_s, ..v }
            }
            (h, Ty::RangeEnds { lo, hi, inclusive })
                if (*h == Ty::Int || h.is_int_range()) && ends_at_global(lo, hi) =>
            {
                let (lo, hi, inclusive) = (lo.clone(), hi.clone(), *inclusive);
                // A top-level `let` end lies in its declared range: a constant outside every
                // value the range can take is an error now, and a value inside every one needs
                // no check (R03.1.10).
                let least = lo.bounds();
                let most = (hi.bounds()).map(|(a, b)| match inclusive {
                    true => (a, b),
                    false => (a.saturating_sub(1), b.saturating_sub(1)),
                });
                if let Some(done) = self.proved_store(&e, least, most, &want_s, span) {
                    return done;
                }
                let lo = self.range_end(&lo, false, span);
                let hi = self.range_end(&hi, !inclusive, span);
                let v = TExpr { ty: Ty::Int, ..e };
                let v = self.checked_between(v, Some(lo), Some(hi), span);
                TExpr { ty: want_s, ..v }
            }
            (Ty::Range(a, b), Ty::Range(lo, hi)) if lo <= a && b <= hi => TExpr { ty: want_s, ..e },
            (h, Ty::Range(lo, hi)) if *h == Ty::Int || h.is_int_range() => {
                let (lo, hi) = (*lo, *hi);
                // A constant is checked here, not when the program runs (R03.1.10), and so is
                // arithmetic on the `let`s that keep one (R02.10.2); a value known to fit needs
                // no check.
                if let Some(done) =
                    self.proved_store(&e, Some((lo, lo)), Some((hi, hi)), &want_s, span)
                {
                    return done;
                }
                let int = |i| TExpr {
                    kind: TK::Int(i),
                    ty: Ty::Int,
                    span,
                };
                let v = TExpr { ty: Ty::Int, ..e };
                let v = self.checked_between(v, Some(int(lo)), Some(int(hi)), span);
                TExpr { ty: want_s, ..v }
            }
            (Ty::Clo(c), Ty::Fn(ps, r, set)) => {
                let (cps, cr) = self.clo_sig(c);
                // The result is joined even when the parameters don't fit, so the error is the only one.
                let fits =
                    cps.len() == ps.len() && cps.iter().zip(ps).all(|(x, y)| self.unify(x, y));
                let ok = self.unify(&cr, r) && fits;
                if !ok {
                    self.err(
                        span,
                        Msg::verdict(
                            cpl_diag::codes::TYPE_MISMATCH,
                            format!("this function does not fit `{}`", self.show(&want_s)),
                        ),
                    );
                }
                match self.shallow_set(set) {
                    SetTy::Var(v) => {
                        if let SetVar::Root(ms) = &mut self.sets[v] {
                            ms.push(Member::Clo((**c).clone()));
                        }
                    }
                    SetTy::Param(i) => {
                        if let Some(f) = self.b.decl {
                            let m = Member::Clo((**c).clone());
                            self.param_adds.entry((f, i)).or_default().push(m);
                        }
                    }
                    SetTy::Global(g, k) => {
                        let m = Member::Clo((**c).clone());
                        self.global_adds.entry((g, k)).or_default().push(m);
                    }
                    _ => self.err(
                        span,
                        Msg::internal("a function value is put in a set no fn or call owns"),
                    ),
                }
                TExpr {
                    kind: TK::Inject(Box::new(e)),
                    ty: want_s,
                    span,
                }
            }
            (Ty::Fn(p1, r1, s1), Ty::Fn(p2, r2, s2)) => {
                let ok = p1.len() == p2.len()
                    && p1.iter().zip(p2).all(|(x, y)| self.unify(x, y))
                    && self.unify(r1, r2);
                if !ok {
                    self.err(
                        span,
                        Msg::verdict(
                            cpl_diag::codes::TYPE_MISMATCH,
                            format!(
                                "expected `{}`, found `{}`",
                                self.show(&want_s),
                                self.show(&have)
                            ),
                        ),
                    );
                }
                let (a, b) = (self.shallow_set(s1), self.shallow_set(s2));
                match (&a, &b) {
                    _ if a == b => e,
                    (SetTy::Var(_), SetTy::Var(_)) if !self.b.joining => {
                        self.unify_set(&a, &b);
                        e
                    }
                    (_, SetTy::Var(v)) => {
                        if let SetVar::Root(ms) = &mut self.sets[*v] {
                            ms.push(Member::Sub(a.clone()));
                        }
                        TExpr {
                            kind: TK::Widen(Box::new(e)),
                            ty: want_s,
                            span,
                        }
                    }
                    (SetTy::Var(_), _) => {
                        self.unify_set(&a, &b);
                        e
                    }
                    // Into a global's own set: the program puts `a` in it.
                    (_, SetTy::Global(g, k)) => {
                        let m = Member::Sub(a.clone());
                        self.global_adds.entry((*g, *k)).or_default().push(m);
                        TExpr {
                            kind: TK::Widen(Box::new(e)),
                            ty: want_s,
                            span,
                        }
                    }
                    // Into the fn's own set parameter, which each call passes: the fn adds `a`.
                    (_, SetTy::Param(j)) if self.b.decl.is_some() => {
                        let g = self.b.decl.unwrap_or(FnId(0));
                        let m = Member::Sub(a.clone());
                        self.param_adds.entry((g, *j)).or_default().push(m);
                        TExpr {
                            kind: TK::Widen(Box::new(e)),
                            ty: want_s,
                            span,
                        }
                    }
                    _ => {
                        self.err(span, Unbuilt::FnValueSetsDiffer);
                        e
                    }
                }
            }
            // A struct whose function fields hold fewer functions than the slot allows is rebuilt field by field.
            (Ty::Struct(d1, t1, s1), Ty::Struct(d2, t2, s2))
                if d1 == d2
                    && s1.iter().zip(s2).any(|(a, b)| {
                        let (a, b) = (self.shallow_set(a), self.shallow_set(b));
                        a != b
                            && (self.b.joining || !matches!(a, SetTy::Var(_)))
                            && matches!(b, SetTy::Var(_) | SetTy::Param(_) | SetTy::Global(..))
                    }) =>
            {
                for (x, y) in t1.iter().zip(t2) {
                    self.unify(x, y);
                }
                let fields = self.items.s(*d1).fields.clone();
                let (from, to) = (Subst { tys: t1, sets: s1 }, Subst { tys: t2, sets: s2 });
                self.b.scopes.push(HashMap::new());
                let tmp = self.temp("$widen", have.clone());
                let read = TExpr {
                    kind: TK::Var(tmp),
                    ty: have.clone(),
                    span,
                };
                let mut out = Vec::new();
                for (i, (_, fty)) in fields.iter().enumerate() {
                    let f = TExpr {
                        kind: TK::Field(Box::new(read.clone()), i),
                        ty: from.ty(fty),
                        span,
                    };
                    out.push(self.coerce(f, &to.ty(fty)));
                }
                self.b.scopes.pop();
                TExpr {
                    kind: TK::Block(
                        vec![TStmt::Let(tmp, e)],
                        Some(Box::new(TExpr {
                            kind: TK::Struct(out),
                            ty: want_s.clone(),
                            span,
                        })),
                    ),
                    ty: want_s,
                    span,
                }
            }
            _ => {
                let downstream = self.mentions_error(&have) || self.mentions_error(&want_s);
                if !self.unify(&have, &want_s) && !downstream {
                    let help = self.narrows_cells(&have, &want_s).then(|| {
                        let text = self.text(span);
                        format!("check each element into the range: `[x for x in {text}]`")
                    });
                    self.err_help(
                        span,
                        Msg::verdict(
                            cpl_diag::codes::TYPE_MISMATCH,
                            format!(
                                "expected `{}`, found `{}`",
                                self.show(&want_s),
                                self.show(&have)
                            ),
                        ),
                        help,
                    );
                }
                e
            }
        }
    }
    /// The same type with new, empty sets: the type of a place two values with different sets both flow into.
    pub(super) fn join_ty(&mut self, t: &Ty) -> Ty {
        match self.shallow(t) {
            Ty::Fn(ps, r, _) => Ty::Fn(ps, r, self.new_set()),
            Ty::Clo(c) => {
                let (ps, r) = self.clo_sig(&c);
                Ty::Fn(ps, Box::new(r), self.new_set())
            }
            Ty::Struct(d, ts, ss) if !ss.is_empty() => {
                Ty::Struct(d, ts, ss.iter().map(|_| self.new_set()).collect())
            }
            other => other,
        }
    }
}

/// Whether `t` names type parameter `i`.
pub(super) fn mentions(t: &Ty, i: usize) -> bool {
    match t {
        Ty::Param(j) => *j == i,
        Ty::Raw(t) | Ty::Ref(t) | Ty::Lent(t) | Ty::Mut(t) | Ty::RangeTo(t, ..) => mentions(t, i),
        Ty::Struct(_, ts, _) => ts.iter().any(|t| mentions(t, i)),
        Ty::Fn(ps, r, _) => ps.iter().any(|t| mentions(t, i)) || mentions(r, i),
        _ => false,
    }
}

/// The type parameter a fn makes its result of: its result is one, and no parameter takes it,
/// so the fn builds the value rather than passing one through.
pub(super) fn made_param(fd: &decls::FnDecl) -> Option<usize> {
    match fd.ret {
        Ty::Param(i) if !takes(fd, i) => Some(i),
        _ => None,
    }
}

/// Whether fn `fd` takes type parameter `i` in a parameter or its receiver: then a call's `i`
/// is what it is given, not made by the fn, so `List<0..=9>.pop()` is that list's own.
fn takes(fd: &decls::FnDecl, i: usize) -> bool {
    (fd.params.iter()).any(|p| mentions(&p.ty, i))
        || fd.self_ty.as_ref().is_some_and(|s| mentions(s, i))
}

/// `t` has no variable or error left in it.
pub(super) fn ground(t: &Ty) -> bool {
    match t {
        Ty::Var(_) | Ty::Error => false,
        Ty::Raw(e) | Ty::Ref(e) | Ty::Lent(e) | Ty::RangeTo(e, ..) => ground(e),
        Ty::Struct(_, ts, _) => ts.iter().all(ground),
        Ty::Fn(ps, r, _) => ps.iter().all(ground) && ground(r),
        _ => true,
    }
}

/// How a diagnostic writes `t`, once `resolve` has looked through what inference has solved:
/// `domains` and `generics` name the domains and type parameters in scope.
pub fn show_ty(
    items: &dyn cpl_next_decls::views::Items,
    option: Option<StructId>,
    domains: &[DomInfo],
    generics: &[Sym],
    resolve: &dyn Fn(&Ty) -> Ty,
    t: &Ty,
) -> String {
    let show = |t: &Ty| show_ty(items, option, domains, generics, resolve, t);
    match resolve(t) {
        Ty::Int => "Int".into(),
        Ty::Range(lo, i64::MAX) => format!("{lo}.."),
        Ty::Range(lo, hi) => format!("{lo}..={hi}"),
        Ty::RangeOf(_) => "a range with run-time ends".into(),
        Ty::AnyRange(_) => "_..=_".into(),
        Ty::RangeEnds { lo, hi, inclusive } => {
            let end = |e: &cpl_next_ir::ty::End| match e {
                cpl_next_ir::ty::End::Lit(k) => k.to_string(),
                cpl_next_ir::ty::End::Of(t) => show(t),
                cpl_next_ir::ty::End::Global { id, .. } => items.text(items.top_name(*id)).into(),
            };
            let dots = if inclusive { "..=" } else { ".." };
            format!("{}{dots}{}", end(&lo), end(&hi))
        }
        Ty::RangeTo(d, m, lo, k) => {
            let k = match k {
                0 => String::new(),
                k if k > 0 => format!(" + {k}"),
                k => format!(" - {}", -k),
            };
            lo.to_string() + &format!("..={}.{}{k}", show(&d), items.f(m).name)
        }
        Ty::Const(cpl_next_ir::ty::ConstArg::Int(k)) => k.to_string(),
        Ty::Const(cpl_next_ir::ty::ConstArg::At(_)) => "a constant".into(),
        Ty::Bool => "Bool".into(),
        Ty::Char => "Char".into(),
        Ty::U64 => "U64".into(),
        Ty::I128 => "I128".into(),
        Ty::F64 => "F64".into(),
        Ty::Unit => "()".into(),
        Ty::Raw(e) => format!("Raw<{}>", show(&e)),
        Ty::Mut(e) => format!("mut {}", show(&e)),
        Ty::Ref(e) => format!("&{}", show(&e)),
        Ty::Lent(e) => show(&e),
        Ty::Struct(d, ts, _) if Some(d) == option => format!("?{}", show(&ts[0])),
        Ty::Struct(d, ts, _) => {
            let n = &items.s(d).name;
            if items.s(d).record {
                let fields = items.s(d).fields.iter().zip(&ts);
                let fs: Vec<String> = (fields
                    .map(|((f, _), t)| format!("{}: {}", items.text(*f), show(t))))
                .collect();
                return format!("({})", fs.join(", "));
            }
            if ts.is_empty() {
                n.clone()
            } else {
                format!(
                    "{n}<{}>",
                    ts.iter().map(&show).collect::<Vec<_>>().join(", ")
                )
            }
        }
        Ty::Fn(ps, r, _) => format!(
            "({}) -> {}",
            ps.iter().map(&show).collect::<Vec<_>>().join(", "),
            show(&r)
        ),
        Ty::Clo(_) => "a function".into(),
        Ty::Param(i) => (generics.get(i)).map_or("?".into(), |g| items.text(*g).to_string()),
        Ty::Dom(d) => domains[d].name.clone(),
        Ty::Var(_) => "_".into(),
        Ty::Error => "<error>".into(),
    }
}

/// The range an `Int` expression is known to lie in: a literal, a `let` that keeps one, a
/// variable of a range type, and
/// `+`, `-` and `*` of those (R02.10.2); no loop or call is followed.
pub(super) fn known_range(
    lets: &HashMap<VarRef, (i64, i64)>,
    declared: &dyn Fn(VarRef) -> Ty,
    e: &TExpr,
) -> Option<(i64, i64)> {
    match &e.kind {
        TK::Int(k) => Some((*k, *k)),
        TK::Neg(x) => {
            let (a, b) = known_range(lets, declared, x)?;
            Some((b.checked_neg()?, a.checked_neg()?))
        }
        TK::Var(v) => lets.get(v).copied().or(match declared(*v) {
            Ty::Range(lo, hi) => Some((lo, hi)),
            _ => None,
        }),
        TK::Bin(op, l, r) => {
            let ((a, b), (c, d)) = (
                known_range(lets, declared, l)?,
                known_range(lets, declared, r)?,
            );
            match op {
                BinOp::Add => Some((a.checked_add(c)?, b.checked_add(d)?)),
                BinOp::Sub => Some((a.checked_sub(d)?, b.checked_sub(c)?)),
                BinOp::Mul => {
                    let ps = [
                        a.checked_mul(c)?,
                        a.checked_mul(d)?,
                        b.checked_mul(c)?,
                        b.checked_mul(d)?,
                    ];
                    Some((*ps.iter().min()?, *ps.iter().max()?))
                }
                _ => None,
            }
        }
        _ => match &e.ty {
            Ty::Range(lo, hi) => Some((*lo, *hi)),
            _ => None,
        },
    }
}

/// Whether a range type ends at a top-level `let`, read as it runs (R02.10.3).
fn ends_at_global(lo: &cpl_next_ir::ty::End, hi: &cpl_next_ir::ty::End) -> bool {
    use cpl_next_ir::ty::End;
    [lo, hi].iter().any(|e| matches!(e, End::Global { .. }))
}

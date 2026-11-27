//! Type and function-set variables: unification, coercion, display.

use super::*;

impl<'a> Cx<'a> {
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
        t
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
                while ss.len() < self.d.s(*d).n_sets {
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
            Ty::Struct(d, ts, ss) => Ty::Struct(
                d,
                ts.iter().map(|t| self.zonk(t)).collect(),
                ss.iter().map(|s| self.zonk_set(s)).collect(),
            ),
            Ty::Fn(ps, r, s) => Ty::Fn(
                ps.iter().map(|t| self.zonk(t)).collect(),
                Box::new(self.zonk(&r)),
                self.zonk_set(&s),
            ),
            Ty::Clo(c) => Ty::Clo(Box::new(self.zonk_clo(&c))),
            other => other,
        }
    }
    pub(super) fn zonk_fnref(&self, f: &FnRef) -> FnRef {
        FnRef {
            decl: f.decl,
            targs: f.targs.iter().map(|t| self.zonk(t)).collect(),
            sets: f.sets.iter().map(|s| self.zonk_set(s)).collect(),
        }
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
            SetTy::RetOf(f, k) => SetTy::RetOf(Box::new(self.zonk_fnref(&f)), k),
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
                } else if let SetVar::Root(ms) = &mut self.sets[x] {
                    ms.push(Member::Sub(other));
                }
                true
            }
            (a, b) => a == b,
        }
    }
    fn occurs(&self, v: usize, t: &Ty) -> bool {
        match self.shallow(t) {
            Ty::Var(x) => x == v,
            Ty::Raw(e) | Ty::Ref(e) => self.occurs(v, &e),
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
            (Ty::Error, _) | (_, Ty::Error) => true,
            (Ty::Var(x), Ty::Var(y)) if x == y => true,
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
            (Ty::Raw(x), Ty::Raw(y)) => self.unify(x, y),
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
        match self.shallow(t) {
            Ty::Int => "Int".into(),
            Ty::Range(lo, hi) => format!("{lo}..={hi}"),
            Ty::RangeOf(_) => "a range with run-time ends".into(),
            Ty::Bool => "Bool".into(),
            Ty::Char => "Char".into(),
            Ty::U64 => "U64".into(),
            Ty::Unit => "()".into(),
            Ty::Raw(e) => format!("Raw<{}>", self.show(&e)),
            Ty::Ref(e) => format!("&{}", self.show(&e)),
            Ty::Lent(e) => self.show(&e),
            Ty::Struct(d, ts, _) => {
                let n = &self.d.s(d).name;
                if crate::records::is_record(n) {
                    let fields = self.d.s(d).fields.iter().zip(&ts);
                    let fs: Vec<String> =
                        (fields.map(|((f, _), t)| format!("{f}: {}", self.show(t)))).collect();
                    return format!("({})", fs.join(", "));
                }
                if ts.is_empty() {
                    n.clone()
                } else {
                    format!(
                        "{n}<{}>",
                        ts.iter()
                            .map(|t| self.show(t))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
            Ty::Fn(ps, r, _) => format!(
                "({}) -> {}",
                ps.iter()
                    .map(|t| self.show(t))
                    .collect::<Vec<_>>()
                    .join(", "),
                self.show(&r)
            ),
            Ty::Clo(_) => "a function".into(),
            Ty::Param(i) => self.generics.get(i).cloned().unwrap_or("?".into()),
            Ty::Dom(d) => self.domains[d].name.clone(),
            Ty::Var(_) => "_".into(),
            Ty::Error => "<error>".into(),
        }
    }
    /// The signature of a closure, in the caller's terms.
    pub(super) fn clo_sig(&mut self, c: &CloRef) -> (Vec<Ty>, Ty) {
        match c {
            CloRef::Item(f) => {
                let fd = self.d.f(f.decl);
                let sub = Subst {
                    tys: &f.targs,
                    sets: &f.sets,
                };
                let ps = fd.params.iter().map(|p| sub.ty(&p.ty)).collect();
                let r = self.ret_ty(f);
                (ps, r)
            }
            CloRef::Lambda(id, ..) => (
                self.lambdas[id.0].params.clone(),
                self.lambdas[id.0].ret.clone(),
            ),
        }
    }
    /// The result type of a call, with "whatever f returns" for a function-value result.
    pub(super) fn ret_ty(&mut self, f: &FnRef) -> Ty {
        let fd = self.d.f(f.decl);
        let sub = Subst {
            tys: &f.targs,
            sets: &f.sets,
        };
        // A call to the fn being checked uses its own result sets: recursion does not get a second copy.
        let own = self.ret_sets.clone();
        let me = self.decl;
        let t = map_slots(&sub.ty(&fd.ret), &|k| {
            if me == Some(f.decl) {
                own[&(f.decl, k)].clone()
            } else {
                SetTy::RetOf(Box::new(f.clone()), k)
            }
        });
        self.fresh_sets(&t)
    }
    /// `T` when `t` is `?T`.
    fn optional_id(&self) -> Option<StructId> {
        self.d.core_struct("Optional")
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
            let tmp = self.bind("$opt", v.ty.clone());
            let read = TExpr {
                kind: TK::Var(tmp),
                ty: v.ty.clone(),
                span,
            };
            (vec![TStmt::Let(tmp, v)], read)
        };
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
    /// A value flows into a slot of type `want`. The closed list of conversions lives here.
    /// Whether `a` and `b` could be one type, an unknown fitting anything; nothing is solved.
    pub(super) fn fits(&self, a: &Ty, b: &Ty) -> bool {
        match (self.shallow(a), self.shallow(b)) {
            (Ty::Var(_) | Ty::Error, _) | (_, Ty::Var(_) | Ty::Error) => true,
            (Ty::Struct(x, xs, _), Ty::Struct(y, ys, _)) => {
                x == y && xs.len() == ys.len() && xs.iter().zip(&ys).all(|(p, q)| self.fits(p, q))
            }
            (Ty::Raw(p), Ty::Raw(q)) | (Ty::Ref(p), Ty::Ref(q)) => self.fits(&p, &q),
            (Ty::Range(..) | Ty::RangeOf(_), Ty::Int)
            | (Ty::Int, Ty::Range(..) | Ty::RangeOf(_)) => true,
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
        (self.d.coercions.iter().copied()).find(|f| {
            let fd = self.d.f(*f);
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
            TK::Var(VarRef::Local(l)) => self.locals.get(*l),
            TK::Var(VarRef::Global(g)) => self.globals.get(*g),
            _ => None,
        }?;
        let pushes = stmts.iter().filter(|s| matches!(s, TStmt::Expr(_))).count();
        (local.name == "$list").then_some(pushes)
    }
    fn coerce_by(&mut self, decl: FnId, e: TExpr, want: &Ty) -> TExpr {
        let span = e.span;
        // A literal whose length differs from a constant count is wrong before it runs (R04.8.10).
        if let (Some(n), Ty::Struct(_, ts, _)) = (self.literal_len(&e), want) {
            let count = ts.iter().find_map(|t| match self.shallow(t) {
                Ty::Dom(i) => self.domains[i].count,
                _ => None,
            });
            if let Some(c) = count.filter(|c| *c != n as i64) {
                let w = self.show(want);
                self.err(
                    span,
                    format!(
                        "{}: a list of {n} elements where `{w}` has {c} cells",
                        cpl_diag::codes::TYPE_MISMATCH.0
                    ),
                );
            }
        }
        let f = self.fnref(decl, Vec::new(), Vec::new());
        let ty = self.ret_ty(&f);
        let param = {
            let sub = Subst {
                tys: &f.targs,
                sets: &f.sets,
            };
            sub.ty(&self.d.f(decl).params[0].ty)
        };
        let arg = self.coerce(e, &param);
        if !self.unify(&ty, want) {
            self.err(
                span,
                format!(
                    "{}: expected `{}`, found `{}`",
                    cpl_diag::codes::TYPE_MISMATCH.0,
                    self.show(want),
                    self.show(&ty)
                ),
            );
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
    pub(super) fn coerce(&mut self, e: TExpr, want: &Ty) -> TExpr {
        let (have, want_s) = (self.shallow(&e.ty), self.shallow(want));
        let span = e.span;
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
            // An integer literal is that position of an integer domain (R04.2.18).
            (Ty::Int, Ty::Dom(_)) if matches!(e.kind, TK::Int(_) | TK::Neg(_)) => {
                self.position(want_s.clone(), e)
            }
            // An integer literal in a `U64` slot is that `U64`: its bits (R02.1).
            (Ty::Int, Ty::U64) if matches!(e.kind, TK::Int(_)) => TExpr {
                kind: TK::ToU64(Box::new(e)),
                ty: Ty::U64,
                span,
            },
            // Loads widen, stores check (R03.1.4).
            (Ty::Range(..) | Ty::RangeOf(_), Ty::Int) => TExpr { ty: Ty::Int, ..e },
            (Ty::RangeOf(a), Ty::RangeOf(b)) if a == b => e,
            (Ty::Int | Ty::Range(..) | Ty::RangeOf(_), Ty::RangeOf(r)) => {
                let end = |v: Option<VarRef>| {
                    v.map(|v| TExpr {
                        kind: TK::Var(v),
                        ty: Ty::Int,
                        span,
                    })
                };
                let (lo, hi) = (end(self.ranges[*r].lo), end(self.ranges[*r].hi));
                let v = TExpr { ty: Ty::Int, ..e };
                let v = self.checked_between(v, lo, hi, span);
                TExpr { ty: want_s, ..v }
            }
            (Ty::Range(a, b), Ty::Range(lo, hi)) if lo <= a && b <= hi => TExpr { ty: want_s, ..e },
            (Ty::Int | Ty::Range(..) | Ty::RangeOf(_), Ty::Range(lo, hi)) => {
                let (lo, hi) = (*lo, *hi);
                // A constant is checked here, not when the program runs (R03.1.10).
                if let TK::Int(i) = e.kind {
                    if i < lo || i > hi {
                        self.err(
                            span,
                            format!(
                                "{}: {i} is outside `{lo}..={hi}`",
                                cpl_diag::codes::RANGE_STORE_CONST.0
                            ),
                        );
                    }
                    return TExpr { ty: want_s, ..e };
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
                let ok = cps.len() == ps.len()
                    && cps.iter().zip(ps).all(|(x, y)| self.unify(x, y))
                    && self.unify(&cr, r);
                if !ok {
                    self.err(
                        span,
                        format!(
                            "{}: this function does not fit `{}`",
                            cpl_diag::codes::TYPE_MISMATCH.0,
                            self.show(&want_s)
                        ),
                    );
                }
                match self.shallow_set(set) {
                    SetTy::Var(v) => {
                        if let SetVar::Root(ms) = &mut self.sets[v] {
                            ms.push(Member::Clo((**c).clone()));
                        }
                    }
                    _ => self.err(span, "slice: this function value came from a parameter or a call, so the functions it can hold are fixed there; a new one cannot be added here"),
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
                        format!(
                            "{}: expected `{}`, found `{}`",
                            cpl_diag::codes::TYPE_MISMATCH.0,
                            self.show(&want_s),
                            self.show(&have)
                        ),
                    );
                }
                let (a, b) = (self.shallow_set(s1), self.shallow_set(s2));
                match (&a, &b) {
                    _ if a == b => e,
                    (SetTy::Var(_), SetTy::Var(_)) if !self.joining => {
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
                    _ => {
                        self.err(
                            span,
                            "slice: these two function values have different fixed sets",
                        );
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
                            && (self.joining || !matches!(a, SetTy::Var(_)))
                            && matches!(b, SetTy::Var(_))
                    }) =>
            {
                for (x, y) in t1.iter().zip(t2) {
                    self.unify(x, y);
                }
                let fields = self.d.s(*d1).fields.clone();
                let (from, to) = (Subst { tys: t1, sets: s1 }, Subst { tys: t2, sets: s2 });
                self.scopes.push(HashMap::new());
                let tmp = self.bind("$widen", have.clone());
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
                self.scopes.pop();
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
                if !self.unify(&have, &want_s) {
                    self.err(
                        span,
                        format!(
                            "{}: expected `{}`, found `{}`",
                            cpl_diag::codes::TYPE_MISMATCH.0,
                            self.show(&want_s),
                            self.show(&have)
                        ),
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

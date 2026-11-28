//! Operators and fn items as values at a bound (R08.1.9): a bare operator is its `@operator`
//! trait's fn item, and an item passed where a struct with a `@from_op` fn is wanted, or at a
//! bound the item has no impl of but such a struct has, is that fn's call on the item.
use super::*;
use cpl_next_syntax::grammar::Decorator;

impl Cx<'_> {
    /// A bare operator (`+`) wanted as a value of an open type a bound constrains, or of a struct
    /// with a `@from_op` fn: its trait's fn item. Wanted as a fn, or by an open type nothing
    /// bounds (R08.2.11), it stays the lambda it is written as.
    pub(super) fn op_item(&mut self, op: BinOp, want: Option<&Ty>, span: Span) -> Option<TExpr> {
        let open = match want.map(|w| self.shallow(w)) {
            Some(Ty::Var(v)) => self.obliged(v),
            Some(Ty::Struct(s, ..)) => self.op_struct_fn(s).is_some(),
            _ => false,
        };
        let m = (self.syms.get(&op.to_string()))
            .and_then(|op| self.rows.binary_operator(op))
            .filter(|_| open)?;
        let f = self.fnref(m, Vec::new(), Vec::new());
        Some(TExpr {
            kind: TK::FnItem(f.clone()),
            ty: Ty::Clo(Box::new(CloRef::Item(f))),
            span,
        })
    }
    /// The body of trait fn `m` with none of its own: the call of `Self`'s impl of it.
    pub(super) fn trait_dispatch(&mut self, m: FnId, tr: TraitId, params: &[LocalId]) -> TExpr {
        let span = Span::new(0, 0);
        let var = |l, ty| TExpr {
            kind: TK::Var(VarRef::Local(l)),
            ty,
            span,
        };
        let fd = self.items.f(m);
        let recv = (self.b.self_local).map(|l| Box::new(TArg::Value(var(l, Ty::Param(0)))));
        let args = (params.iter().zip(&fd.params))
            .map(|(&l, p)| TArg::Value(var(l, p.ty.clone())))
            .collect();
        let n = self.items.t(tr).defaults.len();
        TExpr {
            kind: TK::CallTrait {
                ctx: self.trait_ctx(tr, &fd.name),
                tr: decls::TraitRef {
                    tr,
                    args: (1..=n).map(Ty::Param).collect(),
                },
                method: fd.name.clone(),
                self_ty: Box::new(Ty::Param(0)),
                targs: (n + 1..fd.generics.len()).map(Ty::Param).collect(),
                sets: Box::default(),
                recv,
                args,
            },
            ty: self.b.ret.clone(),
            span,
        }
    }
    fn op_struct_fn(&self, s: StructId) -> Option<FnId> {
        (self.marks.marked_fns(Decorator::FromOp).iter().copied())
            .find(|f| self.items.f(*f).owner == Some(decls::Owner::Struct(s)))
    }
    /// Whether some impl of `tr` is for `t`'s own type.
    pub(super) fn has_impl(&self, t: &Ty, tr: TraitId) -> bool {
        (self.impls().impls_of(tr))
            .any(|im| !im.blanket() && self.impls().fits_impl(im, t).is_some())
    }
    /// The traits open type `v` is bound by so far.
    fn bounds_on(&self, v: usize) -> Vec<TraitId> {
        (self.b.obligations.iter())
            .filter(|(t, ..)| self.shallow(t) == Ty::Var(v))
            .map(|(_, r, ..)| r.tr)
            .collect()
    }
    /// Whether `t` is a bare operator's item: the fn of an `@operator` trait.
    fn is_operator(&self, t: &Ty) -> bool {
        let Ty::Clo(c) = t else { return false };
        let CloRef::Item(f) = &**c else { return false };
        matches!(self.items.f(f.decl).owner, Some(decls::Owner::Trait(tr)) if self.rows.is_operator_trait(tr))
    }
    /// The `@from_op` fn whose struct has an impl of a bound on `v` that `item` has none of (any
    /// bound, for a bare operator, which always takes the struct's impl), and whose bounds on its
    /// operand `item` meets.
    fn op_bound_fn(&self, v: usize, item: &Ty) -> Option<FnId> {
        let op = self.is_operator(item);
        let missing: Vec<TraitId> = (self.bounds_on(v).into_iter())
            .filter(|tr| op || !self.has_impl(item, *tr))
            .collect();
        (self.marks.marked_fns(Decorator::FromOp).iter().copied()).find(|&m| {
            let fd = self.items.f(m);
            let Some(decls::Owner::Struct(s)) = fd.owner else {
                return false;
            };
            let meets = match fd.params.first().map(|p| &p.ty) {
                Some(Ty::Param(i)) => (fd.bounds.iter())
                    .filter(|(j, _)| j == i)
                    .all(|(_, r)| self.has_impl(item, r.tr)),
                _ => false,
            };
            meets
                && missing.iter().any(|tr| {
                    (self.impls().impls_of(*tr))
                        .any(|im| matches!(im.for_, Ty::Struct(d, ..) if d == s))
                })
        })
    }
    /// A fn value given for `v` with no impl of a bound on `v` for its own type is the fn type
    /// an impl of one targets, as a lambda written there is (`fn_impl_for`), and coerces to it
    /// (R08.1.10): a fn item, a generator's item (`fn_item_value`) or a closure held in a name.
    fn item_as_fn_impl(&mut self, e: &TExpr, want: &Ty) {
        let (Ty::Var(v), Ty::Clo(c)) = (self.shallow(want), self.shallow(&e.ty)) else {
            return;
        };
        if (self.bounds_on(v).iter()).any(|tr| self.has_impl(&e.ty, *tr)) {
            return;
        }
        let arity = self.clo_sig(&c).0.len();
        self.fn_impl_for(v, arity);
    }
    /// `e` as the `@from_op` fn's call when `want` asks for one (R08.1.9); otherwise `e`.
    pub(super) fn lift_item(&mut self, e: TExpr, want: &Ty) -> TExpr {
        let m = match (&e.kind, self.shallow(want)) {
            (TK::FnItem(_), Ty::Struct(s, ..)) => self.op_struct_fn(s),
            (TK::FnItem(_), Ty::Var(v)) => self.op_bound_fn(v, &e.ty),
            _ => None,
        };
        let Some(m) = m else {
            self.item_as_fn_impl(&e, want);
            return e;
        };
        let span = e.span;
        let f = self.fnref(m, Vec::new(), Vec::new());
        let ret = self.ret_ty(&f);
        let param = Subst {
            tys: &f.targs,
            sets: &f.sets,
        }
        .ty(&self.items.f(m).params[0].ty);
        self.unify(&ret, want);
        self.oblige(&f, span);
        let arg = self.coerce(e, &param);
        TExpr {
            kind: TK::Call {
                f,
                recv: None,
                args: vec![TArg::Value(arg)],
            },
            ty: ret,
            span,
        }
    }
}

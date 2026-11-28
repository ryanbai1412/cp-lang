//! Loops: `for` over a value, an iterator, a walk or a bound, and `yield from` (R06.3).

use super::*;
use cpl_next_decls::decls::Role;
use cpl_next_syntax::SyntaxToken;

impl<'a> Cx<'a> {
    /// `for pats in v`: a value walks by its `recipe`, a bound's walk, or its `each` (else
    /// `iter`); each item is bound by `pats` around `body`.
    fn walk_value(
        &mut self,
        mut call: TExpr,
        pats: &[next::Pattern],
        body: impl FnOnce(&mut Self, TExpr, &mut Vec<TStmt>),
        span: Span,
        out: &mut Vec<TStmt>,
    ) {
        if let Some(walk) = self.recipe_walk(&call) {
            let each = |cx: &mut Self, elem: Ty| cx.walk_each(pats, elem, body, span);
            return self.value_walk(walk, call, each, out);
        }
        // `for x in g.out(u)` through a bound on `g`'s type parameter; a bound's value
        // (`s.keys`) walks as the value it returns.
        let yields = |tr: &TraitRef, m: &str| {
            self.d
                .member(&self.items.t(tr.tr).methods, m)
                .is_some_and(|m| self.items.f(m).region.is_some())
        };
        call = match call.kind {
            TK::CallTrait {
                tr,
                method,
                recv: Some(recv),
                args,
                ..
            } if yields(&tr, &method) => {
                return self.trait_walk((tr, method, recv, args), pats, body, span, out);
            }
            kind => TExpr { kind, ..call },
        };
        if !matches!(&call.kind, TK::Call { f, .. } if self.items.f(f.decl).region.is_some()) {
            // A type without its own `each` walks through its `Seq` impl (R07.1.1).
            let rt = self.shallow(&call.ty);
            if let Ty::Param(p) = rt {
                return self.bound_walk(p, call, pats, body, span, out);
            }
            self.b.allow_region = true;
            let iteration = self.rows.role_trait(Role::Iteration);
            let own = (iteration.and_then(|tr| self.rows.own_walk(tr)))
                .filter(|&(n, _)| self.impls().lookup_member(&rt, n).is_some());
            let walk = (own.or_else(|| iteration.and_then(|tr| self.rows.walk(tr))))
                .map_or_else(String::new, |(n, _)| self.syms.text(n).to_string());
            call = self.method_call(call, &walk, vec![], span, false);
            self.b.allow_region = false;
        }
        let TK::Call { f, recv, args } = call.kind else {
            return;
        };
        let Some((false, elem)) = self.items.f(f.decl).region.clone() else {
            return self.err(
                span,
                Msg::verdict(
                    cpl_diag::codes::TYPE_MISMATCH,
                    "this cannot be walked with `for`",
                ),
            );
        };
        let elem = self.result_of(&f, &elem);
        let (var, body) = self.walk_each(pats, elem, body, span);
        out.push(TStmt::Region {
            callee: Callee::Fn(f),
            recv,
            args,
            var,
            lend: false,
            body,
        });
    }
    /// `for x in g.out(u)` with `out` a generator of a bound on `g`'s type parameter: the
    /// impl each instance picks runs the body.
    fn trait_walk(
        &mut self,
        (tr, method, recv, args): (TraitRef, String, Box<TArg>, Vec<TArg>),
        pats: &[next::Pattern],
        body: impl FnOnce(&mut Self, TExpr, &mut Vec<TStmt>),
        span: Span,
        out: &mut Vec<TStmt>,
    ) {
        let Some(m) = self.method_in(&self.items.t(tr.tr).methods, &method) else {
            return self.err(
                span,
                Msg::internal("a bound's walk names no fn of its trait"),
            );
        };
        let Some((false, elem)) = self.items.f(m).region.clone() else {
            return self.err(
                span,
                Msg::verdict(
                    cpl_diag::codes::TYPE_MISMATCH,
                    "this cannot be walked with `for`",
                ),
            );
        };
        let (TArg::Value(r) | TArg::Mut(r)) = recv.as_ref();
        let mut tys = vec![r.ty.clone()];
        tys.extend(tr.args.iter().cloned());
        let elem = Subst {
            tys: &tys,
            sets: &[],
        }
        .ty(&elem);
        let (var, body) = self.walk_each(pats, elem, body, span);
        out.push(TStmt::Region {
            callee: Callee::Trait(tr, method),
            recv: Some(recv),
            args,
            var,
            lend: false,
            body,
        });
    }
    /// `for x in s` with `s` of a type parameter: its bound's `iter` walks it, and each
    /// instance picks the impl (R07.1.1).
    fn bound_walk(
        &mut self,
        p: usize,
        recv: TExpr,
        pats: &[next::Pattern],
        body: impl FnOnce(&mut Self, TExpr, &mut Vec<TStmt>),
        span: Span,
        out: &mut Vec<TStmt>,
    ) {
        let walk = (self.rows.role_trait(Role::Iteration))
            .and_then(|tr| self.rows.walk(tr))
            .map(|(n, m)| (self.syms.text(n).to_string(), m));
        let found = (self.bound_traits(p).into_iter())
            .find(|tr| Some(tr.tr) == self.rows.role_trait(Role::Iteration))
            .zip(walk);
        let Some((tr, (walk, m))) = found else {
            let shown = self.show(&recv.ty);
            return self.err(
                span,
                Msg::verdict(
                    cpl_diag::codes::UNKNOWN_FIELD,
                    format!("`{shown}` is not bound by the iteration role's trait (R11.6.1)"),
                ),
            );
        };
        let Some((false, elem)) = self.items.f(m).region.clone() else {
            return self.err(
                span,
                Msg::verdict(
                    cpl_diag::codes::TYPE_MISMATCH,
                    "this cannot be walked with `for`",
                ),
            );
        };
        self.reach(m);
        let mut tys = vec![recv.ty.clone()];
        tys.extend(tr.args.iter().cloned());
        let elem = Subst {
            tys: &tys,
            sets: &[],
        }
        .ty(&elem);
        let (var, body) = self.walk_each(pats, elem, body, span);
        out.push(TStmt::Region {
            callee: Callee::Trait(tr.clone(), walk),
            recv: Some(Box::new(TArg::Value(recv))),
            args: Vec::new(),
            var,
            lend: false,
            body,
        });
    }
    /// One walk's body over items of type `elem`: a scope binding the item by `pats`, then
    /// `body`. Gives the item's variable and the checked body.
    pub(super) fn walk_each(
        &mut self,
        pats: &[next::Pattern],
        elem: Ty,
        body: impl FnOnce(&mut Self, TExpr, &mut Vec<TStmt>),
        span: Span,
    ) -> (VarRef, TExpr) {
        self.b.scopes.push(HashMap::new());
        let (var, open) = self.bind_each(pats, elem.clone(), span);
        let item = TExpr {
            kind: TK::Var(var),
            ty: elem,
            span,
        };
        let b = self.scoped_block(span, |cx, out| {
            out.extend(open);
            body(cx, item, out);
            None
        });
        self.b.scopes.pop();
        (var, b)
    }
    /// The variable a walk binds each item to, and what opens a destructured one: a name
    /// binds the item, `_` a slot nothing reads, and a tuple or record pattern, or several
    /// patterns (a table's `r, c`), a slot whose parts the body opens with.
    fn bind_each(&mut self, pats: &[next::Pattern], elem: Ty, span: Span) -> (VarRef, Vec<TStmt>) {
        if let [next::Pattern::Path(p)] = pats {
            if let Some(n) = super::pattern::bind_name(p) {
                let var = self.bind(n.text(), elem);
                return (var, Vec::new());
            }
        }
        if let [] | [next::Pattern::Wildcard(_)] = pats {
            return (self.temp("$each", elem), Vec::new());
        }
        let var = self.temp("$item", elem.clone());
        let item = TExpr {
            kind: TK::Var(var),
            ty: elem.clone(),
            span,
        };
        let mut open = Vec::new();
        match pats {
            [p] => self.let_pattern(p, false, item, &mut open),
            ps => {
                let tys = self.field_tys(&elem).map(|(_, t)| t).unwrap_or_default();
                if tys.len() != ps.len() && !self.mentions_error(&elem) {
                    let (k, n, t) = (ps.len(), tys.len(), self.show(&elem));
                    let what = format!("these {k} patterns name the parts of `{t}`, which has {n}");
                    self.err(span, Msg::verdict(cpl_diag::codes::TYPE_MISMATCH, what));
                }
                for (k, p) in ps.iter().enumerate() {
                    let ty = tys.get(k).cloned().unwrap_or(Ty::Error);
                    let kind = TK::Field(Box::new(item.clone()), k);
                    self.let_pattern(p, false, TExpr { kind, ty, span }, &mut open);
                }
            }
        }
        (var, open)
    }
    /// `for pats in iter: body` over a value already checked (a table's `r, c`, a
    /// comprehension's clause).
    pub(super) fn for_over(
        &mut self,
        pats: &[next::Pattern],
        iter: TExpr,
        body: impl FnOnce(&mut Self, &mut Vec<TStmt>),
        at: Span,
        out: &mut Vec<TStmt>,
    ) {
        self.walk_value(iter, pats, |cx, _, out| body(cx, out), at, out)
    }
    /// `for pat in iter: body`: a declared domain's ordinals, a closed range of `Int`s, or a
    /// value walked through its `Seq` (R06.3).
    pub(super) fn for_loop(
        &mut self,
        pat: &next::Pattern,
        iter: &next::Expr,
        body: impl FnOnce(&mut Self, &mut Vec<TStmt>),
        at: Span,
        out: &mut Vec<TStmt>,
    ) {
        let iter_at = self.at(iter);
        let it = super::index::unparen(iter.clone());
        // A bare `..` names no domain to walk (R04.4.11).
        if let next::Expr::Range(r) = &it {
            if r.lo().is_none() && r.hi().is_none() && r.step().is_none() {
                let text = cpl_diag::msgs::check::BARE_MEANS_ALL_DIMENSION.say(&[]);
                let help = "name what to walk: `for x in D` or `for i in 0..n`".to_string();
                let msg = Msg::verdict(cpl_diag::codes::EXPECTED_EXPR, text);
                return self.err_help(iter_at, msg, Some(help));
            }
        }
        // A name, or `_`: an item nothing reads.
        let bound: Option<Option<SyntaxToken>> = match pat {
            next::Pattern::Path(p) => super::pattern::bind_name(p).map(Some),
            next::Pattern::Wildcard(_) => Some(None),
            _ => None,
        };
        let Some(name) = bound else {
            return self.walk_iter(pat, &it, body, at, out);
        };
        let counter = |cx: &mut Self, ty: Ty| match &name {
            Some(n) => cx.bind(n.text(), ty),
            None => cx.temp("$each", ty),
        };
        // A declared domain's positions are its ordinals; any other walks its `Seq`.
        if let next::Expr::Name(dn) = &it {
            let ordinals = (dn.ident())
                .and_then(|t| self.domain_named(t.text()))
                .filter(|d| matches!(d, Ty::Dom(_)));
            if let Some(dty) = ordinals {
                let lo = TExpr {
                    kind: TK::Int(0),
                    ty: Ty::Int,
                    span: at,
                };
                let hi = TExpr {
                    kind: TK::DomLen(dty.clone()),
                    ty: Ty::Int,
                    span: at,
                };
                self.b.scopes.push(HashMap::new());
                let var = counter(self, dty);
                let b = self.scoped_block(at, |cx, out| {
                    body(cx, out);
                    None
                });
                self.b.scopes.pop();
                return out.push(TStmt::For(var, lo, hi, Box::new(b)));
            }
        }
        let next::Expr::Range(r) = &it else {
            return self.walk_iter(pat, &it, body, at, out);
        };
        let (Some(lo), Some(hi), None) = (r.lo(), r.hi(), r.step()) else {
            return self.walk_iter(pat, &it, body, at, out);
        };
        let inclusive = r.is_inclusive();
        let first = self.expr(&lo, Some(&Ty::Int));
        if self.ord_ends(&first.ty, None).is_some() {
            let r = self.range_from(first, &hi, inclusive, None, None, iter_at);
            return self.walk_value(
                r,
                std::slice::from_ref(pat),
                |cx, _, out| body(cx, out),
                at,
                out,
            );
        }
        let lo = self.coerce(first, &Ty::Int);
        let hi_at = self.at(&hi);
        let hi = self.expr(&hi, Some(&Ty::Int));
        let hi = self.coerce(hi, &Ty::Int);
        // `lo..=hi` is `lo..hi+1` here, so it fails at the largest Int where the language does not.
        let hi = match inclusive {
            true => {
                let one = TExpr {
                    kind: TK::Int(1),
                    ty: Ty::Int,
                    span: hi_at,
                };
                self.binary(BinOp::Add, hi, one, hi_at)
            }
            false => hi,
        };
        self.b.scopes.push(HashMap::new());
        let var = counter(self, Ty::Int);
        let b = self.scoped_block(at, |cx, out| {
            body(cx, out);
            None
        });
        self.b.scopes.pop();
        out.push(TStmt::For(var, lo, hi, Box::new(b)));
    }
    /// `for pat in xs` through the type's own walk: `gen(args)`, `xs.getter`, or `xs`'s `Seq`.
    fn walk_iter(
        &mut self,
        pat: &next::Pattern,
        iter: &next::Expr,
        body: impl FnOnce(&mut Self, &mut Vec<TStmt>),
        at: Span,
        out: &mut Vec<TStmt>,
    ) {
        self.b.allow_region = matches!(iter, next::Expr::Call(_) | next::Expr::Field(_));
        let call = self.expr(iter, None);
        self.b.allow_region = false;
        self.walk_value(
            call,
            std::slice::from_ref(pat),
            |cx, _, out| body(cx, out),
            at,
            out,
        )
    }
    /// `yield from xs` is `for x in xs: yield x`, over any `Seq` (R06.3.6).
    pub(super) fn yield_all(&mut self, seq: &next::Expr, span: Span, out: &mut Vec<TStmt>) {
        let Some((false, t)) = self.b.region.clone() else {
            let code = cpl_diag::codes::TYPE_MISMATCH;
            return self.err(
                span,
                Msg::verdict(code, "`yield` outside a fn that `yields`"),
            );
        };
        let it = super::index::unparen(seq.clone());
        self.b.allow_region = matches!(it, next::Expr::Call(_) | next::Expr::Field(_));
        let call = self.expr(&it, None);
        self.b.allow_region = false;
        self.b.loops += 1;
        let each = move |cx: &mut Self, item: TExpr, out: &mut Vec<TStmt>| {
            let v = cx.coerce(item, &t);
            out.push(TStmt::Yield(v))
        };
        self.walk_value(call, &[], each, span, out);
        self.b.loops -= 1;
    }
}

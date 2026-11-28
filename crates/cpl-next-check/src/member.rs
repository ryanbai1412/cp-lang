//! Members: a field read or a method call on a receiver, found on its type, a bound or an
//! extension fn (R08.1.4).
use super::call::CallArg;
use super::expr::block;
use super::*;
use cpl_next_decls::decls::Role;
use cpl_next_ir::unbuilt::Unbuilt;

/// Whether a call with these arguments could be of `f`: `D.count(pred)` is not a call of the
/// getter `Domain.count`, which takes none, so a member that takes them is looked for instead.
/// A member's name and where it is written: `m` of `x.m`, or one a form calls for the program.
#[derive(Clone, Copy)]
pub(super) struct Called<'n> {
    pub text: &'n str,
    /// Its id, when a declaration has the name: what members are looked up by.
    pub sym: Option<Sym>,
    pub span: Span,
}

impl<'n> Called<'n> {
    /// The member name `text` written at `span`.
    pub fn written(syms: &Syms, text: &'n str, span: Span) -> Self {
        Called {
            text,
            sym: syms.get(text),
            span,
        }
    }
}

/// The fn of `members` that `name` names.
fn member_of(members: &HashMap<Sym, FnId>, name: Called<'_>) -> Option<FnId> {
    members.get(&name.sym?).copied()
}

/// The name an expression is, when it is only a name: `x`, not `x.y` or `f(x)`.
/// What `f.clear` of a `@memo` fn calls: a clear bound beside a local or global `f`, or the
/// fn decls declared beside a declared one.
enum Clear {
    Bound(VarRef),
    Declared(FnId),
}

pub(super) fn written_types(g: &next::GenericExpr) -> Vec<next::Type> {
    (g.args().iter().flat_map(|a| a.args()))
        .filter_map(|a| match a {
            next::GenericArg::Type(t) => Some(t),
            next::GenericArg::Param(_) => None,
        })
        .collect()
}

pub(super) fn bare_name(e: &next::Expr) -> Option<String> {
    match e {
        next::Expr::Name(r) => Some(r.ident()?.text().to_string()),
        _ => None,
    }
}

pub(super) fn takes(f: &decls::FnDecl, call: Option<&[CallArg]>) -> bool {
    call.is_none_or(|args| args.len() <= f.params.len())
}

impl<'a> Cx<'a> {
    pub(super) fn method_call(
        &mut self,
        recv: TExpr,
        name: &str,
        args: Vec<TArg>,
        span: Span,
        mutates: bool,
    ) -> TExpr {
        let rt = self.recv_ty(&recv.ty);
        if rt == Ty::Error {
            return Self::error_node(span);
        }
        let found = self.member_named(&rt, self.syms.get(name));
        let at = self.b.module;
        let hidden = found
            .as_ref()
            .and_then(|x| decls::hidden_member(self.items, self.names, x, &rt, name, at));
        if let Some((msg, help)) = hidden {
            self.err_help(span, msg, Some(help));
            return Self::error_node(span);
        }
        let Some(decls::Member::Method(m, ts, ss)) = found else {
            self.err(
                span,
                Msg::verdict(
                    cpl_diag::codes::UNKNOWN_FIELD,
                    format!("`{}` has no `{name}`", self.show(&rt)),
                ),
            );
            return TExpr {
                kind: TK::Error,
                ty: Ty::Error,
                span,
            };
        };
        self.call_member(recv, (m, ts, ss), args, span, mutates)
    }
    /// `rt`'s member named `name`; none when nothing anywhere declares that name.
    pub(super) fn member_named(&self, rt: &Ty, name: Option<Sym>) -> Option<decls::Member> {
        self.impls().lookup_member(rt, name?)
    }
    /// A receiver's type with its arguments resolved, so an impl's other parameters follow from
    /// them (`K` of `Slice<S>` from `S`'s impl of `Indexed<K, T>`).
    pub(super) fn recv_ty(&self, t: &Ty) -> Ty {
        match self.shallow(t) {
            Ty::Struct(id, args, sets) => {
                Ty::Struct(id, args.iter().map(|a| self.shallow(a)).collect(), sets)
            }
            t => t,
        }
    }
    /// `recv.m(args)` with `m` already found on `recv`'s type.
    pub(super) fn call_member(
        &mut self,
        recv: TExpr,
        (m, ts, ss): (FnId, Vec<Ty>, Vec<SetTy>),
        args: Vec<TArg>,
        span: Span,
        mutates: bool,
    ) -> TExpr {
        let fd = self.items.f(m);
        let f = self.fnref(m, ts, ss);
        self.oblige(&f, span);
        self.solve(false);
        let sub = Subst {
            tys: &f.targs,
            sets: &f.sets,
        };
        let ptys: Vec<Ty> = fd.params.iter().map(|p| sub.ty(&p.ty)).collect();
        let args: Vec<TArg> = args
            .into_iter()
            .zip(&ptys)
            .map(|(a, pty)| match a {
                TArg::Value(v) => TArg::Value(self.coerce(v, pty)),
                m => m,
            })
            .collect();
        let recv = if mutates || fd.self_mode == SelfMode::Mut {
            self.mut_recv(recv, span)
        } else if self.facts.writes_cache(m) {
            TArg::Mut(recv)
        } else {
            TArg::Value(recv)
        };
        let ty = self.ret_ty(&f);
        TExpr {
            kind: TK::Call {
                f,
                recv: Some(Box::new(recv)),
                args,
            },
            ty,
            span,
        }
    }
    /// A name two `use` lines import as different fns, neither more specific (R08.1.4).
    pub(super) fn two_imports(&mut self, n: &str, span: Span) {
        let code = cpl_diag::codes::AMBIGUOUS_METHOD;
        self.err_help(
            span,
            Msg::verdict(
                code,
                format!("two `use` lines import `{n}`, and neither is more specific"),
            ),
            Some(format!("keep one `use` of `{n}`, or call it by its module")),
        );
    }
    /// `x.f(a)` where `f` is a fn without `self` in scope: the call it means is `f(x, a)`
    /// (R06.1.2).
    /// The one trait that declares `name` and extends a trait `rt` has while `rt` lacks it, as
    /// `Ordered` extends `Domain` with `.next` (R04.2.31): naming it says what to bound or impl.
    fn lacked_trait_help(&self, rt: &Ty, name: Called<'_>) -> Option<String> {
        let sym = name.sym?;
        let extends = |a: TraitId, b: TraitId| {
            let mut todo = vec![a];
            let mut seen = HashSet::new();
            while let Some(t) = todo.pop() {
                if t == b {
                    return true;
                }
                if seen.insert(t.0) {
                    todo.extend(self.items.t(t).supers.iter().map(|s| s.tr));
                }
            }
            false
        };
        let has = |tr: TraitId| match rt {
            Ty::Param(p) => self.bound_traits(*p).iter().any(|b| extends(b.tr, tr)),
            t => !matches!(t, Ty::Var(_) | Ty::Error) && self.impls().has_impl(t, tr),
        };
        let mut lacked = (self.methods.traits_declaring(sym).into_iter())
            .filter(|&tr| !has(tr) && self.items.t(tr).supers.iter().any(|s| has(s.tr)));
        let tr = lacked.next().filter(|_| lacked.next().is_none())?;
        let (tn, shown) = (&self.items.t(tr).name, self.show(rt));
        Some(match rt {
            Ty::Param(_) => format!("`{}` is `{tn}`'s: bound `{shown}: {tn}`", name.text),
            _ => format!(
                "`{}` is `{tn}`'s, and `{shown}` does not implement `{tn}`",
                name.text
            ),
        })
    }
    fn free_fn_help(
        &self,
        base: Span,
        name: Called<'_>,
        call: Option<&[CallArg]>,
    ) -> Option<String> {
        let args = call?;
        let Some(decls::Named::Fn(f)) = self.named(self.b.module, name.text) else {
            return None;
        };
        if self.items.f(f).self_ty.is_some() {
            return None;
        }
        let mut written = vec![self.text(base).to_string()];
        written.extend(args.iter().map(|a| self.text(a.at).to_string()));
        let m = &name.text;
        Some(format!(
            "`{m}` is a free fn, not a method: `{m}({})`",
            written.join(", ")
        ))
    }
    /// `a.pow(e)` where `pow` is the fn of an `@operator` trait whose `@aka` names it: the
    /// operator is its one spelling, so the name is E0203 with the operator as the fix
    /// (R12.13.2, R01.11.1).
    fn operator_by_name(
        &self,
        found: &decls::Member,
        base: Span,
        name: Called<'_>,
        call: Option<&[CallArg]>,
    ) -> Option<(Msg, String)> {
        let decls::Member::Method(m, ..) = found else {
            return None;
        };
        let tr = match self.items.f(*m).owner? {
            decls::Owner::Impl(i) => self.items.imp(i).trait_.tr,
            decls::Owner::Trait(tr) => tr,
            decls::Owner::Struct(_) => return None,
        };
        let op = self.syms.text(self.rows.operator_of(tr)?);
        (self.rows.operator_method(tr)?.0 == name.sym?).then_some(())?;
        (self.items.t(tr).aka.iter()).find(|a| *a == name.text)?;
        let recv = self.text(base);
        let fix = match call.unwrap_or(&[]) {
            [] => format!("{op}{recv}"),
            [a] => format!("{recv} {op} {}", self.text(a.at)),
            _ => return None,
        };
        let text = format!("`{}` is what `{op}` calls: write the operator", name.text);
        let msg = Msg::verdict(cpl_diag::codes::UNKNOWN_FIELD, text);
        Some((msg, format!("write `{fix}`")))
    }
    /// `x.m(..)` where a fn the receiver has declares `@aka(m)` or `@was(m)`: the call it means.
    /// The name `@was` gives back is said in an E0503 (R06.4.11).
    fn renamed(
        &self,
        rt: &Ty,
        base: Span,
        name: Called<'_>,
        call: Option<&[CallArg]>,
    ) -> Option<(Option<String>, String)> {
        let m = &name.text;
        let f = (self.syms.get(m))
            .map_or_else(Vec::new, |s| self.methods.renamed_from(s))
            .into_iter()
            .find_map(|id| {
                let f = self.items.f(id);
                (self.member_named(rt, self.syms.get(&f.name)).is_some()
                    || decls::self_fits(self.items, id, rt))
                .then_some(f)
            })?;
        let now = &f.name;
        // The call as written with the member's name replaced: `x.m(a)` reads `x.now(a)`.
        let mut written = self.text(Span::new(base.start, name.span.start)).to_owned() + now;
        if let Some(args) = call {
            let args: Vec<&str> = args.iter().map(|a| self.text(a.at)).collect();
            written = format!("{written}({})", args.join(", "));
        }
        match f.was.iter().any(|w| w == m) {
            true => Some((Some(now.clone()), format!("write `{written}`"))),
            false => Some((None, format!("`{m}` is spelled `{now}`: `{written}`"))),
        }
    }
    /// A fn this module declares with a `self` of the receiver's own type is never called when
    /// the type has a member of its name: the member is found first (R08.1.4).
    fn hidden_by_member(&mut self, rt: &Ty, base: Span, name: Called<'_>) {
        let Ty::Struct(s, ..) = rt else {
            return;
        };
        let m = &name.text;
        let mine = match self.named(self.b.module, m) {
            Some(decls::Named::Fn(f)) => {
                let fd = self.items.f(f);
                fd.module == self.b.module
                    && matches!(&fd.self_ty, Some(Ty::Struct(t, ..)) if t == s)
            }
            _ => false,
        };
        if !mine {
            return;
        }
        let inner = self.text(base);
        // `(0..3).sum` keeps the parentheses a member call on it needs; `sum(0..3)` needs none.
        let shown = match postfix_safe(inner) {
            true => inner.to_string(),
            false => format!("({inner})"),
        };
        let ty = &self.items.s(*s).name;
        self.err_help(
            name.span,
            Msg::verdict(cpl_diag::codes::AMBIGUOUS_METHOD, format!("`{shown}.{m}` never calls the `fn {m}` that takes it: `{ty}.{m}` is the type's own member, and members are found before extension fns")),
            Some(format!("rename the fn, or call it as a plain fn: `{m}({inner})`")),
        );
    }
    /// `T.zero` or `T.f(args)` on a type parameter: a fn without `self` of one of its bounds,
    /// chosen per instance by `T`.
    fn bound_static(
        &mut self,
        base: &next::Expr,
        name: Called<'_>,
        call: Option<&[CallArg]>,
        span: Span,
    ) -> Option<TExpr> {
        let Some(n) = &bare_name(base) else {
            return None;
        };
        if self.lookup(n).is_some() {
            return None;
        }
        let g = self.b.syms.of(self.syms, n)?;
        let i = self.b.generics.iter().position(|x| *x == g)?;
        self.type_static(Ty::Param(i), name, call, span)
    }
    /// The bound of type parameter `i` whose trait gives the fn `name` that `pick` takes:
    /// two written traits are no choice (R08.1.4), and one trait at several arguments is left open
    /// for the bound it pushes, `T: Tr<..>`, to fix.
    pub(super) fn bound_giving(
        &mut self,
        i: usize,
        name: Called<'_>,
        pick: impl Fn(&decls::FnDecl) -> bool,
        span: Span,
    ) -> Result<Option<(decls::TraitRef, FnId)>, (TraitId, TraitId)> {
        let gives: Vec<(decls::TraitRef, FnId)> = (self.bound_traits(i).into_iter())
            .filter_map(|tr| {
                let m = member_of(&self.items.t(tr.tr).methods, name)?;
                pick(self.items.f(m)).then_some((tr, m))
            })
            .collect();
        // A bound written beats the `Domain` a domain parameter meets unwritten.
        let written = self.bounds_of(i, false);
        let gives: Vec<(decls::TraitRef, FnId)> = match gives.iter().any(|g| written.contains(&g.0))
        {
            true => gives
                .into_iter()
                .filter(|g| written.contains(&g.0))
                .collect(),
            false => gives,
        };
        let Some((first, m)) = gives.first().cloned() else {
            return Ok(None);
        };
        if let Some((other, _)) = gives.iter().find(|(r, _)| r.tr != first.tr) {
            return Err((first.tr, other.tr));
        }
        if gives.len() == 1 {
            return Ok(Some((first, m)));
        }
        let r = decls::TraitRef {
            tr: first.tr,
            args: first.args.iter().map(|_| self.new_ty()).collect(),
        };
        self.b
            .obligations
            .push((Ty::Param(i), r.clone(), self.b.item.at(span), Origin::Bound));
        Ok(Some((r, m)))
    }
    /// `shown.m`, which traits `a` and `b` both give `ty`: the call names its trait,
    /// `a.m(args)` (R08.1.4).
    pub(super) fn two_traits_give(
        &mut self,
        (a, b): (TraitId, TraitId),
        shown: &str,
        ty: &Ty,
        name: Called<'_>,
        args: &str,
        span: Span,
    ) -> TExpr {
        let (a, b) = (&self.items.t(a).name, &self.items.t(b).name);
        let (ty, m) = (self.show(ty), &name.text);
        let text = cpl_diag::msgs::check::AMBIGUOUS_METHOD_IS_AMBIGUOUS_BOTH_AND
            .say(&[&shown, m, a, b, &ty, m]);
        let help = format!("name the trait: `{a}.{m}({args})`");
        self.err_help(
            name.span,
            Msg::verdict(cpl_diag::codes::AMBIGUOUS_METHOD, text),
            Some(help),
        );
        Self::error_node(span)
    }
    /// `Trait.m(x, ..)`: the `m` of `x`'s impl of the trait named, as when two traits give `x`
    /// an `m` (R08.1.4).
    fn trait_call(
        &mut self,
        base: &next::Expr,
        name: Called<'_>,
        call: Option<&[CallArg]>,
        span: Span,
        want: Option<&Ty>,
    ) -> Option<TExpr> {
        let (Some(n), Some([first, rest @ ..])) = (&bare_name(base), call) else {
            return None;
        };
        if first.label.is_some() || self.lookup(n).is_some() {
            return None;
        }
        let Some(decls::Named::Trait(tr)) = self.named(self.b.module, n) else {
            return None;
        };
        let recv = self.arg_expr(first, None);
        let rt = self.shallow(&recv.ty);
        let Some(decls::Member::Method(m, ts, ss)) = self
            .syms
            .get(name.text)
            .and_then(|sym| self.impls().impl_member(&rt, sym, Some(tr)))
        else {
            if !self.is_error(&rt) {
                let ty = self.show(&rt);
                let text = format!(
                    "`{ty}` has no impl of `{}` with a `{}`",
                    self.items.t(tr).name,
                    name.text
                );
                self.err(span, Msg::verdict(cpl_diag::codes::MISSING_IMPL, text));
            }
            return Some(Self::error_node(span));
        };
        let f = self.fnref(m, ts, ss);
        let recv = match first.mutable {
            true => self.mut_recv(recv, span),
            false => self.recv_arg(m, recv, span),
        };
        Some(self.call_method(f, recv, false, Some(rest), span, want))
    }
    /// `T.MIN` where no bound of `T` gives `MIN` but a trait does: a body is checked against its
    /// bounds alone (R08.2.2), so the error names the bound to write. None when no trait has a
    /// self-less fn `name`.
    fn unbound_static(&mut self, t: &str, name: Called<'_>, span: Span) -> Option<TExpr> {
        let mut givers: Vec<&str> = (name
            .sym
            .map_or_else(Vec::new, |s| self.methods.traits_declaring(s))
            .into_iter())
        .map(|tr| self.items.t(tr))
        .filter(|tr| !tr.alias)
        .filter(|tr| {
            (member_of(&tr.methods, name))
                .is_some_and(|m| self.items.f(m).self_mode == SelfMode::None)
        })
        .map(|tr| tr.name.as_str())
        .collect();
        givers.sort_unstable();
        givers.dedup();
        let shown = givers
            .iter()
            .map(|g| format!("`{t}: {g}`"))
            .collect::<Vec<_>>();
        if shown.is_empty() {
            return None;
        }
        let m = &name.text;
        self.err_help(
            span,
            Msg::verdict(
                cpl_diag::codes::MISSING_IMPL,
                format!("`{t}.{m}` needs a bound on `{t}` that gives `{m}`, and `{t}` has none (R08.2.2)"),
            ),
            Some(format!("write {}", shown.join(" or "))),
        );
        Some(Self::error_node(span))
    }
    /// `Int.zero`, or `F.zero` in a default where `F` names the call's type: a self-less fn of a
    /// trait the named type implements. None for a name that is a value, a struct or no type.
    fn named_type_static(
        &mut self,
        base: &next::Expr,
        name: Called<'_>,
        call: Option<&[CallArg]>,
        span: Span,
    ) -> Option<TExpr> {
        let Some(n) = &bare_name(base) else {
            return None;
        };
        if self.lookup(n).is_some()
            || !matches!(
                self.named(self.b.module, n),
                None | Some(decls::Named::Alias(_))
            )
        {
            return None;
        }
        let t = match self.named(self.b.module, n) {
            _ if (self.b.syms.of(self.syms, n)).is_some_and(|g| self.b.generics.contains(&g)) => {
                return None
            }
            Some(decls::Named::Alias(id)) => {
                let (t, errs) = self.with_body_scope(|at| decls::alias_type(at, id, Vec::new()));
                if !errs.is_empty() {
                    return None;
                }
                t
            }
            Some(_) => return None,
            None => (self
                .b
                .syms
                .of(self.syms, n)
                .and_then(|s| self.b.doms.get(&s).cloned()))
            .or_else(|| {
                self.b
                    .syms
                    .of(self.syms, n)
                    .and_then(|s| self.owned_named(s))
            })
            .or_else(|| decls::prim_type(n))?,
        };
        let t = self.shallow(&t);
        if matches!(t, Ty::Error | Ty::Param(_) | Ty::Var(_)) {
            return None;
        }
        // `type P = Pair<Int>` then `P.new(..)`: the struct's own fn before a trait's.
        if let (Ty::Struct(d, given, _), Some(args)) = (&t, call) {
            if let Some(m) = member_of(&self.items.s(*d).methods, name) {
                let f = self.fnref(m, given.clone(), Vec::new());
                return Some(self.call_fn(f, None, args, span, None));
            }
        }
        self.type_static(t, name, call, span)
    }
    /// `D.find(p)` on a domain struct with no fn `find`: its walk of positions answers (R04.2.20).
    fn walks_for(&self, d: StructId, name: Called<'_>) -> bool {
        let s = self.items.s(d);
        member_of(&s.methods, name).is_none()
            && s.generics.is_empty()
            && self.is_domain(&Ty::Struct(d, Vec::new(), Vec::new()))
    }
    /// `Pair<V>.count(p)`: a member a generic domain type lacks is its walk's, as `Two.count(p)`
    /// is for one with no type parameters (R04.2.27).
    fn walks_at(&mut self, d: StructId, written: &[next::Type], name: Called<'_>) -> bool {
        if written.is_empty() || member_of(&self.items.s(d).methods, name).is_some() {
            return false;
        }
        let mut pre = Vec::new();
        let given = (written.iter())
            .map(|t| self.body_ty(t, &mut pre))
            .collect::<Vec<_>>();
        let given = self.shapes.with_defaults_of(d, &given);
        self.is_domain(&Ty::Struct(d, given, Vec::new()))
    }
    /// `T.f(args)`, as `V.cuts.count`: a fn without `self` of a trait `T` implements, or of a
    /// bound when `T` is a parameter, as a made fn's `A.count` is.
    pub(super) fn type_static(
        &mut self,
        t: Ty,
        name: Called<'_>,
        call: Option<&[CallArg]>,
        span: Span,
    ) -> Option<TExpr> {
        // A parameter's fn is its bound's, chosen per instance, as a made fn's `A.count` is.
        if let Ty::Param(i) = t {
            let n = self.show(&t);
            let pick = |fd: &decls::FnDecl| fd.self_mode == SelfMode::None && takes(fd, call);
            let found = match self.bound_giving(i, name, pick, span) {
                Ok(found) => found,
                Err(two) => {
                    return Some(self.two_traits_give(two, &n, &Ty::Param(i), name, "", span))
                }
            };
            let Some((tr, m)) = found else {
                if let Some(help) = self.lacked_trait_help(&Ty::Param(i), name) {
                    let text = format!("`{n}` has no member `{}`", name.text);
                    let msg = Msg::verdict(cpl_diag::codes::UNKNOWN_FIELD, text);
                    self.err_help(span, msg, Some(help));
                    return Some(Self::error_node(span));
                }
                let bound_free = self.bound_traits(i).is_empty();
                return bound_free
                    .then(|| self.unbound_static(&n, name, span))
                    .flatten();
            };
            return Some(self.static_call(Ty::Param(i), tr, m, name, call, span));
        }
        let found: Vec<(TraitId, FnId)> = (name
            .sym
            .map_or_else(Vec::new, |s| self.methods.traits_declaring(s))
            .into_iter())
        .filter_map(|tr| {
            let td = self.items.t(tr);
            let m = member_of(&td.methods, name)?;
            let fd = self.items.f(m);
            (fd.self_mode == SelfMode::None
                && takes(fd, call)
                && td.defaults.is_empty()
                && self.impls().has_impl(&t, tr))
            .then_some((tr, m))
        })
        .collect();
        // An impl for a written type beats a blanket one of another trait (R08.1.4): `Cut<D>`'s
        // `Domain.count` over `Positions`' `impl<D: Domain> .. for D`.
        let rank = |tr: TraitId| {
            let any = decls::TraitRef {
                tr,
                args: Vec::new(),
            };
            (self.impls().best(&t, &any).first()).map_or(0, |(k, _)| {
                decls::impl_rank(self.items, self.impls().imp(*k))
            })
        };
        let best = found.iter().map(|&(tr, _)| rank(tr)).min();
        let found: Vec<(TraitId, FnId)> = found
            .into_iter()
            .filter(|&(tr, _)| Some(rank(tr)) == best)
            .collect();
        let (tr, m) = match found[..] {
            [] => return None,
            [one] => one,
            [(a, _), (b, _), ..] => {
                let (a, b) = (&self.items.t(a).name, &self.items.t(b).name);
                let msg = format!(
                    "`{}` has `{}` from `{a}` and from `{b}`: say which, as `{a}.{}` (R08.1.4)",
                    self.show(&t),
                    name.text,
                    name.text
                );
                self.err(span, Msg::verdict(cpl_diag::codes::AMBIGUOUS_METHOD, msg));
                return Some(Self::error_node(span));
            }
        };
        let r = decls::TraitRef {
            tr,
            args: Vec::new(),
        };
        self.b
            .obligations
            .push((t.clone(), r.clone(), self.b.item.at(span), Origin::Bound));
        Some(self.static_call(t, r, m, name, call, span))
    }
    /// `gen` named where no fn or value is: the self-less fn of that name a trait in view
    /// declares returning `Self`, which a written type argument or the expected type picks the
    /// impl of (R08.2.8).
    pub(super) fn returned_by(&self, callee: &next::Expr) -> Option<(String, (TraitId, FnId))> {
        let Some(n) = &bare_name(callee) else {
            return None;
        };
        if self.lookup(n).is_some() || self.named(self.b.module, n).is_some() {
            return None;
        }
        let sym = self.syms.get(n)?;
        let found = (self.names.names(self.b.module).into_iter().map(|(_, x)| x))
            .filter_map(|x| match x {
                decls::Named::Trait(tr) => Some(tr),
                _ => None,
            })
            .filter_map(|tr| {
                let m = *self.items.t(tr).methods.get(&sym)?;
                let fd = self.items.f(m);
                (fd.self_mode == SelfMode::None && fd.ret == Ty::Param(0)).then_some((tr, m))
            })
            .min_by_key(|(tr, _)| tr.0)?;
        Some((n.clone(), found))
    }
    /// `gen<T>(args)`: `T.gen(args)`, through `T`'s impl of the trait that declares it.
    pub(super) fn call_by_return(
        &mut self,
        t: Ty,
        (tr, m): (TraitId, FnId),
        n: &str,
        args: &[CallArg],
        span: Span,
    ) -> TExpr {
        let r = decls::with_defaults(
            self.items,
            decls::TraitRef {
                tr,
                args: Vec::new(),
            },
            &t,
        );
        self.b
            .obligations
            .push((t.clone(), r.clone(), self.b.item.at(span), Origin::Bound));
        let name = Called {
            text: n,
            sym: self.syms.get(n),
            span,
        };
        self.static_call(t, r, m, name, Some(args), span)
    }
    /// Each bound trait `tr`'s fn `m` states past `Self: tr`, its `where Self: X` too, as the
    /// call's obligation over `tys`: through a bound `T: Thing`, `T` must meet them by its own
    /// bounds (R08.1.4). `Self: tr` is the call's own, which the caller already obliges.
    fn method_bounds(&mut self, tr: TraitId, m: FnId, tys: &[Ty], span: Span) {
        let sub = Subst { tys, sets: &[] };
        let own = |i: usize, r: &decls::TraitRef| i == 0 && r.tr == tr;
        for (i, r) in (self.items.f(m).bounds.iter()).filter(|(i, r)| !own(*i, r)) {
            let r = decls::TraitRef {
                tr: r.tr,
                args: r.args.iter().map(|a| sub.ty(a)).collect(),
            };
            (self.b.obligations).push((tys[*i].clone(), r, self.b.item.at(span), Origin::Bound));
        }
    }
    /// A call of trait `tr`'s fn `m` without `self`, its impl chosen by `self_ty`.
    pub(super) fn static_call(
        &mut self,
        self_ty: Ty,
        tr: decls::TraitRef,
        m: FnId,
        name: Called<'_>,
        call: Option<&[CallArg]>,
        span: Span,
    ) -> TExpr {
        self.reach(m);
        let fd = self.items.f(m);
        if call.is_none() && !fd.getter {
            self.err(
                span,
                Msg::verdict(
                    cpl_diag::codes::UNKNOWN_FIELD,
                    format!("`{}` is a method: call it with `()`", name.text),
                ),
            );
        }
        let mut tys = vec![self_ty.clone()];
        tys.extend(tr.args.iter().cloned());
        let outer = tys.len();
        while tys.len() < fd.generics.len() {
            tys.push(self.new_ty());
        }
        let sets = self.trait_sets(m);
        let sub = Subst {
            tys: &tys,
            sets: &sets,
        };
        let formals = self.formals(&sub, fd);
        self.method_bounds(tr.tr, m, &tys, span);
        let own: Box<[SetTy]> = sets[..fd.n_sets].into();
        let ret = self.trait_ret_in(m, tys.clone(), sets);
        let (pre, args) = self.args(call.unwrap_or(&[]), &formals, span);
        let call = TExpr {
            kind: TK::CallTrait {
                ctx: self.trait_ctx(tr.tr, name.text),
                tr,
                method: name.text.to_string(),
                recv: None,
                self_ty: Box::new(self_ty),
                targs: tys[outer..].into(),
                sets: own,
                args,
            },
            ty: ret,
            span,
        };
        block(pre, call)
    }
    /// `opt.k`: what the innermost `with` around it set, else the field's default (R09.1.2).
    fn ctx_read(&mut self, base: &next::Expr, name: Called<'_>, span: Span) -> Option<TExpr> {
        let Some(n) = &bare_name(base) else {
            return None;
        };
        let Some(decls::Named::Context(id)) = (self.lookup(n).is_none())
            .then(|| self.named(self.b.module, n))
            .flatten()
        else {
            return None;
        };
        let c = self.items.context(id);
        let Some(f) = c
            .fields
            .iter()
            .find(|f| self.syms.get(name.text) == Some(f.name))
        else {
            let code = cpl_diag::codes::UNKNOWN_FIELD;
            self.err(
                span,
                Msg::verdict(code, format!("context `{n}` has no field `{}`", name.text)),
            );
            return Some(TExpr {
                kind: TK::Error,
                ty: Ty::Error,
                span,
            });
        };
        let set = (self.b.frames.iter().rev()).find(|x| x.ctx == id && x.field == name.text);
        let (value, m) = match set {
            Some(x) => (x.value.clone(), self.b.module),
            None => (f.default.clone()?, c.module),
        };
        let m = std::mem::replace(&mut self.b.module, m);
        let ty = match &f.ty {
            decls::TyForm::Written(t) => self.body_ty(t, &mut Vec::new()),
            _ => {
                let msg = format!("context `{n}`'s field `{}` has no written type", name.text);
                self.err(span, Msg::internal(msg));
                Ty::Error
            }
        };
        let v = self.expr(&value, Some(&ty));
        self.b.module = m;
        Some(self.coerce(v, &ty))
    }

    /// `m.f` of an inline `mod m:`: what `m` declares `pub` as `f` (R11.1.3, R11.1.2).
    fn module_member(
        &mut self,
        base: &next::Expr,
        name: Called<'_>,
        call: Option<&[CallArg]>,
        span: Span,
        want: Option<&Ty>,
    ) -> Option<TExpr> {
        let Some(m) = &bare_name(base) else {
            return None;
        };
        let Some(decls::Named::Module(t)) = (self.lookup(m).is_none())
            .then(|| self.named(self.b.module, m))
            .flatten()
        else {
            return None;
        };
        let error = TExpr {
            kind: TK::Error,
            ty: Ty::Error,
            span,
        };
        let x = match (self.syms.get(name.text)).and_then(|s| self.names.export(t, s)) {
            Some((x, true)) => x,
            Some((_, false)) => {
                let code = cpl_diag::codes::PRIVATE_NAME;
                let text = format!("`{}` is private to module `{m}`", name.text);
                let help = format!("mark `{}` `pub` in `mod {m}`", name.text);
                self.err_help(span, Msg::verdict(code, text), Some(help));
                return Some(error);
            }
            None => {
                let code = cpl_diag::codes::UNRESOLVED_NAME;
                let text = format!("module `{m}` has no `{}`", name.text);
                self.err(span, Msg::verdict(code, text));
                return Some(error);
            }
        };
        if let Some(args) = call {
            return Some(self.call_of(name.text, x, &[], args, span, want));
        }
        match x {
            decls::Named::Const(c) => Some(self.const_read(c, span)),
            decls::Named::Fn(f) => {
                let f = self.fnref(f, Vec::new(), Vec::new());
                Some(self.fn_item_value(f, span))
            }
            _ => {
                let code = cpl_diag::codes::UNRESOLVED_NAME;
                let text = format!("`{m}.{}` is not a value", name.text);
                self.err(span, Msg::verdict(code, text));
                Some(error)
            }
        }
    }
    /// `recv.name(args)` as a call of trait `tr`'s fn `m`: every impl's fn, or the trait's
    /// default, may be the one that runs, and the bound `rt: tr` picks it.
    fn bound_trait_call(
        &mut self,
        recv: TExpr,
        rt: &Ty,
        (tr, m): (&decls::TraitRef, FnId),
        name: Called<'_>,
        call: Option<&[CallArg]>,
        span: Span,
    ) -> TExpr {
        self.reach(m);
        let fd = self.items.f(m);
        let held = fd.region.is_some() && !std::mem::take(&mut self.b.allow_region);
        if held
            && (fd.region.as_ref().is_some_and(|r| r.0)
                || self.role_structs.role_struct(Role::Recipe).is_none())
        {
            let what = Unbuilt::GeneratorAsValue {
                name: fd.name.clone(),
            };
            self.err(span, what);
        }
        let fd = self.items.f(m);
        let mut tys = vec![rt.clone()];
        tys.extend(tr.args.iter().cloned());
        let outer = tys.len();
        while tys.len() < fd.generics.len() {
            tys.push(self.new_ty());
        }
        let sets = self.trait_sets(m);
        let sub = Subst {
            tys: &tys,
            sets: &sets,
        };
        let formals = self.formals(&sub, fd);
        self.method_bounds(tr.tr, m, &tys, span);
        let own: Box<[SetTy]> = sets[..fd.n_sets].into();
        let ret = self.trait_ret_in(m, tys.clone(), sets);
        let (pre, args) = self.args(call.unwrap_or(&[]), &formals, span);
        let recv = if fd.self_mode == SelfMode::Mut {
            self.mut_recv(recv, span)
        } else {
            TArg::Value(recv)
        };
        let call = TExpr {
            kind: TK::CallTrait {
                ctx: self.trait_ctx(tr.tr, name.text),
                tr: tr.clone(),
                method: name.text.to_string(),
                recv: Some(Box::new(recv)),
                self_ty: Box::new(rt.clone()),
                targs: tys[outer..].into(),
                sets: own,
                args,
            },
            ty: ret,
            span,
        };
        if held && self.role_structs.role_struct(Role::Recipe).is_some() {
            return self.recipe(pre, call);
        }
        block(pre, call)
    }

    /// `[D]T`'s key `D` when it is a parameter with no `Domain` bound: at `D = Int` its instance
    /// is a `List<T>`, so nothing binds to `Array`'s own fns.
    pub(super) fn open_key(&self, rt: &Ty) -> Option<usize> {
        let Ty::Struct(s, ts, _) = rt else {
            return None;
        };
        if Some(*s) != self.array_id() || ts.len() != 2 {
            return None;
        }
        let Ty::Param(i) = self.shallow(&ts[0]) else {
            return None;
        };
        let dom = self.rows.role_trait(Role::Positions)?;
        (!self.bounds_of(i, false).iter().any(|r| r.tr == dom)).then_some(i)
    }
    /// `a.len` on an open `[D]T` (`open_key`): a member of a trait `List` implements too, called
    /// through the trait so each instance picks its impl (R04.8.11). A name no member gives is an
    /// extension fn's, as `max` is `Seq`'s, found as on any receiver.
    fn open_array_member(
        &mut self,
        recv: TExpr,
        rt: &Ty,
        key: usize,
        name: Called<'_>,
        call: Option<&[CallArg]>,
        span: Span,
    ) -> TExpr {
        let through = match self.member_named(rt, name.sym) {
            Some(decls::Member::Method(m, ts, ss)) => self.listed_too(rt, m, (&ts, &ss), key, span),
            Some(decls::Member::Field(..)) | None => None,
        };
        let Some(tr) = through else {
            return self.open_refusal(key, name.text, span);
        };
        let Some(m) = member_of(&self.items.t(tr.tr).methods, name) else {
            self.err(
                span,
                Msg::internal("an impl gives a member its trait lacks"),
            );
            return Self::error_node(span);
        };
        self.b
            .obligations
            .push((rt.clone(), tr.clone(), self.b.item.at(span), Origin::Bound));
        self.bound_trait_call(recv, rt, (&tr, m), name, call, span)
    }
    /// `a.m` on an open `[D]T` where `m` is `Array`'s alone: a `List` has none.
    pub(super) fn open_refusal(&mut self, key: usize, m: &str, span: Span) -> TExpr {
        let d = self.show(&Ty::Param(key));
        let text = format!("`[{d}]T` has `{m}` only where `{d}: Domain`");
        let help = format!("bound `{d}: Domain`");
        self.err_help(
            span,
            Msg::verdict(cpl_diag::codes::MISSING_IMPL, text),
            Some(help),
        );
        Self::error_node(span)
    }
    /// The trait of `Array`'s impl member `m` at `ts`, where `List` implements it at `D = Int`
    /// too: the obligation that `List<T>` does is the impl's bounds on `T`.
    pub(super) fn listed_too(
        &mut self,
        rt: &Ty,
        m: FnId,
        (ts, ss): (&[Ty], &[SetTy]),
        key: usize,
        span: Span,
    ) -> Option<decls::TraitRef> {
        // A trait's default method, as `Indexed`'s `cut_after`, is `Array`'s through the impl
        // of that trait it has, at that impl's arguments.
        let (k, ts) = match self.items.f(m).owner {
            Some(decls::Owner::Impl(k)) => (k, ts.to_vec()),
            Some(decls::Owner::Trait(tr)) => {
                let any = decls::TraitRef {
                    tr,
                    args: Vec::new(),
                };
                match self.impls().best(rt, &any)[..] {
                    [(decls::ImplIdx::Decl(k), ref binds)] => (k, binds.clone()),
                    _ => return None,
                }
            }
            _ => return None,
        };
        let ts = &ts[..];
        let (list, im) = (
            self.role_structs.role_struct(decls::Role::List)?,
            self.items.imp(k),
        );
        let at = |tys: &[Ty]| {
            let sub = Subst { tys, sets: ss };
            decls::TraitRef {
                tr: im.trait_.tr,
                args: im.trait_.args.iter().map(|a| sub.ty(a)).collect(),
            }
        };
        let ints: Vec<Ty> = (ts.iter())
            .map(|t| match self.shallow(t) {
                Ty::Param(i) if i == key => Ty::Int,
                t => t,
            })
            .collect();
        let (tr, listed) = (at(ts), at(&ints));
        let elem = ints.last()?.clone();
        let lt = Ty::Struct(list, vec![elem], Vec::new());
        if !self.impls().holds(&lt, &listed) {
            return None;
        }
        (self.b.obligations).push((lt, listed, self.b.item.at(span), Origin::Bound));
        Some(tr)
    }

    pub(super) fn member(
        &mut self,
        base: &next::Expr,
        name: Called<'_>,
        call: Option<&[CallArg]>,
        span: Span,
        want: Option<&Ty>,
        written_targs: &[Ty],
    ) -> TExpr {
        // A tuple's `.1` is its field `_1`, read or called (R02.5).
        let digit = format!("_{}", name.text);
        let name = match name.text.starts_with(|c: char| c.is_ascii_digit()) {
            true => Called::written(self.syms, &digit, name.span),
            false => name,
        };
        if let Some(e) = self.ctx_read(base, name, span) {
            self.no_targs(name, written_targs, span);
            return e;
        }
        if let Some(e) = self.module_member(base, name, call, span, want) {
            self.no_targs(name, written_targs, span);
            return e;
        }
        // `f.clear()` of a `@memo` fn: the fn declared beside it as its clear (R06.4.1).
        if let (Some(f), Some(args)) = (bare_name(base), call) {
            match self.memo_clear(&f, name.text) {
                Some(Clear::Bound(v)) => {
                    let ty = self.var_ty(v);
                    let clear = TExpr {
                        kind: TK::Var(v),
                        ty,
                        span,
                    };
                    return self.call_value(clear, args, span);
                }
                Some(Clear::Declared(c)) => {
                    let n = self.items.f(c).name.clone();
                    return self.call_of(&n, decls::Named::Fn(c), &[], args, span, want);
                }
                None => {}
            }
        }
        if let Some(e) = self.domain_member(base, name, call, span, want) {
            self.no_targs(name, written_targs, span);
            return e;
        }
        if let Some(e) = self.bound_static(base, name, call, span) {
            self.no_targs(name, written_targs, span);
            return e;
        }
        if let Some(e) = self.named_type_static(base, name, call, span) {
            self.no_targs(name, written_targs, span);
            return e;
        }
        if let Some(e) = self.trait_call(base, name, call, span, want) {
            self.no_targs(name, written_targs, span);
            return e;
        }
        // `Type.f(args)` or `Type<A>.f(args)`: a fn of the type with no receiver.
        let (ty_name, written) = match base {
            next::Expr::Generic(g) => match g.base().as_ref().and_then(bare_name) {
                Some(n) => (Some(n), written_types(g)),
                None => (None, Vec::new()),
            },
            _ => (bare_name(base), Vec::new()),
        };
        let ty_name = ty_name.as_ref();
        // `Enum.Variant(args)`, or `Enum.Variant` with no payload (R02.7.1).
        if let Some(n) = ty_name {
            if let (Some(e), true) = (
                self.struct_named(self.b.module, n),
                self.lookup(n).is_none(),
            ) {
                if let Some(i) = self.variant_named(e, name.text) {
                    self.no_targs(name, written_targs, span);
                    let mut pre = Vec::new();
                    let given: Vec<Ty> = (written.iter())
                        .map(|t| self.body_ty(t, &mut pre))
                        .collect();
                    let given = self.shapes.with_defaults_of(e, &given);
                    let v = self.variant(e, i, given, call.unwrap_or(&[]), span);
                    return block(pre, v);
                }
            }
        }
        let named = ty_name
            .filter(|n| self.lookup(n).is_none())
            .and_then(|n| Some((n, self.struct_named(self.b.module, n)?)));
        // `Dir4.up`: a struct's const.
        if let (Some((_, d)), None) = (named, call) {
            if let Some(&c) =
                (self.syms.get(name.text)).and_then(|n| self.items.s(d).consts.get(&n))
            {
                self.no_targs(name, written_targs, span);
                return self.const_read(c, span);
            }
        }
        // `MaxSub.zero`: a self-less `@get` fn of a trait the struct implements, read without
        // `()`, where the struct has no member of that name.
        if let (Some((_, d)), None) = (named, call) {
            if member_of(&self.items.s(d).methods, name).is_none() && !self.walks_for(d, name) {
                let mut pre = Vec::new();
                let given = (written.iter().take(self.items.s(d).generics.len()))
                    .map(|t| self.body_ty(t, &mut pre))
                    .collect::<Vec<_>>();
                let t = Ty::Struct(d, self.shapes.with_defaults_of(d, &given), Vec::new());
                if let Some(e) = self.type_static(t, name, None, span) {
                    self.no_targs(name, written_targs, span);
                    return block(pre, e);
                }
            }
        }
        // `Subset<B>.empty`: a self-less `@get` fn of the type, read without `()`.
        let get = named.filter(|_| call.is_none()).is_some_and(|(_, d)| {
            (member_of(&self.items.s(d).methods, name)).is_some_and(|m| {
                let f = self.items.f(m);
                f.getter && f.self_mode == SelfMode::None
            })
        });
        let call = call.or(get.then_some(&[][..]));
        let walks = named.is_some_and(|(_, d)| self.walks_at(d, &written, name));
        if let (Some(n), Some(args)) = (ty_name, call) {
            if let Some((_, d)) = named.filter(|&(_, d)| !walks && !self.walks_for(d, name)) {
                let s = self.items.s(d);
                let Some(m) = member_of(&s.methods, name) else {
                    // `Str.default()`: a self-less fn of a trait the struct implements.
                    let mut pre = Vec::new();
                    let given = (written.iter().take(s.generics.len()))
                        .map(|t| self.body_ty(t, &mut pre))
                        .collect::<Vec<_>>();
                    let t = Ty::Struct(d, self.shapes.with_defaults_of(d, &given), Vec::new());
                    if let Some(e) = self.type_static(t, name, Some(args), span) {
                        return block(pre, e);
                    }
                    self.err(
                        span,
                        Msg::verdict(
                            cpl_diag::codes::UNKNOWN_FIELD,
                            format!("`{n}` has no `{}`", name.text),
                        ),
                    );
                    return TExpr {
                        kind: TK::Error,
                        ty: Ty::Error,
                        span,
                    };
                };
                if written.len() > s.generics.len() {
                    self.err(
                        span,
                        Msg::verdict(
                            cpl_diag::codes::WRONG_ARG_COUNT,
                            format!(
                                "`{n}` takes {} type arguments, found {}",
                                s.generics.len(),
                                written.len()
                            ),
                        ),
                    );
                }
                let mut pre = Vec::new();
                let given: Vec<Ty> = (written.iter().take(s.generics.len()))
                    .map(|t| self.body_ty(t, &mut pre))
                    .collect();
                let given = self.shapes.with_defaults_of(d, &given);
                let f = self.fnref(m, given, Vec::new());
                let call = self.call_fn(f, None, args, span, want);
                return block(pre, call);
            }
        }
        // A `for` walks `base.name`, not `base`.
        let walk = std::mem::take(&mut self.b.allow_region);
        let recv = self.expr(base, None);
        self.b.allow_region = walk;
        self.member_on(recv, name, call, span, want, written_targs)
    }

    /// `name` read or called on `recv`, which is checked already: a field-fn value `.name`
    /// reads it on its parameter this way.
    pub(super) fn member_on(
        &mut self,
        recv: TExpr,
        name: Called,
        call: Option<&[CallArg]>,
        span: Span,
        want: Option<&Ty>,
        written_targs: &[Ty],
    ) -> TExpr {
        let rt = self.recv_ty(&recv.ty);
        if rt == Ty::Error {
            return Self::error_node(span);
        }
        // A closure's own type has no members: the impls of its fn type give them, as at a
        // bound (`impl Named for (Int, Int) -> Int` gives `f.name()`).
        let (recv, rt) = match &rt {
            Ty::Clo(c) if self.member_named(&rt, name.sym).is_none() => {
                let (ps, r) = self.clo_sig(c);
                let ft = Ty::Fn(ps, Box::new(r), self.new_set());
                if self.member_named(&ft, name.sym).is_some() {
                    (self.coerce(recv, &ft), ft)
                } else {
                    (recv, rt)
                }
            }
            _ => (recv, rt),
        };
        // `recv.name` without `()` reads a `@get` fn that extends `recv` before a method of that
        // name; walked by a `for`, it names the yielding method.
        let walked = call.is_none() && self.b.allow_region;
        let getter = (call.is_none() && !walked)
            .then(|| self.named(self.b.module, name.text))
            .flatten()
            .or_else(|| {
                (call.is_none() && !walked)
                    .then(|| {
                        (self.syms.get(name.text))
                            .and_then(|n| decls::method_for(self.items, self.methods, n, &rt))
                    })
                    .flatten()
                    .map(decls::Named::Fn)
            })
            .filter(|g| matches!(g, decls::Named::Fn(f) if self.items.f(*f).getter));
        // A method of a bound on a type parameter.
        if let Ty::Param(i) = &rt {
            let pick = |fd: &decls::FnDecl| getter.is_none() || fd.getter;
            match self.bound_giving(*i, name, pick, span) {
                Ok(Some((tr, m))) => {
                    let e = self.bound_trait_call(recv, &rt, (&tr, m), name, call, span);
                    if let Some(w) = want.filter(|_| self.unknown(&e.ty)) {
                        self.unify(&e.ty, w);
                    }
                    return e;
                }
                Ok(None) => {}
                Err(two) => {
                    let shown = self.text(recv.span);
                    return self.two_traits_give(two, shown, &rt, name, shown, span);
                }
            }
        }
        if let Some(k) = self.open_key(&rt) {
            if self.member_named(&rt, name.sym).is_some() {
                let e = self.open_array_member(recv, &rt, k, name, call, span);
                if let Some(w) = want.filter(|_| self.unknown(&e.ty)) {
                    self.unify(&e.ty, w);
                }
                return e;
            }
        }
        // Members are found before extension fns: two impls that give `name` leave the call to
        // say which (R08.1.2, R08.1.4).
        if !matches!(rt, Ty::Param(_)) && self.member_named(&rt, name.sym).is_none() {
            let choices = (self.syms.get(name.text)).map_or_else(Vec::new, |sym| {
                self.impls().impl_choices(&rt, sym, None, |_| true)
            });
            let mut trs: Vec<TraitId> = choices.iter().map(|c| c.0.tr).collect();
            trs.sort_by_key(|t| t.0);
            trs.dedup();
            match trs[..] {
                // One trait at several arguments: the bound picks the impl once they are known.
                [tr] if choices.len() > 1 => {
                    let r = decls::TraitRef {
                        tr,
                        args: (0..choices[0].0.args.len())
                            .map(|_| self.new_ty())
                            .collect(),
                    };
                    self.b.obligations.push((
                        rt.clone(),
                        r.clone(),
                        self.b.item.at(span),
                        Origin::Bound,
                    ));
                    let Some(m) = member_of(&self.items.t(tr).methods, name) else {
                        self.err(
                            span,
                            Msg::internal("an impl gives a member its trait lacks"),
                        );
                        return Self::error_node(span);
                    };
                    let e = self.bound_trait_call(recv, &rt, (&r, m), name, call, span);
                    // The expected type may say the arguments: `let b: Bool = 5.cv()`.
                    if let Some(w) = want.filter(|_| self.unknown(&e.ty)) {
                        self.unify(&e.ty, w);
                    }
                    return e;
                }
                [a, b, ..] => {
                    let shown = self.text(recv.span);
                    return self.two_traits_give((a, b), shown, &rt, name, shown, span);
                }
                _ => {}
            }
        }
        if self.is_ambiguous(self.b.module, name.text) && self.member_named(&rt, name.sym).is_none()
        {
            self.two_imports(name.text, name.span);
            return TExpr {
                kind: TK::Error,
                ty: Ty::Error,
                span,
            };
        }
        // A free fn whose `self` the receiver fits extends it; members are found first (R08.1.4).
        let ext = match self.member_named(&rt, name.sym) {
            None => ((self.syms.get(name.text))
                .and_then(|n| decls::shaped_member(self.items, self.methods, &rt, n))
                .map(decls::Named::Fn))
            .or_else(|| {
                // A free fn with no `self` in view does not hide the method a module declares.
                (self.named(self.b.module, name.text)).filter(
                    |n| !matches!(n, decls::Named::Fn(f) if self.items.f(*f).self_ty.is_none()),
                )
            })
            .or_else(|| {
                // A list is `[Int]T`, so a `self: [D]T` fn takes it too (R04.8.11).
                let as_array = match (&rt, self.array_id()) {
                    (Ty::Struct(l, ts, _), Some(a))
                        if Some(*l) == self.role_structs.role_struct(Role::List) =>
                    {
                        Some(Ty::Struct(a, vec![Ty::Int, ts[0].clone()], Vec::new()))
                    }
                    _ => None,
                };
                (self.syms.get(name.text))
                    .and_then(|n| {
                        decls::method_for(self.items, self.methods, n, &rt).or_else(|| {
                            as_array
                                .and_then(|r| decls::method_for(self.items, self.methods, n, &r))
                        })
                    })
                    .map(decls::Named::Fn)
            }),
            Some(decls::Member::Method(m, ..)) if !self.items.f(m).getter && getter.is_some() => {
                getter
            }
            Some(_) => {
                self.hidden_by_member(&rt, recv.span, name);
                None
            }
        };
        let ext = match (ext, call) {
            (Some(decls::Named::Fn(f)), Some(args)) => match self.overload_for(f, args, span) {
                Some(f) => Some(decls::Named::Fn(f)),
                None => return Self::error_node(span),
            },
            (ext, _) => ext,
        };
        if let Some(decls::Named::Fn(fid)) = ext {
            if let Some(st) = self.items.f(fid).self_ty.clone() {
                let mut ts = self.written_targs(fid, Vec::new(), written_targs, span);
                let ranged = self.ranged_targ(fid, &mut ts);
                let f = self.fnref(fid, ts, Vec::new());
                let sub = Subst {
                    tys: &f.targs,
                    sets: &f.sets,
                };
                let st = sub.ty(&st);
                let st = self.fresh_sets(&st);
                // `self: 0..` takes an integer as a parameter of that type does: checked in.
                let recv_ty = self.shallow(&recv.ty);
                let scalar = self.shallow(&st).is_int_range()
                    && (recv_ty == Ty::Int || recv_ty.is_int_range());
                // A closure's own type meets a fn-typed `self` as an argument does: coerced in.
                let clo = match (self.shallow(&recv.ty), self.shallow(&st)) {
                    (Ty::Clo(c), Ty::Fn(ps, ..)) => self.clo_sig(&c).0.len() == ps.len(),
                    _ => false,
                };
                // A list receiver is the list itself, literal or not: its key is `Int` (R04.8.11).
                let listed = match (&recv_ty, self.shallow(&st)) {
                    (Ty::Struct(l, ..), Ty::Struct(a, ts, _)) => {
                        Some(*l) == self.role_structs.role_struct(Role::List)
                            && Some(a) == self.array_id()
                            && self.unify(&ts[0], &Ty::Int)
                    }
                    _ => false,
                };
                let scalar = scalar || listed;
                if scalar || clo || self.unify(&recv.ty, &st) {
                    let recv = if scalar || clo {
                        self.coerce(recv, &st)
                    } else {
                        recv
                    };
                    let fd = self.items.f(fid);
                    if call.is_none() && !fd.getter {
                        self.err(
                            span,
                            Msg::verdict(
                                cpl_diag::codes::UNKNOWN_FIELD,
                                format!("`{}` is a method: call it with `()`", name.text),
                            ),
                        );
                    }
                    let getter = fd.getter && fd.params.is_empty();
                    let recv = self.recv_arg(fid, recv, span);
                    let v = self.call_method(f, recv, getter, call, span, want);
                    return self.held_to(v, ranged);
                }
            }
        }
        let found = self.member_named(&rt, name.sym);
        let at = self.b.module;
        let hidden = (found.as_ref())
            .and_then(|x| self.operator_by_name(x, recv.span, name, call))
            .or_else(|| {
                (found.as_ref()).and_then(|x| {
                    decls::hidden_member(self.items, self.names, x, &rt, name.text, at)
                })
            });
        if let Some((msg, help)) = hidden {
            self.err_help(span, msg, Some(help));
            return Self::error_node(span);
        }
        match found {
            Some(decls::Member::Field(i, ty)) => {
                self.no_targs(name, written_targs, span);
                let f = TExpr {
                    kind: TK::Field(Box::new(recv), i),
                    ty,
                    span,
                };
                match call {
                    Some(args) => self.call_value(f, args, span),
                    None => f,
                }
            }
            Some(decls::Member::Method(m, ts, ss)) => {
                let fd = self.items.f(m);
                let walked = walked && fd.region.is_some();
                if call.is_none() && !fd.getter && !walked {
                    self.err(
                        span,
                        Msg::verdict(
                            cpl_diag::codes::UNKNOWN_FIELD,
                            format!("`{}` is a method: call it with `()`", name.text),
                        ),
                    );
                }
                let getter = (fd.getter || walked) && fd.params.is_empty();
                let mut ts = self.written_targs(m, ts, written_targs, span);
                let ranged = self.ranged_targ(m, &mut ts);
                let f = self.fnref(m, ts, ss);
                let recv = self.recv_arg(m, recv, span);
                let v = self.call_method(f, recv, getter, call, span, want);
                self.held_to(v, ranged)
            }
            None => {
                if let Some((was, help)) = self.renamed(&rt, recv.span, name, call) {
                    let (code, text) = match was {
                        Some(now) => (
                            cpl_diag::codes::REMOVED_SYNTAX,
                            cpl_diag::msgs::check::REMOVED_SYNTAX_IS_GONE_IN_0
                                .say(&[&name.text, &now]),
                        ),
                        None => (
                            cpl_diag::codes::UNKNOWN_FIELD,
                            format!("`{}` has no member `{}`", self.show(&rt), name.text),
                        ),
                    };
                    self.err_help(span, Msg::verdict(code, text), Some(help));
                    return TExpr {
                        kind: TK::Error,
                        ty: Ty::Error,
                        span,
                    };
                }
                if self.mentions_error(&rt) {
                    return Self::error_node(span);
                }
                let help = (self.free_fn_help(recv.span, name, call))
                    .or_else(|| self.lacked_trait_help(&rt, name));
                self.err_help(
                    span,
                    Msg::verdict(
                        cpl_diag::codes::UNKNOWN_FIELD,
                        format!("`{}` has no member `{}`", self.show(&rt), name.text),
                    ),
                    help,
                );
                TExpr {
                    kind: TK::Error,
                    ty: Ty::Error,
                    span,
                }
            }
        }
    }

    /// The name `@memo` fn `f`'s clear is bound under, found through `f`'s binding (R06.4.1).
    /// The clear `@memo` gave `f`, when `member` is the name its expansion gave it.
    fn memo_clear(&self, f: &str, member: &str) -> Option<Clear> {
        let (clear, name) = match self.lookup(f) {
            Some((v, _)) => {
                let c = *self.b.memo_clears.get(&v)?;
                let name = match c {
                    VarRef::Local(k) => &self.b.locals[k].name,
                    VarRef::Global(k) => &self.globals[k].name,
                };
                (Clear::Bound(c), name)
            }
            None => {
                let Some(decls::Named::Fn(w)) = self.named(self.b.module, f) else {
                    return None;
                };
                let c = self.facts.memo_clear(w)?;
                (Clear::Declared(c), &self.items.f(c).name)
            }
        };
        (name.rsplit_once('.').map(|(_, m)| m) == Some(member)).then_some(clear)
    }
    /// The type a binding was given.
    pub(super) fn var_ty(&self, v: VarRef) -> Ty {
        match v {
            VarRef::Local(k) => self.b.locals[k].ty.clone(),
            VarRef::Global(k) => self.globals[k].ty.clone(),
        }
    }
}

/// Whether source text `e` takes `.m` as written: no operator, space or prefix at its top level,
/// so `0..3` and `-x` are not, and `[1, 2]` and `f(a, b).c` are.
fn postfix_safe(e: &str) -> bool {
    let mut depth = 0usize;
    let mut quote = None;
    let mut prev = None;
    for c in e.chars() {
        match (quote, c) {
            (Some(q), _) if c == q && prev != Some('\\') => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(c),
            (None, '(' | '[' | '{') => depth += 1,
            (None, ')' | ']' | '}') => depth = depth.saturating_sub(1),
            (None, '.') if depth == 0 && prev == Some('.') => return false,
            (None, c) if depth == 0 && (c.is_whitespace() || "+-*/%<>=!&|^?:~".contains(c)) => {
                return false;
            }
            _ => {}
        }
        prev = Some(c);
    }
    true
}

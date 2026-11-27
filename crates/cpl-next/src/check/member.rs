//! Members: a field read or a method call on a receiver, found on its type, a bound or an
//! extension fn (R08.1.4).
use super::expr::block;
use super::*;

/// Whether a call with these arguments could be of `f`: `D.count(pred)` is not a call of the
/// getter `Domain.count`, which takes none, so a member that takes them is looked for instead.
pub(super) fn takes(f: &decls::FnDecl, call: Option<&[ast::Arg]>) -> bool {
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
        let rt = self.shallow(&recv.ty);
        if rt == Ty::Error {
            return Self::error_node(span);
        }
        let Some(decls::Member::Method(m, ts, ss)) = self.d.lookup_member(&rt, name) else {
            self.err(
                span,
                format!(
                    "{}: `{}` has no `{name}`",
                    cpl_diag::codes::UNKNOWN_FIELD.0,
                    self.show(&rt)
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
    /// `recv.m(args)` with `m` already found on `recv`'s type.
    pub(super) fn call_member(
        &mut self,
        recv: TExpr,
        (m, ts, ss): (FnId, Vec<Ty>, Vec<SetTy>),
        args: Vec<TArg>,
        span: Span,
        mutates: bool,
    ) -> TExpr {
        let fd = self.d.f(m);
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
        } else if self.d.writes_cache(m) {
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
        let code = cpl_diag::codes::AMBIGUOUS_METHOD.0;
        self.err_help(
            span,
            format!("{code}: two `use` lines import `{n}`, and neither is more specific"),
            Some(format!("keep one `use` of `{n}`, or call it by its module")),
        );
    }
    /// `x.f(a)` where `f` is a fn without `self` in scope: the call it means is `f(x, a)`
    /// (R06.1.2).
    fn free_fn_help(
        &self,
        base: &ast::Expr,
        name: &ast::Name,
        call: Option<&[ast::Arg]>,
    ) -> Option<String> {
        let args = call?;
        let Some(decls::Named::Fn(f)) = self.d.views[self.module.0].get(&name.text) else {
            return None;
        };
        if self.d.f(*f).self_ty.is_some() {
            return None;
        }
        let mut written = vec![self.text(base.span).to_string()];
        written.extend(args.iter().map(|a| self.text(a.value.span).to_string()));
        let m = &name.text;
        Some(format!(
            "`{m}` is a free fn, not a method: `{m}({})`",
            written.join(", ")
        ))
    }
    /// `x.m(..)` where a fn the receiver has declares `@aka(m)` or `@was(m)`: the call it means.
    /// The name `@was` gives back is said in an E0503 (R06.4.11).
    fn renamed(
        &self,
        rt: &Ty,
        base: &ast::Expr,
        name: &ast::Name,
        call: Option<&[ast::Arg]>,
    ) -> Option<(Option<String>, String)> {
        let m = &name.text;
        let f = (0..self.d.fns.len()).map(FnId).find_map(|id| {
            let f = self.d.f(id);
            (self.d.seen(f.module)
                && (f.aka.contains(m) || f.was.contains(m))
                && (self.d.lookup_member(rt, &f.name).is_some() || self.d.self_fits(id, rt)))
            .then_some(f)
        })?;
        let now = &f.name;
        let mut written = format!("{}.{now}", self.text(base.span));
        if let Some(args) = call {
            let args: Vec<&str> = args.iter().map(|a| self.text(a.value.span)).collect();
            written = format!("{written}({})", args.join(", "));
        }
        match f.was.contains(m) {
            true => Some((Some(now.clone()), format!("write `{written}`"))),
            false => Some((None, format!("`{m}` is spelled `{now}`: `{written}`"))),
        }
    }
    /// A fn this module declares with a `self` of the receiver's own type is never called when
    /// the type has a member of its name: the member is found first (R08.1.4).
    fn hidden_by_member(&mut self, rt: &Ty, base: &ast::Expr, name: &ast::Name) {
        let Ty::Struct(s, ..) = rt else {
            return;
        };
        let m = &name.text;
        let mine = match self.d.views[self.module.0].get(m) {
            Some(decls::Named::Fn(f)) => {
                let fd = self.d.f(*f);
                fd.module == self.module && matches!(&fd.self_ty, Some(Ty::Struct(t, ..)) if t == s)
            }
            _ => false,
        };
        if !mine {
            return;
        }
        let shown = self.text(base.span);
        let ty = &self.d.s(*s).name;
        self.err_help(
            name.span,
            format!(
                "{}: `{shown}.{m}` never calls the `fn {m}` that takes it: `{ty}.{m}` is the type's own member, and members are found before extension fns",
                cpl_diag::codes::AMBIGUOUS_METHOD.0
            ),
            Some(format!("rename the fn, or call it as a plain fn: `{m}({shown})`")),
        );
    }
    /// `T.zero` or `T.f(args)` on a type parameter: a fn without `self` of one of its bounds,
    /// chosen per instance by `T`.
    fn bound_static(
        &mut self,
        base: &ast::Expr,
        name: &ast::Name,
        call: Option<&[ast::Arg]>,
        span: Span,
    ) -> Option<TExpr> {
        let ExprKind::Name(n) = &base.kind else {
            return None;
        };
        if self.lookup(n).is_some() {
            return None;
        }
        let i = self.generics.iter().position(|g| g == n)?;
        let (tr, m) = (self.bound_traits(i).into_iter()).find_map(|tr| {
            let m = *self.d.traits[tr.tr.0].methods.get(&name.text)?;
            let fd = self.d.f(m);
            (fd.self_mode == SelfMode::None && takes(fd, call)).then_some((tr, m))
        })?;
        Some(self.static_call(Ty::Param(i), tr, m, name, call, span))
    }
    /// `T.f(args)` on a type that is not a parameter, as `V.cuts.count`: a fn without `self` of
    /// a trait it implements.
    pub(super) fn type_static(
        &mut self,
        t: Ty,
        name: &ast::Name,
        call: Option<&[ast::Arg]>,
        span: Span,
    ) -> Option<TExpr> {
        let (tr, m) = (0..self.d.traits.len()).find_map(|k| {
            let td = &self.d.traits[k];
            let m = *td.methods.get(&name.text)?;
            let fd = self.d.f(m);
            let tr = TraitId(k);
            (fd.self_mode == SelfMode::None
                && takes(fd, call)
                && td.defaults.is_empty()
                && self.d.has_impl(&t, tr))
            .then_some((tr, m))
        })?;
        let r = decls::TraitRef {
            tr,
            args: Vec::new(),
        };
        self.obligations
            .push((t.clone(), r.clone(), span, Origin::Bound));
        Some(self.static_call(t, r, m, name, call, span))
    }
    /// A call of trait `tr`'s fn `m` without `self`, its impl chosen by `self_ty`.
    pub(super) fn static_call(
        &mut self,
        self_ty: Ty,
        tr: decls::TraitRef,
        m: FnId,
        name: &ast::Name,
        call: Option<&[ast::Arg]>,
        span: Span,
    ) -> TExpr {
        self.reach(m);
        let fd = self.d.f(m);
        if call.is_none() && !fd.getter {
            self.err(
                span,
                format!(
                    "{}: `{}` is a method: call it with `()`",
                    cpl_diag::codes::UNKNOWN_FIELD.0,
                    name.text
                ),
            );
        }
        let mut tys = vec![self_ty.clone()];
        tys.extend(tr.args.iter().cloned());
        let sub = Subst {
            tys: &tys,
            sets: &[],
        };
        let formals = self.formals(&sub, fd);
        let ret = sub.ty(&fd.ret);
        let (pre, args) = self.args(call.unwrap_or(&[]), &formals, span);
        let call = TExpr {
            kind: TK::CallTrait {
                tr,
                method: name.text.clone(),
                recv: None,
                self_ty: Box::new(self_ty),
                args,
            },
            ty: ret,
            span,
        };
        block(pre, call)
    }
    pub(super) fn member(
        &mut self,
        base: &ast::Expr,
        name: &ast::Name,
        call: Option<&[ast::Arg]>,
        span: Span,
        want: Option<&Ty>,
        written_targs: &[Ty],
    ) -> TExpr {
        if let Some(e) = self.domain_member(base, name, call, span, want) {
            self.no_targs(name, written_targs, span);
            return e;
        }
        if let Some(e) = self.bound_static(base, name, call, span) {
            self.no_targs(name, written_targs, span);
            return e;
        }
        // `Type.f(args)` or `Type<A>.f(args)`: a fn of the type with no receiver.
        let (ty_name, written) = match &base.kind {
            ExprKind::Name(n) => (Some(n), &[][..]),
            ExprKind::Generic { base, args } => match &base.kind {
                ExprKind::Name(n) => (Some(n), &args[..]),
                _ => (None, &[][..]),
            },
            _ => (None, &[][..]),
        };
        // `Enum.Variant(args)`, or `Enum.Variant` with no payload (R02.7.1).
        if let Some(n) = ty_name {
            if let (Some(e), true) = (
                self.d.struct_named(self.module, n),
                self.lookup(n).is_none(),
            ) {
                if let Some(i) = (self.d.s(e).variants.iter()).position(|(v, _)| *v == name.text) {
                    self.no_targs(name, written_targs, span);
                    let mut pre = Vec::new();
                    let given: Vec<Ty> = (written.iter())
                        .map(|t| self.body_ty(t, span, &mut pre))
                        .collect();
                    let given = self.d.with_defaults_of(e, given);
                    let v = self.variant(e, i, given, call.unwrap_or(&[]), span);
                    return block(pre, v);
                }
            }
        }
        if let (Some(n), Some(args)) = (ty_name, call) {
            if let (Some(d), true) = (
                self.d.struct_named(self.module, n),
                self.lookup(n).is_none(),
            ) {
                let s = self.d.s(d);
                let Some(m) = s.methods.get(&name.text).copied() else {
                    self.err(
                        span,
                        format!(
                            "{}: `{n}` has no `{}`",
                            cpl_diag::codes::UNKNOWN_FIELD.0,
                            name.text
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
                        format!(
                            "{}: `{n}` takes {} type arguments, found {}",
                            cpl_diag::codes::WRONG_ARG_COUNT.0,
                            s.generics.len(),
                            written.len()
                        ),
                    );
                }
                let mut pre = Vec::new();
                let given: Vec<Ty> = (written.iter().take(s.generics.len()))
                    .map(|t| self.body_ty(t, span, &mut pre))
                    .collect();
                let given = self.d.with_defaults_of(d, given);
                let f = self.fnref(m, given, Vec::new());
                let call = self.call_fn(f, None, args, span, want);
                return block(pre, call);
            }
        }
        // A `for` walks `base.name`, not `base`.
        let walk = std::mem::take(&mut self.allow_region);
        let recv = self.expr(base, None);
        self.allow_region = walk;
        let rt = self.shallow(&recv.ty);
        if rt == Ty::Error {
            return Self::error_node(span);
        }
        // `recv.name` without `()` reads a `@get` fn that extends `recv` before a method of that
        // name; walked by a `for`, it names the yielding method.
        let walked = call.is_none() && self.allow_region;
        let getter = (call.is_none() && !walked)
            .then(|| self.d.views[self.module.0].get(&name.text).copied())
            .flatten()
            .or_else(|| {
                (call.is_none() && !walked)
                    .then(|| self.d.method_for(&name.text, &rt))
                    .flatten()
                    .map(decls::Named::Fn)
            })
            .filter(|g| matches!(g, decls::Named::Fn(f) if self.d.f(*f).getter));
        // A method of a bound on a type parameter.
        if let Ty::Param(i) = &rt {
            for tr in &self.bound_traits(*i) {
                let t = &self.d.traits[tr.tr.0];
                let m = t.methods.get(&name.text).copied();
                if let Some(m) = m.filter(|m| getter.is_none() || self.d.f(*m).getter) {
                    // Every impl's fn, or the trait's default, may be the one that runs.
                    self.reach(m);
                    let fd = self.d.f(m);
                    let held = fd.region.is_some() && !std::mem::take(&mut self.allow_region);
                    if held && (fd.region.as_ref().is_some_and(|r| r.0) || self.d.recipe.is_none())
                    {
                        let what = format!(
                            "slice: `{}` yields or lends, and is used other than in a `for` or as the target of an assignment",
                            fd.name
                        );
                        self.err(span, what);
                    }
                    let fd = self.d.f(m);
                    let mut tys = vec![rt.clone()];
                    tys.extend(tr.args.iter().cloned());
                    let sub = Subst {
                        tys: &tys,
                        sets: &[],
                    };
                    let formals = self.formals(&sub, fd);
                    let ret = sub.ty(&fd.ret);
                    let (pre, args) = self.args(call.unwrap_or(&[]), &formals, span);
                    let recv = if fd.self_mode == SelfMode::Mut {
                        self.mut_recv(recv, span)
                    } else {
                        TArg::Value(recv)
                    };
                    let call = TExpr {
                        kind: TK::CallTrait {
                            tr: tr.clone(),
                            method: name.text.clone(),
                            recv: Some(Box::new(recv)),
                            self_ty: Box::new(rt.clone()),
                            args,
                        },
                        ty: ret,
                        span,
                    };
                    if held && self.d.recipe.is_some() {
                        return self.recipe(pre, call);
                    }
                    return block(pre, call);
                }
            }
        }
        if self.d.ambiguous[self.module.0].contains(&name.text)
            && self.d.lookup_member(&rt, &name.text).is_none()
        {
            self.two_imports(&name.text, name.span);
            return TExpr {
                kind: TK::Error,
                ty: Ty::Error,
                span,
            };
        }
        // A free fn whose `self` the receiver fits extends it; members are found first (R08.1.4).
        let ext = match self.d.lookup_member(&rt, &name.text) {
            None => (self.d.shaped_member(&rt, &name.text).map(decls::Named::Fn))
                .or_else(|| self.d.views[self.module.0].get(&name.text).copied())
                .or_else(|| self.d.method_for(&name.text, &rt).map(decls::Named::Fn)),
            Some(decls::Member::Method(m, ..)) if !self.d.f(m).getter && getter.is_some() => getter,
            Some(_) => {
                self.hidden_by_member(&rt, base, name);
                None
            }
        };
        if let Some(decls::Named::Fn(fid)) = ext {
            if let Some(st) = self.d.f(fid).self_ty.clone() {
                let ts = self.written_targs(fid, Vec::new(), written_targs, span);
                let f = self.fnref(fid, ts, Vec::new());
                let sub = Subst {
                    tys: &f.targs,
                    sets: &f.sets,
                };
                let st = sub.ty(&st);
                let st = self.fresh_sets(&st);
                // `self: 0..` takes an integer as a parameter of that type does: checked in.
                let scalar = matches!(self.shallow(&st), Ty::Range(..) | Ty::RangeOf(_))
                    && matches!(
                        self.shallow(&recv.ty),
                        Ty::Int | Ty::Range(..) | Ty::RangeOf(_)
                    );
                if scalar || self.unify(&recv.ty, &st) {
                    let recv = if scalar { self.coerce(recv, &st) } else { recv };
                    let fd = self.d.f(fid);
                    if call.is_none() && !fd.getter {
                        self.err(
                            span,
                            format!(
                                "{}: `{}` is a method: call it with `()`",
                                cpl_diag::codes::UNKNOWN_FIELD.0,
                                name.text
                            ),
                        );
                    }
                    let getter = fd.getter && fd.params.is_empty();
                    let recv = self.recv_arg(fid, recv, span);
                    return self.call_method(f, recv, getter, call, span, want);
                }
            }
        }
        match self.d.lookup_member(&rt, &name.text) {
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
                let fd = self.d.f(m);
                let walked = walked && fd.region.is_some();
                if call.is_none() && !fd.getter && !walked {
                    self.err(
                        span,
                        format!(
                            "{}: `{}` is a method: call it with `()`",
                            cpl_diag::codes::UNKNOWN_FIELD.0,
                            name.text
                        ),
                    );
                }
                let getter = (fd.getter || walked) && fd.params.is_empty();
                let ts = self.written_targs(m, ts, written_targs, span);
                let f = self.fnref(m, ts, ss);
                let recv = self.recv_arg(m, recv, span);
                self.call_method(f, recv, getter, call, span, want)
            }
            None => {
                self.missing.push(name.text.clone());
                if let Some((was, help)) = self.renamed(&rt, base, name, call) {
                    let (code, text) = match was {
                        Some(now) => (
                            cpl_diag::codes::REMOVED_SYNTAX.0,
                            cpl_diag::msgs::check::REMOVED_SYNTAX_IS_GONE_IN_0
                                .say(&[&name.text, &now]),
                        ),
                        None => (
                            cpl_diag::codes::UNKNOWN_FIELD.0,
                            format!("`{}` has no member `{}`", self.show(&rt), name.text),
                        ),
                    };
                    self.err_help(span, format!("{code}: {text}"), Some(help));
                    return TExpr {
                        kind: TK::Error,
                        ty: Ty::Error,
                        span,
                    };
                }
                let help = self.free_fn_help(base, name, call);
                self.err_help(
                    span,
                    format!(
                        "{}: `{}` has no member `{}`",
                        cpl_diag::codes::UNKNOWN_FIELD.0,
                        self.show(&rt),
                        name.text
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
}

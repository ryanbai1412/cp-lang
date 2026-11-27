//! Members: a field read or a method call on a receiver, found on its type, a bound or an
//! extension fn (R08.1.4).
use super::expr::block;
use super::*;

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
        let recv = if mutates || fd.self_mode == SelfMode::Mut || self.d.writes_cache(m) {
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
        let recv = self.expr(base, None);
        let rt = self.shallow(&recv.ty);
        // A method of a bound on a type parameter.
        if let Ty::Param(i) = &rt {
            for tr in &self.bound_traits(*i) {
                let t = &self.d.traits[tr.tr.0];
                if let Some(m) = t.methods.get(&name.text).copied() {
                    // Every impl's fn, or the trait's default, may be the one that runs.
                    self.reach(m);
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
                        TArg::Mut(recv)
                    } else {
                        TArg::Value(recv)
                    };
                    let call = TExpr {
                        kind: TK::CallTrait {
                            tr: tr.clone(),
                            method: name.text.clone(),
                            recv: Box::new(recv),
                            args,
                        },
                        ty: ret,
                        span,
                    };
                    return block(pre, call);
                }
            }
        }
        // A free fn whose `self` the receiver fits extends it; members are found first (R08.1.4).
        let ext = match self.d.lookup_member(&rt, &name.text) {
            None => (self.d.views[self.module.0].get(&name.text).copied())
                .or_else(|| self.d.method_for(&name.text, &rt).map(decls::Named::Fn)),
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
                if self.unify(&recv.ty, &st) {
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
                let ts = self.written_targs(m, ts, written_targs, span);
                let f = self.fnref(m, ts, ss);
                let recv = self.recv_arg(m, recv, span);
                self.call_method(f, recv, getter, call, span, want)
            }
            None if name.text == "ord"
                && call.is_none()
                && matches!(rt, Ty::Dom(_) | Ty::Param(_)) =>
            {
                TExpr {
                    kind: TK::Ordinal(Box::new(recv)),
                    ty: Ty::Int,
                    span,
                }
            }
            None => {
                self.missing.push(name.text.clone());
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

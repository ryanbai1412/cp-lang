//! Expressions: operators, calls, members, names.

use super::*;

impl<'a> Cx<'a> {
    pub(super) fn binary(&mut self, op: BinOp, l: TExpr, r: TExpr, span: Span) -> TExpr {
        // `s + t` on strings is a new string (R02.3.3).
        if op == BinOp::Add && self.is_str(&l.ty) && self.is_str(&r.ty) {
            return self.lib_call("str_concat", vec![l, r], span);
        }
        let want = if matches!(op, BinOp::And | BinOp::Or) {
            Ty::Bool
        } else {
            Ty::Int
        };
        let l = self.coerce(l, &want);
        let r = self.coerce(r, &want);
        if matches!(op, BinOp::Coalesce | BinOp::Pow) {
            self.err(span, "slice: unsupported operator");
        }
        TExpr {
            kind: TK::Bin(op, Box::new(l), Box::new(r)),
            ty: want,
            span,
        }
    }
    /// The arguments of a call bound to its parameters: positional ones in order, named ones by
    /// label, the rest from their defaults (R06.1.5). In parameter order; what must run first to
    /// keep the written left-to-right order (R05.11) is in the first part.
    pub(super) fn args(
        &mut self,
        args: &[ast::Arg],
        formals: &[Formal<'a>],
        span: Span,
    ) -> (Vec<TStmt>, Vec<TArg>) {
        let mut slot: Vec<Option<usize>> = vec![None; formals.len()];
        let mut next = 0;
        let mut excess = false;
        for (k, a) in args.iter().enumerate() {
            let at = match &a.name {
                None => {
                    while next < formals.len() && formals[next].named_only {
                        next += 1;
                    }
                    next += 1;
                    excess |= next > formals.len();
                    (next <= formals.len()).then(|| next - 1)
                }
                Some(n) => {
                    let at = formals.iter().position(|f| f.label == n.text);
                    if at.is_none() {
                        self.err(a.value.span, format!("no parameter named `{}`", n.text));
                    }
                    at
                }
            };
            match at {
                Some(i) if slot[i].is_none() => slot[i] = Some(k),
                Some(i) => self.err(
                    a.value.span,
                    format!("`{}` is given twice", formals[i].label),
                ),
                None => {}
            }
        }
        let missing = (0..formals.len()).any(|i| slot[i].is_none() && formals[i].default.is_none());
        if excess || (missing && args.iter().all(|a| a.name.is_none())) {
            self.err(
                span,
                format!(
                    "{}: expected {} arguments, found {}",
                    cpl_diag::codes::WRONG_ARG_COUNT.0,
                    formals.len(),
                    args.len()
                ),
            );
        } else if missing {
            for (i, f) in formals.iter().enumerate() {
                if slot[i].is_none() && f.default.is_none() {
                    self.err(
                        span,
                        format!(
                            "{}: no argument for `{}`",
                            cpl_diag::codes::WRONG_ARG_COUNT.0,
                            f.label
                        ),
                    );
                }
            }
        }
        let mut typed: Vec<Option<TArg>> = Vec::with_capacity(args.len());
        for (k, a) in args.iter().enumerate() {
            typed.push(match slot.iter().position(|s| *s == Some(k)) {
                Some(i) => Some(self.arg(a, &formals[i].ty, formals[i].mutable)),
                None => {
                    self.expr(&a.value, None);
                    None
                }
            });
        }
        // Bound in another order than written, or with a default before a written argument: the
        // written ones that compute something run first, in order.
        let written: Vec<usize> = slot.iter().flatten().copied().collect();
        let last = slot.iter().rposition(Option::is_some);
        let reordered = written.windows(2).any(|w| w[0] > w[1])
            || last.is_some_and(|l| slot[..l].iter().any(Option::is_none));
        let mut pre = Vec::new();
        if reordered {
            for t in typed.iter_mut() {
                if let Some(TArg::Value(v)) = t {
                    if !pure(v) {
                        let ty = v.ty.clone();
                        let tmp = self.bind("$arg", ty.clone());
                        let read = TExpr {
                            kind: TK::Var(tmp),
                            ty,
                            span: v.span,
                        };
                        pre.push(TStmt::Let(tmp, std::mem::replace(v, read)));
                    }
                }
            }
        }
        let mut out = Vec::with_capacity(formals.len());
        for (i, f) in formals.iter().enumerate() {
            out.push(match slot[i].and_then(|k| typed[k].take()) {
                Some(a) => a,
                None => TArg::Value(match f.default {
                    Some((e, m)) => self.default_arg(e, m, &f.ty),
                    None => TExpr {
                        kind: TK::Error,
                        ty: Ty::Error,
                        span,
                    },
                }),
            });
        }
        (pre, out)
    }
    fn arg(&mut self, a: &ast::Arg, pty: &Ty, pmut: bool) -> TArg {
        let inner = match &a.value.kind {
            ExprKind::Mut(e) => e,
            _ => &a.value,
        };
        if pmut {
            if !a.mutable && !matches!(a.value.kind, ExprKind::Mut(_)) {
                self.err(
                    a.value.span,
                    format!(
                        "{}: a `mut` parameter is marked `mut` at the call",
                        cpl_diag::codes::MISSING_MUT_AT_CALL.0
                    ),
                );
            }
            let p = self.place(inner);
            if let Some(name) = self.let_root(&p) {
                self.err(
                    a.value.span,
                    format!(
                        "{}: `{name}` is bound by `let`, so it cannot be lent `mut`; declare it with `var`",
                        cpl_diag::codes::MUT_OF_IMMUTABLE.0
                    ),
                );
            }
            if !self.unify(&p.ty, pty) {
                self.err(
                    a.value.span,
                    format!(
                        "expected `{}`, found `{}`",
                        self.show(pty),
                        self.show(&p.ty)
                    ),
                );
            }
            TArg::Mut(p)
        } else {
            let v = self.expr(inner, Some(pty));
            TArg::Value(self.coerce(v, pty))
        }
    }
    /// A default is read where its fn or struct is declared: none of the caller's names are seen.
    fn default_arg(&mut self, e: &ast::Expr, m: ModId, ty: &Ty) -> TExpr {
        let module = std::mem::replace(&mut self.module, m);
        let scopes = std::mem::replace(&mut self.scopes, vec![HashMap::new()]);
        let self_local = self.self_local.take();
        let v = self.expr(e, Some(ty));
        let v = self.coerce(v, ty);
        self.module = module;
        self.scopes = scopes;
        self.self_local = self_local;
        v
    }
    /// The parameters of a fn as a call binds them.
    pub(super) fn formals(&self, sub: &Subst, fd: &'a decls::FnDecl) -> Vec<Formal<'a>> {
        fd.params
            .iter()
            .map(|p| Formal {
                label: &p.label,
                ty: sub.ty(&p.ty),
                mutable: p.mutable,
                named_only: p.named_only,
                default: p.default.as_ref().map(|e| (e, fd.module)),
            })
            .collect()
    }
    /// A reference to a fn with fresh type and set arguments after `given`.
    pub(super) fn fnref(
        &mut self,
        decl: FnId,
        given_tys: Vec<Ty>,
        given_sets: Vec<SetTy>,
    ) -> FnRef {
        self.reach(decl);
        let f = self.d.f(decl);
        let mut targs = given_tys;
        while targs.len() < f.generics.len() {
            targs.push(self.new_ty());
        }
        let mut sets = given_sets;
        while sets.len() < f.n_sets {
            sets.push(self.new_set());
        }
        FnRef { decl, targs, sets }
    }
    /// A call of the library fn `name` with arguments already typed.
    pub(super) fn lib_call(&mut self, name: &str, args: Vec<TExpr>, span: Span) -> TExpr {
        let Some(decl) = self.d.core_fn(name) else {
            self.err(span, format!("the library has no `{name}`"));
            return TExpr {
                kind: TK::Error,
                ty: Ty::Error,
                span,
            };
        };
        let f = self.fnref(decl, Vec::new(), Vec::new());
        let ty = self.ret_ty(&f);
        let fd = self.d.f(f.decl);
        let sub = Subst {
            tys: &f.targs,
            sets: &f.sets,
        };
        let params: Vec<Ty> = fd.params.iter().map(|p| sub.ty(&p.ty)).collect();
        let args = args
            .into_iter()
            .zip(&params)
            .map(|(a, p)| TArg::Value(self.coerce(a, p)))
            .collect();
        TExpr {
            kind: TK::Call {
                f,
                recv: None,
                args,
            },
            ty,
            span,
        }
    }
    /// One value of a `print`: a plain one joins `pieces`, one the library writes
    /// (R10.3.2) becomes a call of its writer after what was gathered so far.
    fn show_value(&mut self, v: TExpr, stmts: &mut Vec<TStmt>, pieces: &mut Vec<Piece>) {
        let span = v.span;
        let writer = match self.shallow(&v.ty) {
            // A position prints as its integer (R04.2.15).
            Ty::Dom(_) | Ty::Param(_) => {
                pieces.push(Piece::Value(TExpr {
                    kind: TK::DomOrd(Box::new(v)),
                    ty: Ty::Int,
                    span,
                }));
                return;
            }
            Ty::Int | Ty::Bool | Ty::Char | Ty::Error => {
                pieces.push(Piece::Value(v));
                return;
            }
            Ty::Struct(d, ..) => ["List", "Array", "Str"]
                .into_iter()
                .zip(["write_list", "write_array", "write_str"])
                .find(|(ty, _)| Some(d) == self.d.core_struct(ty))
                .map(|(_, f)| f),
            _ => None,
        };
        let Some(writer) = writer else {
            let t = self.show(&v.ty);
            self.err(span, format!("slice: printing a `{t}`"));
            return;
        };
        if !pieces.is_empty() {
            stmts.push(TStmt::Expr(TExpr {
                kind: TK::Print(std::mem::take(pieces)),
                ty: Ty::Unit,
                span,
            }));
        }
        let call = self.lib_call(writer, vec![v], span);
        stmts.push(TStmt::Expr(call));
    }
    fn is_str(&self, t: &Ty) -> bool {
        matches!(self.shallow(t), Ty::Struct(d, ..) if Some(d) == self.d.core_struct("Str"))
    }
    /// `a < b`, `a == b` and the rest on a type that is not a number: its `Ord` or `Eq` fn, from
    /// the bound on a type parameter or the type's impl (R08.1.5). `a > b` is `b < a`, `a <= b` is
    /// `not b < a`, and `a != b` is `not a == b`; operands are still read left to right.
    fn op_call(&mut self, op: CmpOp, a: TExpr, b: TExpr, span: Span) -> TExpr {
        let mk = |kind, ty| TExpr { kind, ty, span };
        let Some((trait_name, method)) = op.trait_method() else {
            return mk(TK::Error, Ty::Error);
        };
        let swap = matches!(op, CmpOp::Gt | CmpOp::Le);
        let mut pre = Vec::new();
        let a = if swap && !matches!(a.kind, TK::Var(_)) {
            let tmp = self.bind("$lhs", a.ty.clone());
            let read = mk(TK::Var(tmp), a.ty.clone());
            pre.push(TStmt::Let(tmp, a));
            read
        } else {
            a
        };
        let (recv, arg) = if swap { (b, a) } else { (a, b) };
        let rt = self.shallow(&recv.ty);
        let call = if let Ty::Param(i) = &rt {
            let tr = (self.bound_traits(*i).into_iter())
                .find(|t| self.d.traits[t.tr.0].name == trait_name);
            tr.map(|tr| {
                if let Some(&m) = self.d.traits[tr.tr.0].methods.get(method) {
                    self.reach(m);
                }
                let arg = self.coerce(arg, &rt);
                mk(
                    TK::CallTrait {
                        tr: tr.tr,
                        method: method.to_string(),
                        recv: Box::new(TArg::Value(recv)),
                        args: vec![TArg::Value(arg)],
                    },
                    Ty::Bool,
                )
            })
        } else if let Some(decls::Member::Method(m, ts, ss)) =
            self.d.impl_member(&rt, method, Some(trait_name))
        {
            let f = self.fnref(m, ts, ss);
            let ty = self.ret_ty(&f);
            let fd = self.d.f(f.decl);
            let sub = Subst {
                tys: &f.targs,
                sets: &f.sets,
            };
            let param = fd.params.first().map_or(Ty::Error, |p| sub.ty(&p.ty));
            let arg = self.coerce(arg, &param);
            let recv = self.recv_arg(m, recv, span);
            Some(mk(
                TK::Call {
                    f,
                    recv: Some(Box::new(recv)),
                    args: vec![TArg::Value(arg)],
                },
                ty,
            ))
        } else {
            None
        };
        let Some(call) = call else {
            let shown = self.show(&rt);
            if matches!(rt, Ty::Struct(..)) {
                // Its fields may give it a structural impl (R08.1.6).
                self.err(span, format!("slice: derived `{trait_name}` for `{shown}`"));
            } else {
                self.err(
                    span,
                    format!(
                        "{}: `{shown}` has no `impl {trait_name}`, which `{}` needs (R08.1.5)",
                        cpl_diag::codes::MISSING_IMPL.0,
                        op.text()
                    ),
                );
            }
            return mk(TK::Error, Ty::Error);
        };
        let call = if matches!(op, CmpOp::Ne | CmpOp::Le | CmpOp::Ge) {
            mk(TK::Not(Box::new(call)), Ty::Bool)
        } else {
            call
        };
        block(pre, call)
    }
    /// How a method takes its receiver: a `mut self` (or a cache it fills) needs a place.
    fn recv_arg(&mut self, m: FnId, recv: TExpr, span: Span) -> TArg {
        let place = matches!(recv.kind, TK::Var(_) | TK::Field(..));
        if self.d.f(m).self_mode == SelfMode::Mut {
            if !place {
                self.err(span, "a `mut self` method needs a place");
            }
            TArg::Mut(recv)
        } else if self.d.writes_cache(m) && place {
            TArg::Mut(recv)
        } else {
            TArg::Value(recv)
        }
    }
    pub(super) fn call_fn(
        &mut self,
        f: FnRef,
        recv: Option<TArg>,
        args: &[ast::Arg],
        span: Span,
        want: Option<&Ty>,
    ) -> TExpr {
        let fd = self.d.f(f.decl);
        let sub = Subst {
            tys: &f.targs,
            sets: &f.sets,
        };
        // The expected type fixes type arguments the arguments cannot: `let xs: List<Int> = List.new()`.
        let ty = self.ret_ty(&f);
        if let Some(w) = want {
            if matches!(self.shallow(w), Ty::Struct(..) | Ty::Raw(_))
                && matches!(fd.ret, Ty::Struct(..) | Ty::Param(_) | Ty::Raw(_))
            {
                self.unify(&ty, w);
            }
        }
        let mut formals = self.formals(&sub, fd);
        for p in &mut formals {
            p.ty = self.fresh_sets(&p.ty);
        }
        // The core file stands in for std until std's own `List` and `Str` load (stage 4):
        // a call std's signature takes but the stand-in's does not is not built, not an error.
        if fd.module == decls::CORE && args.len() > formals.len() {
            self.err(
                span,
                format!(
                    "slice: `{}` with {} arguments (the core stand-in takes {})",
                    fd.name,
                    args.len(),
                    formals.len()
                ),
            );
            return TExpr {
                kind: TK::Error,
                ty: Ty::Error,
                span,
            };
        }
        if fd.region.is_some() && !std::mem::take(&mut self.allow_region) {
            self.err(
                span,
                format!(
                    "`{}` yields or lends: use it in a `for` or as the target of an assignment",
                    fd.name
                ),
            );
        }
        self.oblige(&f, span);
        self.solve(false);
        let (mut pre, args) = self.args(args, &formals, span);
        self.solve(false);
        let recv = match recv {
            Some(TArg::Value(r)) if !pre.is_empty() && !pure(&r) => {
                let tmp = self.bind("$recv", r.ty.clone());
                let read = TExpr {
                    kind: TK::Var(tmp),
                    ty: r.ty.clone(),
                    span: r.span,
                };
                pre.insert(0, TStmt::Let(tmp, r));
                Some(TArg::Value(read))
            }
            r => r,
        };
        let call = TExpr {
            kind: TK::Call {
                f,
                recv: recv.map(Box::new),
                args,
            },
            ty,
            span,
        };
        block(pre, call)
    }
    /// `recv.name(args)` with already-typed arguments: used for `get` and `set` behind indexing.
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
            self.err(span, format!("`{}` has no `{name}`", self.show(&rt)));
            return TExpr {
                kind: TK::Error,
                ty: Ty::Error,
                span,
            };
        };
        let fd = self.d.f(m);
        let f = self.fnref(m, ts, ss);
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
    pub(super) fn member(
        &mut self,
        base: &ast::Expr,
        name: &ast::Name,
        call: Option<&[ast::Arg]>,
        span: Span,
        want: Option<&Ty>,
    ) -> TExpr {
        // `D.count`: the size of a domain.
        if let ExprKind::Name(n) = &base.kind {
            if let (Some(dty), "count", None) = (self.domain_named(n), name.text.as_str(), call) {
                return TExpr {
                    kind: TK::DomLen(dty),
                    ty: Ty::Int,
                    span,
                };
            }
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
                    self.err(span, format!("`{n}` has no `{}`", name.text));
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
                            "`{n}` takes {} type arguments, found {}",
                            s.generics.len(),
                            written.len()
                        ),
                    );
                }
                let mut pre = Vec::new();
                let given: Vec<Ty> = (written.iter().take(s.generics.len()))
                    .map(|t| self.body_ty(t, span, &mut pre))
                    .collect();
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
                    if fd.getter && matches!(fd.body, cpl_syntax::ast::FnBody::None) {
                        self.err(span, "slice: a field requirement (`@get` with no body)");
                    }
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
                            tr: tr.tr,
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
            Some(_) => None,
        };
        if let Some(decls::Named::Fn(fid)) = ext {
            if let Some(st) = self.d.f(fid).self_ty.clone() {
                let f = self.fnref(fid, Vec::new(), Vec::new());
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
                            format!("`{}` is a method: call it with `()`", name.text),
                        );
                    }
                    let recv = self.recv_arg(fid, recv, span);
                    return self.call_fn(f, Some(recv), call.unwrap_or(&[]), span, want);
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
                        format!("`{}` is a method: call it with `()`", name.text),
                    );
                }
                let f = self.fnref(m, ts, ss);
                let recv = self.recv_arg(m, recv, span);
                self.call_fn(f, Some(recv), call.unwrap_or(&[]), span, want)
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
                self.err(
                    span,
                    format!("`{}` has no member `{}`", self.show(&rt), name.text),
                );
                TExpr {
                    kind: TK::Error,
                    ty: Ty::Error,
                    span,
                }
            }
        }
    }
    pub(super) fn call_value(&mut self, callee: TExpr, args: &[ast::Arg], span: Span) -> TExpr {
        let (ps, r) = match self.shallow(&callee.ty) {
            Ty::Fn(ps, r, _) => (ps, *r),
            Ty::Clo(c) => self.clo_sig(&c),
            Ty::Error => {
                return TExpr {
                    kind: TK::Error,
                    ty: Ty::Error,
                    span,
                }
            }
            t => {
                self.err(span, format!("`{}` cannot be called", self.show(&t)));
                return TExpr {
                    kind: TK::Error,
                    ty: Ty::Error,
                    span,
                };
            }
        };
        let formals: Vec<Formal> = ps
            .into_iter()
            .map(|ty| Formal {
                label: "",
                ty,
                mutable: false,
                named_only: false,
                default: None,
            })
            .collect();
        if args
            .iter()
            .any(|a| a.mutable || matches!(a.value.kind, ExprKind::Mut(_)))
        {
            self.err(
                span,
                "a `mut` argument to a function value is not built yet",
            );
        }
        let (pre, args) = self.args(args, &formals, span);
        let args = args
            .into_iter()
            .map(|a| match a {
                TArg::Value(v) | TArg::Mut(v) => v,
            })
            .collect();
        let call = TExpr {
            kind: TK::CallValue {
                callee: Box::new(callee),
                args,
            },
            ty: r,
            span,
        };
        block(pre, call)
    }
    pub(super) fn name(&mut self, n: &str, span: Span) -> TExpr {
        if let Some((v, ty)) = self.lookup(n) {
            return TExpr {
                kind: TK::Var(v),
                ty,
                span,
            };
        }
        // A field of `self`, written bare inside a method.
        if let Some(s) = self.self_expr(span) {
            if let Some(decls::Member::Field(i, ty)) = self.d.lookup_member(&s.ty, n) {
                return TExpr {
                    kind: TK::Field(Box::new(s), i),
                    ty,
                    span,
                };
            }
        }
        if let Some(d) = self.d.fn_named(self.module, n) {
            let f = self.fnref(d, Vec::new(), Vec::new());
            return TExpr {
                kind: TK::FnItem(f.clone()),
                ty: Ty::Clo(Box::new(CloRef::Item(f))),
                span,
            };
        }
        self.err(span, format!("unknown name `{n}`"));
        TExpr {
            kind: TK::Error,
            ty: Ty::Error,
            span,
        }
    }
    /// The value of `e`. One read from a slot of a range type is an `Int` (R03.1.4), unless it
    /// goes straight into a slot of the same range.
    pub(super) fn expr(&mut self, e: &ast::Expr, want: Option<&Ty>) -> TExpr {
        let v = self.expr_as_typed(e, want);
        match self.shallow(&v.ty) {
            have @ (Ty::Range(..) | Ty::RangeOf(_))
                if want.map(|w| self.shallow(w)).as_ref() != Some(&have) =>
            {
                TExpr { ty: Ty::Int, ..v }
            }
            _ => v,
        }
    }
    /// `e` with the type its slot has: a range stays a range.
    pub(super) fn expr_as_typed(&mut self, e: &ast::Expr, want: Option<&Ty>) -> TExpr {
        let span = e.span;
        let mk = |kind, ty| TExpr { kind, ty, span };
        match &e.kind {
            ExprKind::Int(i) => mk(TK::Int(*i as i64), Ty::Int),
            ExprKind::Bool(b) => mk(TK::Bool(*b), Ty::Bool),
            ExprKind::Str(ps) if ps.iter().all(|p| matches!(p, StrPiece::Text(_))) => {
                let text: String = ps
                    .iter()
                    .map(|p| match p {
                        StrPiece::Text(t) => t.as_str(),
                        _ => "",
                    })
                    .collect();
                if !text.is_ascii() {
                    self.err(span, "slice: a string with a character outside ASCII");
                }
                let n = mk(TK::Int(text.len() as i64), Ty::Int);
                let raw = mk(TK::Text(text), Ty::Raw(Box::new(Ty::Char)));
                self.lib_call("str_of", vec![raw, n], span)
            }
            ExprKind::Char(c) if c.is_ascii() => {
                let code = mk(TK::Int(i64::from(*c as u8)), Ty::Int);
                mk(TK::CharOf(Box::new(code)), Ty::Char)
            }
            ExprKind::Paren(inner) => self.expr(inner, want),
            ExprKind::None => {
                let ty = match want.filter(|w| self.optional_of(w).is_some()) {
                    Some(w) => w.clone(),
                    None => {
                        self.err(
                            span,
                            format!(
                                "{}: nothing here says what `None` is an optional of; write the type",
                                cpl_diag::codes::INFERENCE_NEEDS_ANNOTATION.0
                            ),
                        );
                        return mk(TK::Error, Ty::Error);
                    }
                };
                let no = mk(TK::Bool(false), Ty::Bool);
                let inner = self.optional_of(&ty).unwrap_or(Ty::Error);
                mk(TK::Struct(vec![no, mk(TK::Unset, inner)]), ty)
            }
            ExprKind::Unwrap(x) => {
                let v = self.expr(x, None);
                let Some(inner) = self.optional_of(&v.ty) else {
                    if self.shallow(&v.ty) != Ty::Error {
                        let t = self.show(&v.ty);
                        self.err(span, format!("slice: `!` on a `{t}`"));
                    }
                    return mk(TK::Error, Ty::Error);
                };
                self.scopes.push(HashMap::new());
                let (bind, flag, val) = self.opened(v, inner.clone());
                self.scopes.pop();
                let check = TStmt::Assert(
                    flag,
                    "the value is None".to_string(),
                    span,
                    CheckKind::Unwrap,
                );
                let mut stmts = bind;
                stmts.push(check);
                mk(TK::Block(stmts, Some(Box::new(val))), inner)
            }
            ExprKind::Binary {
                op: BinOp::Coalesce,
                lhs,
                rhs,
                ..
            } => {
                let l = self.expr(lhs, None);
                let Some(inner) = self.optional_of(&l.ty) else {
                    if self.shallow(&l.ty) != Ty::Error {
                        let t = self.show(&l.ty);
                        self.err(span, format!("slice: `??` on a `{t}`"));
                    }
                    return mk(TK::Error, Ty::Error);
                };
                // An optional right side keeps the result optional: `a ?? b ?? 9` (R01.8.6).
                let r = self.expr(rhs, Some(&inner));
                let opt = l.ty.clone();
                let (r, out) = match self.optional_of(&r.ty) {
                    Some(_) => (self.coerce(r, &opt), opt.clone()),
                    None => (self.coerce(r, &inner), inner.clone()),
                };
                self.scopes.push(HashMap::new());
                let (bind, flag, val) = self.opened(l, inner);
                self.scopes.pop();
                let val = if out == opt { self.some(val, opt) } else { val };
                let pick = mk(
                    TK::If(Box::new(flag), Box::new(val), Box::new(r)),
                    out.clone(),
                );
                mk(TK::Block(bind, Some(Box::new(pick))), out)
            }
            ExprKind::Name(n) => self.name(n, span),
            ExprKind::SelfValue => self.self_expr(span).unwrap_or(mk(TK::Error, Ty::Error)),
            ExprKind::Tuple(xs) if xs.is_empty() => mk(TK::Unit, Ty::Unit),
            ExprKind::Unary { op, expr } => {
                let (want_t, kind): (Ty, fn(Box<TExpr>) -> TK) = match op {
                    UnOp::Not => (Ty::Bool, TK::Not),
                    UnOp::Neg => (Ty::Int, TK::Neg),
                    _ => {
                        self.err(span, "slice: unsupported operator");
                        (Ty::Int, TK::Neg)
                    }
                };
                let v = self.expr(expr, Some(&want_t));
                let v = self.coerce(v, &want_t);
                mk(kind(Box::new(v)), want_t)
            }
            ExprKind::Binary { op, lhs, rhs, .. } => {
                let l = self.expr(lhs, None);
                let r = self.expr(rhs, None);
                self.binary(*op, l, r, span)
            }
            ExprKind::Compare { first, rest } => {
                let up = |o: &CmpOp| matches!(o, CmpOp::Lt | CmpOp::Le);
                let down = |o: &CmpOp| matches!(o, CmpOp::Gt | CmpOp::Ge);
                if rest.iter().any(|(o, _)| up(o)) && rest.iter().any(|(o, _)| down(o)) {
                    self.err(
                        span,
                        format!(
                            "{}: a chain that goes both up and down; write the two comparisons with `and`",
                            cpl_diag::codes::MIXED_COMPARISON_CHAIN.0
                        ),
                    );
                }
                // `a <= b < c` is `a <= b and b < c` with `b` read once, and `c` not read
                // when the first link is false (R01.5.3).
                self.scopes.push(HashMap::new());
                let mut prev = self.expr(first, None);
                let mut links: Vec<(Vec<TStmt>, TExpr)> = Vec::new();
                for (k, (op, rhs)) in rest.iter().enumerate() {
                    let r = self.expr(rhs, Some(&prev.ty));
                    let r = self.coerce(r, &prev.ty.clone());
                    if self.is_str(&prev.ty)
                        && matches!(op, CmpOp::Eq | CmpOp::Ne)
                        && rest.len() == 1
                    {
                        self.scopes.pop();
                        let eq = self.lib_call("str_eq", vec![prev, r], span);
                        return if *op == CmpOp::Eq {
                            eq
                        } else {
                            mk(TK::Not(Box::new(eq)), Ty::Bool)
                        };
                    }
                    let primitive = matches!(
                        self.shallow(&prev.ty),
                        Ty::Int | Ty::Bool | Ty::Char | Ty::Dom(_) | Ty::Error
                    );
                    if matches!(op, CmpOp::In | CmpOp::NotIn) {
                        self.err(span, "slice: comparison of this type is not supported");
                    }
                    // An operand two links share is held, so it is read once.
                    let mut hold = Vec::new();
                    let r = if k + 1 < rest.len() {
                        let tmp = self.bind("$link", r.ty.clone());
                        let read = mk(TK::Var(tmp), r.ty.clone());
                        hold.push(TStmt::Let(tmp, r));
                        read
                    } else {
                        r
                    };
                    // Characters are ordered by code (R02.2.1).
                    let side = |x: TExpr| {
                        if x.ty == Ty::Char {
                            mk(TK::CodeOf(Box::new(x)), Ty::Int)
                        } else {
                            x
                        }
                    };
                    let c = if primitive || matches!(op, CmpOp::In | CmpOp::NotIn) {
                        mk(
                            TK::Cmp(*op, Box::new(side(prev)), Box::new(side(r.clone()))),
                            Ty::Bool,
                        )
                    } else {
                        self.op_call(*op, prev, r.clone(), span)
                    };
                    links.push((hold, c));
                    prev = r;
                }
                self.scopes.pop();
                let mut acc: Option<TExpr> = None;
                for (hold, c) in links.into_iter().rev() {
                    let c = match acc {
                        None => c,
                        Some(rest) => mk(
                            TK::If(
                                Box::new(c),
                                Box::new(rest),
                                Box::new(mk(TK::Bool(false), Ty::Bool)),
                            ),
                            Ty::Bool,
                        ),
                    };
                    acc = Some(if hold.is_empty() {
                        c
                    } else {
                        mk(TK::Block(hold, Some(Box::new(c))), Ty::Bool)
                    });
                }
                acc.unwrap_or(prev)
            }
            ExprKind::Field { base, name }
                if name.text.starts_with(|c: char| c.is_ascii_digit()) =>
            {
                let name = ast::Name {
                    text: format!("_{}", name.text),
                    span: name.span,
                };
                self.member(base, &name, None, span, want)
            }
            ExprKind::Tuple(xs) if (2..=4).contains(&xs.len()) => {
                let callee = ast::Expr {
                    kind: ExprKind::Name(format!("Tuple{}", xs.len())),
                    span,
                };
                let args = xs
                    .iter()
                    .map(|x| ast::Arg {
                        name: None,
                        mutable: false,
                        value: x.clone(),
                    })
                    .collect();
                self.expr(
                    &ast::Expr {
                        kind: ExprKind::Call {
                            callee: Box::new(callee),
                            args,
                        },
                        span,
                    },
                    want,
                )
            }
            ExprKind::Field { base, name } => self.member(base, name, None, span, want),
            ExprKind::Index { base, indices } if indices.len() == 1 => {
                let b = self.expr(base, None);
                let i = self.expr(&indices[0], Some(&Ty::Int));
                let i = self.index_arg(&b.ty, i);
                self.method_call(b, "get", vec![TArg::Value(i)], span, false)
            }
            ExprKind::Call { callee, args } => match &callee.kind {
                ExprKind::Field { base, name } => self.member(base, name, Some(args), span, want),
                // `write` is `print` without the line end; the slice uses it to write `print` for containers in the library.
                ExprKind::Name(n) if (n == "print" || n == "write") && self.lookup(n).is_none() => {
                    if args.iter().any(|a| a.name.is_some()) {
                        self.err(span, "slice: named arguments to `print`");
                        return TExpr {
                            kind: TK::Error,
                            ty: Ty::Error,
                            span,
                        };
                    }
                    // Text and plain values gather into one `Print`; a value the library
                    // writes ends the one before it and is a call of its writer.
                    let mut stmts = Vec::new();
                    let mut pieces = Vec::new();
                    for (k, a) in args.iter().enumerate() {
                        if k > 0 {
                            pieces.push(Piece::Text(" ".into()));
                        }
                        match &a.value.kind {
                            ExprKind::Str(ps) => {
                                for p in ps {
                                    match p {
                                        StrPiece::Text(t) => pieces.push(Piece::Text(t.clone())),
                                        StrPiece::Expr(x) => {
                                            let v = self.expr(x, None);
                                            self.show_value(v, &mut stmts, &mut pieces);
                                        }
                                        StrPiece::Spec(..) => {
                                            self.err(span, "slice: no format specs")
                                        }
                                    }
                                }
                            }
                            _ => {
                                let v = self.expr(&a.value, None);
                                self.show_value(v, &mut stmts, &mut pieces);
                            }
                        }
                    }
                    if n == "print" {
                        pieces.push(Piece::Text("\n".into()));
                    }
                    let last = mk(TK::Print(pieces), Ty::Unit);
                    if stmts.is_empty() {
                        last
                    } else {
                        mk(TK::Block(stmts, Some(Box::new(last))), Ty::Unit)
                    }
                }
                ExprKind::Name(n) if n == "Char" && args.len() == 1 => {
                    let v = self.expr(&args[0].value, Some(&Ty::Int));
                    let v = self.coerce(v, &Ty::Int);
                    self.scopes.push(HashMap::new());
                    let tmp = self.bind("$code", Ty::Int);
                    self.scopes.pop();
                    let read = mk(TK::Var(tmp), Ty::Int);
                    let lo = mk(
                        TK::Cmp(
                            CmpOp::Le,
                            Box::new(mk(TK::Int(0), Ty::Int)),
                            Box::new(read.clone()),
                        ),
                        Ty::Bool,
                    );
                    let hi = mk(
                        TK::Cmp(
                            CmpOp::Le,
                            Box::new(read.clone()),
                            Box::new(mk(TK::Int(127), Ty::Int)),
                        ),
                        Ty::Bool,
                    );
                    let ok = mk(TK::Bin(BinOp::And, Box::new(lo), Box::new(hi)), Ty::Bool);
                    let check = TStmt::Assert(
                        ok,
                        "a `Char` is 0..=127".to_string(),
                        span,
                        CheckKind::RangeStore,
                    );
                    mk(
                        TK::Block(
                            vec![TStmt::Let(tmp, v), check],
                            Some(Box::new(mk(TK::CharOf(Box::new(read)), Ty::Char))),
                        ),
                        Ty::Char,
                    )
                }
                ExprKind::Name(n) if n == "Int" && args.len() == 1 => {
                    let v = self.expr(&args[0].value, None);
                    if self.shallow(&v.ty) == Ty::Char {
                        return mk(TK::CodeOf(Box::new(v)), Ty::Int);
                    }
                    if !matches!(
                        self.shallow(&v.ty),
                        Ty::Dom(_) | Ty::Param(_) | Ty::Int | Ty::Error
                    ) {
                        self.err(span, format!("`{}` has no position", self.show(&v.ty)));
                    }
                    mk(TK::DomOrd(Box::new(v)), Ty::Int)
                }
                ExprKind::Name(n)
                    if self.lookup(n).is_none()
                        && self.domain_named(n).is_some()
                        && args.len() == 1 =>
                {
                    let dty = self.domain_named(n).unwrap_or(Ty::Error);
                    let v = self.expr(&args[0].value, Some(&Ty::Int));
                    let v = self.coerce(v, &Ty::Int);
                    mk(TK::DomFrom(dty.clone(), Box::new(v)), dty)
                }
                ExprKind::Name(n) if n == "read" && self.lookup(n).is_none() => match want {
                    Some(w) => self.read_as(&w.clone(), span),
                    None => mk(TK::ReadInt, Ty::Int),
                },
                ExprKind::Generic { base, args: targs }
                    if targs.len() == 1
                        && args.is_empty()
                        && matches!(&base.kind, ExprKind::Name(n) if n == "read" && self.lookup(n).is_none()) =>
                {
                    let mut pre = Vec::new();
                    let t = self.body_ty(&targs[0], span, &mut pre);
                    let v = self.read_as(&t, span);
                    if pre.is_empty() {
                        v
                    } else {
                        let ty = v.ty.clone();
                        mk(TK::Block(pre, Some(Box::new(v))), ty)
                    }
                }
                ExprKind::Name(n) if self.declared(n) => self.call_named(n, &[], args, span, want),
                // `Box<Int>(5)`, `List<Int>()`: the written type arguments come first.
                ExprKind::Generic { base, args: targs } if matches!(&base.kind, ExprKind::Name(n) if self.declared(n)) => {
                    match &base.kind {
                        ExprKind::Name(n) => self.call_named(n, targs, args, span, want),
                        _ => {
                            let c = self.expr(callee, None);
                            self.call_value(c, args, span)
                        }
                    }
                }
                _ => {
                    let c = self.expr(callee, None);
                    self.call_value(c, args, span)
                }
            },
            ExprKind::Lambda {
                params,
                param_tys,
                body,
                ..
            } => {
                let expected = want.map(|w| self.shallow(w));
                let (eps, er) = match &expected {
                    Some(Ty::Fn(ps, r, _)) => (Some(ps.clone()), Some((**r).clone())),
                    _ => (None, None),
                };
                self.scopes.push(HashMap::new());
                let mut ids = Vec::new();
                let mut ptys = Vec::new();
                for (i, p) in params.iter().enumerate() {
                    let PatternKind::Name(name) = &p.kind else {
                        self.err(span, "slice: unsupported lambda parameter");
                        continue;
                    };
                    let ty = match (
                        param_tys.get(i).and_then(|t| t.as_ref()),
                        eps.as_ref().and_then(|ps| ps.get(i)),
                    ) {
                        (Some(t), _) => {
                            let mut errs = Vec::new();
                            let t = decls::body_type(
                                self.d,
                                self.module,
                                &self.generics,
                                &self.doms,
                                t,
                                &mut errs,
                            );
                            self.fresh_sets(&t)
                        }
                        (None, Some(t)) => t.clone(),
                        (None, None) => {
                            self.err(span, format!("the type of `{name}` is not known here; write `({name}: T) => ...`"));
                            Ty::Error
                        }
                    };
                    ptys.push(ty.clone());
                    match self.bind(name, ty) {
                        VarRef::Local(id) | VarRef::Global(id) => ids.push(id),
                    }
                }
                let saved_ret = self.ret.clone();
                // A lambda is not part of the generator or lender around it: no `yield` or `lend` inside.
                let saved_region = self.region.take();
                let saved_loops = std::mem::take(&mut self.loops);
                let ret = er.unwrap_or_else(|| self.new_ty());
                self.ret = ret.clone();
                let b = self.expr(body, Some(&ret));
                let b = self.coerce(b, &ret);
                self.ret = saved_ret;
                self.region = saved_region;
                self.loops = saved_loops;
                self.scopes.pop();
                let id = LambdaId(self.lambdas.len());
                self.lambdas.push(LambdaInfo {
                    params: ptys,
                    ret,
                    owner: self.decl,
                });
                let targs = (0..self.generics.len()).map(Ty::Param).collect();
                let sets = (0..self.n_sets).map(SetTy::Param).collect();
                mk(
                    TK::Lambda {
                        id,
                        params: ids,
                        body: Box::new(b),
                    },
                    Ty::Clo(Box::new(CloRef::Lambda(id, targs, sets))),
                )
            }
            ExprKind::List(xs) => {
                // `[a, b]` is `List.new()` and a `push` per element. The slice finds `List` by name; the design wants a marker.
                let Some(list) = self.d.core_struct("List") else {
                    self.err(span, "no `List` is declared");
                    return mk(TK::Error, Ty::Error);
                };
                let ls = self.d.s(list);
                let elem = match want.map(|w| self.shallow(w)) {
                    Some(Ty::Struct(d, ts, _)) if d == list => ts[0].clone(),
                    _ => self.new_ty(),
                };
                let lty = Ty::Struct(list, vec![elem.clone()], Vec::new());
                let Some(ctor) = ls.ctor else {
                    self.err(span, "`List` declares no `new`");
                    return mk(TK::Error, Ty::Error);
                };
                let new = self.fnref(ctor, vec![elem.clone()], Vec::new());
                let push = ls.methods["push"];
                self.scopes.push(HashMap::new());
                let tmp = self.bind("$list", lty.clone());
                let read = TExpr {
                    kind: TK::Var(tmp),
                    ty: lty.clone(),
                    span,
                };
                let mut stmts = vec![TStmt::Let(
                    tmp,
                    TExpr {
                        kind: TK::Call {
                            f: new,
                            recv: None,
                            args: vec![],
                        },
                        ty: lty.clone(),
                        span,
                    },
                )];
                for x in xs {
                    let v = self.expr(x, Some(&elem));
                    let v = self.coerce(v, &elem);
                    let f = self.fnref(push, vec![elem.clone()], Vec::new());
                    stmts.push(TStmt::Expr(TExpr {
                        kind: TK::Call {
                            f,
                            recv: Some(Box::new(TArg::Mut(read.clone()))),
                            args: vec![TArg::Value(v)],
                        },
                        ty: Ty::Unit,
                        span,
                    }));
                }
                self.scopes.pop();
                mk(TK::Block(stmts, Some(Box::new(read))), lty)
            }
            ExprKind::Comprehension { body, clauses }
                if clauses.len() == 1 && clauses[0].conds.is_empty() =>
            {
                // `[e for x in xs]` is `List.new()` and a `for` that pushes: the same statements a user would write.
                let Some(list) = self.d.core_struct("List") else {
                    self.err(span, "no `List` is declared");
                    return mk(TK::Error, Ty::Error);
                };
                let ls = self.d.s(list);
                let elem = match want.map(|w| self.shallow(w)) {
                    Some(Ty::Struct(d, ts, _)) if d == list => ts[0].clone(),
                    _ => self.new_ty(),
                };
                let lty = Ty::Struct(list, vec![elem.clone()], Vec::new());
                let Some(ctor) = ls.ctor else {
                    self.err(span, "`List` declares no `new`");
                    return mk(TK::Error, Ty::Error);
                };
                let new = self.fnref(ctor, vec![elem.clone()], Vec::new());
                self.scopes.push(HashMap::new());
                let tmp = self.bind("$comp", lty.clone());
                let mut stmts = vec![TStmt::Let(
                    tmp,
                    TExpr {
                        kind: TK::Call {
                            f: new,
                            recv: None,
                            args: vec![],
                        },
                        ty: lty.clone(),
                        span,
                    },
                )];
                let at = |kind| ast::Expr {
                    kind,
                    span: body.span,
                };
                let push = at(ExprKind::Call {
                    callee: Box::new(at(ExprKind::Field {
                        base: Box::new(at(ExprKind::Name("$comp".into()))),
                        name: ast::Name {
                            text: "push".into(),
                            span: body.span,
                        },
                    })),
                    args: vec![ast::Arg {
                        name: None,
                        mutable: false,
                        value: (**body).clone(),
                    }],
                });
                let pat = match &clauses[0].pat.kind {
                    PatternKind::Wildcard => ast::Pattern {
                        kind: PatternKind::Name("$each".into()),
                        span: clauses[0].pat.span,
                    },
                    _ => clauses[0].pat.clone(),
                };
                let each = ast::Stmt {
                    kind: StmtKind::For {
                        label: None,
                        pat,
                        iter: clauses[0].iter.clone(),
                        body: ast::Block {
                            stmts: vec![ast::Stmt {
                                kind: StmtKind::Expr(push),
                                span: body.span,
                                decorators: Vec::new(),
                                public: false,
                            }],
                            span: body.span,
                        },
                        else_body: None,
                    },
                    span,
                    decorators: Vec::new(),
                    public: false,
                };
                self.stmt(&each, &mut stmts);
                let read = TExpr {
                    kind: TK::Var(tmp),
                    ty: lty.clone(),
                    span,
                };
                self.scopes.pop();
                mk(TK::Block(stmts, Some(Box::new(read))), lty)
            }
            ExprKind::Block(b) => self.block(b, want),
            ExprKind::If {
                branches,
                otherwise,
            } => {
                let mut out: Option<TExpr> = None;
                let want_owned = want.cloned();
                let else_ = match otherwise {
                    Some(b) => self.block(b, want_owned.as_ref()),
                    None => mk(TK::Unit, Ty::Unit),
                };
                // With no expected type, the branches meet in a type of their own: each is converted into it.
                let ty = match (&want_owned, otherwise.is_some()) {
                    (_, false) => Ty::Unit,
                    (Some(w), true) => w.clone(),
                    (None, true) => self.join_ty(&else_.ty.clone()),
                };
                let was_joining = self.joining;
                self.joining = want_owned.is_none() && otherwise.is_some();
                let mut tail = if otherwise.is_some() && self.shallow(&ty) != Ty::Unit {
                    self.coerce(else_, &ty)
                } else {
                    else_
                };
                for (c, b) in branches.iter().rev() {
                    let c = self.cond(c);
                    let b = self.block(b, if otherwise.is_some() { Some(&ty) } else { None });
                    self.scopes.pop();
                    let b = if otherwise.is_some() && self.shallow(&ty) != Ty::Unit {
                        self.coerce(b, &ty)
                    } else {
                        b
                    };
                    tail = mk(TK::If(Box::new(c), Box::new(b), Box::new(tail)), ty.clone());
                    out = Some(tail.clone());
                }
                self.joining = was_joining;
                out.unwrap_or(tail)
            }
            ExprKind::Match { scrutinee, arms } => self.match_(scrutinee, arms, want, span),
            other => {
                self.err(
                    span,
                    format!("slice: expression {}", crate::kind_name(other)),
                );
                mk(TK::Error, Ty::Error)
            }
        }
    }
    /// A name that is a domain: one declared in a body, or a type parameter used as one.
    pub(super) fn domain_named(&self, n: &str) -> Option<Ty> {
        if let Some(d @ Ty::Dom(_)) = self.doms.get(n) {
            return Some(d.clone());
        }
        self.generics.iter().position(|g| g == n).map(Ty::Param)
    }
    pub(super) fn self_field(&self, n: &str) -> Option<usize> {
        let s = self.self_expr(Span::new(0, 0))?;
        match self.d.lookup_member(&s.ty, n) {
            Some(decls::Member::Field(i, _)) => Some(i),
            _ => None,
        }
    }

    /// A name the program declares, not a local or a field of `self`.
    fn declared(&self, n: &str) -> bool {
        self.lookup(n).is_none()
            && self.self_field(n).is_none()
            && self.d.named(self.module, n).is_some()
    }

    /// A call of a declared fn or struct, with the type arguments written at the call.
    fn call_named(
        &mut self,
        n: &str,
        written: &[ast::Type],
        args: &[ast::Arg],
        span: Span,
        want: Option<&Ty>,
    ) -> TExpr {
        let mk = |kind, ty| TExpr { kind, ty, span };
        let mut pre = Vec::new();
        let given: Vec<Ty> = written
            .iter()
            .map(|t| self.body_ty(t, span, &mut pre))
            .collect();
        let takes = match self.d.views[self.module.0][n] {
            Named::Fn(d) => self.d.f(d).generics.len(),
            Named::Struct(d) => self.d.s(d).generics.len(),
            _ => 0,
        };
        if given.len() > takes {
            self.err(
                span,
                format!("`{n}` takes {takes} type arguments, found {}", given.len()),
            );
        }
        let mut given = given;
        given.truncate(takes);
        let named = self.d.views[self.module.0][n];
        let ctor = match named {
            Named::Struct(d) => self.d.s(d).ctor.filter(|_| {
                !matches!(
                    self.decl.and_then(|f| self.d.f(f).owner),
                    Some(decls::Owner::Struct(o)) if o == d
                )
            }),
            _ => None,
        };
        let v = match (ctor, named) {
            // R02.6.3: outside its own body, `T(args)` calls `T`'s static `new` when it has one.
            (Some(new), _) => {
                let f = self.fnref(new, given, Vec::new());
                self.call_fn(f, None, args, span, want)
            }
            (None, Named::Fn(d)) => {
                let f = self.fnref(d, given, Vec::new());
                let fd = self.d.f(d);
                // A free fn's `self` is its first argument when called by name (R08.1.4).
                let first = args.first().filter(|a| a.name.is_none());
                match (fd.owner, fd.self_ty.clone(), first) {
                    (None, Some(st), Some(a)) if fd.self_mode != SelfMode::None => {
                        let sub = Subst {
                            tys: &f.targs,
                            sets: &f.sets,
                        };
                        let st = sub.ty(&st);
                        let st = self.fresh_sets(&st);
                        let e = self.expr(&a.value, Some(&st));
                        let e = self.coerce(e, &st);
                        let recv = self.recv_arg(d, e, span);
                        self.call_fn(f, Some(recv), &args[1..], span, want)
                    }
                    _ => self.call_fn(f, None, args, span, want),
                }
            }
            (None, Named::Struct(d)) => {
                let s = self.d.s(d);
                let fresh = s.generics.len() - given.len();
                let mut targs = given;
                targs.extend((0..fresh).map(|_| self.new_ty()));
                let sets: Vec<SetTy> = (0..s.n_sets).map(|_| self.new_set()).collect();
                self.construct(d, targs, sets, args, span)
            }
            _ => {
                self.err(span, format!("`{n}` cannot be called"));
                mk(TK::Error, Ty::Error)
            }
        };
        if pre.is_empty() {
            v
        } else {
            let ty = v.ty.clone();
            mk(TK::Block(pre, Some(Box::new(v))), ty)
        }
    }
}

/// A parameter as a call binds it.
pub(super) struct Formal<'d> {
    pub label: &'d str,
    pub ty: Ty,
    pub mutable: bool,
    pub named_only: bool,
    pub default: Option<(&'d ast::Expr, ModId)>,
}

/// Reading it has no effect, so when it runs does not matter.
fn pure(e: &TExpr) -> bool {
    match &e.kind {
        TK::Int(_) | TK::Bool(_) | TK::Unit | TK::Var(_) | TK::FnItem(_) | TK::Lambda { .. } => {
            true
        }
        TK::Field(b, _) => pure(b),
        _ => false,
    }
}

impl Cx<'_> {
    /// `S(args)`: the fields of struct `d` from `args`, given its type and set arguments.
    fn construct(
        &mut self,
        d: StructId,
        targs: Vec<Ty>,
        sets: Vec<SetTy>,
        args: &[ast::Arg],
        span: Span,
    ) -> TExpr {
        let s = self.d.s(d);
        let sub = Subst {
            tys: &targs,
            sets: &sets,
        };
        let formals: Vec<Formal> = (s.fields.iter().zip(&s.defaults))
            .map(|((n, t), e)| Formal {
                label: n,
                ty: sub.ty(t),
                mutable: false,
                named_only: false,
                default: e.as_ref().map(|e| (e, s.module)),
            })
            .collect();
        let (pre, fields) = self.args(args, &formals, span);
        let fields = (fields.into_iter())
            .map(|a| match a {
                TArg::Value(v) | TArg::Mut(v) => v,
            })
            .collect();
        let ty = Ty::Struct(d, targs, sets);
        block(
            pre,
            TExpr {
                kind: TK::Struct(fields),
                ty,
                span,
            },
        )
    }

    /// Variant `i` of enum `e`, its payload built from `args` (R02.7.1).
    fn variant(
        &mut self,
        e: StructId,
        i: usize,
        mut targs: Vec<Ty>,
        args: &[ast::Arg],
        span: Span,
    ) -> TExpr {
        let s = self.d.s(e);
        let fresh = s.generics.len().saturating_sub(targs.len());
        targs.extend((0..fresh).map(|_| self.new_ty()));
        let p = s.variants[i].1;
        if self.d.s(p).n_sets > 0 {
            self.err(span, "slice: a function value in a variant's payload");
        }
        let payload = self.construct(p, targs.clone(), Vec::new(), args, span);
        TExpr {
            kind: TK::Variant(i, Box::new(payload)),
            ty: Ty::Struct(e, targs, Vec::new()),
            span,
        }
    }

    /// `match x: arms`: a test of `x`'s variant per arm, with the payload bound in its arm, and every
    /// variant met by an arm or a default (R02.7.2, R02.7.4).
    fn match_(
        &mut self,
        scrut: &ast::Expr,
        arms: &[ast::Arm],
        want: Option<&Ty>,
        span: Span,
    ) -> TExpr {
        let mk = |kind, ty| TExpr { kind, ty, span };
        let x = self.expr(scrut, None);
        let xt = self.shallow(&x.ty);
        let (e, targs) = match &xt {
            Ty::Struct(e, targs, _) if !self.d.s(*e).variants.is_empty() => (*e, targs.clone()),
            _ => {
                self.err(span, "slice: a match on a value that is not an enum");
                return mk(TK::Error, Ty::Error);
            }
        };
        let variants = self.d.s(e).variants.clone();
        self.scopes.push(HashMap::new());
        let tmp = self.bind("$match", xt.clone());
        let stmts = vec![TStmt::Let(tmp, x)];
        let whole = || TExpr {
            kind: TK::Var(tmp),
            ty: xt.clone(),
            span,
        };
        let mut ty = want.cloned();
        let mut cases: Vec<Option<TExpr>> = vec![None; variants.len()];
        let mut default: Option<TExpr> = None;
        for arm in arms {
            if arm.guard.is_some() {
                self.err(arm.pat.span, "slice: a match guard");
            }
            self.scopes.push(HashMap::new());
            let mut binds = Vec::new();
            let which = match &arm.pat.kind {
                PatternKind::Wildcard => None,
                PatternKind::Name(v) if variants.iter().any(|(n, _)| n == v) => {
                    variants.iter().position(|(n, _)| n == v)
                }
                PatternKind::Name(b) => {
                    let l = self.bind(b, xt.clone());
                    binds.push(TStmt::Let(l, whole()));
                    None
                }
                PatternKind::Variant { path, args } => {
                    let v = &path[path.len() - 1].text;
                    match variants.iter().position(|(n, _)| n == v) {
                        Some(i) => {
                            let p = variants[i].1;
                            let fields = self.d.s(p).fields.clone();
                            if args.len() != fields.len() {
                                self.err(
                                    arm.pat.span,
                                    format!(
                                        "`{v}` has {} fields, found {}",
                                        fields.len(),
                                        args.len()
                                    ),
                                );
                            }
                            let sub = Subst {
                                tys: &targs,
                                sets: &[],
                            };
                            let payload = mk(
                                TK::Payload(Box::new(whole()), i),
                                Ty::Struct(p, targs.clone(), Vec::new()),
                            );
                            for (j, (a, (_, t))) in args.iter().zip(&fields).enumerate() {
                                match &a.kind {
                                    PatternKind::Wildcard => {}
                                    PatternKind::Name(b) => {
                                        let t = sub.ty(t);
                                        let l = self.bind(b, t.clone());
                                        let field = TK::Field(Box::new(payload.clone()), j);
                                        binds.push(TStmt::Let(l, mk(field, t)));
                                    }
                                    _ => self.err(a.span, "slice: a nested pattern"),
                                }
                            }
                            Some(i)
                        }
                        None => {
                            let en = self.d.s(e).name.clone();
                            self.err(arm.pat.span, format!("`{en}` has no variant `{v}`"));
                            None
                        }
                    }
                }
                _ => {
                    self.err(
                        arm.pat.span,
                        "slice: a pattern other than a variant or a name",
                    );
                    None
                }
            };
            let covered = which.map_or(default.is_some(), |i| cases[i].is_some());
            if default.is_some() || covered {
                self.err(
                    arm.pat.span,
                    format!(
                        "{}: this arm never runs: the arms before it match everything it does",
                        cpl_diag::codes::UNREACHABLE_ARM.0
                    ),
                );
            }
            let body = self.block(&arm.body, ty.as_ref());
            self.scopes.pop();
            let body = match &ty {
                Some(t) if self.shallow(t) != Ty::Unit => self.coerce(body, t),
                Some(_) => body,
                None => {
                    ty = Some(body.ty.clone());
                    body
                }
            };
            let bt = body.ty.clone();
            let body = mk(TK::Block(binds, Some(Box::new(body))), bt);
            match which {
                Some(i) if cases[i].is_none() => cases[i] = Some(body),
                Some(_) => {}
                None => default = default.or(Some(body)),
            }
        }
        let ty = ty.unwrap_or(Ty::Unit);
        let missing: Vec<&str> = (variants.iter().zip(&cases))
            .filter(|(_, c)| c.is_none())
            .map(|((n, _), _)| n.as_str())
            .collect();
        let mut tail = match (default, missing.is_empty()) {
            (Some(d), _) => d,
            (None, true) => match cases.pop() {
                Some(Some(last)) => last,
                _ => mk(TK::Error, ty.clone()),
            },
            (None, false) => {
                self.err(
                    span,
                    format!(
                        "{}: `match` misses {}",
                        cpl_diag::codes::NON_EXHAUSTIVE_MATCH.0,
                        missing.join(", ")
                    ),
                );
                mk(TK::Error, ty.clone())
            }
        };
        for (i, c) in cases.into_iter().enumerate().rev() {
            if let Some(c) = c {
                let tag = mk(TK::Tag(Box::new(whole())), Ty::Int);
                let test = mk(
                    TK::Cmp(
                        CmpOp::Eq,
                        Box::new(tag),
                        Box::new(mk(TK::Int(i as i64), Ty::Int)),
                    ),
                    Ty::Bool,
                );
                tail = mk(
                    TK::If(Box::new(test), Box::new(c), Box::new(tail)),
                    ty.clone(),
                );
            }
        }
        self.scopes.pop();
        mk(TK::Block(stmts, Some(Box::new(tail))), ty)
    }
}

fn block(pre: Vec<TStmt>, e: TExpr) -> TExpr {
    if pre.is_empty() {
        return e;
    }
    let (ty, span) = (e.ty.clone(), e.span);
    TExpr {
        kind: TK::Block(pre, Some(Box::new(e))),
        ty,
        span,
    }
}

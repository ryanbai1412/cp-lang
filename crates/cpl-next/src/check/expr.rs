//! Expressions: operators, calls, members, names.

use super::*;

impl<'a> Cx<'a> {
    pub(super) fn binary(&mut self, op: BinOp, l: TExpr, r: TExpr, span: Span) -> TExpr {
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
    pub(super) fn args(
        &mut self,
        args: &[ast::Arg],
        params: &[(Ty, bool)],
        span: Span,
    ) -> Vec<TArg> {
        if args.len() != params.len() {
            self.err(
                span,
                format!("expected {} arguments, found {}", params.len(), args.len()),
            );
        }
        args.iter()
            .zip(params)
            .map(|(a, (pty, pmut))| {
                let inner = match &a.value.kind {
                    ExprKind::Mut(e) => e,
                    _ => &a.value,
                };
                if *pmut {
                    if !a.mutable && !matches!(a.value.kind, ExprKind::Mut(_)) {
                        self.err(
                            a.value.span,
                            "a `mut` parameter is marked `mut` at the call",
                        );
                    }
                    let p = self.place(inner);
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
        let params: Vec<(Ty, bool)> = fd
            .params
            .iter()
            .map(|p| (sub.ty(&p.ty), p.mutable))
            .collect();
        let params: Vec<(Ty, bool)> = params
            .into_iter()
            .map(|(t, m)| (self.fresh_sets(&t), m))
            .collect();
        if fd.region.is_some() && !std::mem::take(&mut self.allow_region) {
            self.err(
                span,
                format!(
                    "`{}` yields or lends: use it in a `for` or as the target of an assignment",
                    fd.name
                ),
            );
        }
        let args = self.args(args, &params, span);
        TExpr {
            kind: TK::Call {
                f,
                recv: recv.map(Box::new),
                args,
            },
            ty,
            span,
        }
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
        let recv = if mutates || fd.self_mode == SelfMode::Mut {
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
        // `Type.f(args)`: a fn of the type with no receiver.
        if let ExprKind::Name(n) = &base.kind {
            if let (Some(d), true) = (self.d.struct_named(n), self.lookup(n).is_none()) {
                if let (s, Some(args)) = (self.d.s(d), call) {
                    let Some(m) = s.methods.get(&name.text).copied() else {
                        self.err(span, format!("`{n}` has no `{}`", name.text));
                        return TExpr {
                            kind: TK::Error,
                            ty: Ty::Error,
                            span,
                        };
                    };
                    let f = self.fnref(m, Vec::new(), Vec::new());
                    return self.call_fn(f, None, args, span, want);
                }
            }
        }
        let recv = self.expr(base, None);
        let rt = self.shallow(&recv.ty);
        // A method of a bound on a type parameter.
        if let (Ty::Param(i), Some(decl)) = (&rt, self.decl) {
            for (pi, tr) in &self.d.f(decl).bounds {
                let t = &self.d.traits[tr.0];
                if pi != i {
                    continue;
                }
                if let Some(m) = t.methods.get(&name.text).copied() {
                    let fd = self.d.f(m);
                    let params: Vec<(Ty, bool)> = fd
                        .params
                        .iter()
                        .map(|p| (p.ty.clone(), p.mutable))
                        .collect();
                    let args = self.args(call.unwrap_or(&[]), &params, span);
                    let recv = if fd.self_mode == SelfMode::Mut {
                        TArg::Mut(recv)
                    } else {
                        TArg::Value(recv)
                    };
                    return TExpr {
                        kind: TK::CallTrait {
                            tr: *tr,
                            method: name.text.clone(),
                            recv: Box::new(recv),
                            args,
                        },
                        ty: fd.ret.clone(),
                        span,
                    };
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
                let recv = if fd.self_mode == SelfMode::Mut {
                    if !matches!(recv.kind, TK::Var(_) | TK::Field(..)) {
                        self.err(span, "a `mut self` method needs a place");
                    }
                    TArg::Mut(recv)
                } else {
                    TArg::Value(recv)
                };
                self.call_fn(f, Some(recv), call.unwrap_or(&[]), span, want)
            }
            None => {
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
        let params: Vec<(Ty, bool)> = ps.into_iter().map(|p| (p, false)).collect();
        let args = self.args(args, &params, span);
        let args = args
            .into_iter()
            .map(|a| match a {
                TArg::Value(v) | TArg::Mut(v) => v,
            })
            .collect();
        TExpr {
            kind: TK::CallValue {
                callee: Box::new(callee),
                args,
            },
            ty: r,
            span,
        }
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
        if let Some(d) = self.d.fn_named(n) {
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
            ExprKind::Paren(inner) => self.expr(inner, want),
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
                // `a <= b < c` is `a <= b and b < c`; the slice re-evaluates `b`, so it must be simple.
                let mut prev = self.expr(first, None);
                let mut acc: Option<TExpr> = None;
                for (op, rhs) in rest {
                    let r = self.expr(rhs, Some(&prev.ty));
                    let r = self.coerce(r, &prev.ty.clone());
                    if !matches!(
                        self.shallow(&prev.ty),
                        Ty::Int | Ty::Bool | Ty::Dom(_) | Ty::Param(_) | Ty::Error
                    ) || matches!(op, CmpOp::In | CmpOp::NotIn)
                    {
                        self.err(span, "slice: comparison of this type is not supported");
                    }
                    let c = mk(TK::Cmp(*op, Box::new(prev), Box::new(r.clone())), Ty::Bool);
                    acc = Some(match acc {
                        None => c,
                        Some(a) => mk(TK::Bin(BinOp::And, Box::new(a), Box::new(c)), Ty::Bool),
                    });
                    prev = r;
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
            ExprKind::Call { args, .. } if args.iter().any(|a| a.name.is_some()) => {
                self.err(span, "slice: named arguments");
                TExpr {
                    kind: TK::Error,
                    ty: Ty::Error,
                    span,
                }
            }
            ExprKind::Call { callee, args } => match &callee.kind {
                ExprKind::Field { base, name } => self.member(base, name, Some(args), span, want),
                // `write` is `print` without the line end; the slice uses it to write `print` for containers in the library.
                ExprKind::Name(n) if (n == "print" || n == "write") && self.lookup(n).is_none() => {
                    let newline = n == "print";
                    let mut pieces = Vec::new();
                    match args.first().map(|a| &a.value.kind) {
                        Some(ExprKind::Str(ps)) => {
                            for p in ps {
                                match p {
                                    StrPiece::Text(t) => pieces.push(Piece::Text(t.clone())),
                                    StrPiece::Expr(x) => {
                                        let v = self.expr(x, None);
                                        if !matches!(
                                            self.shallow(&v.ty),
                                            Ty::Int | Ty::Bool | Ty::Error
                                        ) {
                                            self.err(x.span, "slice: only Int and Bool print");
                                        }
                                        pieces.push(Piece::Value(v));
                                    }
                                    StrPiece::Spec(..) => self.err(span, "slice: no format specs"),
                                }
                            }
                        }
                        Some(_) => {
                            for (k, a) in args.iter().enumerate() {
                                let v = self.expr(&a.value, None);
                                // A list or array prints through the library's own fn.
                                let via = match self.shallow(&v.ty) {
                                    Ty::Struct(d, ..) if Some(d) == self.d.struct_named("List") => {
                                        Some("write_list")
                                    }
                                    Ty::Struct(d, ..)
                                        if Some(d) == self.d.struct_named("Array") =>
                                    {
                                        Some("write_array")
                                    }
                                    _ => None,
                                };
                                if let (Some(via), 1) =
                                    (via.and_then(|n| self.d.fn_named(n)), args.len())
                                {
                                    let f = self.fnref(via, Vec::new(), Vec::new());
                                    let fd = self.d.f(f.decl);
                                    let pt = Subst {
                                        tys: &f.targs,
                                        sets: &f.sets,
                                    }
                                    .ty(&fd.params[0].ty);
                                    let v = self.coerce(v, &pt);
                                    let call = TExpr {
                                        kind: TK::Call {
                                            f,
                                            recv: None,
                                            args: vec![TArg::Value(v)],
                                        },
                                        ty: Ty::Unit,
                                        span,
                                    };
                                    let end = TExpr {
                                        kind: TK::Print(if newline {
                                            vec![Piece::Text("\n".into())]
                                        } else {
                                            vec![]
                                        }),
                                        ty: Ty::Unit,
                                        span,
                                    };
                                    return mk(
                                        TK::Block(vec![TStmt::Expr(call)], Some(Box::new(end))),
                                        Ty::Unit,
                                    );
                                }
                                if !matches!(self.shallow(&v.ty), Ty::Int | Ty::Bool | Ty::Error) {
                                    self.err(a.value.span, "slice: only Int and Bool print");
                                }
                                if k > 0 {
                                    pieces.push(Piece::Text(" ".into()));
                                }
                                pieces.push(Piece::Value(v));
                            }
                        }
                        None => {}
                    }
                    if newline {
                        pieces.push(Piece::Text("\n".into()));
                    }
                    mk(TK::Print(pieces), Ty::Unit)
                }
                ExprKind::Name(n) if n == "Int" && args.len() == 1 => {
                    let v = self.expr(&args[0].value, None);
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
                ExprKind::Name(n)
                    if self.lookup(n).is_none()
                        && self.self_field(n).is_none()
                        && self.d.names.contains_key(n) =>
                {
                    match self.d.names[n] {
                        Named::Fn(d) => {
                            let f = self.fnref(d, Vec::new(), Vec::new());
                            self.call_fn(f, None, args, span, want)
                        }
                        Named::Struct(d) => {
                            let s = self.d.s(d);
                            let targs: Vec<Ty> = s.generics.iter().map(|_| self.new_ty()).collect();
                            let sets: Vec<SetTy> = (0..s.n_sets).map(|_| self.new_set()).collect();
                            let sub = Subst {
                                tys: &targs,
                                sets: &sets,
                            };
                            let params: Vec<(Ty, bool)> =
                                s.fields.iter().map(|(_, t)| (sub.ty(t), false)).collect();
                            let fields = self
                                .args(args, &params, span)
                                .into_iter()
                                .map(|a| match a {
                                    TArg::Value(v) | TArg::Mut(v) => v,
                                })
                                .collect();
                            mk(TK::Struct(fields), Ty::Struct(d, targs, sets))
                        }
                        _ => {
                            self.err(span, format!("`{n}` cannot be called"));
                            mk(TK::Error, Ty::Error)
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
                            let t =
                                decls::body_type(self.d, &self.generics, &self.doms, t, &mut errs);
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
                let Some(list) = self.d.struct_named("List") else {
                    self.err(span, "no `List` is declared");
                    return mk(TK::Error, Ty::Error);
                };
                let ls = self.d.s(list);
                let elem = match want.map(|w| self.shallow(w)) {
                    Some(Ty::Struct(d, ts, _)) if d == list => ts[0].clone(),
                    _ => self.new_ty(),
                };
                let lty = Ty::Struct(list, vec![elem.clone()], Vec::new());
                let new = self.fnref(ls.methods["new"], vec![elem.clone()], Vec::new());
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
                let Some(list) = self.d.struct_named("List") else {
                    self.err(span, "no `List` is declared");
                    return mk(TK::Error, Ty::Error);
                };
                let ls = self.d.s(list);
                let elem = match want.map(|w| self.shallow(w)) {
                    Some(Ty::Struct(d, ts, _)) if d == list => ts[0].clone(),
                    _ => self.new_ty(),
                };
                let lty = Ty::Struct(list, vec![elem.clone()], Vec::new());
                let new = self.fnref(ls.methods["new"], vec![elem.clone()], Vec::new());
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
            _ => {
                self.err(span, "slice: unsupported expression");
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
}

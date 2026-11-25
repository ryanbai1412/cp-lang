//! Statements, scopes, domains, ranges and reads.

use super::*;

impl<'a> Cx<'a> {
    /// The innermost scope.
    pub(super) fn scope(&mut self) -> &mut HashMap<String, usize> {
        if self.scopes.is_empty() {
            self.scopes.push(HashMap::new());
        }
        let last = self.scopes.len() - 1;
        &mut self.scopes[last]
    }
    pub(super) fn add_local(&mut self, name: &str, ty: Ty) -> LocalId {
        self.locals.push(Local {
            name: name.to_string(),
            ty,
            top: false,
            mutable: false,
            fixed: false,
        });
        let id = self.locals.len() - 1;
        self.scope().insert(name.to_string(), id);
        id
    }
    /// A new binding: a global at top level, a local in a fn.
    pub(super) fn bind(&mut self, name: &str, ty: Ty) -> VarRef {
        if self.decl.is_none() && self.scopes.len() == 1 {
            self.globals.push(Local {
                name: name.to_string(),
                ty,
                top: true,
                mutable: false,
                fixed: false,
            });
            self.global_names
                .insert(name.to_string(), self.globals.len() - 1);
            VarRef::Global(self.globals.len() - 1)
        } else if self.decl.is_none() {
            // A binding inside a top-level block lives in the globals too, under its scope.
            self.globals.push(Local {
                name: name.to_string(),
                ty,
                top: false,
                mutable: false,
                fixed: false,
            });
            let id = self.globals.len() - 1;
            self.scope().insert(name.to_string(), id);
            VarRef::Global(id)
        } else {
            VarRef::Local(self.add_local(name, ty))
        }
    }
    pub(super) fn lookup(&self, name: &str) -> Option<(VarRef, Ty)> {
        for (depth, s) in self.scopes.iter().enumerate().rev() {
            if let Some(id) = s.get(name) {
                return Some(if self.decl.is_none() {
                    let _ = depth;
                    (VarRef::Global(*id), self.globals[*id].ty.clone())
                } else {
                    (VarRef::Local(*id), self.locals[*id].ty.clone())
                });
            }
        }
        // A field of `self` is nearer than a top-level binding.
        if self.self_field(name).is_some() {
            return None;
        }
        self.global_names
            .get(name)
            .map(|id| (VarRef::Global(*id), self.globals[*id].ty.clone()))
    }
    pub(super) fn self_expr(&self, span: Span) -> Option<TExpr> {
        let id = self.self_local?;
        Some(TExpr {
            kind: TK::Var(VarRef::Local(id)),
            ty: self.locals[id].ty.clone(),
            span,
        })
    }

    pub(super) fn block(&mut self, b: &ast::Block, want: Option<&Ty>) -> TExpr {
        self.scopes.push(HashMap::new());
        let mut out = Vec::new();
        let mut value = None;
        for (i, s) in b.stmts.iter().enumerate() {
            if i + 1 == b.stmts.len() {
                if let StmtKind::Expr(e) = &s.kind {
                    let v = self.expr(e, want);
                    value = Some(match want {
                        Some(w) if self.shallow(w) != Ty::Unit => self.coerce(v, w),
                        _ => v,
                    });
                    break;
                }
            }
            self.stmt(s, &mut out);
        }
        self.scopes.pop();
        let ty = value
            .as_ref()
            .map(|v: &TExpr| v.ty.clone())
            .unwrap_or(Ty::Unit);
        TExpr {
            kind: TK::Block(out, value.map(Box::new)),
            ty,
            span: b.span,
        }
    }
    /// A condition, in a scope of its own that holds the names its `let`s bind: the caller
    /// checks what the condition guards, then pops the scope.
    pub(super) fn cond(&mut self, c: &Cond) -> TExpr {
        self.scopes.push(HashMap::new());
        self.cond_in(c)
    }
    fn cond_in(&mut self, c: &Cond) -> TExpr {
        match c {
            // `let v = e`: holds when the optional `e` has a value, which `v` then names.
            Cond::Let {
                pat:
                    ast::Pattern {
                        kind: ast::PatternKind::Name(name),
                        ..
                    },
                value,
                and,
            } => {
                let span = value.span;
                let v = self.expr(value, None);
                let Some(inner) = self.optional_of(&v.ty) else {
                    if self.shallow(&v.ty) != Ty::Error {
                        let t = self.show(&v.ty);
                        self.err(span, format!("slice: `if let` on a `{t}`"));
                    }
                    return TExpr {
                        kind: TK::Error,
                        ty: Ty::Error,
                        span,
                    };
                };
                let (hold, flag, val) = self.opened(v, inner.clone());
                let named = self.bind(name, inner);
                let rest = match and {
                    Some(more) => self.cond_in(more),
                    None => TExpr {
                        kind: TK::Bool(true),
                        ty: Ty::Bool,
                        span,
                    },
                };
                let then = TExpr {
                    kind: TK::Block(vec![TStmt::Let(named, val)], Some(Box::new(rest))),
                    ty: Ty::Bool,
                    span,
                };
                let no = TExpr {
                    kind: TK::Bool(false),
                    ty: Ty::Bool,
                    span,
                };
                TExpr {
                    kind: TK::Block(
                        hold,
                        Some(Box::new(TExpr {
                            kind: TK::If(Box::new(flag), Box::new(then), Box::new(no)),
                            ty: Ty::Bool,
                            span,
                        })),
                    ),
                    ty: Ty::Bool,
                    span,
                }
            }
            Cond::Expr(e) => {
                let v = self.expr(e, Some(&Ty::Bool));
                self.coerce(v, &Ty::Bool)
            }
            _ => {
                self.errors.push("slice: unsupported condition".into());
                TExpr {
                    kind: TK::Error,
                    ty: Ty::Bool,
                    span: Span::new(0, 0),
                }
            }
        }
    }
    pub(super) fn stmt(&mut self, s: &ast::Stmt, out: &mut Vec<TStmt>) {
        match &s.kind {
            StmtKind::Let {
                mutable,
                pat,
                ty,
                init: Some(init),
                otherwise: None,
                ..
            } => {
                if let PatternKind::Tuple(ps) = &pat.kind {
                    // `let (a, b): (T, U) = read()` reads each part in order. Tuples as values are not in the slice.
                    let (tys, call) = match (ty.as_ref().map(|t| &t.kind), Self::read_call(init)) {
                        (Some(TypeKind::Tuple(ts)), Some((None, call))) if ts.len() == ps.len() => {
                            (ts.clone(), call)
                        }
                        (
                            None,
                            Some((
                                Some(ast::Type {
                                    kind: TypeKind::Tuple(ts),
                                    ..
                                }),
                                call,
                            )),
                        ) if ts.len() == ps.len() => (ts, call),
                        _ => {
                            // Any other tuple pattern binds the whole value, then each part by its field.
                            let tmp = format!("$tuple{}", pat.span.start);
                            let mut whole = s.clone();
                            if let StmtKind::Let { pat, mutable, .. } = &mut whole.kind {
                                *pat = ast::Pattern {
                                    kind: PatternKind::Name(tmp.clone()),
                                    span: pat.span,
                                };
                                *mutable = false;
                            }
                            self.stmt(&whole, out);
                            for (k, p) in ps.iter().enumerate() {
                                if matches!(p.kind, PatternKind::Wildcard) {
                                    continue;
                                }
                                let at = |kind| ast::Expr { kind, span: p.span };
                                let part = at(ExprKind::Field {
                                    base: Box::new(at(ExprKind::Name(tmp.clone()))),
                                    name: ast::Name {
                                        text: k.to_string(),
                                        span: p.span,
                                    },
                                });
                                let mut one = s.clone();
                                if let StmtKind::Let { pat, ty, init, .. } = &mut one.kind {
                                    *pat = p.clone();
                                    *ty = None;
                                    *init = Some(part);
                                }
                                self.stmt(&one, out);
                            }
                            return;
                        }
                    };
                    for (p, t) in ps.iter().zip(tys) {
                        let mut one = s.clone();
                        if let StmtKind::Let { pat, ty, init, .. } = &mut one.kind {
                            *pat = p.clone();
                            *ty = Some(t);
                            *init = Some(call.clone());
                        }
                        self.stmt(&one, out);
                    }
                    return;
                }
                let PatternKind::Name(name) = &pat.kind else {
                    return self.err(s.span, "slice: unsupported pattern");
                };
                let declared = ty.as_ref().map(|t| self.body_ty(t, s.span, out));
                let v = self.expr(init, declared.as_ref());
                // `var a: [V]Int = 0` fills every slot: it is `Array.filled(0)`.
                let fill = match (&declared, self.d.core_struct("Array")) {
                    (Some(array @ Ty::Struct(d, ts, _)), Some(arr))
                        if *d == arr && !matches!(self.shallow(&v.ty), Ty::Struct(..)) =>
                    {
                        Some((arr, ts.clone(), array.clone()))
                    }
                    _ => None,
                };
                let v = match fill {
                    Some((arr, ts, array)) => {
                        let elem = ts[1].clone();
                        let v = self.coerce(v, &elem);
                        let f = self.fnref(self.d.s(arr).methods["filled"], ts.clone(), Vec::new());
                        TExpr {
                            kind: TK::Call {
                                f,
                                recv: None,
                                args: vec![TArg::Value(v)],
                            },
                            ty: array,
                            span: s.span,
                        }
                    }
                    None => v,
                };
                let v = match &declared {
                    Some(t) => self.coerce(v, t),
                    // A `var` holding a function can be reassigned, so it gets a set; a `let` keeps the closure's own type.
                    None if *mutable => match self.shallow(&v.ty) {
                        Ty::Clo(c) => {
                            let (ps, r) = self.clo_sig(&c);
                            let slot = Ty::Fn(ps, Box::new(r), self.new_set());
                            self.coerce(v, &slot)
                        }
                        Ty::Fn(ps, r, set) if !matches!(self.shallow_set(&set), SetTy::Var(_)) => {
                            let slot = Ty::Fn(ps, r, self.new_set());
                            self.coerce(v, &slot)
                        }
                        _ => v,
                    },
                    None => v,
                };
                let var = self.bind(name, v.ty.clone());
                match var {
                    VarRef::Local(i) => {
                        (self.locals[i].mutable, self.locals[i].fixed) = (*mutable, !*mutable)
                    }
                    VarRef::Global(i) => {
                        (self.globals[i].mutable, self.globals[i].fixed) = (*mutable, !*mutable)
                    }
                }
                out.push(TStmt::Let(var, v));
            }
            StmtKind::Assign { target, op, value } => {
                // `a[i] = v` and `a[i] op= v` go through the type's `get` and `set`.
                if let ExprKind::Index { base, indices } = &target.kind {
                    let base_t = self.place(base);
                    let idx = self.expr(&indices[0], Some(&Ty::Int));
                    let mut idx = self.index_arg(&base_t.ty, idx);
                    let rhs = match op {
                        AssignOp::Set => self.expr(value, None),
                        // The index once, then the value, then one read and one write.
                        AssignOp::Binary(b) => {
                            let tmp = self.bind("$idx", idx.ty.clone());
                            let ity = idx.ty.clone();
                            out.push(TStmt::Let(
                                tmp,
                                std::mem::replace(
                                    &mut idx,
                                    TExpr {
                                        kind: TK::Var(tmp),
                                        ty: ity,
                                        span: target.span,
                                    },
                                ),
                            ));
                            let v = self.expr(value, Some(&Ty::Int));
                            let vt = self.bind("$rhs", v.ty.clone());
                            let vty = v.ty.clone();
                            out.push(TStmt::Let(vt, v));
                            let v = TExpr {
                                kind: TK::Var(vt),
                                ty: vty,
                                span: value.span,
                            };
                            let cur = self.method_call(
                                base_t.clone(),
                                "get",
                                vec![TArg::Value(idx.clone())],
                                target.span,
                                false,
                            );
                            self.binary(*b, cur, v, target.span)
                        }
                        _ => return self.err(s.span, "slice: unsupported assignment"),
                    };
                    let call = self.method_call(
                        base_t,
                        "set",
                        vec![TArg::Value(idx), TArg::Value(rhs)],
                        target.span,
                        true,
                    );
                    out.push(TStmt::Expr(call));
                    return;
                }
                if let ExprKind::Call { .. } = &target.kind {
                    // `t.at(i) = v`: `at` lends a place; the assignment runs while it is lent.
                    self.allow_region = true;
                    let call = self.expr(target, None);
                    self.allow_region = false;
                    let TK::Call { f, recv, args } = call.kind else {
                        return;
                    };
                    let Some((true, elem)) = self.d.f(f.decl).region.clone() else {
                        return self.err(
                            s.span,
                            "this call does not lend a place, so it cannot be assigned",
                        );
                    };
                    let elem = Subst {
                        tys: &f.targs,
                        sets: &f.sets,
                    }
                    .ty(&elem);
                    self.scopes.push(HashMap::new());
                    let var = self.bind("$lent", elem.clone());
                    let lent = TExpr {
                        kind: TK::Var(var),
                        ty: elem.clone(),
                        span: target.span,
                    };
                    let v = self.expr(value, Some(&elem));
                    let v = match op {
                        AssignOp::Set => v,
                        AssignOp::Binary(b) => self.binary(*b, lent.clone(), v, target.span),
                        _ => return self.err(s.span, "slice: unsupported assignment"),
                    };
                    let v = self.coerce(v, &elem);
                    self.scopes.pop();
                    let body = TExpr {
                        kind: TK::Block(vec![TStmt::Assign(lent, v)], None),
                        ty: Ty::Unit,
                        span: s.span,
                    };
                    out.push(TStmt::Region {
                        f,
                        recv,
                        args,
                        var,
                        lend: true,
                        body,
                    });
                    return;
                }
                let place = self.place(target);
                let mut root = &place;
                let mut cache = false;
                while let TK::Field(b, i) = &root.kind {
                    if let Ty::Struct(sid, ..) = self.shallow(&b.ty) {
                        {
                            let sd = self.d.s(sid);
                            cache |= sd.cached.get(*i).copied().unwrap_or(false);
                            if !cache && !sd.var_fields.get(*i).copied().unwrap_or(true) {
                                let (st, f) = (sd.name.clone(), sd.fields[*i].0.clone());
                                self.err(target.span, format!("{}: field `{st}.{f}` is fixed (not `var`) and cannot be assigned", cpl_diag::codes::ASSIGN_FIXED_FIELD.0));
                            }
                        }
                    }
                    root = b;
                }
                if let Some(name) = self.let_root(&place).filter(|_| !cache) {
                    self.err(target.span, format!("{}: `{name}` is bound by `let` and cannot be assigned; declare it with `var`", cpl_diag::codes::ASSIGN_TO_LET.0));
                }
                let v = match op {
                    AssignOp::Set => self.expr(value, Some(&place.ty)),
                    AssignOp::Binary(b) => {
                        let v = self.expr(value, Some(&place.ty));
                        self.binary(*b, place.clone(), v, target.span)
                    }
                    _ => return self.err(s.span, "slice: unsupported assignment"),
                };
                let v = self.coerce(v, &place.ty);
                out.push(TStmt::Assign(place, v));
            }
            StmtKind::Expr(e) => {
                let v = self.expr(e, None);
                out.push(TStmt::Expr(v));
            }
            StmtKind::While {
                cond,
                body,
                else_body: None,
            } => {
                let c = self.cond(cond);
                self.loops += 1;
                let b = self.block(body, None);
                self.loops -= 1;
                self.scopes.pop();
                out.push(TStmt::While(c, b));
            }
            StmtKind::For { .. } if !self.in_for => {
                self.loops += 1;
                self.in_for = true;
                self.stmt(s, out);
                self.in_for = false;
                self.loops -= 1;
            }
            StmtKind::For {
                pat,
                iter,
                body,
                else_body: None,
                ..
            } => {
                self.in_for = false;
                if let (PatternKind::Name(name), ExprKind::Name(dn)) = (&pat.kind, &iter.kind) {
                    if let Some(dty) = self.domain_named(dn) {
                        let lo = TExpr {
                            kind: TK::Int(0),
                            ty: Ty::Int,
                            span: s.span,
                        };
                        let hi = TExpr {
                            kind: TK::DomLen(dty.clone()),
                            ty: Ty::Int,
                            span: s.span,
                        };
                        self.scopes.push(HashMap::new());
                        let var = self.bind(name, dty);
                        let b = self.block(body, None);
                        self.scopes.pop();
                        out.push(TStmt::For(var, lo, hi, b));
                        return;
                    }
                }
                if let PatternKind::Tuple(_) = &pat.kind {
                    // `for (a, b) in xs` binds the item, then `let (a, b) = item` opens the body.
                    let tmp = format!("$item{}", s.span.start);
                    let mut one = s.clone();
                    if let StmtKind::For {
                        pat: p, body: b, ..
                    } = &mut one.kind
                    {
                        let open = ast::Stmt {
                            kind: StmtKind::Let {
                                mutable: false,
                                pat: pat.clone(),
                                ty: None,
                                init: Some(ast::Expr {
                                    kind: ExprKind::Name(tmp.clone()),
                                    span: pat.span,
                                }),
                                otherwise: None,
                                guard: None,
                            },
                            span: pat.span,
                            decorators: Vec::new(),
                            public: false,
                        };
                        *p = ast::Pattern {
                            kind: PatternKind::Name(tmp),
                            span: pat.span,
                        };
                        b.stmts.insert(0, open);
                    }
                    return self.stmt(&one, out);
                }
                if let (PatternKind::Name(name), false) =
                    (&pat.kind, matches!(iter.kind, ExprKind::Range { .. }))
                {
                    // `for x in gen(args)`, or `for x in xs` through the type's own `each`.
                    self.allow_region = matches!(iter.kind, ExprKind::Call { .. });
                    let mut call = self.expr(iter, None);
                    self.allow_region = false;
                    if !matches!(&call.kind, TK::Call { f, .. } if self.d.f(f.decl).region.is_some())
                    {
                        self.allow_region = true;
                        call = self.method_call(call, "each", vec![], s.span, false);
                        self.allow_region = false;
                    }
                    let TK::Call { f, recv, args } = call.kind else {
                        return;
                    };
                    let Some((false, elem)) = self.d.f(f.decl).region.clone() else {
                        return self.err(s.span, "this cannot be walked with `for`");
                    };
                    let elem = Subst {
                        tys: &f.targs,
                        sets: &f.sets,
                    }
                    .ty(&elem);
                    self.scopes.push(HashMap::new());
                    let var = self.bind(name, elem);
                    let body = self.block(body, None);
                    self.scopes.pop();
                    out.push(TStmt::Region {
                        f,
                        recv,
                        args,
                        var,
                        lend: false,
                        body,
                    });
                    return;
                }
                let (
                    PatternKind::Name(name),
                    ExprKind::Range {
                        lo: Some(lo),
                        hi: Some(hi),
                        inclusive,
                        step: None,
                    },
                ) = (&pat.kind, &iter.kind)
                else {
                    return self.err(s.span, "slice: `for` takes `name in lo..hi`");
                };
                // `lo..=hi` is `lo..hi+1` here, so it fails at the largest Int where the language does not.
                let hi1;
                let hi = if *inclusive {
                    hi1 = Self::plus_one(hi);
                    &hi1
                } else {
                    hi
                };
                let lo = self.expr(lo, Some(&Ty::Int));
                let lo = self.coerce(lo, &Ty::Int);
                let hi = self.expr(hi, Some(&Ty::Int));
                let hi = self.coerce(hi, &Ty::Int);
                self.scopes.push(HashMap::new());
                let var = self.bind(name, Ty::Int);
                let b = self.block(body, None);
                self.scopes.pop();
                out.push(TStmt::For(var, lo, hi, b));
            }
            StmtKind::Return(e) => {
                let ret = self.ret.clone();
                let v = e.as_ref().map(|e| {
                    let v = self.expr(e, Some(&ret));
                    self.coerce(v, &ret)
                });
                out.push(TStmt::Return(v));
            }
            StmtKind::Assert { cond, message } => {
                let c = self.expr(cond, Some(&Ty::Bool));
                let c = self.coerce(c, &Ty::Bool);
                let msg = match message.as_ref().map(|m| &m.kind) {
                    Some(ExprKind::Str(ps)) => ps
                        .iter()
                        .map(|p| {
                            if let StrPiece::Text(t) = p {
                                t.clone()
                            } else {
                                String::new()
                            }
                        })
                        .collect(),
                    _ => "assertion failed".to_string(),
                };
                out.push(TStmt::Assert(c, msg, s.span, CheckKind::Assert));
            }
            StmtKind::Domain { name, def } => {
                let ExprKind::Range {
                    lo: Some(lo),
                    hi: Some(hi),
                    inclusive,
                    step: None,
                } = &def.kind
                else {
                    return self.err(
                        s.span,
                        format!("slice: a domain that is {}", crate::kind_name(&def.kind)),
                    );
                };
                let (lo, end, within) = self.domain_ends(lo, hi, *inclusive);
                self.declare_domain(&name.text, lo, end, within, s.span, out);
            }
            StmtKind::Yield { value, from: false } => match self.region.clone() {
                Some((false, t)) => {
                    let v = self.expr(value, Some(&t));
                    let v = self.coerce(v, &t);
                    out.push(TStmt::Yield(v));
                }
                _ => self.err(s.span, "`yield` outside a fn that `yields`"),
            },
            StmtKind::Lend { place, .. } => match self.region.clone() {
                // Every path through a lender reaches one `lend`; inside a loop it could run twice.
                Some((true, _)) if self.loops > 0 => self.err(
                    s.span,
                    "`lend` inside a loop: a lender lends one place once",
                ),
                Some((true, t)) => {
                    let p = self.place(place);
                    if !self.unify(&p.ty, &t) {
                        self.err(
                            s.span,
                            format!("expected `{}`, found `{}`", self.show(&t), self.show(&p.ty)),
                        );
                    }
                    out.push(TStmt::LendOut(p));
                }
                _ => self.err(s.span, "`lend` outside a fn that `lends`"),
            },
            StmtKind::Break(None) => out.push(TStmt::Break),
            StmtKind::Continue(None) => out.push(TStmt::Continue),
            other => self.err(
                s.span,
                format!("slice: statement {}", crate::kind_name(other)),
            ),
        }
    }
    /// `read()` or `read<T>()`: the type argument if written, and the bare `read()` call.
    pub(super) fn read_call(e: &ast::Expr) -> Option<(Option<ast::Type>, ast::Expr)> {
        let ExprKind::Call { callee, args } = &e.kind else {
            return None;
        };
        if !args.is_empty() {
            return None;
        }
        match &callee.kind {
            ExprKind::Name(n) if n == "read" => Some((None, e.clone())),
            ExprKind::Generic { base, args: targs }
                if targs.len() == 1 && matches!(&base.kind, ExprKind::Name(n) if n == "read") =>
            {
                Some((
                    Some(targs[0].clone()),
                    ast::Expr {
                        kind: ExprKind::Call {
                            callee: base.clone(),
                            args: Vec::new(),
                        },
                        span: e.span,
                    },
                ))
            }
            _ => None,
        }
    }
    /// The start of a domain `lo..hi` or `lo..=hi`, the integer after its last position, and
    /// what the types of the two ends say every position lies within.
    pub(super) fn domain_ends(
        &mut self,
        lo: &ast::Expr,
        hi: &ast::Expr,
        inclusive: bool,
    ) -> (TExpr, TExpr, (Option<i64>, Option<i64>)) {
        // An end keeps the range type it was written with.
        let lo = self.expr_as_typed(lo, None);
        let hi = self.expr_as_typed(hi, None);
        let ends = |e: &TExpr, ty: Ty| match (&e.kind, ty) {
            (TK::Int(k), _) => (Some(*k), Some(*k)),
            (_, Ty::Range(a, b)) => (Some(a), Some(b)),
            _ => (None, None),
        };
        let least = ends(&lo, self.shallow(&lo.ty)).0;
        let most = ends(&hi, self.shallow(&hi.ty)).1;
        let most = if inclusive {
            most
        } else {
            most.map(|m| m.saturating_sub(1))
        };
        let lo = self.coerce(lo, &Ty::Int);
        let hi = self.coerce(hi, &Ty::Int);
        let end = if inclusive {
            let span = hi.span;
            let one = TExpr {
                kind: TK::Int(1),
                ty: Ty::Int,
                span,
            };
            TExpr {
                kind: TK::Bin(BinOp::Add, Box::new(hi), Box::new(one)),
                ty: Ty::Int,
                span,
            }
        } else {
            hi
        };
        (lo, end, (least, most))
    }
    /// A domain's size is a hidden variable next to its declaration.
    pub(super) fn declare_domain(
        &mut self,
        name: &str,
        lo: TExpr,
        end: TExpr,
        within: (Option<i64>, Option<i64>),
        span: Span,
        out: &mut Vec<TStmt>,
    ) -> usize {
        let first = self.bind(&format!("{name}$lo"), Ty::Int);
        out.push(TStmt::Let(first, lo));
        let len = self.bind(&format!("{name}$len"), Ty::Int);
        // Spec 4: a range whose start is past its end is empty, never a trap.
        let tmp = self.bind(&format!("{name}$n"), Ty::Int);
        let var = |r| TExpr {
            kind: TK::Var(r),
            ty: Ty::Int,
            span,
        };
        let n = TExpr {
            kind: TK::Bin(BinOp::Sub, Box::new(end), Box::new(var(first))),
            ty: Ty::Int,
            span,
        };
        out.push(TStmt::Let(tmp, n));
        let v = |k| TExpr {
            kind: k,
            ty: Ty::Int,
            span,
        };
        let neg = TExpr {
            kind: TK::Cmp(
                CmpOp::Lt,
                Box::new(v(TK::Var(tmp))),
                Box::new(v(TK::Int(0))),
            ),
            ty: Ty::Bool,
            span,
        };
        out.push(TStmt::Let(
            len,
            v(TK::If(
                Box::new(neg),
                Box::new(v(TK::Int(0))),
                Box::new(v(TK::Var(tmp))),
            )),
        ));
        let info = DomInfo {
            name: name.to_string(),
            len,
            lo: first,
            within,
            owner: self.decl,
        };
        if let (None, Some(&Ty::Dom(i))) = (self.decl, self.d.top_doms.get(name)) {
            if self.domains[i].name.is_empty() {
                self.domains[i] = info;
                self.doms.insert(name.to_string(), Ty::Dom(i));
                return i;
            }
        }
        self.domains.push(info);
        self.doms
            .insert(name.to_string(), Ty::Dom(self.domains.len() - 1));
        self.domains.len() - 1
    }
    pub(super) fn plus_one(e: &ast::Expr) -> ast::Expr {
        Self::and_one(e, BinOp::Add)
    }
    pub(super) fn minus_one(e: &ast::Expr) -> ast::Expr {
        Self::and_one(e, BinOp::Sub)
    }
    fn and_one(e: &ast::Expr, op: BinOp) -> ast::Expr {
        let one = ast::Expr {
            kind: ExprKind::Int(1),
            span: e.span,
        };
        ast::Expr {
            kind: ExprKind::Binary {
                op,
                op_span: e.span,
                lhs: Box::new(e.clone()),
                rhs: Box::new(one),
            },
            span: e.span,
        }
    }
    /// `[lo..hi]T` and `[n]T` index by a range with no name: each gets a domain of its own, declared in `out`.
    pub(super) fn name_ranges(
        &mut self,
        t: &ast::Type,
        span: Span,
        out: &mut Vec<TStmt>,
    ) -> ast::Type {
        let mut t = t.clone();
        match &mut t.kind {
            TypeKind::Array { index, elem } => {
                **elem = self.name_ranges(elem, span, out);
                if index.len() == 1 {
                    let zero = || ast::Expr {
                        kind: ExprKind::Int(0),
                        span,
                    };
                    let ends = match &index[0].kind {
                        TypeKind::Range {
                            lo: Some(lo),
                            hi: Some(hi),
                            inclusive,
                        } => Some(((**lo).clone(), (**hi).clone(), *inclusive)),
                        TypeKind::Named { name, args }
                            if args.is_empty()
                                && self.lookup(&name.text).is_some()
                                && self.domain_named(&name.text).is_none() =>
                        {
                            Some((
                                zero(),
                                ast::Expr {
                                    kind: ExprKind::Name(name.text.clone()),
                                    span: name.span,
                                },
                                false,
                            ))
                        }
                        _ => None,
                    };
                    if let Some((lo, hi, inclusive)) = ends {
                        let (lo, end, within) = self.domain_ends(&lo, &hi, inclusive);
                        let name = format!("$range{}", self.domains.len());
                        self.declare_domain(&name, lo, end, within, span, out);
                        index[0] = ast::Type {
                            kind: TypeKind::Named {
                                name: ast::Name {
                                    text: name,
                                    span: index[0].span,
                                },
                                args: Vec::new(),
                            },
                            span: index[0].span,
                        };
                    }
                }
            }
            TypeKind::Named { args, .. } => {
                for a in args.iter_mut() {
                    *a = self.name_ranges(a, span, out);
                }
            }
            // The ends are values: they are worked out once, here, and the range gets a name.
            TypeKind::Range { lo, hi, inclusive }
                if decls::static_range(lo, hi, *inclusive).is_none() =>
            {
                let name = format!("$ends{}", self.ranges.len());
                let mut end =
                    |cx: &mut Self, e: &Option<Box<ast::Expr>>, which: &str, open: bool| {
                        e.as_ref().map(|b| {
                            let b = if open {
                                Self::minus_one(b)
                            } else {
                                (**b).clone()
                            };
                            let b = cx.expr(&b, Some(&Ty::Int));
                            let b = cx.coerce(b, &Ty::Int);
                            let v = cx.bind(&format!("{name}${which}"), Ty::Int);
                            out.push(TStmt::Let(v, b));
                            v
                        })
                    };
                let lo = end(self, lo, "lo", false);
                let hi = end(self, hi, "hi", !*inclusive);
                self.ranges.push(RangeInfo { lo, hi });
                self.doms
                    .insert(name.clone(), Ty::RangeOf(self.ranges.len() - 1));
                t.kind = TypeKind::Named {
                    name: ast::Name {
                        text: name,
                        span: t.span,
                    },
                    args: Vec::new(),
                };
            }
            _ => {}
        }
        t
    }
    pub(super) fn body_ty(&mut self, t: &ast::Type, span: Span, out: &mut Vec<TStmt>) -> Ty {
        let t = self.name_ranges(t, span, out);
        let mut errs = Vec::new();
        let t = decls::body_type(
            self.d,
            self.module,
            &self.generics,
            &self.doms,
            &t,
            &mut errs,
        );
        for e in errs {
            self.err(span, e);
        }
        self.fresh_sets(&t)
    }
    /// `v`, after a check that it is between the two ends, each inclusive.
    pub(super) fn checked_between(
        &mut self,
        v: TExpr,
        lo: Option<TExpr>,
        hi: Option<TExpr>,
        span: Span,
    ) -> TExpr {
        let kind = if matches!(v.kind, TK::ReadInt) {
            CheckKind::ReadRange
        } else {
            CheckKind::RangeStore
        };
        self.scopes.push(HashMap::new());
        let tmp = self.bind("$ranged", Ty::Int);
        let var = TExpr {
            kind: TK::Var(tmp),
            ty: Ty::Int,
            span,
        };
        let mut stmts = vec![TStmt::Let(tmp, v)];
        for (bound, op) in [(lo, CmpOp::Ge), (hi, CmpOp::Le)] {
            if let Some(b) = bound {
                let c = TExpr {
                    kind: TK::Cmp(op, Box::new(var.clone()), Box::new(b)),
                    ty: Ty::Bool,
                    span,
                };
                stmts.push(TStmt::Assert(
                    c,
                    "value outside its range".to_string(),
                    span,
                    kind,
                ));
            }
        }
        self.scopes.pop();
        TExpr {
            kind: TK::Block(stmts, Some(Box::new(var))),
            ty: Ty::Int,
            span,
        }
    }
    /// `read()` at a known type.
    pub(super) fn read_as(&mut self, t: &Ty, span: Span) -> TExpr {
        match self.shallow(t) {
            Ty::Struct(d, ts, _)
                if Some(d) == self.d.core_struct("Array")
                    && self.shallow(&ts[1]) == Ty::Int
                    && self.d.core_fn("read_array").is_some() =>
            {
                let Some(read_array) = self.d.core_fn("read_array") else {
                    return TExpr {
                        kind: TK::Error,
                        ty: Ty::Error,
                        span,
                    };
                };
                let f = self.fnref(read_array, vec![ts[0].clone()], Vec::new());
                let ty = self.ret_ty(&f);
                TExpr {
                    kind: TK::Call {
                        f,
                        recv: None,
                        args: vec![],
                    },
                    ty,
                    span,
                }
            }
            want @ (Ty::Range(..) | Ty::RangeOf(_)) => {
                let v = self.read_as(&Ty::Int, span);
                self.coerce(v, &want)
            }
            Ty::Int | Ty::Var(_) | Ty::Error => {
                self.unify(t, &Ty::Int);
                TExpr {
                    kind: TK::ReadInt,
                    ty: Ty::Int,
                    span,
                }
            }
            Ty::Struct(d, ts, sets) if self.d.s(d).name.starts_with("Tuple") => {
                let parts = ts.iter().map(|p| self.read_as(p, span)).collect();
                TExpr {
                    kind: TK::Struct(parts),
                    ty: Ty::Struct(d, ts, sets),
                    span,
                }
            }
            other => {
                self.err(span, format!("slice: `read` of `{}`", self.show(&other)));
                TExpr {
                    kind: TK::Error,
                    ty: Ty::Error,
                    span,
                }
            }
        }
    }
    /// An `Int` used to index an array is checked against the array's domain. The proof that removes the check is the facts layer's.
    pub(super) fn index_arg(&mut self, base: &Ty, i: TExpr) -> TExpr {
        if let Ty::Struct(d, ts, _) = self.shallow(base) {
            if Some(d) == self.d.core_struct("Array") && self.shallow(&i.ty) == Ty::Int {
                let dty = self.shallow(&ts[0]);
                if matches!(dty, Ty::Dom(_) | Ty::Param(_)) {
                    let span = i.span;
                    if let (Ty::Dom(d), TK::Int(k)) = (&dty, &i.kind) {
                        let (least, most) = self.domains[*d].within;
                        if least.is_some_and(|l| *k < l) || most.is_some_and(|m| *k > m) {
                            let name = self.domains[*d].name.clone();
                            self.err(
                                span,
                                format!(
                                    "{}: `{k}` is never a position of `{name}`",
                                    cpl_diag::codes::RANGE_STORE_CONST.0
                                ),
                            );
                        }
                    }
                    return TExpr {
                        kind: TK::DomFrom(dty.clone(), Box::new(i)),
                        ty: dty,
                        span,
                    };
                }
            }
        }
        // Keys never wrap (R07.2.17).
        if let TK::Neg(k) = &i.kind {
            if let TK::Int(k) = k.kind {
                self.err(
                    i.span,
                    format!(
                        "{}: `-{k}` is never a key: keys do not wrap; count from `.len`",
                        cpl_diag::codes::NEGATIVE_LITERAL_INDEX.0
                    ),
                );
            }
        }
        i
    }
    /// An expression that names storage: a variable, or a field of one.
    /// The name of the `let` a place lies in, if it lies in one.
    pub(super) fn let_root(&self, place: &TExpr) -> Option<String> {
        let mut root = place;
        while let TK::Field(b, _) = &root.kind {
            root = b;
        }
        let TK::Var(v) = &root.kind else { return None };
        let (fixed, name) = match v {
            VarRef::Local(i) => (self.locals[*i].fixed, &self.locals[*i].name),
            VarRef::Global(i) => (self.globals[*i].fixed, &self.globals[*i].name),
        };
        (fixed && !name.starts_with('$')).then(|| name.clone())
    }

    pub(super) fn place(&mut self, e: &ast::Expr) -> TExpr {
        let v = self.expr_as_typed(e, None);
        if !matches!(v.kind, TK::Var(_) | TK::Field(..) | TK::Error) {
            self.err(e.span, "this is not a place that can be assigned or lent");
        }
        v
    }
}

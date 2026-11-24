//! `if`, `if let`, `match` and the pattern tests they lower to.

use super::*;
use cpl_diag::msgs;

impl<'a> Lowerer<'a> {
    pub(in crate::lower) fn if_chain(
        &mut self,
        branches: &[(ast::Cond, ast::Block)],
        otherwise: Option<&ast::Block>,
        tail: Tail,
        out: &mut Vec<Stmt>,
    ) {
        let Some(((cond, body), rest)) = branches.split_first() else {
            if let Some(block) = otherwise {
                out.extend(self.block(block, tail).stmts);
            }
            return;
        };
        match cond {
            // `if const` and `if T: Tr`: settled here; only the branch picked is lowered.
            ast::Cond::Const(cond)
            | ast::Cond::Expr(
                cond @ ast::Expr {
                    kind: ast::ExprKind::HasBound { .. },
                    ..
                },
            ) => match self.settle_bool(cond) {
                Some(true) => out.extend(self.block(body, tail).stmts),
                Some(false) => self.if_chain(rest, otherwise, tail, out),
                None => {}
            },
            ast::Cond::Expr(cond) => {
                let cond = self.expect(cond, &Ty::Bool);
                if matches!(cond.kind, ExprKind::Bool(true)) {
                    out.extend(self.block(body, tail).stmts);
                    return;
                }
                if matches!(cond.kind, ExprKind::Bool(false)) {
                    self.if_chain(rest, otherwise, tail, out);
                    return;
                }
                // In the branch the test holds; past it, the test has
                // failed.
                let (test, _) = &branches[0];
                let mark = self.cx.present.len();
                let mutators = self.mutators.clone();
                let held = narrow::in_branch(test, body, &mut |f, x| mutators.writes(f, x));
                self.cx.present.extend(held.into_iter().map(String::from));
                let then = self.block(body, tail);
                self.cx.present.truncate(mark);
                let mutators = self.mutators.clone();
                let failed = narrow::past(test, rest, otherwise, &mut |f, x| mutators.writes(f, x));
                self.cx.present.extend(failed.into_iter().map(String::from));
                let mut els = Vec::new();
                self.if_chain(rest, otherwise, tail, &mut els);
                self.cx.present.truncate(mark);
                out.push(Stmt::If(cond, then, Block { stmts: els }));
            }
            ast::Cond::Let { .. } | ast::Cond::Guard { .. } => {
                if self.given_ty(tail).is_some_and(|t| t.is_none()) {
                    self.probe_given_ty(cond, body, tail);
                }
                let mut els = Vec::new();
                self.if_chain(rest, otherwise, tail, &mut els);
                self.clause_chain(cond, body, tail, &els, out);
            }
        }
    }

    /// `if o is Some(x): [x] else: []`: the `else` is lowered before the branch that binds
    /// `x`, so the branch is read first for the type the `if` gives.
    pub(in crate::lower) fn probe_given_ty(
        &mut self,
        cond: &ast::Cond,
        body: &ast::Block,
        tail: Tail,
    ) {
        let diag_len = self.diags.len();
        let saved_cx = self.cx.clone();
        let saved_outer = self.outer_cxs.clone();
        self.clause_chain(cond, body, tail, &[], &mut Vec::new());
        // A view from the branch (`s[..q]`) is no type for the `if`: the
        // `else` may give an owned value, and the view copies to meet it.
        let ty = (self.given_ty(tail).flatten()).filter(|t| *t != Ty::Error && !t.holds_view());
        self.cx = saved_cx;
        self.outer_cxs = saved_outer;
        self.diags.truncate(diag_len);
        match tail {
            Tail::Return => self.cx.ret = ty,
            _ => self
                .cx
                .dos
                .last_mut()
                .into_iter()
                .for_each(|f| f.ty = ty.clone()),
        }
    }

    /// The type a value-giving tail gives so far: `Some(None)` while not yet known.
    pub(in crate::lower) fn given_ty(&self, tail: Tail) -> Option<Option<Ty>> {
        match tail {
            Tail::Give => self.cx.dos.last().map(|f| f.ty.clone()),
            Tail::Return => Some(self.cx.ret.clone()),
            _ => None,
        }
    }

    /// Lowers `if let pat = value` / `value is pat` / `while let pat = value`.
    /// Variant patterns (`Seeking`, `Phase.Seeking`, `Split(base)`) match an
    /// enum value directly via `self.pattern`; all other patterns unwrap an
    /// optional via `self.optional`.
    pub(in crate::lower) fn cond_let(
        &mut self,
        pat: &ast::Pattern,
        value: &ast::Expr,
        what: &str,
        before: &mut Vec<Stmt>,
        binds: &mut Vec<Stmt>,
    ) -> Option<Expr> {
        if let Some(present) = self.opt_mut_alias(pat, value) {
            return Some(present);
        }
        if let ast::PatternKind::Literal(lit) = &pat.kind {
            if matches!(lit.kind, ast::ExprKind::None) {
                let (present, _) = self.optional(value, what, before)?;
                let span = present.span;
                return Some(Expr {
                    kind: ExprKind::Unary(UnOp::Not, Box::new(present)),
                    ty: Ty::Bool,
                    span,
                });
            }
        }
        // `if let some(Seeking) = x` and `if let (some(l), some(r)) = (a, b)` test the whole value.
        let whole = match &pat.kind {
            ast::PatternKind::Variant { path, args } => {
                path.len() == 1 && path[0].text == "some" && args.len() == 1 && refutable(&args[0])
            }
            ast::PatternKind::Tuple(_) | ast::PatternKind::Record { .. } => refutable(pat),
            _ => false,
        };
        if whole {
            return self.whole_cond(pat, value, before, binds);
        }
        if let ast::PatternKind::Variant { path, args } = &pat.kind {
            // R04.2.14: `if let V(j) = x` is the pattern form of `V.at(x)`.
            if let ([name], [arg]) = (&path[..], &args[..]) {
                if let Some(id) = self.resolve_domain_name(&name.text) {
                    let label = self.expr(value, Some(&Ty::Int));
                    if label.ty == Ty::Error {
                        return None;
                    }
                    let label = self.coerce(label, &Ty::Int, value.span);
                    let at = self.domain_try_label(id, label, value.span);
                    let slot = self.temp(at.ty.clone());
                    before.push(Stmt::Let(slot, Some(at)));
                    let held = self.local(slot, value.span);
                    let present = self.mk_is_some(held.clone(), Ty::Bool, value.span);
                    let inner = self.mk_unwrap(held, false, Ty::Pos(id), value.span);
                    self.bind(arg, inner, binds);
                    return Some(present);
                }
            }
            if path.len() == 1 && path[0].text == "some" && args.len() == 1 {
                let (present, inner) = self.optional(value, what, before)?;
                self.bind(&args[0], inner, binds);
                return Some(present);
            }
            let value_expr = self.expr(value, None);
            if self.is_enum_ty(&value_expr.ty) {
                let tmp = self.temp(value_expr.ty.clone());
                before.push(Stmt::Let(tmp, Some(value_expr)));
                let local = self.local(tmp, value.span);
                let test = self.pattern(pat, &local, binds);
                return Some(
                    test.unwrap_or_else(|| self.mk(ExprKind::Bool(true), Ty::Bool, value.span)),
                );
            }
            if value_expr.ty == Ty::Error {
                return None;
            }
        }
        let (present, inner) = self.optional(value, what, before)?;
        self.bind(pat, inner, binds);
        Some(present)
    }

    /// Evaluates the optional `value` into a temporary; gives the test
    /// that it is present and the value inside.
    pub(in crate::lower) fn optional(
        &mut self,
        value: &ast::Expr,
        what: &str,
        out: &mut Vec<Stmt>,
    ) -> Option<(Expr, Expr)> {
        self.cx.as_written = narrow::asks_optional(value);
        let value_expr = self.expr(value, None);
        self.cx.as_written = false;
        let Ty::Opt(inner) = value_expr.ty.clone() else {
            if value_expr.ty != Ty::Error {
                self.not_optional(what, &value_expr, value.span);
            }
            return None;
        };
        let span = value_expr.span;
        let slot = self.temp(value_expr.ty.clone());
        let held = Expr {
            kind: ExprKind::Local(slot),
            ty: value_expr.ty.clone(),
            span,
        };
        out.push(Stmt::Let(slot, Some(value_expr)));
        let present = Expr {
            kind: ExprKind::IsSome(Box::new(held.clone())),
            ty: Ty::Bool,
            span,
        };
        let inside = Expr {
            kind: ExprKind::Unwrap(Box::new(held), false),
            ty: *inner,
            span,
        };
        Some((present, inside))
    }

    /// The clauses of `if let a = x, let b = y, a < b:` after the first
    /// branch's `else` is known: each clause is tried only when those
    /// before it held, and the first that fails goes to `els`.
    pub(in crate::lower) fn clause_chain(
        &mut self,
        cond: &ast::Cond,
        body: &ast::Block,
        tail: Tail,
        els: &[Stmt],
        out: &mut Vec<Stmt>,
    ) {
        let els = || Block {
            stmts: els.to_vec(),
        };
        match cond {
            ast::Cond::Expr(test) => {
                let test = self.expect(test, &Ty::Bool);
                let mark = self.cx.present.len();
                let mutators = self.mutators.clone();
                let held = narrow::in_branch(cond, body, &mut |f, x| mutators.writes(f, x));
                self.cx.present.extend(held.into_iter().map(String::from));
                let then = self.block(body, tail);
                self.cx.present.truncate(mark);
                out.push(Stmt::If(test, then, els()));
            }
            ast::Cond::Guard { test, and } => {
                let test = self.expect(test, &Ty::Bool);
                let mut then = Vec::new();
                self.clause_chain(and, body, tail, els().stmts.as_slice(), &mut then);
                out.push(Stmt::If(test, Block { stmts: then }, els()));
            }
            ast::Cond::Let { pat, value, and } => {
                self.cx.scopes.push(HashMap::new());
                let mut then = Vec::new();
                let Some(present) = self.cond_let(pat, value, "if let", out, &mut then) else {
                    self.cx.scopes.pop();
                    return;
                };
                match and {
                    Some(and) => {
                        self.clause_chain(and, body, tail, els().stmts.as_slice(), &mut then)
                    }
                    None => then.extend(self.block(body, tail).stmts),
                }
                self.cx.scopes.pop();
                out.push(Stmt::If(present, Block { stmts: then }, els()));
            }
            ast::Cond::Const(c) => {
                self.unsupported(c.span, "`const` after another clause of a condition")
            }
        }
    }

    pub(in crate::lower) fn match_stmt(
        &mut self,
        whole: &ast::Expr,
        scrutinee: &ast::Expr,
        arms: &[ast::Arm],
        tail: Tail,
        out: &mut Vec<Stmt>,
    ) {
        self.cx.as_written = narrow::asks_optional(scrutinee);
        let value = self.expr(scrutinee, None);
        self.cx.as_written = false;
        let value = self.hold(value, out);
        let span = self.span(whole.span);
        let trap = Stmt::Trap("no arm of this `match` fits the value".into(), span);
        if arms.iter().all(|arm| arm.guard.is_none()) {
            let mut lowered = Vec::new();
            for arm in arms {
                self.cx.scopes.push(HashMap::new());
                let mut body = Vec::new();
                let test = self.pattern(&arm.pat, &value, &mut body);
                body.extend(self.block(&arm.body, tail).stmts);
                self.cx.scopes.pop();
                lowered.push((test, body));
            }
            let mut chain = vec![trap];
            for (test, body) in lowered.into_iter().rev() {
                chain = match test {
                    None => body,
                    Some(test) => vec![Stmt::If(
                        test,
                        Block { stmts: body },
                        Block { stmts: chain },
                    )],
                };
            }
            out.extend(chain);
            return;
        }
        // With guards an arm can fit the pattern and still decline, so the
        // arms are tried in turn until one has run.
        let done = self.temp(Ty::Bool);
        let done_expr = Expr {
            kind: ExprKind::Local(done),
            ty: Ty::Bool,
            span,
        };
        let not_done = Expr {
            kind: ExprKind::Unary(UnOp::Not, Box::new(done_expr.clone())),
            ty: Ty::Bool,
            span,
        };
        let flag = |value: bool| Expr {
            kind: ExprKind::Bool(value),
            ty: Ty::Bool,
            span,
        };
        out.push(Stmt::Let(done, Some(flag(false))));
        for arm in arms {
            self.cx.scopes.push(HashMap::new());
            let mut body = Vec::new();
            let test = self.pattern(&arm.pat, &value, &mut body);
            let guard = arm.guard.as_ref().map(|g| self.expect(g, &Ty::Bool));
            let mut run = vec![Stmt::Assign(
                Place {
                    kind: PlaceKind::Local(done),
                    ty: Ty::Bool,
                    span,
                },
                AssignOp::Set,
                flag(true),
            )];
            run.extend(self.block(&arm.body, tail).stmts);
            self.cx.scopes.pop();
            match guard {
                Some(guard) => body.push(Stmt::If(guard, Block { stmts: run }, Block::default())),
                None => body.extend(run),
            }
            let test = match test {
                None => not_done.clone(),
                Some(test) => Expr {
                    kind: ExprKind::Binary(BinOp::And, Box::new(not_done.clone()), Box::new(test)),
                    ty: Ty::Bool,
                    span,
                },
            };
            out.push(Stmt::If(test, Block { stmts: body }, Block::default()));
        }
        // An arm that gives its value has left by here, unless the arms
        // are written out as statements that store it.
        let stored = tail == Tail::Give && self.cx.dos.last().is_some_and(|frame| frame.open);
        if tail == Tail::Discard || stored {
            out.push(Stmt::If(
                not_done,
                Block { stmts: vec![trap] },
                Block::default(),
            ));
        } else {
            out.push(trap);
        }
    }

    /// The test that `value` fits `pat` (`None` when it always does), with
    /// the statements that bind the pattern's names once it has.
    pub(in crate::lower) fn pattern(
        &mut self,
        pat: &ast::Pattern,
        value: &Expr,
        binds: &mut Vec<Stmt>,
    ) -> Option<Expr> {
        let span = self.span(pat.span);
        let and = |a: Option<Expr>, b: Option<Expr>| match (a, b) {
            (Some(a), Some(b)) => Some(Expr {
                kind: ExprKind::Binary(BinOp::And, Box::new(a), Box::new(b)),
                ty: Ty::Bool,
                span,
            }),
            (a, b) => a.or(b),
        };
        match &pat.kind {
            ast::PatternKind::Wildcard | ast::PatternKind::Domain(_) | ast::PatternKind::Error => {
                None
            }
            ast::PatternKind::Name(name) => {
                self.bind_name(name, value.clone(), binds);
                None
            }
            ast::PatternKind::Var(inner) => self.pattern(inner, value, binds),
            ast::PatternKind::Literal(lit) => {
                if let (ast::ExprKind::None, Ty::Opt(_)) = (&lit.kind, &value.ty) {
                    let present = Expr {
                        kind: ExprKind::IsSome(Box::new(value.clone())),
                        ty: Ty::Bool,
                        span,
                    };
                    return Some(Expr {
                        kind: ExprKind::Unary(UnOp::Not, Box::new(present)),
                        ty: Ty::Bool,
                        span,
                    });
                }
                let lit = self.expect(lit, &value.ty);
                Some(Expr {
                    kind: ExprKind::Cmp(CmpOp::Eq, Box::new(value.clone()), Box::new(lit)),
                    ty: Ty::Bool,
                    span,
                })
            }
            ast::PatternKind::Variant { path, args } => {
                let name = path.last().expect("a variant pattern has a name");
                if path.len() == 1 && name.text == "some" {
                    let Ty::Opt(inner_ty) = value.ty.clone() else {
                        if value.ty != Ty::Error {
                            self.report(
                                pat.span,
                                msgs::ir::TYPE_MISMATCH_SOME_MATCHES_AN_OPTIONAL
                                    .at(&[&self.show(&value.ty)]),
                            );
                        }
                        return None;
                    };
                    if args.len() != 1 {
                        self.report(
                            pat.span,
                            msgs::ir::WRONG_ARG_COUNT_SOME_HOLDS_1_VALUE.at(&[&args.len()]),
                        );
                        return None;
                    }
                    let is_some = self.mk_is_some(value.clone(), Ty::Bool, pat.span);
                    let unwrapped = self.mk_unwrap(value.clone(), false, *inner_ty, pat.span);
                    return and(Some(is_some), self.pattern(&args[0], &unwrapped, binds));
                }
                if let Ty::Pos(d) = value.ty {
                    if let Some(info) = self.sum_enums.get(&d).cloned() {
                        let Some(tag) = info.variants.iter().position(|(v, _)| *v == name.text)
                        else {
                            let message =
                                format!("`{}` has no variant `{}`", self.domains[d], name.text);
                            self.error(codes::UNKNOWN_FIELD, name.span, message);
                            return None;
                        };
                        let factors = &info.variants[tag].1;
                        if !args.is_empty() && factors.len() != args.len() {
                            self.report(
                                pat.span,
                                msgs::ir::WRONG_ARG_COUNT_HOLDS_VALUES_BUT_THIS.at(&[
                                    &name.text,
                                    &factors.len(),
                                    &args.len(),
                                ]),
                            );
                            return None;
                        }
                        let mut test = Some(self.is_sum_variant(d, tag, value.clone(), pat.span));
                        for (i, arg) in args.iter().enumerate() {
                            let payload =
                                self.decode_sum_variant_field(d, tag, i, value.clone(), pat.span);
                            test = and(test, self.pattern(arg, &payload, binds));
                        }
                        return test;
                    }
                }
                let Ty::Enum(id) = value.ty else {
                    if value.ty != Ty::Error {
                        self.report(
                            pat.span,
                            msgs::check::TYPE_MISMATCH_IS_A_VARIANT_BUT
                                .at(&[&name.text, &self.show(&value.ty)]),
                        );
                    }
                    return None;
                };
                let variants = &self.enums[id].variants;
                let Some(tag) = variants.iter().position(|(v, _)| *v == name.text) else {
                    let message =
                        format!("`{}` has no variant `{}`", self.enums[id].name, name.text);
                    self.error(codes::UNKNOWN_FIELD, name.span, message);
                    return None;
                };
                let fields = variants[tag].1.clone();
                if !args.is_empty() && fields.len() != args.len() {
                    self.report(
                        pat.span,
                        msgs::ir::WRONG_ARG_COUNT_HOLDS_VALUES_BUT_THIS.at(&[
                            &name.text,
                            &fields.len(),
                            &args.len(),
                        ]),
                    );
                    return None;
                }
                let mut test = Some(Expr {
                    kind: ExprKind::IsVariant(Box::new(value.clone()), tag),
                    ty: Ty::Bool,
                    span,
                });
                for (i, (arg, ty)) in args.iter().zip(fields).enumerate() {
                    let payload = Expr {
                        kind: ExprKind::Payload(Box::new(value.clone()), tag, i),
                        ty,
                        span,
                    };
                    test = and(test, self.pattern(arg, &payload, binds));
                }
                test
            }
            ast::PatternKind::Record { .. } => {
                let tuple = self.record_pattern(pat, &value.ty)?;
                self.pattern(&tuple, value, binds)
            }
            ast::PatternKind::Tuple(pats) => {
                let Some(fields) = value.ty.fields().filter(|f| f.len() == pats.len()) else {
                    if value.ty != Ty::Error {
                        self.report(
                            pat.span,
                            msgs::ir::TYPE_MISMATCH_A_PATTERN_OF_PARTS
                                .at(&[&pats.len(), &self.show(&value.ty)]),
                        );
                    }
                    return None;
                };
                let mut test = None;
                for (i, (p, ty)) in pats.iter().zip(fields).enumerate() {
                    let field = Expr {
                        kind: ExprKind::Field(Box::new(value.clone()), i),
                        ty,
                        span,
                    };
                    test = and(test, self.pattern(p, &field, binds));
                }
                test
            }
            ast::PatternKind::Or(alts) => self.or_pattern(alts, value, binds, span),
        }
    }
}

impl Lowerer<'_> {
    /// R02.7.6: `(to: u, ..)` over a record value, as the tuple pattern over
    /// its fields in declaration order.
    pub(in crate::lower) fn record_pattern(
        &mut self,
        pat: &ast::Pattern,
        ty: &Ty,
    ) -> Option<ast::Pattern> {
        let message = match ty {
            Ty::Error => return None,
            Ty::Record(fields) => {
                let names: Vec<&str> = fields.iter().map(|(n, _)| n.as_str()).collect();
                match pat.record_as_tuple(&names) {
                    Ok(tuple) => return Some(tuple),
                    Err(message) => message,
                }
            }
            _ => format!("a record pattern can't match a `{}`", self.show(ty)),
        };
        self.error(codes::TYPE_MISMATCH, pat.span, message);
        None
    }
}

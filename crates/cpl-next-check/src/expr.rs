//! Expressions: operators, calls, members, names.

use super::*;
use cpl_next_ir::ids::ItemId;
use cpl_next_ir::unbuilt::Unbuilt;
use cpl_syntax::ast::FmtKind;

impl<'a> Cx<'a> {
    /// Std's `@role(list)` struct, which literals, comprehensions and tables build.
    pub(super) fn list_struct(&mut self, span: Span) -> Option<StructId> {
        let list = self.d.sugar.list;
        if list.is_none() {
            self.err(span, Msg::internal("no struct is filling `@role(list)`"));
        }
        list
    }
    pub(super) fn binary(&mut self, op: BinOp, l: TExpr, r: TExpr, span: Span) -> TExpr {
        if self.is_error(&l.ty) || self.is_error(&r.ty) {
            return Self::error_node(span);
        }
        let logic = matches!(op, BinOp::And | BinOp::Or);
        let shift = matches!(op, BinOp::Shl | BinOp::Shr);
        // `U64` arithmetic wraps, and a shift's count is an `Int` (R02.1).
        let unsigned = !logic
            && (self.shallow(&l.ty) == Ty::U64 || (!shift && self.shallow(&r.ty) == Ty::U64));
        let float = !logic && !shift && [&l.ty, &r.ty].iter().any(|t| self.shallow(t) == Ty::F64);
        // An `Int` beside an `I128` becomes one; a shift keeps what it shifts (R02.1).
        let wide = !logic
            && (self.shallow(&l.ty) == Ty::I128 || (!shift && self.shallow(&r.ty) == Ty::I128));
        if wide && unsigned {
            let msg = Msg::verdict(
                cpl_diag::codes::TYPE_MISMATCH,
                "an `I128` and a `U64` do not combine: say which one the result is, `I128(x)` or `U64(x)`",
            );
            self.err(span, msg);
            return Self::error_node(span);
        }
        let want = match (logic, unsigned, float) {
            (true, ..) => Ty::Bool,
            (_, _, true) => Ty::F64,
            (_, true, _) => Ty::U64,
            _ if wide => Ty::I128,
            _ => Ty::Int,
        };
        if float && !matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div) {
            self.err(span, Unbuilt::FloatOperator);
        }
        let bits = matches!(op, BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor);
        if bits && [&l.ty, &r.ty].iter().any(|t| self.shallow(t) == Ty::Bool) {
            return self.bool_bits(op, &l.ty, &r.ty, span);
        }
        // `+` and the rest on a struct, or on a type parameter bound by the operator's trait,
        // are that trait's fn (R08.1.5).
        if let Some(found) = (!logic)
            .then(|| self.operator_trait(op.text(), true, &l.ty))
            .flatten()
        {
            return self.operator_call(found, l, Some(r), span);
        }
        let (l, r) = match self.position_arith(op, l, r, span) {
            Ok(e) => return e,
            Err(lr) => *lr,
        };
        let operand = [&l.ty, &r.ty]
            .into_iter()
            .find(|t| self.declares_operators(t));
        if let Some(t) = operand.filter(|_| !logic) {
            let t = self.show(t);
            self.err(span, Unbuilt::OperatorTrait { ty: t });
            return TExpr {
                kind: TK::Error,
                ty: Ty::Error,
                span,
            };
        }
        let l = self.coerce(l, &want);
        let r = self.coerce(r, if shift { &Ty::Int } else { &want });
        if op == BinOp::Coalesce {
            self.err(span, Unbuilt::Operator);
        }
        TExpr {
            kind: TK::Bin(op, Box::new(l), Box::new(r)),
            ty: want,
            span,
        }
    }
    /// A struct, whose operators are its impls of the operator traits (R08.1.5).
    fn declares_operators(&self, t: &Ty) -> bool {
        matches!(self.shallow(t), Ty::Struct(..))
    }
    /// The `@operator(text)` trait, binary or prefix, and its fn, when `t`'s operator is that
    /// trait's: `t` is a struct, or a type parameter bound by the trait.
    fn operator_trait(&self, text: &str, binary: bool, t: &Ty) -> Option<(TraitId, String, FnId)> {
        let found = self.d.operator_traits(text).find_map(|tr| {
            let (n, m) = self.d.operator_method(tr)?;
            (self.d.f(m).params.len() == usize::from(binary)).then(|| (tr, n.to_string(), m))
        })?;
        let applies = match self.shallow(t) {
            // An unbounded parameter's `+` is a missing bound, unless it is a position (R08.2.2).
            Ty::Param(p) => {
                self.bound_traits(p).iter().any(|b| b.tr == found.0) || !self.is_position(t)
            }
            t => self.declares_operators(&t),
        };
        applies.then_some(found)
    }
    /// `w by k` on a walk that is not a range literal: `w`'s `@operator("by")` trait fn, whose
    /// result is a walk of `w`'s type (R01.8.4).
    fn step_walk(&mut self, walk: &ast::Expr, step: &ast::Expr, span: Span) -> TExpr {
        let w = self.expr_as_typed(walk, None);
        if self.is_error(&w.ty) {
            return Self::error_node(span);
        }
        let Some(found) = self.operator_trait("by", true, &w.ty) else {
            self.err(span, Unbuilt::ByOnWalk);
            return Self::error_node(span);
        };
        let k = self.expr(step, Some(&Ty::Int));
        self.operator_call(found, w, Some(k), span)
    }
    /// `x in s` as `s`'s core `@operator("in")` trait fn, `contains(x)` (R12.4.7).
    fn membership(&mut self, x: TExpr, s: TExpr, span: Span) -> TExpr {
        if self.is_error(&x.ty) || self.is_error(&s.ty) {
            return Self::error_node(span);
        }
        let found = self.d.operator_traits("in").find_map(|tr| {
            let (n, m) = self.d.operator_method(tr)?;
            Some((tr, n.to_string(), m))
        });
        let Some(found) = found else {
            self.err(span, Unbuilt::Membership);
            return Self::error_node(span);
        };
        TExpr {
            ty: Ty::Bool,
            ..self.operator_call(found, s, Some(x), span)
        }
    }
    /// `a op b` or `op a` as the operator trait's fn of `a`'s type, which is also its result.
    fn operator_call(
        &mut self,
        (tr, method, m): (TraitId, String, FnId),
        l: TExpr,
        r: Option<TExpr>,
        span: Span,
    ) -> TExpr {
        self.reach(m);
        let self_ty = l.ty.clone();
        let tr = decls::TraitRef {
            tr,
            args: r.iter().map(|r| r.ty.clone()).collect(),
        };
        (self.b.obligations).push((self_ty.clone(), tr.clone(), span, Origin::Bound));
        TExpr {
            kind: TK::CallTrait {
                ctx: self.trait_ctx(tr.tr, &method),
                tr,
                method,
                self_ty: Box::new(self_ty.clone()),
                targs: Box::default(),
                recv: Some(Box::new(TArg::Value(l))),
                args: r.map(TArg::Value).into_iter().collect(),
            },
            ty: self_ty,
            span,
        }
    }
    /// `&`, `|` and `^` are integer operators; on a `Bool` the error names the `Bool` one
    /// (R01.5.9).
    fn bool_bits(&mut self, op: BinOp, l: &Ty, r: &Ty, span: Span) -> TExpr {
        let a = |t: String| match t.starts_with(['A', 'E', 'I', 'O', 'U']) {
            true => format!("an {t}"),
            false => format!("a {t}"),
        };
        let (l, r) = (a(self.show(l)), a(self.show(r)));
        let text = cpl_diag::msgs::check::BAD_OPERANDS_CAN_T_COMBINE_AND.say(&[&op.text(), &l, &r]);
        let help = match op {
            BinOp::BitXor => cpl_diag::habits::BAD_OPERANDS_3.say(&[]),
            BinOp::BitAnd => "on `Bool`s, `&` is `and`".to_string(),
            _ => "on `Bool`s, `|` is `or`".to_string(),
        };
        let code = cpl_diag::codes::BAD_OPERANDS;
        self.err_help(span, Msg::verdict(code, text), Some(help));
        TExpr {
            kind: TK::Error,
            ty: Ty::Error,
            span,
        }
    }
    /// The arguments of a call bound to its parameters: positional ones in order, named ones by
    /// label, the rest from their defaults (R06.1.5). In parameter order; what must run first to
    /// keep the written left-to-right order (R05.11) is in the first part.
    /// `n(x for x in xs, ..)` where `n` is a free fn with no `self`.
    fn fold_form(&self, n: &str, args: &[ast::Arg]) -> bool {
        args.first()
            .is_some_and(|a| a.name.is_none() && a.value.is_generator())
            && matches!(self.d.named(self.b.module, n), Some(decls::Named::Fn(f)) if self.d.f(f).self_ty.is_none())
    }
    /// More parts than the widest tuple: the last part is a tuple of the rest, nested as far as
    /// needed.
    fn nest_parts(&self, mut xs: Vec<ast::Expr>) -> ExprKind {
        let wide = self.d.widest_tuple();
        if wide < 2 || xs.len() <= wide {
            return ExprKind::Tuple(xs);
        }
        let rest = xs.split_off(wide - 1);
        let span = Span::new(rest[0].span.start, rest[rest.len() - 1].span.end);
        let kind = self.nest_parts(rest);
        xs.push(ast::Expr { kind, span });
        ExprKind::Tuple(xs)
    }
    /// The arguments past the others gathered into the one `..xs` parameter takes: a
    /// tuple when positional, a record when named, in written order (R06.1.5, decision 324).
    fn pack_variadic(
        &mut self,
        args: &[ast::Arg],
        formals: &[Formal<'_>],
        span: Span,
    ) -> Option<Vec<ast::Arg>> {
        let at = formals.iter().position(|f| f.variadic)?;
        let v = &formals[at];
        let fixed: Vec<&Formal> = formals.iter().filter(|f| !f.variadic).collect();
        let mut positional = formals[..at].iter().filter(|f| !f.named_only).count();
        let (mut out, mut tuple, mut record) = (Vec::new(), Vec::new(), Vec::new());
        for a in args {
            match &a.name {
                None if positional > 0 => {
                    positional -= 1;
                    out.push(a.clone());
                }
                None => tuple.push(a.value.clone()),
                Some(n) if fixed.iter().any(|f| f.label == n.text) => out.push(a.clone()),
                Some(n) => record.push((n.clone(), a.value.clone())),
            }
        }
        let at = (tuple.iter().chain(record.iter().map(|(_, x)| x)))
            .map(|x| x.span)
            .reduce(|a, b| Span::new(a.start, b.end))
            .unwrap_or(span);
        let kind = match (tuple.is_empty(), record.is_empty()) {
            (_, true) => self.nest_parts(tuple),
            (true, false) => ExprKind::Record(record),
            (false, false) => {
                self.err(
                    at,
                    Msg::verdict(
                        cpl_diag::codes::WRONG_ARG_COUNT,
                        format!(
                            "`..{}` takes positional or named arguments, not both",
                            v.label
                        ),
                    ),
                );
                ExprKind::Tuple(tuple)
            }
        };
        out.push(ast::Arg {
            name: Some(ast::Name {
                text: v.label.to_string(),
                span: at,
            }),
            mutable: false,
            value: ast::Expr { kind, span: at },
        });
        Some(out)
    }
    pub(super) fn args(
        &mut self,
        args: &[ast::Arg],
        formals: &[Formal<'_>],
        span: Span,
    ) -> (Vec<TStmt>, Vec<TArg>) {
        let packed;
        let args = match self.pack_variadic(args, formals, span) {
            Some(p) => {
                packed = p;
                &packed[..]
            }
            None => args,
        };
        let seen = self.errors.len();
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
                        self.err(
                            a.value.span,
                            Msg::verdict(
                                cpl_diag::codes::UNKNOWN_ARG_NAME,
                                format!("no parameter named `{}`", n.text),
                            ),
                        );
                    }
                    at
                }
            };
            match at {
                Some(i) if slot[i].is_none() => slot[i] = Some(k),
                Some(i) => self.err(
                    a.value.span,
                    Msg::verdict(
                        cpl_diag::codes::WRONG_ARG_COUNT,
                        format!("`{}` is given twice", formals[i].label),
                    ),
                ),
                None => {}
            }
        }
        let missing = (0..formals.len()).any(|i| slot[i].is_none() && formals[i].default.is_none());
        if excess || (missing && args.iter().all(|a| a.name.is_none())) {
            self.err(
                span,
                Msg::verdict(
                    cpl_diag::codes::WRONG_ARG_COUNT,
                    format!("expected {} arguments, found {}", formals.len(), args.len()),
                ),
            );
        } else if missing {
            for (i, f) in formals.iter().enumerate() {
                if slot[i].is_none() && f.default.is_none() {
                    self.err(
                        span,
                        Msg::verdict(
                            cpl_diag::codes::WRONG_ARG_COUNT,
                            format!("no argument for `{}`", f.label),
                        ),
                    );
                }
            }
        }
        // Arguments that do not fit the parameters make the call the error: its bounds are not asked.
        if self.errors.len() > seen {
            self.b.obligations.retain(|o| o.2 != span);
        }
        // A lambda with unwritten parameter types is checked after the other arguments, whose
        // types and bounds may fix the ones it takes (R06.1.2).
        // So is an integer literal for a parameter bound by `Domain`: the domain the others fix
        // makes it a position of that domain (R04.2.18).
        let late: Vec<bool> = (args.iter().enumerate())
            .map(|(k, a)| {
                let position = a.value.int_literal().is_some()
                    && (slot.iter().position(|s| *s == Some(k)))
                        .is_some_and(|i| self.domain_var(&formals[i].ty));
                position
                    || matches!(&a.value.kind, ExprKind::Lambda { params, param_tys, .. }
                        if params.len() > param_tys.iter().flatten().count())
            })
            .collect();
        let mut typed: Vec<Option<TArg>> = (0..args.len()).map(|_| None).collect();
        for pass in [false, true] {
            if pass {
                self.solve(false);
            }
            for (k, a) in args.iter().enumerate() {
                if late[k] != pass {
                    continue;
                }
                typed[k] = match slot.iter().position(|s| *s == Some(k)) {
                    Some(i) => Some(self.arg(a, &formals[i].ty, formals[i].mutable)),
                    None => {
                        self.expr(&a.value, None);
                        None
                    }
                };
            }
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
                // A built struct's parts are read one by one, so a name in it (a variadic
                // pack's) stays the name the struct reads where it is made, not a value moved
                // into a temp.
                let Some(TArg::Value(v)) = t else { continue };
                let mut parts = vec![v];
                while let Some(v) = parts.pop() {
                    if pure(v) {
                        continue;
                    }
                    let v = match v {
                        TExpr {
                            kind: TK::Struct(fs),
                            ..
                        } => {
                            parts.extend(fs.iter_mut().rev());
                            continue;
                        }
                        v => v,
                    };
                    let ty = v.ty.clone();
                    let tmp = self.temp("$arg", ty.clone());
                    let read = TExpr {
                        kind: TK::Var(tmp),
                        ty,
                        span: v.span,
                    };
                    pre.push(TStmt::Let(tmp, std::mem::replace(v, read)));
                }
            }
        }
        let mut out = Vec::with_capacity(formals.len());
        for (i, f) in formals.iter().enumerate() {
            out.push(match slot[i].and_then(|k| typed[k].take()) {
                Some(TArg::Value(v)) if f.variadic => TArg::Value(TExpr {
                    ty: v.ty.clone(),
                    span: v.span,
                    kind: TK::Pack(Box::new(v)),
                }),
                Some(a) => a,
                None => TArg::Value(match f.default {
                    Some((e, m, ref item, ref named)) => {
                        self.default_arg((e, m, item.clone()), named, &f.ty)
                    }
                    None => TExpr {
                        kind: TK::Error,
                        ty: Ty::Error,
                        span,
                    },
                }),
            });
        }
        if formals.iter().any(|f| f.written.is_some()) {
            self.check_jagged(formals, &mut out, &mut pre);
        }
        (pre, out)
    }
    /// `f(a, b)` for `fn f(n: 0..=30, k: 0..=n)`: each argument is bound to its parameter's
    /// name, so `k`'s written range reads `n` as `a`, and `b` is checked into it here, at the
    /// call, where an ordinary range parameter's argument is (R06.4.1).
    fn check_jagged(&mut self, formals: &[Formal<'_>], out: &mut [TArg], pre: &mut Vec<TStmt>) {
        self.b.scopes.push(HashMap::new());
        for (f, a) in formals.iter().zip(out.iter_mut()) {
            if let (TArg::Value(v), false) = (a, f.variadic) {
                let ty = v.ty.clone();
                let tmp = self.bind(f.name, ty.clone());
                let read = TExpr {
                    kind: TK::Var(tmp),
                    ty,
                    span: v.span,
                };
                pre.push(TStmt::Let(tmp, std::mem::replace(v, read)));
            }
        }
        for (f, a) in formals.iter().zip(out.iter()) {
            let (Some(w), TArg::Value(v)) = (f.written, a) else {
                continue;
            };
            let ty = self.body_ty_open(w, &[], v.span, pre);
            let checked = self.coerce(v.clone(), &ty);
            let tmp = self.temp("$jagged", ty);
            pre.push(TStmt::Let(tmp, checked));
        }
        self.b.scopes.pop();
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
                    Msg::verdict(
                        cpl_diag::codes::MISSING_MUT_AT_CALL,
                        "a `mut` parameter is marked `mut` at the call",
                    ),
                );
            }
            let p = self.place(inner);
            if let Some(name) = self.let_root(&p) {
                let msg = Msg::verdict(cpl_diag::codes::MUT_OF_IMMUTABLE, format!("`{name}` is bound by `let`, so it cannot be lent `mut`; declare it with `var`"));
                self.err(a.value.span, self.with_var_fix(&p, &name, msg));
            }
            if !self.unify(&p.ty, pty) {
                self.err(
                    a.value.span,
                    Msg::verdict(
                        cpl_diag::codes::TYPE_MISMATCH,
                        format!(
                            "expected `{}`, found `{}`",
                            self.show(pty),
                            self.show(&p.ty)
                        ),
                    ),
                );
            }
            TArg::Mut(p)
        } else {
            let v = self.expr(inner, Some(pty));
            let v = self.lift_item(v, pty);
            TArg::Value(self.coerce(v, pty))
        }
    }
    /// A default is read where its fn or struct is declared: none of the caller's names are seen,
    /// and the declaration's type parameters name this call's types (`id: F = F.zero`).
    fn default_arg(
        &mut self,
        (e, m, item): (&ast::Expr, ModId, ItemId),
        named: &[(String, Ty)],
        ty: &Ty,
    ) -> TExpr {
        let module = std::mem::replace(&mut self.b.module, m);
        let item = std::mem::replace(&mut self.b.item, item);
        let scopes = std::mem::replace(&mut self.b.scopes, vec![HashMap::new()]);
        let self_local = self.b.self_local.take();
        // The caller's type parameters stay unnamed but still number a lambda's: `by: T -> K =
        // x => x` read in a generic fn is an instance per the caller's type arguments.
        let unnamed = vec![String::new(); self.b.generics.len()];
        let generics = std::mem::replace(&mut self.b.generics, unnamed);
        let mut doms = self.b.doms.clone();
        doms.extend(named.iter().cloned());
        let doms = std::mem::replace(&mut self.b.doms, doms);
        let v = self.expr(e, Some(ty));
        let v = self.coerce(v, ty);
        self.b.module = module;
        self.b.item = item;
        self.b.scopes = scopes;
        self.b.self_local = self_local;
        self.b.generics = generics;
        self.b.doms = doms;
        v
    }
    /// The parameters of a fn as a call binds them.
    pub(super) fn formals(&self, sub: &Subst, fd: &'a decls::FnDecl) -> Vec<Formal<'a>> {
        let named = named_params(&fd.generics, sub);
        fd.params
            .iter()
            .map(|p| Formal {
                label: &p.label,
                name: &p.name,
                written: p.written.as_ref(),
                ty: sub.ty(&p.ty),
                mutable: p.mutable,
                named_only: p.named_only,
                variadic: p.variadic,
                default: (p.default.as_ref())
                    .map(|e| (e, fd.module, fd.item.clone(), named.clone())),
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
        let ctx = self.ctx_copy(decl);
        let f = self.d.f(decl);
        // An impl parameter only a bound names that the receiver's type cannot fix (`K` of
        // `impl<S: Indexed<K, T>, K, T>` on a `Slice<S>` of a type parameter) is left to the bound.
        let mut targs: Vec<Ty> = (given_tys.into_iter())
            .map(|t| match t {
                Ty::Error => self.new_ty(),
                t => t,
            })
            .collect();
        while targs.len() < f.generics.len() {
            targs.push(self.new_ty());
        }
        let mut sets = given_sets;
        // A fn's call of itself passes its own result sets: recursion gets no second copy.
        while sets.len() < f.n_sets + f.n_ret_sets {
            let i = sets.len();
            sets.push(match self.b.decl == Some(decl) && i >= f.n_sets {
                true => SetTy::Param(i),
                false => self.new_set(),
            });
        }
        let r = FnRef {
            decl,
            targs,
            sets,
            ctx,
        };
        if !r.sets.is_empty() {
            self.set_call(&r);
        }
        r
    }
    /// A call of std's fn marked `@marker`, with arguments already typed.
    pub(super) fn marked_call(
        &mut self,
        decl: Option<FnId>,
        marker: &str,
        args: Vec<TExpr>,
        span: Span,
    ) -> TExpr {
        let Some(decl) = decl else {
            self.err(span, Msg::internal(format!("no fn is marked `@{marker}`")));
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
    /// One `if` branch: its condition, its block checked against `want`, and whether it leaves.
    fn if_arm(
        &mut self,
        (c, b): &(ast::Cond, ast::Block),
        want: Option<&Ty>,
    ) -> (TExpr, TExpr, bool) {
        let narrows =
            matches!(c, ast::Cond::Expr(e) if matches!(e.kind, ExprKind::HasBound { .. }));
        let c = self.cond(c);
        // The branch a bound test guards sees the bound (R08.2.9), so the bounds it obliges are
        // met inside it, and those from before it outside it.
        if narrows {
            self.solve(false);
        }
        let n = self.b.assumed.len();
        if let TK::HasBound(t, r) = &c.kind {
            if let Ty::Param(i) = self.shallow(t) {
                self.b.assumed.push((i, r.clone()));
            }
        }
        // A bound test that failed to check is the error: the branch it narrows is downstream of it.
        let b2 = if narrows && self.is_error(&c.ty) {
            Self::error_node(c.span)
        } else {
            self.block(b, want)
        };
        if narrows {
            self.solve(false);
        }
        self.b.assumed.truncate(n);
        self.b.scopes.pop();
        (c, b2, b.leaves())
    }
    /// `a op b` for an ordering `op`: primitives and plain variants compare directly, any other
    /// type through its `Ord` impl.
    pub(super) fn ordered(&mut self, op: CmpOp, a: TExpr, b: TExpr, span: Span) -> TExpr {
        let mk = |kind, ty| TExpr { kind, ty, span };
        let primitive = matches!(
            self.shallow(&a.ty),
            Ty::Int | Ty::Bool | Ty::Char | Ty::U64 | Ty::I128 | Ty::F64 | Ty::Dom(_) | Ty::Error
        );
        // Characters are ordered by code (R02.2.1).
        let char = self.shallow(&a.ty) == Ty::Char;
        let side = |x: TExpr| {
            if char {
                mk(TK::CodeOf(Box::new(x)), Ty::Int)
            } else {
                x
            }
        };
        // Variants that carry nothing compare by their place in the enum (R08.1.6).
        let tags = !primitive
            && self.plain_enum(&a.ty).is_some()
            && self.shallow(&a.ty) == self.shallow(&b.ty);
        if tags {
            let (a, b) = (self.enum_tag(a), self.enum_tag(b));
            mk(TK::Cmp(op, Box::new(a), Box::new(b)), Ty::Bool)
        } else if primitive {
            mk(TK::Cmp(op, Box::new(side(a)), Box::new(side(b))), Ty::Bool)
        } else {
            self.op_call(op, a, b, span)
        }
    }
    /// `a < b`, `a == b` and the rest on a type that is not a number: its `Ord` or `Eq` fn, from
    /// the bound on a type parameter or the type's impl (R08.1.5). `a > b` is `b < a`, `a <= b` is
    /// `not b < a`, and `a != b` is `not a == b`; operands are still read left to right.
    pub(super) fn op_call(&mut self, op: CmpOp, a: TExpr, b: TExpr, span: Span) -> TExpr {
        let mk = |kind, ty| TExpr { kind, ty, span };
        // `>`, `<=` and `>=` follow from `<`, and `!=` from `==` (R08.1.5).
        let token = match op {
            CmpOp::Eq | CmpOp::Ne => "==",
            CmpOp::Lt | CmpOp::Le | CmpOp::Gt | CmpOp::Ge => "<",
            CmpOp::In | CmpOp::NotIn => return mk(TK::Error, Ty::Error),
        };
        let ops: Vec<TraitId> = self.d.operator_traits(token).collect();
        let named = (ops.iter().copied()).find(|&tr| self.d.seen(self.d.t(tr).module));
        let Some((named, method)) = named.and_then(|tr| Some((tr, self.d.operator_method(tr)?.0)))
        else {
            let msg = format!(
                "`{}` needs an `@operator(\"{token}\")` trait (R08.1.5)",
                op.text()
            );
            self.err(span, Msg::verdict(cpl_diag::codes::MISSING_IMPL, msg));
            return mk(TK::Error, Ty::Error);
        };
        let trait_name = self.d.t(named).name.clone();
        let swap = matches!(op, CmpOp::Gt | CmpOp::Le);
        let mut pre = Vec::new();
        let a = if swap && !matches!(a.kind, TK::Var(_)) {
            let tmp = self.temp("$lhs", a.ty.clone());
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
                .find(|t| ops.contains(&t.tr))
                .or_else(|| {
                    self.blanket(*i, named).then(|| decls::TraitRef {
                        tr: named,
                        args: Vec::new(),
                    })
                });
            tr.map(|tr| {
                if let Some(m) = self.d.member(&self.d.t(tr.tr).methods, method) {
                    self.reach(m);
                }
                let arg = self.coerce(arg, &rt);
                mk(
                    TK::CallTrait {
                        ctx: self.trait_ctx(tr.tr, method),
                        tr: tr.clone(),
                        method: method.to_string(),
                        self_ty: Box::new(recv.ty.clone()),
                        targs: Box::default(),
                        recv: Some(Box::new(TArg::Value(recv))),
                        args: vec![TArg::Value(arg)],
                    },
                    Ty::Bool,
                )
            })
        } else if let Some(decls::Member::Method(m, ts, ss)) = self
            .d
            .sym(method)
            .and_then(|sym| self.impls().impl_member(&rt, sym, Some(named)))
        {
            let f = self.fnref(m, ts, ss);
            let ty = self.ret_ty(&f);
            let fd = self.d.f(f.decl);
            let sub = Subst {
                tys: &f.targs,
                sets: &f.sets,
            };
            let param = fd.params.first().map_or(Ty::Error, |p| sub.ty(&p.ty));
            // The impl's own bounds hold too: `?T`'s `==` needs `T: Eq` (R08.1.5).
            if let Some(decls::Owner::Impl(k)) = fd.owner {
                let im = self.d.imp(k);
                if !im.bounds.is_empty() {
                    let tr = decls::TraitRef {
                        tr: im.trait_.tr,
                        args: im.trait_.args.iter().map(|a| sub.ty(a)).collect(),
                    };
                    (self.b.obligations).push((recv.ty.clone(), tr, span, Origin::Bound));
                }
            }
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
            if matches!(rt, Ty::Struct(id, ..) if !self.d.s(id).variants.is_empty()) {
                // Its variants may give it a structural impl (R08.1.6).
                self.err(
                    span,
                    Unbuilt::Derived {
                        tr: trait_name.clone(),
                        ty: shown,
                    },
                );
            } else {
                self.err(
                    span,
                    Msg::verdict(
                        cpl_diag::codes::MISSING_IMPL,
                        format!(
                            "`{shown}` has no `impl {trait_name}`, which `{}` needs (R08.1.5)",
                            op.text()
                        ),
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
    pub(super) fn name(&mut self, n: &str, span: Span) -> TExpr {
        if let Some(e) = self.itself(n, span) {
            return e;
        }
        if let Some((v, ty)) = self.lookup(n) {
            return TExpr {
                kind: TK::Var(v),
                ty,
                span,
            };
        }
        // A field of `self`, written bare inside a method.
        if let Some(s) = self.self_expr(span) {
            if let Some(decls::Member::Field(i, ty)) = self
                .d
                .sym(n)
                .and_then(|sym| self.impls().lookup_member(&s.ty, sym))
            {
                return TExpr {
                    kind: TK::Field(Box::new(s), i),
                    ty,
                    span,
                };
            }
        }
        if let Some(decls::Named::Const(c)) = self.d.named(self.b.module, n) {
            return self.const_read(c, span);
        }
        if let Some(d) = self.d.fn_named(self.b.module, n) {
            if self.d.roles.todo == Some(d) {
                return todo_trap(span);
            }
            let f = self.fnref(d, Vec::new(), Vec::new());
            return self.fn_item_value(f, span);
        }
        if let Some(d) = self.domain_named(n) {
            return self.domain_value(d, span);
        }
        if let Some((e, i)) = self.scoped_variant(n) {
            return self.variant(e, i, Vec::new(), &[], span);
        }
        if let Some(&(_, _, id)) = (self.b.group.iter()).find(|g| g.1 == n && g.0 > span.start) {
            let set = self.names_set();
            self.b.ahead.push((id, n.to_string(), span, set));
            return TExpr {
                kind: TK::Itself(id),
                ty: self.lambda_ty(id),
                span,
            };
        }
        if self.d.is_ambiguous(self.b.module, n) {
            self.two_imports(n, span);
        } else if !self.b.refused_names.contains(n) {
            let mut msg = Msg::verdict(
                cpl_diag::codes::UNRESOLVED_NAME,
                format!("unknown name `{n}`"),
            );
            if let Some(fix) = self.import_fix(n, span) {
                msg.fixes.push(fix);
            } else if let Some(near) = self.closest_name(n) {
                msg.help = Some(format!("did you mean `{near}`?"));
                msg.fixes.push(cpl_next_ir::msg::Fix {
                    title: format!("change to `{near}`"),
                    edits: vec![(self.b.item.at(span), near)],
                });
            }
            self.err(span, msg);
        }
        TExpr {
            kind: TK::Error,
            ty: Ty::Error,
            span,
        }
    }
    /// The name in scope that `n`, unknown, is most likely a slip for.
    fn closest_name(&self, n: &str) -> Option<String> {
        let in_scope = |&(_, &id): &(&String, &usize)| match self.b.decl {
            Some(_) => !self.b.locals[id].synthetic,
            None => !self.globals[id].synthetic,
        };
        let scoped = self.b.scopes.iter().flat_map(|s| s.iter()).filter(in_scope);
        let globals = (self.global_names.iter()).filter(|&(_, &g)| !self.globals[g].synthetic);
        let module = self.d.names(self.b.module);
        let names = (module.iter().map(|(name, _)| *name))
            .chain(globals.map(|(name, _)| name.as_str()))
            .chain(scoped.map(|(name, _)| name.as_str()));
        cpl_diag::closest(n, names)
    }

    /// The `use` line that brings in `n`, unknown at `span`, from the first std module that
    /// exports it as `pub`.
    fn import_fix(&self, n: &str, span: Span) -> Option<cpl_next_ir::msg::Fix> {
        let module = (self.d.module_names.iter().enumerate())
            .find(|(i, m)| {
                m.starts_with("std.") && self.d.export(ModId(*i), n).is_some_and(|e| e.1)
            })
            .map(|(_, m)| m.as_str())?;
        let text = self.src.text_of(&self.b.item.at(span));
        let (point, edit) = cpl_diag::import_edit(&text.text, module.strip_prefix("std.")?, n);
        Some(cpl_next_ir::msg::Fix {
            title: format!("add `use {module}.{{{n}}}`"),
            edits: vec![(text.at_offset(point.start)?, edit)],
        })
    }

    /// The value of `e`. One read from a slot of a range type is an `Int` (R03.1.4), unless it
    /// goes straight into a slot of the same range.
    /// Where node `n` is written, counted from its item's start, as every span in a body is.
    pub(super) fn at(&self, n: &impl AstNode) -> Span {
        n.syntax().span()
    }

    pub(super) fn expr(&mut self, e: &next::Expr, want: Option<&Ty>) -> TExpr {
        if self.b.rhs_extent.take_if(|at| *at == e.span).is_some() {
            return self.extent(e, want);
        }
        let v = self.expr_as_typed(e, want);
        match self.shallow(&v.ty) {
            have @ (Ty::Range(..) | Ty::RangeOf(_) | Ty::RangeTo(..))
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
            // A literal with an exponent whose value is a whole number is an `Int` (R01.4.2).
            ExprKind::Float(text) => match e.int_literal() {
                Some(i) => mk(TK::Int(i), Ty::Int),
                None => match text.parse::<f64>() {
                    Ok(x) => mk(TK::Float(x.to_bits()), Ty::F64),
                    Err(_) => {
                        self.err(
                            span,
                            Unbuilt::FloatLiteral {
                                text: text.to_string(),
                            },
                        );
                        mk(TK::Error, Ty::Error)
                    }
                },
            },
            ExprKind::Bool(b) => mk(TK::Bool(*b), Ty::Bool),
            ExprKind::Keep { target, max, value } => self.keep_expr(target, *max, value, span),
            ExprKind::Str(ps) if ps.iter().all(|p| matches!(p, StrPiece::Text(_))) => {
                let text: String = ps
                    .iter()
                    .map(|p| match p {
                        StrPiece::Text(t) => t.as_str(),
                        _ => "",
                    })
                    .collect();
                if !text.is_ascii() {
                    self.err(span, Unbuilt::NonAsciiString);
                }
                let n = mk(TK::Int(text.len() as i64), Ty::Int);
                let raw = mk(TK::Text(text), Ty::Raw(Box::new(Ty::Char)));
                self.marked_call(self.d.sugar.str_literal, "str_literal", vec![raw, n], span)
            }
            // `"a{x}b"` appends each piece in turn through the `@role(interpolation)` fn (R10.3.2).
            ExprKind::Str(ps) if let Some(put) = self.d.roles().interpolate => {
                let lit = |ps| ast::Expr {
                    kind: ExprKind::Str(ps),
                    span,
                };
                let empty = self.expr(&lit(Vec::new()), None);
                let ty = empty.ty.clone();
                let buf = self.temp("$str", ty.clone());
                match buf {
                    VarRef::Local(l) => self.b.locals[l].mutable = true,
                    VarRef::Global(g) => self.globals[g].mutable = true,
                }
                let read = mk(TK::Var(buf), ty.clone());
                let mut stmts = vec![TStmt::Let(buf, empty)];
                for p in ps {
                    let (v, width, zero, digits) = match p {
                        StrPiece::Text(t) => {
                            let v = self.expr(&lit(vec![StrPiece::Text(t.clone())]), None);
                            (v, 0, false, None)
                        }
                        StrPiece::Expr(x) => (self.expr(x, None), 0, false, None),
                        StrPiece::Spec(x, spec) => {
                            let digits = match spec.kind {
                                FmtKind::Dec => None,
                                FmtKind::Fixed(d) => Some(d),
                                FmtKind::Hex | FmtKind::Bin => {
                                    self.err(span, Unbuilt::HexFormat);
                                    continue;
                                }
                            };
                            (self.expr(x, None), spec.width, spec.zero, digits)
                        }
                    };
                    let f = self.fnref(put, Vec::new(), Vec::new());
                    let formals = self.formals(
                        &Subst {
                            tys: &f.targs,
                            sets: &f.sets,
                        },
                        self.d.f(put),
                    );
                    self.unify(&formals[0].ty, &ty);
                    self.unify(&formals[1].ty, &v.ty);
                    self.oblige(&f, span);
                    // The layout a spec gives: its width, its zero fill and its decimals.
                    let digits = match digits {
                        Some(d) => ExprKind::Int(i128::from(d)),
                        None => ExprKind::None,
                    };
                    let lay = [
                        ExprKind::Int(i128::from(width)),
                        ExprKind::Bool(zero),
                        digits,
                    ];
                    let mut args = vec![TArg::Mut(read.clone()), TArg::Value(v)];
                    for (k, kind) in lay.into_iter().enumerate() {
                        let Some(want) = formals.get(2 + k).map(|p| p.ty.clone()) else {
                            break;
                        };
                        let x = self.expr(&ast::Expr { kind, span }, Some(&want));
                        args.push(TArg::Value(self.coerce(x, &want)));
                    }
                    let call = TK::Call {
                        f,
                        recv: None,
                        args,
                    };
                    stmts.push(TStmt::Expr(mk(call, Ty::Unit)));
                }
                mk(TK::Block(stmts, Some(Box::new(read))), ty)
            }
            ExprKind::Char(c) if c.is_ascii() => {
                let code = mk(TK::Int(i64::from(*c as u8)), Ty::Int);
                mk(TK::CharOf(Box::new(code)), Ty::Char)
            }
            ExprKind::Paren(inner) => self.expr(inner, want),
            ExprKind::None => {
                let ty = match want.filter(|w| self.optional_of(w).is_some()) {
                    Some(w) => w.clone(),
                    // Wanted as the error, as beside an operand that is one: it is the error too.
                    None if want.is_some_and(|w| self.shallow(w) == Ty::Error) => {
                        return mk(TK::Error, Ty::Error);
                    }
                    None => {
                        self.err(
                            span,
                            Msg::verdict(
                                cpl_diag::codes::INFERENCE_NEEDS_ANNOTATION,
                                "nothing here says what `None` is an optional of; write the type",
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
                    if !self.mentions_error(&v.ty) {
                        let t = self.show(&v.ty);
                        self.err(span, Unbuilt::Unwrap { ty: t });
                    }
                    return mk(TK::Error, Ty::Error);
                };
                self.b.scopes.push(HashMap::new());
                let (bind, flag, val) = self.opened(v, inner.clone());
                self.b.scopes.pop();
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
                let l = match lhs.bare_try() {
                    Some(_) => self.extent(lhs, None),
                    None => self.expr(lhs, None),
                };
                let Some(inner) = self.optional_of(&l.ty) else {
                    if !self.mentions_error(&l.ty) {
                        let t = self.show(&l.ty);
                        self.err(span, Unbuilt::Coalesce { ty: t });
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
                self.b.scopes.push(HashMap::new());
                let (bind, flag, val) = self.opened(l, inner);
                self.b.scopes.pop();
                let val = if out == opt { self.some(val, opt) } else { val };
                let pick = mk(
                    TK::If(Box::new(flag), Box::new(val), Box::new(r)),
                    out.clone(),
                );
                mk(TK::Block(bind, Some(Box::new(pick))), out)
            }
            ExprKind::Name(n) => match self.expected_variant(n, want) {
                Some((e, i)) => self.variant(e, i, Vec::new(), &[], span),
                None => match self.filled(n, want, span) {
                    Some(e) => e,
                    None => match self.defaulted_fn_value(n, e, want) {
                        Some(v) => v,
                        None => self.name(n, span),
                    },
                },
            },
            ExprKind::SelfValue => self.self_expr(span).unwrap_or(mk(TK::Error, Ty::Error)),
            ExprKind::Tuple(xs) if xs.is_empty() => mk(TK::Unit, Ty::Unit),
            // `~x` flips every bit: `x ^ ones`.
            ExprKind::Unary {
                op: UnOp::BitNot,
                expr,
            } => {
                let v = self.expr(expr, None);
                let t = match self.shallow(&v.ty) {
                    Ty::U64 => Ty::U64,
                    _ => Ty::Int,
                };
                let v = self.coerce(v, &t);
                let ones = mk(TK::Int(-1), Ty::Int);
                let ones = match t {
                    Ty::U64 => mk(TK::ToU64(Box::new(ones)), Ty::U64),
                    _ => ones,
                };
                mk(TK::Bin(BinOp::BitXor, Box::new(v), Box::new(ones)), t)
            }
            ExprKind::Unary { op, expr } => {
                let (want_t, kind): (Ty, fn(Box<TExpr>) -> TK) = match op {
                    UnOp::Not => (Ty::Bool, TK::Not),
                    UnOp::Neg => (Ty::Int, TK::Neg),
                    _ => {
                        self.err(span, Unbuilt::Operator);
                        (Ty::Int, TK::Neg)
                    }
                };
                let v = self.expr(expr, Some(&want_t));
                if let Some(found) = (*op == UnOp::Neg)
                    .then(|| self.operator_trait("-", false, &v.ty))
                    .flatten()
                {
                    return self.operator_call(found, v, None, span);
                }
                let want_t = match self.shallow(&v.ty) {
                    Ty::F64 if *op == UnOp::Neg => Ty::F64,
                    Ty::I128 if *op == UnOp::Neg => Ty::I128,
                    t if *op == UnOp::Neg && self.declares_operators(&t) => {
                        let t = self.show(&t);
                        self.err(span, Unbuilt::OperatorTrait { ty: t });
                        return mk(TK::Error, Ty::Error);
                    }
                    _ => want_t,
                };
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
                        Msg::verdict(cpl_diag::codes::MIXED_COMPARISON_CHAIN, "a chain that goes both up and down; write the two comparisons with `and`"),
                    );
                }
                // `None == x`: the `None` takes its type from the other side.
                if let [(op @ (CmpOp::Eq | CmpOp::Ne), rhs)] = rest.as_slice() {
                    if matches!(super::index::unparen(first).kind, ExprKind::None) {
                        let r = self.expr(rhs, None);
                        let l = self.expr(first, Some(&r.ty));
                        if self.is_error(&l.ty) || self.is_error(&r.ty) {
                            return Self::error_node(span);
                        }
                        if let Some(test) = self.presence(*op, &l, &r, span) {
                            return test;
                        }
                        return self.ordered(*op, l, r, span);
                    }
                }
                // `a <= b < c` is `a <= b and b < c` with `b` read once, and `c` not read
                // when the first link is false (R01.5.3).
                self.b.scopes.push(HashMap::new());
                let mut prev = self.expr(first, None);
                let mut links: Vec<(Vec<TStmt>, TExpr)> = Vec::new();
                for (k, (op, rhs)) in rest.iter().enumerate() {
                    // `in` stands alone (R01.5.3); its operands are not one type.
                    if matches!(op, CmpOp::In | CmpOp::NotIn) {
                        // `x in V` on a domain named as a type: is integer `x` a position (R04.2.6).
                        let r = match self.type_path(rhs) {
                            Some(d @ Ty::Dom(_)) => Err(d),
                            _ => Ok(self.expr(rhs, None)),
                        };
                        self.b.scopes.pop();
                        if rest.len() > 1 {
                            let text = cpl_diag::msgs::check::MIXED_COMPARISON_CHAIN_CHAINED_COMPARISONS_CANNOT_CONTAIN_IN.say(&[]);
                            self.err(
                                span,
                                Msg::verdict(cpl_diag::codes::MIXED_COMPARISON_CHAIN, text),
                            );
                            return Self::error_node(span);
                        }
                        let found = match r {
                            Ok(r) => self.membership(prev, r, span),
                            Err(d) => {
                                let x = self.coerce(prev, &Ty::Int);
                                mk(TK::DomHas(d, Box::new(x)), Ty::Bool)
                            }
                        };
                        return match op {
                            CmpOp::NotIn => mk(TK::Not(Box::new(found)), Ty::Bool),
                            _ => found,
                        };
                    }
                    let r = self.expr(rhs, Some(&prev.ty));
                    // Either operand fixes the type: `u != from` with `from: ?V` lifts `u` (R04.4.23).
                    let lift =
                        self.optional_of(&prev.ty).is_none() && self.optional_of(&r.ty).is_some();
                    let r = if lift {
                        prev = self.coerce(prev, &r.ty.clone());
                        r
                    } else {
                        self.coerce(r, &prev.ty.clone())
                    };
                    // A comparison with an operand that is the error is the error, as `binary` is.
                    if self.is_error(&prev.ty) || self.is_error(&r.ty) {
                        self.b.scopes.pop();
                        return Self::error_node(span);
                    }
                    if rest.len() == 1 {
                        if let Some(test) = self.presence(*op, &prev, &r, span) {
                            self.b.scopes.pop();
                            return test;
                        }
                    }
                    // An operand two links share is held, so it is read once.
                    let mut hold = Vec::new();
                    let r = if k + 1 < rest.len() {
                        let tmp = self.temp("$link", r.ty.clone());
                        let read = mk(TK::Var(tmp), r.ty.clone());
                        hold.push(TStmt::Let(tmp, r));
                        read
                    } else {
                        r
                    };
                    let c = self.ordered(*op, prev, r.clone(), span);
                    links.push((hold, c));
                    prev = r;
                }
                self.b.scopes.pop();
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
            ExprKind::Tuple(xs) if self.d.tuple(xs.len()).is_some() => {
                // `(R, C)` named as a value is the product domain's whole range (R04.2.11).
                if let Some(p) = self.type_path(e) {
                    return self.domain_value(p, span);
                }
                let Some(d) = self.d.tuple(xs.len()) else {
                    return Self::error_node(span);
                };
                let args: Vec<ast::Arg> = (xs.iter())
                    .map(|x| ast::Arg {
                        name: None,
                        mutable: false,
                        value: x.clone(),
                    })
                    .collect();
                let targs = (0..xs.len()).map(|_| self.new_ty()).collect();
                let sets = (0..self.d.s(d).n_sets).map(|_| self.new_set()).collect();
                self.construct(d, targs, sets, &args, span, want)
            }
            // `(lo: 2, hi: 7)` is its record struct's constructor, by field name (R02.5.2).
            ExprKind::Record(fs) => {
                let names: Vec<&str> = fs.iter().map(|(n, _)| n.text.as_str()).collect();
                let callee = ast::Expr {
                    kind: ExprKind::Name(crate::records::name(&names)),
                    span,
                };
                let args = (fs.iter())
                    .map(|(n, x)| ast::Arg {
                        name: Some(n.clone()),
                        mutable: false,
                        value: x.clone(),
                    })
                    .collect();
                let call = ExprKind::Call {
                    callee: Box::new(callee),
                    args,
                };
                self.expr(&ast::Expr { kind: call, span }, want)
            }
            ExprKind::Field { base, name } => match self.owned_domain(base, name) {
                Some(d) => self.domain_value(d, span),
                None => self.member(base, name, None, span, want, &[]),
            },
            ExprKind::Index { indices, .. } if indices.len() > 1 => {
                let mut pre = Vec::new();
                let one = match self.key_ctor(e, &mut pre) {
                    Ok(Some(one)) => one,
                    Ok(None) => match super::index::packed(e) {
                        Some(one) => one,
                        None => return Self::error_node(span),
                    },
                    Err(()) => return Self::error_node(span),
                };
                let v = self.expr(&one, want);
                block(pre, v)
            }
            // A slice is contiguous, so its range takes no step (R01.8.5).
            ExprKind::Index { base, indices }
                if indices
                    .iter()
                    .any(|e| match &super::index::unparen(e).kind {
                        ExprKind::Range { step, .. } => step.is_some(),
                        ExprKind::By { walk, .. } => matches!(walk.kind, ExprKind::Range { .. }),
                        _ => false,
                    }) =>
            {
                let b = self.expr(base, None);
                let text =
                    cpl_diag::msgs::check::BAD_OPERANDS_STEPPED_SLICE.say(&[&self.show(&b.ty)]);
                let help = "walk it instead: `a[i] for i in l..r by k`".to_string();
                self.err_help(
                    span,
                    Msg::verdict(cpl_diag::codes::BAD_OPERANDS, text),
                    Some(help),
                );
                Self::error_node(span)
            }
            ExprKind::Index { base, indices } if indices.len() == 1 => {
                let again = self.b.indexed_again.take();
                let point = |e: &ast::Expr| {
                    !matches!(super::index::unparen(e).kind, ExprKind::Range { .. })
                };
                if point(&indices[0])
                    && matches!(&super::index::unparen(base).kind,
                        ExprKind::Index { indices: first, .. } if first.iter().all(point))
                {
                    self.b.indexed_again = Some((span, indices[0].span));
                }
                let b = self.expr(base, None);
                self.b.indexed_again = None;
                if self.is_error(&b.ty) {
                    return Self::error_node(span);
                }
                // A point of a table's point is `g[r, c]`, never `g[r][c]` (R01.8.5).
                if let Some((whole, j)) =
                    again.filter(|_| point(&indices[0]) && self.is_table(&b.ty))
                {
                    let text = cpl_diag::msgs::check::REMOVED_SYNTAX_INDEXES_TWICE_0_3
                        .say(&[&self.text(span), &self.text(j)]);
                    let help = "all indices go in one pair of brackets: `g[r, c]`, never `g[r][c]`";
                    let code = cpl_diag::codes::REMOVED_SYNTAX;
                    self.err_help(whole, Msg::verdict(code, text), Some(help.to_string()));
                    return Self::error_node(span);
                }
                if let Ty::Raw(t) = self.shallow(&b.ty) {
                    let i = self.expr(&indices[0], Some(&Ty::Int));
                    let i = self.coerce(i, &Ty::Int);
                    return TExpr {
                        kind: TK::Slot(Box::new(b), Box::new(i)),
                        ty: *t,
                        span,
                    };
                }
                if let ExprKind::Range {
                    lo,
                    hi,
                    inclusive,
                    step: None,
                } = &super::index::unparen(&indices[0]).kind
                {
                    if lo.is_none() || hi.is_none() {
                        let ends = (lo.as_deref(), hi.as_deref(), *inclusive);
                        return self.open_slice(b, ends, span);
                    }
                }
                let want = match &super::index::unparen(&indices[0]).kind {
                    ExprKind::Range { .. } => (self.key_of(&b.ty)).and_then(|k| self.range_over(k)),
                    ExprKind::Tuple(parts) => self.ranged_key(&b.ty, parts),
                    _ => None,
                };
                let i = self.expr(&indices[0], Some(want.as_ref().unwrap_or(&Ty::Int)));
                if self.is_error(&i.ty) {
                    return Self::error_node(span);
                }
                if let Some(f) = self.index_fn(&b.ty, &i.ty) {
                    self.index_call(f, b, vec![Some(i)], span)
                } else {
                    self.literal_key(base, &indices[0], &b.ty);
                    let i = self.index_arg(&b.ty, i);
                    self.elem(b, i, span)
                }
            }
            ExprKind::Call { callee, args } => match &callee.kind {
                ExprKind::Field { base, name } => {
                    self.member(base, name, Some(args), span, want, &[])
                }
                ExprKind::Generic { base, args: targs }
                    if matches!(base.kind, ExprKind::Field { .. }) =>
                {
                    self.generic_method(base, targs, args, span, want)
                }
                // `Int(c)`, `Char(k)`, `U64(x)`: a core type has no constructor, so this converts.
                ExprKind::Name(n)
                    if args.len() == 1
                        && args[0].name.is_none()
                        && self.lookup(n).is_none()
                        && !self.declared(n)
                        && decls::prim_type(n).is_some() =>
                {
                    let to = decls::prim_type(n).unwrap_or(Ty::Error);
                    self.convert(to, &args[0].value, span, Origin::Convert)
                }
                // `D(x)` converts into a domain, unless `D`'s constructor takes one field, as a
                // library domain's `Id(k)` does (R04.2.28).
                ExprKind::Name(n)
                    if self.lookup(n).is_none()
                        && (self.domain_named(n)).is_some_and(|t| !self.built_from(&t, 1))
                        && args.len() == 1 =>
                {
                    let dty = self.domain_named(n).unwrap_or(Ty::Error);
                    self.domain_from(dty, &args[0].value, span)
                }
                ExprKind::Name(n) if self.bare_variant(n, want).is_some() => {
                    match self.bare_variant(n, want) {
                        Some((e, i)) => self.variant(e, i, Vec::new(), args, span),
                        None => self.call_named(n, &[], args, span, want),
                    }
                }
                // A generator argument to a fn that takes no sequence is the method fold of that
                // sequence: `max(f(x) for x in xs)` is `(f(x) for x in xs).max()` (R01.10.1).
                ExprKind::Name(n) if self.declared(n) && self.fold_form(n, args) => {
                    let recv = ast::Expr {
                        kind: ExprKind::Field {
                            base: Box::new(args[0].value.clone()),
                            name: ast::Name {
                                text: n.clone(),
                                span: callee.span,
                            },
                        },
                        span,
                    };
                    let call = ast::Expr {
                        kind: ExprKind::Call {
                            callee: Box::new(recv),
                            args: args[1..].to_vec(),
                        },
                        span,
                    };
                    self.expr(&call, want)
                }
                ExprKind::Name(n) if self.lookup(n).is_none() && self.names_io(n) => {
                    let args = self.io_args(args);
                    self.call_named(n, &[], &args, span, want)
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
                ExprKind::Generic { base, args: targs }
                    if targs.len() == 1 && self.returned_by(base).is_some() =>
                {
                    let mut pre = Vec::new();
                    let t = self.body_ty(&targs[0], span, &mut pre);
                    match self.returned_by(base) {
                        Some((n, f)) => block(pre, self.call_by_return(t, f, &n, args, span)),
                        None => Self::error_node(span),
                    }
                }
                ExprKind::Name(_)
                    if want.is_some_and(|w| !matches!(self.shallow(w), Ty::Var(_)))
                        && self.returned_by(callee).is_some() =>
                {
                    let t = want.cloned().unwrap_or(Ty::Error);
                    match self.returned_by(callee) {
                        Some((n, f)) => self.call_by_return(t, f, &n, args, span),
                        None => Self::error_node(span),
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
                param_mut,
                ret_ty,
                body,
            } => match self.op_item(e, want) {
                Some(item) => item,
                None => {
                    let sig = (&params[..], &param_tys[..], &param_mut[..]);
                    self.lambda(sig, ret_ty.as_ref(), body, want, span)
                }
            },
            ExprKind::Generic { base, args } => self.generic_value(base, args, span),
            ExprKind::List(xs) => self.list_literal(xs, want, span),
            ExprKind::Map(entries) => self.map_literal(entries, want, span),
            ExprKind::Comprehension { body, clauses } => {
                self.comprehension(body, clauses, want, span)
            }
            ExprKind::Table {
                domains,
                params,
                body,
            } if !domains.is_empty() && (params.len() == 1 || params.len() == domains.len()) => {
                // `[R, C] p => e` is a table over the product `(R, C)`, `p` the cell; `r, c`
                // name its parts (R04.2.11).
                let dom = match &domains[..] {
                    [d] => d.clone(),
                    ds => ast::Type {
                        kind: TypeKind::Tuple(ds.to_vec()),
                        span,
                    },
                };
                let pat = match &params[..] {
                    [p] => p.clone(),
                    ps => ast::Pattern {
                        kind: PatternKind::Tuple(ps.to_vec()),
                        span,
                    },
                };
                self.table(&dom, &pat, body, span, want)
            }
            ExprKind::Block(b) => self.block(b, want),
            ExprKind::If {
                branches,
                otherwise,
            } => {
                let valued = otherwise.is_some();
                let was_joining = self.b.joining;
                self.b.joining = want.is_none() && valued;
                // With no expected type, the branches meet in a type of their own: the `else`'s,
                // or, when it is a bare `None`, that of the first branch giving a value. The rest
                // are checked against it.
                let mut ty = want.filter(|_| valued).cloned();
                let mut arms: Vec<Option<(TExpr, TExpr, bool)>> =
                    branches.iter().map(|_| None).collect();
                let mut else_ = None;
                if let (Some(o), None) = (otherwise, &ty) {
                    if gives_none(o) {
                        let first =
                            (branches.iter()).position(|(_, b)| !b.leaves() && !gives_none(b));
                        if let Some(i) = first {
                            let arm = self.if_arm(&branches[i], None);
                            ty = Some(self.join_ty(&arm.1.ty.clone()));
                            arms[i] = Some(arm);
                        }
                    } else {
                        self.b.joining = was_joining;
                        let e = self.block(o, None);
                        self.b.joining = want.is_none();
                        let t = self.join_ty(&e.ty.clone());
                        let e = if !o.leaves() && self.shallow(&t) != Ty::Unit {
                            self.coerce(e, &t)
                        } else {
                            e
                        };
                        ty = Some(t);
                        else_ = Some(e);
                    }
                }
                for (arm, br) in arms.iter_mut().zip(branches) {
                    if arm.is_none() {
                        *arm = Some(self.if_arm(br, ty.as_ref()));
                    }
                }
                self.b.joining = was_joining;
                let checked = else_.is_some();
                let else_ = match (else_, otherwise) {
                    (Some(e), _) => e,
                    (None, Some(o)) => self.block(o, ty.as_ref()),
                    (None, None) => mk(TK::Unit, Ty::Unit),
                };
                let ty = match ty {
                    _ if !valued => Ty::Unit,
                    Some(t) => t,
                    None => self.join_ty(&else_.ty.clone()),
                };
                let converts = valued && self.shallow(&ty) != Ty::Unit;
                let else_leaves = otherwise.as_ref().is_some_and(|o| o.leaves());
                let mut tail = if converts && !else_leaves && !checked {
                    self.coerce(else_, &ty)
                } else {
                    else_
                };
                for (c, b, leaves) in arms.into_iter().flatten().rev() {
                    // A branch that leaves gives no value to convert.
                    let b = if converts && !leaves {
                        self.coerce(b, &ty)
                    } else {
                        b
                    };
                    tail = mk(TK::If(Box::new(c), Box::new(b), Box::new(tail)), ty.clone());
                }
                tail
            }
            ExprKind::Match { scrutinee, arms } => self.match_(scrutinee, arms, want, span),
            ExprKind::Range {
                lo,
                hi,
                inclusive,
                step,
            } if lo.is_some() || hi.is_some() => {
                let (lo, hi) = (lo.as_deref(), hi.as_deref());
                self.range_value(lo, hi, *inclusive, step.as_deref(), want, span)
            }
            other @ ExprKind::Range { step: None, .. } => match self.whole_of(want) {
                Some(d) => self.domain_value(d, span),
                // Its domain comes from its place: an index or a `Range<D>` slot (R04.8.6).
                None if matches!(
                    other,
                    ExprKind::Range {
                        lo: None,
                        hi: None,
                        ..
                    }
                ) =>
                {
                    let text = cpl_diag::msgs::check::TYPE_MISMATCH_FULL_RANGE.say(&[]);
                    self.err(span, Msg::verdict(cpl_diag::codes::TYPE_MISMATCH, text));
                    Self::error_node(span)
                }
                None => self.unbuilt_expr(other, span),
            },
            ExprKind::By { walk, step } => match &walk.kind {
                ExprKind::Range {
                    lo: Some(lo),
                    hi: Some(hi),
                    inclusive,
                    step: None,
                } => self.range_value(Some(lo), Some(hi), *inclusive, Some(step), want, span),
                _ => self.step_walk(walk, step, span),
            },
            ExprKind::With { base, updates } => self.record_update(base, updates, span),
            ExprKind::Without {
                prefix,
                base,
                removed,
            } => self.without(prefix, base, removed, want, span),
            ExprKind::Try(x) => self.question(x, span),
            ExprKind::HasBound { param, bound } => self.has_bound(param, bound, span),
            other => self.unbuilt_expr(other, span),
        }
    }
    fn unbuilt_expr(&mut self, kind: &ExprKind, span: Span) -> TExpr {
        let kind = crate::kind_name(kind);
        self.err(span, Unbuilt::Expression { kind });
        Self::error_node(span)
    }
    pub(super) fn self_field(&self, n: &str) -> Option<usize> {
        let s = self.self_expr(Span::new(0, 0))?;
        match self
            .d
            .sym(n)
            .and_then(|sym| self.impls().lookup_member(&s.ty, sym))
        {
            Some(decls::Member::Field(i, _)) => Some(i),
            _ => None,
        }
    }

    /// A name the program declares, not a local or a field of `self`.
    pub(super) fn declared(&self, n: &str) -> bool {
        self.lookup(n).is_none()
            && self.itself(n, Span::new(0, 0)).is_none()
            && self.self_field(n).is_none()
            && self.d.named(self.b.module, n).is_some()
    }

    /// A call of a declared fn or struct, with the type arguments written at the call.
    /// `print`'s `sep:` and `precision:` from the innermost `with io` where it is written, unless
    /// the call names them (R09.1.2, R10.3.4).
    /// Whether `n` names the fn filling the `print` or `write` role here (R11.6.1).
    fn names_io(&self, n: &str) -> bool {
        let r = &self.d.roles;
        matches!(self.d.named(self.b.module, n), Some(Named::Fn(f)) if [r.print, r.write].contains(&Some(f)))
    }
    fn io_args(&self, args: &[ast::Arg]) -> Vec<ast::Arg> {
        let mut out = args.to_vec();
        let Some(Named::Context(io)) = self.d.named(self.b.module, "io") else {
            return out;
        };
        for field in ["sep", "precision"] {
            let named = |a: &ast::Arg| a.name.as_ref().is_some_and(|n| n.text == field);
            let set = (self.b.frames.iter().rev()).find(|x| x.ctx == io && x.field == field);
            if let (false, Some(x)) = (args.iter().any(named), set) {
                out.push(ast::Arg {
                    name: Some(ast::Name {
                        text: field.into(),
                        span: x.value.span,
                    }),
                    mutable: false,
                    value: x.value.clone(),
                });
            }
        }
        out
    }
}

/// A parameter as a call binds it.
pub(super) struct Formal<'d> {
    pub label: &'d str,
    /// What its fn's body calls it.
    pub name: &'d str,
    /// A jagged parameter's written range (`k: 0..=n`), read with each earlier parameter's name
    /// bound to its argument.
    pub written: Option<&'d ast::Type>,
    pub ty: Ty,
    pub mutable: bool,
    pub named_only: bool,
    pub variadic: bool,
    /// Its expression, read in the module and with spans from the item that declares it, and
    /// the types the declaration's type parameters stand for at this call.
    pub default: Option<DefaultAt<'d>>,
}

/// A default's expression, its module and item, and its type parameters' types at the call.
pub(super) type DefaultAt<'d> = (&'d ast::Expr, ModId, ItemId, Vec<(String, Ty)>);

/// Each type parameter's name and the type `sub` gives it.
pub(super) fn named_params(generics: &[String], sub: &Subst) -> Vec<(String, Ty)> {
    (generics.iter().enumerate())
        .map(|(i, g)| (g.clone(), sub.ty(&Ty::Param(i))))
        .collect()
}

/// Reading it has no effect, so when it runs does not matter.
pub(super) fn pure(e: &TExpr) -> bool {
    match &e.kind {
        TK::Int(_)
        | TK::Bool(_)
        | TK::Unit
        | TK::Var(_)
        | TK::FnItem(_)
        | TK::Itself(_)
        | TK::Lambda { .. } => true,
        TK::Field(b, _) => pure(b),
        _ => false,
    }
}

pub(super) fn block(pre: Vec<TStmt>, e: TExpr) -> TExpr {
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

/// Whether block `b` ends in a bare `None`, which takes its type from the other branches.
fn gives_none(b: &ast::Block) -> bool {
    matches!(b.stmts.last().map(|s| &s.kind),
        Some(ast::StmtKind::Expr(e)) if matches!(super::index::unparen(e).kind, ExprKind::None))
}

/// The `@role(todo)` fn read as a value: it traps where it is reached and gives no value (R01.6.7).
fn todo_trap(span: Span) -> TExpr {
    let never = TExpr {
        kind: TK::Bool(false),
        ty: Ty::Bool,
        span,
    };
    let trap = TStmt::Assert(never, "`todo` reached".into(), span, CheckKind::Todo);
    TExpr {
        kind: TK::Block(vec![trap], None),
        ty: Ty::Unit,
        span,
    }
}

/// Whether `tok` is the word `None`, which always means the empty optional, whatever is in scope:
/// the one literal word the lexer leaves a name (`true` and `false` are tokens of their own).
pub(super) fn none_word(tok: &cpl_next_syntax::SyntaxToken) -> bool {
    tok.kind() == cpl_next_syntax::TokenKind::Ident
        && cpl_next_syntax::LITERAL_WORDS.contains(&tok.text())
}

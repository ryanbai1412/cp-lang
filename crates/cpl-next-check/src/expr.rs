//! Expressions: operators, calls, members, names.

use super::*;
use cpl_next_ir::ids::ItemId;
use cpl_next_ir::ops::Fmt;
use cpl_next_ir::unbuilt::Unbuilt;
use cpl_syntax::ast::FmtKind;

impl<'a> Cx<'a> {
    /// Std's `@list` struct, which literals, comprehensions and tables build.
    pub(super) fn list_struct(&mut self, span: Span) -> Option<StructId> {
        let list = self.d.sugar.list;
        if list.is_none() {
            self.err(span, Msg::internal("no struct is marked `@list`"));
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
        let want = match (logic, unsigned, float) {
            (true, ..) => Ty::Bool,
            (_, _, true) => Ty::F64,
            (_, true, _) => Ty::U64,
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
        let overflow = if self.wraps && want == Ty::Int {
            Overflow::Wraps
        } else {
            Overflow::Traps
        };
        TExpr {
            kind: TK::Bin(op, Box::new(l), Box::new(r), overflow),
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
        let found = (self.d.operators.iter().filter(|(o, _)| o == text)).find_map(|(_, tr)| {
            let (n, &m) = self.d.traits[tr.0].methods.iter().next()?;
            (self.d.f(m).params.len() == usize::from(binary)).then(|| (*tr, n.clone(), m))
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
    /// `x in s` as `s`'s core `@operator("in")` trait fn, `contains(x)` (R12.4.7).
    fn membership(&mut self, x: TExpr, s: TExpr, span: Span) -> TExpr {
        if self.is_error(&x.ty) || self.is_error(&s.ty) {
            return Self::error_node(span);
        }
        let found = (self.d.operators.iter().filter(|(o, _)| o == "in")).find_map(|(_, tr)| {
            let (n, &m) = self.d.traits[tr.0].methods.iter().next()?;
            Some((*tr, n.clone(), m))
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
        (self.obligations).push((self_ty.clone(), tr.clone(), span, Origin::Bound));
        TExpr {
            kind: TK::CallTrait {
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
            && matches!(self.d.views[self.module.0].get(n), Some(decls::Named::Fn(f)) if self.d.f(*f).self_ty.is_none())
    }
    /// The arguments past the others gathered into the one a last `..xs` parameter takes: a
    /// tuple when positional, a record when named, in written order (R06.1.5, decision 324).
    fn pack_variadic(
        &mut self,
        args: &[ast::Arg],
        formals: &[Formal<'a>],
        span: Span,
    ) -> Option<Vec<ast::Arg>> {
        let (v, fixed) = formals.split_last().filter(|(v, _)| v.variadic)?;
        let mut positional = fixed.iter().filter(|f| !f.named_only).count();
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
            (_, true) => ExprKind::Tuple(tuple),
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
        formals: &[Formal<'a>],
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
            self.obligations.retain(|o| o.2 != span);
        }
        // A lambda with unwritten parameter types is checked after the other arguments, whose
        // types and bounds may fix the ones it takes (R06.1.2).
        let late = |a: &ast::Arg| {
            matches!(&a.value.kind, ExprKind::Lambda { params, param_tys, .. }
                if params.len() > param_tys.iter().flatten().count())
        };
        let mut typed: Vec<Option<TArg>> = (0..args.len()).map(|_| None).collect();
        for pass in [false, true] {
            if pass {
                self.solve(false);
            }
            for (k, a) in args.iter().enumerate() {
                if late(a) != pass {
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
                    Some((e, m, ref item)) => self.default_arg(e, m, item.clone(), &f.ty),
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
                    Msg::verdict(
                        cpl_diag::codes::MISSING_MUT_AT_CALL,
                        "a `mut` parameter is marked `mut` at the call",
                    ),
                );
            }
            let p = self.place(inner);
            if let Some(name) = self.let_root(&p) {
                self.err(
                    a.value.span,
                    Msg::verdict(cpl_diag::codes::MUT_OF_IMMUTABLE, format!("`{name}` is bound by `let`, so it cannot be lent `mut`; declare it with `var`")),
                );
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
    /// A default is read where its fn or struct is declared: none of the caller's names are seen.
    fn default_arg(&mut self, e: &ast::Expr, m: ModId, item: ItemId, ty: &Ty) -> TExpr {
        let module = std::mem::replace(&mut self.module, m);
        let item = std::mem::replace(&mut self.item, item);
        let scopes = std::mem::replace(&mut self.scopes, vec![HashMap::new()]);
        let self_local = self.self_local.take();
        let v = self.expr(e, Some(ty));
        let v = self.coerce(v, ty);
        self.module = module;
        self.item = item;
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
                variadic: p.variadic,
                default: p.default.as_ref().map(|e| (e, fd.module, fd.item.clone())),
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
            sets.push(match self.decl == Some(decl) && i >= f.n_sets {
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
            self.set_calls.push((r.clone(), self.decl));
        }
        r
    }
    /// A call of the library fn `name` with arguments already typed.
    pub(super) fn lib_call(&mut self, name: &str, args: Vec<TExpr>, span: Span) -> TExpr {
        let decl = self.d.core_fn(name);
        self.marked_call(decl, name, args, span)
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
    /// One value of a `print`: a plain one joins `pieces`, one the library writes
    /// (R10.3.2) becomes a call of its writer after what was gathered so far.
    fn show_value(&mut self, v: TExpr, fmt: Fmt, stmts: &mut Vec<TStmt>, pieces: &mut Vec<Piece>) {
        let span = v.span;
        let t = self.shallow(&v.ty);
        // A value built from an error (`?<error>`) was diagnosed where the error is.
        if t != Ty::Error && self.mentions_error(&t) {
            return pieces.push(Piece::Value(Self::error_node(span), fmt));
        }
        let number = matches!(
            t,
            Ty::Int | Ty::U64 | Ty::F64 | Ty::Dom(_) | Ty::Param(_) | Ty::Error
        );
        if fmt != Fmt::default() && !number
            || fmt.digits.is_some() && !matches!(t, Ty::F64 | Ty::Error)
        {
            self.err(span, Unbuilt::FormatSpec { ty: self.show(&t) });
            return;
        }
        let writer = match t {
            // A position prints as its integer (R04.2.15).
            Ty::Dom(_) | Ty::Param(_) => {
                let ord = TExpr {
                    kind: TK::DomOrd(Box::new(v)),
                    ty: Ty::Int,
                    span,
                };
                pieces.push(Piece::Value(ord, fmt));
                return;
            }
            // `Bool` has no one way to be written: the word is the program's choice (R10.3.3).
            Ty::Bool => {
                let code = cpl_diag::codes::MISSING_IMPL;
                self.err(
                    span,
                    Msg::verdict(code, "`Bool` is not `Show`: write `print(yes(b))`"),
                );
                return;
            }
            Ty::Int | Ty::Char | Ty::U64 | Ty::F64 | Ty::Error => {
                pieces.push(Piece::Value(v, fmt));
                return;
            }
            // A type that says how it is written (`fn write(self)`) is written by it.
            t @ Ty::Struct(..)
                if let Some(decls::Member::Method(m, ts, ss)) =
                    self.d.lookup_member(&t, "write") =>
            {
                if !pieces.is_empty() {
                    stmts.push(TStmt::Expr(TExpr {
                        kind: TK::Print(std::mem::take(pieces)),
                        ty: Ty::Unit,
                        span,
                    }));
                }
                let call = self.call_member(v, (m, ts, ss), vec![], span, false);
                stmts.push(TStmt::Expr(call));
                return;
            }
            Ty::Struct(d, ..) => [self.d.sugar.list, self.d.sugar.array]
                .into_iter()
                .zip(["write_list", "write_array"])
                .find(|(ty, _)| Some(d) == *ty)
                .map(|(_, f)| f),
            _ => None,
        };
        let Some(writer) = writer else {
            let t = self.show(&v.ty);
            self.err(span, Unbuilt::Printing { ty: t });
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
    /// One `if` branch: its condition, its block checked against `want`, and whether it leaves.
    fn if_arm(
        &mut self,
        (c, b): &(ast::Cond, ast::Block),
        want: Option<&Ty>,
    ) -> (TExpr, TExpr, bool) {
        let narrows =
            matches!(c, ast::Cond::Expr(e) if matches!(e.kind, ExprKind::HasBound { .. }));
        let c = self.cond(c);
        // The branch a bound test guards sees the bound (R08.2.9).
        let n = self.assumed.len();
        if let TK::HasBound(t, r) = &c.kind {
            if let Ty::Param(i) = self.shallow(t) {
                self.assumed.push((i, r.clone()));
            }
        }
        // A bound test that failed to check is the error: the branch it narrows is downstream of it.
        let b2 = if narrows && self.is_error(&c.ty) {
            Self::error_node(c.span)
        } else {
            self.block(b, want)
        };
        self.assumed.truncate(n);
        self.scopes.pop();
        (c, b2, b.leaves())
    }
    /// `a op b` for an ordering `op`: primitives and plain variants compare directly, any other
    /// type through its `Ord` impl.
    pub(super) fn ordered(&mut self, op: CmpOp, a: TExpr, b: TExpr, span: Span) -> TExpr {
        let mk = |kind, ty| TExpr { kind, ty, span };
        let primitive = matches!(
            self.shallow(&a.ty),
            Ty::Int | Ty::Bool | Ty::Char | Ty::U64 | Ty::F64 | Ty::Dom(_) | Ty::Error
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
                .find(|t| self.d.traits[t.tr.0].name == trait_name)
                .or_else(|| {
                    let tr = TraitId(
                        (self.d.traits.iter())
                            .position(|t| t.name == trait_name && self.d.seen(t.module))?,
                    );
                    self.blanket(*i, tr).then(|| decls::TraitRef {
                        tr,
                        args: Vec::new(),
                    })
                });
            tr.map(|tr| {
                if let Some(&m) = self.d.traits[tr.tr.0].methods.get(method) {
                    self.reach(m);
                }
                let arg = self.coerce(arg, &rt);
                mk(
                    TK::CallTrait {
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
            // The impl's own bounds hold too: `?T`'s `==` needs `T: Eq` (R08.1.5).
            if let Some(decls::Owner::Impl(k)) = fd.owner {
                let im = &self.d.impls[k];
                if !im.bounds.is_empty() {
                    let tr = decls::TraitRef {
                        tr: im.trait_.tr,
                        args: im.trait_.args.iter().map(|a| sub.ty(a)).collect(),
                    };
                    (self.obligations).push((recv.ty.clone(), tr, span, Origin::Bound));
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
                        tr: trait_name.to_string(),
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
    /// How a method takes its receiver: a `mut self` (or a cache it fills) needs a place.
    /// A method call; a getter with nothing to pass is read like a field, so `()` after it calls
    /// what it reads (R01.8.7, R02.6.6).
    pub(super) fn call_method(
        &mut self,
        f: FnRef,
        recv: TArg,
        getter: bool,
        call: Option<&[ast::Arg]>,
        span: Span,
        want: Option<&Ty>,
    ) -> TExpr {
        match call {
            Some(args) if getter => {
                let v = self.call_fn(f, Some(recv), &[], span, None);
                self.call_value(v, args, span)
            }
            _ => self.call_fn(f, Some(recv), call.unwrap_or(&[]), span, want),
        }
    }
    /// The receiver of a method that changes it, lent `mut`: never a `let` binding (R05.4.2).
    pub(super) fn mut_recv(&mut self, recv: TExpr, span: Span) -> TArg {
        if let Some(name) = self.let_root(&recv) {
            self.err(
                span,
                Msg::verdict(cpl_diag::codes::MUT_OF_IMMUTABLE, format!("`{name}` is bound by `let`, so it cannot be lent `mut`; declare it with `var`")),
            );
        }
        TArg::Mut(recv)
    }
    pub(super) fn recv_arg(&mut self, m: FnId, recv: TExpr, span: Span) -> TArg {
        let place = matches!(recv.kind, TK::Var(_) | TK::Field(..));
        if self.d.f(m).self_mode == SelfMode::Mut {
            if !place && !self.is_error(&recv.ty) {
                self.err(
                    span,
                    Msg::verdict(
                        cpl_diag::codes::MUT_OF_IMMUTABLE,
                        "a `mut self` method needs a place",
                    ),
                );
            }
            self.mut_recv(recv, span)
        } else if self.d.writes_cache(m) && place {
            TArg::Mut(recv)
        } else {
            TArg::Value(recv)
        }
    }
    /// `@same_keys`: arrays over two different domains are not paired by position (R04b.1.3).
    fn same_keys(&mut self, args: &[TArg], span: Span) {
        let mut first: Option<usize> = None;
        for a in args {
            let (TArg::Value(e) | TArg::Mut(e)) = a;
            let Ty::Struct(d, ts, _) = self.shallow(&e.ty) else {
                continue;
            };
            let index = if Some(d) == self.array_id() {
                self.shallow(&ts[0])
            } else {
                Ty::Error
            };
            let Ty::Dom(i) = index else {
                continue;
            };
            match first {
                None => first = Some(i),
                Some(f) if self.domains[f].name != self.domains[i].name || f != i => {
                    let (a, b) = (self.domains[f].name.clone(), self.domains[i].name.clone());
                    self.err(
                        span,
                        Msg::verdict(cpl_diag::codes::WRONG_DOMAIN, format!("an array over `{b}` is paired with one over `{a}`: their positions are unrelated")),
                    );
                    return;
                }
                Some(_) => {}
            }
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
        // A call of a fn refused as declared is downstream of that refusal: it is the error.
        if (fd.errors.iter()).any(|e| e.refusal.is_some()) {
            return Self::error_node(span);
        }
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
        let variadic = formals.last().is_some_and(|f| f.variadic);
        if fd.module == decls::CORE && args.len() > formals.len() && !variadic {
            self.err(
                span,
                Unbuilt::CoreStandInArity {
                    name: fd.name.clone(),
                    given: args.len(),
                    takes: formals.len(),
                },
            );
            return TExpr {
                kind: TK::Error,
                ty: Ty::Error,
                span,
            };
        }
        let held = fd.region.is_some() && !std::mem::take(&mut self.allow_region);
        let unbuilt = held && (fd.region.as_ref().is_some_and(|r| r.0) || self.d.recipe.is_none());
        if unbuilt {
            self.err(
                span,
                Unbuilt::GeneratorAsValue {
                    name: fd.name.clone(),
                },
            );
        }
        self.oblige(&f, span);
        self.solve(false);
        let seen = self.errors.len();
        let (mut pre, args) = self.args(args, &formals, span);
        // A call its arguments do not fit was diagnosed there, and one not built is refused:
        // its value is the error, and the bounds its type arguments would have met are not asked.
        if unbuilt || self.errors.len() > seen {
            self.obligations.retain(|o| o.2 != span);
            return Self::error_node(span);
        }
        self.solve(false);
        if fd.same_keys {
            self.same_keys(&args, span);
        }
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
        if held && self.d.recipe.is_some() {
            return self.recipe(pre, call);
        }
        block(pre, call)
    }
    /// `recv.name(args)` of a trait bound on the type parameter `recv` is.
    pub(super) fn bound_call(
        &mut self,
        p: usize,
        recv: TExpr,
        name: &str,
        args: Vec<TArg>,
        span: Span,
    ) -> TExpr {
        for tr in &self.bound_traits(p) {
            let Some(m) = self.d.traits[tr.tr.0].methods.get(name).copied() else {
                continue;
            };
            // A lender is called only where its place is used (`for`, an assignment): elsewhere
            // the bound's reading fn of that name is meant (`IndexedMut.at` vs `Indexed.at`).
            if self.d.f(m).region.is_some() && !self.allow_region {
                continue;
            }
            self.reach(m);
            let mut tys = vec![recv.ty.clone()];
            tys.extend(tr.args.iter().cloned());
            let ret = self.trait_ret(m, tys);
            return TExpr {
                kind: TK::CallTrait {
                    tr: tr.clone(),
                    method: name.to_string(),
                    self_ty: Box::new(recv.ty.clone()),
                    targs: Box::default(),
                    recv: Some(Box::new(TArg::Value(recv))),
                    args,
                },
                ty: ret,
                span,
            };
        }
        let shown = self.show(&recv.ty);
        self.err(
            span,
            Msg::verdict(
                cpl_diag::codes::UNKNOWN_FIELD,
                format!("`{shown}` has no `{name}`"),
            ),
        );
        TExpr {
            kind: TK::Error,
            ty: Ty::Error,
            span,
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
                };
            }
            t => {
                self.err(
                    span,
                    Msg::verdict(
                        cpl_diag::codes::NOT_CALLABLE,
                        format!("`{}` cannot be called", self.show(&t)),
                    ),
                );
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
                variadic: false,
                default: None,
            })
            .collect();
        if args
            .iter()
            .any(|a| a.mutable || matches!(a.value.kind, ExprKind::Mut(_)))
        {
            self.err(span, Unbuilt::MutArgToFnValue);
        }
        let seen = self.errors.len();
        let (pre, args) = self.args(args, &formals, span);
        if self.errors.len() > seen {
            return Self::error_node(span);
        }
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
            if let Some(decls::Member::Field(i, ty)) = self.d.lookup_member(&s.ty, n) {
                return TExpr {
                    kind: TK::Field(Box::new(s), i),
                    ty,
                    span,
                };
            }
        }
        if let Some(decls::Named::Const(c)) = self.d.named(self.module, n) {
            return self.const_read(c, span);
        }
        if let Some(d) = self.d.fn_named(self.module, n) {
            let f = self.fnref(d, Vec::new(), Vec::new());
            return TExpr {
                kind: TK::FnItem(f.clone()),
                ty: Ty::Clo(Box::new(CloRef::Item(f))),
                span,
            };
        }
        if let Some(d) = self.domain_named(n) {
            return self.domain_value(d, span);
        }
        if let Some((e, i)) = self.scoped_variant(n) {
            return self.variant(e, i, Vec::new(), &[], span);
        }
        // `todo` traps where it is reached and gives no value (R01.6.7).
        if n == "todo" {
            let never = TExpr {
                kind: TK::Bool(false),
                ty: Ty::Bool,
                span,
            };
            let trap = TStmt::Assert(never, "`todo` reached".into(), span, CheckKind::Todo);
            return TExpr {
                kind: TK::Block(vec![trap], None),
                ty: Ty::Unit,
                span,
            };
        }
        if let Some(&(_, _, id)) = (self.group.iter()).find(|g| g.1 == n && g.0 > span.start) {
            let set = self.names_set();
            self.ahead.push((id, n.to_string(), span, set));
            return TExpr {
                kind: TK::Itself(id),
                ty: self.lambda_ty(id),
                span,
            };
        }
        if self.d.ambiguous[self.module.0].contains(n) {
            self.two_imports(n, span);
        } else if !self.refused_names.contains(n) {
            self.err(
                span,
                Msg::verdict(
                    cpl_diag::codes::UNRESOLVED_NAME,
                    format!("unknown name `{n}`"),
                ),
            );
        }
        TExpr {
            kind: TK::Error,
            ty: Ty::Error,
            span,
        }
    }
    /// The value of `e`. One read from a slot of a range type is an `Int` (R03.1.4), unless it
    /// goes straight into a slot of the same range.
    pub(super) fn expr(&mut self, e: &ast::Expr, want: Option<&Ty>) -> TExpr {
        if self.rhs_extent.take_if(|at| *at == e.span).is_some() {
            return self.extent(e, want);
        }
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
                    if self.shallow(&v.ty) != Ty::Error {
                        let t = self.show(&v.ty);
                        self.err(span, Unbuilt::Unwrap { ty: t });
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
                let l = match lhs.bare_try() {
                    Some(_) => self.extent(lhs, None),
                    None => self.expr(lhs, None),
                };
                let Some(inner) = self.optional_of(&l.ty) else {
                    if self.shallow(&l.ty) != Ty::Error {
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
            ExprKind::Name(n) => match self.expected_variant(n, want) {
                Some((e, i)) => self.variant(e, i, Vec::new(), &[], span),
                None => match self.filled(n, want, span) {
                    Some(e) => e,
                    None => self.name(n, span),
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
                mk(
                    TK::Bin(BinOp::BitXor, Box::new(v), Box::new(ones), Overflow::Traps),
                    t,
                )
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
                // `a <= b < c` is `a <= b and b < c` with `b` read once, and `c` not read
                // when the first link is false (R01.5.3).
                self.scopes.push(HashMap::new());
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
                        self.scopes.pop();
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
                        self.scopes.pop();
                        return Self::error_node(span);
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
                    let c = self.ordered(*op, prev, r.clone(), span);
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
            ExprKind::Tuple(xs) if self.d.tuples.contains_key(&xs.len()) => {
                // `(R, C)` named as a value is the product domain's whole range (R04.2.11).
                if let Some(p) = self.type_path(e) {
                    return self.domain_value(p, span);
                }
                let d = self.d.tuples[&xs.len()];
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
            ExprKind::Field { base, name } => self.member(base, name, None, span, want, &[]),
            ExprKind::Index { indices, .. } if indices.len() > 1 => match super::index::packed(e) {
                Some(one) => self.expr(&one, want),
                None => Self::error_node(span),
            },
            ExprKind::Index { base, indices } if indices.len() == 1 => {
                let b = self.expr(base, None);
                if self.is_error(&b.ty) {
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
                // `write` is `print` without the line end; the slice uses it to write `print` for containers in the library.
                ExprKind::Name(n) if (n == "print" || n == "write") && self.lookup(n).is_none() => {
                    // `print` reads `io` where it is written; a call's argument beats it (R09.1.2).
                    let d = self.d;
                    let io = match d.named(self.module, "io") {
                        Some(decls::Named::Context(id)) => Some(id),
                        _ => None,
                    };
                    let io_set = |cx: &Self, f: &str| {
                        (cx.frames.iter().rev())
                            .find(|x| Some(x.ctx) == io && x.field == f)
                            .map(|x| x.value.kind.clone())
                    };
                    let text = |k: Option<ExprKind>| match k {
                        Some(ExprKind::Str(ps)) => match &ps[..] {
                            [] => Some(String::new()),
                            [StrPiece::Text(t)] => Some(t.clone()),
                            _ => None,
                        },
                        _ => None,
                    };
                    let mut sep = text(io_set(self, "sep")).unwrap_or_else(|| " ".into());
                    // `precision: k` sets the digits of each `F64` it prints (R10.3.4).
                    let mut precision = match io_set(self, "precision") {
                        Some(ExprKind::Int(k)) => u32::try_from(k).ok(),
                        _ => None,
                    };
                    for a in args.iter().filter(|a| a.name.is_some()) {
                        match (a.name.as_ref().map(|n| n.text.as_str()), &a.value.kind) {
                            (Some("precision"), ExprKind::Int(k)) if u32::try_from(*k).is_ok() => {
                                precision = u32::try_from(*k).ok()
                            }
                            (Some("sep"), k) if text(Some(k.clone())).is_some() => {
                                sep = text(Some(k.clone())).unwrap_or_default()
                            }
                            _ => {
                                self.err(span, Unbuilt::PrintNamedArgs);
                                return TExpr {
                                    kind: TK::Error,
                                    ty: Ty::Error,
                                    span,
                                };
                            }
                        }
                    }
                    let fmt_of = |cx: &Self, v: &TExpr| match cx.shallow(&v.ty) {
                        Ty::F64 => Fmt {
                            digits: precision,
                            ..Fmt::default()
                        },
                        Ty::Int | Ty::U64 | Ty::Char | Ty::Dom(_) | Ty::Param(_) | Ty::Error => {
                            Fmt::default()
                        }
                        // The library writes it, and is not told the precision.
                        _ if precision.is_some() => Fmt {
                            digits: precision,
                            ..Fmt::default()
                        },
                        _ => Fmt::default(),
                    };
                    let args: Vec<_> = args.iter().filter(|a| a.name.is_none()).collect();
                    // Text and plain values gather into one `Print`; a value the library
                    // writes ends the one before it and is a call of its writer.
                    let mut stmts = Vec::new();
                    let mut pieces = Vec::new();
                    for (k, a) in args.iter().enumerate() {
                        if k > 0 {
                            pieces.push(Piece::Text(sep.clone()));
                        }
                        match &a.value.kind {
                            ExprKind::Str(ps) => {
                                for p in ps {
                                    match p {
                                        StrPiece::Text(t) => pieces.push(Piece::Text(t.clone())),
                                        StrPiece::Expr(x) => {
                                            let v = self.expr(x, None);
                                            let fmt = fmt_of(self, &v);
                                            self.show_value(v, fmt, &mut stmts, &mut pieces);
                                        }
                                        StrPiece::Spec(x, spec) => {
                                            let digits = match spec.kind {
                                                FmtKind::Dec => None,
                                                FmtKind::Fixed(d) => Some(d),
                                                FmtKind::Hex | FmtKind::Bin => {
                                                    self.err(span, Unbuilt::HexFormat);
                                                    continue;
                                                }
                                            };
                                            let fmt = Fmt {
                                                width: spec.width,
                                                zero: spec.zero,
                                                digits,
                                            };
                                            let v = self.expr(x, None);
                                            self.show_value(v, fmt, &mut stmts, &mut pieces);
                                        }
                                    }
                                }
                            }
                            _ => {
                                let v = self.expr(&a.value, None);
                                let fmt = fmt_of(self, &v);
                                self.show_value(v, fmt, &mut stmts, &mut pieces);
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
                ExprKind::Name(n)
                    if self.lookup(n).is_none()
                        && self.domain_named(n).is_some()
                        && args.len() == 1 =>
                {
                    let dty = self.domain_named(n).unwrap_or(Ty::Error);
                    self.domain_from(dty, &args[0].value, span)
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
                ret_ty,
                body,
                ..
            } => match self.op_item(e, want) {
                Some(item) => item,
                None => self.lambda(params, param_tys, ret_ty.as_ref(), body, want, span),
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
                let was_joining = self.joining;
                self.joining = want.is_none() && valued;
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
                        self.joining = was_joining;
                        let e = self.block(o, None);
                        self.joining = want.is_none();
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
                self.joining = was_joining;
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
                _ => {
                    self.err(span, Unbuilt::ByOnWalk);
                    mk(TK::Error, Ty::Error)
                }
            },
            ExprKind::With { base, updates } => self.record_update(base, updates, span),
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
        match self.d.lookup_member(&s.ty, n) {
            Some(decls::Member::Field(i, _)) => Some(i),
            _ => None,
        }
    }

    /// A name the program declares, not a local or a field of `self`.
    fn declared(&self, n: &str) -> bool {
        self.lookup(n).is_none()
            && self.itself(n, Span::new(0, 0)).is_none()
            && self.self_field(n).is_none()
            && self.d.named(self.module, n).is_some()
    }
}

/// A parameter as a call binds it.
pub(super) struct Formal<'d> {
    pub label: &'d str,
    pub ty: Ty,
    pub mutable: bool,
    pub named_only: bool,
    pub variadic: bool,
    /// Its expression, read in the module and with spans from the item that declares it.
    pub default: Option<(&'d ast::Expr, ModId, ItemId)>,
}

/// Reading it has no effect, so when it runs does not matter.
fn pure(e: &TExpr) -> bool {
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

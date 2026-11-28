//! Expressions: operators, calls, members, names.

use super::call::CallArg;
use super::lambda::{LamBody, LamParam};
use super::member::Called;
use super::*;
use cpl_next_decls::decls::Role;
use cpl_next_ir::unbuilt::Unbuilt;
use cpl_next_syntax::literal::{self, FmtKind};
use cpl_next_syntax::ops::Kind;
use cpl_next_syntax::TokenKind;

impl<'a> Cx<'a> {
    /// Std's `@role(list)` struct, which literals, comprehensions and tables build.
    pub(super) fn list_struct(&mut self, span: Span) -> Option<StructId> {
        let list = self.role_structs.role_struct(Role::List);
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
            .then(|| self.operator_trait(&op.to_string(), true, &l.ty))
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
        // The right side is the operator trait's for `want`: `U64 ** e` takes `Pow`'s
        // `Rhs = Int`, as a shift's count is an `Int` (R08.1.5).
        let rhs = match shift {
            true => Ty::Int,
            false => self.op_rhs(op, &want).unwrap_or_else(|| want.clone()),
        };
        let l = self.coerce(l, &want);
        let r = self.coerce(r, &rhs);
        if op == BinOp::Coalesce {
            self.err(span, Unbuilt::Operator);
        }
        TExpr {
            kind: TK::Bin(op, Box::new(l), Box::new(r)),
            ty: want,
            span,
        }
    }
    /// The right side the impl of `op`'s binary trait for `t` takes, as its first trait argument.
    fn op_rhs(&self, op: BinOp, t: &Ty) -> Option<Ty> {
        let binary = |tr: &TraitId| {
            (self.rows.operator_method(*tr)).is_some_and(|(_, m)| self.items.f(m).params.len() == 1)
        };
        let ops = self.rows.operator_traits(self.syms.get(&op.to_string())?);
        let tr = *ops.iter().find(|tr| binary(tr))?;
        let any = decls::TraitRef {
            tr,
            args: Vec::new(),
        };
        let impls = self.impls();
        let [(k, binds)] = &impls.best(t, &any)[..] else {
            return None;
        };
        let sub = Subst {
            tys: binds,
            sets: &[],
        };
        Some(sub.ty(impls.imp(*k).trait_.args.first()?))
    }
    /// A struct, whose operators are its impls of the operator traits (R08.1.5).
    fn declares_operators(&self, t: &Ty) -> bool {
        matches!(self.shallow(t), Ty::Struct(..))
    }
    /// The `@operator(text)` trait, binary or prefix, and its fn, when `t`'s operator is that
    /// trait's: `t` is a struct, or a type parameter bound by the trait.
    fn operator_trait(&self, text: &str, binary: bool, t: &Ty) -> Option<(TraitId, String, FnId)> {
        let traits = self.rows.operator_traits(self.syms.get(text)?);
        let found = traits.into_iter().find_map(|tr| {
            let (n, m) = self.rows.operator_method(tr)?;
            let n = self.syms.text(n).to_string();
            (self.items.f(m).params.len() == usize::from(binary)).then_some((tr, n, m))
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
    fn step_walk(&mut self, walk: &next::Expr, step: &next::Expr, span: Span) -> TExpr {
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
    pub(super) fn membership(&mut self, x: TExpr, s: TExpr, span: Span) -> TExpr {
        if self.is_error(&x.ty) || self.is_error(&s.ty) {
            return Self::error_node(span);
        }
        let traits =
            (self.syms.get("in")).map_or_else(Vec::new, |op| self.rows.operator_traits(op));
        let found = traits.into_iter().find_map(|tr| {
            let (n, m) = self.rows.operator_method(tr)?;
            Some((tr, self.syms.text(n).to_string(), m))
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
        (self.b.obligations).push((
            self_ty.clone(),
            tr.clone(),
            self.b.item.at(span),
            Origin::Bound,
        ));
        TExpr {
            kind: TK::CallTrait {
                ctx: self.trait_ctx(tr.tr, &method),
                tr,
                method,
                self_ty: Box::new(self_ty.clone()),
                targs: Box::default(),
                sets: Box::default(),
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
        let text =
            cpl_diag::msgs::check::BAD_OPERANDS_CAN_T_COMBINE_AND.say(&[&op.to_string(), &l, &r]);
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
    /// A reference to a fn with fresh type and set arguments after `given`.
    pub(super) fn fnref(
        &mut self,
        decl: FnId,
        given_tys: Vec<Ty>,
        given_sets: Vec<SetTy>,
    ) -> FnRef {
        self.reach(decl);
        let ctx = self.ctx_copy(decl);
        let f = self.items.f(decl);
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
        let fd = self.items.f(f.decl);
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
        let ops = (self.syms.get(token)).map_or_else(Vec::new, |op| self.rows.operator_traits(op));
        let named = (ops.iter().copied()).find(|&tr| self.doms.seen(self.items.t(tr).module));
        let Some((named, method)) =
            named.and_then(|tr| Some((tr, self.syms.text(self.rows.operator_method(tr)?.0))))
        else {
            let msg = format!("`{}` needs an `@operator(\"{token}\")` trait (R08.1.5)", op);
            self.err(span, Msg::verdict(cpl_diag::codes::MISSING_IMPL, msg));
            return mk(TK::Error, Ty::Error);
        };
        let trait_name = self.items.t(named).name.clone();
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
                if let Some(m) = self.method_in(&self.items.t(tr.tr).methods, method) {
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
                        sets: Box::default(),
                        recv: Some(Box::new(TArg::Value(recv))),
                        args: vec![TArg::Value(arg)],
                    },
                    Ty::Bool,
                )
            })
        } else if let Some(decls::Member::Method(m, ts, ss)) = self
            .syms
            .get(method)
            .and_then(|sym| self.impls().impl_member(&rt, sym, Some(named)))
        {
            let f = self.fnref(m, ts, ss);
            let ty = self.ret_ty(&f);
            let fd = self.items.f(f.decl);
            let sub = Subst {
                tys: &f.targs,
                sets: &f.sets,
            };
            let param = fd.params.first().map_or(Ty::Error, |p| sub.ty(&p.ty));
            // The impl's own bounds hold too: `?T`'s `==` needs `T: Eq` (R08.1.5).
            if let Some(decls::Owner::Impl(k)) = fd.owner {
                let im = self.items.imp(k);
                if !im.bounds.is_empty() {
                    let tr = decls::TraitRef {
                        tr: im.trait_.tr,
                        args: im.trait_.args.iter().map(|a| sub.ty(a)).collect(),
                    };
                    (self.b.obligations).push((
                        recv.ty.clone(),
                        tr,
                        self.b.item.at(span),
                        Origin::Bound,
                    ));
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
            if matches!(rt, Ty::Struct(id, ..) if !self.items.s(id).variants.is_empty()) {
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
                            op
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
    /// Whether bare name `n` is the `@role(todo)` fn, found in [`Cx::name`]'s order: a nearer
    /// binding or a const of that name hides it (R01.6.7).
    pub(super) fn traps(&self, n: &next::NameRef) -> bool {
        let Some(t) = n.ident() else { return false };
        let n = t.text();
        !self.hidden(n)
            && (self.fn_named(self.b.module, n))
                .is_some_and(|d| self.rows.role_fn(Role::Todo) == Some(d))
    }
    /// Whether bare name `n` is the empty optional: the word `None`, unless a binding or a const
    /// of that name is nearer, in [`Cx::name`]'s order.
    pub(super) fn names_none(&self, n: &next::NameRef) -> bool {
        n.is_none() && n.ident().is_some_and(|t| !self.hidden(t.text()))
    }
    /// Whether a name `n` resolves before the module's fns: a local or parameter, the fn
    /// itself, a field of `self`, or a const.
    fn hidden(&self, n: &str) -> bool {
        self.lookup(n).is_some()
            || self.itself(n, Span::new(0, 0)).is_some()
            || self.self_field(n).is_some()
            || matches!(self.named(self.b.module, n), Some(decls::Named::Const(_)))
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
                .syms
                .get(n)
                .and_then(|sym| self.impls().lookup_member(&s.ty, sym))
            {
                return TExpr {
                    kind: TK::Field(Box::new(s), i),
                    ty,
                    span,
                };
            }
        }
        if let Some(decls::Named::Const(c)) = self.named(self.b.module, n) {
            return self.const_read(c, span);
        }
        if let Some(d) = self.fn_named(self.b.module, n) {
            if self.rows.role_fn(Role::Todo) == Some(d) {
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
        if let Some(&(_, _, id)) =
            (self.b.group.iter()).find(|g| g.1.as_deref() == Some(n) && g.0 > span.start)
        {
            let set = self.names_set();
            self.b.ahead.push((id, n.to_string(), span, set));
            return TExpr {
                kind: TK::Itself(id),
                ty: self.lambda_ty(id),
                span,
            };
        }
        if self.is_ambiguous(self.b.module, n) {
            self.two_imports(n, span);
        } else if self.old_optional_spelling(n, span) {
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
        let module = self.names.names(self.b.module);
        let names = (module.iter().map(|(name, _)| self.syms.text(*name)))
            .chain(globals.map(|(name, _)| name.as_str()))
            .chain(scoped.map(|(name, _)| name.as_str()));
        cpl_diag::closest(n, names)
    }

    /// The `use` line that brings in `n`, unknown at `span`, from the first std module that
    /// exports it as `pub`.
    fn import_fix(&self, n: &str, span: Span) -> Option<cpl_next_ir::msg::Fix> {
        let at = *self.names.exporters(self.syms.get(n)?).first()?;
        let module = self.names.module_name(at);
        let text = self.bodies.source(&self.b.item.at(span));
        let (point, edit) = cpl_diag::use_edit(&text.text, module, n);
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
        let at = self.at(e);
        if self.b.rhs_extent.take_if(|x| *x == at).is_some() {
            return self.extent(e, want);
        }
        let v = self.expr_as_typed(e, want);
        match self.shallow(&v.ty) {
            have if have.is_int_range()
                && want.map(|w| self.shallow(w)).as_ref() != Some(&have) =>
            {
                TExpr { ty: Ty::Int, ..v }
            }
            _ => v,
        }
    }
    pub(super) fn expr_as_typed(&mut self, e: &next::Expr, want: Option<&Ty>) -> TExpr {
        let span = self.at(e);
        match e {
            next::Expr::Literal(l) => self.literal(l, want, span),
            next::Expr::Name(n) if self.names_none(n) => self.none_value(want, span),
            next::Expr::Name(n) => match n.ident() {
                Some(t) if t.kind() == TokenKind::SelfKw => self
                    .self_expr(span)
                    .unwrap_or_else(|| Self::error_node(span)),
                Some(t) => self.name_value(t.text(), want, span),
                None => Self::error_node(span),
            },
            next::Expr::Paren(p) => match p.inner() {
                Some(x) => self.expr(&x, want),
                None => Self::error_node(span),
            },
            next::Expr::Tuple(t) => self.tuple(t, want, span),
            next::Expr::List(l) => self.list(l, want, span),
            next::Expr::Map(m) => self.map_literal(m, want, span),
            next::Expr::Comp(c) => self.comp_of(c.body(), c.clauses().collect(), want, span),
            next::Expr::Generator(g) => self.comp_of(g.body(), g.clauses().collect(), want, span),
            next::Expr::Table(t) => self.table(t, span, want),
            next::Expr::Record(r) => match r.rest() {
                Some(rest) if r.is_update() => match rest.expr() {
                    Some(base) => self.record_update(&base, &r.fields().collect::<Vec<_>>(), span),
                    None => Self::error_node(span),
                },
                Some(rest) => {
                    let prefix: Vec<next::RecordField> = r.fields().collect();
                    self.without(&prefix, &rest, want, span)
                }
                None => self.record_value(r, want, span),
            },
            next::Expr::Generic(g) => self.generic_value(g, span),
            next::Expr::OpValue(o) => self.op_value(o, want, span),
            next::Expr::Block(b) => self.block_expr(b, want),
            next::Expr::FieldFn(f) => self.field_fn(f, want, span),
            next::Expr::With(w) => match w.base() {
                Some(base) => {
                    let updates: Vec<next::RecordField> = w.fields().collect();
                    self.record_update(&base, &updates, span)
                }
                None => Self::error_node(span),
            },
            next::Expr::Prefix(p) => self.prefix(p, want, span),
            next::Expr::Bin(b) => self.bin_expr(b, want, span),
            next::Expr::Compare(c) => self.compare(c, want, span),
            next::Expr::Range(r) => self.range(r, want, span),
            next::Expr::Call(c) => self.call_value_of(c, want, span),
            next::Expr::Index(i) => self.index_value(i, span),
            next::Expr::Field(f) => self.field_value(f, want, span),
            next::Expr::Try(t) => match t.inner() {
                Some(x) => self.question(&x, span),
                None => Self::error_node(span),
            },
            next::Expr::Unwrap(u) => self.unwrap_expr(u, want, span),
            next::Expr::Keep(k) => match (k.target(), k.value()) {
                (Some(t), Some(v)) => self.keep_expr(&t, k.is_max(), &v, span),
                _ => Self::error_node(span),
            },
            next::Expr::Filled(f) => match (f.target(), f.value()) {
                (Some(t), Some(v)) => self.fill_expr(&t, &v, span),
                _ => Self::error_node(span),
            },
            next::Expr::Lambda(l) => self.lambda_expr(l, want, span),
            next::Expr::If(i) => self.if_expr(i, want, span),
            next::Expr::Match(m) => self.match_(m, want, span),
            // `else: return None` where a value goes: the statement, giving no value.
            next::Expr::Return(r) => self.leaving(next::Stmt::Return(r.clone()), span),
            next::Expr::Break(b) => self.leaving(next::Stmt::Break(b.clone()), span),
            next::Expr::Continue(c) => self.leaving(next::Stmt::Continue(c.clone()), span),
            next::Expr::Hole(_)
            | next::Expr::Intrinsic(_)
            | next::Expr::Mut(_)
            | next::Expr::Is(_)
            | next::Expr::Fill(_) => self.unbuilt_expr(e, span),
        }
    }
    /// E0410 for each leave under `node` from inside a part of a value (R01.7.6), once each
    /// though a body is checked once per copy.
    pub(super) fn leaves_from_parts(&mut self, node: &cpl_next_syntax::tree::SyntaxNode) {
        for (at, word) in cpl_next_syntax::visit::leaves_from_parts(node) {
            let site = self.b.item.at(at);
            if !self.left_parts.insert(site) {
                continue;
            }
            let text = cpl_diag::msgs::check::LEAVES_A_PART_OF_A_VALUE.say(&[&word]);
            let help = format!(
                "give the `if` or `match` its own `let` first, then use the name: `let v = if ..: .. else: {word}`"
            );
            self.err_help(
                at,
                Msg::verdict(cpl_diag::codes::LEAVES_A_PART, text),
                Some(help),
            );
        }
    }
    fn leaving(&mut self, s: next::Stmt, span: Span) -> TExpr {
        let mut out = Vec::new();
        self.stmt(&s, &mut out);
        TExpr {
            kind: TK::Block(out, None),
            ty: Ty::Unit,
            span,
        }
    }
    fn unbuilt_expr(&mut self, e: &next::Expr, span: Span) -> TExpr {
        let kind = cpl_next_decls::kind_name(e);
        self.err(span, Unbuilt::Expression { kind });
        Self::error_node(span)
    }
    /// `(x, y) => e`, its parameters' and result's types as written.
    fn lambda_expr(&mut self, l: &next::LambdaExpr, want: Option<&Ty>, span: Span) -> TExpr {
        let Some(body) = l.body() else {
            return Self::error_node(span);
        };
        let mut pre = Vec::new();
        let mut params = Vec::new();
        for p in l.params().map(|ps| ps.lambda_params()).unwrap_or_default() {
            let ty = p.ty.map(|t| self.body_ty(&t, &mut pre));
            params.push(LamParam {
                pat: Some(p.pattern),
                ty,
                mutable: p.is_mut,
            });
        }
        let ret = l.ret().map(|t| self.body_ty(&t, &mut pre));
        let v = self.lambda_of(params, ret, LamBody::Arm(body), want, span);
        block(pre, v)
    }
    /// A bare operator as a value (`fold(+, xs)`): its trait's fn item at a bound, else the
    /// lambda `(a, b) => a op b`; `and`/`or` take `Bool`s.
    fn op_value(&mut self, o: &next::OpValue, want: Option<&Ty>, span: Span) -> TExpr {
        let kind = (o.op())
            .and_then(|t| cpl_next_syntax::ops::binary(t.kind()))
            .map(|b| b.kind);
        let kind = match kind {
            Some(
                k @ (Kind::Bin(_)
                | Kind::Cmp(
                    CmpOp::Lt | CmpOp::Le | CmpOp::Gt | CmpOp::Ge | CmpOp::Eq | CmpOp::Ne,
                )),
            ) => k,
            _ => return self.unbuilt_expr(&next::Expr::OpValue(o.clone()), span),
        };
        if let Kind::Bin(op) = kind {
            if let Some(item) = self.op_item(op, want, span) {
                return item;
            }
        }
        let logic = matches!(kind, Kind::Bin(BinOp::And | BinOp::Or));
        let param = || LamParam {
            pat: None,
            ty: logic.then_some(Ty::Bool),
            mutable: false,
        };
        let body = LamBody::Built(Box::new(move |cx: &mut Cx, args: Vec<TExpr>, _: &Ty| {
            let mut args = args.into_iter();
            let (Some(a), Some(b)) = (args.next(), args.next()) else {
                return Self::error_node(span);
            };
            match kind {
                Kind::Bin(op) => cx.binary(op, a, b, span),
                Kind::Cmp(op) => match cx.presence(op, &a, &b, span) {
                    Some(test) => test,
                    None => cx.ordered(op, a, b, span),
                },
            }
        }));
        self.lambda_of(vec![param(), param()], None, body, want, span)
    }
    /// `.name` as a value: the lambda `it => it.name`.
    fn field_fn(&mut self, f: &next::FieldFn, want: Option<&Ty>, span: Span) -> TExpr {
        let Some(tok) = f.field() else {
            return Self::error_node(span);
        };
        let (text, at) = (tok.text().to_string(), tok.span());
        let param = LamParam {
            pat: None,
            ty: None,
            mutable: false,
        };
        let body = LamBody::Built(Box::new(move |cx: &mut Cx, args: Vec<TExpr>, _: &Ty| {
            let Some(it) = args.into_iter().next() else {
                return Self::error_node(span);
            };
            let it = TExpr { span: at, ..it };
            let name = Called {
                text: &text,
                sym: cx.syms.get(&text),
                span: at,
            };
            cx.member_on(it, name, None, span, None, &[])
        }));
        self.lambda_of(vec![param], None, body, want, span)
    }
    /// A name as a value: a variant the expected enum has, a declared fn filled to the fn
    /// wanted, or what the name is bound to.
    pub(super) fn name_value(&mut self, n: &str, want: Option<&Ty>, span: Span) -> TExpr {
        if let Some((e, i)) = self.expected_variant(n, want) {
            return self.variant(e, i, Vec::new(), &[], span);
        }
        if let Some(e) = self.filled(n, want, span) {
            return e;
        }
        match self.defaulted_fn_value(n, span, want) {
            Some(v) => v,
            None => self.name(n, span),
        }
    }
    /// `(a: x, b: y)`: a call of the record type its field names make (R02.6.3).
    fn record_value(&mut self, r: &next::RecordExpr, want: Option<&Ty>, span: Span) -> TExpr {
        let fields: Vec<next::RecordField> = r.fields().collect();
        let names: Vec<String> = (fields.iter())
            .map(|f| f.name().map_or_else(String::new, |t| t.text().to_string()))
            .collect();
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        let callee = crate::records::name(&refs);
        let args: Vec<CallArg> = (fields.iter().zip(&names))
            .map(|(f, n)| CallArg::labelled(Some(n.clone()), f.value(), self.at(f)))
            .collect();
        self.call_named(&callee, &[], &args, span, want)
    }
    /// `base.name`: a domain a type owns, or a member of the value.
    fn field_value(&mut self, f: &next::FieldExpr, want: Option<&Ty>, span: Span) -> TExpr {
        let (Some(base), Some(name)) = (f.base(), f.name()) else {
            return Self::error_node(span);
        };
        match self.owned_domain(&base, &name) {
            Some(d) => self.domain_value(d, span),
            None => {
                let at = name.span();
                let name = Called::written(self.syms, name.text(), at);
                self.member(&base, name, None, span, want, &[])
            }
        }
    }
    /// `s[i]`, `s[i, j]`, `s[l..r]`: an element through the type's index, a slot of a raw
    /// buffer, or a slice (R01.8.5, R04.4.11).
    fn index_value(&mut self, ix: &next::IndexExpr, span: Span) -> TExpr {
        let Some(base) = ix.base() else {
            return Self::error_node(span);
        };
        let indices: Vec<next::Expr> = ix.indices().collect();
        let range_of = |e: &next::Expr| match super::index::unparen(e.clone()) {
            next::Expr::Range(r) => Some(r),
            _ => None,
        };
        // A slice is contiguous, so its range takes no step (R01.8.5).
        if indices
            .iter()
            .any(|e| range_of(e).is_some_and(|r| r.step().is_some()))
        {
            let b = self.expr(&base, None);
            let text = cpl_diag::msgs::check::BAD_OPERANDS_STEPPED_SLICE.say(&[&self.show(&b.ty)]);
            let help = "walk it instead: `a[i] for i in l..r by k`".to_string();
            let msg = Msg::verdict(cpl_diag::codes::BAD_OPERANDS, text);
            self.err_help(span, msg, Some(help));
            return Self::error_node(span);
        }
        if let [one] = &indices[..] {
            return self.index_one(&base, one, span);
        }
        let b = self.expr(&base, None);
        if self.is_error(&b.ty) {
            return Self::error_node(span);
        }
        let Some(i) = self.index_parts(&b.ty, &indices, span) else {
            return Self::error_node(span);
        };
        match self.index_fn(&b.ty, &i.ty) {
            Some(f) => self.index_call(f, b, vec![Some(i)], span),
            None => {
                let i = self.index_arg(&b.ty, i);
                self.elem(b, i, span)
            }
        }
    }
    fn index_one(&mut self, base: &next::Expr, key: &next::Expr, span: Span) -> TExpr {
        let again = self.b.indexed_again.take();
        let point =
            |e: &next::Expr| !matches!(super::index::unparen(e.clone()), next::Expr::Range(_));
        let inner_points = match super::index::unparen(base.clone()) {
            next::Expr::Index(first) => first.indices().all(|e| point(&e)),
            _ => false,
        };
        if point(key) && inner_points {
            self.b.indexed_again = Some((span, self.at(key)));
        }
        let b = self.expr(base, None);
        self.b.indexed_again = None;
        if self.is_error(&b.ty) {
            return Self::error_node(span);
        }
        // A point of a table's point is `g[r, c]`, never `g[r][c]` (R01.8.5).
        if let Some((whole, j)) = again.filter(|_| point(key) && self.is_table(&b.ty)) {
            let text = cpl_diag::msgs::check::REMOVED_SYNTAX_INDEXES_TWICE_0_3
                .say(&[&self.text(span), &self.text(j)]);
            let help = "all indices go in one pair of brackets: `g[r, c]`, never `g[r][c]`";
            let code = cpl_diag::codes::REMOVED_SYNTAX;
            self.err_help(whole, Msg::verdict(code, text), Some(help.to_string()));
            return Self::error_node(span);
        }
        if let Ty::Raw(t) = self.shallow(&b.ty) {
            let i = self.expr(key, Some(&Ty::Int));
            let i = self.coerce(i, &Ty::Int);
            return TExpr {
                kind: TK::Slot(Box::new(b), Box::new(i)),
                ty: *t,
                span,
            };
        }
        let k = super::index::unparen(key.clone());
        if let next::Expr::Range(r) = &k {
            if r.step().is_none() && (r.lo().is_none() || r.hi().is_none()) {
                let (lo, hi) = (r.lo(), r.hi());
                return self.open_slice(b, (lo.as_ref(), hi.as_ref(), r.is_inclusive()), span);
            }
        }
        let want = match &k {
            next::Expr::Range(_) => (self.key_of(&b.ty)).and_then(|k| self.range_over(k)),
            next::Expr::Tuple(t) => {
                let parts: Vec<next::Expr> = t.elems().collect();
                self.ranged_key(&b.ty, &parts)
            }
            _ => None,
        };
        // A branching key takes its type from its branches, so `par[if c: y else: x]` reads a
        // position of `par`'s domain when its branches are positions.
        let branches = matches!(k, next::Expr::If(_) | next::Expr::Match(_));
        let hint = want.as_ref().or((!branches).then_some(&Ty::Int));
        let i = self.expr(key, hint);
        if self.is_error(&i.ty) {
            return Self::error_node(span);
        }
        match self.index_fn(&b.ty, &i.ty) {
            Some(f) => self.index_call(f, b, vec![Some(i)], span),
            None => {
                self.literal_key(base, key, &b.ty);
                let i = self.index_arg(&b.ty, i);
                self.elem(b, i, span)
            }
        }
    }
    /// `callee(args)`: a method, a conversion into a core type or domain, a variant, a fold of
    /// a generator, a declared fn, a fn chosen by the type it returns, or a fn value.
    fn call_value_of(&mut self, c: &next::CallExpr, want: Option<&Ty>, span: Span) -> TExpr {
        let Some(callee) = c.callee() else {
            return Self::error_node(span);
        };
        let written: Vec<next::Arg> = c.args().map(|l| l.args().collect()).unwrap_or_default();
        let args: Vec<CallArg> = written.iter().map(CallArg::of).collect();
        let lone = match &written[..] {
            [a] if a.label().is_none() => a.value(),
            _ => None,
        };
        let name = match &callee {
            next::Expr::Name(n) => n.ident().map(|t| t.text().to_string()),
            _ => None,
        };
        let unbound = |cx: &Self, n: &str| cx.lookup(n).is_none();
        // Inside a fn a trait owns, a bare name of the trait's own member without `self`, a
        // supertrait's included, is `Self.n`: past the locals, before the enclosing scopes
        // (R08.1.12).
        let owned = (self.b.decl)
            .is_some_and(|f| matches!(self.items.f(f).owner, Some(decls::Owner::Trait(_))));
        if let Some(n) = name.as_deref().filter(|n| owned && unbound(self, n)) {
            let own = Called::written(self.syms, n, self.at(&callee));
            let pick = |fd: &decls::FnDecl| {
                fd.self_mode == SelfMode::None && super::member::takes(fd, Some(&args))
            };
            let found = own.sym.is_some_and(|sym| {
                (self.bound_traits(0).into_iter()).any(|tr| {
                    (self.items.t(tr.tr).methods.get(&sym)).is_some_and(|m| pick(self.items.f(*m)))
                })
            });
            if found {
                return match self.bound_giving(0, own, pick, span) {
                    Ok(Some((tr, m))) => {
                        self.static_call(Ty::Param(0), tr, m, own, Some(&args), span)
                    }
                    Ok(None) => Self::error_node(span),
                    Err(two) => self.two_traits_give(two, n, &Ty::Param(0), own, "", span),
                };
            }
        }
        match (&callee, name.as_deref()) {
            (next::Expr::Field(f), _) => {
                let (Some(base), Some(m)) = (f.base(), f.name()) else {
                    return Self::error_node(span);
                };
                let name = Called::written(self.syms, m.text(), m.span());
                self.member(&base, name, Some(&args), span, want, &[])
            }
            (next::Expr::Generic(g), _) if matches!(g.base(), Some(next::Expr::Field(_))) => {
                self.generic_method(g, &args, span, want)
            }
            // `Int(c)`, `Char(k)`, `U64(x)`: a core type has no constructor, so this converts.
            (_, Some(n))
                if lone.is_some()
                    && unbound(self, n)
                    && !self.declared(n)
                    && decls::prim_type(n).is_some() =>
            {
                let to = decls::prim_type(n).unwrap_or(Ty::Error);
                let x = lone.unwrap_or(callee.clone());
                self.convert(to, &x, span, Origin::Convert)
            }
            // `D(x)` converts into a domain, unless `D`'s constructor takes one field, as a
            // library domain's `Id(k)` does (R04.2.28).
            (_, Some(n))
                if unbound(self, n)
                    && (self.domain_named(n)).is_some_and(|t| !self.built_from(&t, 1))
                    && written.len() == 1 =>
            {
                let dty = self.domain_named(n).unwrap_or(Ty::Error);
                match written[0].value() {
                    Some(x) => self.domain_from(dty, &x, span),
                    None => Self::error_node(span),
                }
            }
            (_, Some(n)) if self.bare_variant(n, want).is_some() => {
                match self.bare_variant(n, want) {
                    Some((e, i)) => self.variant(e, i, Vec::new(), &args, span),
                    None => self.call_named(n, &[], &args, span, want),
                }
            }
            // A `@free_form` trait method's plain call is the method call on its first argument,
            // so a type's own member or override is the call, as `x.n(..)` would pick (R01.10.1).
            (_, Some(n)) if unbound(self, n) && self.free_form_call(n, &args) => {
                let at = self.at(&callee);
                if let Some(v) = self.free_form_tests(n, &args, at, span, want) {
                    return v;
                }
                let Some(recv) = written[0].value() else {
                    return Self::error_node(span);
                };
                let name = Called::written(self.syms, n, at);
                self.member(&recv, name, Some(&args[1..]), span, want, &[])
            }
            // A generator argument to a fn that takes no sequence is the method fold of that
            // sequence: `max(f(x) for x in xs)` is `(f(x) for x in xs).max()` (R01.10.1).
            (_, Some(n)) if self.declared(n) && self.fold_form(n, &args) => {
                let Some(seq) = written[0].value() else {
                    return Self::error_node(span);
                };
                let at = self.at(&callee);
                let name = Called::written(self.syms, n, at);
                self.member(&seq, name, Some(&args[1..]), span, want, &[])
            }
            (_, Some(n)) if unbound(self, n) && self.names_io(n) => {
                let args = self.io_args(&args);
                self.call_named(n, &[], &args, span, want)
            }
            (_, Some(n)) if self.declared(n) => self.call_named(n, &[], &args, span, want),
            // `Box<Int>(5)`, `List<Int>()`: the written type arguments come first.
            (next::Expr::Generic(g), _) if self.declared_base(g).is_some() => {
                let n = self.declared_base(g).unwrap_or_default();
                let targs: Vec<next::Type> = super::member::written_types(g);
                self.call_named(&n, &targs, &args, span, want)
            }
            (next::Expr::Generic(g), _) if self.returned_by_generic(g).is_some() => {
                let Some((t, n, f)) = self.returned_by_generic(g) else {
                    return Self::error_node(span);
                };
                let mut pre = Vec::new();
                let t = self.body_ty(&t, &mut pre);
                let v = self.call_by_return(t, f, &n, &args, span);
                block(pre, v)
            }
            (next::Expr::Name(_), _)
                if want.is_some_and(|w| !matches!(self.shallow(w), Ty::Var(_)))
                    && self.returned_by(&callee).is_some() =>
            {
                let t = want.cloned().unwrap_or(Ty::Error);
                match self.returned_by(&callee) {
                    Some((n, f)) => self.call_by_return(t, f, &n, &args, span),
                    None => Self::error_node(span),
                }
            }
            _ => {
                let c = self.expr(&callee, None);
                self.call_value(c, &args, span)
            }
        }
    }
    /// `n` of `n<T..>` when it is a declared name.
    fn declared_base(&self, g: &next::GenericExpr) -> Option<String> {
        let next::Expr::Name(n) = g.base()? else {
            return None;
        };
        let n = n.ident()?.text().to_string();
        self.declared(&n).then_some(n)
    }
    /// `f<T>(..)` of a fn chosen by the one type it returns: that type, and the fn.
    fn returned_by_generic(
        &self,
        g: &next::GenericExpr,
    ) -> Option<(next::Type, String, (TraitId, FnId))> {
        let targs: Vec<next::Type> = super::member::written_types(g);
        let [t] = &targs[..] else {
            return None;
        };
        let (n, f) = self.returned_by(&g.base()?)?;
        Some((t.clone(), n, f))
    }
    /// `if c: a elif d: b else: e`: with no expected type, the branches meet in a type of their
    /// own: the `else`'s, or, when it is a bare `None`, that of the first branch giving a
    /// value. The rest are checked against it.
    pub(super) fn if_chain(
        &mut self,
        branches: Vec<(next::Condition, next::ArmBody)>,
        otherwise: Option<next::ArmBody>,
        want: Option<&Ty>,
        span: Span,
    ) -> TExpr {
        let mk = |kind, ty| TExpr { kind, ty, span };
        let valued = otherwise.is_some();
        let was_joining = self.b.joining;
        self.b.joining = want.is_none() && valued;
        let mut ty = want.filter(|_| valued).cloned();
        let mut arms: Vec<Option<(TExpr, TExpr, bool)>> = branches.iter().map(|_| None).collect();
        let mut else_ = None;
        if let (Some(o), None) = (&otherwise, &ty) {
            let none = |n: &next::NameRef| self.names_none(n);
            if gives_none(o, &none) {
                let first = (branches.iter())
                    .position(|(_, b)| !b.leaves(&|n| self.traps(n)) && !gives_none(b, &none));
                if let Some(i) = first {
                    let arm = self.if_arm(&branches[i], None);
                    ty = Some(self.join_ty(&arm.1.ty.clone()));
                    arms[i] = Some(arm);
                }
            } else {
                self.b.joining = was_joining;
                let e = self.arm_body(o, None);
                self.b.joining = want.is_none();
                let t = self.join_ty(&e.ty.clone());
                let e = match !o.leaves(&|n| self.traps(n)) && self.shallow(&t) != Ty::Unit {
                    true => self.coerce(e, &t),
                    false => e,
                };
                ty = Some(t);
                else_ = Some(e);
            }
        }
        for (arm, br) in arms.iter_mut().zip(&branches) {
            if arm.is_none() {
                *arm = Some(self.if_arm(br, ty.as_ref()));
            }
        }
        self.b.joining = was_joining;
        let checked = else_.is_some();
        let else_ = match (else_, &otherwise) {
            (Some(e), _) => e,
            (None, Some(o)) => self.arm_body(o, ty.as_ref()),
            (None, None) => mk(TK::Unit, Ty::Unit),
        };
        let ty = match ty {
            _ if !valued => Ty::Unit,
            Some(t) => t,
            None => self.join_ty(&else_.ty.clone()),
        };
        let converts = valued && self.shallow(&ty) != Ty::Unit;
        let else_leaves = otherwise
            .as_ref()
            .is_some_and(|o| o.leaves(&|n| self.traps(n)));
        let mut tail = match converts && !else_leaves && !checked {
            true => self.coerce(else_, &ty),
            false => else_,
        };
        for (c, b, leaves) in arms.into_iter().flatten().rev() {
            // A branch that leaves gives no value to convert.
            let b = match converts && !leaves {
                true => self.coerce(b, &ty),
                false => b,
            };
            tail = mk(TK::If(Box::new(c), Box::new(b), Box::new(tail)), ty.clone());
        }
        tail
    }
    fn if_arm(
        &mut self,
        (c, b): &(next::Condition, next::ArmBody),
        want: Option<&Ty>,
    ) -> (TExpr, TExpr, bool) {
        let parts: Vec<next::CondPart> = c.parts().collect();
        let narrows = matches!(&parts[..], [next::CondPart::Bound(_)]);
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
        let b2 = match narrows && self.is_error(&c.ty) {
            true => Self::error_node(c.span),
            false => self.arm_body(b, want),
        };
        if narrows {
            self.solve(false);
        }
        self.b.assumed.truncate(n);
        self.b.scopes.pop();
        (c, b2, b.leaves(&|n| self.traps(n)))
    }
    fn if_expr(&mut self, i: &next::IfExpr, want: Option<&Ty>, span: Span) -> TExpr {
        if i.is_const() {
            self.err(
                span,
                Unbuilt::Statement {
                    kind: "`if const`".to_string(),
                },
            );
            return Self::error_node(span);
        }
        let mut branches: Vec<(next::Condition, next::ArmBody)> = Vec::new();
        if let (Some(c), Some(b)) = (i.cond(), i.body()) {
            branches.push((c, b));
        }
        for e in i.elifs() {
            if let (Some(c), Some(b)) = (e.cond(), e.body()) {
                branches.push((c, b));
            }
        }
        let otherwise = i.else_clause().and_then(|e| e.body());
        self.if_chain(branches, otherwise, want, span)
    }
    /// `if` written as a statement: its blocks give a value only against `want`, as the last
    /// line of a block with an `else`.
    pub(super) fn if_stmt(&mut self, i: &next::IfStmt, want: Option<&Ty>) -> TExpr {
        let span = self.at(i);
        if i.is_const() {
            self.err(
                span,
                Unbuilt::Statement {
                    kind: "`if const`".to_string(),
                },
            );
            return Self::error_node(span);
        }
        let mut branches: Vec<(next::Condition, next::ArmBody)> = Vec::new();
        if let (Some(c), Some(b)) = (i.cond(), i.body()) {
            branches.push((c, next::ArmBody::Block(b)));
        }
        for e in i.elifs() {
            if let (Some(c), Some(b)) = (e.cond(), e.body()) {
                branches.push((c, b));
            }
        }
        let otherwise = i.else_clause().and_then(|e| e.body());
        self.if_chain(branches, otherwise, want, span)
    }
    pub(super) fn self_field(&self, n: &str) -> Option<usize> {
        let s = self.self_expr(Span::new(0, 0))?;
        match self
            .syms
            .get(n)
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
            && self.named(self.b.module, n).is_some()
    }
}

/// Each type parameter's name and the type `sub` gives it.
pub(super) fn named_params(generics: &[Sym], sub: &Subst) -> Vec<(Sym, Ty)> {
    (generics.iter().enumerate())
        .map(|(i, g)| (*g, sub.ty(&Ty::Param(i))))
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

/// A branch whose value is a bare `None`, as `is_none` resolves a name.
fn gives_none(b: &next::ArmBody, is_none: &dyn Fn(&next::NameRef) -> bool) -> bool {
    let none = |e: &next::Expr| matches!(super::index::unparen(e.clone()), next::Expr::Name(n) if is_none(&n));
    let stmt =
        |s: &next::Stmt| matches!(s, next::Stmt::Expr(x) if x.expr().is_some_and(|e| none(&e)));
    match b {
        next::ArmBody::Expr(e) => none(e),
        next::ArmBody::Stmt(s) => stmt(s),
        next::ArmBody::Block(b) => {
            matches!(b.entries().last(), Some(next::BlockEntry::Stmt(s)) if stmt(&s))
        }
    }
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

impl Cx<'_> {
    /// What module `m` names the source name `n`: the `Names` view, by its interned symbol.
    pub(super) fn named(&self, m: ModId, n: &str) -> Option<Named> {
        self.names.named(m, self.syms.get(n)?)
    }
    pub(super) fn fn_named(&self, m: ModId, n: &str) -> Option<FnId> {
        self.names.fn_named(m, self.syms.get(n)?)
    }
    pub(super) fn struct_named(&self, m: ModId, n: &str) -> Option<StructId> {
        self.names.struct_named(m, self.syms.get(n)?)
    }
    pub(super) fn is_ambiguous(&self, m: ModId, n: &str) -> bool {
        (self.syms.get(n)).is_some_and(|s| self.names.is_ambiguous(m, s))
    }
    pub(super) fn variant_named(&self, e: StructId, n: &str) -> Option<usize> {
        self.shapes.variant(e, self.syms.get(n)?)
    }

    pub(super) fn field_index(&self, s: StructId, n: &str) -> Option<usize> {
        let n = self.syms.get(n)?;
        (self.items.s(s).fields.iter()).position(|(f, _)| *f == n)
    }

    pub(super) fn field_names(&self, s: StructId) -> Vec<String> {
        let fields = self.shapes.field_names(s).into_iter();
        fields.map(|f| self.shapes.text(f).to_string()).collect()
    }

    pub(super) fn method_in(&self, members: &HashMap<Sym, FnId>, n: &str) -> Option<FnId> {
        members.get(&self.syms.get(n)?).copied()
    }
    /// `42`, `1e9`, `2.5`, `true`, `'a'`, or a string and its holes.
    pub(super) fn literal(&mut self, l: &next::Literal, _want: Option<&Ty>, span: Span) -> TExpr {
        let mk = |kind, ty| TExpr { kind, ty, span };
        if let Some(pieces) = l.pieces() {
            return self.string(pieces, span);
        }
        let Some(tok) = l.token() else {
            return Self::error_node(span);
        };
        let text = tok.text();
        match tok.kind() {
            TokenKind::Int => match literal::int_value(text) {
                Some(i) => mk(TK::Int(i as i64), Ty::Int),
                None => Self::error_node(span),
            },
            // A literal with an exponent whose value is a whole number is an `Int` (R01.4.2).
            TokenKind::Float => match (text.contains(['e', 'E']))
                .then(|| literal::int_value(text))
                .flatten()
                .and_then(|i| i64::try_from(i).ok())
            {
                Some(i) => mk(TK::Int(i), Ty::Int),
                None => match text.parse::<f64>() {
                    Ok(x) => mk(TK::Float(x.to_bits()), Ty::F64),
                    Err(_) => {
                        let text = text.to_string();
                        self.err(span, Unbuilt::FloatLiteral { text });
                        mk(TK::Error, Ty::Error)
                    }
                },
            },
            TokenKind::True => mk(TK::Bool(true), Ty::Bool),
            TokenKind::False => mk(TK::Bool(false), Ty::Bool),
            TokenKind::Char => match literal::char_value(text) {
                Some(c) if c.is_ascii() => {
                    let code = mk(TK::Int(i64::from(c as u8)), Ty::Int);
                    mk(TK::CharOf(Box::new(code)), Ty::Char)
                }
                Some(_) => {
                    let kind = "Char".to_string();
                    self.err(span, Unbuilt::Expression { kind });
                    Self::error_node(span)
                }
                None => Self::error_node(span),
            },
            _ => Self::error_node(span),
        }
    }

    /// A string: its text alone, or each piece appended in turn through the
    /// `@role(interpolation)` fn (R10.3.2).
    fn string(&mut self, pieces: Vec<next::StrPiece>, span: Span) -> TExpr {
        let mk = |kind, ty| TExpr { kind, ty, span };
        if pieces.iter().all(|p| matches!(p, next::StrPiece::Text(_))) {
            let text = (pieces.into_iter())
                .map(|p| match p {
                    next::StrPiece::Text(t) => t,
                    next::StrPiece::Interp(_) => String::new(),
                })
                .collect();
            return self.text_value(text, span);
        }
        let Some(put) = self.rows.role_fn(Role::Interpolation) else {
            let kind = "string with holes and no `@role(interpolation)` fn".to_string();
            self.err(span, Unbuilt::Expression { kind });
            return Self::error_node(span);
        };
        let empty = self.text_value(String::new(), span);
        let ty = empty.ty.clone();
        let buf = self.temp("$str", ty.clone());
        match buf {
            VarRef::Local(l) => self.b.locals[l].mutable = true,
            VarRef::Global(g) => self.globals[g].mutable = true,
        }
        let read = mk(TK::Var(buf), ty.clone());
        let mut stmts = vec![TStmt::Let(buf, empty)];
        for p in pieces {
            let (v, width, zero, digits) = match p {
                next::StrPiece::Text(t) => (self.text_value(t, span), 0, false, None),
                next::StrPiece::Interp(h) => {
                    let spec = h.format();
                    let digits = match spec.map(|s| s.kind) {
                        None | Some(FmtKind::Dec) => None,
                        Some(FmtKind::Fixed(d)) => Some(d),
                        Some(FmtKind::Hex | FmtKind::Bin) => {
                            self.err(span, Unbuilt::HexFormat);
                            continue;
                        }
                    };
                    let v = match h.value() {
                        Some(x) => self.expr(&x, None),
                        None => Self::error_node(span),
                    };
                    let (width, zero) = spec.map_or((0, false), |s| (s.width, s.zero));
                    (v, width, zero, digits)
                }
            };
            let f = self.fnref(put, Vec::new(), Vec::new());
            let formals = self.formals(
                &Subst {
                    tys: &f.targs,
                    sets: &f.sets,
                },
                self.items.f(put),
            );
            self.unify(&formals[0].ty, &ty);
            self.unify(&formals[1].ty, &v.ty);
            self.oblige(&f, span);
            // The layout a spec gives: its width, its zero fill and its decimals.
            let mut args = vec![TArg::Mut(read.clone()), TArg::Value(v)];
            for k in 0..3 {
                let Some(want) = formals.get(2 + k).map(|p| p.ty.clone()) else {
                    break;
                };
                let x = match (k, digits) {
                    (0, _) => mk(TK::Int(i64::from(width)), Ty::Int),
                    (1, _) => mk(TK::Bool(zero), Ty::Bool),
                    (_, Some(d)) => mk(TK::Int(i64::from(d)), Ty::Int),
                    (_, None) => self.none_value(Some(&want), span),
                };
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

    /// A string with no holes, through `@sugar(str_literal)`.
    pub(super) fn text_value(&mut self, text: String, span: Span) -> TExpr {
        let mk = |kind, ty| TExpr { kind, ty, span };
        if !text.is_ascii() {
            self.err(span, Unbuilt::NonAsciiString);
        }
        let n = mk(TK::Int(text.len() as i64), Ty::Int);
        let raw = mk(TK::Text(text), Ty::Raw(Box::new(Ty::Char)));
        self.marked_call(
            self.rows.role_fn(Role::StrLiteral),
            "str_literal",
            vec![raw, n],
            span,
        )
    }

    /// `None`, an optional of the type its place wants.
    pub(super) fn none_value(&mut self, want: Option<&Ty>, span: Span) -> TExpr {
        let mk = |kind, ty| TExpr { kind, ty, span };
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

    /// `x!`: the value, checked present.
    pub(super) fn unwrap_expr(
        &mut self,
        u: &next::UnwrapExpr,
        _want: Option<&Ty>,
        span: Span,
    ) -> TExpr {
        let mk = |kind, ty| TExpr { kind, ty, span };
        let Some(x) = u.inner() else {
            return Self::error_node(span);
        };
        let v = self.expr(&x, None);
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

    /// `-x`, `not x` and `~x`.
    pub(super) fn prefix(&mut self, p: &next::PrefixExpr, _want: Option<&Ty>, span: Span) -> TExpr {
        let mk = |kind, ty| TExpr { kind, ty, span };
        let (Some(op), Some(x)) = (p.op_kind(), p.operand()) else {
            return Self::error_node(span);
        };
        // `~x` flips every bit: `x ^ ones`.
        if op == UnOp::BitNot {
            let v = self.expr(&x, None);
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
            return mk(TK::Bin(BinOp::BitXor, Box::new(v), Box::new(ones)), t);
        }
        let (want_t, kind): (Ty, fn(Box<TExpr>) -> TK) = match op {
            UnOp::Not => (Ty::Bool, TK::Not),
            UnOp::Neg => (Ty::Int, TK::Neg),
            _ => {
                self.err(span, Unbuilt::Operator);
                (Ty::Int, TK::Neg)
            }
        };
        let v = self.expr(&x, Some(&want_t));
        if let Some(found) = (op == UnOp::Neg)
            .then(|| self.operator_trait(&op.to_string(), false, &v.ty))
            .flatten()
        {
            return self.operator_call(found, v, None, span);
        }
        let want_t = match self.shallow(&v.ty) {
            Ty::F64 if op == UnOp::Neg => Ty::F64,
            Ty::I128 if op == UnOp::Neg => Ty::I128,
            t if op == UnOp::Neg && self.declares_operators(&t) => {
                let t = self.show(&t);
                self.err(span, Unbuilt::OperatorTrait { ty: t });
                return mk(TK::Error, Ty::Error);
            }
            _ => want_t,
        };
        let v = self.coerce(v, &want_t);
        mk(kind(Box::new(v)), want_t)
    }

    /// `a + b` and the other binary operators; `a ?? b` takes `b` when `a` is `None`.
    pub(super) fn bin_expr(&mut self, b: &next::BinExpr, _want: Option<&Ty>, span: Span) -> TExpr {
        let (Some(op), Some(lhs), Some(rhs)) = (b.op_kind(), b.lhs(), b.rhs()) else {
            return Self::error_node(span);
        };
        match op {
            BinOp::Coalesce => self.coalesce(&lhs, &rhs, span),
            // The parser reports `<=>`: it is `cmp(a, b)` (decision 4.5).
            BinOp::Spaceship => Self::error_node(span),
            _ => {
                let l = self.expr(&lhs, None);
                let r = self.expr(&rhs, None);
                self.binary(op, l, r, span)
            }
        }
    }

    /// `a + b ?? 0` reads two ways, so it has to say which (R01.5.6), unless `a` is a position,
    /// whose step (`i + 1 ?? I.last`) reads one way, or the right side is a way out
    /// (`cur + 1 ?? continue`). The parser judges an operator on the right.
    fn coalesce_mixed(&mut self, lhs: &next::Expr, rhs: &next::Expr, l: &TExpr) {
        let next::Expr::Bin(b) = lhs else { return };
        let arithmetic = matches!(
            b.op_kind(),
            Some(BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem)
        );
        let leaves = matches!(
            rhs,
            next::Expr::Return(_) | next::Expr::Break(_) | next::Expr::Continue(_)
        );
        // A position's step reads its integer (`position_arith`).
        let step = matches!(&l.kind, TK::Bin(_, a, _) if matches!(a.kind, TK::DomOrd(_)));
        if !arithmetic || leaves || step {
            return;
        }
        let text = cpl_diag::msgs::syntax::COALESCE_MIXED.say(&[]);
        let msg = Msg::verdict(cpl_diag::codes::UNPARENTHESIZED_COALESCE, text);
        let help = "write `(a ?? 0) + b` for a default on `a` alone, or `(a + b) ?? 0` for a default on the sum";
        let at = self.at(lhs);
        self.err_help(at, msg, Some(help.to_string()));
    }

    /// `some` and `none`, the optional spellings before `Some` and `None` (decision 157): E0503
    /// with the fix `cpl migrate` applies. False for any other name.
    pub(super) fn old_optional_spelling(&mut self, n: &str, span: Span) -> bool {
        let new = match n {
            "some" => "Some",
            "none" => "None",
            _ => return false,
        };
        let text = cpl_diag::msgs::check::REMOVED_SYNTAX_IS_SPELLED_SINCE_0.say(&[&n, &new]);
        let mut msg = Msg::verdict(cpl_diag::codes::REMOVED_SYNTAX, text);
        msg.help = Some(cpl_diag::habits::OLD_OPTIONAL_SPELLING.say(&[&n]));
        msg.fixes.push(cpl_next_ir::msg::Fix {
            title: format!("write `{new}`"),
            edits: vec![(self.b.item.at(span), new.to_string())],
        });
        self.err(span, msg);
        true
    }
    fn coalesce(&mut self, lhs: &next::Expr, rhs: &next::Expr, span: Span) -> TExpr {
        let mk = |kind, ty| TExpr { kind, ty, span };
        let l = match lhs.bare_try() {
            Some(_) => self.extent(lhs, None),
            None => self.expr(lhs, None),
        };
        self.coalesce_mixed(lhs, rhs, &l);
        let Some(inner) = self.optional_of(&l.ty) else {
            if !self.mentions_error(&l.ty) {
                let t = self.show(&l.ty);
                self.err(span, Unbuilt::Coalesce { ty: t });
            }
            return mk(TK::Error, Ty::Error);
        };
        // A read whose result is a new value: both sides flow into fresh sets, so a lambda on
        // the right is no member of the left's (`self.g ?? (_ => true)` leaves `g`'s set alone).
        let joined = self.join_ty(&inner);
        // An optional right side keeps the result optional: `a ?? b ?? 9` (R01.8.6).
        let r = self.expr(rhs, Some(&joined));
        let opt = l.ty.clone();
        let was_joining = std::mem::replace(&mut self.b.joining, true);
        let (r, out) = match self.optional_of(&r.ty) {
            Some(_) => (self.coerce(r, &opt), opt.clone()),
            None => (self.coerce(r, &joined), joined),
        };
        self.b.scopes.push(HashMap::new());
        let (bind, flag, val) = self.opened(l, inner);
        self.b.scopes.pop();
        let val = match out == opt {
            true => self.some(val, opt),
            false => self.coerce(val, &out),
        };
        self.b.joining = was_joining;
        let pick = mk(
            TK::If(Box::new(flag), Box::new(val), Box::new(r)),
            out.clone(),
        );
        mk(TK::Block(bind, Some(Box::new(pick))), out)
    }

    /// `a < b`, a chain `a <= b < c`, and `x in xs`.
    pub(super) fn compare(
        &mut self,
        c: &next::CompareExpr,
        _want: Option<&Ty>,
        span: Span,
    ) -> TExpr {
        let mk = |kind, ty| TExpr { kind, ty, span };
        let mut operands = c.operands();
        let Some(first) = operands.next() else {
            return Self::error_node(span);
        };
        let rest: Vec<(CmpOp, next::Expr)> = c.ops().into_iter().zip(operands).collect();
        let up = |o: &CmpOp| matches!(o, CmpOp::Lt | CmpOp::Le);
        let down = |o: &CmpOp| matches!(o, CmpOp::Gt | CmpOp::Ge);
        if rest.iter().any(|(o, _)| up(o)) && rest.iter().any(|(o, _)| down(o)) {
            self.err(
                span,
                Msg::verdict(
                    cpl_diag::codes::MIXED_COMPARISON_CHAIN,
                    "a chain that goes both up and down; write the two comparisons with `and`",
                ),
            );
        }
        // `None == x`: the `None` takes its type from the other side.
        if let [(op @ (CmpOp::Eq | CmpOp::Ne), rhs)] = rest.as_slice() {
            if matches!(super::index::unparen(first.clone()), next::Expr::Name(n) if self.names_none(&n))
            {
                let r = self.expr(rhs, None);
                let l = self.none_value(Some(&r.ty), span);
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
        let mut prev = self.expr(&first, None);
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
            let lift = self.optional_of(&prev.ty).is_none() && self.optional_of(&r.ty).is_some();
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

    /// `()`, or `(a, b)` as its tuple struct's constructor.
    pub(super) fn tuple(&mut self, t: &next::TupleExpr, want: Option<&Ty>, span: Span) -> TExpr {
        let xs: Vec<next::Expr> = t.elems().collect();
        if xs.is_empty() {
            return TExpr {
                kind: TK::Unit,
                ty: Ty::Unit,
                span,
            };
        }
        let Some(d) = self.shapes.tuple(xs.len()) else {
            let kind = "Tuple".to_string();
            self.err(span, Unbuilt::Expression { kind });
            return Self::error_node(span);
        };
        // `(R, C)` named as a value is the product domain's whole range (R04.2.11).
        if let Some(p) = self.type_path(&next::Expr::Tuple(t.clone())) {
            return self.domain_value(p, span);
        }
        let args: Vec<call::CallArg> = xs.iter().map(call::CallArg::plain).collect();
        let targs = (0..xs.len()).map(|_| self.new_ty()).collect();
        let sets = (0..self.items.s(d).n_sets)
            .map(|_| self.new_set())
            .collect();
        self.construct(d, targs, sets, &args, span, want)
    }

    /// `[a, b, c]`.
    pub(super) fn list(&mut self, l: &next::ListExpr, want: Option<&Ty>, span: Span) -> TExpr {
        let xs: Vec<next::Expr> = l.elems().collect();
        self.list_literal(&xs, want, span)
    }

    /// `lo..hi`, `lo..=hi by s`, `..` in its place, and `w by s` on any walk.
    pub(super) fn range(&mut self, r: &next::RangeExpr, want: Option<&Ty>, span: Span) -> TExpr {
        if let Some(walk) = r.walk() {
            let Some(step) = r.step() else {
                return Self::error_node(span);
            };
            if let next::Expr::Range(w) = &walk {
                if let (Some(lo), Some(hi), None) = (w.lo(), w.hi(), w.step()) {
                    let inclusive = w.is_inclusive();
                    return self.range_value(
                        Some(&lo),
                        Some(&hi),
                        inclusive,
                        Some(&step),
                        want,
                        span,
                    );
                }
            }
            return self.step_walk(&walk, &step, span);
        }
        let (lo, hi, step) = (r.lo(), r.hi(), r.step());
        if lo.is_some() || hi.is_some() {
            let inclusive = r.is_inclusive();
            return self.range_value(
                lo.as_ref(),
                hi.as_ref(),
                inclusive,
                step.as_ref(),
                want,
                span,
            );
        }
        match self.whole_of(want) {
            Some(d) if step.is_none() => self.domain_value(d, span),
            // Its domain comes from its place: an index or a `Range<D>` slot (R04.8.6).
            None if step.is_none() => {
                let text = cpl_diag::msgs::check::TYPE_MISMATCH_FULL_RANGE.say(&[]);
                self.err(span, Msg::verdict(cpl_diag::codes::TYPE_MISMATCH, text));
                Self::error_node(span)
            }
            _ => {
                let kind = "range with a step and no domain".to_string();
                self.err(span, Unbuilt::Expression { kind });
                Self::error_node(span)
            }
        }
    }
}

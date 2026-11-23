//! Calls to the user's fns and generic fns, and struct constructors.

use super::*;

impl<'a> Lowerer<'a> {
    /// `Some(e)`: `e` takes the wanted `?T`'s `T`, so `Some(3)` is a `?Mod<7>` where one is wanted.
    pub(in crate::lower) fn some_of(
        &mut self,
        arg: &ast::Expr,
        hint: Option<&Ty>,
        span: Span,
    ) -> Expr {
        let inner_h = match hint {
            Some(Ty::Opt(inner)) => Some(&**inner),
            other => other,
        };
        let mut val = self.expr(arg, inner_h);
        if let Some(want) = inner_h.filter(|w| **w != val.ty) {
            val = self.try_coerce(val, want).unwrap_or_else(|v| v);
        }
        let ty = Ty::opt(val.ty.clone());
        self.mk(ExprKind::Some(Box::new(val)), ty, span)
    }

    /// The symbol `e` names, if it is a bare name no local hides.
    pub(in crate::lower) fn sym(&self, e: &ast::Expr) -> Option<Sym> {
        match &e.kind {
            ast::ExprKind::Name(name) if self.lookup(name).is_none() => {
                if let Some(ga) = self.gen_env.get(name) {
                    return match ga {
                        GenArg::Domain(d) => Some(Sym::Domain(*d)),
                        GenArg::Ty(Ty::Struct(id)) => Some(Sym::Struct(*id)),
                        GenArg::Ty(Ty::Enum(id)) => Some(Sym::Enum(*id)),
                        _ => None,
                    };
                }
                self.get_sym(name)
            }
            _ => None,
        }
    }

    /// `op(a, b)` for an operation value `op` (`min`, `+`): the operation
    /// itself, on `a` and `b`.
    pub(in crate::lower) fn bare_op_call(
        &mut self,
        op: BareOp,
        op_span: Span,
        args: &[ast::Arg],
        span: Span,
        hint: Option<&Ty>,
    ) -> Option<Expr> {
        if args.len() != 2 || args.iter().any(|a| a.name.is_some() || a.mutable) {
            return None;
        }
        let (lhs, rhs) = (args[0].value.clone(), args[1].value.clone());
        let bin = |op| ast::ExprKind::Binary {
            op,
            op_span,
            lhs: Box::new(lhs.clone()),
            rhs: Box::new(rhs.clone()),
        };
        let named = |f: &str| ast::ExprKind::Call {
            callee: Box::new(ast::Expr {
                kind: ast::ExprKind::Name(f.into()),
                span: op_span,
            }),
            args: args.to_vec(),
        };
        let kind = match op {
            BareOp::Add => bin(ast::BinOp::Add),
            BareOp::Mul => bin(ast::BinOp::Mul),
            BareOp::BitXor => bin(ast::BinOp::BitXor),
            BareOp::BitOr => bin(ast::BinOp::BitOr),
            BareOp::BitAnd => bin(ast::BinOp::BitAnd),
            BareOp::Max => named("max"),
            BareOp::Min => named("min"),
            BareOp::Gcd => named("gcd"),
            BareOp::Lcm => named("lcm"),
        };
        let e = ast::Expr { kind, span };
        Some(self.expr(&e, hint))
    }

    pub(in crate::lower) fn call_fn_val(
        &mut self,
        callee: Expr,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let Ty::Fn(param_tys, ret_ty) = callee.ty.clone() else {
            if callee.ty != Ty::Error {
                self.error(
                    codes::NOT_CALLABLE,
                    span,
                    format!("a `{}` can't be called", self.show(&callee.ty)),
                );
            }
            return self.err_expr(span);
        };
        let mut vals = Vec::with_capacity(args.len());
        for (i, arg) in args.iter().enumerate() {
            let v = match (param_tys.get(i), arg.mutable) {
                // The fn changes the caller's variable, as a top-level
                // fn with a `mut` parameter does.
                (Some(Ty::Mut(want)), true) => {
                    let place = self.place(&arg.value);
                    if let PlaceKind::Local(slot) = place.kind {
                        self.cx.mut_slots.insert(slot);
                    }
                    if place.ty == Ty::Bool && matches!(place.kind, PlaceKind::Index(..)) {
                        self.unsupported(arg.value.span, "a `Bool` element passed as `mut`");
                    } else if place.ty != **want && place.ty != Ty::Error && **want != Ty::Error {
                        self.mismatch(arg.value.span, want, &place.ty);
                    }
                    Arg::Mut(place)
                }
                (Some(Ty::Mut(_)), false) => {
                    let message =
                        "this parameter is `mut`: the fn changes what it is given".to_string();
                    self.error(codes::ASSIGN_TO_LET, arg.value.span, message);
                    self.help("write `mut` before it, as for a top-level fn");
                    Arg::Value(self.err_expr(arg.value.span))
                }
                (Some(_), true) => {
                    let message = "this parameter is not `mut`: the fn takes a copy".to_string();
                    self.error(codes::ASSIGN_TO_LET, arg.value.span, message);
                    self.help("drop the `mut`, or declare the parameter `mut`");
                    Arg::Value(self.err_expr(arg.value.span))
                }
                (Some(want), false) => Arg::Value(self.expect(&arg.value, want)),
                (None, _) => Arg::Value(self.expr(&arg.value, None)),
            };
            vals.push(v);
        }
        self.mk(ExprKind::CallExpr(Box::new(callee), vals), *ret_ty, span)
    }

    pub(in crate::lower) fn call(
        &mut self,
        whole: &ast::Expr,
        callee: &ast::Expr,
        args: &[ast::Arg],
        hint: Option<&Ty>,
    ) -> Expr {
        let span = whole.span;
        match &callee.kind {
            ast::ExprKind::Paren(inner) => self.call(whole, inner, args, hint),
            ast::ExprKind::Name(name) => {
                // `yes(ok)` under `with io(answers: ..)`. (R10.3.3)
                if name == "yes"
                    && self.lookup(name).is_none()
                    && !args
                        .iter()
                        .any(|a| a.name.as_ref().is_some_and(|n| n.text == "answers"))
                {
                    if let Some(value) = self.context("io", "answers") {
                        let mut args = args.to_vec();
                        args.push(ast::Arg {
                            name: Some(ast::Name {
                                text: "answers".into(),
                                span: value.span,
                            }),
                            mutable: false,
                            value,
                        });
                        return self.call(whole, callee, &args, hint);
                    }
                }
                // Local generic function (`fn id<T>(x: T) -> T = x`)?
                if let Some(decl) = self.cx.local_gen_fns.get(name).cloned() {
                    return self.call_local_gen_fn(&decl, args, span);
                }
                if let Some(&op) = self.cx.op_locals.get(name) {
                    if let Some(e) = self.bare_op_call(op, callee.span, args, span, hint) {
                        return e;
                    }
                }
                // Local variable, receiver field, or captured fn value?
                if self.lookup(name).is_some() {
                    let fv = self.name(name, None, callee.span);
                    return self.call_fn_val(fv, args, span);
                }
                if let Some((slot, owner)) = self.cx.receiver {
                    if let Some((index, ty)) = self.field_of(&Ty::Struct(owner), name) {
                        if matches!(ty, Ty::Fn(..)) {
                            let base = self.local(slot, callee.span);
                            let fv =
                                self.mk(ExprKind::Field(Box::new(base), index), ty, callee.span);
                            return self.call_fn_val(fv, args, span);
                        }
                    }
                }
                if let Some(slot) = self.capture_from_outer(name) {
                    let fv = self.local(slot, callee.span);
                    return self.call_fn_val(fv, args, span);
                }
                if let Some((slot, owner)) = self.capture_self_from_outer() {
                    if let Some((index, ty)) = self.field_of(&Ty::Struct(owner), name) {
                        if matches!(ty, Ty::Fn(..)) {
                            let base = self.local(slot, callee.span);
                            let fv =
                                self.mk(ExprKind::Field(Box::new(base), index), ty, callee.span);
                            return self.call_fn_val(fv, args, span);
                        }
                    }
                }
                if let Some(ga) = self.gen_env.get(name).cloned() {
                    match ga {
                        GenArg::Domain(id) => {
                            let Some(value) = self.one_arg(name, args, &Ty::Int, span) else {
                                return self.err_expr(span);
                            };
                            return self.mk(
                                ExprKind::Cast(Cast::FromLabel(id), Box::new(value)),
                                Ty::Pos(id),
                                span,
                            );
                        }
                        GenArg::Ty(Ty::Struct(id)) => {
                            return self.construct_or_new(id, args, span);
                        }
                        GenArg::Ty(own @ (Ty::Tuple(_) | Ty::Record(_))) => {
                            if let Some(e) = self.fields_ctor(own, args, span) {
                                return e;
                            }
                        }
                        GenArg::Ty(Ty::Int) => return self.prelude("Int", args, hint, span),
                        GenArg::Ty(Ty::Float) => return self.prelude("F64", args, hint, span),
                        GenArg::Ty(Ty::Char) => return self.prelude("Char", args, hint, span),
                        GenArg::Ty(Ty::Str) => return self.prelude("Str", args, hint, span),
                        GenArg::Ty(Ty::Bool) => return self.prelude("Bool", args, hint, span),
                        _ => {}
                    }
                }
                if (name == "Some" || name == "some")
                    && args.len() == 1
                    && args[0].name.is_none()
                    && !args[0].mutable
                {
                    return self.some_of(&args[0].value, hint, span);
                }
                if name == "same" && args.len() == 2 {
                    let op_a = self.op_identity(&args[0].value);
                    let op_b = self.op_identity(&args[1].value);
                    if op_a.is_some() || op_b.is_some() {
                        return self.mk(
                            ExprKind::Bool(op_a.is_some() && op_a == op_b),
                            Ty::Bool,
                            span,
                        );
                    }
                    if let (Some(ta), Some(tb)) = (
                        self.probe_ty_silent(&args[0].value),
                        self.probe_ty_silent(&args[1].value),
                    ) {
                        let is_marker = |t: &Ty| matches!(t, Ty::Struct(s) if self.structs[*s].def.fields.is_empty());
                        if is_marker(&ta) || is_marker(&tb) {
                            return self.mk(ExprKind::Bool(ta == tb), Ty::Bool, span);
                        }
                    }
                }
                // A plain call of two values is the core binary `min`/`max`, and a
                // free `count`/`all`/`any` of one generator tests each value it makes,
                // even where std.seq's fold of that name is in scope (decision 53).
                if super::names::core_over_std(name, args) {
                    return self.prelude(name, args, hint, span);
                }
                match self.get_sym(name) {
                    Some(Sym::Fn(id)) => self.user_call(id, None, args, span),
                    Some(Sym::GenFn(gfid))
                        if matches!(name.as_str(), "min" | "max")
                            && args.len() >= 2
                            && args.iter().all(|a| a.name.is_none())
                            && self.gen_fns[gfid]
                                .decl
                                .params
                                .first()
                                .is_some_and(|p| p.name.text == "self") =>
                    {
                        self.min_max(name == "max", args, span)
                    }
                    Some(Sym::GenFn(gfid)) => {
                        self.call_hint = hint.cloned();
                        self.call_gen_fn(gfid, Vec::new(), None, args, span)
                    }
                    Some(Sym::Struct(id)) => self.construct_or_new(id, args, span),
                    Some(Sym::GenStruct(_))
                        if prefix_shape_struct(name, 3).is_some()
                            && args.len() == 1
                            && args[0].name.is_none()
                            && matches!(
                                self.probe_ty_silent(&args[0].value),
                                Some(Ty::Array { ref dims, .. }) if dims.len() == 2
                            ) =>
                    {
                        self.grid_shape_ctor(prefix_shape_struct(name, 3).unwrap(), args, span)
                    }
                    Some(Sym::GenStruct(gid)) => {
                        self.call_gen_struct(gid, Vec::new(), args, hint, span)
                    }
                    Some(Sym::Variant(id, tag)) => self.variant(id, tag, args, span),
                    Some(Sym::Enum(eid))
                        if !self.enums[eid].variants.is_empty()
                            && self.enums[eid].variants.iter().all(|(_, fs)| fs.is_empty()) =>
                    {
                        let Some(exprs) = self.positional_any(name, args, 1, span) else {
                            return self.err_expr(span);
                        };
                        let v = self.expr(exprs[0], None);
                        let iv = self.coerce(v, &Ty::Int, exprs[0].span);
                        let loc = self.temp(Ty::Int);
                        let init = Stmt::Let(loc, Some(iv));
                        let discs = self
                            .enum_discriminants
                            .get(eid)
                            .cloned()
                            .unwrap_or_default();
                        let n = self.enums[eid].variants.len();
                        let mut acc =
                            self.mk(ExprKind::Variant(n - 1, Vec::new()), Ty::Enum(eid), span);
                        for tag in (0..n.saturating_sub(1)).rev() {
                            let disc = discs.get(tag).copied().unwrap_or(tag as i64);
                            let cond = self.mk(
                                ExprKind::Cmp(
                                    CmpOp::Eq,
                                    Box::new(self.mk(ExprKind::Local(loc), Ty::Int, span)),
                                    Box::new(self.int_lit(disc, span)),
                                ),
                                Ty::Bool,
                                span,
                            );
                            let then =
                                self.mk(ExprKind::Variant(tag, Vec::new()), Ty::Enum(eid), span);
                            acc = self.mk(
                                ExprKind::If(Box::new(cond), Box::new(then), Box::new(acc)),
                                Ty::Enum(eid),
                                span,
                            );
                        }
                        self.mk(
                            ExprKind::Do(Block {
                                stmts: vec![init, Stmt::Give(acc)],
                            }),
                            Ty::Enum(eid),
                            span,
                        )
                    }
                    Some(Sym::Domain(id)) => {
                        if let Some(factors) = self.product_domains.get(&id).cloned() {
                            if args.len() == factors.len() {
                                let mut parts = Vec::with_capacity(factors.len());
                                let named = args.iter().any(|a| a.name.is_some());
                                if named {
                                    let mut slots: Vec<Option<Expr>> = vec![None; factors.len()];
                                    for (i, arg) in args.iter().enumerate() {
                                        let idx = arg
                                            .name
                                            .as_ref()
                                            .and_then(|n| {
                                                factors.iter().position(|(fnm, _)| {
                                                    fnm.as_deref() == Some(n.text.as_str())
                                                })
                                            })
                                            .unwrap_or(i);
                                        let want = self.product_factor_ty(&factors[idx].1);
                                        slots[idx] = Some(self.expect(&arg.value, &want));
                                    }
                                    if slots.iter().all(|s| s.is_some()) {
                                        for s in slots {
                                            parts.push(s.unwrap());
                                        }
                                        return self.encode_product(id, parts, span);
                                    }
                                } else {
                                    for ((_, f), arg) in factors.iter().zip(args) {
                                        let want = self.product_factor_ty(f);
                                        parts.push(self.expect(&arg.value, &want));
                                    }
                                    return self.encode_product(id, parts, span);
                                }
                            }
                            if args.len() == 1 && args[0].name.is_none() {
                                let v = self.expr(&args[0].value, None);
                                if matches!(v.ty, Ty::Tuple(_) | Ty::Record(_)) {
                                    return self.product_from_tuple(id, v, span);
                                }
                                let value = if v.ty == Ty::Char {
                                    self.mk(
                                        ExprKind::Cast(Cast::CharToInt, Box::new(v)),
                                        Ty::Int,
                                        args[0].value.span,
                                    )
                                } else {
                                    self.coerce(v, &Ty::Int, args[0].value.span)
                                };
                                return self.mk(
                                    ExprKind::Cast(Cast::FromLabel(id), Box::new(value)),
                                    Ty::Pos(id),
                                    span,
                                );
                            }
                        }
                        let Some(exprs) = self.positional_any(name, args, 1, span) else {
                            return self.err_expr(span);
                        };
                        let v = self.expr(exprs[0], None);
                        let value = if v.ty == Ty::Char {
                            self.mk(
                                ExprKind::Cast(Cast::CharToInt, Box::new(v)),
                                Ty::Int,
                                exprs[0].span,
                            )
                        } else {
                            self.coerce(v, &Ty::Int, exprs[0].span)
                        };
                        self.mk(
                            ExprKind::Cast(Cast::FromLabel(id), Box::new(value)),
                            Ty::Pos(id),
                            span,
                        )
                    }
                    Some(Sym::Alias(aid)) => {
                        let (m, ty_ast, offset) = self.aliases[aid];
                        let sm = std::mem::replace(&mut self.cur_mod, m);
                        let so = std::mem::replace(&mut self.offset, offset);
                        let resolved = self.resolve_ty(ty_ast);
                        self.cur_mod = sm;
                        self.offset = so;
                        match resolved {
                            Ty::Struct(id) => self.construct_or_new(id, args, span),
                            Ty::Pos(id) => {
                                let Some(value) = self.one_arg(name, args, &Ty::Int, span) else {
                                    return self.err_expr(span);
                                };
                                self.mk(
                                    ExprKind::Cast(Cast::FromLabel(id), Box::new(value)),
                                    Ty::Pos(id),
                                    span,
                                )
                            }
                            _ => {
                                self.error(
                                    codes::NOT_CALLABLE,
                                    callee.span,
                                    format!("`{name}` is a type and can't be called"),
                                );
                                self.err_expr(span)
                            }
                        }
                    }
                    Some(Sym::Global(id)) => {
                        let fv = self.mk(
                            ExprKind::Global(id),
                            self.globals[id].ty.clone(),
                            callee.span,
                        );
                        self.call_fn_val(fv, args, span)
                    }
                    Some(Sym::ModConst(mcid)) => {
                        let fv = self.ensure_mod_const(mcid, callee.span);
                        self.call_fn_val(fv, args, span)
                    }
                    Some(Sym::GenAlias(gaid)) => {
                        let tpl_mod = self.gen_aliases[gaid].mod_id;
                        let tpl_offset = self.gen_aliases[gaid].offset;
                        let generics = self.gen_aliases[gaid].generics;
                        let target_ty = self.gen_aliases[gaid].ty;
                        let gen_names: Vec<String> =
                            generics.iter().map(|g| g.text.clone()).collect();
                        let mut env = HashMap::new();
                        if let Some(h) = hint {
                            self.unify_ast_ty(target_ty, h, tpl_mod, &gen_names, &mut env);
                        }
                        if env.len() == gen_names.len() {
                            let sm = std::mem::replace(&mut self.cur_mod, tpl_mod);
                            let so = std::mem::replace(&mut self.offset, tpl_offset);
                            let se = std::mem::replace(&mut self.gen_env, env);
                            let resolved = self.resolve_ty(target_ty);
                            self.cur_mod = sm;
                            self.offset = so;
                            self.gen_env = se;
                            if let Ty::Struct(sid) = resolved {
                                return self.construct_or_new(sid, args, span);
                            }
                        }
                        if let ast::TypeKind::Named { name: tname, .. } = &target_ty.kind {
                            if let Some(Sym::GenStruct(gsid)) =
                                self.mod_syms[tpl_mod].get(&tname.text).copied()
                            {
                                return self.call_gen_struct(gsid, Vec::new(), args, hint, span);
                            }
                        }
                        self.error(
                            codes::NOT_CALLABLE,
                            callee.span,
                            format!("`{name}` is a type and can't be called"),
                        );
                        self.err_expr(span)
                    }
                    Some(
                        Sym::Enum(_)
                        | Sym::GenEnum(_)
                        | Sym::Trait(_)
                        | Sym::Mod(_)
                        | Sym::GenConst(_),
                    ) => {
                        self.error(
                            codes::NOT_CALLABLE,
                            callee.span,
                            format!("`{name}` is a type and can't be called"),
                        );
                        self.err_expr(span)
                    }
                    None => self.prelude(name, args, hint, span),
                }
            }
            ast::ExprKind::Generic { base, args: types } => {
                self.call_generic(base, types, args, hint, span)
            }
            ast::ExprKind::Field { base, name } => {
                let call = self.method(whole, base, name, args, hint);
                self.char_fn_on_str(base, name);
                self.join_receiver(base, name, args, whole.span);
                call
            }
            _ => {
                let fv = self.expr(callee, None);
                self.call_fn_val(fv, args, span)
            }
        }
    }

    pub(in crate::lower) fn call_generic(
        &mut self,
        base: &ast::Expr,
        types: &[ast::Type],
        args: &[ast::Arg],
        hint: Option<&Ty>,
        span: Span,
    ) -> Expr {
        // Check `base.method::<T>(...)`.
        if let ast::ExprKind::Field { base: recv, name } = &base.kind {
            if let ("parse", [ty], []) = (name.text.as_str(), types, args) {
                if self.probe_ty_silent(recv) == Some(Ty::Str) {
                    return self.parse_str(recv, ty, span);
                }
            }
            if let Some(Sym::Mod(target_mod)) = self.sym(recv) {
                let saved_mod = std::mem::replace(&mut self.cur_mod, target_mod);
                let sym = self.mod_syms[target_mod].get(&name.text).copied();
                self.cur_mod = saved_mod;
                match sym {
                    Some(Sym::GenFn(gfid)) => {
                        let gargs = self.resolve_gen_args(types);
                        return self.call_gen_fn(gfid, gargs, None, args, span);
                    }
                    Some(Sym::GenStruct(gid)) => {
                        let gargs = self.resolve_gen_args_for_struct(gid, types);
                        return self.call_gen_struct(gid, gargs, args, hint, span);
                    }
                    _ => {}
                }
            }
            let val = self.expr(recv, None);
            if name.text == "parse"
                && types.len() == 1
                && args.is_empty()
                && matches!(val.ty, Ty::Str | Ty::StrView)
            {
                let spec = self.resolve(&types[0], true);
                let out_ty = spec.ty();
                if let Spec::Plain(Ty::Pos(d)) = spec {
                    if self.counted_domains.contains(&d) {
                        if let Some(base_shift) = self.io_base.filter(|&b| b != 0) {
                            let parsed = self.mk(
                                ExprKind::Parse(Box::new(val), Box::new(Spec::Plain(Ty::Int))),
                                Ty::Opt(Box::new(Ty::Int)),
                                span,
                            );
                            let opt_slot = self.temp(Ty::Opt(Box::new(Ty::Int)));
                            let res_ty = Ty::Opt(Box::new(Ty::Pos(d)));
                            let is_some = self.mk(
                                ExprKind::IsSome(Box::new(self.local(opt_slot, span))),
                                Ty::Bool,
                                span,
                            );
                            let raw = self.mk(
                                ExprKind::Unwrap(Box::new(self.local(opt_slot, span)), false),
                                Ty::Int,
                                span,
                            );
                            let ord = self.int_add(raw, -base_shift);
                            let ord_slot = self.temp(Ty::Int);
                            let zero = self.int_lit(0, span);
                            let len = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
                            let ge0 = self.mk(
                                ExprKind::Cmp(
                                    CmpOp::Ge,
                                    Box::new(self.local(ord_slot, span)),
                                    Box::new(zero),
                                ),
                                Ty::Bool,
                                span,
                            );
                            let lt_len = self.mk(
                                ExprKind::Cmp(
                                    CmpOp::Lt,
                                    Box::new(self.local(ord_slot, span)),
                                    Box::new(len),
                                ),
                                Ty::Bool,
                                span,
                            );
                            let in_range = self.mk(
                                ExprKind::Binary(BinOp::And, Box::new(ge0), Box::new(lt_len)),
                                Ty::Bool,
                                span,
                            );
                            let pos_val =
                                self.cast(Cast::Trust, self.local(ord_slot, span), Ty::Pos(d));
                            let some_pos =
                                self.mk(ExprKind::Some(Box::new(pos_val)), res_ty.clone(), span);
                            let none_pos = self.mk(ExprKind::None, res_ty.clone(), span);
                            let checked = self.mk(
                                ExprKind::Do(Block {
                                    stmts: vec![
                                        Stmt::Let(ord_slot, Some(ord)),
                                        Stmt::Give(self.mk(
                                            ExprKind::If(
                                                Box::new(in_range),
                                                Box::new(some_pos),
                                                Box::new(none_pos.clone()),
                                            ),
                                            res_ty.clone(),
                                            span,
                                        )),
                                    ],
                                }),
                                res_ty.clone(),
                                span,
                            );
                            return self.mk(
                                ExprKind::Do(Block {
                                    stmts: vec![
                                        Stmt::Let(opt_slot, Some(parsed)),
                                        Stmt::Give(self.mk(
                                            ExprKind::If(
                                                Box::new(is_some),
                                                Box::new(checked),
                                                Box::new(none_pos),
                                            ),
                                            res_ty.clone(),
                                            span,
                                        )),
                                    ],
                                }),
                                res_ty,
                                span,
                            );
                        }
                    }
                }
                return self.mk(
                    ExprKind::Parse(Box::new(val), Box::new(spec)),
                    Ty::Opt(Box::new(out_ty)),
                    span,
                );
            }
            if let Ty::Struct(sid) = val.ty {
                if let Some(&gfid) = self.structs[sid].gen_methods.get(&name.text) {
                    let gargs = self.resolve_gen_args(types);
                    let mut_self = self.gen_fns[gfid]
                        .decl
                        .params
                        .first()
                        .is_some_and(|p| p.name.text == "self" && p.mutable);
                    let recv_arg = if mut_self {
                        Arg::Mut(self.place(recv))
                    } else {
                        Arg::Value(val)
                    };
                    return self.call_gen_fn(gfid, gargs, Some(recv_arg), args, span);
                }
            }
        }

        let name = match &base.kind {
            ast::ExprKind::Name(name) => name.as_str(),
            _ => "",
        };
        // `read::<Int>()`, `read<1..=3>()`, `read<[n]1..=3>()`, `read<[n]E>(base: 1)`: a read that says its own type.
        if name == "read" && self.get_sym(name).is_none() {
            if let [ty] = types {
                let spec = self.resolve(ty, true);
                if args.is_empty() {
                    return self.read(spec, span);
                }
                return self.read_with_args(spec, args, span);
            }
        }
        if matches!(name, "List" | "Set" | "HashSet" | "Map" | "HashMap") {
            let ty = ast::Type {
                kind: ast::TypeKind::Named {
                    name: ast::Name {
                        text: name.to_string(),
                        span: base.span,
                    },
                    args: types.to_vec(),
                },
                span: base.span,
            };
            let ty = self.resolve_ty(&ty);
            if args.is_empty() {
                return self.mk(ExprKind::Tuple(Vec::new()), ty, span);
            }
            if args.len() == 1 && matches!(name, "List" | "Set" | "HashSet") {
                let seq = self.expr(&args[0].value, None);
                if seq.ty == ty {
                    return seq;
                }
                if let Ok(c) = self.try_coerce(seq.clone(), &ty) {
                    return c;
                }
                // Copy elements into a fresh List/Set.
                let is_set = matches!(ty, Ty::Set(_) | Ty::HashSet(_));
                return self.do_expr(Some(&ty), span, |lw, out| {
                    let acc = lw.temp(ty.clone());
                    out.push(Stmt::Let(
                        acc,
                        Some(lw.mk(ExprKind::Tuple(Vec::new()), ty.clone(), span)),
                    ));
                    let (iter, elem) = lw.iter(&args[0].value);
                    let lid = lw.open_loop(None);
                    let slot = lw.temp(elem);
                    let op = if is_set {
                        Mutation::Insert
                    } else {
                        Mutation::Push
                    };
                    let rty = if is_set { Ty::Bool } else { Ty::Unit };
                    let item = lw.local(slot, span);
                    let m = lw.mk(
                        ExprKind::Mutate(
                            op,
                            Box::new(Place {
                                kind: PlaceKind::Local(acc),
                                ty: ty.clone(),
                                span: lw.span(span),
                            }),
                            vec![item],
                        ),
                        rty,
                        span,
                    );
                    lw.cx.loops.pop();
                    out.push(Stmt::For(
                        lid,
                        slot,
                        iter,
                        Block {
                            stmts: vec![Stmt::Expr(m)],
                        },
                    ));
                    let res = lw.local(acc, span);
                    out.push(Stmt::Give(res));
                });
            }
        }
        if let (Some(Sym::GenFn(gfid)), [ty]) = (self.get_sym(name), types) {
            if name == "read_all" && self.io_mod == Some(self.gen_fns[gfid].mod_id) {
                return self.read_all(ty, args, span);
            }
        }
        let lookup = prefix_shape_struct(name, types.len()).unwrap_or(name);
        match self.get_sym(lookup) {
            Some(Sym::GenStruct(gid)) => {
                let gargs = self.resolve_gen_args_for_struct(gid, types);
                self.call_gen_struct(gid, gargs, args, hint, span)
            }
            Some(Sym::GenFn(gfid)) => {
                let gargs = self.resolve_gen_args(types);
                self.call_gen_fn(gfid, gargs, None, args, span)
            }
            Some(Sym::GenAlias(gaid)) => {
                let gargs = self.resolve_gen_args(types);
                let tpl_mod = self.gen_aliases[gaid].mod_id;
                let tpl_offset = self.gen_aliases[gaid].offset;
                let generics = self.gen_aliases[gaid].generics;
                let target_ty = self.gen_aliases[gaid].ty;
                let mut env = HashMap::new();
                for (p, a) in generics.iter().zip(gargs) {
                    env.insert(p.text.clone(), a);
                }
                let sm = std::mem::replace(&mut self.cur_mod, tpl_mod);
                let so = std::mem::replace(&mut self.offset, tpl_offset);
                let se = std::mem::replace(&mut self.gen_env, env);
                let resolved = self.resolve_ty(target_ty);
                self.cur_mod = sm;
                self.offset = so;
                self.gen_env = se;
                if let Ty::Struct(sid) = resolved {
                    return self.construct_or_new(sid, args, span);
                }
                self.err_expr(span)
            }
            None => {
                self.error(
                    codes::UNSUPPORTED,
                    span,
                    format!("`{name}` is not in scope during lowering"),
                );
                self.err_expr(span)
            }
            Some(sym) => {
                let kind = match sym {
                    Sym::Fn(_) => "non-generic function",
                    Sym::Struct(_) => "non-generic struct",
                    Sym::Enum(_) => "non-generic enum",
                    Sym::GenEnum(_) => "generic enum",
                    Sym::Variant(_, _) => "enum variant",
                    Sym::Domain(_) => "domain",
                    Sym::Alias(_) => "non-generic type alias",
                    Sym::Global(_) | Sym::GenConst(_) | Sym::ModConst(_) => "value",
                    Sym::Trait(_) => "trait",
                    Sym::Mod(_) => "module",
                    Sym::GenFn(_) | Sym::GenStruct(_) | Sym::GenAlias(_) => unreachable!(),
                };
                self.error(
                    codes::UNSUPPORTED,
                    span,
                    format!(
                        "`{name}` is a {kind} and cannot be called with explicit type arguments"
                    ),
                );
                self.err_expr(span)
            }
        }
    }

    /// Calls a struct `S(...)`: outside `S`'s own body, routes to `S.new(...)` if `new` exists;
    /// inside `S`'s own body, constructs `S` directly from its fields.
    pub(in crate::lower) fn construct_or_new(
        &mut self,
        id: StructId,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let inside_self = self.self_struct == Some(id)
            || self.self_struct.is_some_and(|sid| {
                matches!(
                    (&self.structs[sid].origin_gen, &self.structs[id].origin_gen),
                    (Some((a, _)), Some((b, _))) if a == b
                )
            });
        if !inside_self {
            if let Some(&new_fn) = self.structs[id].methods.get("new") {
                return self.user_call(new_fn, None, args, span);
            }
            if let Some(&new_gfn) = self.structs[id].gen_methods.get("new") {
                return self.call_gen_fn(new_gfn, Vec::new(), None, args, span);
            }
        }
        self.construct(id, args, span)
    }

    /// Instantiates a generic struct (inferring omitted type args from `self_struct`, `hint`, or call args)
    /// and constructs it via `construct_or_new`.
    pub(in crate::lower) fn call_gen_struct(
        &mut self,
        gid: GenStructId,
        explicit: Vec<GenArg>,
        args: &[ast::Arg],
        hint: Option<&Ty>,
        span: Span,
    ) -> Expr {
        let decl = self.gen_structs[gid].decl;
        let tpl_mod = self.gen_structs[gid].mod_id;
        let gen_names: Vec<String> = decl.generics.iter().map(|g| g.text.clone()).collect();
        let min_explicit = decl
            .generic_defaults
            .iter()
            .take_while(|d| d.is_none())
            .count();
        if !explicit.is_empty()
            && explicit.len() >= min_explicit
            && explicit.len() <= gen_names.len()
        {
            self.emit_anon_product_defs(&explicit, span);
            let sid = self.instantiate_struct(gid, explicit);
            return self.construct_or_new(sid, args, span);
        }
        // A hint says which instantiation is meant, even in the struct's own body
        // (`fn swap(self) -> Pair<B, A> = Pair(second, first)`).
        if let Some(Ty::Struct(hsid)) = hint {
            if let Some((hgid, _)) = &self.structs[*hsid].origin_gen {
                if *hgid == gid {
                    return self.construct_or_new(*hsid, args, span);
                }
            }
        }
        // Inside the struct's own body (`Mod(x % P)`, `Dsu(...)`, `Order(...)`), reuse `self_struct`!
        if explicit.is_empty() {
            if let Some(sid) = self.self_struct {
                if self.structs[sid]
                    .origin_gen
                    .as_ref()
                    .is_some_and(|(g, _)| *g == gid)
                {
                    return self.construct_or_new(sid, args, span);
                }
            }
        }
        // Otherwise infer generic params from `fn new(...)` (if present) or constructor fields (`Lit(Road(a), true)`).
        let mut inferred: GenEnv = HashMap::new();
        for (n, a) in gen_names.iter().zip(&explicit) {
            inferred.insert(n.clone(), a.clone());
        }
        let new_params: Option<&[ast::Param]> = decl.methods.iter().find_map(|s| match &s.kind {
            ast::StmtKind::Fn(f) if f.name.text == "new" => Some(f.params.as_slice()),
            _ => None,
        });
        if let Some(params) = new_params {
            let mut pos_idx = 0usize;
            for arg in args {
                let param = if let Some(aname) = &arg.name {
                    params.iter().find(|p| p.label_text() == aname.text)
                } else {
                    let p = params.get(pos_idx);
                    pos_idx += 1;
                    p
                };
                if let Some(p) = param {
                    if let Some(pat_ty) = &p.ty {
                        self.bind_bare_op(pat_ty, &arg.value, &gen_names, &mut inferred);
                        self.unify_lambda_params(
                            pat_ty,
                            &arg.value,
                            tpl_mod,
                            &gen_names,
                            &mut inferred,
                        );
                        self.bind_range_domain(
                            pat_ty,
                            &arg.value,
                            &gen_names,
                            &mut inferred,
                            span,
                            None,
                        );
                    }
                    if !matches!(arg.value.kind, ast::ExprKind::Lambda { .. })
                        && !matches!(&arg.value.kind, ast::ExprKind::Name(n) if matches!(n.as_str(), "min" | "max"))
                    {
                        if let Some(pty) = self.probe_ty_silent(&arg.value) {
                            if let Some(pat_ty) = &p.ty {
                                self.unify_ast_ty(pat_ty, &pty, tpl_mod, &gen_names, &mut inferred);
                            }
                        }
                    }
                }
            }
            for g in &gen_names {
                if !inferred.contains_key(g) {
                    let dom_from_arg = args.iter().find_map(|arg| {
                        if matches!(arg.value.kind, ast::ExprKind::Lambda { .. }) {
                            return None;
                        }
                        match self.probe_ty_silent(&arg.value)? {
                            Ty::Array { dims, elem } if matches!(*elem, Ty::List(_)) => {
                                match dims.as_slice() {
                                    [Dim::Pos(d)] => Some(*d),
                                    _ => None,
                                }
                            }
                            Ty::Struct(sid) => {
                                let info = &self.structs[sid];
                                info.env
                                    .get(g)
                                    .and_then(|a| match a {
                                        GenArg::Domain(d) | GenArg::Ty(Ty::Pos(d)) => Some(*d),
                                        _ => None,
                                    })
                                    .or_else(|| {
                                        info.origin_gen.as_ref().and_then(|(_, gargs)| {
                                            gargs.iter().find_map(|a| match a {
                                                GenArg::Domain(d) | GenArg::Ty(Ty::Pos(d)) => {
                                                    Some(*d)
                                                }
                                                _ => None,
                                            })
                                        })
                                    })
                            }
                            _ => None,
                        }
                    });
                    if let Some(d) = dom_from_arg {
                        inferred.insert(g.clone(), GenArg::Domain(d));
                    }
                }
            }
        } else {
            for (f, arg) in decl.fields.iter().zip(args) {
                self.bind_bare_op(&f.ty, &arg.value, &gen_names, &mut inferred);
                self.unify_lambda_params(&f.ty, &arg.value, tpl_mod, &gen_names, &mut inferred);
                self.bind_range_domain(&f.ty, &arg.value, &gen_names, &mut inferred, span, None);
                if !matches!(arg.value.kind, ast::ExprKind::Lambda { .. })
                    && !matches!(&arg.value.kind, ast::ExprKind::Name(n) if matches!(n.as_str(), "min" | "max"))
                {
                    if let Some(pty) = self.probe_ty_silent(&arg.value) {
                        self.unify_ast_ty(&f.ty, &pty, tpl_mod, &gen_names, &mut inferred);
                    }
                }
            }
        }
        if gen_names.iter().any(|g| g == "F") && !inferred.contains_key("F") {
            if let Some(act_arg) = args
                .iter()
                .find(|a| a.name.as_ref().is_some_and(|n| n.text == "act"))
            {
                if let Some(Ty::Struct(sid)) = self.probe_ty_silent(&act_arg.value) {
                    let sname = self.structs[sid].def.name.as_str();
                    let elem_ty = match inferred.get("T") {
                        Some(GenArg::Ty(t)) => t.clone(),
                        _ => Ty::Int,
                    };
                    let inferred_f = if let Some((orig_gid, gargs)) = &self.structs[sid].origin_gen
                    {
                        if self.gen_structs[*orig_gid].decl.name.text == "Action"
                            && gargs.len() == 2
                        {
                            match &gargs[1] {
                                GenArg::Ty(t) => Some(t.clone()),
                                _ => None,
                            }
                        } else {
                            None
                        }
                    } else {
                        let base = sname
                            .strip_prefix('m')
                            .and_then(|rest| rest.split_once('_'))
                            .map_or(sname, |(_, b)| b);
                        match base {
                            "AddAction" => Some(elem_ty),
                            "AssignAction" => Some(Ty::opt(elem_ty)),
                            "AffineAction" => Some(Ty::Tuple(vec![elem_ty.clone(), elem_ty])),
                            "FlipAction" => Some(Ty::Bool),
                            _ => None,
                        }
                    };
                    if let Some(f_ty) = inferred_f {
                        inferred.insert("F".to_string(), GenArg::Ty(f_ty));
                    }
                }
            }
        }
        let mut final_args = Vec::with_capacity(gen_names.len());
        for (i, g) in gen_names.iter().enumerate() {
            match inferred.get(g) {
                Some(a) => final_args.push(a.clone()),
                None => {
                    if let Some(Some(def_ty)) = decl.generic_defaults.get(i) {
                        let saved_mod = std::mem::replace(&mut self.cur_mod, tpl_mod);
                        let saved_env = std::mem::replace(&mut self.gen_env, inferred.clone());
                        let resolved = self.resolve_ty(def_ty);
                        self.gen_env = saved_env;
                        self.cur_mod = saved_mod;
                        final_args.push(GenArg::Ty(resolved));
                    } else {
                        final_args.push(GenArg::Ty(Ty::Int));
                    }
                }
            }
        }
        self.emit_anon_product_defs(&final_args, span);
        let sid = self.instantiate_struct(gid, final_args);
        self.construct_or_new(sid, args, span)
    }

    pub(in crate::lower) fn bind_bare_op(
        &self,
        pat: &ast::Type,
        arg: &ast::Expr,
        gen_names: &[String],
        inferred: &mut GenEnv,
    ) {
        if let ast::TypeKind::Named { name, args } = &pat.kind {
            if args.is_empty()
                && gen_names.contains(&name.text)
                && !inferred.contains_key(&name.text)
            {
                if let Some(op) = self.detect_bare_op(arg) {
                    inferred.insert(name.text.clone(), GenArg::Op(op));
                }
            }
        }
    }

    pub(in crate::lower) fn emit_anon_product_defs(&mut self, gargs: &[GenArg], span: Span) {
        let mut defs = Vec::new();
        for ga in gargs {
            if let GenArg::Domain(d) = ga {
                if self.anon_product_domain_set.contains(d) {
                    self.def_anon_product_domain(*d, span, &mut defs);
                }
            }
        }
        self.cx.pending.extend(defs);
    }

    /// Positional arguments only, and exactly `n` of them.
    pub(in crate::lower) fn positional_any<'b>(
        &mut self,
        what: &str,
        args: &'b [ast::Arg],
        n: usize,
        span: Span,
    ) -> Option<Vec<&'b ast::Expr>> {
        if let Some(named) = args.iter().find_map(|a| a.name.as_ref()) {
            let message = format!("`{what}` has no argument named `{}`", named.text);
            self.error(codes::UNKNOWN_ARG_NAME, named.span, message);
            return None;
        }
        if args.len() != n {
            let message = format!(
                "`{what}` takes {n} argument{}, but {} {} given",
                if n == 1 { "" } else { "s" },
                args.len(),
                if args.len() == 1 { "was" } else { "were" }
            );
            self.error(codes::WRONG_ARG_COUNT, span, message);
            return None;
        }
        Some(args.iter().map(|a| &a.value).collect())
    }

    pub(in crate::lower) fn one_arg(
        &mut self,
        what: &str,
        args: &[ast::Arg],
        want: &Ty,
        span: Span,
    ) -> Option<Expr> {
        let exprs = self.positional_any(what, args, 1, span)?;
        Some(self.expect(exprs[0], want))
    }

    /// Calls the user's fn `id`. Arguments are matched by position, then
    /// by name, then from defaults.
    pub(in crate::lower) fn user_call(
        &mut self,
        mut id: FnId,
        receiver: Option<Arg>,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        if self.sigs[id].returns_fresh_domain && self.sigs[id].used_fresh_call {
            id = self.clone_fresh_domain_fn(id);
        }
        id = self.view_param_fn(id, args);
        self.queue_fn(id);
        let thunked: Vec<ast::Arg>;
        let decl_params: Vec<&ast::Param> = self.sigs[id]
            .decl
            .params
            .iter()
            .filter(|p| p.name.text != "self")
            .collect();
        let args = match super::mono::thunk_lazy_args(&decl_params, args) {
            Some(t) => {
                thunked = t;
                thunked.as_slice()
            }
            None => args,
        };
        let params = self.sigs[id].params.clone();
        let name = self.sigs[id].decl.name.text.clone();
        let mut given: Vec<Option<&ast::Arg>> = vec![None; params.len()];
        let mut next = 0;
        let mut ok = true;
        for arg in args {
            let index = match &arg.name {
                Some(arg_name) => match params.iter().position(|p| p.label == arg_name.text) {
                    Some(index) => index,
                    None => {
                        let message =
                            format!("`{name}` has no parameter named `{}`", arg_name.text);
                        self.error(codes::UNKNOWN_ARG_NAME, arg_name.span, message);
                        ok = false;
                        continue;
                    }
                },
                None => {
                    while next < params.len() && given[next].is_some() {
                        next += 1;
                    }
                    next
                }
            };
            if index >= params.len() {
                let message = format!(
                    "`{name}` takes {} argument{}, but {} were given",
                    params.len(),
                    if params.len() == 1 { "" } else { "s" },
                    args.len()
                );
                self.error(codes::WRONG_ARG_COUNT, span, message);
                ok = false;
                break;
            }
            given[index] = Some(arg);
        }
        let mut lowered: Vec<Arg> = receiver.into_iter().collect();
        let mut pre_stmts: Vec<Stmt> = Vec::new();
        let mut post_stmts: Vec<Stmt> = Vec::new();
        // A domain the fn shares between its calls stands for this call's
        // array until it returns; then it is what it was, for a caller
        // further out that is the same fn.
        let mut put_back: Vec<Stmt> = Vec::new();
        for param in &params {
            let Ty::Array { dims, .. } = &param.ty else {
                continue;
            };
            let [Dim::Pos(d)] = dims.as_slice() else {
                continue;
            };
            let d = *d;
            let known = put_back
                .iter()
                .any(|s| matches!(s, Stmt::DefDomain { id, .. } if *id == d));
            if !self.range_domains_shared.contains(&d) || known {
                continue;
            }
            let (lo, len) = (self.temp(Ty::Int), self.temp(Ty::Int));
            pre_stmts.push(Stmt::Let(
                lo,
                Some(self.mk(ExprKind::DomainLo(d), Ty::Int, span)),
            ));
            pre_stmts.push(Stmt::Let(
                len,
                Some(self.mk(ExprKind::DomainLen(d), Ty::Int, span)),
            ));
            put_back.push(Stmt::DefDomain {
                id: d,
                lo: self.local(lo, span),
                len: self.local(len, span),
            });
        }
        // The domains a `mut` array sizes, known before the loop: its
        // size is set ahead of every argument.
        let mut sized = HashSet::new();
        for (param, arg) in params.iter().zip(&given) {
            let Some(arg) = arg.filter(|_| param.mutable) else {
                continue;
            };
            if let Some((d, _)) = self
                .probe_ty_silent(&arg.value)
                .and_then(|ty| self.range_domain_of(&param.ty, &ty))
            {
                sized.insert(d);
            }
        }
        let outer_sized = self.call_sized.replace(sized);
        let mut mut_sized = HashSet::new();
        for (at, (param, arg)) in params.iter().zip(given).enumerate() {
            match arg {
                Some(arg) if param.mutable => {
                    if !arg.mutable {
                        let message =
                            format!("`{name}` changes `{}`, so pass it with `mut`", param.name);
                        self.error(codes::MISSING_MUT_AT_CALL, arg.value.span, message);
                        ok = false;
                    }
                    let place = if let ast::ExprKind::Call {
                        callee,
                        args: pargs,
                    } = &arg.value.kind
                    {
                        if let ast::ExprKind::Field { base, name: pname } = &callee.kind {
                            if let Some(proj) = self.find_projection_on(base, &pname.text) {
                                self.lower_projection_mut_call(
                                    proj,
                                    base,
                                    pargs,
                                    arg.value.span,
                                    &mut pre_stmts,
                                    &mut post_stmts,
                                )
                            } else {
                                self.place(&arg.value)
                            }
                        } else {
                            self.place(&arg.value)
                        }
                    } else {
                        self.place(&arg.value)
                    };
                    if let PlaceKind::Local(s) = match &place.kind {
                        PlaceKind::Local(s) => PlaceKind::Local(*s),
                        _ => PlaceKind::Global(usize::MAX),
                    } {
                        self.cx.mut_slots.insert(s);
                    }
                    if place.ty == Ty::Bool && matches!(place.kind, PlaceKind::Index(..)) {
                        self.unsupported(arg.value.span, "a `Bool` element passed as `mut`");
                        ok = false;
                    }
                    let mut place = place;
                    let synthetic_dom = self.range_domain_of(&param.ty, &place.ty).or_else(|| {
                        if self
                            .list_given(&param.ty)
                            .is_some_and(|list| list == place.ty)
                        {
                            if let Ty::Array { dims, .. } = &param.ty {
                                if let [Dim::Pos(d)] = dims.as_slice() {
                                    return Some((*d, 0));
                                }
                            }
                        }
                        None
                    });
                    if let Some((d, lo)) = synthetic_dom {
                        let len = self.mk(
                            ExprKind::Builtin(
                                Builtin::Len,
                                vec![super::stmt::place_to_expr(&place)],
                            ),
                            Ty::Int,
                            arg.value.span,
                        );
                        // Before any argument is worked out, and so before
                        // those given by value: they come second.
                        let size = match mut_sized.insert(d) {
                            true => Stmt::DefDomain {
                                id: d,
                                lo: self.int_lit(lo, arg.value.span),
                                len,
                            },
                            false => self.size_domain(d, lo, len, arg.value.span),
                        };
                        pre_stmts.push(size);
                        place.ty = param.ty.clone();
                    }
                    if place.ty != param.ty && place.ty != Ty::Error && param.ty != Ty::Error {
                        self.mismatch(arg.value.span, &param.ty, &place.ty);
                        ok = false;
                    }
                    lowered.push(Arg::Mut(place));
                }
                Some(arg) => {
                    if param.ty == Ty::Unit && self.detect_bare_op(&arg.value).is_some() {
                        lowered.push(Arg::Value(self.mk(
                            ExprKind::Unit,
                            Ty::Unit,
                            arg.value.span,
                        )));
                    } else if param.ty == Ty::Opt(Box::new(Ty::Unit))
                        && self.detect_bare_op(&arg.value).is_some()
                    {
                        let unit = self.mk(ExprKind::Unit, Ty::Unit, arg.value.span);
                        let some = ExprKind::Some(Box::new(unit));
                        lowered.push(Arg::Value(self.mk(some, param.ty.clone(), arg.value.span)));
                    } else if let Some(list) = self.list_given(&param.ty) {
                        // A list where a `[D]T` is asked for is made as
                        // the list it is; passing it sizes `D`.
                        let value = self.expect_keeping_view(&arg.value, list, &param.ty);
                        lowered.push(Arg::Value(self.coerce(value, &param.ty, arg.value.span)));
                    } else if let Some((d, first, other)) = self.other_first(&param.ty, &arg.value)
                    {
                        let message = format!(
                            "`{name}` takes `{}` indexed by the same `{}` as an argument before it, which starts at {first}; this one starts at {other}",
                            param.name,
                            self.domain_said(d)
                        );
                        self.error(codes::TYPE_MISMATCH, arg.value.span, message);
                        ok = false;
                    } else {
                        let declared = self.sigs[id]
                            .decl
                            .params
                            .get(at)
                            .and_then(|p| p.ty.as_ref());
                        lowered.push(Arg::Value(
                            self.expect_declared(&arg.value, &param.ty, declared),
                        ));
                    }
                }
                None => match param.default {
                    Some(default) if !param.mutable => {
                        let sm = std::mem::replace(&mut self.cur_mod, self.sigs[id].mod_id);
                        let saved = std::mem::replace(&mut self.offset, self.sigs[id].offset);
                        let se = std::mem::replace(&mut self.gen_env, self.sigs[id].env.clone());
                        let value =
                            if param.ty == Ty::Unit && self.detect_bare_op(default).is_some() {
                                self.mk(ExprKind::Unit, Ty::Unit, default.span)
                            } else {
                                self.expect(default, &param.ty)
                            };
                        self.cur_mod = sm;
                        self.offset = saved;
                        self.gen_env = se;
                        lowered.push(Arg::Value(value));
                    }
                    _ => {
                        if ok {
                            let message = format!("`{name}` needs the argument `{}`", param.name);
                            self.error(codes::WRONG_ARG_COUNT, span, message);
                        }
                        ok = false;
                    }
                },
            }
        }
        self.call_sized = outer_sized;
        let ret = self.ret_of(id, span);
        if self.sigs[id].returns_fresh_domain {
            self.sigs[id].used_fresh_call = true;
        }
        if !ok {
            return self.err_expr(span);
        }
        post_stmts.extend(put_back);
        let call_expr = self.call_holding_cells(id, lowered, ret.clone(), span);
        let call = if pre_stmts.is_empty() && post_stmts.is_empty() {
            call_expr
        } else {
            let mut stmts = pre_stmts;
            if ret == Ty::Unit {
                stmts.push(Stmt::Expr(call_expr));
                stmts.extend(post_stmts);
                stmts.push(Stmt::Give(self.mk(ExprKind::Unit, Ty::Unit, span)));
            } else {
                let res_slot = self.temp(ret.clone());
                stmts.push(Stmt::Let(res_slot, Some(call_expr)));
                stmts.extend(post_stmts);
                stmts.push(Stmt::Give(self.local(res_slot, span)));
            }
            self.mk(ExprKind::Do(Block { stmts }), ret, span)
        };
        // What the fn gives in terms of an array's plain range is the
        // caller's plain `Int`s.
        let plain = self.plain_ty(&call.ty);
        self.try_coerce(call, &plain).unwrap_or_else(|call| call)
    }

    /// `Edge(u: a, v: b, w: 5)` or `Edge(a, b, 5)`.
    /// `P(read() for _ in fields(P))`, `P(x + y for (x, y) in fields(a, b))`:
    /// the body once per field, giving that field (decision 28.9).
    /// `own` is the struct, tuple or record type built; a type parameter
    /// `T(f(x) for x in fields(a))` builds whatever `T` is.
    pub(in crate::lower) fn fields_ctor(
        &mut self,
        own: Ty,
        args: &[ast::Arg],
        span: Span,
    ) -> Option<Expr> {
        let [arg] = args else {
            return None;
        };
        let ast::ExprKind::Comprehension { body, clauses } = &arg.value.kind else {
            return None;
        };
        let [clause] = clauses.as_slice() else {
            return None;
        };
        if arg.name.is_some() {
            return None;
        }
        let values = self.reflected_values(&clause.iter)?;
        let field_tys = self.field_tys(&own)?;
        let own_name = self.show(&own);
        let mut stmts = Vec::new();
        let mut temps = Vec::new();
        for a in values {
            if let ast::ExprKind::Name(n) = &a.value.kind {
                if self.lookup(n).is_none() && n.starts_with(|c: char| c.is_ascii_uppercase()) {
                    let is_own = match self.gen_env.get(n) {
                        Some(GenArg::Ty(t)) => *t == own,
                        _ => {
                            matches!((self.get_sym(n), &own), (Some(Sym::Struct(sid)), Ty::Struct(id)) if sid == *id)
                        }
                    };
                    if !is_own {
                        let message =
                            format!("`fields({n})` here names the type it builds, `{own_name}`");
                        self.error(codes::TYPE_MISMATCH, a.value.span, message);
                        return Some(self.err_expr(span));
                    }
                    continue;
                }
            }
            let v = self.expr(&a.value, None);
            if v.ty == Ty::Error {
                return Some(self.err_expr(span));
            }
            // Values of other types walk along when they have as many fields.
            let tys = match self.field_tys(&v.ty) {
                Some(tys) if tys.len() == field_tys.len() => tys,
                _ => {
                    let message = format!(
                        "`fields` here walks values with the fields of the type it builds, `{own_name}`; this is a `{}`",
                        self.show(&v.ty)
                    );
                    self.error(codes::TYPE_MISMATCH, a.value.span, message);
                    return Some(self.err_expr(span));
                }
            };
            let slot = self.temp(v.ty.clone());
            stmts.push(Stmt::Let(slot, Some(v)));
            temps.push((slot, a.value.span, tys));
        }
        let mut items = Vec::new();
        for (i, fty) in field_tys.iter().enumerate() {
            self.cx.scopes.push(HashMap::new());
            self.cx.local_syms.push(HashMap::new());
            if !temps.is_empty() {
                let mut vals: Vec<Expr> = temps
                    .iter()
                    .map(|(slot, sp, tys)| {
                        let base = self.local(*slot, *sp);
                        self.mk(ExprKind::Field(Box::new(base), i), tys[i].clone(), *sp)
                    })
                    .collect();
                let value = if vals.len() == 1 {
                    vals.pop().unwrap()
                } else {
                    let ty = Ty::Tuple(vals.iter().map(|e| e.ty.clone()).collect());
                    self.mk(ExprKind::Tuple(vals), ty, temps[0].1)
                };
                self.bind(&clause.pat, value, &mut stmts);
            }
            let v = self.expect(body, fty);
            let range = match &own {
                Ty::Struct(id) => self
                    .struct_field_range(*id, i)
                    .and_then(|r| r.scalar().cloned()),
                _ => None,
            };
            let v = match range {
                Some(b) => self.check_range(v, &b, body.span),
                None => v,
            };
            let slot = self.temp(fty.clone());
            stmts.push(Stmt::Let(slot, Some(v)));
            items.push(self.local(slot, body.span));
            self.cx.local_syms.pop();
            self.cx.scopes.pop();
        }
        let value = self.mk(ExprKind::Tuple(items), own.clone(), span);
        stmts.push(Stmt::Give(value));
        Some(self.mk(ExprKind::Do(Block { stmts }), own, span))
    }

    pub(in crate::lower) fn construct(
        &mut self,
        id: StructId,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        if let Some(e) = self.fields_ctor(Ty::Struct(id), args, span) {
            return e;
        }
        let fields = self.structs[id].def.fields.clone();
        let Some(decl) = self.structs[id].decl else {
            if self.structs[id].def.name.starts_with("_local_") {
                let name = self.structs[id].def.name.clone();
                let mut given: Vec<Option<&ast::Arg>> = vec![None; fields.len()];
                for (i, arg) in args.iter().enumerate() {
                    let index = match &arg.name {
                        Some(arg_name) => fields.iter().position(|(f, _)| *f == arg_name.text),
                        None => (i < fields.len()).then_some(i),
                    };
                    match index {
                        Some(index) => given[index] = Some(arg),
                        None => {
                            let message = match &arg.name {
                                Some(arg_name) => {
                                    format!("`{name}` has no field `{}`", arg_name.text)
                                }
                                None => format!(
                                    "`{name}` has {} fields, but {} values were given",
                                    fields.len(),
                                    args.len()
                                ),
                            };
                            self.error(codes::WRONG_ARG_COUNT, arg.value.span, message);
                            return self.err_expr(span);
                        }
                    }
                }
                let mut values = Vec::new();
                for ((field, ty), arg) in fields.iter().zip(given) {
                    if let Some(arg) = arg {
                        values.push(self.expect(&arg.value, ty));
                    } else {
                        self.error(
                            codes::WRONG_ARG_COUNT,
                            span,
                            format!("`{name}` needs a value for `{field}`"),
                        );
                        return self.err_expr(span);
                    }
                }
                return self.mk(ExprKind::Tuple(values), Ty::Struct(id), span);
            }
            self.error(
                codes::NOT_CALLABLE,
                span,
                format!(
                    "`{}` cannot be constructed directly",
                    self.structs[id].def.name
                ),
            );
            return self.err_expr(span);
        };
        let name = decl.name.text.clone();
        let mut given: Vec<Option<&ast::Arg>> = vec![None; fields.len()];
        for (i, arg) in args.iter().enumerate() {
            let index = match &arg.name {
                Some(arg_name) => fields.iter().position(|(f, _)| *f == arg_name.text),
                None => (i < fields.len()).then_some(i),
            };
            match index {
                Some(index) => given[index] = Some(arg),
                None => {
                    let message = match &arg.name {
                        Some(arg_name) => format!("`{name}` has no field `{}`", arg_name.text),
                        None => format!(
                            "`{name}` has {} fields, but {} values were given",
                            fields.len(),
                            args.len()
                        ),
                    };
                    self.error(codes::WRONG_ARG_COUNT, arg.value.span, message);
                    return self.err_expr(span);
                }
            }
        }
        let mut values = Vec::new();
        for (i, ((field, ty), arg)) in fields.iter().zip(given).enumerate() {
            match (arg, &decl.fields[i].default) {
                (Some(arg), _) => {
                    let v = if *ty == Ty::Unit && self.detect_bare_op(&arg.value).is_some() {
                        self.mk(ExprKind::Unit, Ty::Unit, arg.value.span)
                    } else {
                        self.expect_declared(&arg.value, ty, Some(&decl.fields[i].ty))
                    };
                    let v = match self
                        .struct_field_range(id, i)
                        .and_then(|r| r.scalar().cloned())
                    {
                        Some(b) => self.check_range(v, &b, arg.value.span),
                        None => v,
                    };
                    values.push(v);
                }
                (None, Some(default)) => {
                    let saved = std::mem::replace(&mut self.offset, self.structs[id].offset);
                    let v = if matches!(ty, Ty::Array { .. }) {
                        self.let_value(Some(&decl.fields[i].ty), Some(default), default.span)
                            .unwrap_or_else(|| self.err_expr(default.span))
                    } else {
                        self.expect(default, ty)
                    };
                    let v = match self
                        .struct_field_range(id, i)
                        .and_then(|r| r.scalar().cloned())
                    {
                        Some(b) => self.check_range(v, &b, default.span),
                        None => v,
                    };
                    values.push(v);
                    self.offset = saved;
                }
                (None, None) => {
                    let message = format!("`{name}` needs a value for its field `{field}`");
                    self.error(codes::WRONG_ARG_COUNT, span, message);
                    return self.err_expr(span);
                }
            }
        }
        self.mk(ExprKind::Tuple(values), Ty::Struct(id), span)
    }

    /// Lowers `e` for a field or parameter of type `want`, declared as
    /// `declared`. `Node([none, none], 0)`: a list written out where a
    /// one-index array is declared is the cells of that array, as in
    /// `let kid: [0..2]?Int = [none, none]`, with the same count check.
    /// An array given for a `D` that an argument before bound to arrays
    /// starting elsewhere: the domain, its first index, and this one's.
    pub(in crate::lower) fn other_first(
        &mut self,
        want: &Ty,
        arg: &ast::Expr,
    ) -> Option<(DomainId, i64, i64)> {
        let Ty::Array { dims, .. } = want else {
            return None;
        };
        let [Dim::Pos(d)] = dims.as_slice() else {
            return None;
        };
        let first = *self.range_domains.get(d)?;
        let other = match self.probe_ty_silent(arg)? {
            Ty::Array { dims, .. } => match dims.as_slice() {
                [Dim::Int] => 0,
                [Dim::IntFrom(lo)] => *lo,
                _ => return None,
            },
            _ => return None,
        };
        (other != first).then_some((*d, first, other))
    }

    pub(in crate::lower) fn expect_declared(
        &mut self,
        e: &ast::Expr,
        want: &Ty,
        declared: Option<&ast::Type>,
    ) -> Expr {
        let cells = matches!(&e.kind, ast::ExprKind::List(items) if !items.is_empty())
            && matches!(want, Ty::Array { dims, .. } if matches!(dims[..], [Dim::Int | Dim::IntFrom(_)]));
        let ranged_comp = matches!(&e.kind, ast::ExprKind::Table { .. })
            && declared.is_some_and(|d| {
                self.range_info_from_ast(d)
                    .and_then(|r| r.elem().and_then(|e| e.scalar()).cloned())
                    .is_some()
            });
        match declared.filter(|_| cells || ranged_comp) {
            Some(declared) => match self.let_value(Some(declared), Some(e), e.span) {
                Some(value) => self.coerce(value, want, e.span),
                None => self.err_expr(e.span),
            },
            None => self.expect(e, want),
        }
    }
}

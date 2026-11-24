//! Operators: unary, binary, casts, comparisons, and `none`-absorbing
//! (tropical) `+` and `-`.

use super::*;

impl<'a> Lowerer<'a> {
    pub(in crate::lower) fn unary(
        &mut self,
        op: UnOp,
        operand: &ast::Expr,
        hint: Option<&Ty>,
        span: Span,
    ) -> Expr {
        match op {
            UnOp::Pos => {
                let value = self.expr(operand, hint);
                match &value.ty {
                    Ty::Int | Ty::I128 | Ty::U64 | Ty::Float | Ty::Error => value,
                    _ => {
                        let message =
                            format!("`+` can't be applied to a `{}`", self.show(&value.ty));
                        self.error(codes::BAD_OPERANDS, span, message);
                        self.err_expr(span)
                    }
                }
            }
            UnOp::Neg => {
                let value = self.expr(operand, hint);
                match (&value.kind, &value.ty) {
                    (ExprKind::Int(v), _) => self.mk(ExprKind::Int(-*v), Ty::Int, span),
                    (ExprKind::Float(v), _) => self.mk(ExprKind::Float(-*v), Ty::Float, span),
                    (_, Ty::Int | Ty::I128 | Ty::U64 | Ty::Float) => {
                        let ty = value.ty.clone();
                        self.mk_unary(op, value, ty, span)
                    }
                    (_, Ty::Struct(sid)) => {
                        let sid = *sid;
                        if let Some(fn_id) = self.struct_op_fn(sid, "neg") {
                            let ret = self.ret_of(fn_id, span);
                            return self.mk(
                                ExprKind::Call(fn_id, vec![Arg::Value(value)]),
                                ret,
                                span,
                            );
                        }
                        let message =
                            format!("`-` can't be applied to a `{}`", self.show(&value.ty));
                        self.error(codes::BAD_OPERANDS, span, message);
                        self.err_expr(span)
                    }
                    (_, Ty::Error) => value,
                    // `-x` on a `?Int` or `?F64` keeps `none`.
                    (_, Ty::Opt(inner)) if matches!(**inner, Ty::Int | Ty::Float) => {
                        let zero = if **inner == Ty::Int {
                            self.mk(ExprKind::Int(0), Ty::Int, span)
                        } else {
                            self.mk(ExprKind::Float(0.0), Ty::Float, span)
                        };
                        self.tropical(BinOp::Sub, zero, value, span)
                    }
                    _ => {
                        let message =
                            format!("`-` can't be applied to a `{}`", self.show(&value.ty));
                        self.error(codes::BAD_OPERANDS, span, message);
                        self.err_expr(span)
                    }
                }
            }
            UnOp::Not => {
                let value = self.expect(operand, &Ty::Bool);
                self.mk_unary(op, value, Ty::Bool, span)
            }
            UnOp::BitNot => {
                let value = self.expr(operand, hint.filter(|ty| matches!(ty, Ty::I128 | Ty::U64)));
                if let Ty::Opt(inner) = &value.ty {
                    if **inner == Ty::Int {
                        let neg_one = self.mk(ExprKind::Int(-1), Ty::Int, span);
                        return self.tropical(BinOp::BitXor, value, neg_one, span);
                    }
                }
                let ty = match value.ty {
                    Ty::I128 | Ty::U64 | Ty::Error => value.ty.clone(),
                    _ => Ty::Int,
                };
                let value = self.coerce(value, &ty, operand.span);
                self.mk_unary(op, value, ty, span)
            }
        }
    }

    pub(in crate::lower) fn cast(&self, cast: Cast, value: Expr, ty: Ty) -> Expr {
        let span = value.span;
        Expr {
            kind: self.vouch_pos(ExprKind::Cast(cast, Box::new(value)), &ty, span),
            ty,
            span,
        }
    }

    pub(in crate::lower) fn binary(
        &mut self,
        op: BinOp,
        lhs: &ast::Expr,
        rhs: &ast::Expr,
        hint: Option<&Ty>,
        span: Span,
    ) -> Expr {
        use BinOp::*;
        match op {
            And | Or => {
                let l = self.expect(lhs, &Ty::Bool);
                let past = self.cx.present.len();
                let mutators = self.mutators.clone();
                let held = narrow::in_rhs(op, lhs, rhs, &mut |f, x| mutators.writes(f, x));
                self.cx.present.extend(held.into_iter().map(String::from));
                let r = self.worked_out_here(rhs.span, |lw| lw.expect(rhs, &Ty::Bool));
                self.cx.present.truncate(past);
                return self.mk_binary(op, l, r, Ty::Bool, span);
            }
            Coalesce => {
                self.cx.as_written = narrow::asks_optional(lhs);
                let l = if has_try(lhs) {
                    self.try_extent(lhs, span)
                } else {
                    self.expr_maybe_opt_reduce(lhs)
                };
                self.cx.as_written = false;
                let Ty::Opt(inner) = l.ty.clone() else {
                    if l.ty != Ty::Error {
                        let message = format!(
                            "`??` gives a default for an optional, but the left side is a `{}`",
                            self.show(&l.ty)
                        );
                        self.error(codes::BAD_OPERANDS, span, message);
                        self.expr(rhs, None);
                    }
                    return self.err_expr(span);
                };
                if let ast::ExprKind::Block(block) = &rhs.kind {
                    if let [way_out @ ast::Stmt {
                        kind:
                            ast::StmtKind::Return(_)
                            | ast::StmtKind::Break(_)
                            | ast::StmtKind::Continue(_),
                        ..
                    }] = block.stmts.as_slice()
                    {
                        if self.cx.way_out.take() != Some(span) {
                            self.error(
                                codes::UNSUPPORTED,
                                rhs.span,
                                "`?? return`, `?? break` and `?? continue` have to be the whole right side of a `let`, an assignment or a statement",
                            );
                            self.help("give the value a name first: `let u = o ?? return`");
                            return self.err_expr(span);
                        }
                        let slot = self.temp(l.ty.clone());
                        let held = self.local(slot, span);
                        let some = self.mk_is_some(held.clone(), Ty::Bool, span);
                        let missing = self.mk_unary(UnOp::Not, some, Ty::Bool, span);
                        let mut leave = Vec::new();
                        // The way out is lowered as the statement it is,
                        // outside any value block.
                        let dos = std::mem::take(&mut self.cx.dos);
                        self.stmt(way_out, &mut leave);
                        self.cx.dos = dos;
                        self.cx.pending.push(Stmt::Let(slot, Some(l)));
                        self.cx.pending.push(Stmt::If(
                            missing,
                            Block { stmts: leave },
                            Block::default(),
                        ));
                        return self.mk_unwrap(held, false, *inner, span);
                    }
                }
                // The default is read as what the optional holds, so that a
                // plain value stays one when the left side is an optional of
                // optionals (`xs.last ?? 7` on a `List<?Int>`). Only a bare
                // `none` needs the optional's own type to be one.
                let hint = match rhs.kind {
                    ast::ExprKind::None => &l.ty,
                    _ => &*inner,
                };
                let r = self.worked_out_here(rhs.span, |lw| lw.expr(rhs, Some(hint)));
                // `a ?? b ?? 0`: an optional default keeps the result optional.
                let ty = if r.ty == l.ty { l.ty.clone() } else { *inner };
                let r = self.coerce(r, &ty, rhs.span);
                return self.mk_binary(op, l, r, ty, span);
            }
            _ => {}
        }
        let unwrap_first_last = |e: Expr| -> Expr {
            match (&e.kind, &e.ty) {
                (ExprKind::Builtin(Builtin::First, args), Ty::Opt(inner)) => Expr {
                    kind: ExprKind::Builtin(Builtin::FirstVal, args.clone()),
                    ty: (**inner).clone(),
                    span: e.span,
                },
                (ExprKind::Builtin(Builtin::Last, args), Ty::Opt(inner)) => Expr {
                    kind: ExprKind::Builtin(Builtin::LastVal, args.clone()),
                    ty: (**inner).clone(),
                    span: e.span,
                },
                _ => e,
            }
        };
        // Where an `I128` or a `U64` is wanted, the arithmetic is done in
        // one: `let p: I128 = a * b` and `let top: U64 = 1 << 63` do not
        // overflow an `Int` on the way.
        let wide = hint.filter(|ty| matches!(ty, Ty::I128 | Ty::U64));
        // Where a list is wanted, a list written out on the left of `+`
        // holds what the list wanted holds: `let xs: List<Z> = [0] + rest`.
        let joined = hint.filter(|ty| {
            op == Add
                && matches!(ty, Ty::List(_))
                && matches!(
                    lhs.kind,
                    ast::ExprKind::List(_) | ast::ExprKind::Binary { .. } | ast::ExprKind::Paren(_)
                )
        });
        let joined_owned = if op == Add
            && joined.is_none()
            && matches!(&lhs.kind, ast::ExprKind::List(items) if items.is_empty())
        {
            self.probe_ty_silent(rhs)
                .and_then(|ty| seq_concat_elem(&ty).cloned())
                .map(Ty::list)
        } else {
            None
        };
        let mut l = unwrap_first_last(self.expr(lhs, wide.or(joined).or(joined_owned.as_ref())));
        if let (Some(wide), Ty::Int) = (wide, &l.ty) {
            l = self.coerce(l, wide, lhs.span);
        }
        // A place, a parameter or a return type says what `read()` reads;
        // the other side of an operator does not.
        // A shift is by an `Int`, whatever is shifted.
        let r_concat_hint = (op == Add && !is_bare_read(rhs))
            .then(|| seq_concat_elem(&l.ty).cloned().map(Ty::list))
            .flatten();
        let r_hint = match op {
            Shl | Shr | Pow => Some(&Ty::Int),
            _ if r_concat_hint.is_some() => r_concat_hint.as_ref(),
            _ if matches!(l.ty, Ty::Pos(_) | Ty::Bound(_)) => None,
            _ => (!is_bare_read(rhs)).then_some(&l.ty),
        };
        let r = unwrap_first_last(self.expr(rhs, r_hint));
        // Dispatch operator traits (`impl Add/Sub/Mul/Div for Struct`).
        if let Ty::Struct(sid) = l.ty {
            // `z ** e` is the struct's `pow(e: Int)`: `Mod<P>` has one.
            if let (Pow, Ty::Int, Some(fn_id)) = (op, &r.ty, self.own_method(sid, "pow")) {
                self.queue_fn(fn_id);
                let ret = self.ret_of(fn_id, span);
                return self.mk(
                    ExprKind::Call(fn_id, vec![Arg::Value(l), Arg::Value(r)]),
                    ret,
                    span,
                );
            }
            let mname = match op {
                Add => Some("add"),
                Sub => Some("sub"),
                Mul => Some("mul"),
                Div => Some("div"),
                Rem => Some("rem"),
                BitOr => Some("bitor"),
                BitAnd => Some("bitand"),
                BitXor => Some("bitxor"),
                _ => None,
            };
            if let Some((fn_id, want_rhs)) =
                mname.and_then(|m| self.struct_op_fn_for(sid, m, &r.ty))
            {
                let r = self.coerce(r, &want_rhs, rhs.span);
                let ret = self.ret_of(fn_id, span);
                return self.mk(
                    ExprKind::Call(fn_id, vec![Arg::Value(l), Arg::Value(r)]),
                    ret,
                    span,
                );
            }
        }
        // `1 - z` with `z: Mod<P>`: the number becomes a `Mod<P>` first,
        // as it does on the right in `z - 1`.
        if let (Ty::Int, Ty::Struct(sid)) = (&l.ty, &r.ty) {
            let sid = *sid;
            let mname = match op {
                Add => Some("add"),
                Sub => Some("sub"),
                Mul => Some("mul"),
                Div => Some("div"),
                _ => None,
            };
            if self.is_number(sid) {
                if let Some(fn_id) = mname.and_then(|m| self.struct_op_fn(sid, m)) {
                    let l = self.coerce(l, &Ty::Struct(sid), lhs.span);
                    let ret = self.ret_of(fn_id, span);
                    return self.mk(
                        ExprKind::Call(fn_id, vec![Arg::Value(l), Arg::Value(r)]),
                        ret,
                        span,
                    );
                }
            }
        }
        let arith = matches!(op, Add | Sub | Mul | Div | Pow);
        let (l, r, ty) = match (&l.ty, &r.ty) {
            (Ty::Error, _) | (_, Ty::Error) => return self.err_expr(span),
            (a, b)
                if (matches!(op, Add | Sub | Mul | Div)
                    && tropical_side(a).is_some()
                    && tropical_side(b).is_some()
                    || matches!(op, Rem | Pow | BitAnd | BitOr | BitXor | Shl | Shr)
                        && tropical_side(a) == Some(Ty::Int)
                        && tropical_side(b) == Some(Ty::Int))
                    && (matches!(a, Ty::Opt(_)) || matches!(b, Ty::Opt(_))) =>
            {
                return self.tropical(op, l, r, span);
            }
            (Ty::Int, Ty::Int) => (l, r, Ty::Int),
            // A shift keeps the type of what is shifted.
            (Ty::I128 | Ty::U64, Ty::Int) if matches!(op, Shl | Shr) => {
                let ty = l.ty.clone();
                (l, r, ty)
            }
            // An `Int` beside an `I128` or a `U64` becomes one.
            (wide @ (Ty::I128 | Ty::U64), Ty::Int) | (Ty::Int, wide @ (Ty::I128 | Ty::U64))
                if !matches!(op, Shl | Shr) =>
            {
                let wide = wide.clone();
                let l = self.coerce(l, &wide, lhs.span);
                let r = self.coerce(r, &wide, rhs.span);
                (l, r, wide)
            }
            (Ty::I128, Ty::I128) | (Ty::U64, Ty::U64) if !matches!(op, Shl | Shr) => {
                let ty = l.ty.clone();
                (l, r, ty)
            }
            (Ty::Float, Ty::Float) if arith => (l, r, Ty::Float),
            (Ty::Float, Ty::Int) if arith => {
                let r = self.coerce(r, &Ty::Float, rhs.span);
                (l, r, Ty::Float)
            }
            (Ty::Int, Ty::Float) if arith => {
                let l = self.coerce(l, &Ty::Float, lhs.span);
                (l, r, Ty::Float)
            }
            (Ty::Bool, Ty::Bool) if matches!(op, BitAnd | BitOr | BitXor) => (l, r, Ty::Bool),
            (Ty::Str | Ty::StrView | Ty::Char, Ty::Str | Ty::StrView)
            | (Ty::Str | Ty::StrView, Ty::Char)
                if op == Add =>
            {
                let has_view = l.ty == Ty::StrView || r.ty == Ty::StrView;
                if has_view {
                    let l = if l.ty == Ty::Char {
                        self.as_str(l, lhs.span)
                    } else {
                        l
                    };
                    let r = if r.ty == Ty::Char {
                        self.as_str(r, rhs.span)
                    } else {
                        r
                    };
                    return self.mk_builtin(Builtin::StrConcat, [l, r], Ty::Str, span);
                }
                let l = self.as_str(l, lhs.span);
                let r = self.as_str(r, rhs.span);
                (l, r, Ty::Str)
            }
            (a, b)
                if op == Add
                    && seq_concat_elem(a).is_some()
                    && seq_concat_elem(a) == seq_concat_elem(b) =>
            {
                let elem = seq_concat_elem(a).unwrap().clone();
                let list_ty = Ty::list(elem);
                if !matches!(l.ty, Ty::List(_)) || !matches!(r.ty, Ty::List(_)) {
                    return self.mk_builtin(Builtin::ListConcat, [l, r], list_ty, span);
                }
                return self.do_expr(Some(&list_ty), span, |lw, out| {
                    let slot = lw.temp(list_ty.clone());
                    let cloned = lw.mk_builtin(Builtin::Copy, [l], list_ty.clone(), span);
                    out.push(Stmt::Let(slot, Some(cloned)));
                    let ext = lw.mk_mutate(
                        Mutation::Extend,
                        Place {
                            kind: PlaceKind::Local(slot),
                            ty: list_ty.clone(),
                            span: lw.span(span),
                        },
                        vec![r],
                        Ty::Unit,
                        span,
                    );
                    out.push(Stmt::Expr(ext));
                    let res = lw.local(slot, span);
                    out.push(Stmt::Give(res));
                });
            }
            (Ty::Char, Ty::Char) if op == Sub => {
                let l = self.cast(Cast::CharToInt, l, Ty::Int);
                let r = self.cast(Cast::CharToInt, r, Ty::Int);
                (l, r, Ty::Int)
            }
            (Ty::Char, Ty::Int) if matches!(op, Add | Sub) => {
                let l = self.cast(Cast::CharToInt, l, Ty::Int);
                let sum = self.mk_binary(op, l, r, Ty::Int, span);
                return self.cast(Cast::StepToChar, sum, Ty::Char);
            }
            (Ty::Pos(a), Ty::Pos(b)) | (Ty::Bound(a), Ty::Bound(b)) if a == b && op == Sub => {
                let l = self.cast(Cast::Ord, l, Ty::Int);
                let r = self.cast(Cast::Ord, r, Ty::Int);
                (l, r, Ty::Int)
            }
            (Ty::Pos(d), Ty::Int) if matches!(op, Add | Sub) => {
                let d = *d;
                (self.cast(Cast::Label(d), l, Ty::Int), r, Ty::Int)
            }
            (Ty::List(elem), Ty::Int) if op == Mul => {
                let elem = (**elem).clone();
                return self.list_repeat(l, r, elem, span);
            }
            _ => {
                let message = format!(
                    "`{}` can't be applied to `{}` and `{}`",
                    op.text(),
                    self.show(&l.ty),
                    self.show(&r.ty)
                );
                self.error(codes::BAD_OPERANDS, span, message);
                return self.err_expr(span);
            }
        };
        if let Some(wrap) = self.wrapping(op, &ty).filter(|_| l.ty == ty && r.ty == ty) {
            return self.mk_builtin(wrap, [l, r], ty, span);
        }
        self.mk_binary(op, l, r, ty, span)
    }

    /// `a + b` and `a - b` where a side is a `?Int` or `?F64`: `none` on
    /// either side makes the whole `none`.
    pub(in crate::lower) fn tropical(&mut self, op: BinOp, l: Expr, r: Expr, span: Span) -> Expr {
        let float =
            tropical_side(&l.ty) == Some(Ty::Float) || tropical_side(&r.ty) == Some(Ty::Float);
        let num = if float { Ty::Float } else { Ty::Int };
        let ty = Ty::opt(num.clone());
        self.do_expr(Some(&ty), span, |lw, out| {
            let mut present: Option<Expr> = None;
            let mut sides = Vec::with_capacity(2);
            for side in [l, r] {
                let side_ty = side.ty.clone();
                let slot = lw.temp(side_ty.clone());
                out.push(Stmt::Let(slot, Some(side)));
                let held = lw.local(slot, span);
                let value = if let Ty::Opt(inner) = &side_ty {
                    let some = lw.mk_is_some(held.clone(), Ty::Bool, span);
                    present = Some(match present.take() {
                        Some(first) => lw.mk_binary(BinOp::And, first, some, Ty::Bool, span),
                        None => some,
                    });
                    lw.mk_unwrap(held, false, (**inner).clone(), span)
                } else {
                    held
                };
                sides.push(lw.coerce(value, &num, span));
            }
            let r = sides.pop().expect("two sides");
            let l = sides.pop().expect("two sides");
            let sum = lw.mk_binary(op, l, r, num.clone(), span);
            let some = lw.coerce(sum, &ty, span);
            let none = lw.mk(ExprKind::None, ty.clone(), span);
            let cond = present.expect("one side is optional");
            let value = lw.mk_if(cond, some, none, ty.clone(), span);
            out.push(Stmt::Give(value));
        })
    }

    /// Whether `<` means something for the type.
    pub(in crate::lower) fn ordered(&self, ty: &Ty) -> bool {
        match ty {
            Ty::Int
            | Ty::I128
            | Ty::U64
            | Ty::Float
            | Ty::Bool
            | Ty::Char
            | Ty::Str
            | Ty::StrView
            | Ty::Pos(_)
            | Ty::Bound(_)
            | Ty::Unit => true,
            Ty::Tuple(items) => items.iter().all(|t| self.ordered(t)),
            Ty::Record(fields) => fields.iter().all(|(_, t)| self.ordered(t)),
            Ty::List(elem) | Ty::ListView(elem, _) | Ty::Keyed(elem, _) | Ty::MutView(elem, _) => {
                self.ordered(elem)
            }
            Ty::Struct(id) => {
                let fields: Vec<Ty> = self.structs[*id]
                    .def
                    .fields
                    .iter()
                    .map(|(_, t)| t.clone())
                    .collect();
                self.structs[*id].def.ord.is_some() || fields.iter().all(|t| self.ordered(t))
            }
            Ty::Enum(id) => {
                let payloads: Vec<Ty> = self.enums[*id]
                    .variants
                    .iter()
                    .flat_map(|(_, fs)| fs.clone())
                    .collect();
                payloads.iter().all(|t| self.ordered(t))
            }
            Ty::Error => true,
            _ => false,
        }
    }

    /// A comparison. One that tests for `none` reads its names as the
    /// optionals they were declared as.
    pub(in crate::lower) fn compare(
        &mut self,
        first: &ast::Expr,
        rest: &consts::Chain,
        span: Span,
    ) -> Expr {
        if !narrow::tests_none(first, rest) {
            return self.compare_sides(first, rest, span);
        }
        let present = std::mem::take(&mut self.cx.present);
        let value = self.compare_sides(first, rest, span);
        self.cx.present = present;
        value
    }
}

/// The number type of one side of a `none`-absorbing `+` or `-`: the type
/// itself, or what its optional holds.
pub(in crate::lower) fn tropical_side(ty: &Ty) -> Option<Ty> {
    let inner = match ty {
        Ty::Opt(inner) => &**inner,
        other => other,
    };
    matches!(inner, Ty::Int | Ty::Float).then(|| inner.clone())
}

pub(in crate::lower) fn seq_concat_elem(ty: &Ty) -> Option<&Ty> {
    match ty {
        Ty::List(e) | Ty::ListView(e, _) | Ty::Keyed(e, _) | Ty::MutView(e, _) => Some(e),
        Ty::Array { dims, elem } if dims.len() == 1 => Some(elem),
        _ => None,
    }
}

/// Whether a `?` is written somewhere in `e`.
pub(in crate::lower) fn has_try(e: &ast::Expr) -> bool {
    struct Find(bool);
    impl cpl_syntax::visit::Visitor for Find {
        fn expr(&mut self, e: &ast::Expr) {
            self.0 |= matches!(e.kind, ast::ExprKind::Try(_));
        }
    }
    let mut find = Find(false);
    cpl_syntax::visit::walk_expr(e, &mut find);
    find.0
}

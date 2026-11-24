//! Comparisons and their chains: `lo <= x < hi` runs each operand once,
//! left to right, and stops at the first false link. (R01.5.3)

use super::*;

impl<'a> Lowerer<'a> {
    pub(in crate::lower) fn compare_sides(
        &mut self,
        first: &ast::Expr,
        rest: &[(CmpOp, ast::Expr)],
        span: Span,
    ) -> Expr {
        if let [(op @ (CmpOp::In | CmpOp::NotIn), container)] = rest {
            let found = self.contains(first, container, span);
            return if *op == CmpOp::In {
                found
            } else {
                self.mk_unary(UnOp::Not, found, Ty::Bool, span)
            };
        }
        // `none == x` needs the other side's type first.
        let mut operands: Vec<(Expr, Span)> = Vec::new();
        let asts: Vec<&ast::Expr> = std::iter::once(first)
            .chain(rest.iter().map(|(_, e)| e))
            .collect();
        for (i, e) in asts.iter().enumerate() {
            let value = if matches!(e.kind, ast::ExprKind::None) {
                match (operands.last(), asts.get(i + 1)) {
                    (Some((prev, _)), _) => {
                        let hint = Ty::opt(strip_opt(&prev.ty));
                        self.expr(e, Some(&hint))
                    }
                    (None, Some(next)) => {
                        let next = self.expr(next, None);
                        let hint = Ty::opt(strip_opt(&next.ty));
                        self.expr(e, Some(&hint))
                    }
                    _ => self.expr(e, None),
                }
            } else if matches!(&e.kind, ast::ExprKind::List(_)) {
                match (operands.last(), asts.get(i + 1)) {
                    (Some((prev, _)), _) if matches!(prev.ty, Ty::List(_) | Ty::Array { .. }) => {
                        self.expr(e, Some(&prev.ty))
                    }
                    (Some((prev, _)), _) if matches!(&prev.ty, Ty::Opt(inner) if matches!(**inner, Ty::List(_) | Ty::Array { .. })) =>
                    {
                        let Ty::Opt(inner) = &prev.ty else {
                            unreachable!()
                        };
                        self.expr(e, Some(inner))
                    }
                    (None, Some(next)) if matches!(&e.kind, ast::ExprKind::List(items) if items.is_empty()) =>
                    {
                        let next = self.expr(next, None);
                        let hint = match &next.ty {
                            Ty::Opt(inner) if matches!(**inner, Ty::List(_) | Ty::Array { .. }) => {
                                inner.as_ref()
                            }
                            other => other,
                        };
                        self.expr(e, Some(hint))
                    }
                    _ => self.expr(e, None),
                }
            } else if matches!(
                &e.kind,
                ast::ExprKind::If { .. }
                    | ast::ExprKind::Paren(_)
                    | ast::ExprKind::Block(_)
                    | ast::ExprKind::Tuple(_)
                    | ast::ExprKind::Record(_)
                    | ast::ExprKind::Range { .. }
            ) || operands
                .last()
                .is_some_and(|(prev, _)| matches!(prev.ty, Ty::Enum(_)))
            {
                // `x == E.V(..)` of a generic enum: the variant takes `x`'s type arguments.
                self.expr(e, operands.last().map(|(prev, _)| &prev.ty))
            } else if operands
                .last()
                .is_some_and(|(prev, _)| matches!(prev.ty, Ty::Opt(_)))
            {
                if matches!(
                    &e.kind,
                    ast::ExprKind::Call { callee, .. }
                        if matches!(&callee.kind, ast::ExprKind::Name(n) if n == "some" || n == "Some")
                ) {
                    self.expr(e, operands.last().map(|(prev, _)| &prev.ty))
                } else {
                    self.expr_maybe_opt_reduce(e)
                }
            } else if matches!(e.kind, ast::ExprKind::Int(_))
                && operands.last().is_some_and(|(prev, _)| prev.ty == Ty::U64)
            {
                // `x == 0xFFFF_FFFF_FFFF_FFFF` beside a `U64` is one.
                self.expr(e, Some(&Ty::U64))
            } else {
                self.expr(e, None)
            };
            // A view compares by what it walks, and its walk is the next operand's hint.
            let value = self.shown_struct(value);
            operands.push((value, e.span));
        }
        if rest.len() == 1 {
            let (r, r_span) = operands.pop().expect("two operands");
            let (l, l_span) = operands.pop().expect("two operands");
            return self.cmp(rest[0].0, l, l_span, r, r_span, span);
        }
        // `lo <= x < hi`: each operand is evaluated once, left to right, and a
        // false link stops the chain before the next operand is read.
        let first = operands.remove(0);
        let links = rest.iter().map(|(op, _)| *op).zip(operands).collect();
        self.chain_from(first, links, true, span)
    }

    /// Each link holds its right operand, which the next link reads again.
    fn chain_from(
        &mut self,
        (left, l_span): (Expr, Span),
        mut links: Vec<(CmpOp, (Expr, Span))>,
        hold_left: bool,
        span: Span,
    ) -> Expr {
        let (op, (right, r_span)) = links.remove(0);
        if links.is_empty() {
            return self.cmp(op, left, l_span, right, r_span, span);
        }
        let mut stmts = Vec::new();
        let left = if hold_left {
            self.hold_unless_atom(left, &mut stmts)
        } else {
            left
        };
        let right = self.hold_unless_atom(right, &mut stmts);
        let one = self.cmp(op, left, l_span, right.clone(), r_span, span);
        let tail = self.chain_from((right, r_span), links, false, span);
        let both = ExprKind::Binary(BinOp::And, Box::new(one), Box::new(tail));
        let both = self.mk(both, Ty::Bool, span);
        if stmts.is_empty() {
            return both;
        }
        stmts.push(Stmt::Give(both));
        self.mk(ExprKind::Do(Block { stmts }), Ty::Bool, span)
    }

    fn hold_unless_atom(&mut self, value: Expr, stmts: &mut Vec<Stmt>) -> Expr {
        let atom = matches!(
            value.kind,
            ExprKind::Local(_)
                | ExprKind::Global(_)
                | ExprKind::Int(_)
                | ExprKind::Char(_)
                | ExprKind::Float(_)
        );
        if atom {
            value
        } else {
            self.hold(value, stmts)
        }
    }

    pub(in crate::lower) fn cmp(
        &mut self,
        op: CmpOp,
        l: Expr,
        l_span: Span,
        r: Expr,
        r_span: Span,
        span: Span,
    ) -> Expr {
        if l.ty == Ty::Error || r.ty == Ty::Error {
            return self.err_expr(span);
        }
        match op {
            CmpOp::In | CmpOp::NotIn => {
                self.unsupported(span, "`in` inside a chain of comparisons");
                return self.err_expr(span);
            }
            CmpOp::Order => {
                self.unsupported(span, "`<=>`");
                return self.err_expr(span);
            }
            _ => {}
        }
        // Bring both sides to one type.
        let (l, r) = if l.ty == r.ty
            || matches!(
                (&l.ty, &r.ty),
                (Ty::Str, Ty::StrView) | (Ty::StrView, Ty::Str)
            )
            || matches!(
                (&l.ty, &r.ty),
                (Ty::List(a), Ty::ListView(b, _) | Ty::Keyed(b, _) | Ty::MutView(b, _))
                    | (Ty::ListView(a, _) | Ty::Keyed(a, _) | Ty::MutView(a, _), Ty::List(b))
                    | (Ty::ListView(a, _) | Ty::Keyed(a, _), Ty::MutView(b, _))
                    | (Ty::MutView(a, _), Ty::ListView(b, _) | Ty::Keyed(b, _))
                    if a == b
            ) {
            (l, r)
        } else if matches!((&l.ty, &r.ty), (Ty::Int, Ty::U64) | (Ty::U64, Ty::Int)) {
            // An `Int` can be negative and a `U64` past `2**63`, so the
            // two are compared as the numbers they are, in an `I128`. A
            // literal that is not negative is a `U64` already.
            let as_u64 = |e: &Expr| matches!(e.kind, ExprKind::Int(v) if v >= 0) || e.ty == Ty::U64;
            let (cast_int, cast_u64, ty) = if as_u64(&l) && as_u64(&r) {
                (Some(Cast::IntToU64), None, Ty::U64)
            } else {
                (Some(Cast::IntToI128), Some(Cast::U64ToI128), Ty::I128)
            };
            let bring = |lw: &Self, e: Expr| match if e.ty == Ty::Int { cast_int } else { cast_u64 }
            {
                Some(cast) => lw.cast(cast, e, ty.clone()),
                None => e,
            };
            (bring(self, l), bring(self, r))
        } else {
            let want = match (&l.ty, &r.ty) {
                (a, b) if a.list_meet(b).is_some() => a.list_meet(b).expect("checked"),
                (Ty::Float, Ty::Int) | (Ty::Int, Ty::Float) => Ty::Float,
                (Ty::Int, wide @ (Ty::I128 | Ty::U64)) => wide.clone(),
                (Ty::Opt(_), _) => l.ty.clone(),
                (_, Ty::Opt(_)) => r.ty.clone(),
                _ => l.ty.clone(),
            };
            let l = match self.try_coerce(l, &want) {
                Ok(l) => l,
                Err(l) => {
                    self.compare_mismatch(span, &l, &r);
                    return self.err_expr(span);
                }
            };
            let r = match self.try_coerce(r, &want) {
                Ok(r) => r,
                Err(r) => {
                    self.compare_mismatch(span, &l, &r);
                    return self.err_expr(span);
                }
            };
            (l, r)
        };
        let _ = (l_span, r_span);
        if !matches!(op, CmpOp::Eq | CmpOp::Ne) {
            if matches!(l.ty, Ty::Opt(_)) {
                self.error(
                    codes::OPTIONAL_COMPARE,
                    span,
                    "an optional has no order: `none` is neither small nor large",
                );
                self.help("give it a value first: `(x ?? INF) < y`");
                return self.err_expr(span);
            }
            if !self.ordered(&l.ty) {
                let message = format!("a `{}` has no order to compare by", self.show(&l.ty));
                self.error(codes::BAD_OPERANDS, span, message);
                return self.err_expr(span);
            }
        }
        self.mk_cmp(op, l, r, Ty::Bool, span)
    }
}

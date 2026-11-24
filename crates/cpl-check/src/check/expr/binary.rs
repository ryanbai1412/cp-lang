//! Binary operators: arithmetic, comparison chains, ranges and coalescing.

use super::*;
use cpl_diag::msgs;

impl<'a> Checker<'a> {
    /// What `-x` is on a struct with an `impl Neg`: its `neg`'s result,
    /// or the struct when that is written bare.
    pub(super) fn neg_impl(&mut self, ty: &Ty) -> Option<Ty> {
        let Ty::Named { name, .. } = ty else {
            return None;
        };
        let impls = self.op_impls(ty, name, "Neg", "neg");
        let (_, out, ret) = impls.first()?;
        Some(out.as_ref().or(ret.as_ref()).unwrap_or(ty).clone())
    }

    /// The impls of operator trait `trait_name` for `left`'s struct `owner`,
    /// each as its right operand's type, its `Out` and its `method`'s result,
    /// with the impl's type parameters read from `left`: `impl Mul for Mat<T>`
    /// at a `Mat<Int>` has `T` = `Int`.
    fn op_impls(
        &mut self,
        left: &Ty,
        owner: &str,
        trait_name: &str,
        method: &str,
    ) -> Vec<(Option<Ty>, Option<Ty>, Option<Ty>)> {
        (self.env.impls(owner).iter())
            .filter(|(implemented, _)| {
                implemented.as_deref().and_then(|t| t.split('<').next()) == Some(trait_name)
            })
            .map(|(_, def)| match &**def {
                Def::Impl {
                    trait_args,
                    out_ty,
                    methods,
                    target,
                    ..
                } => {
                    let mut on = crate::ty::Bindings::default();
                    target.unify(left, &mut on);
                    let rhs_ty = trait_args.first().map(|t| t.subst(&on));
                    let ret = methods.iter().find_map(|(m, ty)| match ty {
                        Ty::Fn { ret, .. }
                            if m == method && !matches!(**ret, Ty::Unknown | Ty::Unit) =>
                        {
                            Some(ret.subst(&on))
                        }
                        _ => None,
                    });
                    (rhs_ty, out_ty.as_ref().map(|t| t.subst(&on)), ret)
                }
                _ => (None, None, None),
            })
            .collect()
    }

    pub(super) fn bad_operand(&mut self, ty: &Ty, expr: &Expr, op: &str) -> Ty {
        if ty.is_vague() {
            return ty.clone();
        }
        if matches!(ty, Ty::Bool | Ty::Str) {
            self.report(
                expr.span,
                msgs::check::BAD_OPERANDS_NEEDS_A_NUMBER_BUT,
                &[&op, &self.quote(expr.span), &ty.with_article()],
            );
            return Ty::Error;
        }
        Ty::Unknown
    }

    pub(super) fn binary(&mut self, op: BinOp, op_span: Span, lhs: &Expr, rhs: &Expr) -> Ty {
        let left = if op == BinOp::Coalesce {
            self.infer_optional(lhs)
        } else {
            self.infer(lhs)
        };
        if let Ty::Param(param) = &left {
            let trait_name = op_trait(op).map(|(t, _)| t);
            // `P - 1` reads the value of a value parameter (`Mod<P>`), and a step from a
            // `D: Domain` position is an `Int` (R04.8.2): neither needs an operator bound.
            let value = matches!(&lhs.kind, ExprKind::Name(n) if n == param);
            if trait_name.is_some()
                && (value
                    || matches!(op, BinOp::Add | BinOp::Sub)
                        && self.param_has_bound(param, "Domain"))
            {
                self.infer(rhs);
                return Ty::Int;
            }
            if let Some(trait_name) = trait_name {
                if !self.param_has_bound(param, trait_name) {
                    self.report(
                        op_span,
                        msgs::check::MISSING_IMPL_REQUIRES_WHICH_IS_NOT,
                        &[&op.text(), &param, &trait_name],
                    );
                    self.infer(rhs);
                    return Ty::Error;
                }
            }
        }

        let past = self.present.len();
        let held = narrow::in_rhs(op, lhs, rhs, &mut |f, x| self.call_writes(f, x));
        self.present.extend(held.into_iter().map(String::from));
        // The right of `and`, `or` and `??` may not run: a `?` there is in a branch.
        let skipped = u32::from(matches!(op, BinOp::And | BinOp::Or | BinOp::Coalesce));
        self.branch += skipped;
        let right = self.infer(rhs);
        self.branch -= skipped;
        self.present.truncate(past);
        match op {
            BinOp::And | BinOp::Or => {
                self.require_bool(&left, lhs, &format!("`{}`", op.text()));
                self.require_bool(&right, rhs, &format!("`{}`", op.text()));
                Ty::Bool
            }
            BinOp::Coalesce => {
                // `a[i] + best ?? 0` reads two ways. A step along a domain
                // (`i + 1 ?? I.last`) and a way out (`x + 1 ?? continue`)
                // read one way.
                if let ExprKind::Binary {
                    op: inner,
                    lhs: first,
                    ..
                } = &lhs.kind
                {
                    let arithmetic = matches!(
                        inner,
                        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem
                    );
                    let step = matches!(self.infer(first), Ty::Pos(_)) || left.is_vague();
                    if arithmetic && !step && !matches!(rhs.kind, ExprKind::Block(_)) {
                        if let Some(diag) = self.report(lhs.span, msgs::syntax::COALESCE_MIXED, &[])
                        {
                            diag.help = Some(
                                "write `(a ?? 0) + b` for a default on `a` alone, or `(a + b) ?? 0` for a default on the sum".into(),
                            );
                        }
                    }
                }
                self.coalesce_ty(left, right, rhs)
            }
            // `total + ` mid-edit: the half that is there gives the whole.
            _ if matches!(rhs.kind, ExprKind::Missing) && left.is_numeric() => left.widen(),
            _ if let Some((trait_name, owner)) = self.operator_trait(op, &left) => {
                let op_method = op_trait(op).map_or("", |(_, m)| m);
                let matching = self.op_impls(&left, &owner, trait_name, op_method);
                if matching.is_empty() {
                    if let Some(diag) = self.report(
                        op_span,
                        msgs::check::MISSING_IMPL_NEEDS_IMPL_FOR_AND,
                        &[&op.text(), &trait_name, &owner],
                    ) {
                        diag.help = Some(format!(
                            "an operator on a struct is the trait's fn: write `impl {trait_name} for {owner}`"
                        ));
                    }
                    return Ty::Error;
                }
                let pick = matching.iter().find(|(rhs_ty, _, _)| match rhs_ty {
                    Some(want) => right.fits(want),
                    None if op == BinOp::Pow => right.is_int() || right.is_vague(),
                    None => right.fits(&left) || right.is_int() || right.is_vague(),
                });
                match pick {
                    Some((_, out_ty, method_ret)) => out_ty
                        .clone()
                        .or_else(|| method_ret.clone())
                        .unwrap_or(left),
                    None if right == Ty::Error || right.is_vague() => left,
                    None if matches!((&left, &right), (Ty::Named { name: a, .. }, Ty::Named { name: b, .. }) if a == b) =>
                    {
                        self.report(
                            op_span,
                            msgs::check::TYPE_MISMATCH_MIXES_A_AND_A,
                            &[&op.text(), &left, &right],
                        );
                        Ty::Error
                    }
                    None => {
                        let wants: Vec<Ty> = matching
                            .iter()
                            .map(|m| m.0.clone().unwrap_or(left.clone()))
                            .collect();
                        self.no_operator_impl((op, op_span), rhs, &right, &wants, trait_name, owner)
                    }
                }
            }
            _ if left == Ty::Error || right == Ty::Error => Ty::Error,
            // At a type parameter the bounds allow it, `a + b` is the parameter again.
            BinOp::Add
            | BinOp::Sub
            | BinOp::Mul
            | BinOp::Div
            | BinOp::Rem
            | BinOp::BitAnd
            | BinOp::BitOr
            | BinOp::BitXor
            | BinOp::Pow
            | BinOp::Shl
            | BinOp::Shr
                if matches!(left, Ty::Param(_))
                    && (right == left
                        || matches!(op, BinOp::Pow | BinOp::Shl | BinOp::Shr)
                            && right.is_int()) =>
            {
                left
            }
            // `x % P` reads the value of a value parameter (`Mod<P>`).
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem
                if left.is_int()
                    && matches!((&right, &rhs.kind), (Ty::Param(p), ExprKind::Name(n)) if p == n) =>
            {
                Ty::Int
            }
            _ if left.is_vague() || right.is_vague() => Ty::Unknown,
            // `none` absorbs arithmetic (`+ - * /`) on `?Int`/`?F64` and integer/bitwise ops (`% ** & | ^ << >>`) on `?Int`.
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div
                if tropical_side(&left).is_some()
                    && tropical_side(&right).is_some()
                    && (matches!(left, Ty::Optional(_)) || matches!(right, Ty::Optional(_))) =>
            {
                let int = tropical_side(&left) == Some(true) && tropical_side(&right) == Some(true);
                Ty::Optional(Box::new(if int { Ty::Int } else { Ty::Float }))
            }
            BinOp::Rem
            | BinOp::Pow
            | BinOp::BitAnd
            | BinOp::BitOr
            | BinOp::BitXor
            | BinOp::Shl
            | BinOp::Shr
                if tropical_side(&left) == Some(true)
                    && tropical_side(&right) == Some(true)
                    && (matches!(left, Ty::Optional(_)) || matches!(right, Ty::Optional(_))) =>
            {
                Ty::Optional(Box::new(Ty::Int))
            }
            // A shift keeps the type of what is shifted; otherwise an `Int`
            // beside an `I128` or a `U64` becomes one.
            BinOp::Shl | BinOp::Shr if left.is_int() && right.is_int() => left.widen(),
            _ if matches!((&left, &right), (Ty::I128, Ty::U64) | (Ty::U64, Ty::I128)) => {
                if let Some(diag) = self.report(
                    op_span,
                    msgs::check::BAD_OPERANDS_CAN_T_COMBINE_AN,
                    &[&op.text()],
                ) {
                    diag.help = Some("say which one the result is: `I128(x)` or `U64(x)`".into());
                }
                Ty::Error
            }
            _ if left.is_int() && right.is_int() => match (&left, &right) {
                (wide @ (Ty::I128 | Ty::U64), _) | (_, wide @ (Ty::I128 | Ty::U64)) => wide.clone(),
                _ => Ty::Int,
            },
            _ if left.is_numeric() && right.is_numeric() => Ty::Float,
            BinOp::Add if left == Ty::Str && right == Ty::Str => Ty::Str,
            // `xs + view`: a struct view joins as the list its walk yields (R04a.2.3).
            BinOp::Add
                if let (Some(le), Ty::Named { .. }) = (seq_concat_elem(&left), &right)
                    && right.seq_elem().is_none()
                    && self.walked_elem(&right).as_ref() == Some(le) =>
            {
                Ty::list(le.clone())
            }
            BinOp::Add
                if let (Some(le), Some(re)) = (seq_concat_elem(&left), seq_concat_elem(&right)) =>
            {
                if le == re || re.is_vague() {
                    Ty::list(le.clone())
                } else if le.is_vague() {
                    Ty::list(re.clone())
                } else {
                    Ty::Unknown
                }
            }
            // A step can leave the domain, so `i + k` and `i - k` are a
            // `?I`: `none` past either end (in v0.3, `Int` per R04.8.2).
            BinOp::Add | BinOp::Sub
                if right.is_int() && matches!(&left, Ty::Pos(d) if !d.contains('.')) =>
            {
                Ty::Int
            }
            // `i - j` is how far apart two positions of one domain are.
            BinOp::Sub if matches!((&left, &right), (Ty::Pos(a), Ty::Pos(b)) if a == b) => Ty::Int,
            _ => {
                // Only the clear mistakes are reported: arithmetic on a Bool,
                // or a Str mixed with a number. Positions, chars and library
                // types have their own operators, which `std` will define.
                let primitive = |t: &Ty| t.is_numeric() || matches!(t, Ty::Bool | Ty::Str);
                let clear = primitive(&left) && primitive(&right);
                if clear {
                    let into_string = match (op, &left, &right) {
                        (BinOp::Add, ..) => self.string_fix(lhs, rhs),
                        (BinOp::Mul, Ty::Str, r) if r.is_int() => self.repeat_fix(lhs, rhs),
                        (BinOp::Mul, l, Ty::Str) if l.is_int() => self.repeat_fix(rhs, lhs),
                        _ => None,
                    };
                    if let Some(diag) = self.report(
                        op_span,
                        msgs::check::BAD_OPERANDS_CAN_T_COMBINE_AND,
                        &[
                            &op.text(),
                            &left.widen().with_article(),
                            &right.widen().with_article(),
                        ],
                    ) {
                        advice::bad_operands(diag, (op, op_span), (&left, &right), into_string);
                    }
                    return Ty::Error;
                }
                Ty::Unknown
            }
        }
    }

    /// `lo..` runs to `Int.MAX` or the end of `lo`'s domain, and `..hi`
    /// starts at 0 or the domain's first value (R01.8.4).
    fn open_range_end(end: &Ty) -> bool {
        match end {
            Ty::Named { name, .. } => name == "Cut",
            _ => matches!(
                end,
                Ty::Int
                    | Ty::Range { .. }
                    | Ty::Pos(_)
                    | Ty::Param(_)
                    | Ty::Assoc { .. }
                    | Ty::Unknown
                    | Ty::Error
            ),
        }
    }

    /// A range runs between two ends of one domain, and `..=` ends at a
    /// position, never at a boundary.
    pub(super) fn range_ends(
        &mut self,
        range: &Expr,
        hi: Option<&Expr>,
        inclusive: bool,
        lo_ty: &Option<Ty>,
        hi_ty: &Option<Ty>,
    ) {
        match (lo_ty, hi_ty) {
            (None, None) if !self.full_ranges.contains(&range.span) => {
                if let Some(diag) =
                    self.report(range.span, msgs::check::TYPE_MISMATCH_FULL_RANGE, &[])
                {
                    diag.help = Some("write the ends it runs between, as in `0..n`".into());
                }
                return;
            }
            (Some(end), None) | (None, Some(end)) if !Self::open_range_end(end) => {
                if let Some(diag) = self.report(
                    range.span,
                    msgs::check::TYPE_MISMATCH_OPEN_RANGE_END,
                    &[end],
                ) {
                    diag.help = Some("write both ends of the range".into());
                }
                return;
            }
            _ => {}
        }
        let domain = |ty: &Option<Ty>| match ty {
            Some(Ty::Pos(d)) => Some(d.strip_suffix(".cuts").unwrap_or(d).to_string()),
            Some(Ty::Named { name, args }) if name == "Cut" => match args.as_slice() {
                [Ty::Pos(d)] => Some(d.clone()),
                _ => None,
            },
            _ => None,
        };
        if let (Some(a), Some(b)) = (domain(lo_ty), domain(hi_ty)) {
            if a != b {
                if let Some(diag) = self.report(
                    range.span,
                    msgs::check::WRONG_DOMAIN_RANGE_BETWEEN_AND,
                    &[&a, &b],
                ) {
                    diag.help = Some("both ends of a range are in one domain".into());
                }
                return;
            }
        }
        let Some(hi) = hi else {
            return;
        };
        let hi_is_bounds = match hi_ty {
            Some(Ty::Pos(d)) => d.ends_with(".cuts"),
            Some(Ty::Named { name, .. }) if name == "Cut" => true,
            _ => false,
        };
        let spec = true;
        if !spec && inclusive && hi_is_bounds && !self.expected_bounds_range {
            let message =
                cpl_diag::msgs::check::WRONG_DOMAIN_ENDS_AT_A_POSITION.at(&[&self.quote(hi.span)]);
            // R04.4.6: where a piece is wanted, `j..=i` on cuts is the wrong type.
            let code = if spec {
                codes::TYPE_MISMATCH
            } else {
                codes::WRONG_DOMAIN
            };
            self.report_as(code, range.span, message);
        }
    }

    /// Reports a comparison between positions of two different domains.
    /// The trait an arithmetic operator asks of a struct on its left, and
    /// the struct's name: `a + b` with `a: Pt` is `Add` for `Pt`.
    pub(super) fn operator_trait(
        &mut self,
        op: BinOp,
        left: &Ty,
    ) -> Option<(&'static str, String)> {
        let (trait_name, _) = op_trait(op)?;
        let Ty::Named { name, .. } = left else {
            return None;
        };
        // A view such as `Rev<S>` is a sequence, so `a.rev + b` concatenates.
        (left.seq_elem().is_none()
            && matches!(self.type_def(name).as_deref(), Some(Def::Struct { .. })))
        .then(|| (trait_name, name.clone()))
    }

    pub(super) fn comparable(
        &mut self,
        op: ast::CmpOp,
        tests_none: bool,
        left: &Ty,
        left_expr: &Expr,
        right: &Ty,
        right_expr: &Expr,
    ) {
        if !tests_none
            && matches!(
                op,
                ast::CmpOp::Lt | ast::CmpOp::Le | ast::CmpOp::Gt | ast::CmpOp::Ge
            )
            && (matches!(left, Ty::Optional(_)) || matches!(right, Ty::Optional(_)))
        {
            let op_str = match op {
                ast::CmpOp::Lt => "<",
                ast::CmpOp::Le => "<=",
                ast::CmpOp::Gt => ">",
                ast::CmpOp::Ge => ">=",
                _ => "?",
            };
            self.optional_compare(op_str, (left_expr, left), (right_expr, right));
            return;
        }
        if !tests_none
            && !left.is_vague()
            && !right.is_vague()
            && !matches!(left, Ty::Param(_) | Ty::TypeName(_))
        {
            let req = match op {
                ast::CmpOp::Eq | ast::CmpOp::Ne => Some("Eq"),
                ast::CmpOp::Lt | ast::CmpOp::Le | ast::CmpOp::Gt | ast::CmpOp::Ge => Some("Ord"),
                _ => None,
            };
            if let Some(trait_name) = req {
                if !self.implements_trait(left, trait_name) {
                    self.report(
                        left_expr.span,
                        msgs::check::MISSING_IMPL_DOES_NOT_IMPLEMENT,
                        &[&left, &trait_name],
                    );
                    return;
                }
            }
        }
        let (Ty::Pos(a), Ty::Pos(b)) = (left, right) else {
            return;
        };
        if a == b {
            return;
        }
        self.report(
            left_expr.span.to(right_expr.span),
            msgs::check::WRONG_DOMAIN_IS_AND_IS_POSITIONS,
            &[
                &self.quote(left_expr.span),
                &left.with_article(),
                &self.quote(right_expr.span),
                &right.with_article(),
            ],
        );
    }
}

/// Whether `ty` can stand on one side of a `none`-absorbing `+` or `-`: a
/// number or an optional number. `Some(true)` for the integers.
/// The trait an arithmetic operator is the fn of, and that fn's name:
/// `a * b` is `Mul`'s `mul`.
fn op_trait(op: BinOp) -> Option<(&'static str, &'static str)> {
    Some(match op {
        BinOp::Add => ("Add", "add"),
        BinOp::Sub => ("Sub", "sub"),
        BinOp::Mul => ("Mul", "mul"),
        BinOp::Div => ("Div", "div"),
        BinOp::Rem => ("Rem", "rem"),
        BinOp::Pow => ("Pow", "pow"),
        _ => return None,
    })
}

pub(super) fn tropical_side(ty: &Ty) -> Option<bool> {
    let inner = match ty {
        Ty::Optional(inner) => inner,
        other => other,
    };
    inner.is_numeric().then(|| inner.is_int())
}

impl Checker<'_> {
    /// `none` does not pass through `+=` and `-=` (v2.2): a `none` that
    /// stays `none` hides a table that was never filled in.
    pub(crate) fn optional_fold(
        &mut self,
        target: &Expr,
        target_ty: &Ty,
        op: cpl_syntax::ast::AssignOp,
    ) {
        let cpl_syntax::ast::AssignOp::Binary(bin @ (BinOp::Add | BinOp::Sub)) = op else {
            return;
        };
        if !matches!(target_ty, Ty::Optional(inner) if inner.is_numeric()) {
            return;
        }
        // `a.last += 1`: `last` is no place, which lowering reports and fixes.
        if let ExprKind::Field { base, name } = &target.kind {
            let base_ty = self.infer_quietly(base);
            let listed = matches!(&base_ty, Ty::Array { .. })
                || matches!(&base_ty, Ty::Named { name, .. } if name == "List");
            if name.text == "last" && listed {
                return;
            }
        }
        let sign = if bin == BinOp::Add { "+" } else { "-" };
        if let Some(diag) = self.report(
            target.span,
            msgs::check::BAD_OPERANDS_CAN_T_BE_APPLIED,
            &[&sign, &target_ty.with_article()],
        ) {
            diag.help = Some(format!(
                "write `x = x {sign} y` when `None` should stay `None`, or `x min= y + w` for a transition"
            ));
        }
    }

    pub(super) fn coalesce_ty(&mut self, left: Ty, right: Ty, rhs: &Expr) -> Ty {
        // `o ?? return`: the right side never gives a value.
        let diverges = matches!(&rhs.kind, ExprKind::Block(block) if matches!(
            block.stmts.as_slice(),
            [cpl_syntax::ast::Stmt {
                kind: cpl_syntax::ast::StmtKind::Return(_)
                    | cpl_syntax::ast::StmtKind::Break(_)
                    | cpl_syntax::ast::StmtKind::Continue(_),
                ..
            }]
        ));
        match left {
            Ty::Optional(inner) if !inner.is_vague() || diverges => *inner,
            Ty::Error if diverges => Ty::Error,
            Ty::Error => right,
            _ if diverges => Ty::Unknown,
            _ if right.is_vague() => Ty::Unknown,
            _ => right.widen(),
        }
    }
}

pub(super) fn seq_concat_elem(ty: &Ty) -> Option<&Ty> {
    match ty.rev_source().unwrap_or(ty) {
        Ty::Named { name, args }
            if matches!(name.as_str(), "List" | "ListView") && args.len() == 1 =>
        {
            Some(&args[0])
        }
        Ty::Array { index, elem } if index.len() == 1 => Some(elem),
        _ => None,
    }
}

/// `a < b > c` as `a < b and b > c`.
pub(super) fn split_chain(src: &str, first: &Expr, rest: &[(ast::CmpOp, Expr)]) -> Option<String> {
    let mut parts = Vec::with_capacity(rest.len());
    let mut left = first;
    for (_, right) in rest {
        parts.push(src.get(left.span.start as usize..right.span.end as usize)?);
        left = right;
    }
    Some(parts.join(" and "))
}

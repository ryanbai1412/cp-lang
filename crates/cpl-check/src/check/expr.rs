//! Expression types.

use super::*;
use cpl_syntax::ast::{self, BinOp, Cond, Expr, ExprKind, StrPiece, UnOp};

fn range_of(elem: Ty) -> Ty {
    Ty::Named {
        name: "Range".into(),
        args: vec![elem],
    }
}

impl<'a> Checker<'a> {
    pub(crate) fn infer(&mut self, expr: &Expr) -> Ty {
        // An outermost place expression (`a[i]`, not the `a` inside it)
        // uses its place: a conflict with a live lend is reported once
        // its parts are known.
        let uses_place = self.enter_place(expr);
        let ty = self.infer_kind(expr);
        if uses_place {
            self.check_access(expr, false);
        }
        if matches!(
            expr.kind,
            ExprKind::Call { .. }
                | ExprKind::Index { .. }
                | ExprKind::Paren(_)
                | ExprKind::Range { .. }
                | ExprKind::Unwrap(_)
                | ExprKind::Try(_)
        ) {
            self.shapes.push((expr.span, ty.clone()));
        }
        ty
    }

    fn infer_kind(&mut self, expr: &Expr) -> Ty {
        match &expr.kind {
            ExprKind::Int(_) => Ty::Int,
            ExprKind::Float(_) => Ty::Float,
            ExprKind::Char(_) => Ty::Char,
            ExprKind::Bool(_) => Ty::Bool,
            ExprKind::None if self.local("none").is_some() => self.name_ty("none", expr.span),
            ExprKind::None => Ty::Unknown,
            ExprKind::Str(pieces) => {
                for piece in pieces {
                    if let StrPiece::Expr(inner) | StrPiece::Spec(inner, _) = piece {
                        self.infer(inner);
                    }
                }
                Ty::Str
            }
            ExprKind::Name(name) => self.name_ty(name, expr.span),
            ExprKind::SelfValue => self.name_ty("self", expr.span),
            ExprKind::Paren(inner) => self.infer(inner),
            ExprKind::Unary { op, expr: inner } => {
                let ty = self.infer(inner);
                match op {
                    UnOp::Not => {
                        self.require_bool(&ty, inner, "`not`");
                        Ty::Bool
                    }
                    UnOp::Pos if ty.is_numeric() => ty.widen(),
                    UnOp::Pos => {
                        if ty.is_vague() {
                            return ty;
                        }
                        let message = format!(
                            "unary `+` needs a number, but `{}` is {}",
                            self.quote(inner.span),
                            ty.with_article()
                        );
                        self.error(codes::TYPE_MISMATCH, inner.span, message);
                        Ty::Error
                    }
                    UnOp::Neg | UnOp::BitNot if ty.is_numeric() => ty.widen(),
                    UnOp::Neg if matches!(&ty, Ty::Optional(inner) if inner.is_numeric()) => ty,
                    UnOp::BitNot if matches!(&ty, Ty::Optional(inner) if inner.is_int()) => ty,
                    UnOp::Neg | UnOp::BitNot => {
                        self.bad_operand(&ty, inner, if *op == UnOp::Neg { "-" } else { "~" })
                    }
                }
            }
            ExprKind::Binary {
                op,
                op_span,
                lhs,
                rhs,
            } => {
                // `(l + 1)!` and `l + 1 ?? r` ask for the optional.
                let written = std::mem::take(&mut self.as_written);
                let ty = self.binary(*op, *op_span, lhs, rhs);
                match ty {
                    Ty::Optional(inner) if !written && self.step_stays(expr, &inner) => *inner,
                    ty => ty,
                }
            }
            ExprKind::Compare { first, rest } => {
                // `f<A>(x)` is two comparisons: a fn's type arguments are
                // written `f::<A>(x)`.
                if let ExprKind::Name(name) = &first.kind {
                    let generic = self.local(name).is_none()
                        && matches!(
                            self.global(name).as_deref(),
                            Some(Def::Fn { generics, .. }) if !generics.is_empty()
                        );
                    if generic {
                        let message = format!(
                            "`{name}` is a generic fn and is only called, as `{name}(...)`: its type parameters come from its arguments"
                        );
                        if let Some(diag) = self.error(codes::BAD_OPERANDS, expr.span, message) {
                            diag.help = Some(format!(
                                "to name one that no argument gives, write `{name}<T>(...)`"
                            ));
                        }
                        return Ty::Error;
                    }
                }
                let tests_none = narrow::tests_none(first, rest);
                let side = |this: &mut Self, e: &Expr| {
                    if tests_none {
                        this.infer_optional(e)
                    } else {
                        this.infer(e)
                    }
                };
                let mut left = side(self, first);
                let mut left_expr: &Expr = first;
                let mut any_vague = left.is_vague();
                for (op, right_expr) in rest {
                    let right = side(self, right_expr);
                    any_vague |= right.is_vague();
                    if matches!(op, ast::CmpOp::In | ast::CmpOp::NotIn) {
                        if matches!(&right, Ty::Named { name, .. } if name == "List") {
                            self.list_in(*op, left_expr, right_expr, &right, rest.len() == 1);
                        }
                    } else {
                        self.comparable(*op, tests_none, &left, left_expr, &right, right_expr);
                    }
                    left = right;
                    left_expr = right_expr;
                }
                // §130: chained comparisons must point in one direction and cannot contain `==` or `!=`.
                let is_module_generic_call = matches!(
                    &first.kind,
                    ExprKind::Field { base, .. }
                        if matches!(&base.kind, ExprKind::Name(m) if self.env.is_module(m))
                ) && rest.len() == 2
                    && rest[0].0 == ast::CmpOp::Lt
                    && rest[1].0 == ast::CmpOp::Gt;
                if rest.len() >= 2 && !any_vague && !is_module_generic_call {
                    let has_lt = rest
                        .iter()
                        .any(|(op, _)| matches!(op, ast::CmpOp::Lt | ast::CmpOp::Le));
                    let has_gt = rest
                        .iter()
                        .any(|(op, _)| matches!(op, ast::CmpOp::Gt | ast::CmpOp::Ge));
                    let has_eq = rest
                        .iter()
                        .any(|(op, _)| matches!(op, ast::CmpOp::Eq | ast::CmpOp::Ne));
                    let split = split_chain(self.src, first, rest).map(|text| cpl_diag::Fix {
                        title: format!("write `{text}`"),
                        edits: vec![(expr.span, text)],
                    });
                    if has_eq {
                        if let Some(diag) = self.error(
                            codes::MIXED_COMPARISON_CHAIN,
                            expr.span,
                            "chained comparisons cannot contain `==` or `!=`".to_string(),
                        ) {
                            diag.help = Some("join the comparisons with `and` instead".to_string());
                            diag.fixes.extend(split);
                        }
                    } else if has_lt && has_gt {
                        if let Some(diag) = self.error(
                            codes::MIXED_COMPARISON_CHAIN,
                            expr.span,
                            "chained comparisons must point in one direction".to_string(),
                        ) {
                            diag.help =
                                Some("split mixed `<` and `>` comparisons with `and`".to_string());
                            diag.fixes.extend(split);
                        }
                    }
                }
                if rest.iter().any(|(op, _)| *op == ast::CmpOp::Order) {
                    return Ty::Unknown;
                }
                Ty::Bool
            }
            ExprKind::Range {
                lo,
                hi,
                inclusive,
                step,
            } => {
                let lo_ty = lo.as_ref().map(|e| self.infer(e));
                let hi_ty = hi.as_ref().map(|e| self.infer(e));
                if let Some(step) = step {
                    self.infer(step);
                }
                let lo_ty = lo_ty.map(|t| self.unalias_bounds(t));
                let hi_ty = hi_ty.map(|t| self.unalias_bounds(t));
                let bounds_domain = |ty: &Option<Ty>| match ty {
                    Some(Ty::Pos(d)) if d.ends_with(".cuts") => {
                        Some((d.clone(), d.strip_suffix(".cuts").unwrap().to_string()))
                    }
                    Some(Ty::Named { name, args }) if name == "Cut" => match args.as_slice() {
                        [Ty::Pos(d)] => Some((format!("{d}.cuts"), d.clone())),
                        _ => None,
                    },
                    _ => None,
                };
                let elem = match (bounds_domain(&lo_ty), bounds_domain(&hi_ty), &lo_ty, &hi_ty) {
                    // Two boundaries enclose positions (`l.l..r.r` is a range of `V`),
                    // except when the enclosing context explicitly expects `Range<V.cuts>`.
                    (Some((bd, base)), _, _, _) | (_, Some((bd, base)), _, _) => {
                        let spec = true;
                        let both_cuts =
                            bounds_domain(&lo_ty).is_some() && bounds_domain(&hi_ty).is_some();
                        if self.expected_bounds_range
                            || (spec && (*inclusive || !self.expected_piece))
                            || (self.iterating_range && both_cuts && !*inclusive)
                        {
                            Ty::Pos(bd)
                        } else {
                            Ty::Pos(base)
                        }
                    }
                    (_, _, Some(Ty::Pos(d)), _) | (_, _, _, Some(Ty::Pos(d))) => Ty::Pos(d.clone()),
                    (_, _, Some(a), Some(b))
                        if (*a == Ty::I128 && (*b == Ty::I128 || b.is_int()))
                            || (*b == Ty::I128 && a.is_int()) =>
                    {
                        Ty::I128
                    }
                    (_, _, Some(Ty::I128), None) | (_, _, None, Some(Ty::I128)) => Ty::I128,
                    (_, _, Some(a), Some(b)) if a.is_int() && b.is_int() => {
                        let lo_min = lo.as_ref().and_then(|e| self.bounds(e).0);
                        let hi_max = hi.as_ref().and_then(|e| self.bounds(e).1).map(|h| {
                            if *inclusive {
                                h
                            } else {
                                h - 1
                            }
                        });
                        Ty::Range {
                            lo: lo_min,
                            hi: hi_max,
                            text: self.text(expr.span).to_string(),
                        }
                    }
                    (_, _, Some(a), None) | (_, _, None, Some(a)) if a.is_int() => Ty::Int,
                    (_, _, Some(a), Some(Ty::Unknown)) | (_, _, Some(Ty::Unknown), Some(a))
                        if a.is_int() =>
                    {
                        Ty::Int
                    }
                    _ => Ty::Unknown,
                };
                self.range_ends(expr, hi.as_deref(), *inclusive, &lo_ty, &hi_ty);
                range_of(elem)
            }
            ExprKind::By { walk, step } => {
                let step_ty = self.infer(step);
                self.expect_fits(&step_ty, &Ty::Int, step, |c, got, _| {
                    format!(
                        "a step is an Int, and `{}` is {}",
                        c.quote(step.span),
                        got.with_article()
                    )
                });
                self.infer(walk)
            }
            ExprKind::Call { callee, args } => {
                let ty = self.call(expr, callee, args);
                self.member_for_call(expr, callee, args);
                ty
            }
            ExprKind::Index { base, indices } => self.index(expr, base, indices),
            ExprKind::Field { base, name } => self.field(expr, base, name),
            ExprKind::Generic { base, args } => {
                let base_ty = self.infer(base);
                let args: Vec<Ty> = args.iter().map(|a| self.type_arg(a)).collect();
                // `Prefix<T>`, `Prefix<R, C, T>`, `Diff<R, C, T>`: shape-dispatched
                // struct families resolve to their internal specialization.
                if let ExprKind::Name(name) = &base.kind {
                    if self.local(name).is_none() {
                        if let Some(shape) = prefix_shape_struct(name, args.len()) {
                            if self.global(shape).is_some() {
                                return Ty::Named {
                                    name: shape.to_string(),
                                    args,
                                };
                            }
                        }
                    }
                }
                // `Sum<Int>`, `f::<Int>`: the declared type with its
                // parameters filled in.
                let declared = match &base.kind {
                    ExprKind::Name(name) if self.local(name).is_none() => {
                        match self.global(name).as_deref() {
                            Some(
                                Def::Value { ty, generics, .. } | Def::Fn { ty, generics, .. },
                            ) if !generics.is_empty() => Some((ty.clone(), generics.clone())),
                            _ => None,
                        }
                    }
                    _ => None,
                };
                match (base_ty, declared) {
                    // `scc::<V>(..)`: parameters after the ones given are
                    // read off the arguments.
                    (_, Some((ty, generics))) => {
                        if args.len() > generics.len() {
                            let message = format!(
                                "`{}` takes {}, but {} given",
                                self.quote(base.span),
                                plural(generics.len(), "type argument", "type arguments"),
                                if args.len() == 1 {
                                    "1 was".to_string()
                                } else {
                                    format!("{} were", args.len())
                                }
                            );
                            self.error(codes::WRONG_ARG_COUNT, expr.span, message);
                            return Ty::Error;
                        }
                        let bindings = generics.into_iter().zip(args).collect();
                        ty.subst(&bindings)
                    }
                    (Ty::TypeName(name), None) => Ty::Named { name, args },
                    (other, None) => other,
                }
            }
            ExprKind::Tuple(items) if items.is_empty() => Ty::Unit,
            ExprKind::Tuple(items) => {
                let tys = items.iter().map(|i| self.infer(i)).collect();
                for item in items {
                    self.no_escape(item, "storing it in a list or tuple");
                }
                Ty::Tuple(tys)
            }
            ExprKind::Record(fields) => {
                let mut seen: Vec<&str> = Vec::new();
                let mut out = Vec::with_capacity(fields.len());
                for (name, value) in fields {
                    if seen.contains(&name.text.as_str()) {
                        let message = format!("duplicate field `{}` in record", name.text);
                        self.error(codes::DUPLICATE_DEFINITION, name.span, message);
                    } else {
                        seen.push(&name.text);
                    }
                    out.push((name.text.clone(), self.infer(value)));
                }
                Ty::Record(out)
            }
            ExprKind::Map(entries) => {
                let (mut key, mut value) = (Ty::Unknown, Ty::Unknown);
                for (i, (k, v)) in entries.iter().enumerate() {
                    let (k, v) = (self.infer(k), self.infer(v));
                    if i == 0 {
                        (key, value) = (k.widen(), v.widen());
                    }
                }
                Ty::Named {
                    name: "Map".into(),
                    args: vec![key, value],
                }
            }
            ExprKind::List(items) => {
                let mut elem = Ty::Unknown;
                for (i, item) in items.iter().enumerate() {
                    let ty = self.infer(item);
                    if i == 0 {
                        elem = ty.widen();
                    }
                    self.no_escape(item, "storing it in a list or tuple");
                }
                if items.is_empty() {
                    Ty::Unknown
                } else {
                    Ty::list(elem)
                }
            }
            ExprKind::Comprehension { body, clauses } => {
                self.push_scope();
                self.branch += 1;
                for clause in clauses {
                    let iter_ty = self.infer(&clause.iter);
                    let elem = self.element_ty(&iter_ty);
                    self.bind(&clause.pat, &elem, false, "for");
                    for cond in &clause.conds {
                        self.cond(cond);
                    }
                }
                let body_ty = self.infer(body).widen();
                self.branch -= 1;
                self.pop_scope();
                Ty::list(body_ty)
            }
            ExprKind::Table {
                domains,
                params,
                body,
            } => {
                let index: Vec<Ty> = domains.iter().map(|d| self.resolve_type(d)).collect();
                self.push_scope();
                for (i, param) in params.iter().enumerate() {
                    let ty = index.get(i).cloned().unwrap_or(Ty::Unknown);
                    self.bind(param, &ty, false, "param");
                }
                let elem = self.infer(body).widen();
                self.pop_scope();
                Ty::Array {
                    index: self.flat_index(index),
                    elem: Box::new(elem),
                }
            }
            ExprKind::TableFill { domain, value } => {
                let dom_ty = self.infer(domain);
                let idx_ty = match dom_ty {
                    Ty::Named { ref name, ref args } if name == "Range" && args.len() == 1 => {
                        args[0].clone()
                    }
                    Ty::Unknown | Ty::Error => Ty::Int,
                    other => other,
                };
                let elem = self.infer(value).widen();
                Ty::Array {
                    index: vec![idx_ty],
                    elem: Box::new(elem),
                }
            }
            ExprKind::Lambda {
                params,
                param_tys,
                param_mut,
                ret_ty,
                body,
            } => {
                let mut given = match self
                    .lambda_params
                    .iter()
                    .position(|(at, _)| *at == expr.span)
                {
                    Some(i) => self.lambda_params.swap_remove(i).1,
                    None => Vec::new(),
                };
                if let [Ty::Tuple(items)] = given.as_slice() {
                    if params.len() > 1 && items.len() == params.len() {
                        given = items.clone();
                    }
                }
                self.push_scope();
                let mut sig_params = Vec::with_capacity(params.len());
                let any_typed = param_tys.iter().any(Option::is_some);
                for (i, param) in params.iter().enumerate() {
                    let ty = param_tys
                        .get(i)
                        .and_then(|t| t.as_ref())
                        .map(|t| self.resolve_type(t))
                        .unwrap_or_else(|| given.get(i).cloned().unwrap_or(Ty::Unknown));
                    let mutable = param_mut.get(i).copied().unwrap_or(false);
                    self.bind(param, &ty, mutable, "param");
                    sig_params.push(crate::ty::Param {
                        name: String::new(),
                        ty,
                        mutable,
                        has_default: false,
                        named_only: false,
                        variadic: false,
                        lazy: false,
                    });
                }
                // A lambda's `return` and trailing value belong to the lambda.
                let declared_ret = ret_ty.as_ref().map(|t| self.resolve_type(t));
                let lam_depth = self.scopes.len();
                self.fns.push((
                    String::new(),
                    declared_ret.clone().unwrap_or(Ty::Unknown),
                    false,
                    lam_depth,
                ));
                self.branch += 1;
                let ret = self.infer(body);
                self.branch -= 1;
                self.fns.pop();
                self.pop_scope();
                let final_ret = if let Some(dret) = declared_ret {
                    if !matches!(ret, Ty::Unknown | Ty::Error) && !self.fits(&ret, &dret) {
                        self.error(
                            codes::TYPE_MISMATCH,
                            body.span,
                            format!("expected `{dret}`, found `{ret}`"),
                        );
                    }
                    dret
                } else {
                    ret
                };
                if any_typed || ret_ty.is_some() {
                    Ty::Fn {
                        params: sig_params,
                        ret: Box::new(final_ret),
                    }
                } else {
                    Ty::Unknown
                }
            }
            ExprKind::Block(block) => self.block(block),
            ExprKind::If {
                branches,
                otherwise,
            } => {
                let mut result = None;
                // As a value, each branch is somewhere `e?` can't leave from.
                let value = u32::from(self.plain_if.take() != Some(expr.span));
                // Inside a branch its condition holds; after it, in the
                // branches that follow, it does not.
                let mark = self.lend_mark();
                let present = self.present.len();
                let saved_moved = self.moved.clone();
                let mut post_moved = saved_moved.clone();
                // `if const` with a test known here picks its branch now:
                // the others are not compiled, so they are not checked.
                let mut picked = false;
                for (at, (cond, body)) in branches.iter().enumerate() {
                    if picked {
                        break;
                    }
                    self.moved = saved_moved.clone();
                    self.push_scope();
                    self.cond(cond);
                    let known = match cond {
                        Cond::Const(test) => self.const_bool(test),
                        _ => None,
                    };
                    picked = known == Some(true);
                    if known == Some(false) {
                        self.pop_scope();
                        continue;
                    }
                    let branch = self.lend_mark();
                    self.add_cond_facts(cond, false);
                    let past = self.present.len();
                    let held = narrow::in_branch(cond, body, &mut |f, x| self.call_writes(f, x));
                    self.present.extend(held.into_iter().map(String::from));
                    self.branch += value;
                    let ty = self.block(body);
                    self.branch -= value;
                    let branch_leaves = matches!(
                        body.stmts.last().map(|s| &s.kind),
                        Some(
                            ast::StmtKind::Return(_)
                                | ast::StmtKind::Break(_)
                                | ast::StmtKind::Continue(_)
                        )
                    );
                    if !branch_leaves {
                        for (k, v) in self.moved.clone() {
                            if self.local(&k).is_some() || self.global(&k).is_some() {
                                post_moved.entry(k).or_insert(v);
                            }
                        }
                    }
                    self.present.truncate(past);
                    let failed = narrow::past(
                        cond,
                        &branches[at + 1..],
                        otherwise.as_ref(),
                        &mut |f, x| self.call_writes(f, x),
                    );
                    self.present.extend(failed.into_iter().map(String::from));
                    self.lend_reset(branch);
                    self.add_cond_facts(cond, true);
                    self.pop_scope();
                    result.get_or_insert(ty);
                }
                let leaves = |body: &ast::Block| {
                    matches!(
                        body.stmts.last().map(|s| &s.kind),
                        Some(
                            ast::StmtKind::Return(_)
                                | ast::StmtKind::Break(_)
                                | ast::StmtKind::Continue(_)
                        )
                    )
                };
                // `if i == j: return` leaves `i != j` for the rest of the block.
                let after = match (branches.as_slice(), otherwise) {
                    ([(cond, body)], None) if leaves(body) => Some(cond),
                    _ => None,
                };
                let result_ty = match otherwise {
                    Some(_) if picked => result.unwrap_or(Ty::Unit),
                    Some(body) => {
                        self.moved = saved_moved.clone();
                        self.branch += value;
                        let ty = self.block(body);
                        self.branch -= value;
                        if !leaves(body) {
                            for (k, v) in self.moved.clone() {
                                if self.local(&k).is_some() || self.global(&k).is_some() {
                                    post_moved.entry(k).or_insert(v);
                                }
                            }
                        }
                        match result {
                            Some(first) if first.is_vague() => ty,
                            Some(first) => first.widen(),
                            None => ty,
                        }
                    }
                    None => Ty::Unit,
                };
                self.moved = post_moved;
                self.present.truncate(present);
                self.lend_reset(mark);
                if let Some(cond) = after {
                    self.add_cond_facts(cond, true);
                }
                result_ty
            }
            ExprKind::Match { scrutinee, arms } => {
                let value = u32::from(self.plain_if.take() != Some(expr.span));
                let scrutinee_ty = self.infer_optional(scrutinee);
                let saved_moved = self.moved.clone();
                let mut post_moved = saved_moved.clone();
                let mut result = None;
                for arm in arms {
                    self.moved = saved_moved.clone();
                    self.capitalized_pattern(&arm.pat, &scrutinee_ty);
                    self.push_scope();
                    self.bind(&arm.pat, &scrutinee_ty, false, "let");
                    if let Some(guard) = &arm.guard {
                        let ty = self.infer(guard);
                        self.require_bool(&ty, guard, "the guard");
                    }
                    self.branch += value;
                    let ty = self.block(&arm.body);
                    self.branch -= value;
                    let arm_leaves = matches!(
                        arm.body.stmts.last().map(|s| &s.kind),
                        Some(
                            ast::StmtKind::Return(_)
                                | ast::StmtKind::Break(_)
                                | ast::StmtKind::Continue(_)
                        )
                    );
                    if !arm_leaves {
                        for (k, v) in self.moved.clone() {
                            if self.local(&k).is_some() || self.global(&k).is_some() {
                                post_moved.entry(k).or_insert(v);
                            }
                        }
                    }
                    self.pop_scope();
                    // The value is the first arm's that has one: an arm
                    // that leaves (`Skip => continue`) has none.
                    let leaves = arm.body.stmts.last().is_some_and(|last| {
                        matches!(
                            last.kind,
                            ast::StmtKind::Return(_)
                                | ast::StmtKind::Break(_)
                                | ast::StmtKind::Continue(_)
                        )
                    });
                    if result.is_none() && !leaves {
                        result = Some(ty.widen());
                    }
                }
                self.moved = post_moved;
                let result = result.unwrap_or(Ty::Unit);
                self.require_every_variant(&scrutinee_ty, arms, expr.span);
                self.require_every_lit_union(scrutinee, arms, expr.span);
                result
            }
            ExprKind::With { base, updates } => {
                let ty = self.infer(base);
                for (name, value) in updates {
                    let value_ty = self.infer(value);
                    if let Some(field_ty) = self.field_of(&ty, name) {
                        self.expect_fits(&value_ty, &field_ty, value, |c, got, want| {
                            format!(
                                "`{}` is {}, but `{}` is {}",
                                name.text,
                                want.with_article(),
                                c.quote(value.span),
                                got.with_article()
                            )
                        });
                    }
                }
                if let ExprKind::Name(base_name) = &base.kind {
                    let moves_heap = match &ty {
                        Ty::Record(fs) => fs.iter().any(|(fname, fty)| {
                            !updates.iter().any(|(u, _)| u.text == *fname) && self.is_heap_type(fty)
                        }),
                        _ => self.is_heap_type(&ty),
                    };
                    if moves_heap {
                        self.moved.insert(base_name.clone(), base.span);
                    }
                }
                ty
            }
            ExprKind::HasBound { .. } => Ty::Bool,
            ExprKind::Without {
                prefix,
                base,
                removed,
            } => {
                let mut out_fields: Vec<(String, Ty)> = prefix
                    .iter()
                    .map(|(n, e)| (n.text.clone(), self.infer(e)))
                    .collect();
                let base_ty = self.infer(base);
                let rem_names: Vec<String> = removed.iter().map(|n| n.text.clone()).collect();
                match base_ty {
                    Ty::Record(fs) => {
                        out_fields.extend(
                            fs.into_iter()
                                .filter(|(n, _)| !rem_names.iter().any(|r| r == n)),
                        );
                        Ty::Record(out_fields)
                    }
                    Ty::Param(p) => {
                        if let Some(req) = self.row_bounds.get(&p).cloned() {
                            for r in removed {
                                if !req.iter().any(|(fnm, _)| fnm == &r.text) {
                                    self.error(
                                        codes::UNKNOWN_FIELD,
                                        r.span,
                                        format!(
                                            "row `{p}` has no named field `{}` to remove",
                                            r.text
                                        ),
                                    );
                                }
                            }
                        }
                        Ty::RowWithout(out_fields, Box::new(Ty::Param(p)), rem_names)
                    }
                    other => {
                        if out_fields.is_empty() {
                            other
                        } else {
                            Ty::Record(out_fields)
                        }
                    }
                }
            }
            ExprKind::Try(inner) if self.branch > 0 => {
                let inner_text = self.quote(inner.span);
                let message = format!(
                    "`{inner_text}?` is inside a branch, a lambda or a comprehension, and can't leave the fn from there"
                );
                if let Some(diag) = self.error(codes::BAD_OPERANDS, expr.span, message) {
                    diag.help = Some(format!("bind it first: `let x = {inner_text}?`"));
                }
                self.infer(inner);
                Ty::Error
            }
            ExprKind::Keep { target, value, .. } => {
                let target_ty = self.infer(target);
                let value_ty = self.infer(value);
                self.row_keep(target, &target_ty, value, &value_ty);
                Ty::Bool
            }
            ExprKind::Mut(inner) => self.infer(inner),
            ExprKind::Try(inner) | ExprKind::Unwrap(inner) => match self.infer_optional(inner) {
                Ty::Optional(inner) => *inner,
                Ty::Error => Ty::Error,
                _ => Ty::Unknown,
            },
            ExprKind::Missing | ExprKind::Error => Ty::Error,
        }
    }

    /// A `match` on an enum whose variants hold nothing has an arm for
    /// each of them, or one that takes whatever is left.
    fn require_every_variant(&mut self, scrutinee: &Ty, arms: &[cpl_syntax::ast::Arm], span: Span) {
        use cpl_syntax::ast::{Pattern, PatternKind};
        let Ty::Named { name, .. } = scrutinee else {
            return;
        };
        let Some(Def::Enum { variants, .. }) = self.type_def(name).as_deref().cloned() else {
            return;
        };
        fn irrefutable(pat: &Pattern) -> bool {
            match &pat.kind {
                PatternKind::Wildcard
                | PatternKind::Name(_)
                | PatternKind::Domain(_)
                | PatternKind::Error => true,
                PatternKind::Tuple(items) => items.iter().all(irrefutable),
                _ => false,
            }
        }
        // `None`: the pattern takes every value.
        fn named(
            pat: &Pattern,
            variants: &[(String, Vec<Ty>)],
            out: &mut Vec<String>,
        ) -> Option<()> {
            match &pat.kind {
                PatternKind::Variant { path, args } if args.iter().all(irrefutable) => {
                    out.push(path.last()?.text.clone());
                    Some(())
                }
                PatternKind::Name(n) if variants.iter().any(|(v, _)| v == n) => {
                    out.push(n.clone());
                    Some(())
                }
                PatternKind::Or(pats) => pats.iter().try_for_each(|p| named(p, variants, out)),
                PatternKind::Var(inner) => named(inner, variants, out),
                PatternKind::Literal(_)
                | PatternKind::Tuple(_)
                | PatternKind::Record { .. }
                | PatternKind::Variant { .. } => Some(()),
                PatternKind::Wildcard
                | PatternKind::Name(_)
                | PatternKind::Domain(_)
                | PatternKind::Error => None,
            }
        }
        let mut seen = Vec::new();
        for arm in arms.iter().filter(|arm| arm.guard.is_none()) {
            if named(&arm.pat, &variants, &mut seen).is_none() {
                return;
            }
        }
        let missing: Vec<String> = variants
            .iter()
            .filter(|(v, _)| !seen.contains(v))
            .map(|(v, _)| format!("`{v}`"))
            .collect();
        if missing.is_empty() {
            return;
        }
        let message = format!("this `match` has no arm for {}", missing.join(", "));
        if let Some(diag) = self.error(codes::NON_EXHAUSTIVE_MATCH, span, message) {
            diag.help = Some("add an arm for each, or `_:` for whatever is left".to_string());
        }
    }

    /// R01.3.3: a capitalized pattern name is a variant or a constant, never
    /// a binding, so against a scalar with no such constant it is an error.
    fn capitalized_pattern(&mut self, pat: &cpl_syntax::ast::Pattern, ty: &Ty) {
        let name = match &pat.kind {
            cpl_syntax::ast::PatternKind::Name(name) => name,
            cpl_syntax::ast::PatternKind::Variant { path, args }
                if path.len() == 1 && args.is_empty() =>
            {
                &path[0].text
            }
            _ => return,
        };
        if !name.starts_with(|c: char| c.is_ascii_uppercase())
            || !(ty.is_numeric() || matches!(ty, Ty::Bool | Ty::Char | Ty::Str))
            || self.local(name).is_some()
            || self.global(name).is_some()
        {
            return;
        }
        let message = format!(
            "`{name}` is a variant, but the value matched is a `{}`",
            ty.widen()
        );
        if let Some(diag) = self.error(codes::TYPE_MISMATCH, pat.span, message) {
            diag.help = Some(
                "a capitalized name in a pattern is a variant or constant; bind with a lowercase name, or write `_`"
                    .to_string(),
            );
        }
    }

    fn require_every_lit_union(
        &mut self,
        scrutinee: &Expr,
        arms: &[cpl_syntax::ast::Arm],
        span: Span,
    ) {
        use cpl_syntax::ast::{Pattern, PatternKind};
        let ExprKind::Name(name) = &scrutinee.kind else {
            return;
        };
        let Some(lits) = self.lit_unions.get(name).cloned() else {
            return;
        };
        fn collect(src: &str, pat: &Pattern, out: &mut Vec<String>) -> Option<()> {
            match &pat.kind {
                PatternKind::Literal(lit) => {
                    let s = lit.span.text(src).trim().to_string();
                    out.push(s);
                    Some(())
                }
                PatternKind::Or(pats) => pats.iter().try_for_each(|p| collect(src, p, out)),
                PatternKind::Var(inner) => collect(src, inner, out),
                PatternKind::Wildcard | PatternKind::Name(_) | PatternKind::Error => None,
                _ => Some(()),
            }
        }
        let mut seen = Vec::new();
        for arm in arms.iter().filter(|arm| arm.guard.is_none()) {
            if collect(self.src, &arm.pat, &mut seen).is_none() {
                return;
            }
        }
        let missing: Vec<String> = lits
            .iter()
            .filter(|lit| !seen.contains(lit))
            .map(|lit| format!("`{lit}`"))
            .collect();
        if missing.is_empty() {
            return;
        }
        let message = format!("this `match` has no arm for {}", missing.join(", "));
        if let Some(diag) = self.error(codes::NON_EXHAUSTIVE_MATCH, span, message) {
            diag.help = Some("add an arm for each, or `_:` for whatever is left".to_string());
        }
    }

    /// The test of an `if const`, where it is known without choosing type
    /// arguments: `true`, a `const` that is, comparisons of whole numbers,
    /// and `not`, `and`, `or` of those. `None` leaves both branches to be
    /// checked.
    pub(crate) fn const_bool(&mut self, e: &Expr) -> Option<bool> {
        match &e.kind {
            ExprKind::Bool(b) => Some(*b),
            ExprKind::Paren(inner) => self.const_bool(inner),
            ExprKind::Name(name) if self.local(name).is_none() => {
                match self.global(name).as_deref() {
                    Some(Def::Value { settled, .. }) => *settled,
                    _ => None,
                }
            }
            ExprKind::Unary {
                op: UnOp::Not,
                expr,
            } => Some(!self.const_bool(expr)?),
            ExprKind::Binary {
                op: BinOp::And,
                lhs,
                rhs,
                ..
            } => Some(self.const_bool(lhs)? && self.const_bool(rhs)?),
            ExprKind::Binary {
                op: BinOp::Or,
                lhs,
                rhs,
                ..
            } => Some(self.const_bool(lhs)? || self.const_bool(rhs)?),
            ExprKind::Compare { first, rest } => {
                let int = |e: &Expr| match e.kind {
                    ExprKind::Int(n) => Some(n),
                    _ => None,
                };
                let mut left = int(first)?;
                for (op, next) in rest {
                    let right = int(next)?;
                    let holds = match op {
                        ast::CmpOp::Eq => left == right,
                        ast::CmpOp::Ne => left != right,
                        ast::CmpOp::Lt => left < right,
                        ast::CmpOp::Le => left <= right,
                        ast::CmpOp::Gt => left > right,
                        ast::CmpOp::Ge => left >= right,
                        _ => return None,
                    };
                    if !holds {
                        return Some(false);
                    }
                    left = right;
                }
                Some(true)
            }
            _ => None,
        }
    }

    /// A `?` in an `if` or `while` test: the test has no value to give
    /// when it is `None` (R02.4.7).
    fn question_in_condition(&mut self, expr: &Expr) -> bool {
        struct Find {
            bounds: Vec<Span>,
            found: Option<Span>,
        }
        impl cpl_syntax::visit::Visitor for Find {
            fn expr(&mut self, e: &Expr) {
                match &e.kind {
                    ExprKind::Lambda { .. } => self.bounds.push(e.span),
                    // R02.4.7: the left operand of `??` bounds a `?`.
                    ExprKind::Binary {
                        op: ast::BinOp::Coalesce,
                        lhs,
                        ..
                    } => self.bounds.push(lhs.span),
                    ExprKind::Try(_) if self.found.is_none() => {
                        let inside = self
                            .bounds
                            .iter()
                            .any(|l| l.start <= e.span.start && e.span.end <= l.end);
                        if !inside {
                            self.found = Some(e.span);
                        }
                    }
                    _ => {}
                }
            }
        }
        let mut find = Find {
            bounds: Vec::new(),
            found: None,
        };
        cpl_syntax::visit::walk_expr(expr, &mut find);
        let Some(at) = find.found else {
            return false;
        };
        let message = format!(
            "`{}` is a `?` in a condition: a `None` gives the test no value",
            self.quote(at)
        );
        let help = format!(
            "say what `None` means: `({}) ?? false`",
            self.quote(expr.span)
        );
        let replacement = format!("({}) ?? false", self.quote(expr.span));
        if let Some(diag) = self.error(codes::QUESTION_IN_CONDITION, at, message) {
            diag.help = Some(help);
            diag.fixes.push(cpl_diag::Fix {
                title: "treat `None` as false".to_string(),
                edits: vec![(expr.span, replacement)],
            });
        }
        true
    }

    pub(crate) fn cond(&mut self, cond: &Cond) {
        match cond {
            Cond::Expr(expr) if self.question_in_condition(expr) => {}
            Cond::Guard { test, and } if self.question_in_condition(test) => {
                self.cond(and);
            }
            Cond::Expr(expr) => {
                let in_opt_fn =
                    matches!(self.fns.last().map(|(_, r, _, _)| r), Some(Ty::Optional(_)));
                if !in_opt_fn {
                    if let Some(try_span) = Self::find_bare_try(expr) {
                        let message =
                            "`?` cannot be used directly in a condition; write `(... ?? false)`"
                                .to_string();
                        let replacement = format!("({}) ?? false", self.quote(expr.span));
                        if let Some(diag) =
                            self.error(codes::QUESTION_IN_CONDITION, try_span, message)
                        {
                            diag.fixes.push(cpl_diag::Fix {
                                title: "treat `None` as false".to_string(),
                                edits: vec![(expr.span, replacement)],
                            });
                        }
                    }
                }
                let ty = self.infer(expr);
                self.require_bool(&ty, expr, "the condition");
            }
            Cond::Const(expr) => {
                self.infer(expr);
            }
            Cond::Guard { test, and } => {
                let in_opt_fn =
                    matches!(self.fns.last().map(|(_, r, _, _)| r), Some(Ty::Optional(_)));
                if !in_opt_fn {
                    if let Some(try_span) = Self::find_bare_try(test) {
                        let message =
                            "`?` cannot be used directly in a condition; write `(... ?? false)`"
                                .to_string();
                        let replacement = format!("({}) ?? false", self.quote(test.span));
                        if let Some(diag) =
                            self.error(codes::QUESTION_IN_CONDITION, try_span, message)
                        {
                            diag.fixes.push(cpl_diag::Fix {
                                title: "treat `None` as false".to_string(),
                                edits: vec![(test.span, replacement)],
                            });
                        }
                    }
                }
                let ty = self.infer(test);
                self.require_bool(&ty, test, "the condition");
                self.cond(and);
            }
            Cond::Let { pat, value, and } => {
                let ty = match self.infer_optional(value) {
                    Ty::Optional(inner) => *inner,
                    Ty::Error => Ty::Error,
                    _ => Ty::Unknown,
                };
                self.bind(pat, &ty, false, "let");
                if let Some(and) = and {
                    self.cond(and);
                }
            }
        }
    }

    fn find_bare_try(expr: &Expr) -> Option<Span> {
        match &expr.kind {
            ExprKind::Try(_) => Some(expr.span),
            ExprKind::Binary {
                op: BinOp::Coalesce,
                ..
            } => None,
            ExprKind::Binary { lhs, rhs, .. } => {
                Self::find_bare_try(lhs).or_else(|| Self::find_bare_try(rhs))
            }
            ExprKind::Compare { first, rest } => Self::find_bare_try(first)
                .or_else(|| rest.iter().find_map(|(_, rhs)| Self::find_bare_try(rhs))),
            ExprKind::Unary { expr: operand, .. }
            | ExprKind::Paren(operand)
            | ExprKind::Unwrap(operand)
            | ExprKind::Field { base: operand, .. } => Self::find_bare_try(operand),
            ExprKind::Call { callee, args } => Self::find_bare_try(callee)
                .or_else(|| args.iter().find_map(|a| Self::find_bare_try(&a.value))),
            ExprKind::Index { base, indices } => {
                Self::find_bare_try(base).or_else(|| indices.iter().find_map(Self::find_bare_try))
            }
            _ => None,
        }
    }

    fn bad_operand(&mut self, ty: &Ty, expr: &Expr, op: &str) -> Ty {
        if ty.is_vague() {
            return ty.clone();
        }
        if matches!(ty, Ty::Bool | Ty::Str) {
            let message = format!(
                "`{op}` needs a number, but `{}` is {}",
                self.quote(expr.span),
                ty.with_article()
            );
            self.error(codes::BAD_OPERANDS, expr.span, message);
            return Ty::Error;
        }
        Ty::Unknown
    }

    fn binary(&mut self, op: BinOp, op_span: Span, lhs: &Expr, rhs: &Expr) -> Ty {
        let left = if op == BinOp::Coalesce {
            self.infer_optional(lhs)
        } else {
            self.infer(lhs)
        };
        if let Ty::Param(param) = &left {
            let trait_name = match op {
                BinOp::Add => Some("Add"),
                BinOp::Sub => Some("Sub"),
                BinOp::Mul => Some("Mul"),
                BinOp::Div => Some("Div"),
                BinOp::Rem => Some("Rem"),
                BinOp::Pow => Some("Pow"),
                _ => None,
            };
            if let Some(trait_name) = trait_name {
                if !self.param_has_bound(param, trait_name) {
                    let message = format!(
                        "`{}` requires `{param}: {trait_name}`, which is not in the bounds",
                        op.text()
                    );
                    self.error(codes::MISSING_IMPL, op_span, message);
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
                        if let Some(diag) = self.error(
                            codes::UNPARENTHESIZED_COALESCE,
                            lhs.span,
                            "`??` mixed with `+ - * / %` needs parentheses".to_string(),
                        ) {
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
            _ if self.operator_trait(op, &left).is_some() => {
                let (trait_name, owner) = self.operator_trait(op, &left).unwrap();
                let op_method = match op {
                    BinOp::Add => "add",
                    BinOp::Sub => "sub",
                    BinOp::Mul => "mul",
                    BinOp::Div => "div",
                    BinOp::Rem => "rem",
                    BinOp::Pow => "pow",
                    _ => "",
                };
                let matching: Vec<(Option<Ty>, Option<Ty>, Option<Ty>)> = self
                    .env
                    .impls(&owner)
                    .iter()
                    .filter(|(implemented, _)| {
                        implemented.as_deref().and_then(|t| t.split('<').next()) == Some(trait_name)
                    })
                    .map(|(_, def)| match &**def {
                        Def::Impl {
                            trait_args,
                            out_ty,
                            methods,
                            ..
                        } => {
                            let rhs_ty = trait_args.first().cloned();
                            let method_ret = methods.iter().find_map(|(m, ty)| {
                                if m == op_method {
                                    if let Ty::Fn { ret, .. } = ty {
                                        if !matches!(**ret, Ty::Unknown | Ty::Unit) {
                                            return Some((**ret).clone());
                                        }
                                    }
                                }
                                None
                            });
                            (rhs_ty, out_ty.clone(), method_ret)
                        }
                        _ => (None, None, None),
                    })
                    .collect();
                if matching.is_empty() {
                    let message = format!(
                        "`{}` needs `impl {trait_name} for {owner}`, and there is none",
                        op.text()
                    );
                    if let Some(diag) = self.error(codes::MISSING_IMPL, op_span, message) {
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
                        let message = format!("`{}` mixes a {left} and a {right}", op.text());
                        self.error(codes::TYPE_MISMATCH, op_span, message);
                        Ty::Error
                    }
                    None => {
                        let message = format!(
                            "`{}` needs `impl {trait_name}<{right}> for {owner}`, and there is none",
                            op.text()
                        );
                        if let Some(diag) = self.error(codes::MISSING_IMPL, op_span, message) {
                            diag.help = Some(format!(
                                "write `impl {trait_name}<{right}> for {owner}` to define `{owner} {} {right}`",
                                op.text()
                            ));
                        }
                        Ty::Error
                    }
                }
            }
            _ if left == Ty::Error || right == Ty::Error => Ty::Error,
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
                let message = format!("`{}` can't combine an `I128` and a `U64`", op.text());
                if let Some(diag) = self.error(codes::BAD_OPERANDS, op_span, message) {
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
            BinOp::Add if seq_concat_elem(&left).is_some() && seq_concat_elem(&right).is_some() => {
                let le = seq_concat_elem(&left).unwrap();
                let re = seq_concat_elem(&right).unwrap();
                if le == re || re.is_vague() {
                    Ty::Named {
                        name: "List".into(),
                        args: vec![le.clone()],
                    }
                } else if le.is_vague() {
                    Ty::Named {
                        name: "List".into(),
                        args: vec![re.clone()],
                    }
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
                    let into_string = (op == BinOp::Add)
                        .then(|| self.string_fix(lhs, rhs))
                        .flatten();
                    let message = format!(
                        "`{}` can't combine {} and {}",
                        op.text(),
                        left.widen().with_article(),
                        right.widen().with_article()
                    );
                    if let Some(diag) = self.error(codes::BAD_OPERANDS, op_span, message) {
                        if let Some((span, text)) = into_string {
                            diag.fix("put the value in the string", span, text);
                        } else if left == Ty::Str || right == Ty::Str {
                            diag.help = Some("put the value in the string: `\"{x}\"`".into());
                        }
                    }
                    return Ty::Error;
                }
                Ty::Unknown
            }
        }
    }

    /// A range runs between two ends of one domain, and `..=` ends at a
    /// position, never at a boundary.
    fn range_ends(
        &mut self,
        range: &Expr,
        hi: Option<&Expr>,
        inclusive: bool,
        lo_ty: &Option<Ty>,
        hi_ty: &Option<Ty>,
    ) {
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
                let message = format!("range between {a} and {b}");
                if let Some(diag) = self.error(codes::WRONG_DOMAIN, range.span, message) {
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
            let message = format!(
                "`..=` ends at a position; a range up to the boundary `{}` is written `..`",
                self.quote(hi.span)
            );
            // R04.4.6: where a piece is wanted, `j..=i` on cuts is the wrong type.
            let code = if spec {
                codes::TYPE_MISMATCH
            } else {
                codes::WRONG_DOMAIN
            };
            self.error(code, range.span, message);
        }
    }

    /// Whether the step `e`, a `?I`, is known to stay inside `I`: a test
    /// or a loop around it orders the name it steps from against another
    /// position of `I` ([`cpl_syntax::order`]).
    fn step_stays(&self, e: &Expr, domain: &Ty) -> bool {
        matches!(domain, Ty::Pos(_))
            && self.steps.bounds(e, 0).into_iter().any(|name| {
                match cpl_syntax::order::end_domain(name) {
                    // `i > D.first`: an end of the domain itself.
                    Some(of) => matches!(domain, Ty::Pos(d) if d == of),
                    None => self.local(name).is_some_and(|local| local.ty == *domain),
                }
            })
    }

    /// `i + 1` used where its domain is wanted: the message for indexing
    /// with it. A step is a `?I`, `none` when it leaves `I`.
    fn unhandled_step(&self, got: &Ty, want: &Ty, at: &Expr) -> Option<String> {
        let Ty::Optional(inner) = got else {
            return None;
        };
        (matches!(**inner, Ty::Pos(_)) && inner.fits(want)).then(|| {
            format!(
                "`{}` is a {got}: it may step outside {inner}",
                self.quote(at.span)
            )
        })
    }

    fn step_help(&self, at: &Expr) -> String {
        let text = self.quote(at.span);
        format!(
            "bind it with `if let j = {text}`, give the edge a value with `??`, or assert with `({text})!`"
        )
    }

    /// Reports a comparison between positions of two different domains.
    /// The trait an arithmetic operator asks of a struct on its left, and
    /// the struct's name: `a + b` with `a: Pt` is `Add` for `Pt`.
    fn operator_trait(&mut self, op: BinOp, left: &Ty) -> Option<(&'static str, String)> {
        let trait_name = match op {
            BinOp::Add => "Add",
            BinOp::Sub => "Sub",
            BinOp::Mul => "Mul",
            BinOp::Div => "Div",
            BinOp::Rem => "Rem",
            BinOp::Pow => "Pow",
            _ => return None,
        };
        let Ty::Named { name, .. } = left else {
            return None;
        };
        matches!(self.type_def(name).as_deref(), Some(Def::Struct { .. }))
            .then(|| (trait_name, name.clone()))
    }

    fn comparable(
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
            let message = format!("optional values can't be ordered with `{op_str}`");
            if let Some(diag) = self.error(codes::OPTIONAL_COMPARE, left_expr.span, message) {
                diag.help = Some("unwrap with `??` or `!` before comparing".into());
            }
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
                    let message = format!("`{left}` does not implement `{trait_name}`");
                    self.error(codes::MISSING_IMPL, left_expr.span, message);
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
        let message = format!(
            "`{}` is {} and `{}` is {}; positions of different domains can't be compared",
            self.quote(left_expr.span),
            left.with_article(),
            self.quote(right_expr.span),
            right.with_article()
        );
        self.error(
            codes::WRONG_DOMAIN,
            left_expr.span.to(right_expr.span),
            message,
        );
    }

    /// What a `for` over a value of type `ty` yields.
    pub(crate) fn element_ty(&mut self, ty: &Ty) -> Ty {
        match ty {
            Ty::Domain(name) => Ty::Pos(name.clone()),
            Ty::TypeName(name) if name == "Bool" => Ty::Bool,
            Ty::TypeName(name) => Ty::Named {
                name: name.clone(),
                args: Vec::new(),
            },
            Ty::Tuple(items)
                if !items.is_empty()
                    && items
                        .iter()
                        .all(|t| matches!(t, Ty::Domain(_) | Ty::TypeName(_))) =>
            {
                let elems = items.iter().map(|t| self.element_ty(t)).collect();
                Ty::Tuple(elems)
            }
            Ty::Record(fields)
                if !fields.is_empty()
                    && fields
                        .iter()
                        .all(|(_, t)| matches!(t, Ty::Domain(_) | Ty::TypeName(_))) =>
            {
                let elems = fields
                    .iter()
                    .map(|(n, t)| (n.clone(), self.element_ty(t)))
                    .collect();
                Ty::Record(elems)
            }
            Ty::Array { elem, .. } => (**elem).clone(),
            Ty::Str => Ty::Char,
            Ty::Named { name, args }
                if matches!(
                    name.as_str(),
                    "List" | "Set" | "HashSet" | "Range" | "SortedRange" | "Gen"
                ) && args.len() == 1 =>
            {
                args[0].clone()
            }
            Ty::Named { name, args } => {
                if let Some(members) = self.members_of(name) {
                    let bindings = members.bindings(args);
                    if let Some((_, each_ty)) = members.members.iter().find(|(m, _)| m == "each") {
                        if let Ty::Fn { ret, .. } = each_ty.subst(&bindings) {
                            if let Ty::Named {
                                name: rname,
                                args: rargs,
                            } = ret.as_ref()
                            {
                                if matches!(rname.as_str(), "Gen" | "List") && rargs.len() == 1 {
                                    return self.settle(&rargs[0]);
                                }
                            }
                        }
                    }
                }
                Ty::Unknown
            }
            Ty::Error => Ty::Error,
            _ => Ty::Unknown,
        }
    }

    /// The type of field `name` on a value of type `ty`, if `ty` is a
    /// struct or record that has it.
    pub(super) fn field_of(&mut self, ty: &Ty, name: &ast::Name) -> Option<Ty> {
        match ty {
            Ty::Record(fields) => fields
                .iter()
                .find(|(n, _)| *n == name.text)
                .map(|(_, t)| t.clone()),
            Ty::Named {
                name: type_name,
                args,
            } => {
                let members = self.members_of(type_name)?;
                let bindings = members.bindings(args);
                members
                    .fields
                    .iter()
                    .find(|f| f.name == name.text)
                    .map(|f| self.settle(&f.ty.subst(&bindings)))
            }
            _ => None,
        }
    }

    /// A struct value seen as a struct with fewer fields, when one is
    /// wanted: a `Group<Int>` has the `id` and `op` of a `Monoid<Int>`, so
    /// it is one (decision 14.1). Anything else is returned as it is.
    pub(crate) fn narrowed(&mut self, got: &Ty, want: &Ty) -> Ty {
        let (
            Ty::Named {
                name: got_name,
                args: got_args,
            },
            Ty::Named {
                name: want_name, ..
            },
        ) = (got, want)
        else {
            return got.clone();
        };
        if got_name == want_name {
            return got.clone();
        }
        let (Some(got_members), Some(want_members)) =
            (self.members_of(got_name), self.members_of(want_name))
        else {
            return got.clone();
        };
        if want_members.fields.is_empty() {
            return got.clone();
        }
        let got_bindings = got_members.bindings(got_args);
        let mut bindings = crate::ty::Bindings::default();
        for want_field in &want_members.fields {
            let Some(got_field) = got_members
                .fields
                .iter()
                .find(|f| f.name == want_field.name)
            else {
                return got.clone();
            };
            let got_ty = got_field.ty.subst(&got_bindings);
            if !self.fits(&got_ty, &want_field.ty) {
                return got.clone();
            }
            want_field.ty.unify(&got_ty, &mut bindings);
        }
        Ty::Named {
            name: want_name.clone(),
            args: want_members
                .generics
                .iter()
                .map(|g| bindings.get(g).cloned().unwrap_or(Ty::Unknown))
                .collect(),
        }
    }

    /// Everything `value.name` can reach on a struct or enum: its fields,
    /// its methods (a `@get` method reads as its value), what its body
    /// owns, and the same from each impl for it.
    pub(crate) fn members_of(&mut self, type_name: &str) -> Option<Members> {
        let mut out = Members::default();
        let add = |out: &mut Members, methods: &[(String, Ty)], getters: &[String]| {
            for (name, ty) in methods {
                let ty = match ty {
                    Ty::Fn { ret, .. } if getters.contains(name) => (**ret).clone(),
                    other => other.clone(),
                };
                out.members.push((name.clone(), ty));
            }
        };
        match self.type_def(type_name).as_deref() {
            Some(Def::Struct {
                generics,
                fields,
                methods,
                getters,
                mutators,
                owned,
                projections,
                ..
            }) => {
                out.mutators.extend(mutators.iter().cloned());
                out.generics = generics.clone();
                out.fields = fields.clone();
                add(&mut out, methods, getters);
                out.members.extend(owned.iter().cloned());
                out.projections.extend(projections.iter().cloned());
            }
            Some(Def::Enum { methods, .. }) => add(&mut out, methods, &[]),
            _ => return None,
        }
        for (trait_name, def) in self.env.impls(type_name) {
            // Every fn of the trait comes with the impl: the ones the
            // trait writes a body for, and the ones the impl must write
            // (one it leaves out is reported once, on the impl).
            if let Some(Def::Trait { methods, .. }) = trait_name
                .as_deref()
                .and_then(|t| t.split('<').next())
                .and_then(|t| self.type_def(t))
                .as_deref()
            {
                for method in methods {
                    out.members.push((method.name.clone(), method.ty.clone()));
                }
            }
            if let Def::Impl {
                methods,
                getters,
                mutators,
                owned,
                projections,
                ..
            } = &*def
            {
                out.mutators.extend(mutators.iter().cloned());
                add(&mut out, methods, getters);
                out.members.extend(owned.iter().cloned());
                out.projections.extend(projections.iter().cloned());
            }
        }
        Some(out)
    }

    /// The top-level fn `name` whose first parameter is a `self` of the
    /// struct or enum `base`, as the method it is: the fn without that
    /// parameter.
    fn own_method(&mut self, base: &Ty, name: &str) -> Option<Ty> {
        if !matches!(base, Ty::Named { .. } | Ty::Record(_) | Ty::Tuple(_)) {
            return None;
        }
        let def = self
            .global(name)
            .filter(|d| matches!(&**d, Def::Fn { .. }))
            .or_else(|| self.global(&crate::prelude::std_method(name)))?;
        let Def::Fn {
            ty: Ty::Fn { params, ret },
            ..
        } = &*def
        else {
            return None;
        };
        let first = params.first().filter(|p| p.name == "self")?;
        let same_shape = match (&first.ty, base) {
            (Ty::Named { name: takes, .. }, Ty::Named { name: on, .. }) => takes == on,
            (Ty::Record(takes), Ty::Record(on)) => {
                takes.len() == on.len() && takes.iter().zip(on).all(|(a, b)| a.0 == b.0)
            }
            (Ty::Tuple(takes), Ty::Tuple(on)) => takes.len() == on.len(),
            (Ty::Tuple(takes), Ty::Record(on)) => takes.len() == on.len(),
            _ => false,
        };
        same_shape.then(|| Ty::Fn {
            params: params[1..].to_vec(),
            ret: ret.clone(),
        })
    }

    fn field(&mut self, expr: &Expr, base: &Expr, name: &ast::Name) -> Ty {
        // R04b.1.7: `let t = w.rebase` names its fresh domain `t.Idx`.
        if let ExprKind::Name(view) = &base.kind {
            if name.text == "Idx" {
                let own = format!("{view}.Idx");
                if let Some(Ty::Array { index, .. }) = self.local(view).map(|l| l.ty.clone()) {
                    if matches!(index.as_slice(), [Ty::Pos(d)] if *d == own) {
                        return Ty::Domain(own);
                    }
                }
                if let Some(def) = self.global(view) {
                    if let Ty::Array { index, .. } = def.value_ty(view) {
                        if matches!(index.as_slice(), [Ty::Pos(d)] if *d == own) {
                            return Ty::Domain(own);
                        }
                    }
                }
            }
        }
        // `treap.merge` after `use lib.treap`: a public name of a module.
        if let ExprKind::Name(alias) = &base.kind {
            if self.local(alias).is_none() && self.env.is_module(alias) {
                return match self.global(&format!("{alias}.{}", name.text)) {
                    Some(def) => def.value_ty(&name.text),
                    None if name.text.is_empty() => Ty::Error,
                    None if self.env.is_private_member(alias, &name.text) => {
                        let message = format!("`{}` is private to module `{alias}`", name.text);
                        self.error(codes::PRIVATE_NAME, name.span, message);
                        Ty::Error
                    }
                    None => {
                        let message =
                            format!("module `{alias}` has no public name `{}`", name.text);
                        self.error(codes::UNRESOLVED_NAME, name.span, message);
                        Ty::Error
                    }
                };
            }
        }
        // `D.cuts` is never empty: its first and last cuts are always there.
        if let ExprKind::Field {
            base: inner,
            name: bounds,
        } = &base.kind
        {
            if matches!(bounds.text.as_str(), "bounds" | "cuts")
                && matches!(name.text.as_str(), "first" | "last")
            {
                if let Ty::Domain(d) = self.infer(inner) {
                    return Ty::Pos(format!("{d}.cuts"));
                }
            }
        }
        let base_ty = self.infer(base);
        if name.text == "bounds" && matches!(&base_ty, Ty::Domain(_)) {
            let message = "`D.bounds` was renamed to `D.cuts`".to_string();
            self.error(codes::REMOVED_SYNTAX, name.span, message);
            return Ty::Error;
        }
        if self.member_advice(&base_ty, name) {
            return Ty::Error;
        }

        let ty = self.member_ty(expr, name, base_ty.clone());
        let built_in = matches!(
            &base_ty,
            Ty::Str | Ty::Range { .. } | Ty::Pos(_) | Ty::Domain(_) | Ty::Array { .. }
        ) || matches!(&base_ty, Ty::Named { name, .. } if name == "List" || name == "Range");
        // A generator only has what std declares on sequences: no `next` (R07.1.1).
        let generator = matches!(&base_ty, Ty::Named { name, .. } if name == "Gen");
        if ty == Ty::Unknown
            && (built_in || generator)
            && !(if generator {
                crate::prelude::spec_receiver_method_names().contains(&name.text)
            } else {
                (crate::prelude::spec_member_names().contains(&name.text)
                    && !(matches!(&base_ty, Ty::Named { name, .. } if name == "Range")
                        && crate::prelude::spec_struct_only_member_names().contains(&name.text)))
                    || crate::builtins::BUILTIN_MEMBERS
                        .iter()
                        .any(|m| m.name == name.text)
            })
            && self.global(&name.text).is_none()
        {
            let message = format!(
                "{} has no `{}` in the spec: no std module declares it",
                base_ty.widen(),
                name.text
            );
            let suggestion = super::spec_member_typo(&base_ty, &name.text);
            let call = Span::new(base.span.end, name.span.end + 2);
            let iter_call = crate::builtins::foreign_iter_call(&name.text)
                && matches!(&base_ty, Ty::Str | Ty::Named { .. })
                && self.text(Span::new(name.span.end, call.end)) == "()";
            if let Some(diag) = self.error(codes::UNKNOWN_FIELD, name.span, message) {
                if iter_call {
                    diag.help = Some("a list or string is iterated as it is: `for x in xs`".into());
                    diag.fix(format!("remove `.{}()`", name.text), call, "");
                } else if let Some(s) = suggestion {
                    diag.suggest(name.span, s);
                } else if let Some(help) = super::advice::foreign_call(&name.text) {
                    diag.help = Some(help.to_string());
                } else if generator {
                    diag.help = Some("a generator pushes its values into a loop: `for x in g:`, or `List(g)` to keep them (R07.1.1)".to_string());
                }
            }
        }
        // A built-in member (`xs.len`, `v.next`) gets its hover here.
        let hovered = self.hovers.last().is_some_and(|h| h.span == name.span);
        let owner = match &base_ty {
            Ty::Str => Some("Str".to_string()),
            Ty::Array { .. } => Some("[D]T".to_string()),
            Ty::Named { name, .. } => Some(name.clone()),
            _ => None,
        };
        if let Some(owner) = owner.as_deref() {
            self.missing_parens(expr, base, name, owner);
        }
        // A catalog method the call types later (`xs.sorted()`) is still
        // known here, by its catalog signature.
        let known = match ty {
            Ty::Error => false,
            Ty::Unknown => owner
                .as_deref()
                .is_some_and(|o| crate::builtins::builtin_member(o, &name.text).is_some()),
            _ => true,
        };
        if !hovered && !name.text.is_empty() && known {
            self.hovers.push(HoverEntry {
                span: name.span,
                name: name.text.clone(),
                ty: ty.clone(),
                keyword: "field",
                decl: None,
                owner,
            });
        }
        let positional = match &base_ty {
            Ty::Domain(_) => ["nth", "at", "wrap"].contains(&name.text.as_str()),
            Ty::Pos(_) => ["offset", "ahead", "back"].contains(&name.text.as_str()),
            _ => false,
        };
        if positional && matches!(ty, Ty::Fn { .. }) {
            self.fixed_io.push(name.span);
        }
        ty
    }

    fn member_ty(&mut self, expr: &Expr, name: &ast::Name, base_ty: Ty) -> Ty {
        if name.text.is_empty() {
            return Ty::Error;
        }
        if let Ty::Param(p) = &base_ty {
            if let Some(req) = self.row_bounds.get(p).cloned() {
                if let Some((_, fty)) = req.iter().find(|(fnm, _)| fnm == &name.text) {
                    return fty.clone();
                }
                self.error(
                    codes::UNKNOWN_FIELD,
                    name.span,
                    format!("row `{p}` does not name field `{}`", name.text),
                );
                return Ty::Error;
            }
        }
        // `tree.Pre`: the domain a value owns, named by its type parameter.
        if let Ty::Named {
            name: type_name,
            args,
        } = &base_ty
        {
            if let Some(members) = self.members_of(type_name) {
                if members.generics.contains(&name.text) {
                    return match members.bindings(args).get(&name.text) {
                        Some(Ty::Pos(domain)) => Ty::Domain(domain.clone()),
                        _ => Ty::Unknown,
                    };
                }
            }
        }
        if let Some(elem) = base_ty.seq_elem() {
            match name.text.as_str() {
                "len" => {
                    if !base_ty.is_sized() {
                        let message = format!(
                            "`.len` requires `Sized`, which `{base_ty}` does not implement"
                        );
                        if let Some(diag) = self.error(codes::MISSING_IMPL, name.span, message) {
                            diag.help = Some(
                                "a lazy sequence has no stored length: `.count()` walks it once"
                                    .into(),
                            );
                            diag.fix("write `.count()`", name.span, "count()");
                        }
                        return Ty::Error;
                    }
                    return Ty::Int;
                }
                "is_empty" => return Ty::Bool,
                "last" | "rev" if !base_ty.is_bidir() => {
                    let message = format!(
                        "`.{}` requires `Bidir`, which `{base_ty}` does not implement",
                        name.text
                    );
                    self.error(codes::MISSING_IMPL, name.span, message);
                    return Ty::Error;
                }
                "items" | "keys" if !base_ty.is_keyed() => {
                    let message = format!(
                        "`.{}` requires `Keyed`, which `{base_ty}` does not implement",
                        name.text
                    );
                    self.error(codes::MISSING_IMPL, name.span, message);
                    return Ty::Error;
                }
                "first" | "last" if base_ty == Ty::Str => {
                    return Ty::Optional(Box::new(elem));
                }
                "get" => {
                    return Ty::Fn {
                        params: vec![crate::Param {
                            name: "i".into(),
                            ty: Ty::Int,
                            mutable: false,
                            has_default: false,
                            named_only: false,
                            variadic: false,
                            lazy: false,
                        }],
                        ret: Box::new(Ty::Optional(Box::new(elem))),
                    };
                }
                "at" => {
                    let (arg, ret) = match &base_ty {
                        Ty::Domain(d) => match self.product_factors(d) {
                            Some(factors) => (
                                Ty::Tuple(vec![Ty::Unknown; factors.len()]),
                                Ty::Optional(Box::new(elem)),
                            ),
                            None => (Ty::Int, Ty::Optional(Box::new(elem))),
                        },
                        _ => (Ty::Int, elem),
                    };
                    return Ty::Fn {
                        params: vec![crate::Param {
                            name: "i".into(),
                            ty: arg,
                            mutable: false,
                            has_default: false,
                            named_only: false,
                            variadic: false,
                            lazy: false,
                        }],
                        ret: Box::new(ret),
                    };
                }
                "set_at" if base_ty.is_indexed_mut() => {
                    return Ty::Fn {
                        params: vec![
                            crate::Param {
                                name: "i".into(),
                                ty: Ty::Int,
                                mutable: false,
                                has_default: false,
                                named_only: false,
                                variadic: false,
                                lazy: false,
                            },
                            crate::Param {
                                name: "x".into(),
                                ty: elem,
                                mutable: false,
                                has_default: false,
                                named_only: false,
                                variadic: false,
                                lazy: false,
                            },
                        ],
                        ret: Box::new(Ty::Unit),
                    };
                }
                "swap_idx" if base_ty.is_indexed_mut() => {
                    return Ty::Fn {
                        params: vec![
                            crate::Param {
                                name: "i".into(),
                                ty: Ty::Int,
                                mutable: false,
                                has_default: false,
                                named_only: false,
                                variadic: false,
                                lazy: false,
                            },
                            crate::Param {
                                name: "j".into(),
                                ty: Ty::Int,
                                mutable: false,
                                has_default: false,
                                named_only: false,
                                variadic: false,
                                lazy: false,
                            },
                        ],
                        ret: Box::new(Ty::Unit),
                    };
                }
                "reach" => {
                    let pred_ty = Ty::Fn {
                        params: vec![
                            crate::Param {
                                name: "a".into(),
                                ty: elem.clone(),
                                mutable: false,
                                has_default: false,
                                named_only: false,
                                variadic: false,
                                lazy: false,
                            },
                            crate::Param {
                                name: "b".into(),
                                ty: elem,
                                mutable: false,
                                has_default: false,
                                named_only: false,
                                variadic: false,
                                lazy: false,
                            },
                        ],
                        ret: Box::new(Ty::Bool),
                    };
                    return Ty::Fn {
                        params: vec![
                            crate::Param {
                                name: "while".into(),
                                ty: pred_ty,
                                mutable: false,
                                has_default: false,
                                named_only: false,
                                variadic: false,
                                lazy: false,
                            },
                            crate::Param {
                                name: "cyclic".into(),
                                ty: Ty::Bool,
                                mutable: false,
                                has_default: true,
                                named_only: true,
                                variadic: false,
                                lazy: false,
                            },
                        ],
                        ret: Box::new(Ty::list(Ty::Int)),
                    };
                }
                "halves" if matches!(&base_ty, Ty::Named { name, .. } if name == "Range") => {
                    return Ty::Fn {
                        params: vec![],
                        ret: Box::new(Ty::Tuple(vec![base_ty.clone(), base_ty.clone()])),
                    };
                }
                _ => {}
            }
        }
        let own_pow =
            matches!(&base_ty, Ty::Named { .. }) && !self.implements_trait(&base_ty, "Pow");
        if name.text == "pow" && !own_pow {
            let message = "power uses `**`, not `.pow(...)`".to_string();
            let fix = self.pow_fix(expr, name);
            if let Some(diag) = self.error(codes::UNKNOWN_FIELD, name.span, message) {
                let shown = fix.as_ref().map_or("a ** e", |f| f.edits[0].1.as_str());
                diag.help = Some(format!("write `{shown}`"));
                diag.fixes.extend(fix);
            }
            return Ty::Error;
        }
        if name.text == "enumerate" && base_ty.is_seq() {
            let message =
                "no `enumerate` on sequences; CP Lang spells index-element pairs `.items`"
                    .to_string();
            if let Some(diag) = self.error(codes::UNKNOWN_FIELD, name.span, message) {
                diag.help = Some("use `.items`: `for (i, x) in xs.items:`".to_string());
            }
            return Ty::Error;
        }
        let renamed = match name.text.as_str() {
            "bsearch_first" => Some((
                "first_true",
                "`r.first_true(ok)` gives the first element where `ok` holds, as `?`",
            )),
            "bsearch_last" => Some((
                "last_true",
                "`r.last_true(ok)` gives the last element where `ok` holds, as `?`",
            )),
            "bounds" if matches!(base_ty, Ty::Domain(_)) => {
                Some(("cuts", "`D.cuts` is the boundaries of `D`"))
            }
            _ => None,
        };
        if let Some((to, help)) = renamed {
            let message = format!("`.{}` is gone in 0.3; it is `.{to}`", name.text);
            if let Some(diag) = self.error(codes::REMOVED_SYNTAX, name.span, message) {
                diag.help = Some(help.to_string());
                diag.fixes.push(cpl_diag::Fix {
                    title: format!("use `.{to}`"),
                    edits: vec![(name.span, to.to_string())],
                });
            }
            return Ty::Error;
        }

        // `r.lo` / `r.hi` on `Range<T>` were renamed to `.start` / `.end` (`end` exclusive).
        if matches!(name.text.as_str(), "lo" | "hi")
            && matches!(&base_ty, Ty::Named { name: r, .. } if r == "Range")
        {
            let replacement = if name.text == "lo" { "start" } else { "end" };
            let message = format!("`Range` uses `.{replacement}`, not `.{}`", name.text);
            if let Some(diag) = self.error(codes::REMOVED_SYNTAX, name.span, message) {
                diag.help = Some(format!(
                    "write `.{replacement}` (`.end` is exclusive for `a..b` and `a..=b` alike)"
                ));
                diag.fixes.push(cpl_diag::Fix {
                    title: format!("use `.{replacement}`"),
                    edits: vec![(name.span, replacement.to_string())],
                });
            }
            return Ty::Error;
        }
        // `r.start` / `r.end` on `Range<T>` give the lower / exclusive-upper endpoint.
        if matches!(name.text.as_str(), "start" | "end") {
            if let Ty::Named { name: r, args } = &base_ty {
                if r == "Range" {
                    return match args.first() {
                        Some(Ty::Pos(d)) => {
                            if d.ends_with(".cuts") {
                                Ty::Pos(d.clone())
                            } else {
                                Ty::Pos(format!("{d}.cuts"))
                            }
                        }
                        Some(Ty::I128) => Ty::I128,
                        Some(Ty::Int | Ty::Range { .. }) => Ty::Int,
                        _ => Ty::Unknown,
                    };
                }
            }
        }
        // `r.drop(k)` / `r.take(k)` on `Range<T>` returns `Range<T>`.
        if matches!(name.text.as_str(), "drop" | "take")
            && matches!(&base_ty, Ty::Named { name: r, .. } if r == "Range")
        {
            return Ty::Fn {
                params: vec![crate::ty::Param {
                    name: "k".into(),
                    ty: Ty::Int,
                    mutable: false,
                    has_default: false,
                    named_only: false,
                    variadic: false,
                    lazy: false,
                }],
                ret: Box::new(base_ty),
            };
        }
        // `s.rev`, `xs.rev`, `a.rev`: O(1) read-only reverse view over a string, list, or 1D domain array.
        if name.text == "rev" {
            if matches!(&base_ty, Ty::Named { name: range, args } if range == "Range" && args.len() == 1)
            {
                return base_ty;
            }
            if base_ty == Ty::Str
                || matches!(&base_ty, Ty::Named { name: list, args } if list == "List" && args.len() == 1)
            {
                self.note_readonly_view(expr, "reverse view");
                return base_ty;
            }
            if let Ty::Array { index, .. } = &base_ty {
                if index.len() == 1 {
                    self.note_readonly_view(expr, "reverse view");
                    return base_ty;
                }
            }
        }
        // `xs.items`: each element with its index (an `I` for `[I]T`, else `Int`).
        if name.text == "items" {
            let pair = match &base_ty {
                Ty::Named { name: list, args } if list == "List" && args.len() == 1 => {
                    Some((Ty::Int, args[0].clone()))
                }
                Ty::Str => Some((Ty::Int, Ty::Char)),
                Ty::Named { name: map, args }
                    if matches!(map.as_str(), "Map" | "HashMap") && args.len() == 2 =>
                {
                    Some((args[0].clone(), args[1].clone()))
                }
                Ty::Array { index, elem } => {
                    let mut at: Vec<Ty> = index
                        .iter()
                        .map(|axis| match axis {
                            Ty::Pos(domain) => Ty::Pos(domain.clone()),
                            _ => Ty::Int,
                        })
                        .collect();
                    // `((r, c), x)` for a grid; one index stands alone.
                    let at = if at.len() == 1 {
                        at.remove(0)
                    } else {
                        Ty::Tuple(at)
                    };
                    Some((at, (**elem).clone()))
                }
                _ => None,
            };
            if let Some((at, elem)) = pair {
                return Ty::Named {
                    name: "List".into(),
                    args: vec![Ty::Tuple(vec![at, elem])],
                };
            }
        }
        if let Ty::Named { name: sr, args } = &base_ty {
            if sr == "SortedRange" && args.len() == 1 {
                if name.text == "len" {
                    if let ExprKind::Field { base: recv, .. } = &expr.kind {
                        if let Some(owner) = self.uncounted_sorted_range_owner(recv) {
                            if let Some(diag) = self.error(
                                codes::TYPE_MISMATCH,
                                name.span,
                                format!(
                                    "`{owner}.range(...).len` is not available because counting a tree range costs O(log n + k)"
                                ),
                            ) {
                                diag.help = Some(format!(
                                    "use `SortedSet` or `SortedBag` for O(log n) `range(r).len`, or count by iterating `{owner}.range(...)`"
                                ));
                            }
                        }
                    }
                }
                return match name.text.as_str() {
                    "first" | "last" => Ty::Optional(Box::new(args[0].clone())),
                    "len" => Ty::Int,
                    "is_empty" => Ty::Bool,
                    "rev" => base_ty.clone(),
                    _ => Ty::Unknown,
                };
            }
        }
        if name.text == "inverse" {
            if let Ty::Array { index, elem } = &base_ty {
                if let Some(ret) = self.array_inverse_ret(index, elem) {
                    return Ty::Fn {
                        params: Vec::new(),
                        ret: Box::new(ret),
                    };
                }
            }
        }
        if matches!(name.text.as_str(), "keys" | "values") {
            if let Ty::Named { name: map, args } = &base_ty {
                if matches!(map.as_str(), "Map" | "HashMap") && args.len() == 2 {
                    let elem = if name.text == "keys" {
                        args[0].clone()
                    } else {
                        args[1].clone()
                    };
                    return Ty::Named {
                        name: "List".into(),
                        args: vec![elem],
                    };
                }
                if name.text == "keys" && matches!(map.as_str(), "List" | "Slice" | "Str") {
                    return Ty::Named {
                        name: "Range".into(),
                        args: vec![Ty::Int],
                    };
                }
            }
            if name.text == "keys" {
                if let Ty::Array { index, .. } = &base_ty {
                    if let Some(first) = index.first() {
                        return Ty::Named {
                            name: "Range".into(),
                            args: vec![first.clone()],
                        };
                    }
                }
            }
            if let Ty::Array { index, elem } = &base_ty {
                if index.len() == 1 {
                    if name.text == "keys" {
                        return Ty::Named {
                            name: "Range".into(),
                            args: vec![index[0].clone()],
                        };
                    }
                    let item = if name.text == "keys" {
                        index[0].clone()
                    } else {
                        (**elem).clone()
                    };
                    return Ty::Named {
                        name: "List".into(),
                        args: vec![item],
                    };
                }
            }
        }
        let found = match &base_ty {
            Ty::Record(fields) => {
                let names: Vec<String> = fields.iter().map(|(n, _)| n.clone()).collect();
                Some((self.field_of(&base_ty, name), names, Vec::new()))
            }
            Ty::Tuple(items) => {
                return match name.text.parse::<usize>() {
                    Ok(i) if i < items.len() => items[i].clone(),
                    _ => self.own_method(&base_ty, &name.text).unwrap_or(Ty::Unknown),
                };
            }
            Ty::Named {
                name: type_name,
                args,
            } => {
                if type_name == "Pair" && name.text == "count" {
                    return Ty::Int;
                }
                if type_name == "Gen" && name.text == "next" {
                    let message =
                        format!("{base_ty} has no method `next`; walk a generator with `for`");
                    self.error(codes::UNKNOWN_FIELD, name.span, message);
                    return Ty::Error;
                }
                if matches!(type_name.as_str(), "List" | "Str" | "Map" | "Set")
                    && name.text == "buf"
                {
                    let message = format!("{base_ty} has no public field `{}`", name.text);
                    self.error(codes::UNKNOWN_FIELD, name.span, message);
                    return Ty::Error;
                }
                if args.is_empty() && self.finite_index_enum_size(type_name).is_some() {
                    match name.text.as_str() {
                        "ord" | "to_idx" | "idx" | "label" => return Ty::Int,
                        "prev" | "next" => return Ty::Optional(Box::new(base_ty.clone())),
                        "offset" | "ahead" | "back" => {
                            return Ty::Fn {
                                params: vec![crate::ty::Param {
                                    name: "k".into(),
                                    ty: Ty::Int,
                                    mutable: false,
                                    has_default: false,
                                    named_only: false,
                                    variadic: false,
                                    lazy: false,
                                }],
                                ret: Box::new(Ty::Optional(Box::new(base_ty.clone()))),
                            };
                        }
                        _ => {}
                    }
                }
                {
                    let has_inherent = match self.type_def(type_name).as_deref() {
                        Some(Def::Struct { methods, .. } | Def::Enum { methods, .. }) => {
                            methods.iter().any(|(m, _)| m == &name.text)
                        }
                        _ => false,
                    };
                    if !has_inherent {
                        let mut trait_hits: Vec<String> = Vec::new();
                        for (trait_name, def) in self.env.impls(type_name) {
                            let Some(tn) = trait_name else { continue };
                            let has_in_impl = matches!(
                                &*def,
                                Def::Impl { methods, .. }
                                    if methods.iter().any(|(m, _)| m == &name.text)
                            );
                            let has_in_trait = self
                                .type_def(tn.split('<').next().unwrap_or(&tn))
                                .as_deref()
                                .is_some_and(|d| {
                                    matches!(
                                        d,
                                        Def::Trait { methods, .. }
                                            if methods.iter().any(|m| m.name == name.text)
                                    )
                                });
                            if (has_in_impl || has_in_trait) && !trait_hits.contains(&tn) {
                                trait_hits.push(tn);
                            }
                        }
                        if trait_hits.len() >= 2 {
                            let message = format!(
                                "`{type_name}.{}` is ambiguous: both `{}` and `{}` define it",
                                name.text, trait_hits[0], trait_hits[1]
                            );
                            if let Some(diag) =
                                self.error(codes::AMBIGUOUS_METHOD, name.span, message)
                            {
                                diag.help = Some(format!(
                                    "disambiguate with `{}.{}(...)` or `{}.{}(...)`",
                                    trait_hits[0], name.text, trait_hits[1], name.text
                                ));
                            }
                            return Ty::Error;
                        }
                    }
                }
                self.members_of(type_name).map(|members| {
                    let bindings = members.bindings(args);
                    (
                        members
                            .fields
                            .iter()
                            .find(|f| f.name == name.text)
                            .map(|f| f.ty.subst(&bindings)),
                        members.fields.iter().map(|f| f.name.clone()).collect(),
                        members
                            .members
                            .into_iter()
                            .map(|(m, ty)| (m, ty.subst(&bindings)))
                            .collect::<Vec<_>>(),
                    )
                })
            }
            Ty::Param(param) => {
                let bounds: Vec<String> = self
                    .param_bounds
                    .iter()
                    .filter(|(p, _)| p == param)
                    .flat_map(|(_, ts)| ts.iter().cloned())
                    .collect();
                for trait_name in bounds {
                    if let Some(Def::Trait { methods, .. }) = self.type_def(&trait_name).as_deref()
                    {
                        if let Some(m) = methods.iter().find(|m| m.name == name.text) {
                            return match &m.ty {
                                Ty::Fn { params, ret }
                                    if !self.in_call_callee && params.is_empty() =>
                                {
                                    (**ret).clone()
                                }
                                other => other.clone(),
                            };
                        }
                    }
                }
                return Ty::Unknown;
            }
            // The members of a domain that give positions or ranges back.
            Ty::Domain(domain) => {
                return match name.text.as_str() {
                    "first" | "last" => Ty::Optional(Box::new(Ty::Pos(domain.clone()))),
                    "all" => Ty::Unknown,
                    "rev" => Ty::Domain(domain.clone()),
                    "mask" => Ty::Fn {
                        params: vec![crate::ty::Param {
                            name: "vs".into(),
                            ty: Ty::Unknown,
                            mutable: false,
                            has_default: false,
                            named_only: false,
                            variadic: false,
                            lazy: false,
                        }],
                        ret: Box::new(Ty::Array {
                            index: vec![Ty::Pos(domain.clone())],
                            elem: Box::new(Ty::Bool),
                        }),
                    },
                    "wrap" | "nth" => Ty::Fn {
                        params: vec![crate::ty::Param {
                            name: "x".into(),
                            ty: Ty::Int,
                            mutable: false,
                            has_default: false,
                            named_only: false,
                            variadic: false,
                            lazy: false,
                        }],
                        ret: Box::new(Ty::Pos(domain.clone())),
                    },
                    "bounds" | "cuts" => Ty::Named {
                        name: "Range".into(),
                        args: vec![Ty::Pos(format!("{domain}.cuts"))],
                    },
                    "Cut" => {
                        self.planned(name.span, &cpl_syntax::planned::CUT_DOMAIN);
                        Ty::Error
                    }
                    "pairs" => Ty::Named {
                        name: "List".into(),
                        args: vec![Ty::Tuple(vec![
                            Ty::Pos(domain.clone()),
                            Ty::Pos(domain.clone()),
                        ])],
                    },
                    "after" | "before" => Ty::Fn {
                        params: vec![crate::ty::Param {
                            name: "pos".into(),
                            ty: Ty::Pos(domain.clone()),
                            mutable: false,
                            has_default: false,
                            named_only: false,
                            variadic: false,
                            lazy: false,
                        }],
                        ret: Box::new(Ty::Named {
                            name: "Range".into(),
                            args: vec![Ty::Pos(domain.clone())],
                        }),
                    },
                    "partition" => Ty::Fn {
                        params: vec![crate::ty::Param {
                            name: "keep".into(),
                            ty: Ty::Fn {
                                params: vec![crate::ty::Param {
                                    name: "".into(),
                                    ty: Ty::Pos(domain.clone()),
                                    mutable: false,
                                    has_default: false,
                                    named_only: false,
                                    variadic: false,
                                    lazy: false,
                                }],
                                ret: Box::new(Ty::Bool),
                            },
                            mutable: false,
                            has_default: false,
                            named_only: false,
                            variadic: false,
                            lazy: false,
                        }],
                        ret: Box::new(Ty::Tuple(vec![
                            Ty::list(Ty::Pos(domain.clone())),
                            Ty::list(Ty::Pos(domain.clone())),
                        ])),
                    },
                    "filter" => Ty::Fn {
                        params: vec![crate::ty::Param {
                            name: "pred".into(),
                            ty: Ty::Fn {
                                params: vec![crate::ty::Param {
                                    name: "".into(),
                                    ty: Ty::Pos(domain.clone()),
                                    mutable: false,
                                    has_default: false,
                                    named_only: false,
                                    variadic: false,
                                    lazy: false,
                                }],
                                ret: Box::new(Ty::Bool),
                            },
                            mutable: false,
                            has_default: false,
                            named_only: false,
                            variadic: false,
                            lazy: false,
                        }],
                        ret: Box::new(Ty::list(Ty::Pos(domain.clone()))),
                    },
                    "sort" => Ty::Unknown,
                    _ => Ty::Unknown,
                };
            }
            Ty::Char => {
                if matches!(name.text.as_str(), "ord" | "to_idx" | "idx") {
                    return Ty::Int;
                }
                return Ty::Unknown;
            }
            Ty::Pos(domain) => {
                match name.text.as_str() {
                    "ord" | "to_idx" | "idx" | "label" => return Ty::Int,
                    "prev" | "next" => return Ty::Optional(Box::new(Ty::Pos(domain.clone()))),
                    "offset" | "ahead" | "back" => {
                        return Ty::Fn {
                            params: vec![crate::ty::Param {
                                name: "k".into(),
                                ty: Ty::Int,
                                mutable: false,
                                has_default: false,
                                named_only: false,
                                variadic: false,
                                lazy: false,
                            }],
                            ret: Box::new(Ty::Optional(Box::new(Ty::Pos(domain.clone())))),
                        };
                    }
                    // The boundaries on either side of a position.
                    // R04.2.19: the pieces on either side of a position.
                    "before" | "after" if !domain.ends_with(".cuts") => {
                        return range_of(Ty::Pos(domain.clone()));
                    }
                    "l" | "r" | "before" | "after" if !domain.ends_with(".cuts") => {
                        return Ty::Pos(format!("{domain}.cuts"));
                    }
                    _ => {}
                }
                if let Some(factors) = self.product_factors(domain) {
                    if let Some((_, ty)) = factors
                        .iter()
                        .find(|(fname, _)| fname.as_deref() == Some(name.text.as_str()))
                    {
                        return ty.clone();
                    }
                    if let Ok(idx) = name.text.parse::<usize>() {
                        if let Some((_, ty)) = factors.get(idx) {
                            return ty.clone();
                        }
                    }
                }
                return Ty::Unknown;
            }
            // `Dir.L`: a variant named through its enum.
            Ty::TypeName(type_name) => {
                if let Some(Def::Enum { variants, .. }) = self.type_def(type_name).as_deref() {
                    if let Some((_, fields)) = variants.iter().find(|(v, _)| *v == name.text) {
                        return Def::variant_value(type_name, fields);
                    }
                }
                if type_name == "Char" && matches!(name.text.as_str(), "count" | "len" | "K") {
                    return Ty::Int;
                }
                if self.finite_index_enum_size(type_name).is_some() {
                    let elem = Ty::Named {
                        name: type_name.clone(),
                        args: Vec::new(),
                    };
                    match name.text.as_str() {
                        "count" | "len" | "K" => return Ty::Int,
                        "at" => {
                            return Ty::Fn {
                                params: vec![crate::ty::Param {
                                    name: "i".into(),
                                    ty: Ty::Int,
                                    mutable: false,
                                    has_default: false,
                                    named_only: false,
                                    variadic: false,
                                    lazy: false,
                                }],
                                ret: Box::new(Ty::Optional(Box::new(elem))),
                            };
                        }
                        "first" | "last" => return Ty::Optional(Box::new(elem)),
                        "all" | "rev" => {
                            return Ty::Named {
                                name: "Range".into(),
                                args: vec![elem],
                            };
                        }
                        "bounds" | "cuts" => {
                            return Ty::Named {
                                name: "Range".into(),
                                args: vec![Ty::Pos(format!("{type_name}.cuts"))],
                            };
                        }
                        _ => {}
                    }
                }
                // `Graph.read()`, `Rooted.Pre`: what the type has without a value.
                self.members_of(type_name)
                    .map(|members| (None, Vec::new(), members.members))
            }
            _ => None,
        };
        let Some((field_ty, field_names, methods)) = found else {
            // Members of library types, arrays and domains (`xs.len`,
            // `V.rev`) come from `std`, which is not loaded yet.
            return if base_ty == Ty::Error {
                Ty::Error
            } else {
                Ty::Unknown
            };
        };
        let ty = match field_ty {
            Some(ty) => self.settle(&ty),
            None => match methods.iter().find(|(m, _)| *m == name.text) {
                Some((_, ty)) => self.settle(ty),
                // `fn area(self: Sq) -> Int` at the top of the file is
                // a method of `Sq`, and of no other struct.
                None if self.own_method(&base_ty, &name.text).is_some() => {
                    match self.own_method(&base_ty, &name.text).unwrap_or(Ty::Unknown) {
                        Ty::Fn { params, ret } if !self.in_call_callee && params.is_empty() => *ret,
                        other => other,
                    }
                }
                None => {
                    let has_range = methods.iter().any(|(m, _)| m == "range_first");
                    let known = field_names
                        .iter()
                        .chain(methods.iter().map(|(m, _)| m))
                        .map(String::as_str)
                        .chain(has_range.then_some("range"));
                    let suggestion = closest(&name.text, known);
                    let message = format!("{} has no field or method `{}`", base_ty, name.text);
                    if let Some(diag) = self.error(codes::UNKNOWN_FIELD, name.span, message) {
                        match suggestion {
                            Some(s) => diag.suggest(name.span, s),
                            None if !field_names.is_empty() => {
                                diag.help =
                                    Some(format!("its fields are {}", list_names(&field_names)));
                            }
                            None => {}
                        }
                    }
                    return Ty::Error;
                }
            },
        };
        self.hovers.push(HoverEntry {
            span: name.span,
            name: name.text.clone(),
            ty: ty.clone(),
            keyword: "field",
            decl: None,
            owner: match &base_ty {
                Ty::Named { name, .. } | Ty::TypeName(name) => Some(name.clone()),
                _ => None,
            },
        });
        ty
    }

    pub(crate) fn array_inverse_ret(&mut self, index: &[Ty], elem: &Ty) -> Option<Ty> {
        if index.len() != 1 {
            return None;
        }
        let mut is_index_ty = |ty: &Ty| match ty {
            Ty::Pos(_) | Ty::Range { .. } | Ty::Param(_) => true,
            Ty::Named { name, args } if args.is_empty() => {
                self.finite_index_enum_size(name).is_some()
            }
            _ => false,
        };
        let d = index[0].clone();
        match elem {
            Ty::Optional(inner) if is_index_ty(inner) => Some(Ty::Array {
                index: vec![(**inner).clone()],
                elem: Box::new(Ty::Optional(Box::new(d))),
            }),
            other if is_index_ty(other) => Some(Ty::Array {
                index: vec![other.clone()],
                elem: Box::new(d),
            }),
            _ => None,
        }
    }

    /// Records `expr` as in bounds when each index has exactly the domain
    /// type it stands for: a position of it, or the same type parameter.
    fn note_typed_index(
        &mut self,
        expr: &Expr,
        base: &Expr,
        indices: &[Expr],
        got: &[Ty],
        want: &[Ty],
    ) {
        fn space(got: &Ty, want: &Ty) -> bool {
            match (got, want) {
                (Ty::Pos(a), Ty::Pos(b)) | (Ty::Param(a), Ty::Param(b)) => a == b,
                (Ty::Tuple(a), Ty::Tuple(b)) => {
                    a.len() == b.len() && a.iter().zip(b).all(|(a, b)| space(a, b))
                }
                _ => false,
            }
        }
        if indices.is_empty() || got.len() != indices.len() || want.len() != indices.len() {
            return;
        }
        if !got.iter().zip(want).all(|(got, want)| space(got, want)) {
            return;
        }
        let proofs: Vec<String> = indices
            .iter()
            .zip(got)
            .map(|(e, ty)| format!("{}: {ty}", self.quote(e.span)))
            .collect();
        let spaces = if proofs.len() == 1 { "space" } else { "spaces" };
        self.typed_indexes.push(crate::check::TypedIndex {
            access: expr.span,
            seq: base.span,
            index: Span::new(indices[0].span.start, indices[indices.len() - 1].span.end),
            proof: format!(
                "{}, the index {spaces} of {}",
                proofs.join(", "),
                self.quote(base.span)
            ),
        });
    }

    fn index(&mut self, expr: &Expr, base: &Expr, indices: &[Expr]) -> Ty {
        let base_ty = self.infer(base);
        if self.double_index(base, indices) {
            return Ty::Error;
        }
        self.literal_slice_index(base, indices);
        if self.walked_not_indexed(base) {
            for index in indices {
                self.infer(index);
            }
            return Ty::Error;
        }
        let prev_piece = std::mem::replace(&mut self.expected_piece, true);
        let index_tys: Vec<Ty> = indices.iter().map(|i| self.infer(i)).collect();
        self.expected_piece = prev_piece;
        if let (ExprKind::Name(base_name), Some(idx_expr)) = (&base.kind, indices.first()) {
            let idx_owner = match &idx_expr.kind {
                ExprKind::Call { callee, .. } => match &callee.kind {
                    ExprKind::Field {
                        base: owner,
                        name: f,
                    } if f.text == "Idx" => match &owner.kind {
                        ExprKind::Name(n) => Some(n.as_str()),
                        _ => None,
                    },
                    ExprKind::Field { base: inner, .. } => match &inner.kind {
                        ExprKind::Field {
                            base: owner,
                            name: f,
                        } if f.text == "Idx" => match &owner.kind {
                            ExprKind::Name(n) => Some(n.as_str()),
                            _ => None,
                        },
                        _ => None,
                    },
                    _ => None,
                },
                _ => None,
            };
            if let Some(other) = idx_owner {
                if other != base_name {
                    let message =
                        format!("`{base_name}` is indexed by `{base_name}.Idx`, not `{other}.Idx`");
                    self.error(codes::WRONG_DOMAIN, idx_expr.span, message);
                }
            }
        }

        if self.cut_range_not_piece(&base_ty, indices, &index_tys) {
            return Ty::Error;
        }
        match &base_ty {
            // `[Sym<V>]T`: symmetric storage over unordered pairs of `V`,
            // indexed as `adj[u, v]` or `adj[p]` with `p: (V, V)`.
            Ty::Array { index, elem } if matches!(index.as_slice(), [Ty::Named { name, args }] if matches!(name.as_str(), "Sym" | "Pair") && args.len() == 1) =>
            {
                let Ty::Named { args, .. } = &index[0] else {
                    unreachable!()
                };
                let dom_ty = &args[0];
                if indices.len() == 1
                    && (index_tys[0].is_vague()
                        || matches!(&index_tys[0], Ty::Tuple(items) if items.len() == 2))
                {
                    let want = Ty::Tuple(vec![dom_ty.clone(), dom_ty.clone()]);
                    self.expect_fits(&index_tys[0], &want, &indices[0], |c, got, want| {
                        format!(
                            "`{}` is indexed by {}, but `{}` is {}",
                            c.quote(base.span),
                            want,
                            c.quote(indices[0].span),
                            got.with_article()
                        )
                    });
                    return (**elem).clone();
                }
                if indices.len() != 2 {
                    let message = format!(
                        "`{}` takes 2 indices ({}, {}), but {} given",
                        self.quote(base.span),
                        dom_ty,
                        dom_ty,
                        if indices.len() == 1 {
                            "1 was".to_string()
                        } else {
                            format!("{} were", indices.len())
                        },
                    );
                    if let Some(diag) = self.error(codes::NOT_INDEXABLE, expr.span, message) {
                        diag.help =
                            Some("index a symmetric table with two positions: `adj[u, v]`".into());
                    }
                    return Ty::Error;
                }
                for (got, index_expr) in index_tys.iter().zip(indices) {
                    if !self.fits(got, dom_ty) && !self.literal_position(index_expr, dom_ty) {
                        let message = format!(
                            "`{}` is indexed by {}, but `{}` is {}",
                            self.quote(base.span),
                            dom_ty,
                            self.quote(index_expr.span),
                            got.with_article()
                        );
                        self.error(codes::TYPE_MISMATCH, index_expr.span, message);
                    }
                }
                (**elem).clone()
            }
            // `[R, C]T` indexed by a single position `p: P` where `domain P = (R, C)`.
            Ty::Array { index, elem }
                if index.len() > 1
                    && indices.len() == 1
                    && matches!(&index_tys[0], Ty::Pos(dom) if self.product_factors(dom).is_some_and(|f| f.len() == index.len() && f.iter().zip(index).all(|((_, ft), want)| self.fits(ft, want)))) =>
            {
                (**elem).clone()
            }
            // `[State]T` where `domain State = (V, K)`: allow multi-index `dp[v, k]`
            // in addition to `dp[s]` and `dp[(v, k)]`.
            Ty::Array { index, elem }
                if index.len() == 1
                    && indices.len() > 1
                    && matches!(&index[0], Ty::Pos(dom) if self.product_factors(dom).is_some_and(|f| f.len() == indices.len())) =>
            {
                let Ty::Pos(dom) = &index[0] else {
                    unreachable!()
                };
                let factors = self.product_factors(dom).unwrap();
                let wants: Vec<Ty> = factors.iter().map(|(_, want)| want.clone()).collect();
                self.note_typed_index(expr, base, indices, &index_tys, &wants);
                for ((got, (_, want)), index_expr) in index_tys.iter().zip(&factors).zip(indices) {
                    self.expect_fits(got, want, index_expr, |c, got, want| {
                        format!(
                            "`{}` is indexed by {}, but `{}` is {}",
                            c.quote(base.span),
                            want,
                            c.quote(index_expr.span),
                            got.with_article()
                        )
                    });
                }
                (**elem).clone()
            }
            // `g[p]` with `p: (Row, Col)`: `a[i, j]` is sugar for `a[(i, j)]`.
            Ty::Array { index, elem }
                if index.len() > 1
                    && indices.len() == 1
                    && (index_tys[0].is_vague()
                        || matches!(&index_tys[0], Ty::Tuple(items) if items.len() == index.len())) =>
            {
                if let (ExprKind::Tuple(parts), Ty::Tuple(got)) = (&indices[0].kind, &index_tys[0])
                {
                    for ((got, want), part) in got.iter().zip(index.iter()).zip(parts) {
                        self.expect_fits(got, want, part, |c, got, want| {
                            format!(
                                "`{}` is indexed by {}, but `{}` is {}",
                                c.quote(base.span),
                                want,
                                c.quote(part.span),
                                got.with_article()
                            )
                        });
                    }
                    return (**elem).clone();
                }
                let want = Ty::Tuple(index.clone());
                self.note_typed_index(expr, base, indices, &index_tys, std::slice::from_ref(&want));
                self.expect_fits(&index_tys[0], &want, &indices[0], |c, got, want| {
                    format!(
                        "`{}` is indexed by {}, but `{}` is {}",
                        c.quote(base.span),
                        want,
                        c.quote(indices[0].span),
                        got.with_article()
                    )
                });
                (**elem).clone()
            }
            // `grid[r]`: leading indices give the O(1) view of what is left,
            // here row `r`, indexed by the columns.
            Ty::Array { index, elem }
                if indices.len() < index.len()
                    && index_tys.iter().zip(index.iter()).zip(indices).all(
                        |((got, want), e)| self.fits(got, want) || self.literal_position(e, want),
                    ) =>
            {
                Ty::Array {
                    index: index[indices.len()..].to_vec(),
                    elem: elem.clone(),
                }
            }
            Ty::Array { index, elem } => {
                if let [whole] = index_tys.as_slice() {
                    if index.len() > 1 && self.position_factors(whole).as_ref() == Some(index) {
                        return (**elem).clone();
                    }
                }
                if index.len() != indices.len() {
                    let wanted: Vec<String> = index.iter().map(Ty::to_string).collect();
                    let message = format!(
                        "`{}` takes {} ({}), but {} given",
                        self.quote(base.span),
                        plural(index.len(), "index", "indices"),
                        wanted.join(", "),
                        if indices.len() == 1 {
                            "1 was".to_string()
                        } else {
                            format!("{} were", indices.len())
                        },
                    );
                    if let Some(diag) = self.error(codes::NOT_INDEXABLE, expr.span, message) {
                        diag.help = Some(
                            "all indices go in one pair of brackets: `g[r, c]`, never `g[r][c]`"
                                .into(),
                        );
                    }
                    return Ty::Error;
                }
                let mut slice = false;
                // The indices a part keeps: `g[1, ..]` is a row, indexed by
                // the columns alone.
                let mut kept = Vec::new();
                for ((want, got), index_expr) in index.iter().zip(&index_tys).zip(indices) {
                    if let (
                        ExprKind::Range {
                            hi: Some(hi),
                            inclusive: true,
                            ..
                        },
                        Ty::Named { name, args },
                        Ty::Pos(dom),
                    ) = (&index_expr.kind, got, want)
                    {
                        if name == "Range" && args.len() == 1 {
                            if let Ty::Pos(cut) = &args[0] {
                                if self.bounds_base_domain(cut).as_deref() == Some(dom.as_str()) {
                                    let msg = format!(
                                        "`..=` ends at a position; a range up to the boundary `{}` is written `..`",
                                        self.quote(hi.span)
                                    );
                                    if let Some(diag) =
                                        self.error(codes::WRONG_DOMAIN, index_expr.span, msg)
                                    {
                                        diag.help = Some(
                                            "use half-open `..` for the positions between two cuts"
                                                .to_string(),
                                        );
                                    }
                                    slice = true;
                                    kept.push(want.clone());
                                    continue;
                                }
                            }
                        }
                    }
                    if let Ty::Named { name, args } = got {
                        if name == "Range" {
                            if matches!(args.as_slice(), [Ty::Pos(d)] if d.ends_with(".cuts"))
                                && !matches!(want, Ty::Pos(w) if w.ends_with(".cuts"))
                            {
                                let message = format!(
                                    "`{}` is a cut range (`{got}`), not a position range for `{}`",
                                    self.quote(index_expr.span),
                                    self.quote(base.span)
                                );
                                self.error(codes::TYPE_MISMATCH, index_expr.span, message);
                            }
                            slice = true;
                            kept.push(want.clone());
                            continue;
                        }
                    }
                    if self.fits(got, want) {
                        continue;
                    }
                    if self.literal_position(index_expr, want)
                        && self.view_kind(base) != Some("slice")
                    {
                        if let (ExprKind::Int(v), Ty::Pos(d)) = (&index_expr.kind, want) {
                            let is_cuts = d.ends_with(".cuts");
                            let base_dom = d.strip_suffix(".cuts").unwrap_or(d);
                            let out_of_bounds = match self.type_def(base_dom).as_deref() {
                                Some(Def::Domain(info)) if is_cuts => {
                                    *v < 0 || info.max_size.is_some_and(|m| *v > m)
                                }
                                Some(Def::Domain(info)) => {
                                    info.min_pos.is_some_and(|lo| *v < lo)
                                        || info.max_pos.is_some_and(|hi| *v > hi)
                                }
                                _ => false,
                            };
                            if out_of_bounds {
                                let message = format!("literal `{v}` is outside domain `{d}`");
                                self.error(codes::RANGE_STORE_CONST, index_expr.span, message);
                            }
                        }

                        continue;
                    }
                    let message = format!(
                        "`{}` is indexed by {}, but `{}` is {}",
                        self.quote(base.span),
                        want,
                        self.quote(index_expr.span),
                        got.with_article()
                    );
                    let base_text = self.quote(base.span);
                    let index_text = self.quote(index_expr.span);
                    let boundary = match (got, want) {
                        (Ty::Pos(got), Ty::Pos(want)) => {
                            got.strip_suffix(".cuts") == Some(want)
                                || want.strip_suffix(".cuts") == Some(got)
                        }
                        _ => false,
                    };
                    // A step that was not bound, clamped or asserted.
                    if let Some(message) = self.unhandled_step(got, want, index_expr) {
                        let help = self.step_help(index_expr);
                        if let Some(diag) =
                            self.error(codes::WRONG_DOMAIN, index_expr.span, message)
                        {
                            diag.help = Some(help);
                        }
                    } else {
                        let origins = [
                            self.origin(base, &format!("declared over {want}")),
                            self.origin(index_expr, &format!("declared as {}", got.with_article())),
                        ];
                        let Some(diag) = self.error(codes::WRONG_DOMAIN, index_expr.span, message)
                        else {
                            continue;
                        };
                        diag.labels.extend(origins.into_iter().flatten());
                        if let (true, false, Ty::Pos(domain)) = (got.is_int(), boundary, want) {
                            diag.fixes.push(convert_to(domain, index_expr.span));
                        }
                        diag.help = Some(if boundary {
                            "a position and the boundaries around it are different indices; use `.l` / `.r` for the boundaries on either side of a position".to_string()
                        } else if got.is_int() {
                            format!(
                                "convert it with `{want}({index_text})`, or write `{base_text}.raw[{index_text}]` to index by number"
                            )
                        } else {
                            format!("index with {}", want.with_article())
                        });
                    }
                }
                if !slice {
                    self.note_typed_index(expr, base, indices, &index_tys, index);
                }
                if slice {
                    // A slice is its array seen through a range: lent,
                    // and indexed by the same positions.
                    self.note_view(expr, "slice");
                    if kept.len() == index.len() {
                        base_ty.clone()
                    } else {
                        Ty::Array {
                            index: kept,
                            elem: elem.clone(),
                        }
                    }
                } else {
                    (**elem).clone()
                }
            }
            // `s[0..5]`: O(1) read-only slice view into a string.
            Ty::Str if matches!(index_tys.as_slice(), [Ty::Named { name, .. }] if name == "Range") =>
            {
                self.note_readonly_view(expr, "slice");
                Ty::Str
            }
            // `xs[0..5]`: O(1) slice view into a list (mutable when the root is a `var`).
            ty if matches!(ty, Ty::Named { name, .. } if name == "List")
                && matches!(index_tys.as_slice(), [Ty::Named { name, .. }] if name == "Range") =>
            {
                self.note_view(expr, "slice");
                ty.clone()
            }
            Ty::Named { name, args } if name == "Rebased" && args.len() == 1 => {
                self.element_ty(&args[0])
            }
            Ty::Named { name, args } if name == "List" && args.len() == 1 => args[0].clone(),
            Ty::Named { name, .. } if name == "Filter" => {
                let message = format!(
                    "`{}` has type `{base_ty}`, which is not indexed",
                    self.quote(base.span)
                );
                self.error(codes::NOT_INDEXABLE, expr.span, message);
                Ty::Error
            }
            Ty::Named { name, .. } if matches!(name.as_str(), "Gen" | "Iter") => {
                let message = format!(
                    "`{}` has type `{base_ty}`, which does not implement `Indexed`",
                    self.quote(base.span)
                );
                self.error(codes::MISSING_IMPL, expr.span, message);
                Ty::Error
            }
            Ty::Named { name, args } if name == "Range" && args.len() == 1 => args[0].clone(),
            Ty::Named { name, args } if name == "Map" && args.len() == 2 => args[1].clone(),
            Ty::Str => Ty::Char,
            ty if ty.is_numeric() || matches!(ty, Ty::Bool | Ty::Char | Ty::Fn { .. }) => {
                let message = format!(
                    "`{}` is {} and can't be indexed",
                    self.quote(base.span),
                    ty.widen().with_article()
                );
                let help = match ty {
                    Ty::Fn { .. } => "call it with `(...)`",
                    _ => "`[...]` reads arrays, maps, ranges and strings",
                };
                if let Some(diag) = self.error(codes::NOT_INDEXABLE, base.span, message) {
                    diag.help = Some(help.into());
                }
                Ty::Error
            }
            Ty::Error => Ty::Error,
            _ => Ty::Unknown,
        }
    }

    /// True if the struct `name` is a number: it has a `zero`, a `one`
    /// and a `new(x: Int)`, so an `Int` is made into it where it is
    /// wanted. `Mod<P>` is one, and so is any struct written the same way.
    pub(crate) fn is_number(&mut self, name: &str) -> bool {
        match self.type_def(name).as_deref() {
            Some(Def::Struct {
                ctor: Some(params),
                owned,
                ..
            }) => {
                matches!(params.as_slice(), [only] if only.ty.is_int())
                    && ["zero", "one"]
                        .iter()
                        .all(|c| owned.iter().any(|(n, _)| n == c))
            }
            _ => false,
        }
    }

    /// If `dom` is a cut domain (`D.bounds` or `domain Cut = D.bounds`),
    /// returns the underlying position domain name `D`.
    pub(crate) fn bounds_base_domain(&mut self, dom: &str) -> Option<String> {
        if let Some(base) = dom.strip_suffix(".bounds") {
            return Some(base.to_string());
        }
        if let Some(Def::Domain(info)) = self.env.global(dom).as_deref() {
            if let Some(base) = info.text.strip_suffix(".bounds") {
                return Some(base.to_string());
            }
        }
        None
    }

    /// `got.fits(want)`, with an `Int` fitting a number struct, cut-domain
    /// values/ranges (§124), and product tuples/records fitting their product domain.
    pub(crate) fn fits(&mut self, got: &Ty, want: &Ty) -> bool {
        if got.fits_in(want, &mut |name| self.is_number(name)) {
            return true;
        }
        match (got, want) {
            (Ty::Named { name, args }, Ty::Pos(dom))
                if name == "Subset"
                    && args.len() == 1
                    && matches!(&args[0], Ty::Pos(d) | Ty::Domain(d) if dom.strip_suffix(".subsets") == Some(d.as_str())) =>
            {
                return true;
            }
            (Ty::Pos(dom), Ty::Named { name, args })
                if name == "Subset"
                    && args.len() == 1
                    && matches!(&args[0], Ty::Pos(d) | Ty::Domain(d) if dom.strip_suffix(".subsets") == Some(d.as_str())) =>
            {
                return true;
            }
            // §124: integer literals and checked arithmetic fit `D.bounds`.
            (a, Ty::Pos(dom)) if a.is_int() && self.bounds_base_domain(dom).is_some() => {
                return true;
            }
            // §124: half-open cut range `Range<D.bounds>` fits `Range<D>`.
            (Ty::Named { name: an, args: aa }, Ty::Named { name: bn, args: ba })
                if an == "Range" && bn == "Range" && aa.len() == 1 && ba.len() == 1 =>
            {
                if let (Ty::Pos(cut), Ty::Pos(dom)) = (&aa[0], &ba[0]) {
                    if self.bounds_base_domain(cut).as_deref() == Some(dom.as_str()) {
                        return true;
                    }
                }
                if matches!(&aa[0], Ty::Int | Ty::Range { .. }) && matches!(&ba[0], Ty::Pos(_)) {
                    return true;
                }
            }
            (Ty::Array { index: a, elem: ae }, Ty::Array { index: b, elem: be })
                if a.len() == 1 && b.len() > 1 =>
            {
                if let Ty::Pos(dom) = &a[0] {
                    if let Some(factors) = self.product_factors(dom) {
                        return factors.len() == b.len()
                            && factors.iter().zip(b).all(|((_, f), w)| self.fits(f, w))
                            && self.fits(ae, be);
                    }
                }
            }
            (Ty::Array { index: a, elem: ae }, Ty::Array { index: b, elem: be })
                if b.len() == 1 && a.len() > 1 =>
            {
                if let Ty::Pos(dom) = &b[0] {
                    if let Some(factors) = self.product_factors(dom) {
                        return factors.len() == a.len()
                            && a.iter().zip(&factors).all(|(g, (_, w))| self.fits(g, w))
                            && self.fits(ae, be);
                    }
                }
            }
            (Ty::Named { name: an, args: aa }, Ty::Named { name: bn, args: ba })
                if an == "List" && bn == "List" && aa.len() == 1 && ba.len() == 1 =>
            {
                return self.fits(&aa[0], &ba[0]);
            }
            (Ty::Tuple(items), Ty::Pos(dom)) => {
                if let Some(factors) = self.product_factors(dom) {
                    return items.len() == factors.len()
                        && items
                            .iter()
                            .zip(&factors)
                            .all(|(g, (_, w))| self.fits(g, w));
                }
            }
            (Ty::Record(fields), Ty::Pos(dom)) => {
                if let Some(factors) = self.product_factors(dom) {
                    return fields.len() == factors.len()
                        && fields.iter().zip(&factors).all(|((gn, g), (wn, w))| {
                            wn.as_ref().is_none_or(|wn| wn == gn) && self.fits(g, w)
                        });
                }
            }
            (Ty::Tuple(a), Ty::Tuple(b)) if a.len() == b.len() => {
                return a.iter().zip(b).all(|(g, w)| self.fits(g, w));
            }
            (_, Ty::Optional(inner)) => {
                return self.fits(got, inner);
            }
            _ => {}
        }
        false
    }

    /// Reports with `message(self, got, want)` unless `got` fits `want`.
    /// A list literal written where a 1D array is wanted is that array's
    /// cells, also inside a tuple literal or as the value of a `match` or
    /// `if` arm: `([p[1], p[0]], [q, r, s])` is a `([2]Site, [3]Site)`.
    fn literal_shape_fits(&mut self, at: &Expr, got: &Ty, want: &Ty) -> bool {
        let tail = |block: &ast::Block| match block.stmts.last().map(|s| &s.kind) {
            Some(ast::StmtKind::Expr(e)) => Some(e.clone()),
            _ => None,
        };
        let part = |c: &mut Self, item: &Expr, g: &Ty, w: &Ty| {
            c.fits(g, w) || c.literal_position(item, w) || c.literal_shape_fits(item, g, w)
        };
        let list_elem = |ty: &Ty| match ty {
            Ty::Named { name, args } if name == "List" && args.len() == 1 => Some(args[0].clone()),
            Ty::Array { index, elem } if index.len() == 1 => Some((**elem).clone()),
            _ => None,
        };
        match (&at.kind, got, want) {
            (ExprKind::List(items), Ty::Named { name, args }, _)
                if name == "List" && list_elem(want).is_some() =>
            {
                let w = list_elem(want).unwrap_or(Ty::Error);
                match args.first() {
                    Some(g) => items.iter().all(|item| part(self, item, g, &w)),
                    None => true,
                }
            }
            (ExprKind::Tuple(items), Ty::Tuple(gots), Ty::Tuple(wants))
                if items.len() == gots.len() && gots.len() == wants.len() =>
            {
                items
                    .iter()
                    .zip(gots.iter().zip(wants))
                    .all(|(item, (g, w))| part(self, item, g, w))
            }
            (ExprKind::Record(items), Ty::Record(gots), Ty::Record(wants))
                if items.len() == gots.len() && gots.len() == wants.len() =>
            {
                items.iter().all(|(name, item)| {
                    let g = gots.iter().find(|(n, _)| *n == name.text).map(|(_, t)| t);
                    let w = wants.iter().find(|(n, _)| *n == name.text).map(|(_, t)| t);
                    matches!((g, w), (Some(g), Some(w)) if part(self, item, g, w))
                })
            }
            (ExprKind::Match { arms, .. }, _, _) => {
                let tails: Vec<_> = arms.iter().map(|arm| tail(&arm.body)).collect();
                tails.iter().all(|t| {
                    t.as_ref()
                        .is_some_and(|e| self.literal_shape_fits(e, got, want))
                })
            }
            (
                ExprKind::If {
                    branches,
                    otherwise,
                },
                _,
                _,
            ) => {
                let mut tails: Vec<_> = branches.iter().map(|(_, b)| tail(b)).collect();
                tails.push(otherwise.as_ref().and_then(tail));
                tails.iter().all(|t| {
                    t.as_ref()
                        .is_some_and(|e| self.literal_shape_fits(e, got, want))
                })
            }
            _ => false,
        }
    }

    pub(crate) fn expect_fits(
        &mut self,
        got: &Ty,
        want: &Ty,
        at: &Expr,
        message: impl FnOnce(&Self, &Ty, &Ty) -> String,
    ) -> bool {
        // `none` fills whatever is declared, and code that already has an
        // error is not reported again.
        let got = &self.narrowed(got, want);
        // §124: `j..=i` over cuts never converts to `Range<D>`.
        if let ExprKind::Range {
            hi: Some(hi),
            inclusive: true,
            ..
        } = &at.kind
        {
            let inner_want = match want {
                Ty::Optional(inner) => inner.as_ref(),
                other => other,
            };
            if let (Ty::Named { name: an, args: aa }, Ty::Named { name: bn, args: ba }) =
                (got, inner_want)
            {
                if an == "Range" && bn == "Range" && aa.len() == 1 && ba.len() == 1 {
                    if let (Ty::Pos(cut), Ty::Pos(dom)) = (&aa[0], &ba[0]) {
                        if self.bounds_base_domain(cut).as_deref() == Some(dom.as_str()) {
                            let msg = format!(
                                "`..=` ends at a position; a range up to the boundary `{}` is written `..`",
                                self.quote(hi.span)
                            );
                            if let Some(diag) = self.error(codes::WRONG_DOMAIN, at.span, msg) {
                                diag.help = Some(
                                    "use half-open `..` for the positions between two cuts"
                                        .to_string(),
                                );
                            }
                            return false;
                        }
                    }
                }
            }
        }
        if let Ty::Range {
            lo: want_lo,
            hi: want_hi,
            ..
        } = want
        {
            let (vlo, vhi) = self.bounds(at);
            let out_low = matches!((vhi, want_lo), (Some(vh), Some(wl)) if vh < *wl);
            let out_high = matches!((vlo, want_hi), (Some(vl), Some(wh)) if vl > *wh);
            if out_low || out_high {
                let msg = format!("`{}` is out of range for `{want}`", self.quote(at.span));
                self.error(codes::RANGE_STORE_CONST, at.span, msg);
                return false;
            }
        }

        if self.fits(got, want)
            || matches!(at.kind, ExprKind::None)
            || got.has_error()
            || want.has_error()
        {
            return true;
        }
        // A list literal `[e0, e1, ...]` fits a 1D `[D]T` array when element
        // types fit (and for a plain enum domain `D`, when the length equals |D|).
        if let ExprKind::List(items) = &at.kind {
            let inner_want = match want {
                Ty::Optional(inner) => inner.as_ref(),
                other => other,
            };
            if let (Ty::Named { name, args }, Ty::Array { index, elem }) = (got, inner_want) {
                if name == "List" && index.len() == 1 {
                    let elem_ok = args.first().is_none_or(|g| self.fits(g, elem));
                    match &index[0] {
                        Ty::Named { name: ename, .. } => {
                            if let Some(Def::Enum { variants, .. }) =
                                self.type_def(ename).as_deref().cloned()
                            {
                                if variants.iter().all(|(_, fs)| fs.is_empty()) {
                                    if items.len() != variants.len() {
                                        let msg = format!(
                                            "`{inner_want}` holds {} elements (one per `{ename}` variant), found {}",
                                            variants.len(),
                                            items.len()
                                        );
                                        self.error(codes::TYPE_MISMATCH, at.span, msg);
                                        return false;
                                    }
                                    if elem_ok {
                                        return true;
                                    }
                                }
                            }
                        }
                        Ty::Pos(_) if elem_ok => return true,
                        _ => {}
                    }
                }
            }
        }
        if self.literal_position(at, want) || self.literal_shape_fits(at, got, want) {
            return true;
        }
        if self.unhandled_step(got, want, at).is_some() {
            let text = format!(
                "expected {want}, got {got}: `{}` may step outside {want}",
                self.quote(at.span)
            );
            let help = self.step_help(at);
            // R04.8.4: a `?V` where a `V` is wanted is the wrong type, not the wrong domain.
            let code = { codes::TYPE_MISMATCH };
            if let Some(diag) = self.error(code, at.span, text) {
                diag.help = Some(help);
            }
            return false;
        }
        if matches!(got, Ty::Domain(_)) && !matches!(want, Ty::Domain(_)) {
            self.planned(at.span, &cpl_syntax::planned::DOMAIN_VALUE);
            return false;
        }
        let text = message(self, got, want);
        let is_empty_fold = matches!(got, Ty::Optional(_))
            && !matches!(want, Ty::Optional(_))
            && matches!(
                &at.kind,
                ExprKind::Call { callee, args }
                    if args.len() == 1
                        && matches!(&callee.kind, ExprKind::Field { name, .. } if name.text == "fold")
            );
        let code = if is_empty_fold {
            codes::EMPTY_FOLD_NO_IDENTITY
        } else if matches!((got, want), (Ty::Pos(_), Ty::Pos(_))) {
            codes::WRONG_DOMAIN
        } else {
            codes::TYPE_MISMATCH
        };
        let quoted = self.quote(at.span);
        let origin = self.origin(at, &format!("declared as {}", got.with_article()));
        if let Some(diag) = self.error(code, at.span, text) {
            diag.labels.extend(origin);
            if let (true, Ty::Pos(domain)) = (got.is_int(), want) {
                diag.help = Some(format!("convert it with `{domain}({quoted})`"));
                diag.fixes.push(convert_to(domain, at.span));
            } else if let Some(help) = rename_fields(got, want) {
                diag.help = Some(help);
            }
        }
        false
    }
}

/// `(a: V, b: V)` given where `(u: V, v: V)` is wanted, inside the same
/// arrays, lists or optionals: the help names the fields to rename.
fn rename_fields(got: &Ty, want: &Ty) -> Option<String> {
    match (got, want) {
        (Ty::Array { elem: got, .. }, Ty::Array { elem: want, .. })
        | (Ty::Optional(got), Ty::Optional(want)) => rename_fields(got, want),
        (
            Ty::Named { name, args: got },
            Ty::Named {
                name: wanted,
                args: want,
            },
        ) if name == wanted && got.len() == want.len() => got
            .iter()
            .zip(want)
            .find_map(|(got, want)| rename_fields(got, want)),
        (Ty::Record(got), Ty::Record(want)) if got.len() >= want.len() => {
            let has = |fields: &[(String, Ty)], name: &str| fields.iter().any(|(n, _)| n == name);
            let missing: Vec<&str> = want
                .iter()
                .map(|(n, _)| n.as_str())
                .filter(|n| !has(got, n))
                .collect();
            let spare: Vec<&str> = got
                .iter()
                .map(|(n, _)| n.as_str())
                .filter(|n| !has(want, n))
                .collect();
            if missing.is_empty() || spare.len() < missing.len() {
                return None;
            }
            let pairs: Vec<String> = spare
                .iter()
                .zip(&missing)
                .map(|(from, to)| format!("`{from}` to `{to}`"))
                .collect();
            Some(format!(
                "the fields are read by name: rename {}",
                pairs.join(", ")
            ))
        }
        _ => None,
    }
}

impl Checker<'_> {
    /// `domain Cut = Pos.cuts` names the cuts of `Pos`: as a range end it
    /// is `Pos.cuts`.
    fn unalias_bounds(&mut self, ty: Ty) -> Ty {
        if let Ty::Pos(d) = &ty {
            if !d.ends_with(".cuts") {
                if let Some(Def::Domain(info)) = self.type_def(d).as_deref() {
                    let text = info.text.trim();
                    if let Some(base) = text.strip_suffix(".cuts") {
                        if !base.is_empty() && base.chars().all(|c| c.is_alphanumeric() || c == '_')
                        {
                            return Ty::Pos(text.to_string());
                        }
                    }
                }
            }
        }
        ty
    }

    /// An integer literal written where a position or cut of an integer
    /// domain is expected (or fills an array of them) is that position.
    fn literal_position(&mut self, at: &Expr, want: &Ty) -> bool {
        // R04.4.26: a literal `j..i` where a `Range<V>` is wanted is the piece between them.
        if let (
            ExprKind::Range {
                lo, hi, step: None, ..
            },
            Ty::Named { name, args },
        ) = (&at.kind, want)
        {
            let [elem @ Ty::Pos(_)] = args.as_slice() else {
                return false;
            };
            if name != "Range" || (lo.is_none() && hi.is_none()) {
                return false;
            }
            let elem = elem.clone();
            return [lo, hi]
                .into_iter()
                .flatten()
                .all(|end| self.literal_position(end, &elem));
        }
        let ExprKind::Int(_) = at.kind else {
            return false;
        };
        let domain = match want {
            Ty::Pos(d) => d,
            Ty::Array { elem, .. } => match elem.as_ref() {
                Ty::Pos(d) => d,
                _ => return false,
            },
            _ => return false,
        };
        let base = domain.strip_suffix(".cuts").unwrap_or(domain).to_string();
        if self
            .local(&base)
            .is_some_and(|l| matches!(l.ty, Ty::Domain(_)))
        {
            return !self.local_product_domains.contains_key(&base);
        }
        let text = match self.type_def(&base).as_deref() {
            Some(Def::Domain(info)) if info.factors.is_empty() => info.text.clone(),
            _ => return false,
        };
        self.literal_in_domain(at, &base, &text, domain.ends_with(".cuts"));
        true
    }

    /// R04.2.18: a literal outside a static domain bound is an error.
    fn literal_in_domain(&mut self, at: &Expr, domain: &str, text: &str, cut: bool) {
        let ExprKind::Int(v) = at.kind else {
            return;
        };
        let bounds = text.split_once("..").and_then(|(lo, hi)| {
            let (hi, inclusive) = match hi.strip_prefix('=') {
                Some(hi) => (hi, true),
                None => (hi, false),
            };
            let lo: i128 = lo.trim().parse().ok()?;
            let hi: i128 = hi.trim().parse().ok()?;
            Some((lo, if inclusive { hi } else { hi - 1 }))
        });
        let Some((lo, hi)) = bounds else {
            return;
        };
        let (lo, hi) = if cut { (0, hi - lo + 1) } else { (lo, hi) };
        if (lo..=hi).contains(&v) {
            return;
        }
        let what = if cut {
            format!("{domain}.cuts")
        } else {
            domain.to_string()
        };
        let message = format!("`{v}` is outside `{what}`, which is {lo}..={hi}");
        if let Some(diag) = self.error(codes::RANGE_STORE_CONST, at.span, message) {
            diag.help = Some(format!("use a literal in {lo}..={hi}"));
        }
    }
}

impl Checker<'_> {
    /// Where the index space of a plain name comes from: its declaration,
    /// as "`name` is <what> here". A top-level name is declared in another
    /// item, so its label is left as a point at the use for the database
    /// to move onto the declaration.
    fn origin(&mut self, expr: &Expr, what: &str) -> Option<cpl_diag::Label> {
        let ExprKind::Name(name) = &expr.kind else {
            return None;
        };
        let span = match self.local(name).map(|local| local.span) {
            Some(span) => span,
            None if self.env.global(name).is_some() => Span::point(expr.span.start),
            None => return None,
        };
        Some(cpl_diag::Label {
            span,
            message: format!("`{name}` is {what} here"),
        })
    }
}

/// Wraps the number at `span` in `domain(...)`, the checked conversion.
fn convert_to(domain: &str, span: Span) -> cpl_diag::Fix {
    cpl_diag::Fix {
        title: format!("convert with `{domain}(...)`"),
        edits: vec![
            (Span::point(span.start), format!("{domain}(")),
            (Span::point(span.end), ")".into()),
        ],
    }
}

/// What a struct or enum, with its impls, lets a value reach by name.
#[derive(Default)]
pub(crate) struct Members {
    generics: Vec<String>,
    pub(crate) fields: Vec<crate::ty::Param>,
    pub(crate) members: Vec<(String, Ty)>,
    pub(crate) projections: Vec<crate::def::Projection>,
    /// The methods that take `mut self`.
    pub(crate) mutators: Vec<String>,
}

impl Members {
    /// What each type parameter is for a value of the type with `args`.
    pub(crate) fn bindings(&self, args: &[Ty]) -> crate::ty::Bindings {
        if args.len() != self.generics.len() {
            return crate::ty::Bindings::default();
        }
        self.generics
            .iter()
            .cloned()
            .zip(args.iter().cloned())
            .collect()
    }
}

pub(crate) fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

pub(crate) fn list_names(names: &[String]) -> String {
    names
        .iter()
        .map(|n| format!("`{n}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Whether `ty` can stand on one side of a `none`-absorbing `+` or `-`: a
/// number or an optional number. `Some(true)` for the integers.
fn tropical_side(ty: &Ty) -> Option<bool> {
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
        let sign = if bin == BinOp::Add { "+" } else { "-" };
        let message = format!("`{sign}=` can't be applied to {}", target_ty.with_article());
        if let Some(diag) = self.error(codes::BAD_OPERANDS, target.span, message) {
            diag.help = Some(format!(
                "write `x = x {sign} y` when `none` should stay `none`, or `x min= y + w` for a transition"
            ));
        }
    }

    fn uncounted_sorted_range_owner(&mut self, expr: &Expr) -> Option<&'static str> {
        let mut cur = expr;
        loop {
            match &cur.kind {
                ExprKind::Paren(inner) => cur = inner,
                ExprKind::Field { base, name } if name.text == "rev" => cur = base,
                ExprKind::Call { callee, .. } => {
                    let ExprKind::Field { base, name } = &callee.kind else {
                        return None;
                    };
                    if name.text != "range" {
                        return None;
                    }
                    let base_ty = self.quietly(|c| c.infer(base));
                    return match base_ty {
                        Ty::Named { name, .. } if name == "Set" => Some("Set"),
                        Ty::Named { name, .. } if name == "Map" => Some("Map"),
                        _ => None,
                    };
                }
                _ => return None,
            }
        }
    }

    fn coalesce_ty(&mut self, left: Ty, right: Ty, rhs: &Expr) -> Ty {
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
            Ty::Error => right,
            _ if diverges => Ty::Unknown,
            _ if right.is_vague() => Ty::Unknown,
            _ => right.widen(),
        }
    }
}

fn seq_concat_elem(ty: &Ty) -> Option<&Ty> {
    match ty {
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
fn split_chain(src: &str, first: &Expr, rest: &[(ast::CmpOp, Expr)]) -> Option<String> {
    let mut parts = Vec::with_capacity(rest.len());
    let mut left = first;
    for (_, right) in rest {
        parts.push(src.get(left.span.start as usize..right.span.end as usize)?);
        left = right;
    }
    Some(parts.join(" and "))
}

impl Checker<'_> {
    pub(crate) fn walk_only_expr(&self, e: &Expr) -> bool {
        let e = crate::check::lends::unparen(e);
        matches!(&e.kind, ExprKind::Name(n) if self.walk_only.contains(n))
    }

    /// R07.1.2: `filter` gives a `Seq`, walked in order with no `s[i]`.
    fn walked_not_indexed(&mut self, base: &Expr) -> bool {
        if !self.walk_only_expr(base) {
            return false;
        }
        let shown = self.quote(base.span);
        let message =
            format!("`{shown}` is a filtered sequence: it can be walked, but not indexed");
        if let Some(diag) = self.error(codes::NOT_INDEXABLE, base.span, message) {
            diag.help = Some(format!("collect it first: `{shown}.collect()[i]`"));
        }
        true
    }
}

impl Checker<'_> {
    /// R01.8.5: `g[r][c]` is spelled `g[r, c]`.
    fn double_index(&mut self, base: &Expr, indices: &[Expr]) -> bool {
        let ExprKind::Index {
            base: inner,
            indices: first,
        } = &crate::check::lends::unparen(base).kind
        else {
            return false;
        };
        let point = |es: &[Expr]| {
            es.iter()
                .all(|e| !matches!(crate::check::lends::unparen(e).kind, ExprKind::Range { .. }))
        };
        let ExprKind::Name(table) = &inner.kind else {
            return false;
        };
        // A row of a `List<List<T>>` is a list of its own; only a table takes `g[r, c]`.
        let ty = match self.local(table) {
            Some(local) => local.ty.clone(),
            None => self
                .global(table)
                .map_or(Ty::Unknown, |def| def.value_ty(table)),
        };
        let is_table = matches!(ty, Ty::Array { index, .. } if index.len() > 1);
        if !point(first) || !point(indices) || !is_table {
            return false;
        }
        let parts: Vec<String> = first
            .iter()
            .chain(indices)
            .map(|e| self.quote(e.span))
            .collect();
        let fixed = format!("{}[{}]", self.quote(inner.span), parts.join(", "));
        let message = format!(
            "`{}[{}]` indexes twice; 0.3 writes one index with commas",
            self.quote(base.span),
            indices
                .iter()
                .map(|e| self.quote(e.span))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let span = base.span.to(indices.last().map_or(base.span, |e| e.span));
        if let Some(diag) = self.error(codes::REMOVED_SYNTAX, span, message) {
            diag.help = Some(format!("write `{fixed}`"));
        }
        true
    }
}

impl Checker<'_> {
    /// R07.4.5: `w[0]` on a slice reads source key 0, not its first element.
    fn literal_slice_index(&mut self, base: &Expr, indices: &[Expr]) {
        let [index] = indices else {
            return;
        };
        let ExprKind::Int(v) = crate::check::lends::unparen(index).kind else {
            return;
        };
        if self.view_kind(base) != Some("slice") {
            return;
        }
        let w = self.quote(base.span);
        if let Ty::Array { index, .. } = self.quietly(|c| c.infer(base)) {
            if let [Ty::Domain(d) | Ty::Pos(d)] = index.as_slice() {
                let message = format!(
                    "`{w}` is a slice of a `{d}` table, so it is indexed by `{d}`, not by the raw `Int` {v}"
                );
                if let Some(diag) = self.error(codes::WRONG_DOMAIN, indices[0].span, message) {
                    diag.help = Some(format!(
                        "index it with a `{d}`: `{w}[{d}({v})]`, or use `{w}.nth({v})!`"
                    ));
                }
                return;
            }
        }
        let message = format!(
            "`{w}[{v}]` reads source key {v}: a slice keeps its source's keys, so this is not its element {v}"
        );
        self.sink.push(
            Diagnostic::warning(codes::LITERAL_SLICE_INDEX, index.span, message).with_help(format!(
                "write `{w}.nth({v})!` (or `{w}.first!`) for the element by position, or `{w}.rebase` to count from 0"
            )),
        );
    }

    /// R07.4.5: `for k in 0..w.len: w[k]` counts from 0 over a slice's keys.
    pub(crate) fn counter_over_slice(&mut self, iter: &Expr, span: cpl_diag::Span) {
        let ExprKind::Range {
            lo: Some(lo),
            hi: Some(hi),
            ..
        } = &crate::check::lends::unparen(iter).kind
        else {
            return;
        };
        if !matches!(lo.kind, ExprKind::Int(0)) {
            return;
        }
        let ExprKind::Field { base, name } = &hi.kind else {
            return;
        };
        if name.text != "len" || self.view_kind(base) != Some("slice") {
            return;
        }
        let w = self.quote(base.span);
        let message =
            format!("`0..{w}.len` counts from 0, but the slice `{w}` keeps its source's keys");
        self.sink.push(
            Diagnostic::warning(codes::LITERAL_SLICE_INDEX, span, message).with_help(format!(
                "walk it (`for x in {w}`, `for (k, x) in {w}.items`), or `{w}.rebase` to count from 0"
            )),
        );
    }
}

impl Checker<'_> {
    /// R07.1.7: parentheses mark work, so `xs.sorted` reads as O(1) but isn't.
    fn missing_parens(&mut self, expr: &Expr, base: &Expr, name: &ast::Name, owner: &str) {
        if name.text.is_empty() {
            return;
        }
        let Some(member) = crate::builtins::builtin_member(owner, &name.text) else {
            return;
        };
        if member.kind != crate::builtins::MemberKind::Method
            || !member.params.is_empty()
            || member.ret.is_empty()
        {
            return;
        }
        let after = self.src.get(expr.span.end as usize..).unwrap_or("");
        if after.trim_start().starts_with('(') {
            return;
        }
        let shown = format!("{}.{}", self.quote(base.span), name.text);
        let message = format!("`{shown}` does work, but without `()` it reads like an O(1) field");
        self.sink.push(
            Diagnostic::warning(codes::PARENS_MEAN_WORK, expr.span, message)
                .with_help(format!("write `{shown}()`")),
        );
    }
}

impl Checker<'_> {
    /// R04.4.5, R04.4.6: only a literal half-open `j..i` of cuts is a piece.
    fn cut_range_not_piece(&mut self, base_ty: &Ty, indices: &[Expr], index_tys: &[Ty]) -> bool {
        if !matches!(base_ty, Ty::Array { .. }) {
            return false;
        }
        for (index, ty) in indices.iter().zip(index_tys) {
            let Ty::Named { name, args } = ty else {
                continue;
            };
            let [Ty::Pos(d)] = args.as_slice() else {
                continue;
            };
            if name != "Range" || !d.ends_with(".cuts") {
                continue;
            }
            let shown = self.quote(index.span);
            let (message, help) = match &crate::check::lends::unparen(index).kind {
                ExprKind::Range {
                    inclusive: true,
                    lo,
                    hi,
                    ..
                } => (
                    format!("`{shown}` is a closed range of cuts, which names no piece"),
                    match (lo, hi) {
                        (Some(lo), Some(hi)) => {
                            format!("write `{}..{}`", self.quote(lo.span), self.quote(hi.span))
                        }
                        _ => "write the half-open `j..i`".to_string(),
                    },
                ),
                _ => (
                    format!(
                        "`{shown}` is a stored `Range<{d}>`; only a literal `j..i` of cuts becomes a piece"
                    ),
                    "write the range where it is used: `a[j..i]`".to_string(),
                ),
            };
            if let Some(diag) = self.error(codes::TYPE_MISMATCH, index.span, message) {
                diag.help = Some(help);
            }
            return true;
        }
        false
    }
}

impl Checker<'_> {
    /// R04b.1.7: `let t = w.rebase` is `[t.Idx]T`, keyed by a domain of its own.
    pub(crate) fn rebased(&mut self, name: &str, init: &Expr) -> Option<Ty> {
        let ExprKind::Field { base, name: field } = &crate::check::lends::unparen(init).kind else {
            return None;
        };
        if field.text != "rebase" {
            return None;
        }
        let elem = self.quietly(|c| c.infer(base)).seq_elem()?;
        Some(Ty::Array {
            index: vec![Ty::Pos(format!("{name}.Idx"))],
            elem: Box::new(elem),
        })
    }
}

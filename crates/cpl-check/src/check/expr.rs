//! Expression types.

mod binary;
mod fits;
mod index;
mod member;

use super::*;
use binary::split_chain;
use cpl_diag::msgs;
use cpl_syntax::ast::{self, BinOp, Cond, Expr, ExprKind, StrPiece, UnOp};
use fits::convert_to;
pub(super) use fits::{narrows_cells, rename_fields};

fn range_of(elem: Ty) -> Ty {
    range_ty(elem)
}

impl<'a> Checker<'a> {
    pub(crate) fn infer(&mut self, expr: &Expr) -> Ty {
        // An outermost place expression (`a[i]`, not the `a` inside it)
        // uses its place: a conflict with a live lend is reported once
        // its parts are known.
        let uses_place = self.enter_place(expr);
        let mut ty = self.infer_kind(expr);
        if self.rhs_extent == Some(expr.span) && !ty.is_vague() && !matches!(ty, Ty::Optional(_)) {
            ty = Ty::Optional(Box::new(ty)); // the extent of its `?` (R02.4.7)
        }
        if uses_place {
            self.check_access(expr, false);
        }
        self.record_shape(expr, &ty);
        ty
    }

    fn infer_kind(&mut self, expr: &Expr) -> Ty {
        match &expr.kind {
            ExprKind::Int(_) => Ty::Int,
            ExprKind::Float(_) => Ty::Float,
            ExprKind::Char(_) => Ty::Char,
            ExprKind::Bool(_) => Ty::Bool,
            ExprKind::None => self.none_literal(expr.span),
            ExprKind::Str(pieces) => {
                for piece in pieces {
                    if let StrPiece::Expr(inner) | StrPiece::Spec(inner, _) = piece {
                        let ty = self.infer(inner);
                        self.require_show(inner, &ty, false, true);
                    }
                }
                Ty::Str
            }
            ExprKind::Name(name) => self.name_ty(name, expr.span),
            ExprKind::SelfValue => self.name_ty("self", expr.span),
            ExprKind::Cpp(_) | ExprKind::Intrinsic(_) => self.cpp_outside_body(expr.span),
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
                        self.report(
                            inner.span,
                            msgs::check::TYPE_MISMATCH_UNARY_NEEDS_A_NUMBER,
                            &[&self.quote(inner.span), &ty.with_article()],
                        );
                        Ty::Error
                    }
                    UnOp::Neg | UnOp::BitNot if ty.is_numeric() => ty.widen(),
                    UnOp::Neg if matches!(&ty, Ty::Optional(inner) if inner.is_numeric()) => ty,
                    UnOp::BitNot if matches!(&ty, Ty::Optional(inner) if inner.is_int()) => ty,
                    UnOp::Neg if let Some(neg) = self.neg_impl(&ty) => neg,
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
                if let Some(ty) = self.keyed_find(expr, first, rest) {
                    return ty;
                }
                // `f<A>(x)` is two comparisons: a fn's type arguments are
                // written `f::<A>(x)`.
                if let ExprKind::Name(name) = &first.kind {
                    let generic = self.local(name).is_none()
                        && matches!(
                            self.global(name).as_deref(),
                            Some(Def::Fn { generics, .. }) if !generics.is_empty()
                        );
                    if generic {
                        if let Some(diag) = self.report(
                            expr.span,
                            msgs::check::BAD_OPERANDS_IS_A_GENERIC_FN,
                            &[&name, &name],
                        ) {
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
                        if rest.len() == 1 && self.lacks_contains(&right) {
                            self.list_in(*op, left_expr, right_expr, &right, true);
                        }
                    } else {
                        self.comparable(*op, tests_none, &left, left_expr, &right, right_expr);
                    }
                    left = right;
                    left_expr = right_expr;
                }
                // §130: chained comparisons must point in one direction and cannot contain `==`, `!=` or `in`.
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
                    let has_in = rest
                        .iter()
                        .any(|(op, _)| matches!(op, ast::CmpOp::In | ast::CmpOp::NotIn));
                    if has_in {
                        if let Some(diag) = self.report(
                            expr.span,
                            msgs::check::MIXED_COMPARISON_CHAIN_CHAINED_COMPARISONS_CANNOT_CONTAIN_IN,
                            &[],
                        ) {
                            diag.help = Some("join the comparisons with `and` instead".to_string());
                            diag.fixes.extend(split);
                        }
                    } else if has_eq {
                        if let Some(diag) = self.report(
                            expr.span,
                            msgs::check::MIXED_COMPARISON_CHAIN_CHAINED_COMPARISONS_CANNOT_CONTAIN,
                            &[],
                        ) {
                            diag.help = Some("join the comparisons with `and` instead".to_string());
                            diag.fixes.extend(split);
                        }
                    } else if has_lt && has_gt {
                        if let Some(diag) = self.report(
                            expr.span,
                            msgs::check::MIXED_COMPARISON_CHAIN_CHAINED_COMPARISONS_MUST_POINT,
                            &[],
                        ) {
                            diag.help =
                                Some("split mixed `<` and `>` comparisons with `and`".to_string());
                            diag.fixes.extend(split);
                        }
                    }
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
                    Some(Ty::Pos(d)) => d
                        .strip_suffix(".cuts")
                        .map(|base| (d.clone(), base.to_string())),
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
                        let both_cuts =
                            bounds_domain(&lo_ty).is_some() && bounds_domain(&hi_ty).is_some();
                        if self.expected_bounds_range
                            || (*inclusive || !self.expected_piece)
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
                    (_, _, Some(Ty::Float), Some(Ty::Float)) => Ty::Float, // an interval (decision 419)
                    _ => Ty::Unknown,
                };
                self.range_ends(expr, hi.as_deref(), *inclusive, &lo_ty, &hi_ty);
                range_of(elem)
            }
            ExprKind::By { walk, step } => {
                let step_ty = self.infer(step);
                self.expect_fits(&step_ty, &Ty::Int, step, &|c, got, _| {
                    cpl_diag::msgs::check::STEP_INT.at(&[&c.quote(step.span), &got.with_article()])
                });
                self.infer(walk)
            }
            ExprKind::Call { callee, args } => {
                if let ExprKind::Field { base, .. } = &callee.kind {
                    self.method_call_on_param(base);
                }
                self.full_ranges(args.iter().map(|a| &a.value));
                self.lent_receiver(callee);
                let outer = self.callee.replace(callee.span);
                let ty = self.call(expr, callee, args);
                self.callee = outer;
                self.member_for_call(expr, callee, args);
                ty
            }
            ExprKind::Index { base, indices } => self.index(expr, base, indices),
            ExprKind::Field { base, name } => self.field(expr, base, name),
            ExprKind::Generic { base, args } => {
                let base_ty = self.infer(base);
                let args: Vec<Ty> = args.iter().map(|a| self.type_arg(a)).collect();
                // `Prefix<R, C, T>` beside `Prefix<D, T>`: the struct of that arity (R02.11.4).
                if let ExprKind::Name(name) = &base.kind {
                    if self.local(name).is_none() {
                        let key = crate::prelude::arity_key(name, args.len());
                        if self.global(&key).is_some() {
                            return Ty::Named { name: key, args };
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
                            self.report(
                                expr.span,
                                msgs::check::WRONG_ARG_COUNT_TAKES_BUT_GIVEN,
                                &[
                                    &self.quote(base.span),
                                    &plural(generics.len(), "type argument", "type arguments"),
                                    &if args.len() == 1 {
                                        "1 was".to_string()
                                    } else {
                                        format!("{} were", args.len())
                                    },
                                ],
                            );
                            return Ty::Error;
                        }
                        let bindings = generics.into_iter().zip(args).collect();
                        ty.subst(&bindings)
                    }
                    // `Box<Int>`: defaults fill the arguments not given (R08.2.5).
                    (Ty::TypeName(name), None) => match self.type_def(&name) {
                        Some(def) if matches!(*def, Def::Struct { .. }) => {
                            self.def_type(&def, &name, expr.span, args)
                        }
                        _ => Ty::Named { name, args },
                    },
                    (other, None) => {
                        let named = match &base.kind {
                            ExprKind::Name(name) => Some(name),
                            _ => None,
                        };
                        match named {
                            Some(name)
                                if !matches!(other, Ty::Error | Ty::Unknown)
                                    && self.type_def(name).is_none() =>
                            {
                                self.report(
                                    expr.span,
                                    msgs::check::TYPE_MISMATCH_TYPE_ARGS_WITHOUT_PARAMS,
                                    &[name],
                                );
                                Ty::Error
                            }
                            _ => other,
                        }
                    }
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
                        self.report(
                            name.span,
                            msgs::check::DUPLICATE_DEFINITION_DUPLICATE_FIELD_IN_RECORD,
                            &[&name.text],
                        );
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
                    let elem = self.walk_elem_ty(&clause.iter, &iter_ty);
                    self.bind_whole(&clause.pat, &elem, false, "for");
                    for cond in &clause.conds {
                        self.cond(cond);
                    }
                }
                let body_ty = self.comprehension_body(body, clauses);
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
                // One parameter over a product is the cell tuple (R04.2.11).
                let cell = (params.len() == 1 && index.len() > 1).then(|| Ty::Tuple(index.clone()));
                for (i, param) in params.iter().enumerate() {
                    let ty = cell.clone().or_else(|| index.get(i).cloned());
                    self.bind(param, &ty.unwrap_or(Ty::Unknown), false, "param");
                    if cell.is_none() && domains.get(i).is_some_and(Self::names_range_keys) {
                        self.note_range_key(param);
                    }
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
            ExprKind::Lambda { .. } => self.lambda(expr),
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
                    self.begin_cond_hold(cond, body);
                    let past = self.present.len();
                    let held = narrow::in_branch(cond, body, &mut |f, x| self.call_writes(f, x));
                    self.present.extend(held.into_iter().map(String::from));
                    self.branch += value;
                    let ty = self.tested_block(cond, body);
                    self.branch -= value;
                    if !leaves(body) {
                        self.keep_moves(&mut post_moved);
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
                            self.keep_moves(&mut post_moved);
                        }
                        match result {
                            Some(first) if first.is_vague() => ty,
                            // `if c: 0 else: z` with `z: Mod<P>` is a `Mod<P>` (decision 3.10).
                            Some(first)
                                if first.is_int()
                                    && matches!(&ty, Ty::Named { name, .. } if self.is_number(name)) =>
                            {
                                ty
                            }
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
                    let arm_leaves = leaves(&arm.body);
                    if !arm_leaves {
                        self.keep_moves(&mut post_moved);
                    }
                    self.pop_scope();
                    // The value is the first arm's that has one: an arm
                    // that leaves (`Skip => continue`) has none.
                    if result.is_none() && !arm_leaves {
                        result = Some(ty.widen());
                    }
                }
                self.moved = post_moved;
                let result = result.unwrap_or(Ty::Unit);
                self.require_reachable_arms(&scrutinee_ty, arms);
                self.require_every_variant(&scrutinee_ty, arms, expr.span);
                self.require_every_lit_union(scrutinee, arms, expr.span);
                result
            }
            ExprKind::With { base, updates } => {
                let ty = self.infer(base);
                for (name, value) in updates {
                    let value_ty = self.infer(value);
                    if let Some(field_ty) = self.field_of(&ty, name) {
                        self.expect_fits(&value_ty, &field_ty, value, &|c, got, want| {
                            cpl_diag::msgs::check::BUT.at(&[
                                &name.text,
                                &want.with_article(),
                                &c.quote(value.span),
                                &got.with_article(),
                            ])
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
            ExprKind::HasBound { bound, .. } => {
                self.trait_exists(bound);
                Ty::Bool
            }
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
                                    self.report(
                                        r.span,
                                        msgs::check::UNKNOWN_FIELD_ROW_HAS_NO_NAMED,
                                        &[&p, &r.text],
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
                if let Some(diag) = self.report(
                    expr.span,
                    msgs::check::BAD_OPERANDS_IS_INSIDE_A_BRANCH,
                    &[&inner_text],
                ) {
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
                other => Self::unwrapped_cells(other),
            },
            ExprKind::Missing | ExprKind::Error => Ty::Error,
        }
    }

    /// After a branch that falls through, what it moved stays moved.
    fn keep_moves(&mut self, post_moved: &mut crate::names::NameMap<Span>) {
        for (k, v) in self.moved.clone() {
            if self.declared_before(&k, u32::MAX) {
                post_moved.entry(k).or_insert(v);
            }
        }
    }

    /// A `match` on an enum whose variants hold nothing has an arm for
    /// each of them, or one that takes whatever is left.
    fn require_every_variant(&mut self, scrutinee: &Ty, arms: &[cpl_syntax::ast::Arm], span: Span) {
        use cpl_syntax::ast::{Pattern, PatternKind};
        let Ty::Named { name, .. } = scrutinee else {
            return;
        };
        let found = self.type_def(name);
        let Some(Def::Enum { variants, .. }) = found.as_deref() else {
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
            if named(&arm.pat, variants, &mut seen).is_none() {
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
        if let Some(diag) = self.report(
            span,
            msgs::check::NON_EXHAUSTIVE_MATCH_THIS_MATCH_HAS_NO,
            &[&missing.join(", ")],
        ) {
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
        if let Some(diag) = self.report(
            pat.span,
            msgs::check::TYPE_MISMATCH_IS_A_VARIANT_BUT,
            &[&name, &ty.widen()],
        ) {
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
        if let Some(diag) = self.report(
            span,
            msgs::check::NON_EXHAUSTIVE_MATCH_THIS_MATCH_HAS_NO,
            &[&missing.join(", ")],
        ) {
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
        let help = format!(
            "say what `None` means: `({}) ?? false`",
            self.quote(expr.span)
        );
        let replacement = format!("({}) ?? false", self.quote(expr.span));
        if let Some(diag) = self.report(
            at,
            msgs::check::QUESTION_IN_CONDITION_IS_A_IN_A,
            &[&self.quote(at)],
        ) {
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
                    if let Some(try_span) = expr.bare_try() {
                        let message = cpl_diag::msgs::check::CANNOT_USED_DIRECTLY_CONDITION.at(&[]);
                        let replacement = format!("({}) ?? false", self.quote(expr.span));
                        if let Some(diag) = self.report_said(try_span, message) {
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
                    if let Some(try_span) = test.bare_try() {
                        let message = cpl_diag::msgs::check::CANNOT_USED_DIRECTLY_CONDITION.at(&[]);
                        let replacement = format!("({}) ?? false", self.quote(test.span));
                        if let Some(diag) = self.report_said(try_span, message) {
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
                let alias = matches!(&value.kind, ExprKind::Name(n)
                    if self.local(n).is_some_and(|l| self.opt_mut_params.contains(&l.span)));
                self.bind(pat, &ty, alias, "let");
                if let Some(and) = and {
                    self.cond(and);
                }
            }
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
}

impl Checker<'_> {}

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

/// What a struct or enum, with its impls, lets a value reach by name.
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

impl Checker<'_> {}

impl Checker<'_> {}

impl Checker<'_> {}

impl Checker<'_> {}

fn range_ty(of: Ty) -> Ty {
    Ty::Named {
        name: "Range".into(),
        args: vec![of],
    }
}

/// `fn(i: Int) -> ret`: a domain's member from an ordinal or label.
fn int_fn(ret: Ty) -> Ty {
    Ty::Fn {
        params: vec![crate::ty::Param {
            name: "i".into(),
            ty: Ty::Int,
            mutable: false,
            has_default: false,
            named_only: false,
            variadic: false,
            lazy: false,
        }],
        ret: Box::new(ret),
    }
}

/// Whether `body` ends by leaving: `return`, `break` or `continue`.
fn leaves(body: &ast::Block) -> bool {
    matches!(
        body.stmts.last().map(|s| &s.kind),
        Some(ast::StmtKind::Return(_) | ast::StmtKind::Break(_) | ast::StmtKind::Continue(_))
    )
}

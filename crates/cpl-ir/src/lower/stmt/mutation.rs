//! What a statement or expression mutates: the names it writes, the slices
//! it takes `mut`, the methods that change their receiver.

use super::*;

impl<'a> Lowerer<'a> {
    pub(in crate::lower) fn is_mut_method_name(name: &str) -> bool {
        matches!(
            name,
            "sort"
                | "reverse"
                | "rotate_left"
                | "rotate_right"
                | "scan_in_place"
                | "suffix_scan_in_place"
                | "next_permutation"
                | "prev_permutation"
                | "nth_element"
                | "swap"
                | "swap_at"
                | "swap_idx"
                | "set"
                | "set_at"
                | "fill"
                | "shuffle"
        )
    }

    pub(in crate::lower) fn collect_mut_slices_expr(&mut self, e: &ast::Expr) {
        match &e.kind {
            ast::ExprKind::Call { callee, args, .. } => {
                for a in args {
                    if a.mutable && matches!(a.value.kind, ast::ExprKind::Index { .. }) {
                        let sp = self.span(a.value.span);
                        self.mut_slice_inits.insert(sp);
                    }
                    self.collect_mut_slices_expr(&a.value);
                }
                if let ast::ExprKind::Field { base, name } = &callee.kind {
                    if matches!(base.kind, ast::ExprKind::Index { .. })
                        && Self::is_mut_method_name(&name.text)
                    {
                        let sp = self.span(base.span);
                        self.mut_slice_inits.insert(sp);
                    }
                }
                self.collect_mut_slices_expr(callee);
            }
            ast::ExprKind::Paren(inner) | ast::ExprKind::Unary { expr: inner, .. } => {
                self.collect_mut_slices_expr(inner);
            }
            ast::ExprKind::Binary { lhs, rhs, .. } => {
                self.collect_mut_slices_expr(lhs);
                self.collect_mut_slices_expr(rhs);
            }
            ast::ExprKind::Index { base, indices } => {
                self.collect_mut_slices_expr(base);
                for i in indices {
                    self.collect_mut_slices_expr(i);
                }
            }
            ast::ExprKind::Field { base, .. } => self.collect_mut_slices_expr(base),
            ast::ExprKind::Lambda { body, .. } => self.collect_mut_slices_expr(body),
            _ => {}
        }
    }

    pub(in crate::lower) fn collect_mut_slices_stmt(
        &mut self,
        stmt: &ast::Stmt,
        rest: &[ast::Stmt],
    ) {
        match &stmt.kind {
            ast::StmtKind::Let {
                pat,
                init: Some(init),
                ..
            } => {
                if let ast::PatternKind::Name(name) = &pat.kind {
                    if matches!(init.kind, ast::ExprKind::Index { .. })
                        && Self::stmts_mutate_name(rest, name)
                    {
                        let sp = self.span(init.span);
                        self.mut_slice_inits.insert(sp);
                    }
                }
                self.collect_mut_slices_expr(init);
            }
            ast::StmtKind::Assign { target, value, .. } => {
                if let ast::ExprKind::Index { base, .. } = &target.kind {
                    if matches!(base.kind, ast::ExprKind::Index { .. }) {
                        let sp = self.span(base.span);
                        self.mut_slice_inits.insert(sp);
                    }
                }
                self.collect_mut_slices_expr(target);
                self.collect_mut_slices_expr(value);
            }
            ast::StmtKind::Expr(e) | ast::StmtKind::Return(Some(e)) => {
                self.collect_mut_slices_expr(e);
            }
            _ => {}
        }
    }

    pub(in crate::lower) fn expr_root_name(e: &ast::Expr) -> Option<&str> {
        match &e.kind {
            ast::ExprKind::Name(n) => Some(n.as_str()),
            ast::ExprKind::Paren(inner) => Self::expr_root_name(inner),
            ast::ExprKind::Index { base, .. } | ast::ExprKind::Field { base, .. } => {
                Self::expr_root_name(base)
            }
            _ => None,
        }
    }

    pub(in crate::lower) fn cond_mutates_name(c: &ast::Cond, name: &str) -> bool {
        match c {
            ast::Cond::Expr(e) | ast::Cond::Const(e) => Self::expr_mutates_name(e, name),
            ast::Cond::Let { value, .. } => Self::expr_mutates_name(value, name),
            ast::Cond::Guard { test, and } => {
                Self::expr_mutates_name(test, name) || Self::cond_mutates_name(and, name)
            }
        }
    }

    pub(in crate::lower) fn expr_mutates_name(e: &ast::Expr, name: &str) -> bool {
        match &e.kind {
            ast::ExprKind::Call { callee, args, .. } => {
                if args
                    .iter()
                    .any(|a| a.mutable && Self::expr_root_name(&a.value) == Some(name))
                {
                    return true;
                }
                if let ast::ExprKind::Field { base, name: method } = &callee.kind {
                    if Self::expr_root_name(base) == Some(name)
                        && matches!(
                            method.text.as_str(),
                            "sort"
                                | "reverse"
                                | "rotate_left"
                                | "rotate_right"
                                | "scan_in_place"
                                | "suffix_scan_in_place"
                                | "next_permutation"
                                | "prev_permutation"
                                | "nth_element"
                                | "swap"
                                | "swap_at"
                                | "swap_idx"
                                | "set"
                                | "set_at"
                                | "fill"
                                | "shuffle"
                        )
                    {
                        return true;
                    }
                }
                Self::expr_mutates_name(callee, name)
                    || args.iter().any(|a| Self::expr_mutates_name(&a.value, name))
            }
            ast::ExprKind::Block(b) => Self::stmts_mutate_name(&b.stmts, name),
            ast::ExprKind::If {
                branches,
                otherwise,
            } => {
                branches.iter().any(|(c, b)| {
                    Self::cond_mutates_name(c, name) || Self::stmts_mutate_name(&b.stmts, name)
                }) || otherwise
                    .as_ref()
                    .is_some_and(|b| Self::stmts_mutate_name(&b.stmts, name))
            }
            ast::ExprKind::Match { scrutinee, arms } => {
                Self::expr_mutates_name(scrutinee, name)
                    || arms.iter().any(|a| {
                        a.guard
                            .as_ref()
                            .is_some_and(|g| Self::expr_mutates_name(g, name))
                            || Self::stmts_mutate_name(&a.body.stmts, name)
                    })
            }
            ast::ExprKind::Paren(inner) | ast::ExprKind::Unary { expr: inner, .. } => {
                Self::expr_mutates_name(inner, name)
            }
            ast::ExprKind::Binary { lhs, rhs, .. } => {
                Self::expr_mutates_name(lhs, name) || Self::expr_mutates_name(rhs, name)
            }
            ast::ExprKind::Index { base, indices } => {
                Self::expr_mutates_name(base, name)
                    || indices.iter().any(|i| Self::expr_mutates_name(i, name))
            }
            ast::ExprKind::Field { base, .. } => Self::expr_mutates_name(base, name),
            ast::ExprKind::Lambda { body, .. } => Self::expr_mutates_name(body, name),
            _ => false,
        }
    }

    pub(in crate::lower) fn stmts_mutate_name(stmts: &[ast::Stmt], name: &str) -> bool {
        for (i, s) in stmts.iter().enumerate() {
            match &s.kind {
                ast::StmtKind::Assign { target, value, .. } => {
                    if Self::expr_root_name(target) == Some(name)
                        || Self::expr_mutates_name(target, name)
                        || Self::expr_mutates_name(value, name)
                    {
                        return true;
                    }
                }
                ast::StmtKind::Expr(e) => {
                    if Self::expr_mutates_name(e, name) {
                        return true;
                    }
                }
                ast::StmtKind::Let {
                    pat,
                    init: Some(init),
                    ..
                } => {
                    if Self::expr_mutates_name(init, name) {
                        return true;
                    }
                    if let ast::PatternKind::Name(sub) = &pat.kind {
                        if Self::expr_root_name(init) == Some(name)
                            && Self::stmts_mutate_name(&stmts[i + 1..], sub)
                        {
                            return true;
                        }
                        if sub == name {
                            return false;
                        }
                    }
                }
                ast::StmtKind::For { iter, body, .. } => {
                    if Self::expr_mutates_name(iter, name)
                        || Self::stmts_mutate_name(&body.stmts, name)
                    {
                        return true;
                    }
                }
                ast::StmtKind::While { cond, body, .. } => {
                    if Self::cond_mutates_name(cond, name)
                        || Self::stmts_mutate_name(&body.stmts, name)
                    {
                        return true;
                    }
                }
                ast::StmtKind::Return(Some(e)) if Self::expr_mutates_name(e, name) => {
                    return true;
                }
                _ => {}
            }
        }
        false
    }
}

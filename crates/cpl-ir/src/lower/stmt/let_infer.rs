//! Peeking ahead for the argument types a local fn is known by only
//! from later calls: `let f = w => ..` then `f("abc")`.

use super::*;

impl<'a> Lowerer<'a> {
    pub(in crate::lower) fn peek_fn_arg_types(
        &mut self,
        fn_name: &str,
        arity: usize,
        rest: &[ast::Stmt],
    ) -> Option<Vec<Ty>> {
        fn find_call_in_expr<'a>(
            fn_name: &str,
            arity: usize,
            e: &'a ast::Expr,
        ) -> Option<&'a [ast::Arg]> {
            match &e.kind {
                ast::ExprKind::Call { callee, args } => {
                    if matches!(&callee.kind, ast::ExprKind::Name(n) if n == fn_name)
                        && args.len() == arity
                    {
                        return Some(args);
                    }
                    if let Some(found) = find_call_in_expr(fn_name, arity, callee) {
                        return Some(found);
                    }
                    for a in args {
                        if let Some(found) = find_call_in_expr(fn_name, arity, &a.value) {
                            return Some(found);
                        }
                    }
                    None
                }
                ast::ExprKind::Field { base, .. }
                | ast::ExprKind::Paren(base)
                | ast::ExprKind::Unary { expr: base, .. }
                | ast::ExprKind::Unwrap(base) => find_call_in_expr(fn_name, arity, base),
                ast::ExprKind::Binary { lhs, rhs, .. } => find_call_in_expr(fn_name, arity, lhs)
                    .or_else(|| find_call_in_expr(fn_name, arity, rhs)),
                ast::ExprKind::List(items) | ast::ExprKind::Tuple(items) => items
                    .iter()
                    .find_map(|x| find_call_in_expr(fn_name, arity, x)),
                ast::ExprKind::If {
                    branches,
                    otherwise,
                } => {
                    for (cond, block) in branches {
                        if let ast::Cond::Expr(ce) = cond {
                            if let Some(found) = find_call_in_expr(fn_name, arity, ce) {
                                return Some(found);
                            }
                        }
                        if let Some(found) = find_call(fn_name, arity, &block.stmts) {
                            return Some(found);
                        }
                    }
                    otherwise
                        .as_ref()
                        .and_then(|b| find_call(fn_name, arity, &b.stmts))
                }
                ast::ExprKind::Block(b) => find_call(fn_name, arity, &b.stmts),
                _ => None,
            }
        }
        fn find_call<'a>(
            fn_name: &str,
            arity: usize,
            stmts: &'a [ast::Stmt],
        ) -> Option<&'a [ast::Arg]> {
            for s in stmts {
                match &s.kind {
                    ast::StmtKind::Let { init: Some(e), .. }
                    | ast::StmtKind::Expr(e)
                    | ast::StmtKind::Assign { value: e, .. }
                    | ast::StmtKind::Return(Some(e))
                    | ast::StmtKind::Assert { cond: e, .. } => {
                        if let Some(args) = find_call_in_expr(fn_name, arity, e) {
                            return Some(args);
                        }
                    }
                    ast::StmtKind::For { iter, body, .. } => {
                        if let Some(args) = find_call_in_expr(fn_name, arity, iter) {
                            return Some(args);
                        }
                        if let Some(args) = find_call(fn_name, arity, &body.stmts) {
                            return Some(args);
                        }
                    }
                    ast::StmtKind::While {
                        cond,
                        body,
                        else_body,
                    } => {
                        if let ast::Cond::Expr(e) = cond {
                            if let Some(args) = find_call_in_expr(fn_name, arity, e) {
                                return Some(args);
                            }
                        }
                        if let Some(args) = find_call(fn_name, arity, &body.stmts) {
                            return Some(args);
                        }
                        if let Some(eb) = else_body {
                            if let Some(args) = find_call(fn_name, arity, &eb.stmts) {
                                return Some(args);
                            }
                        }
                    }
                    _ => {}
                }
            }
            None
        }
        if let Some(args) = find_call(fn_name, arity, rest) {
            let mut tys = Vec::with_capacity(arity);
            for a in args {
                let ty = self.probe_ty_silent(&a.value)?;
                tys.push(ty);
            }
            return Some(tys);
        }
        // Also check when `fn_name` is passed as a callback argument to a typed function `outer(..., fn_name, ...)`.
        fn find_passed_in_expr<'a>(fn_name: &str, e: &'a ast::Expr) -> Option<(&'a str, usize)> {
            match &e.kind {
                ast::ExprKind::Call { callee, args } => {
                    if let ast::ExprKind::Name(cname) = &callee.kind {
                        for (idx, a) in args.iter().enumerate() {
                            if matches!(&a.value.kind, ast::ExprKind::Name(n) if n == fn_name) {
                                return Some((cname.as_str(), idx));
                            }
                            if let ast::ExprKind::Lambda { params, body, .. } = &a.value.kind {
                                if find_call_in_expr(fn_name, params.len(), body).is_some() {
                                    return Some((cname.as_str(), idx));
                                }
                            }
                        }
                    }
                    find_passed_in_expr(fn_name, callee).or_else(|| {
                        args.iter()
                            .find_map(|a| find_passed_in_expr(fn_name, &a.value))
                    })
                }
                ast::ExprKind::Binary { lhs, rhs, .. } => {
                    find_passed_in_expr(fn_name, lhs).or_else(|| find_passed_in_expr(fn_name, rhs))
                }
                ast::ExprKind::Compare { first, rest } => find_passed_in_expr(fn_name, first)
                    .or_else(|| {
                        rest.iter()
                            .find_map(|(_, r)| find_passed_in_expr(fn_name, r))
                    }),
                ast::ExprKind::Unary { expr, .. }
                | ast::ExprKind::Paren(expr)
                | ast::ExprKind::Try(expr) => find_passed_in_expr(fn_name, expr),
                ast::ExprKind::Tuple(items) | ast::ExprKind::List(items) => {
                    items.iter().find_map(|x| find_passed_in_expr(fn_name, x))
                }
                _ => None,
            }
        }
        fn find_passed<'a>(fn_name: &str, stmts: &'a [ast::Stmt]) -> Option<(&'a str, usize)> {
            for s in stmts {
                match &s.kind {
                    ast::StmtKind::Let { init: Some(e), .. }
                    | ast::StmtKind::Expr(e)
                    | ast::StmtKind::Assign { value: e, .. }
                    | ast::StmtKind::Return(Some(e))
                    | ast::StmtKind::Assert { cond: e, .. } => {
                        if let Some(found) = find_passed_in_expr(fn_name, e) {
                            return Some(found);
                        }
                    }
                    ast::StmtKind::For { iter, body, .. } => {
                        if let Some(found) = find_passed_in_expr(fn_name, iter)
                            .or_else(|| find_passed(fn_name, &body.stmts))
                        {
                            return Some(found);
                        }
                    }
                    ast::StmtKind::While {
                        cond,
                        body,
                        else_body,
                    } => {
                        if let ast::Cond::Expr(e) = cond {
                            if let Some(found) = find_passed_in_expr(fn_name, e) {
                                return Some(found);
                            }
                        }
                        if let Some(found) = find_passed(fn_name, &body.stmts) {
                            return Some(found);
                        }
                        if let Some(eb) = else_body {
                            if let Some(found) = find_passed(fn_name, &eb.stmts) {
                                return Some(found);
                            }
                        }
                    }
                    _ => {}
                }
            }
            None
        }
        let (outer_name, arg_idx) = find_passed(fn_name, rest)?;
        let Some(Sym::Fn(fid)) = self.get_sym(outer_name) else {
            return None;
        };
        if !self.ensure_sig(fid, None) {
            return None;
        }
        match &self.sigs[fid].params.get(arg_idx)?.ty {
            Ty::Fn(ps, _) if ps.len() == arity => Some(ps.clone()),
            _ => None,
        }
    }
}

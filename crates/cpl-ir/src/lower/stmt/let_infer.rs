//! Peeking ahead for the types a `let` leaves open: `List.filled(n, none)`,
//! empty maps, and local fn arguments known only from later calls.

use super::*;

impl<'a> Lowerer<'a> {
    /// Infers the full type of `var x = List.filled(..., none)` (or `var x = none`)
    /// from the enclosing function's return type or subsequent assignments in `rest`.
    pub(in crate::lower) fn peek_filled_none_type(
        &mut self,
        var_name: &str,
        depth: usize,
        rest: &[ast::Stmt],
    ) -> Option<Ty> {
        fn has_depth_opt(ty: &Ty, depth: usize) -> bool {
            let mut cur = ty;
            for _ in 0..depth {
                match cur {
                    Ty::List(inner) => cur = inner,
                    _ => return false,
                }
            }
            matches!(cur, Ty::Opt(_))
        }
        fn returns_var(var_name: &str, stmts: &[ast::Stmt]) -> bool {
            if let Some(last) = stmts.last() {
                if let ast::StmtKind::Expr(e) = &last.kind {
                    if matches!(&e.kind, ast::ExprKind::Name(n) if n == var_name) {
                        return true;
                    }
                }
            }
            for s in stmts {
                match &s.kind {
                    ast::StmtKind::Return(Some(e)) if matches!(&e.kind, ast::ExprKind::Name(n) if n == var_name) =>
                    {
                        return true;
                    }
                    ast::StmtKind::Expr(e) => {
                        if let ast::ExprKind::If {
                            branches,
                            otherwise,
                        } = &e.kind
                        {
                            if branches
                                .iter()
                                .any(|(_, b)| returns_var(var_name, &b.stmts))
                                || otherwise
                                    .as_ref()
                                    .is_some_and(|b| returns_var(var_name, &b.stmts))
                            {
                                return true;
                            }
                        }
                    }
                    ast::StmtKind::While { body, .. } | ast::StmtKind::For { body, .. }
                        if returns_var(var_name, &body.stmts) =>
                    {
                        return true;
                    }
                    _ => {}
                }
            }
            false
        }
        if let Some(ret_ty) = self.cx.ret.clone() {
            if has_depth_opt(&ret_ty, depth) && returns_var(var_name, rest) {
                return Some(ret_ty);
            }
        }
        fn index_depth_of(e: &ast::Expr, var_name: &str) -> Option<usize> {
            match &e.kind {
                ast::ExprKind::Name(n) if n == var_name => Some(0),
                ast::ExprKind::Paren(inner) => index_depth_of(inner, var_name),
                ast::ExprKind::Index { base, indices } if indices.len() == 1 => {
                    index_depth_of(base, var_name).map(|d| d + 1)
                }
                _ => None,
            }
        }
        fn collect_rhs<'a>(
            var_name: &str,
            depth: usize,
            stmts: &'a [ast::Stmt],
            out: &mut Vec<&'a ast::Expr>,
            decls: &mut Vec<(&'a str, &'a ast::Type)>,
        ) {
            for s in stmts {
                match &s.kind {
                    ast::StmtKind::Let {
                        pat:
                            ast::Pattern {
                                kind: ast::PatternKind::Name(n),
                                ..
                            },
                        ty: Some(t),
                        ..
                    } => decls.push((n.as_str(), t)),
                    ast::StmtKind::Assign { target, value, .. } => {
                        if index_depth_of(target, var_name) == Some(depth)
                            && !matches!(value.kind, ast::ExprKind::None)
                        {
                            out.push(value);
                        }
                    }
                    ast::StmtKind::Expr(e) => {
                        if depth == 1 {
                            if let ast::ExprKind::Call { callee, args } = &e.kind {
                                if let ast::ExprKind::Field { base, name } = &callee.kind {
                                    if name.text == "push"
                                        && matches!(&base.kind, ast::ExprKind::Name(n) if n == var_name)
                                        && args.len() == 1
                                        && !matches!(args[0].value.kind, ast::ExprKind::None)
                                    {
                                        out.push(&args[0].value);
                                    }
                                }
                            }
                        }
                        if let ast::ExprKind::If {
                            branches,
                            otherwise,
                        } = &e.kind
                        {
                            for (_, b) in branches {
                                collect_rhs(var_name, depth, &b.stmts, out, decls);
                            }
                            if let Some(b) = otherwise {
                                collect_rhs(var_name, depth, &b.stmts, out, decls);
                            }
                        }
                    }
                    ast::StmtKind::While { body, .. } | ast::StmtKind::For { body, .. } => {
                        collect_rhs(var_name, depth, &body.stmts, out, decls);
                    }
                    _ => {}
                }
            }
        }
        let mut candidates = Vec::new();
        let mut decls = Vec::new();
        collect_rhs(var_name, depth, rest, &mut candidates, &mut decls);
        for rhs in candidates {
            let elem = if let ast::ExprKind::Name(n) = &rhs.kind {
                if let Some((_, ty_ast)) = decls.iter().find(|(dn, _)| *dn == n.as_str()) {
                    let t = self.resolve_ty(ty_ast);
                    (t != Ty::Error).then_some(t)
                } else {
                    self.probe_ty_silent(rhs)
                }
            } else {
                self.probe_ty_silent(rhs)
            };
            if let Some(t) = elem {
                if t == Ty::Error || matches!(&t, Ty::Opt(inner) if **inner == Ty::Error) {
                    continue;
                }
                let mut full = match t {
                    Ty::Opt(_) => t,
                    other => Ty::opt(other),
                };
                for _ in 0..depth {
                    full = Ty::list(full);
                }
                return Some(full);
            }
        }
        None
    }

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

    pub(in crate::lower) fn peek_map_kv_types(
        &mut self,
        var_name: &str,
        rest: &[ast::Stmt],
    ) -> (Ty, Ty) {
        for s in rest {
            if let ast::StmtKind::Assign {
                target:
                    ast::Expr {
                        kind: ast::ExprKind::Index { base, indices },
                        ..
                    },
                value,
                ..
            } = &s.kind
            {
                if matches!(&base.kind, ast::ExprKind::Name(n) if n == var_name)
                    && indices.len() == 1
                {
                    let kt = self.checked_ty(&indices[0]).unwrap_or(Ty::Int);
                    let vt = self.checked_ty(value).unwrap_or(Ty::Int);
                    return (kt, vt);
                }
            }
        }
        (Ty::Int, Ty::Int)
    }
}

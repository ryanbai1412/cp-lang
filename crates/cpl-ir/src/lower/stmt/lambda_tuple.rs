//! `let (a, b) = (x => .., y => ..)` binds each lambda by itself.

use super::*;

impl<'a> Lowerer<'a> {
    /// Lowers a `let` of a tuple of lambdas as one `let` per name, so each
    /// lambda is called directly; false when `stmt` is not that shape.
    pub(super) fn let_lambda_tuple(
        &mut self,
        stmt: &ast::Stmt,
        rest: &[ast::Stmt],
        out: &mut Vec<Stmt>,
    ) -> bool {
        let ast::StmtKind::Let {
            mutable,
            pat,
            ty,
            init,
            ..
        } = &stmt.kind
        else {
            return false;
        };
        // `let (pre, suf) = ((i) => .., (i) => ..)`: each lambda is
        // bound by itself, so each is called directly rather than
        // through a `std::function` slot of a tuple. Making a
        // closure has no effect, so the order can't be seen, and
        // the names bound may not appear in the lambdas.
        if let (
            ast::PatternKind::Tuple(pats),
            None,
            Some(ast::Expr {
                kind: ast::ExprKind::Tuple(items),
                ..
            }),
        ) = (&pat.kind, ty, init)
        {
            let named = |p: &ast::Pattern| match &p.kind {
                ast::PatternKind::Name(n) => Some(n.clone()),
                _ => None,
            };
            let names: Option<Vec<String>> = pats.iter().map(named).collect();
            let unguarded = matches!(
                &stmt.kind,
                ast::StmtKind::Let {
                    guard: None,
                    otherwise: None,
                    ..
                }
            );
            if names.is_some_and(|names| {
                unguarded
                    && names.len() == items.len()
                    && items
                        .iter()
                        .all(|e| matches!(e.kind, ast::ExprKind::Lambda { .. }))
                    && !names
                        .iter()
                        .any(|n| items.iter().any(|e| expr_mentions_name(e, n)))
            }) {
                for (p, e) in pats.iter().zip(items) {
                    let one = ast::Stmt {
                        kind: ast::StmtKind::Let {
                            mutable: *mutable,
                            pat: p.clone(),
                            ty: None,
                            init: Some(e.clone()),
                            otherwise: None,
                            guard: None,
                        },
                        ..stmt.clone()
                    };
                    self.stmt_inner(&one, rest, out);
                }
                return true;
            }
        }
        false
    }
}

pub(super) fn expr_mentions_name(e: &ast::Expr, target: &str) -> bool {
    struct Find<'a> {
        target: &'a str,
        found: bool,
    }
    impl cpl_syntax::visit::Visitor for Find<'_> {
        fn expr(&mut self, expr: &ast::Expr) {
            if !self.found {
                if let ast::ExprKind::Name(n) = &expr.kind {
                    if n == self.target {
                        self.found = true;
                    }
                }
            }
        }
    }
    let mut f = Find {
        target,
        found: false,
    };
    cpl_syntax::visit::walk_expr(e, &mut f);
    f.found
}

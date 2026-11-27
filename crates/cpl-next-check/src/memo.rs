//! A `@memo` fn declared in a block.

use super::*;

impl Cx<'_> {
    /// E0311: a `@memo` fn's cache keeps results computed from what it captured, so nothing
    /// may write a captured `var` while the cache lives, to the end of its block (R06.4.1).
    pub(super) fn memo_writes(&mut self, f: &ast::FnDecl, rest: &[ast::Stmt]) {
        use cpl_syntax::visit::{walk_block, walk_expr, walk_stmt, Visitor};
        struct Names(Vec<String>);
        impl Visitor for Names {
            fn expr(&mut self, e: &ast::Expr) {
                if let ExprKind::Name(n) = &e.kind {
                    self.0.push(n.clone());
                }
            }
        }
        fn root(e: &ast::Expr) -> Option<&str> {
            match &e.kind {
                ExprKind::Name(n) => Some(n),
                ExprKind::Field { base, .. } | ExprKind::Index { base, .. } => root(base),
                ExprKind::Paren(x) => root(x),
                _ => None,
            }
        }
        struct Writes<'a> {
            names: &'a std::collections::HashSet<String>,
            at: Vec<(Span, String)>,
        }
        impl Writes<'_> {
            fn note(&mut self, e: &ast::Expr) {
                if let Some(n) = root(e).filter(|n| self.names.contains(*n)) {
                    self.at.push((e.span, n.to_string()));
                }
            }
        }
        impl Visitor for Writes<'_> {
            fn stmt(&mut self, s: &ast::Stmt) {
                if let StmtKind::Assign { target, .. } = &s.kind {
                    self.note(target);
                }
            }
            fn expr(&mut self, e: &ast::Expr) {
                if let ExprKind::Call { args, .. } = &e.kind {
                    for a in args.iter().filter(|a| a.mutable) {
                        self.note(&a.value);
                    }
                }
            }
        }
        let mut names = Names(Vec::new());
        match &f.body {
            FnBody::Block(b) => walk_block(b, &mut names),
            FnBody::Expr(e) => walk_expr(e, &mut names),
            FnBody::None => {}
        }
        let params: Vec<&str> = f.params.iter().map(|p| p.name.text.as_str()).collect();
        let captured: std::collections::HashSet<String> = (names.0.into_iter())
            .filter(|n| !params.contains(&n.as_str()))
            .filter(|n| match self.lookup(n) {
                Some((VarRef::Local(l), _)) => self.locals[l].mutable,
                Some((VarRef::Global(g), _)) => self.globals[g].mutable,
                None => false,
            })
            .collect();
        if captured.is_empty() {
            return;
        }
        let mut w = Writes {
            names: &captured,
            at: Vec::new(),
        };
        rest.iter().for_each(|s| walk_stmt(s, &mut w));
        for (span, n) in w.at {
            let text = format!(
                "`{n}` is written while the cache of `@memo` fn `{}`, which reads it, lives",
                f.name.text
            );
            self.err(span, Msg::verdict(cpl_diag::codes::BAD_MEMO, text));
        }
    }
}

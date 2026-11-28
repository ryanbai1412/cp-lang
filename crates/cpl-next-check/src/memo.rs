//! A `@memo` fn declared in a block.

use super::*;

impl Cx<'_> {
    /// E0311: a `@memo` fn's cache keeps results computed from what it captured, so nothing
    /// may write a captured `var` while the cache lives, to the end of its block (R06.4.1).
    pub(super) fn memo_writes(&mut self, f: &next::FnDecl, rest: &[next::Stmt]) {
        /// The name a place written through `e` starts from.
        fn root(e: &next::Expr) -> Option<String> {
            match e {
                next::Expr::Name(n) => n.ident().map(|t| t.text().to_string()),
                next::Expr::Field(x) => root(&x.base()?),
                next::Expr::Index(x) => root(&x.base()?),
                next::Expr::Paren(x) => root(&x.inner()?),
                _ => None,
            }
        }
        let body = f.body().map(|b| match b {
            next::FnBody::Block(b) => b.syntax().clone(),
            next::FnBody::Expr(e) => e.syntax().clone(),
        });
        let names: Vec<String> = (body.iter().flat_map(|b| b.descendants()))
            .filter_map(next::NameRef::cast)
            .filter_map(|n| n.ident())
            .map(|t| t.text().to_string())
            .collect();
        let params: Vec<String> = (f.params().iter().flat_map(next::ParamList::params))
            .filter_map(|p| p.name())
            .map(|t| t.text().to_string())
            .collect();
        let captured: std::collections::HashSet<String> = (names.into_iter())
            .filter(|n| !params.contains(n))
            .filter(|n| match self.lookup(n) {
                Some((VarRef::Local(l), _)) => self.b.locals[l].mutable,
                Some((VarRef::Global(g), _)) => self.globals[g].mutable,
                None => false,
            })
            .collect();
        if captured.is_empty() {
            return;
        }
        // Each place written below: an assignment's, a `min=`'s, or a `mut` argument's.
        let mut written: Vec<next::Expr> = Vec::new();
        for n in rest.iter().flat_map(|s| s.syntax().descendants()) {
            if let Some(a) = next::AssignStmt::cast(n.clone()) {
                written.extend(a.target());
            } else if let Some(k) = next::KeepExpr::cast(n.clone()) {
                written.extend(k.target());
            } else if let Some(c) = next::CallExpr::cast(n) {
                let args = c
                    .args()
                    .into_iter()
                    .flat_map(|l| l.args().collect::<Vec<_>>());
                written.extend(args.filter(next::Arg::is_mut).filter_map(|a| a.value()));
            }
        }
        let fname = f.name().map_or_else(String::new, |t| t.text().to_string());
        for e in written {
            let Some(n) = root(&e).filter(|n| captured.contains(n)) else {
                continue;
            };
            let text = format!(
                "`{n}` is written while the cache of `@memo` fn `{fname}`, which reads it, lives"
            );
            let span = self.at(&e);
            self.err(span, Msg::verdict(cpl_diag::codes::BAD_MEMO, text));
        }
    }
}

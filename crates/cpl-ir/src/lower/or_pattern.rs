//! `|` patterns that bind names, and `if let` patterns tested against the whole value.

use super::*;

impl<'a> Lowerer<'a> {
    /// `p | q` matches when an alternative does. A name every alternative binds is declared
    /// once, holding what the first alternative that matched bound it to (R05.3).
    pub(super) fn or_pattern(
        &mut self,
        alts: &[ast::Pattern],
        value: &Expr,
        binds: &mut Vec<Stmt>,
        span: Span,
    ) -> Option<Expr> {
        let outer = self.cx.or_binds.take();
        let mut arms = Vec::new();
        for alt in alts {
            self.cx.or_binds = Some(Vec::new());
            let test = self.pattern(alt, value, binds);
            arms.push((test, self.cx.or_binds.take().unwrap_or_default()));
        }
        self.cx.or_binds = outer;
        let names: Vec<String> = arms.first().map_or(
            Vec::new(),
            |(_, b): &(Option<Expr>, Vec<(String, Expr)>)| {
                b.iter().map(|(n, _)| n.clone()).collect()
            },
        );
        for name in names {
            let mut init: Option<Expr> = None;
            for (test, bound) in arms.iter().rev() {
                let Some((_, e)) = bound.iter().find(|(n, _)| *n == name) else {
                    continue;
                };
                init = Some(match (test, init) {
                    (Some(test), Some(rest)) => Expr {
                        kind: ExprKind::If(
                            Box::new(test.clone()),
                            Box::new(e.clone()),
                            Box::new(rest),
                        ),
                        ty: e.ty.clone(),
                        span,
                    },
                    _ => e.clone(),
                });
            }
            if let Some(init) = init {
                self.bind_name(&name, init, binds);
            }
        }
        let mut test: Option<Expr> = None;
        for (one, _) in arms {
            let Some(one) = one else {
                return None;
            };
            test = Some(match test {
                None => one,
                Some(prev) => Expr {
                    kind: ExprKind::Binary(BinOp::Or, Box::new(prev), Box::new(one)),
                    ty: Ty::Bool,
                    span,
                },
            });
        }
        test
    }

    /// A name a pattern binds: declared, or kept for the enclosing `|` pattern.
    pub(super) fn bind_name(&mut self, name: &str, init: Expr, binds: &mut Vec<Stmt>) {
        match &mut self.cx.or_binds {
            Some(bound) => bound.push((name.to_string(), init)),
            None => self.declare(name, init, false, binds),
        }
    }

    /// `if let pat = value` with `pat` tested against the whole value, held in a temporary.
    pub(super) fn whole_cond(
        &mut self,
        pat: &ast::Pattern,
        value: &ast::Expr,
        before: &mut Vec<Stmt>,
        binds: &mut Vec<Stmt>,
    ) -> Option<Expr> {
        let value_expr = self.expr(value, None);
        if value_expr.ty == Ty::Error {
            return None;
        }
        let tmp = self.temp(value_expr.ty.clone());
        before.push(Stmt::Let(tmp, Some(value_expr)));
        let local = self.local(tmp, value.span);
        let test = self.pattern(pat, &local, binds);
        Some(test.unwrap_or_else(|| self.mk(ExprKind::Bool(true), Ty::Bool, value.span)))
    }
}

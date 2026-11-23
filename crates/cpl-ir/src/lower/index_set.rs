//! W8 `IndexSet`: on a struct whose only place protocol is `set`, `x[k] = v` is `x.set(k, v)`
//! and `x[k] op= v` is `x.set(k, x[k] op v)`, with `k` evaluated once.

use super::*;

impl<'a> Lowerer<'a> {
    pub(super) fn index_set(
        &mut self,
        target: &ast::Expr,
        op: ast::AssignOp,
        value: &ast::Expr,
        out: &mut Vec<Stmt>,
    ) -> bool {
        let ast::ExprKind::Index { base, indices } = &target.kind else {
            return false;
        };
        let [key] = indices.as_slice() else {
            return false;
        };
        let Some(Ty::Struct(id)) = self.probe_ty_silent(base) else {
            return false;
        };
        let s = &self.structs[id];
        let has = |m: &str| s.methods.contains_key(m) || s.gen_methods.contains_key(m);
        if !has("set") || has("at_mut") {
            return false;
        }
        let span = target.span;
        let named = |f: &str, args: Vec<ast::Expr>| ast::Expr {
            kind: ast::ExprKind::Call {
                callee: Box::new(ast::Expr {
                    kind: ast::ExprKind::Name(f.into()),
                    span,
                }),
                args: args.into_iter().map(arg).collect(),
            },
            span,
        };
        let stored = match op {
            ast::AssignOp::Set => value.clone(),
            _ if !pure(key) => {
                let message =
                    "this key would be computed twice for `set`: bind it with `let` first";
                self.error(codes::NOT_INDEXABLE, key.span, message.to_string());
                return true;
            }
            ast::AssignOp::Binary(bin) => ast::Expr {
                kind: ast::ExprKind::Binary {
                    op: bin,
                    op_span: span,
                    lhs: Box::new(target.clone()),
                    rhs: Box::new(value.clone()),
                },
                span,
            },
            ast::AssignOp::Max => named("max", vec![target.clone(), value.clone()]),
            ast::AssignOp::Min => named("min", vec![target.clone(), value.clone()]),
        };
        let call = ast::Expr {
            kind: ast::ExprKind::Call {
                callee: Box::new(ast::Expr {
                    kind: ast::ExprKind::Field {
                        base: base.clone(),
                        name: ast::Name {
                            text: "set".into(),
                            span,
                        },
                    },
                    span,
                }),
                args: vec![arg(key.clone()), arg(stored)],
            },
            span,
        };
        let e = self.expr(&call, Some(&Ty::Unit));
        out.push(Stmt::Expr(e));
        true
    }
}

fn arg(value: ast::Expr) -> ast::Arg {
    ast::Arg {
        name: None,
        mutable: false,
        value,
    }
}

fn pure(e: &ast::Expr) -> bool {
    match &e.kind {
        ast::ExprKind::Name(_) | ast::ExprKind::Int(_) => true,
        ast::ExprKind::Field { base, .. } => pure(base),
        _ => false,
    }
}

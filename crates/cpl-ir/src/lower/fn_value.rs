//! A generic fn named as a value (`Heap.new(by: desc)`) is the closure that calls it,
//! `x => desc(x)`, so its type arguments come from wherever the closure's do.

use super::mono::GenFnId;
use super::*;

impl<'a> Lowerer<'a> {
    pub(super) fn gen_fn_as_closure(
        &mut self,
        gfid: GenFnId,
        name: &str,
        hint: Option<&Ty>,
        span: Span,
    ) -> Expr {
        let params: Vec<String> = self.gen_fns[gfid]
            .decl
            .params
            .iter()
            .filter(|p| p.default.is_none() && !p.variadic)
            .enumerate()
            .map(|(i, _)| format!("__{name}{i}"))
            .collect();
        let at = |kind| ast::Expr { kind, span };
        let body = at(ast::ExprKind::Call {
            callee: Box::new(at(ast::ExprKind::Name(name.into()))),
            args: params
                .iter()
                .map(|p| ast::Arg {
                    name: None,
                    mutable: false,
                    value: at(ast::ExprKind::Name(p.clone())),
                })
                .collect(),
        });
        let lambda = at(ast::ExprKind::Lambda {
            params: params
                .iter()
                .map(|p| ast::Pattern {
                    kind: ast::PatternKind::Name(p.clone()),
                    span,
                })
                .collect(),
            param_tys: vec![None; params.len()],
            param_mut: Vec::new(),
            ret_ty: None,
            body: Box::new(body),
        });
        self.expr(&lambda, hint)
    }
}

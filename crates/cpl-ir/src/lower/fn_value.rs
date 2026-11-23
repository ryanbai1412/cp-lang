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

impl<'a> Lowerer<'a> {
    /// `if A: Weighted:` on a type with no `impl`: met by shape when every member the trait
    /// leaves without a body is a field of `A` (a record arc `(to: V, w: Int)` is `Weighted`).
    pub(super) fn has_bound_by_shape(&mut self, param: &str, trait_name: &str) -> bool {
        let Some(GenArg::Ty(ty @ (Ty::Record(_) | Ty::Tuple(_)))) =
            self.gen_env.get(param).cloned()
        else {
            return false;
        };
        let Some(Sym::Trait(tid)) = self.get_sym(trait_name) else {
            return false;
        };
        let needed: Vec<String> = self.traits[tid]
            .methods
            .iter()
            .filter(|m| matches!(m.body, ast::FnBody::None))
            .map(|m| m.name.text.clone())
            .collect();
        let info = &self.traits[tid];
        let (m, offset, generic, fields) =
            (info.mod_id, info.offset, info.generic, info.fields.clone());
        let fields_met = fields.iter().all(|&(name, want)| {
            let Some((_, got)) = self.field_of(&ty, name) else {
                return false;
            };
            if generic {
                return true;
            }
            let saved = (
                std::mem::replace(&mut self.cur_mod, m),
                std::mem::replace(&mut self.offset, offset),
            );
            let want = self.resolve_ty(want);
            (self.cur_mod, self.offset) = saved;
            got == want
        });
        !(needed.is_empty() && fields.is_empty())
            && fields_met
            && needed.iter().all(|m| self.field_of(&ty, m).is_some())
    }
}

/// A trait body's methods, and the fields (`w: Int`) a record meets it with.
pub(super) fn trait_members(body: &ast::Block) -> (Vec<&ast::FnDecl>, Vec<(&str, &ast::Type)>) {
    let (mut methods, mut fields) = (Vec::new(), Vec::new());
    for s in &body.stmts {
        match &s.kind {
            ast::StmtKind::Fn(f) => methods.push(f),
            ast::StmtKind::Let {
                pat:
                    ast::Pattern {
                        kind: ast::PatternKind::Name(n),
                        ..
                    },
                ty: Some(t),
                init: None,
                ..
            } => fields.push((n.as_str(), t)),
            _ => {}
        }
    }
    (methods, fields)
}

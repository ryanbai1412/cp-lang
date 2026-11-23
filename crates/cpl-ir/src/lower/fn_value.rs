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
        let lambda = self.gen_fn_closure_ast(gfid, name, span);
        self.expr(&lambda, hint)
    }

    /// A generic call's argument that names a generic fn for a fn parameter is
    /// that closure, so the call binds its parameters as it does a lambda's:
    /// `ap(v, desc)` with `by: T -> K` is `desc` at `T`.
    pub(super) fn gen_fn_values_as_lambdas(
        &mut self,
        params: &[&ast::Param],
        args: &[ast::Arg],
    ) -> Option<Vec<ast::Arg>> {
        let positional: Vec<&&ast::Param> = params.iter().filter(|p| !p.named_only).collect();
        let mut next = 0;
        let mut changed = false;
        let out = args
            .iter()
            .map(|arg| {
                let param = match &arg.name {
                    Some(n) => params.iter().find(|p| p.label_text() == n.text),
                    None => {
                        next += 1;
                        positional.get(next - 1).copied()
                    }
                };
                let mut arg = arg.clone();
                let ast::ExprKind::Name(name) = &arg.value.kind else {
                    return arg;
                };
                let name = name.clone();
                let fn_slot = param.and_then(|p| p.ty.as_ref()).is_some_and(|t| {
                    matches!(
                        lambda_infer::unwrap_optional(t).kind,
                        ast::TypeKind::Fn { .. }
                    )
                });
                if fn_slot && self.lookup(&name).is_none() {
                    if let Some(Sym::GenFn(gfid)) = self.get_sym(&name) {
                        arg.value = self.gen_fn_closure_ast(gfid, &name, arg.value.span);
                        changed = true;
                    }
                }
                arg
            })
            .collect();
        changed.then_some(out)
    }

    fn gen_fn_closure_ast(&mut self, gfid: GenFnId, name: &str, span: Span) -> ast::Expr {
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
        at(ast::ExprKind::Lambda {
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
        })
    }
}

impl<'a> Lowerer<'a> {
    /// `if A: Weighted:` on a type with no `impl`: met by shape when every member the trait
    /// leaves without a body is a field of `A` (a record arc `(to: V, w: Int)` is `Weighted`).
    /// `if T: Tr`: `T` has an `impl Tr` (for its own struct, or the generic struct it instantiates),
    /// or is a record or tuple with `Tr`'s shape.
    pub(super) fn has_bound(&mut self, param: &str, trait_name: &str) -> bool {
        let by_impl = match self.gen_env.get(param) {
            Some(GenArg::Ty(Ty::Struct(sid))) => self.impls.iter().any(|imp| {
                let ast::TypeKind::Named { name, .. } = &imp.target.kind else {
                    return false;
                };
                let origin = self.structs[*sid].origin_gen.as_ref().map(|(g, _)| *g);
                imp.trait_name.as_deref() == Some(trait_name)
                    && match self.mod_syms[imp.mod_id].get(&name.text) {
                        Some(Sym::Struct(id)) => id == sid,
                        Some(Sym::GenStruct(gid)) => origin == Some(*gid),
                        _ => false,
                    }
            }),
            Some(GenArg::Ty(ty)) => {
                let ty = ty.clone();
                builtin_has(&ty, trait_name) || self.has_plain_impl(&ty, trait_name)
            }
            _ => false,
        };
        by_impl || self.has_bound_by_shape(param, trait_name)
    }

    /// `ty` (`Int`, `F64`, ...) has an `impl Tr` written for exactly it, not for a generic type.
    fn has_plain_impl(&mut self, ty: &Ty, trait_name: &str) -> bool {
        for idx in 0..self.impls.len() {
            let imp = &self.impls[idx];
            if imp.trait_name.as_deref() != Some(trait_name)
                || !matches!(&imp.target.kind, ast::TypeKind::Named { args, .. } if args.is_empty())
            {
                continue;
            }
            let (m, offset, target) = (imp.mod_id, imp.offset, imp.target);
            let saved = (
                std::mem::replace(&mut self.cur_mod, m),
                std::mem::replace(&mut self.offset, offset),
            );
            let got = self.resolve_ty(target);
            (self.cur_mod, self.offset) = saved;
            if got == *ty {
                return true;
            }
        }
        false
    }

    /// An `if const` test, or an `if T: Tr` one, settled at compile time; `None` (reported) when
    /// the `if const` test is not a compile-time constant.
    pub(super) fn settle_bool(&mut self, e: &ast::Expr) -> Option<bool> {
        if let ast::ExprKind::HasBound { param, trait_name } = &e.kind {
            return Some(self.has_bound(&param.text, &trait_name.text));
        }
        let got = self.eval_const_bool(e);
        if got.is_none() {
            self.error(
                codes::BAD_NUMBER,
                e.span,
                "the test of an `if const` must be a compile-time constant",
            );
            self.help(
                "it can compare literals, `const`s and `<const N>` parameters; \
                 use a plain `if` for a value read at run time",
            );
        }
        got
    }

    fn has_bound_by_shape(&mut self, param: &str, trait_name: &str) -> bool {
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

/// Whether a built-in type has a built-in operator or comparison trait, as
/// the checker's bound says (R08.1.7): `if T: Add:` holds for `Int` and `Str`.
fn builtin_has(ty: &Ty, trait_name: &str) -> bool {
    let number = matches!(ty, Ty::Int | Ty::I128 | Ty::U64 | Ty::Float);
    let scalar = number || matches!(ty, Ty::Bool | Ty::Char | Ty::Str);
    match trait_name {
        "Add" => number || *ty == Ty::Str,
        "Sub" | "Mul" | "Div" | "Rem" | "Neg" | "Num" => number,
        "Eq" | "Ord" | "Show" => scalar,
        "Hash" => scalar && *ty != Ty::Float,
        _ => false,
    }
}

//! Impls for types that are not structs: `impl Pick for Int -> Bool` gives
//! every fn of that type its methods (decision 456), while `impl<T> Op<T> for
//! (T, T) -> T` and `impl<T: Tr> Tr for ?T` give them to each type they fit,
//! made the first time one is called.

use super::*;

impl<'a> Lowerer<'a> {
    /// The names an impl target leaves open: `T` in `impl<T> Op<T> for (T, T) -> T`.
    pub(super) fn impl_params(&self, m: ModId, target: &ast::Type) -> Vec<String> {
        fn walk(l: &Lowerer, m: ModId, ty: &ast::Type, out: &mut Vec<String>) {
            match &ty.kind {
                ast::TypeKind::Named { name, args } if args.is_empty() => {
                    let builtin = matches!(
                        name.text.as_str(),
                        "Int" | "I64" | "I128" | "U64" | "F64" | "Bool" | "Char" | "Str" | "Unit"
                    );
                    if !builtin
                        && !l.mod_syms[m].contains_key(&name.text)
                        && !out.contains(&name.text)
                    {
                        out.push(name.text.clone());
                    }
                }
                ast::TypeKind::Named { args, .. } | ast::TypeKind::Tuple(args) => {
                    args.iter().for_each(|a| walk(l, m, a, out))
                }
                ast::TypeKind::Optional(t) | ast::TypeKind::Mut(t) => walk(l, m, t, out),
                ast::TypeKind::Fn { params, ret } => {
                    params.iter().for_each(|p| walk(l, m, p, out));
                    walk(l, m, ret, out);
                }
                _ => {}
            }
        }
        let mut out = Vec::new();
        walk(self, m, target, &mut out);
        out
    }

    /// How a fn-type impl target binds its open names to the fn type `ty`:
    /// `Some(None)` for a target with none, `None` when it cannot fit `ty`.
    pub(super) fn fn_impl_env(
        &self,
        m: ModId,
        target: &ast::Type,
        ty: &Ty,
    ) -> Option<Option<GenEnv>> {
        if !matches!(target.kind, ast::TypeKind::Fn { .. }) {
            return Some(None);
        }
        let params = self.impl_params(m, target);
        if params.is_empty() {
            return Some(None);
        }
        if !matches!(ty, Ty::Fn(..)) {
            return None;
        }
        let mut env = GenEnv::default();
        self.unify_ast_ty(target, ty, m, &params, &mut env);
        params
            .iter()
            .all(|p| env.contains_key(p))
            .then_some(Some(env))
    }

    /// The fn type a closure passed for `param` takes from a fn-type impl of
    /// its bound: `M: Op<Int>` and `impl<T> Op<T> for (T, T) -> T` give
    /// `(Int, Int) -> Int`, so `go((a, b) => a + b)` and `go(+)` are typed.
    pub(super) fn fn_impl_shape(
        &mut self,
        bounds: &[ast::Bound],
        param: &str,
        tpl: (ModId, u32),
        inferred: &GenEnv,
    ) -> Option<Ty> {
        let is_fn = |t: &ast::Type| matches!(t.kind, ast::TypeKind::Fn { .. });
        let ty = self.bound_impl_shape(bounds, param, tpl, inferred, is_fn)?;
        matches!(ty, Ty::Fn(..)).then_some(ty)
    }

    /// The `Monoid<T>` a bare operator passed for `param` becomes when its
    /// bound has an impl for `Monoid<T>`: `M: Op<Int>` and
    /// `impl<T> Op<T> for Monoid<T>` give `Monoid<Int>`, so `go(+)` folds
    /// with the monoid's choices (`Fold = T`), not a fn type's.
    pub(super) fn monoid_impl_shape(
        &mut self,
        bounds: &[ast::Bound],
        param: &str,
        tpl: (ModId, u32),
        inferred: &GenEnv,
    ) -> Option<Ty> {
        let is_monoid = |t: &ast::Type| matches!(&t.kind, ast::TypeKind::Named { name, .. } if name.text == "Monoid");
        let ty = self.bound_impl_shape(bounds, param, tpl, inferred, is_monoid)?;
        self.monoid_struct(&ty).map(|_| ty)
    }

    /// The `Monoid<T>` instance `ty` is, with its generic `of`.
    fn monoid_struct(&self, ty: &Ty) -> Option<GenFnId> {
        let Ty::Struct(sid) = ty else { return None };
        let (gid, _) = self.structs[*sid].origin_gen.as_ref()?;
        if self.gen_structs[*gid].decl.name.text != "Monoid" {
            return None;
        }
        self.structs[*sid].gen_methods.get("of").copied()
    }

    /// The operator `e` names when written bare (`+`, `max`), which
    /// `Monoid.of` gives an identity for; a closure is not one, though
    /// `(a, b) => a + b` folds the same.
    pub(super) fn monoid_bare_op(&self, e: &ast::Expr) -> Option<BareOp> {
        let written = match &e.kind {
            ast::ExprKind::Paren(inner) => return self.monoid_bare_op(inner),
            ast::ExprKind::Name(_) => true,
            ast::ExprKind::Lambda { body, .. } => {
                matches!(&body.kind, ast::ExprKind::Binary { op_span, .. } if *op_span == e.span)
            }
            _ => false,
        };
        written
            .then(|| self.detect_bare_op(e))
            .flatten()
            .filter(|op| *op != BareOp::Lcm)
    }

    /// A bare operator `Monoid.of` takes, passed where a `Monoid<T>` is
    /// wanted: `Monoid<T>.of(op)`.
    pub(super) fn monoid_of_bare_op(&mut self, want: &Ty, arg: &ast::Arg) -> Option<Expr> {
        self.monoid_bare_op(&arg.value)?;
        let of = self.monoid_struct(want)?;
        let span = arg.value.span;
        let arg = ast::Arg {
            name: None,
            ..arg.clone()
        };
        Some(self.call_gen_fn(of, Vec::new(), None, &[arg], span))
    }

    /// The type an impl of `param`'s bound whose target `fits` is for, its
    /// open names bound by the bound's own arguments.
    fn bound_impl_shape(
        &mut self,
        bounds: &[ast::Bound],
        param: &str,
        tpl: (ModId, u32),
        inferred: &GenEnv,
        fits: impl Fn(&ast::Type) -> bool,
    ) -> Option<Ty> {
        let traits = bounds.iter().filter(|b| b.name.text == param);
        for tr in traits.flat_map(|b| &b.traits) {
            let ast::TypeKind::Named { name, args } = &tr.kind else {
                continue;
            };
            let bound_args = self.quietly_in(tpl.0, tpl.1, inferred.clone(), |l| {
                args.iter().map(|a| l.resolve_ty(a)).collect::<Vec<_>>()
            });
            if bound_args.contains(&Ty::Error) {
                continue;
            }
            for idx in 0..self.impls.len() {
                let imp = &self.impls[idx];
                let (m, offset, target) = (imp.mod_id, imp.offset, imp.target);
                if imp.trait_name.as_deref() != Some(name.text.as_str())
                    || !fits(target)
                    || imp.trait_args.len() != bound_args.len()
                {
                    continue;
                }
                let trait_args = imp.trait_args.clone();
                let params = self.impl_params(m, target);
                let mut env = GenEnv::default();
                for (a, t) in trait_args.iter().zip(&bound_args) {
                    self.unify_ast_ty(a, t, m, &params, &mut env);
                }
                if !params.iter().all(|p| env.contains_key(p)) {
                    continue;
                }
                let ty = self.quietly_in(m, offset, env, |l| l.resolve_ty(target));
                if ty != Ty::Error {
                    return Some(ty);
                }
            }
        }
        None
    }

    /// `f` resolved in module `m` at `offset` with `env`, its diagnostics dropped.
    fn quietly_in<R>(
        &mut self,
        m: ModId,
        offset: u32,
        env: GenEnv,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let saved = (
            std::mem::replace(&mut self.cur_mod, m),
            std::mem::replace(&mut self.offset, offset),
            std::mem::replace(&mut self.gen_env, env),
            self.diags.len(),
        );
        let out = f(self);
        (self.cur_mod, self.offset, self.gen_env) = (saved.0, saved.1, saved.2);
        self.diags.truncate(saved.3);
        out
    }

    /// `impl<T> Op<T> for (T, T) -> T`: the method for one fn type, made the
    /// first time a value of that type calls it.
    pub(super) fn fn_impl_method(&mut self, ty: &Ty, method: &str) -> Option<FnId> {
        if !matches!(ty, Ty::Fn(..)) {
            return None;
        }
        for idx in 0..self.impls.len() {
            let (m, offset, target, body) = (
                self.impls[idx].mod_id,
                self.impls[idx].offset,
                self.impls[idx].target,
                self.impls[idx].body,
            );
            if !matches!(target.kind, ast::TypeKind::Fn { .. }) {
                continue;
            }
            let Some(Some(env)) = self.fn_impl_env(m, target, ty) else {
                continue;
            };
            let Some(f) = body.stmts.iter().find_map(|item| match &item.kind {
                ast::StmtKind::Fn(f) if f.name.text == method && f.generics.is_empty() => Some(f),
                _ => None,
            }) else {
                continue;
            };
            let saved = (
                std::mem::replace(&mut self.cur_mod, m),
                std::mem::replace(&mut self.offset, offset),
                std::mem::replace(&mut self.gen_env, env.clone()),
            );
            let fits = self.resolve_ty(target) == *ty;
            self.gen_env = saved.2;
            if !fits {
                (self.cur_mod, self.offset) = (saved.0, saved.1);
                continue;
            }
            let trait_name = self.impls[idx].trait_name.clone();
            let fn_id = self.declare_fn_in(f, m, offset, None, env);
            let suffix = self.prim_methods.len();
            self.sigs[fn_id].name = format!(
                "{}_fn_{}_{suffix}",
                trait_name.as_deref().unwrap_or("impl"),
                f.name.text
            );
            self.prim_self.insert(fn_id, ty.clone());
            self.prim_methods
                .push((ty.clone(), method.to_string(), fn_id));
            self.resolve_fn_sig(fn_id, trait_name.as_deref());
            (self.cur_mod, self.offset) = (saved.0, saved.1);
            return Some(fn_id);
        }
        None
    }

    /// The bare op an argument names (`+`), unless `param`'s bound is a
    /// trait some fn type implements: there `+` is a fn value of that type.
    pub(super) fn bare_op_for(
        &self,
        bounds: &[ast::Bound],
        param: &str,
        e: &ast::Expr,
    ) -> Option<BareOp> {
        (!self.bound_has_fn_impl(bounds, param))
            .then(|| self.detect_bare_op(e))
            .flatten()
    }

    /// Whether `param` is bound by a trait some fn type implements, as in
    /// `impl<T> Op<T> for (T, T) -> T`: a closure passed for it stays a fn
    /// value, so the impl's methods and associated types apply to it.
    fn bound_has_fn_impl(&self, bounds: &[ast::Bound], param: &str) -> bool {
        bounds
            .iter()
            .filter(|b| b.name.text == param)
            .flat_map(|b| &b.traits)
            .filter_map(|t| match &t.kind {
                ast::TypeKind::Named { name, .. } => Some(name.text.as_str()),
                _ => None,
            })
            .any(|tr| {
                self.impls.iter().any(|imp| {
                    imp.trait_name.as_deref() == Some(tr)
                        && matches!(imp.target.kind, ast::TypeKind::Fn { .. })
                })
            })
    }

    /// `impl<T: Tr> Tr for ?T`: the method for one `?T`, made the first
    /// time a value of that type calls it.
    pub(super) fn optional_impl_method(&mut self, ty: &Ty, method: &str) -> Option<FnId> {
        let Ty::Opt(inner) = ty else {
            return None;
        };
        for idx in 0..self.impls.len() {
            let (m, offset, target, body) = (
                self.impls[idx].mod_id,
                self.impls[idx].offset,
                self.impls[idx].target,
                self.impls[idx].body,
            );
            let ast::TypeKind::Optional(param) = &target.kind else {
                continue;
            };
            let ast::TypeKind::Named { name, args } = &param.kind else {
                continue;
            };
            if !args.is_empty() || self.mod_syms[m].contains_key(&name.text) {
                continue;
            }
            let Some(f) = body.stmts.iter().find_map(|item| match &item.kind {
                ast::StmtKind::Fn(f) if f.name.text == method && f.generics.is_empty() => Some(f),
                _ => None,
            }) else {
                continue;
            };
            let trait_name = self.impls[idx].trait_name.clone();
            let env = GenEnv::from([(name.text.clone(), GenArg::Ty((**inner).clone()))]);
            let saved = (
                std::mem::replace(&mut self.cur_mod, m),
                std::mem::replace(&mut self.offset, offset),
            );
            let fn_id = self.declare_fn_in(f, m, offset, None, env);
            let suffix = self.prim_methods.len();
            self.sigs[fn_id].name = format!(
                "{}_opt_{}_{suffix}",
                trait_name.as_deref().unwrap_or("impl"),
                f.name.text
            );
            self.prim_self.insert(fn_id, ty.clone());
            self.prim_methods
                .push((ty.clone(), method.to_string(), fn_id));
            self.resolve_fn_sig(fn_id, trait_name.as_deref());
            (self.cur_mod, self.offset) = saved;
            return Some(fn_id);
        }
        None
    }
}

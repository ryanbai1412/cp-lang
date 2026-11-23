//! A lambda's annotated parameters take part in a generic call's inference:
//! `undoing(s, (mut d, e: (V, V)) => ..)` binds `T` in `add: (mut S, T) -> ()`
//! to `(V, V)` before any default applies.

use super::*;

/// A trait an argument must implement, with its arguments known: `Feed<(w: Int)>`.
#[derive(Clone, Debug)]
pub(super) struct WantBound(String, Vec<Ty>);

impl<'a> Lowerer<'a> {
    /// Unify each annotated parameter of the lambda `arg` with its slot in the fn type `pat`.
    pub(super) fn unify_lambda_params(
        &mut self,
        pat: &ast::Type,
        arg: &ast::Expr,
        tpl_mod: ModId,
        gen_names: &[String],
        env: &mut GenEnv,
    ) {
        // A lambda whose parameters are all annotated has a type of its own, so it binds a bare
        // type parameter as any other argument does: `op: M` given `(i: V, j: V) => ...`.
        if let (ast::TypeKind::Named { name, args }, ast::ExprKind::Lambda { param_tys, .. }) =
            (&pat.kind, &arg.kind)
        {
            if args.is_empty()
                && gen_names.contains(&name.text)
                && !env.contains_key(&name.text)
                && !param_tys.is_empty()
                && param_tys.iter().all(Option::is_some)
            {
                if let Some(ty @ Ty::Fn(..)) = self.probe_ty_silent(arg) {
                    env.insert(name.text.clone(), GenArg::Ty(ty));
                }
            }
            return;
        }
        let (ast::TypeKind::Fn { params, .. }, ast::ExprKind::Lambda { param_tys, .. }) =
            (&pat.kind, &arg.kind)
        else {
            return;
        };
        if params.len() != param_tys.len() {
            return;
        }
        for (slot, ann) in params.iter().zip(param_tys) {
            let Some(ann) = ann else { continue };
            let slot = match &slot.kind {
                ast::TypeKind::Mut(inner) => inner,
                _ => slot,
            };
            let saved = self.diags.len();
            let ty = self.resolve_ty(ann);
            self.diags.truncate(saved);
            self.unify_ast_ty(slot, &ty, tpl_mod, gen_names, env);
        }
    }
    /// The fn type a one-parameter lambda passed as `g: Next<V>` (any trait whose first argument
    /// is the type of what the lambda takes) has, once `V` is known: `scc::<N>(u => adj[u])`.
    pub(super) fn trait_lambda_hint(
        &mut self,
        pty: &ast::Type,
        arg: &ast::Expr,
        inferred: &HashMap<String, GenArg>,
    ) -> Option<Ty> {
        let ast::ExprKind::Lambda { params, body, .. } = &arg.kind else {
            return None;
        };
        let ast::TypeKind::Named { args: targs, .. } = &pty.kind else {
            return None;
        };
        let Some(ast::Type {
            kind: ast::TypeKind::Named { name, args },
            ..
        }) = targs.first()
        else {
            return None;
        };
        if !args.is_empty() {
            return None;
        }
        let pty = match inferred.get(&name.text)? {
            GenArg::Domain(d) => Ty::Pos(*d),
            GenArg::Ty(t) => t.clone(),
            _ => return None,
        };
        let param = self.one_param(params, Some(&pty))?;
        let ret = self.probe_lambda_ret(
            std::slice::from_ref(&param),
            std::slice::from_ref(&pty),
            body,
        )?;
        Some(Ty::Fn(vec![pty], Box::new(ret)))
    }

    /// The one parameter a lambda takes where a `want` is wanted. Several
    /// parameters unpack a tuple or product `want` with as many parts, as one
    /// tuple pattern: `dijkstra((v, k) => …)` over `(V, K)` (decision 159).
    pub(super) fn one_param(
        &self,
        params: &[ast::Pattern],
        want: Option<&Ty>,
    ) -> Option<ast::Pattern> {
        let parts = match want {
            Some(Ty::Tuple(items)) => items.len(),
            Some(Ty::Pos(d)) => self.product_domains.get(d).map_or(1, Vec::len),
            _ => 1,
        };
        match params {
            [param] => Some(param.clone()),
            [first, ..] if params.len() == parts => Some(ast::Pattern {
                kind: ast::PatternKind::Tuple(params.to_vec()),
                span: first.span,
            }),
            _ => None,
        }
    }

    /// True for a fn `(U) -> …` given where `G: Tr<V, …>` is wanted and the
    /// trait's method takes a `V` after `self`, as `Next<V>`'s `out(self, u: V)`:
    /// the fn is that method, so `V` is `U`.
    pub(super) fn fn_meets_trait_over(&self, name: &str, tpl_mod: ModId, ps: &[Ty]) -> bool {
        let Some(Sym::Trait(tid)) = self.mod_syms[tpl_mod].get(name) else {
            return false;
        };
        ps.len() == 1
            && self.traits[*tid]
                .methods
                .iter()
                .any(|m| m.params.len() == 2)
    }

    /// A fn graph's vertices are what it takes: over `((R, C)) -> …` they are
    /// the inline product domain `(R, C)`, as in `components::<(R, C)>`, and
    /// over `(Cell) -> …` they are `Cell`, even where `from:` gave its parts
    /// tuple first (decision 159).
    pub(super) fn fn_graph_vertex_domains(
        &mut self,
        bounds: &[ast::Bound],
        tpl_mod: ModId,
        inferred: &mut GenEnv,
    ) {
        for b in bounds {
            let Some(GenArg::Ty(Ty::Fn(ps, _))) = inferred.get(&b.name.text).cloned() else {
                continue;
            };
            for tr in &b.traits {
                let ast::TypeKind::Named { name, args } = &tr.kind else {
                    continue;
                };
                let Some(ast::TypeKind::Named {
                    name: v,
                    args: vargs,
                }) = args.first().map(|a| &a.kind)
                else {
                    continue;
                };
                if !vargs.is_empty() || !self.fn_meets_trait_over(&name.text, tpl_mod, &ps) {
                    continue;
                }
                if !matches!(inferred.get(&v.text), None | Some(GenArg::Ty(Ty::Tuple(_)))) {
                    continue;
                }
                let d = match &ps[0] {
                    Ty::Pos(d) => *d,
                    Ty::Tuple(items) => {
                        let factors = items
                            .iter()
                            .map(|t| self.ty_to_product_factor(t).map(|f| (None, f)))
                            .collect::<Option<Vec<_>>>();
                        match factors.filter(|f| f.len() > 1) {
                            Some(factors) => self.intern_anon_product_domain(factors),
                            None => continue,
                        }
                    }
                    _ => continue,
                };
                inferred.insert(v.text.clone(), GenArg::Domain(d));
            }
        }
    }

    /// The type a one-parameter lambda's annotation gives its parameter.
    pub(super) fn annotated_param_ty(&mut self, arg: &ast::Expr) -> Option<Ty> {
        let ast::ExprKind::Lambda { param_tys, .. } = &arg.kind else {
            return None;
        };
        let [Some(ann)] = param_tys.as_slice() else {
            return None;
        };
        let saved = self.diags.len();
        let ty = self.resolve_ty(ann);
        self.diags.truncate(saved);
        (ty != Ty::Error).then_some(ty)
    }

    /// The type a lambda body gives with its parameters typed `p_tys`, probed without side effects.
    pub(super) fn probe_lambda_ret(
        &mut self,
        lparams: &[ast::Pattern],
        p_tys: &[Ty],
        body: &ast::Expr,
    ) -> Option<Ty> {
        let diag_len = self.diags.len();
        let saved_cx = self.cx.clone();
        let saved_outer = self.outer_cxs.clone();
        self.cx.scopes.push(HashMap::new());
        let mut pre = Vec::new();
        for (pat, ty) in lparams.iter().zip(p_tys) {
            self.loop_var(pat, ty.clone(), &mut pre);
        }
        let probed = self.expr(body, None);
        self.cx = saved_cx;
        self.outer_cxs = saved_outer;
        self.diags.truncate(diag_len);
        (probed.ty != Ty::Error).then_some(probed.ty)
    }
}

/// `by: ?(T -> K) = None` infers from a lambda as `by: T -> K` does.
pub(super) fn unwrap_optional(ty: &ast::Type) -> &ast::Type {
    match &ty.kind {
        ast::TypeKind::Optional(inner) => inner,
        _ => ty,
    }
}

impl<'a> Lowerer<'a> {
    /// What an argument for `pty` must implement, when `pty` is a bare generic not bound yet whose
    /// trait bound has every argument known: `st: St` with `St: Feed<T>` once `T` is bound.
    pub(super) fn want_bound(
        &mut self,
        pty: &ast::Type,
        decl: &ast::FnDecl,
        tpl_mod: ModId,
        tpl_offset: u32,
        inferred: &GenEnv,
    ) -> Option<WantBound> {
        let name = bare_name(pty)?;
        if inferred.contains_key(name) {
            return None;
        }
        let gen_names: Vec<String> = decl.generics.iter().map(|g| g.text.clone()).collect();
        let bound = decl.bounds.iter().find(|b| b.name.text == name)?;
        for tr in &bound.traits {
            let ast::TypeKind::Named { name: tn, args } = &tr.kind else {
                continue;
            };
            let unbound = |t: &ast::Type| {
                gen_names
                    .iter()
                    .any(|g| !inferred.contains_key(g) && ty_mentions(t, g))
            };
            if args.is_empty() || args.iter().any(unbound) {
                continue;
            }
            let saved_mod = std::mem::replace(&mut self.cur_mod, tpl_mod);
            let saved_offset = std::mem::replace(&mut self.offset, tpl_offset);
            let saved_env = std::mem::replace(&mut self.gen_env, inferred.clone());
            let tys: Vec<Ty> = args.iter().map(|t| self.resolve_ty(t)).collect();
            self.cur_mod = saved_mod;
            self.offset = saved_offset;
            self.gen_env = saved_env;
            if !tys.contains(&Ty::Error) {
                return Some(WantBound(tn.text.clone(), tys));
            }
        }
        None
    }

    /// A generic call's starting bindings: its owner's, its explicit arguments, and what its
    /// result must implement. `acc(0, f)` where a `Feed<X>` is wanted, with
    /// `impl Feed<T> for Acc<S, T>` and `acc` returning `Acc<S, T>`, binds its `T` to `X`
    /// before its lambdas are probed.
    pub(super) fn seed_gens(
        &self,
        owner_env: GenEnv,
        explicit: &[GenArg],
        decl: &ast::FnDecl,
        want: Option<WantBound>,
    ) -> GenEnv {
        let mut env = owner_env;
        for (name, val) in decl.generics.iter().zip(explicit) {
            env.insert(name.text.clone(), val.clone());
        }
        let (Some(WantBound(trait_name, want_args)), Some(ret)) = (want, decl.ret.as_ref()) else {
            return env;
        };
        let ast::TypeKind::Named {
            name: rn,
            args: rargs,
        } = &ret.kind
        else {
            return env;
        };
        for imp in &self.impls {
            let ast::TypeKind::Named {
                name: on,
                args: oargs,
            } = &imp.target.kind
            else {
                continue;
            };
            if imp.trait_name.as_deref() != Some(trait_name.as_str())
                || imp.trait_args.len() != want_args.len()
                || on.text != rn.text
                || oargs.len() != rargs.len()
            {
                continue;
            }
            for (o, r) in oargs.iter().zip(rargs) {
                let (Some(o), Some(r)) = (bare_name(o), bare_name(r)) else {
                    continue;
                };
                if env.contains_key(r) || !decl.generics.iter().any(|g| g.text == r) {
                    continue;
                }
                if let Some(j) = imp.trait_args.iter().position(|t| bare_name(t) == Some(o)) {
                    let arg = match &want_args[j] {
                        Ty::Pos(d) => GenArg::Domain(*d),
                        t => GenArg::Ty(t.clone()),
                    };
                    env.insert(r.to_string(), arg);
                }
            }
            break;
        }
        env
    }
}

fn bare_name(t: &ast::Type) -> Option<&str> {
    match &t.kind {
        ast::TypeKind::Named { name, args } if args.is_empty() => Some(&name.text),
        _ => None,
    }
}

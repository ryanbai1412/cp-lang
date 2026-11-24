//! Binding a generic's parameters by matching the type it was written with against the
//! type a call passes: `List<T>` given `List<Int>` binds `T`, a `Slice<S>` binds `S` to
//! the view's source.

use super::*;

impl<'a> Lowerer<'a> {
    /// Unifies an AST type pattern `pat` against a concrete `Ty` to bind generic parameters.
    #[allow(clippy::only_used_in_recursion)]
    pub(super) fn unify_bounds(
        &self,
        bounds: &[ast::Bound],
        tpl_mod: ModId,
        gen_names: &[String],
        env: &mut GenEnv,
    ) {
        for _ in 0..2 {
            let before = env.len();
            for bound in bounds {
                let concrete = match env.get(&bound.name.text) {
                    Some(GenArg::Ty(t)) => t.clone(),
                    Some(GenArg::Domain(d)) => Ty::Pos(*d),
                    _ => continue,
                };
                for tr in &bound.traits {
                    self.unify_ast_ty(tr, &concrete, tpl_mod, gen_names, env);
                }
            }
            if env.len() == before {
                break;
            }
        }
    }

    pub(super) fn unify_ast_ty(
        &self,
        pat: &ast::Type,
        concrete: &Ty,
        tpl_mod: ModId,
        gen_names: &[String],
        env: &mut GenEnv,
    ) {
        if *concrete == Ty::Error {
            return;
        }
        match &pat.kind {
            ast::TypeKind::Named { name, args }
                if args.is_empty() && gen_names.contains(&name.text) =>
            {
                env.entry(name.text.clone())
                    .or_insert_with(|| match concrete {
                        Ty::Pos(d) => GenArg::Domain(*d),
                        other => GenArg::Ty(other.clone()),
                    });
            }
            ast::TypeKind::Named { name, args }
                if self.alias_trait(&name.text, tpl_mod).is_some() =>
            {
                if let Some(tid) = self.alias_trait(&name.text, tpl_mod) {
                    self.unify_alias(tid, args, concrete, gen_names, env);
                }
            }
            ast::TypeKind::Named { name, args } => match (name.text.as_str(), concrete) {
                (
                    "List" | "Iter",
                    Ty::List(elem)
                    | Ty::ListView(elem, _)
                    | Ty::Keyed(elem, _)
                    | Ty::MutView(elem, _)
                    | Ty::Range(elem),
                ) if args.len() == 1 => {
                    self.unify_ast_ty(&args[0], elem, tpl_mod, gen_names, env);
                }
                ("List" | "Iter", Ty::Str | Ty::StrView) if args.len() == 1 => {
                    self.unify_ast_ty(&args[0], &Ty::Char, tpl_mod, gen_names, env);
                }
                ("List" | "Iter", Ty::Array { elem, .. }) if args.len() == 1 => {
                    self.unify_ast_ty(&args[0], elem, tpl_mod, gen_names, env);
                }
                ("Set" | "HashSet", Ty::Set(elem) | Ty::HashSet(elem)) if args.len() == 1 => {
                    self.unify_ast_ty(&args[0], elem, tpl_mod, gen_names, env);
                }
                ("Map" | "HashMap", Ty::Map(k, v) | Ty::HashMap(k, v)) if args.len() == 2 => {
                    self.unify_ast_ty(&args[0], k, tpl_mod, gen_names, env);
                    self.unify_ast_ty(&args[1], v, tpl_mod, gen_names, env);
                }
                ("Range", Ty::Range(inner)) if args.len() == 1 => {
                    let inner = match inner.as_ref() {
                        Ty::Bound(d) => Ty::Pos(*d),
                        other => other.clone(),
                    };
                    self.unify_ast_ty(&args[0], &inner, tpl_mod, gen_names, env);
                }
                // `Slice<S>` and `Rev<S>` are views of `S`: `S` is the source the view was cut from.
                ("Slice" | "Rev", view) if args.len() == 1 => {
                    let source = match view {
                        Ty::ListView(e, Dim::Int) => Ty::List(e.clone()),
                        Ty::ListView(e, d) => Ty::Array {
                            dims: vec![*d],
                            elem: e.clone(),
                        },
                        Ty::StrView => Ty::Str,
                        other => other.clone(),
                    };
                    self.unify_ast_ty(&args[0], &source, tpl_mod, gen_names, env);
                }
                ("Cut", Ty::Bound(d)) if args.len() == 1 => {
                    self.unify_ast_ty(&args[0], &Ty::Pos(*d), tpl_mod, gen_names, env);
                }
                (_, Ty::Struct(sid)) => {
                    // `S: Keyed<D, T>` given `Keys<S, I, T, K>`: the struct's
                    // impl of the trait says which of its args `D` and `T` are.
                    if let Some(given) = self.struct_trait_args(&name.text, *sid, args.len()) {
                        for (pa, actual) in args.iter().zip(given) {
                            match actual {
                                Some(GenArg::Ty(t)) => {
                                    self.unify_ast_ty(pa, &t, tpl_mod, gen_names, env)
                                }
                                Some(GenArg::Domain(d)) => {
                                    self.unify_ast_ty(pa, &Ty::Pos(d), tpl_mod, gen_names, env)
                                }
                                _ => {}
                            }
                        }
                    } else if let Some((_, actual_args)) = &self.structs[*sid].origin_gen {
                        // Also allow structural subtype `Group<T>` -> `Monoid<T>` -> `Semigroup<T>`:
                        // their first generic arg is `T`!
                        for (pa, actual) in args.iter().zip(actual_args) {
                            match actual {
                                GenArg::Ty(t) | GenArg::Sized(t, _) => {
                                    self.unify_ast_ty(pa, t, tpl_mod, gen_names, env)
                                }
                                GenArg::Domain(d) => {
                                    self.unify_ast_ty(pa, &Ty::Pos(*d), tpl_mod, gen_names, env)
                                }
                                GenArg::Int(v) => {
                                    if let ast::TypeKind::Named { name: n, args: a } = &pa.kind {
                                        if a.is_empty() && gen_names.contains(&n.text) {
                                            env.entry(n.text.clone()).or_insert(GenArg::Int(*v));
                                        }
                                    } else if let ast::TypeKind::Value(e) = &pa.kind {
                                        if let ast::ExprKind::Name(n) = &e.kind {
                                            if gen_names.contains(n) {
                                                env.entry(n.clone()).or_insert(GenArg::Int(*v));
                                            }
                                        }
                                    }
                                }
                                GenArg::RuntimeInt(gid) => {
                                    if let ast::TypeKind::Named { name: n, args: a } = &pa.kind {
                                        if a.is_empty() && gen_names.contains(&n.text) {
                                            env.entry(n.text.clone())
                                                .or_insert(GenArg::RuntimeInt(*gid));
                                        }
                                    }
                                }
                                GenArg::Op(op) => {
                                    if let ast::TypeKind::Named { name: n, args: a } = &pa.kind {
                                        if a.is_empty() && gen_names.contains(&n.text) {
                                            env.entry(n.text.clone()).or_insert(GenArg::Op(*op));
                                        }
                                    }
                                }
                                GenArg::Range(lo, hi) => {
                                    if let ast::TypeKind::Named { name: n, args: a } = &pa.kind {
                                        if a.is_empty() && gen_names.contains(&n.text) {
                                            env.entry(n.text.clone())
                                                .or_insert(GenArg::Range(*lo, *hi));
                                        }
                                    }
                                }
                                GenArg::OptRange(lo, hi) => {
                                    if let ast::TypeKind::Named { name: n, args: a } = &pa.kind {
                                        if a.is_empty() && gen_names.contains(&n.text) {
                                            env.entry(n.text.clone())
                                                .or_insert(GenArg::OptRange(*lo, *hi));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                (_, Ty::Array { dims, elem })
                    if !args.is_empty()
                        && matches!(
                            self.mod_syms[tpl_mod].get(&name.text),
                            Some(Sym::Trait(_))
                        ) =>
                {
                    let is_domain_indexed = matches!(
                        (dims.as_slice(), elem.as_ref()),
                        ([Dim::Pos(_) | Dim::Sym(_)], Ty::List(_))
                    );
                    let trait_takes_pos = match self.mod_syms[tpl_mod].get(&name.text) {
                        Some(Sym::Trait(tid)) => self.traits[*tid]
                            .methods
                            .iter()
                            .any(|m| m.params.len() >= 2),
                        _ => false,
                    };
                    if is_domain_indexed && trait_takes_pos {
                        if let [Dim::Pos(d) | Dim::Sym(d)] = dims.as_slice() {
                            self.unify_ast_ty(&args[0], &Ty::Pos(*d), tpl_mod, gen_names, env);
                        }
                    } else if args.len() == 1 {
                        self.unify_ast_ty(&args[0], elem, tpl_mod, gen_names, env);
                    }
                }
                (_, Ty::Record(rfields))
                    if !self.record_getters(&name.text, tpl_mod).is_empty() =>
                {
                    for (at, getter) in self.record_getters(&name.text, tpl_mod) {
                        if let (Some(arg), Some((_, fty))) =
                            (args.get(at), rfields.iter().find(|(f, _)| *f == getter))
                        {
                            self.unify_ast_ty(arg, fty, tpl_mod, gen_names, env);
                        }
                    }
                }
                (_, Ty::Fn(ps, _)) if self.fn_meets_trait_over(&name.text, args, tpl_mod, ps) => {
                    self.unify_ast_ty(&args[0], &ps[0], tpl_mod, gen_names, env);
                }
                (_, concrete_seq)
                    if args.len() == 1
                        && matches!(
                            self.mod_syms[tpl_mod].get(&name.text),
                            Some(Sym::Trait(_))
                        ) =>
                {
                    if let Some(elem) = concrete_seq.seq_elem() {
                        self.unify_ast_ty(&args[0], &elem, tpl_mod, gen_names, env);
                    }
                }
                _ => {
                    if let Some(Sym::GenAlias(gaid)) = self
                        .mod_syms
                        .get(tpl_mod)
                        .and_then(|m| m.get(&name.text).copied())
                        .or_else(|| self.get_sym(&name.text))
                    {
                        let ga = &self.gen_aliases[gaid];
                        let alias_names: Vec<String> =
                            ga.generics.iter().map(|g| g.text.clone()).collect();
                        let mut alias_env = HashMap::new();
                        self.unify_ast_ty(ga.ty, concrete, ga.mod_id, &alias_names, &mut alias_env);
                        for (param_name, arg_ast) in alias_names.iter().zip(args) {
                            match alias_env.get(param_name) {
                                Some(GenArg::Ty(t)) => {
                                    self.unify_ast_ty(arg_ast, t, tpl_mod, gen_names, env);
                                }
                                Some(GenArg::Domain(d)) => {
                                    self.unify_ast_ty(
                                        arg_ast,
                                        &Ty::Pos(*d),
                                        tpl_mod,
                                        gen_names,
                                        env,
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                }
            },
            ast::TypeKind::Array { index, elem } => {
                if let Ty::Array { dims, elem: ce } = concrete {
                    self.unify_array_dims(index, dims, tpl_mod, gen_names, env);
                    self.unify_ast_ty(elem, ce, tpl_mod, gen_names, env);
                } else if let (
                    Ty::List(ce) | Ty::ListView(ce, _) | Ty::Keyed(ce, _) | Ty::MutView(ce, _),
                    [_],
                ) = (concrete, index.as_slice())
                {
                    self.unify_view_dim(index, concrete, tpl_mod, gen_names, env);
                    self.unify_ast_ty(elem, ce, tpl_mod, gen_names, env);
                } else if let (Ty::Str | Ty::StrView, [_]) = (concrete, index.as_slice()) {
                    self.unify_ast_ty(elem, &Ty::Char, tpl_mod, gen_names, env);
                }
            }
            ast::TypeKind::Optional(inner) => {
                if let Ty::Opt(ci) = concrete {
                    self.unify_ast_ty(inner, ci, tpl_mod, gen_names, env);
                } else {
                    self.unify_ast_ty(inner, concrete, tpl_mod, gen_names, env);
                }
            }
            ast::TypeKind::Tuple(items) => {
                let cfields = concrete.fields().or_else(|| match concrete {
                    Ty::Pos(d) => Some(
                        self.product_domains
                            .get(d)?
                            .iter()
                            .map(|(_, f)| self.product_factor_ty(f))
                            .collect(),
                    ),
                    _ => None,
                });
                for (p, c) in items.iter().zip(cfields.iter().flatten()) {
                    self.unify_ast_ty(p, c, tpl_mod, gen_names, env);
                }
            }
            ast::TypeKind::Record(fields) | ast::TypeKind::Row(fields) => {
                if let Ty::Record(cfields) = concrete {
                    for (pname, p) in fields {
                        if let Some((_, c)) = cfields.iter().find(|(cn, _)| cn == &pname.text) {
                            self.unify_ast_ty(p, c, tpl_mod, gen_names, env);
                        }
                    }
                } else if let Some(cfields) = concrete.fields() {
                    for ((_, p), c) in fields.iter().zip(&cfields) {
                        self.unify_ast_ty(p, c, tpl_mod, gen_names, env);
                    }
                }
            }
            // A generator is lowered as the list of what it yields.
            ast::TypeKind::Yields(inner) => {
                if let Ty::List(elem) = concrete {
                    self.unify_ast_ty(inner, elem, tpl_mod, gen_names, env);
                }
            }
            ast::TypeKind::Mut(inner) => {
                let c = match concrete {
                    Ty::Mut(c) => c.as_ref(),
                    c => c,
                };
                self.unify_ast_ty(inner, c, tpl_mod, gen_names, env);
            }
            ast::TypeKind::Fn { params, ret } => {
                if let Ty::Fn(cp, cr) = concrete {
                    for (p, c) in params.iter().zip(cp) {
                        self.unify_ast_ty(p, c, tpl_mod, gen_names, env);
                    }
                    self.unify_ast_ty(ret, cr, tpl_mod, gen_names, env);
                }
            }
            _ => {}
        }
    }

    /// The args struct `sid`'s impl of trait `trait_name` gives it, each
    /// one its impl names by a bare parameter of the target.
    fn struct_trait_args(
        &self,
        trait_name: &str,
        sid: StructId,
        n: usize,
    ) -> Option<Vec<Option<GenArg>>> {
        (0..self.impls.len()).find_map(|idx| {
            let imp = &self.impls[idx];
            if imp.trait_name.as_deref() != Some(trait_name) || imp.trait_args.len() != n {
                return None;
            }
            let env = self.struct_impl_env(idx, sid)?;
            Some(
                (imp.trait_args.iter())
                    .map(|t| bare_name(t).and_then(|g| env.get(g).cloned()))
                    .collect(),
            )
        })
    }

    /// A trait a record meets through getters, as `Weighted<W>`'s `@get fn w(self) -> W`:
    /// each (parameter index, getter name) whose getter returns that bare parameter, so the
    /// record's field of that name binds it.
    fn record_getters(&self, trait_name: &str, tpl_mod: ModId) -> Vec<(usize, &str)> {
        let Some(Sym::Trait(tid)) = self.mod_syms[tpl_mod].get(trait_name) else {
            return Vec::new();
        };
        let info = &self.traits[*tid];
        info.methods
            .iter()
            .filter(|m| m.params.len() == 1)
            .filter_map(|m| match &m.ret.as_ref()?.kind {
                ast::TypeKind::Named { name, args } if args.is_empty() => {
                    let at = info.generics.iter().position(|g| *g == name.text)?;
                    Some((at, m.name.text.as_str()))
                }
                _ => None,
            })
            .collect()
    }
}

pub(super) fn bare_name(t: &ast::Type) -> Option<&str> {
    match &t.kind {
        ast::TypeKind::Named { name, args } if args.is_empty() => Some(&name.text),
        _ => None,
    }
}

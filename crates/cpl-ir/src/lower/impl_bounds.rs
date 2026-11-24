//! An impl head's inline bounds (R12.3): `impl Seq<T> for Wrap<S: Seq<T>>`
//! takes `T` from what `S` is, and gives `Wrap<S>` nothing where `S` is not a
//! `Seq`.

use super::*;

impl<'a> Lowerer<'a> {
    /// Binds the names an impl's bounds leave open from the types its target
    /// bound: `K` and `T` in `Z<A: Indexed<K, T>>` are `A`'s key and element.
    /// False when a bound does not hold, so the impl does not apply.
    pub(super) fn bind_impl_bounds(
        &mut self,
        bounds: &[ast::Bound],
        m: ModId,
        env: &mut GenEnv,
    ) -> bool {
        let mut open = Vec::new();
        for tr in bounds.iter().flat_map(|b| &b.traits) {
            for name in self.impl_params(m, tr) {
                if !env.contains_key(&name) && !open.contains(&name) {
                    open.push(name);
                }
            }
        }
        for b in bounds {
            let Some(GenArg::Ty(seq)) = env.get(&b.name.text).cloned() else {
                continue;
            };
            for tr in &b.traits {
                let ast::TypeKind::Named { name, args } = &tr.kind else {
                    continue;
                };
                let trait_name = name.text.as_str();
                let (key, elem) = match trait_name {
                    "Seq" | "Bidir" | "Sized" => (None, args.first()),
                    "Keyed" | "Indexed" | "IndexedMut" => (args.first(), args.get(1)),
                    _ => continue,
                };
                let builtin = match trait_name {
                    "Indexed" | "Bidir" => seq.is_indexed(),
                    "IndexedMut" => seq.is_indexed_mut(),
                    _ => seq.is_seq(),
                };
                if !builtin && !(matches!(seq, Ty::Struct(_)) && self.implements(&seq, trait_name))
                {
                    return false;
                }
                if let Some(k) = key {
                    self.unify_ast_ty(k, &seq.key_ty(), m, &open, env);
                }
                if let (Some(t), Some(e)) = (elem, self.seq_elem_of(&seq)) {
                    self.unify_ast_ty(t, &e, m, &open, env);
                }
            }
        }
        true
    }

    /// What walking a value of `ty` gives: a built-in sequence's element, or what the `iter`
    /// of a struct's `impl Seq` yields.
    pub(super) fn seq_elem_of(&mut self, ty: &Ty) -> Option<Ty> {
        if let Some(e) = ty.seq_elem() {
            return Some(e);
        }
        let Ty::Struct(sid) = ty else {
            return None;
        };
        let f = *self.structs[*sid].methods.get("iter")?;
        if !self.sigs[f].params.is_empty() {
            return None;
        }
        self.queue_fn(f);
        match self.ret_of(f, Span::default()) {
            Ty::List(elem) => Some(*elem),
            _ => None,
        }
    }

    /// What a value of `ty` is keyed by: a built-in sequence's key, or the key the `items`
    /// of a struct's `impl Keyed` yields.
    pub(super) fn seq_key_of(&mut self, ty: &Ty) -> Option<Dim> {
        if let Some(k) = ty.seq_key() {
            return Some(k);
        }
        let Ty::Struct(sid) = ty else {
            return None;
        };
        let f = *self.structs[*sid].methods.get("items")?;
        if !self.sigs[f].params.is_empty() {
            return None;
        }
        self.queue_fn(f);
        let Ty::List(pair) = self.ret_of(f, Span::default()) else {
            return None;
        };
        match &*pair {
            Ty::Tuple(parts) if parts.len() == 2 => match parts[0] {
                Ty::Pos(d) => Some(Dim::Pos(d)),
                Ty::Int => Some(Dim::Int),
                _ => None,
            },
            _ => None,
        }
    }

    /// A struct value with an `impl Seq` and no `Show` of its own, as the list its `iter`
    /// walks: any `Seq` prints as a sequence (R10.3.1).
    pub(super) fn walked_struct(&mut self, value: Expr) -> Expr {
        let Ty::Struct(sid) = value.ty else {
            return value;
        };
        if self.structs[sid].def.show_fn.is_some() || !self.implements(&value.ty, "Seq") {
            return value;
        }
        let Some(&f) = self.structs[sid].methods.get("iter") else {
            return value;
        };
        if !self.sigs[f].params.is_empty() {
            return value;
        }
        self.queue_fn(f);
        match self.ret_of(f, Span::default()) {
            ret @ Ty::List(_) => {
                let span = value.span;
                Expr {
                    kind: ExprKind::Call(f, vec![Arg::Value(value)]),
                    ty: ret,
                    span,
                }
            }
            _ => value,
        }
    }

    /// The impl's type parameters bound for the concrete struct `sid`, if impl `idx` is
    /// for `sid` and its bounds hold there (`impl Indexed for Mapped<S: Indexed>`).
    pub(super) fn impl_env_for(&mut self, idx: usize, sid: StructId) -> Option<GenEnv> {
        let sdecl = self.structs[sid].decl?;
        let struct_decl_name = sdecl.name.text.clone();
        let struct_mod = self.structs[sid].mod_id;
        let origin_args: Vec<GenArg> = self.structs[sid]
            .origin_gen
            .as_ref()
            .map(|(_, a)| a.clone())
            .unwrap_or_default();
        let imp_mod = self.impls[idx].mod_id;
        let imp_target = self.impls[idx].target;
        let ast::TypeKind::Named {
            name: tname,
            args: targs,
        } = &imp_target.kind
        else {
            return None;
        };
        // Check whether `tname` in `imp_mod` refers to the same struct.
        let target_matches = match self.mod_syms[imp_mod].get(&tname.text).copied() {
            Some(Sym::Struct(id)) => id == sid,
            Some(Sym::GenStruct(gid)) => self.structs[sid]
                .origin_gen
                .as_ref()
                .is_some_and(|(g, _)| *g == gid),
            _ => tname.text == struct_decl_name && imp_mod == struct_mod,
        };
        if !target_matches {
            return None;
        }
        if targs.len() != origin_args.len() {
            return None;
        }
        // Bind impl type params from `targs` vs `origin_args`.
        let mut env = self.structs[sid].env.clone();
        let mut ok = true;
        for (targ, actual) in targs.iter().zip(&origin_args) {
            match &targ.kind {
                ast::TypeKind::Named { name, args } if args.is_empty() => {
                    env.insert(name.text.clone(), actual.clone());
                }
                ast::TypeKind::Value(expr) => {
                    if let ast::ExprKind::Name(n) = &expr.kind {
                        env.insert(n.clone(), actual.clone());
                    } else if let Some(v) = self.eval_const_int(expr) {
                        if *actual != GenArg::Int(v) {
                            ok = false;
                        }
                    }
                }
                _ => {}
            }
        }
        (ok && self.bind_impl_bounds(self.impls[idx].bounds, imp_mod, &mut env)).then_some(env)
    }
}

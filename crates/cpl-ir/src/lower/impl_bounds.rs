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
}

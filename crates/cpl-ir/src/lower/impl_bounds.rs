//! An impl head's inline bounds (R12.3): `impl Seq<T> for Wrap<S: Seq<T>>`
//! takes `T` from what `S` is, and gives `Wrap<S>` nothing where `S` is not a
//! `Seq`.

use super::*;

impl GenArg {
    /// What a key dimension binds a key generic to: its domain, or `Int` for a plain index.
    pub(super) fn of_key(dim: Dim) -> Option<GenArg> {
        match dim {
            Dim::Pos(d) => Some(GenArg::Domain(d)),
            Dim::Int | Dim::IntFrom(_) => Some(GenArg::Ty(Ty::Int)),
            _ => None,
        }
    }
}

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
                    "Seq" | "Bidir" | "Sized" | "Positional" => (None, args.first()),
                    "Keyed" | "Indexed" | "IndexedMut" => (args.first(), args.get(1)),
                    _ => continue,
                };
                let builtin = match trait_name {
                    "Indexed" | "Bidir" | "Positional" => seq.is_indexed(),
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
                if let (Some(t), Some(e)) = (elem, seq.seq_elem()) {
                    self.unify_ast_ty(t, &e, m, &open, env);
                }
            }
        }
        true
    }
}

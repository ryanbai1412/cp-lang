//! Trait defaults chosen by the most derived trait (decision 469):
//! `Seq` writes `type Taken = Gen<T>`, `Indexed` refines it to
//! `Slice<Self>`, and a type with both impls takes `Indexed`'s.

use super::*;

impl<'a> Lowerer<'a> {
    /// True when trait `sub` names `sup` among its supertraits, at any depth.
    pub(super) fn trait_extends(&self, sub: usize, sup: usize) -> bool {
        let mut seen = Vec::new();
        let mut todo = vec![sub];
        while let Some(t) = todo.pop() {
            if seen.contains(&t) {
                continue;
            }
            seen.push(t);
            let m = self.traits[t].mod_id;
            for name in &self.traits[t].supers {
                if let Some(Sym::Trait(s)) = self.mod_syms[m].get(name).copied() {
                    if s == sup {
                        return true;
                    }
                    todo.push(s);
                }
            }
        }
        false
    }

    /// `Self.<item>` for struct `sid` from the trait defaults of the impls it
    /// has, an impl that chooses `item` itself aside: the most derived wins.
    pub(super) fn default_assoc(&mut self, sid: StructId, item: &str) -> Option<Ty> {
        let mut found: Vec<(usize, usize, GenEnv)> = Vec::new();
        for idx in 0..self.impls.len() {
            let Some(tname) = self.impls[idx].trait_name.as_deref() else {
                continue;
            };
            let Some(Sym::Trait(tid)) = self.mod_syms[self.impls[idx].mod_id].get(tname).copied()
            else {
                continue;
            };
            if self.traits[tid]
                .assoc_defaults
                .iter()
                .all(|(n, _)| *n != item)
            {
                continue;
            }
            let chooses = self.impls[idx].body.stmts.iter().any(
                |s| matches!(&s.kind, ast::StmtKind::TypeAlias { name, .. } if name.text == item),
            );
            if chooses {
                continue;
            }
            if let Some(env) = self.struct_impl_env(idx, sid) {
                found.push((idx, tid, env));
            }
        }
        let pick = (0..found.len()).find(|&i| {
            (0..found.len()).all(|j| i == j || self.trait_extends(found[i].1, found[j].1))
        })?;
        let (idx, tid, env) = found.swap_remove(pick);
        let (imp_mod, imp_offset) = (self.impls[idx].mod_id, self.impls[idx].offset);
        let trait_args = self.impls[idx].trait_args.clone();
        let saved = (
            std::mem::replace(&mut self.cur_mod, imp_mod),
            std::mem::replace(&mut self.offset, imp_offset),
            std::mem::replace(&mut self.gen_env, env),
            self.self_struct.replace(sid),
        );
        let args: Vec<Ty> = trait_args.iter().map(|t| self.resolve_ty(t)).collect();
        let tenv: GenEnv = self.traits[tid]
            .generics
            .iter()
            .zip(args)
            .map(|(g, a)| (g.to_string(), GenArg::Ty(a)))
            .collect();
        let default = self.traits[tid]
            .assoc_defaults
            .iter()
            .find(|(n, _)| *n == item)
            .map(|(_, t)| *t)?;
        self.cur_mod = self.traits[tid].mod_id;
        self.offset = self.traits[tid].offset;
        self.gen_env = tenv;
        let ty = self.resolve_ty(default);
        (self.cur_mod, self.offset, self.gen_env, self.self_struct) = saved;
        Some(ty)
    }
}

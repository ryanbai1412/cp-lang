//! `type` aliases. Inside an alias's right side its own name means what the
//! alias shadows (`type Mod = Mod<P>` names std's `Mod`), so expanding an
//! alias can never re-enter it.

use super::*;

impl<'a> Lowerer<'a> {
    /// Runs `f` on alias `aid`'s right side, where it was declared.
    /// The generic argument a bare name stands for: a bound type
    /// parameter, or a range alias, which passes its range so that
    /// `T.MIN` of `type M = -10..=10` is `-10`.
    pub(super) fn named_gen_arg(&mut self, name: &str) -> Option<GenArg> {
        if let Some(a) = self.gen_env.get(name).cloned() {
            return Some(a);
        }
        let Some(Sym::Alias(aid)) = self.get_sym(name) else {
            return None;
        };
        let range = matches!(self.aliases[aid].1.kind, ast::TypeKind::Range { .. });
        range.then(|| self.in_alias(aid, |lw, ty| lw.resolve_gen_arg(ty)))
    }

    pub(super) fn in_alias<T>(
        &mut self,
        aid: usize,
        f: impl FnOnce(&mut Self, &'a ast::Type) -> T,
    ) -> T {
        let (m, ty, offset) = self.aliases[aid];
        let saved_mod = std::mem::replace(&mut self.cur_mod, m);
        let saved_offset = std::mem::replace(&mut self.offset, offset);
        self.alias_stack.push(aid);
        let out = f(self, ty);
        self.alias_stack.pop();
        self.cur_mod = saved_mod;
        self.offset = saved_offset;
        out
    }

    /// `sym` as seen from here: an alias being expanded means what it shadows.
    pub(super) fn unaliased(&self, sym: Sym) -> Option<Sym> {
        match sym {
            Sym::Alias(aid) if self.alias_stack.contains(&aid) => {
                self.alias_shadows.get(&aid).copied()
            }
            sym => Some(sym),
        }
    }

    /// Records that module `m`'s alias `name`, if it is one, shadows the import `sym`.
    pub(super) fn note_alias_shadow(&mut self, m: ModId, name: &str, sym: Sym) {
        if let Some(&Sym::Alias(aid)) = self.mod_syms[m].get(name) {
            if !matches!(sym, Sym::Alias(other) if other == aid) {
                self.alias_shadows.insert(aid, sym);
            }
        }
    }
}

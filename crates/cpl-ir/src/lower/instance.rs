//! A generic fn's instances, one per type it is called at, and the guard
//! that stops a fn reaching itself at a type that grows on every call.

use super::declare::{opt_mut_instance, param_marker};
use super::*;

/// Each instance's template, its type arguments as shown, and the fn whose
/// lowering asked for it.
#[derive(Default)]
pub(super) struct Growth {
    parent: HashMap<FnId, (GenFnId, String, Option<FnId>)>,
    /// The fn whose body is being lowered.
    pub(super) cur: Option<FnId>,
}

impl<'a> Lowerer<'a> {
    /// Resolves a generic function instantiation with concrete `args`, memoized.
    pub(super) fn instantiate_fn(
        &mut self,
        gfid: GenFnId,
        args: Vec<GenArg>,
        trait_params: &[(usize, Ty)],
        op_params: &[(String, BareOp)],
        rev_params: Vec<String>,
    ) -> FnId {
        let key = (gfid, args.clone(), rev_params.clone());
        if let Some(&id) = self.fn_insts.get(&key) {
            if !self.sigs[id].returns_fresh_domain && self.sigs[id].new_domain_slots.is_empty() {
                return id;
            }
        }
        let tpl_mod = self.gen_fns[gfid].mod_id;
        let tpl_offset = self.gen_fns[gfid].offset;
        let owner = self.gen_fns[gfid].owner;
        let mut env = self.gen_fns[gfid].owner_env.clone();
        let decl = opt_mut_instance(self.gen_fns[gfid].decl, &rev_params);
        for (p, a) in decl.generics.iter().zip(&args) {
            env.insert(p.text.clone(), a.clone());
        }
        // `S: Indexed<K, T>` with `S` keyed by `Int`: a `[K]T` it returns
        // starts at `S`'s first key, not at 0 (R04b.5.3).
        for bound in &decl.bounds {
            for tr in &bound.traits {
                if let ast::TypeKind::Named { name, args: targs } = &tr.kind {
                    if let (
                        true,
                        Some(ast::Type {
                            kind:
                                ast::TypeKind::Named {
                                    name: k,
                                    args: kargs,
                                },
                            ..
                        }),
                    ) = (
                        matches!(name.text.as_str(), "Keyed" | "Indexed" | "IndexedMut"),
                        targs.first(),
                    ) {
                        if kargs.is_empty() && env.get(&k.text) == Some(&GenArg::Ty(Ty::Int)) {
                            env.insert(int_key_marker(&k.text), GenArg::Ty(Ty::Int));
                        }
                    }
                }
            }
        }
        for (idx, ty) in trait_params {
            env.insert(format!("__trait_param_{idx}"), GenArg::Ty(ty.clone()));
        }
        for (name, op) in op_params {
            env.insert(op_param_key(name), GenArg::Op(*op));
        }
        for name in &rev_params {
            env.insert(param_marker(name), GenArg::Int(1));
        }
        let id = self.declare_fn_in(decl, tpl_mod, tpl_offset, owner, env);
        let suffix: Vec<String> = args.iter().map(|a| self.mangle_arg(a)).collect();
        self.record_instance(id, gfid, &args);
        self.sigs[id].name = format!("{}_{}_{id}", self.sigs[id].name, suffix.join("_"));
        self.fn_insts.insert(key, id);
        self.resolve_fn_sig(id, None);
        id
    }

    fn record_instance(&mut self, id: FnId, gfid: GenFnId, args: &[GenArg]) {
        let shown: Vec<String> = args
            .iter()
            .filter_map(|a| match a {
                GenArg::Ty(t) => Some(self.show(t)),
                _ => None,
            })
            .collect();
        self.growth
            .parent
            .insert(id, (gfid, shown.join(", "), self.growth.cur));
    }

    /// E0319 at a call of `name` that would make one more instance in a
    /// chain that grows without end.
    pub(super) fn growing_call(&mut self, gfid: GenFnId, name: &str, span: Span) -> Option<Expr> {
        let (outer, inner) = self.growing_instance(gfid)?;
        self.error(
            codes::GROWING_INSTANTIATION,
            span,
            format!(
                "`{name}` calls itself at a type that grows on every call: \
                 `{name}<{outer}>` needs `{name}<{inner}>`, and so on without end"
            ),
        );
        self.help(
            "a generic fn is compiled once per type: recurse at the same type, \
             or keep the growing part in a value of one fixed type (R08.2.10)",
        );
        Some(self.err_expr(span))
    }

    /// R08.2.10: everything monomorphizes, so a generic fn whose lowering
    /// reaches itself through `GROWTH_LIMIT` instances, each at a new type, is
    /// polymorphic recursion that never ends. Returns the first two
    /// instances' type arguments, outermost first.
    fn growing_instance(&self, gfid: GenFnId) -> Option<(String, String)> {
        const GROWTH_LIMIT: usize = 24;
        let mut chain = Vec::new();
        let mut at = self.growth.cur;
        while let Some((g, shown, parent)) = at.and_then(|id| self.growth.parent.get(&id)) {
            if *g == gfid {
                chain.push(shown.clone());
            }
            at = *parent;
        }
        if chain.len() < GROWTH_LIMIT {
            return None;
        }
        let outer = chain.pop()?;
        Some((outer, chain.pop()?))
    }
}

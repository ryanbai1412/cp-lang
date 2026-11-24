//! A generic fn's instances, one per type it is called at, and the guard
//! that stops a fn reaching itself at a type that grows on every call.

use super::declare::{opt_mut_instance, param_marker};
use super::*;
use cpl_diag::msgs;

/// Each instance's template, its type arguments as shown, and the fn whose
/// lowering asked for it.
#[derive(Default)]
pub(super) struct Growth {
    parent: HashMap<FnId, (GenFnId, String, Option<FnId>)>,
    /// The fn whose body is being lowered.
    pub(super) cur: Option<FnId>,
    /// The module whose code asked for each instance.
    from: HashMap<FnId, usize>,
    /// Each impl whose trait args are being solved, with its self type: an
    /// impl met again for the same type is a cycle, not a proof.
    pub(super) impl_args: Vec<(usize, Ty)>,
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
        range_ends: &[(String, GenArg)],
    ) -> FnId {
        // A `T: Read` fn called under `with io(base: 1)` reads its counted
        // positions 1-based, like the caller's own `read<T>()` (R10.2.2).
        let base_one = self.base_one() && reads_a_param(self.gen_fns[gfid].decl);
        let mut key_args = args.clone();
        key_args.extend(range_ends.iter().map(|(_, a)| a.clone()));
        let key = mono::InstKey::Fn(gfid, key_args, rev_params.clone(), base_one);
        if let Some(&id) = self.insts.get(&key) {
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
            env.insert(mono::own_generic_marker(&p.text), GenArg::Int(1));
        }
        // `A: Acts<T, F>` with `A = Plus`: the body's `act.tag0()` needs
        // the `T` that solving the bound gave `impl<T> Acts<T, T> for Plus`.
        let mut solved = env.clone();
        self.bind_impl_bounds(&decl.bounds, tpl_mod, &mut solved);
        env.extend(
            solved
                .into_iter()
                .filter(|(k, _)| k.starts_with("impl") && k.contains('.')),
        );
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
        if base_one {
            env.insert(IO_BASE_ONE.to_string(), GenArg::Int(1));
        }
        env.extend(range_ends.iter().cloned());
        let id = self.declare_fn_in(decl, tpl_mod, tpl_offset, owner, env);
        let suffix: Vec<String> = args.iter().map(|a| self.mangle_arg(a)).collect();
        self.record_instance(id, gfid, &args);
        self.sigs[id].name = format!("{}_{}_{id}", self.sigs[id].name, suffix.join("_"));
        self.insts.insert(key, id);
        self.resolve_fn_sig(id, None);
        id
    }

    /// Whether the fn being lowered is `m`'s, or an instance a fn of `m`
    /// asked for, directly or through other instances.
    pub(super) fn lowered_for(&self, m: usize) -> bool {
        let mut at = self.growth.cur;
        while let Some(id) = at {
            if self.sigs[id].mod_id == m || self.growth.from.get(&id) == Some(&m) {
                return true;
            }
            at = self.growth.parent.get(&id).and_then(|p| p.2);
        }
        false
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
        self.growth.from.insert(id, self.cur_mod);
    }

    /// E0319 at a call of `name` that would make one more instance in a
    /// chain that grows without end.
    pub(super) fn growing_call(&mut self, gfid: GenFnId, name: &str, span: Span) -> Option<Expr> {
        let (outer, inner) = self.growing_instance(gfid)?;
        self.report(
            span,
            msgs::ir::GROWING_INSTANTIATION_CALLS_ITSELF_AT_A,
            &[&name, &name, &outer, &name, &inner],
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

/// The instance env's mark that the caller reads under `io(base: 1)`.
pub(super) const IO_BASE_ONE: &str = "__io_base_one";

/// Whether a generic fn bounds one of its type parameters by `Read`.
pub(super) fn reads_a_param(decl: &ast::FnDecl) -> bool {
    decl.bounds.iter().any(|b| {
        b.traits
            .iter()
            .any(|t| matches!(&t.kind, ast::TypeKind::Named { name, .. } if name.text == "Read"))
    })
}

//! Resolving written type arguments into concrete generic arguments.

use super::*;
use cpl_diag::msgs;

impl<'a> Lowerer<'a> {
    /// Resolves a list of AST type arguments into concrete `GenArg`s.
    pub(in crate::lower) fn resolve_gen_args(&mut self, args: &[ast::Type]) -> Vec<GenArg> {
        args.iter().map(|t| self.resolve_gen_arg(t)).collect()
    }

    /// A generic fn's type arguments; a `T: Read` given an array of fixed
    /// lengths keeps them for its `read<T>()`.
    pub(in crate::lower) fn fn_gen_args(
        &mut self,
        gfid: GenFnId,
        args: &[ast::Type],
    ) -> Vec<GenArg> {
        let decl = self.gen_fns[gfid].decl;
        let mut out = self.resolve_gen_args(args);
        for (i, t) in args.iter().enumerate() {
            let Some(param) = decl.generics.get(i) else {
                break;
            };
            let reads = decl.bounds.iter().any(|b| {
                b.name.text == param.text
                    && b.traits.iter().any(|t| {
                        matches!(&t.kind, ast::TypeKind::Named { name, .. } if name.text == "Read")
                    })
            });
            let ast::TypeKind::Array { index, .. } = &t.kind else {
                continue;
            };
            let lens: Option<Vec<i64>> = index
                .iter()
                .map(|d| match &d.kind {
                    ast::TypeKind::Value(e) => self.eval_const_int(e),
                    ast::TypeKind::Range {
                        lo: Some(lo),
                        hi: Some(hi),
                        inclusive,
                    } => {
                        let (lo, hi) = (self.eval_const_int(lo)?, self.eval_const_int(hi)?);
                        Some(hi - lo + i64::from(*inclusive))
                    }
                    _ => None,
                })
                .collect();
            if let (true, Some(lens), GenArg::Ty(ty)) = (reads, lens, &out[i]) {
                out[i] = GenArg::Sized(ty.clone(), lens);
            }
        }
        out
    }

    pub(in crate::lower) fn resolve_gen_args_for_struct(
        &mut self,
        gid: usize,
        args: &[ast::Type],
    ) -> Vec<GenArg> {
        let decl = self.gen_structs[gid].decl;
        args.iter()
            .enumerate()
            .map(|(i, t)| {
                if let Some(d) = self.anon_range_domain(decl, i, t) {
                    return GenArg::Domain(d);
                }
                if matches!(
                    &t.kind,
                    ast::TypeKind::Tuple(items) if !items.is_empty()
                ) || matches!(
                    &t.kind,
                    ast::TypeKind::Record(fields) if !fields.is_empty()
                ) {
                    let is_domain_param = self.uses_param_as_domain(decl, i);
                    if !is_domain_param {
                        return GenArg::Ty(self.resolve_ty(t));
                    }
                }
                self.resolve_gen_arg(t)
            })
            .collect()
    }

    pub(in crate::lower) fn resolve_gen_arg(&mut self, ty: &ast::Type) -> GenArg {
        match &ty.kind {
            ast::TypeKind::Value(expr) => {
                if let Some(v) = self.eval_const_int(expr) {
                    return GenArg::Int(v);
                }
                // Could be a domain or type name written in value position.
                if let ast::ExprKind::Name(n) = &expr.kind {
                    if let Some(a) = self.gen_env.get(n).cloned() {
                        return a;
                    }
                    if let Some(Sym::Domain(d)) = self.get_sym(n) {
                        return GenArg::Domain(d);
                    }
                    if let Some(Sym::Global(gid)) = self.get_sym(n) {
                        if self.globals[gid].ty == Ty::Int {
                            return GenArg::RuntimeInt(gid);
                        }
                    }
                }
                self.report(expr.span, msgs::ir::BAD_NUMBER_EXPECTED_A_COMPILE_TIME, &[]);
                GenArg::Int(1)
            }
            ast::TypeKind::Named { name, args } if args.is_empty() => {
                if let Some(a) = self.named_gen_arg(&name.text) {
                    return a;
                }
                // `Comb<Z.MOD>`: a number a type holds, by the type's name.
                if let Some((owner, member)) = name.text.split_once('.') {
                    if let Some(v) = self.const_of_type(owner, member, name.span) {
                        return GenArg::Int(v);
                    }
                }
                if let Some(Sym::Domain(d)) = self.get_sym(&name.text) {
                    return GenArg::Domain(d);
                }
                // A `const` with a known value is that number, so `Comb<MOD>`
                // and `Mod<998244353>` share one `Mod`.
                if let Some(v) = self.const_ints.get(&name.text) {
                    return GenArg::Int(*v);
                }
                if let Some(Sym::Global(gid)) = self.get_sym(&name.text) {
                    if self.globals[gid].ty == Ty::Int {
                        return GenArg::RuntimeInt(gid);
                    }
                }
                if let Some(v) = self.const_ints.get(&name.text).copied() {
                    return GenArg::Int(v);
                }
                if let Some(Sym::ModConst(cid)) = self.get_sym(&name.text) {
                    if let Some(v) = self.eval_const_int(self.mod_consts[cid].init) {
                        return GenArg::Int(v);
                    }
                }
                let resolved = self.resolve_ty(ty);
                match resolved {
                    Ty::Pos(d) => GenArg::Domain(d),
                    other => GenArg::Ty(other),
                }
            }
            ast::TypeKind::Range { lo, hi, inclusive } => {
                let lo_val = lo.as_deref().and_then(|e| self.eval_const_int(e));
                let hi_val = hi.as_deref().and_then(|e| {
                    let v = self.eval_const_int(e)?;
                    Some(if *inclusive { v } else { v - 1 })
                });
                if lo_val.is_some() || hi_val.is_some() {
                    return GenArg::Range(lo_val, hi_val);
                }
                GenArg::Ty(Ty::Int)
            }
            ast::TypeKind::Tuple(items) if !items.is_empty() => {
                let resolved_items: Vec<Ty> = items.iter().map(|t| self.resolve_ty(t)).collect();
                let factors: Option<Vec<(Option<String>, ProductFactor)>> = resolved_items
                    .iter()
                    .map(|t| self.ty_to_product_factor(t).map(|f| (None, f)))
                    .collect();
                if let Some(factors) = factors {
                    let d = self.intern_anon_product_domain(factors);
                    return GenArg::Domain(d);
                }
                GenArg::Ty(Ty::Tuple(resolved_items))
            }
            ast::TypeKind::Record(fields) if !fields.is_empty() => {
                let resolved_fields: Vec<(String, Ty)> = fields
                    .iter()
                    .map(|(n, t)| (n.text.clone(), self.resolve_ty(t)))
                    .collect();
                let factors: Option<Vec<(Option<String>, ProductFactor)>> = resolved_fields
                    .iter()
                    .map(|(n, t)| self.ty_to_product_factor(t).map(|f| (Some(n.clone()), f)))
                    .collect();
                if let Some(factors) = factors {
                    let d = self.intern_anon_product_domain(factors);
                    return GenArg::Domain(d);
                }
                GenArg::Ty(Ty::Record(resolved_fields))
            }
            ast::TypeKind::Optional(inner)
                if matches!(&inner.kind, ast::TypeKind::Range { .. })
                    || matches!(&inner.kind, ast::TypeKind::Named { args, .. } if args.is_empty()) =>
            {
                match self.resolve_gen_arg(inner) {
                    GenArg::Range(lo, hi) => GenArg::OptRange(lo, hi),
                    _ => GenArg::Ty(self.resolve_ty(ty)),
                }
            }
            _ => {
                let resolved = self.resolve_ty(ty);
                match resolved {
                    Ty::Pos(d) => GenArg::Domain(d),
                    other => GenArg::Ty(other),
                }
            }
        }
    }

    /// The number `Owner.member` names: a generic number of the type
    /// (`Mod<P>`'s `P`) or one of its `let` constants (`Z.MOD`).
    pub(in crate::lower) fn const_of_type(
        &mut self,
        owner: &str,
        member: &str,
        span: Span,
    ) -> Option<i64> {
        let owner = ast::Type {
            kind: ast::TypeKind::Named {
                name: ast::Name {
                    text: owner.to_string(),
                    span,
                },
                args: Vec::new(),
            },
            span,
        };
        let dlen = self.diags.len();
        let resolved = self.resolve_ty(&owner);
        self.diags.truncate(dlen);
        let Ty::Struct(sid) = resolved else {
            return None;
        };
        if let Some(GenArg::Int(v)) = self.structs[sid].env.get(member) {
            return Some(*v);
        }
        let init = self.structs[sid].consts.get(member)?.init;
        let env = self.structs[sid].env.clone();
        let outer = std::mem::replace(&mut self.gen_env, env);
        let value = self.eval_const_int(init);
        self.gen_env = outer;
        value
    }

    /// The fields of `decl` that are a row of 1 to 64 cells, the count
    /// written as numbers: `kid: [0..2]?Int`, `cnt: [1..=26]Int`.
    /// What a zero-argument alias declared in this module names: `Sign` in
    /// `type Sign = -1..=1` is the range `-1..=1`.
    pub(in crate::lower) fn local_alias_target(&self, ty: &ast::Type) -> Option<&'a ast::Type> {
        let ast::TypeKind::Named { name, args } = &ty.kind else {
            return None;
        };
        let Some(Sym::Alias(aid)) = self.mod_syms[self.cur_mod].get(&name.text) else {
            return None;
        };
        let (m, aliased, _) = self.aliases[*aid];
        (args.is_empty() && m == self.cur_mod).then_some(aliased)
    }

    pub(in crate::lower) fn fixed_rows(
        &self,
        decl: &ast::StructDecl,
    ) -> std::collections::BTreeMap<usize, u32> {
        let mut rows = std::collections::BTreeMap::new();
        for (at, field) in decl.fields.iter().enumerate() {
            let ast::TypeKind::Array { index, .. } = &field.ty.kind else {
                continue;
            };
            let [index] = index.as_slice() else {
                continue;
            };
            let index = self.local_alias_target(index).unwrap_or(index);
            let ast::TypeKind::Range {
                lo: Some(lo),
                hi: Some(hi),
                inclusive,
            } = &index.kind
            else {
                continue;
            };
            let (Some(lo), Some(hi)) = (self.eval_const_int(lo), self.eval_const_int(hi)) else {
                continue;
            };
            let len = hi.saturating_sub(lo).saturating_add(i64::from(*inclusive));
            if (1..=64).contains(&len) {
                rows.insert(at, len as u32);
            }
        }
        rows
    }
}

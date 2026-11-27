//! Inherent impls: `impl T:` blocks, whose fns are methods of `T` (R08.1.3).
use super::{add_fn, unnamed, Decls, ModId, Owner, StructId, Within};
use crate::ty::*;
use cpl_syntax::ast::{self, StmtKind, TypeKind};

/// `impl<F: Tag> S<F>:` adds its fns to `S`'s methods, its parameters standing for `S`'s and
/// its bounds holding of each fn as if written on it (R08.1.3).
pub(super) fn inherent_impl(
    d: &mut Decls,
    mid: ModId,
    item: u32,
    t: StructId,
    args: &[ast::Type],
    bounds: &[ast::Bound],
    body: &ast::Block,
) {
    let names = &d.views[mid.0];
    let mut generics = Vec::new();
    for a in args {
        unnamed(a, names, &mut generics);
    }
    let plain =
        (args.iter()).all(|a| matches!(&a.kind, TypeKind::Named { args, .. } if args.is_empty()));
    if !plain || generics.len() != args.len() || args.len() != d.s(t).generics.len() {
        // The library's `impl Rev<[D]T>` waits on the same: lookup that matches a receiver
        // against an impl's header, as a trait impl's is.
        if d.is_library(mid) {
            return;
        }
        d.errors.push(
            format!(
                "slice: an inherent impl of `{}` not over all its parameters",
                d.s(t).name
            )
            .into(),
        );
        return;
    }
    // A parameter only its bounds name (`D` of `impl<S: Versioned<D, T>, D> Version<S>`) is
    // each fn's own, bound where the fn is called (R08.1.2).
    let mut only_bounds: Vec<String> = Vec::new();
    for b in bounds {
        let mut named = vec![b.name.text.clone()];
        b.traits.iter().for_each(|t| unnamed(t, names, &mut named));
        for n in named {
            if !generics.contains(&n) && !only_bounds.contains(&n) && !names.contains_key(&n) {
                only_bounds.push(n);
            }
        }
    }
    let n_sets = d.s(t).n_sets;
    let self_ty = Ty::Struct(
        t,
        (0..generics.len()).map(Ty::Param).collect(),
        (0..n_sets).map(SetTy::Param).collect(),
    );
    for m in &body.stmts {
        let StmtKind::Fn(f) = &m.kind else {
            continue;
        };
        let mut f = f.clone();
        f.bounds.splice(0..0, bounds.iter().cloned());
        let own = (only_bounds.iter()).map(|n| ast::Name {
            text: n.clone(),
            span: f.name.span,
        });
        f.generics.splice(0..0, own);
        let within = Within {
            owner: Some(Owner::Struct(t)),
            generics: &generics,
            sets: n_sets,
            self_ty: Some(self_ty.clone()),
            sig: None,
        };
        let fid = add_fn(d, mid, item, m, &f, within);
        if d.structs[t.0]
            .methods
            .insert(f.name.text.clone(), fid)
            .is_some()
        {
            d.misdeclared.push((
                (item + m.span.start) as usize,
                cpl_diag::codes::OVERLOAD_BY_TYPE.0,
                format!("`{}` is already a method of `{}`", f.name.text, d.s(t).name),
            ));
        }
    }
}

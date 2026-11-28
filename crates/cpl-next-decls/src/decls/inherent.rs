//! Inherent impls: `impl T:` blocks, whose fns are methods of `T` (R08.1.3).
use super::{add_fn, unnamed, Decls, ModId, Owner, StructId, Within};
use crate::ty::*;
use crate::views::Scope;
use cpl_diag::Span;
use cpl_next_ir::ids::ItemId;
use cpl_syntax::ast::{self, StmtKind, TypeKind};

/// `impl<F: Tag> S<F>:` adds its fns to `S`'s methods, its parameters standing for `S`'s and
/// its bounds holding of each fn as if written on it (R08.1.3).
pub(super) fn inherent_impl(
    d: &mut Decls,
    mid: ModId,
    item: ItemId,
    t: StructId,
    target: &ast::Type,
    bounds: &[ast::Bound],
    body: &ast::Block,
) {
    let TypeKind::Named { args, .. } = &target.kind else {
        return;
    };
    let names = &|n: &str| d.sym(n).and_then(|s| d.named(mid, s));
    let mut generics = Vec::new();
    for a in args {
        unnamed(a, names, &mut generics);
    }
    let plain =
        (args.iter()).all(|a| matches!(&a.kind, TypeKind::Named { args, .. } if args.is_empty()));
    if !plain || generics.len() != args.len() || args.len() != d.s(t).generics.len() {
        return shaped_impl(d, mid, item, t, target, bounds, body);
    }
    // A parameter only its bounds name (`D` of `impl<S: Versioned<D, T>, D> Version<S>`) is
    // each fn's own, bound where the fn is called (R08.1.2).
    let mut only_bounds: Vec<String> = Vec::new();
    for b in bounds {
        let mut named = vec![b.name.text.clone()];
        b.traits.iter().for_each(|t| unnamed(t, names, &mut named));
        for n in named {
            if !generics.contains(&n) && !only_bounds.contains(&n) && names(&n).is_none() {
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
        let fid = add_fn(d, mid, item.clone(), m, &f, within);
        let n = d.syms.intern(&f.name.text);
        if d.structs[t.0].methods.insert(n, fid).is_some() {
            d.misdeclared.push((
                item.at(Span::new(m.span.start, m.span.start)),
                cpl_diag::codes::OVERLOAD_BY_TYPE,
                format!("`{}` is already a method of `{}`", f.name.text, d.s(t).name),
            ));
        }
    }
}

/// `impl Rev<[D]T>:` adds each fn as one whose `self` is `Rev<[D]T>`, the header's parameters
/// its own: the struct's method for a receiver of that shape (R08.1.3).
fn shaped_impl(
    d: &mut Decls,
    mid: ModId,
    item: ItemId,
    t: StructId,
    target: &ast::Type,
    bounds: &[ast::Bound],
    body: &ast::Block,
) {
    let mut generics = Vec::new();
    unnamed(
        target,
        &|n: &str| d.sym(n).and_then(|s| d.named(mid, s)),
        &mut generics,
    );
    for m in &body.stmts {
        let StmtKind::Fn(f) = &m.kind else {
            continue;
        };
        let mut f = f.clone();
        f.bounds.splice(0..0, bounds.iter().cloned());
        let own = generics.iter().map(|n| ast::Name {
            text: n.clone(),
            span: f.name.span,
        });
        f.generics.splice(0..0, own);
        if let Some(p) = f.params.iter_mut().find(|p| p.name.text == "self") {
            p.ty.get_or_insert_with(|| target.clone());
        }
        let within = Within {
            owner: None,
            generics: &[],
            sets: 0,
            self_ty: None,
            sig: None,
        };
        let fid = add_fn(d, mid, item.clone(), m, &f, within);
        let name = d.syms.intern(&f.name.text);
        d.structs[t.0].shaped.entry(name).or_default().push(fid);
    }
}

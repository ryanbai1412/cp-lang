//! Inherent impls: `impl T:` blocks, whose fns are methods of `T` (R08.1.3).
use super::forms::{ImplForm, TyForm};
use super::{add_fn, unnamed, Decls, ModId, Owner, StructId, Within};
use crate::sym::Sym;
use crate::ty::*;
use cpl_diag::Span;
use cpl_next_ir::ids::ItemId;
use cpl_next_syntax::ast::{GenericArg, Type};

/// `impl<F: Tag> S<F>:` adds its fns to `S`'s methods, its parameters standing for `S`'s and
/// its bounds holding of each fn as if written on it (R08.1.3).
pub(super) fn inherent_impl(d: &mut Decls, mid: ModId, item: ItemId, t: StructId, form: &ImplForm) {
    let TyForm::Written(Type::Path(p)) = &form.self_ty else {
        return;
    };
    let args: Vec<GenericArg> = p
        .generic_args()
        .map(|a| a.args().collect())
        .unwrap_or_default();
    let view = d.views[mid.0].clone();
    let names = |n: Sym| view.get(&n).copied();
    let mut generics = Vec::new();
    for a in &args {
        match a {
            GenericArg::Type(a) => unnamed(a, &names, &mut d.syms, &mut generics),
            GenericArg::Param(q) if !q.is_const() => {
                let n = q.name().map(|n| d.syms.intern(n.text()));
                generics.extend(n.filter(|n| !generics.contains(n)));
            }
            GenericArg::Param(_) => {}
        }
    }
    let bare = |a: &GenericArg| match a {
        GenericArg::Type(Type::Path(q)) => {
            q.generic_args().is_none_or(|g| g.args().next().is_none())
        }
        GenericArg::Param(q) => !q.is_const(),
        _ => false,
    };
    if !args.iter().all(bare) || generics.len() != args.len() || args.len() != d.s(t).generics.len()
    {
        return shaped_impl(d, mid, item, t, form);
    }
    // A parameter only its bounds name (`D` of `impl<S: Versioned<D, T>, D> Version<S>`) is
    // each fn's own, bound where the fn is called (R08.1.2).
    let mut only_bounds: Vec<Sym> = Vec::new();
    for b in &form.bounds {
        let mut named = vec![b.param];
        if let TyForm::Written(bt) = &b.tr {
            unnamed(bt, &names, &mut d.syms, &mut named);
        }
        for n in named {
            if !generics.contains(&n) && !only_bounds.contains(&n) && names(n).is_none() {
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
    for m in &form.methods {
        let mut f = m.clone();
        f.bounds.splice(0..0, form.bounds.iter().cloned());
        f.generics.splice(0..0, only_bounds.iter().copied());
        let within = Within {
            owner: Some(Owner::Struct(t)),
            generics: &generics,
            sets: n_sets,
            self_ty: Some(self_ty.clone()),
            sig: None,
        };
        let fid = add_fn(d, mid, item.clone(), &f, within);
        if d.structs[t.0].methods.insert(f.name, fid).is_some() {
            let msg = format!(
                "`{}` is already a method of `{}`",
                d.syms.text(f.name),
                d.s(t).name
            );
            d.misdeclared.push((
                item.at(Span::new(f.span.start, f.span.start)),
                cpl_diag::codes::OVERLOAD_BY_TYPE,
                msg,
            ));
        }
    }
}

/// `impl Rev<[D]T>:` adds each fn as one whose `self` is `Rev<[D]T>`, the header's parameters
/// its own: the struct's method for a receiver of that shape (R08.1.3).
fn shaped_impl(d: &mut Decls, mid: ModId, item: ItemId, t: StructId, form: &ImplForm) {
    let view = d.views[mid.0].clone();
    let names = |n: Sym| view.get(&n).copied();
    let mut generics = Vec::new();
    if let TyForm::Written(target) = &form.self_ty {
        unnamed(target, &names, &mut d.syms, &mut generics);
    }
    for m in &form.methods {
        let mut f = m.clone();
        f.bounds.splice(0..0, form.bounds.iter().cloned());
        f.generics.splice(0..0, generics.iter().copied());
        if let Some(p) = f.params.iter_mut().find(|p| p.is_self) {
            p.ty.get_or_insert_with(|| form.self_ty.clone());
        }
        let within = Within {
            owner: None,
            generics: &[],
            sets: 0,
            self_ty: None,
            sig: None,
        };
        let fid = add_fn(d, mid, item.clone(), &f, within);
        d.structs[t.0].shaped.entry(f.name).or_default().push(fid);
    }
}

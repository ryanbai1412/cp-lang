//! Pass two of collecting declarations for a struct or a trait, read from its form: its fields'
//! types and set parameters, its default type arguments, its associated types, its supertraits
//! and its methods.
use super::forms::{FieldForm, StructForm, TraitForm, TyForm};
use super::tyconv::{self, conv_form, input_bound, shift_down, trait_ref, TyCx};
use super::*;
use crate::attrs::Decorator;
use cpl_diag::codes::FIELD_BOUND_FROM_INPUT;
use cpl_diag::msgs::check::FIELD_BOUND_FROM_INPUT_A_FIELD_S_BOUNDS;
use cpl_next_ir::ids::ItemId;
use cpl_next_syntax::ast::AstNode as _;

/// The domains module `m`'s types may name, by name.
pub(super) fn doms(d: &mut Decls, m: ModId) -> HashMap<Sym, Ty> {
    d.doms_in(m)
}

/// Where type `t` is written, or `at` for one the compiler made.
fn ty_at(t: &TyForm, at: Span) -> Span {
    match t {
        TyForm::Written(t) => t.syntax().span(),
        _ => at,
    }
}

/// How many set parameters struct `sd`'s fields take, as far as the structs it holds are known:
/// one per field of fn type, and those of each struct a field holds.
pub(super) fn struct_sets(d: &mut Decls, mid: ModId, sd: &StructForm) -> usize {
    let doms = doms(d, mid);
    let view = d.views[mid.0].clone();
    let tables = d.tables();
    let mut cx = TyCx {
        axes: None,
        names: &|n: Sym| view.get(&n).copied(),
        env: &tables,
        syms: &tables.syms,
        item: None,
        generics: &sd.generics,
        outer: &sd.generics,
        args: None,
        expanding: Vec::new(),
        doms: &doms,
        scoped: &tyconv::unscoped,
        self_ty: None,
        rows: None,
        ret_slots: None,
        next_set: Some(0),
    };
    let mut errors = Vec::new();
    for f in &sd.fields {
        conv_form(&mut cx, &f.ty, &mut errors);
    }
    cx.next_set.unwrap_or(0)
}

/// Struct `id`'s default type arguments, which name its earlier parameters and any struct.
pub(super) fn struct_defaults(d: &mut Decls, mid: ModId, id: StructId, sd: &StructForm) {
    if sd.generic_defaults.iter().all(Option::is_none) {
        return;
    }
    let doms = doms(d, mid);
    let view = d.views[mid.0].clone();
    let tables = d.tables();
    let mut cx = TyCx {
        axes: None,
        names: &|n: Sym| view.get(&n).copied(),
        env: &tables,
        syms: &tables.syms,
        item: None,
        generics: &sd.generics,
        outer: &sd.generics,
        args: None,
        expanding: Vec::new(),
        doms: &doms,
        scoped: &tyconv::unscoped,
        self_ty: None,
        rows: None,
        ret_slots: None,
        next_set: None,
    };
    let mut errors = Vec::new();
    let tys = (sd.generic_defaults.iter())
        .map(|t| t.as_ref().map(|t| conv_form(&mut cx, t, &mut errors)))
        .collect();
    Arc::make_mut(&mut d.struct_defaults).insert(id, tys);
    d.structs[id.0].errors.extend(errors);
}

/// Struct `id`'s fields, bounds and methods.
pub(super) fn struct_body(d: &mut Decls, mid: ModId, item: &ItemId, id: StructId, sd: &StructForm) {
    let doms = doms(d, mid);
    let view = d.views[mid.0].clone();
    let tables = d.tables();
    let mut cx = TyCx {
        axes: None,
        names: &|n: Sym| view.get(&n).copied(),
        env: &tables,
        syms: &tables.syms,
        item: None,
        generics: &sd.generics,
        outer: &sd.generics,
        args: None,
        expanding: Vec::new(),
        doms: &doms,
        scoped: &tyconv::unscoped,
        self_ty: None,
        rows: None,
        ret_slots: None,
        next_set: Some(0),
    };
    let mut errors = Vec::new();
    let mut fields: Vec<(Sym, Ty)> = Vec::new();
    for f in &sd.fields {
        let bound = match &f.ty {
            TyForm::Written(t) => input_bound(t, &tables.values, &tables.syms),
            _ => None,
        };
        let ty = match bound {
            Some((written, name)) => {
                let field = tables.syms.text(f.name).to_string();
                let text = FIELD_BOUND_FROM_INPUT_A_FIELD_S_BOUNDS.say(&[&field, &written, &name]);
                let at = item.at(ty_at(&f.ty, f.span));
                errors.push(Msg::at(at, Msg::verdict(FIELD_BOUND_FROM_INPUT, text)));
                Ty::Error
            }
            None => conv_form(&mut cx, &f.ty, &mut errors),
        };
        fields.push((f.name, ty));
    }
    let n_sets = cx.next_set.unwrap_or(0);
    d.structs[id.0].bounds = (sd.bounds.iter())
        .filter_map(|b| {
            let i = sd.generics.iter().position(|g| *g == b.param)?;
            let (n, _) = bound_trait(&d.syms, &b.tr)?;
            match view.get(&n) {
                Some(Named::Trait(tr)) => Some((i, *tr)),
                _ => None,
            }
        })
        .collect();
    d.structs[id.0].errors.extend(errors);
    let self_ty = Ty::Struct(
        id,
        (0..sd.generics.len()).map(Ty::Param).collect(),
        (0..n_sets).map(SetTy::Param).collect(),
    );
    let mut methods = HashMap::new();
    for m in &sd.methods {
        // The struct's bounds hold in each of its methods (`struct Range<D: Domain>`).
        let mut f = m.clone();
        f.bounds.splice(0..0, sd.bounds.iter().cloned());
        let within = Within {
            owner: Some(Owner::Struct(id)),
            generics: &sd.generics,
            sets: n_sets,
            self_ty: Some(self_ty.clone()),
            sig: None,
        };
        let fid = add_fn(d, mid, item.clone(), &f, within);
        methods.insert(f.name, fid);
    }
    for l in &sd.laws {
        let within = Within {
            owner: Some(Owner::Struct(id)),
            generics: &sd.generics,
            sets: n_sets,
            self_ty: Some(self_ty.clone()),
            sig: None,
        };
        add_fn(d, mid, item.clone(), l, within);
    }
    let ctor = d.member(&methods, "new");
    let cached = |f: &FieldForm| f.attrs.iter().any(|a| a.attr == Decorator::Cache);
    let sdecl = &mut d.structs[id.0];
    sdecl.fields = fields;
    sdecl.var_fields = sd.fields.iter().map(|f| f.var).collect();
    sdecl.pub_fields = sd.fields.iter().map(|f| f.public).collect();
    sdecl.cached = sd.fields.iter().map(cached).collect();
    sdecl.defaults = sd.fields.iter().map(|f| f.default.clone()).collect();
    sdecl.field_at = (sd.fields.iter())
        .map(|f| {
            let s = ty_at(&f.ty, f.span);
            item.at(Span::new(s.start, s.start))
        })
        .collect();
    sdecl.n_sets = n_sets;
    sdecl.ctor = ctor;
    sdecl.methods = methods;
}

/// Trait `id`'s defaults, associated types, supertraits and methods. Its parameters are `Self`,
/// its own, then `Self.A` for each associated type `A`.
pub(super) fn trait_body(d: &mut Decls, mid: ModId, item: &ItemId, id: TraitId, t: &TraitForm) {
    let mut tgen = vec![d.syms.intern("Self")];
    tgen.extend(t.generics.iter().copied());
    for a in &t.assoc {
        let n = format!("Self.{}", d.text(a.name));
        tgen.push(d.syms.intern(&n));
    }
    let doms = doms(d, mid);
    let view = d.views[mid.0].clone();
    let tables = d.tables();
    let mut cx = TyCx {
        axes: None,
        names: &|n: Sym| view.get(&n).copied(),
        env: &tables,
        syms: &tables.syms,
        item: None,
        generics: &tgen,
        outer: &tgen,
        args: None,
        expanding: Vec::new(),
        doms: &doms,
        scoped: &tyconv::unscoped,
        self_ty: Some(Ty::Param(0)),
        rows: None,
        ret_slots: None,
        next_set: None,
    };
    let mut errors = Vec::new();
    let mut defaults: Vec<Option<Ty>> = (t.generic_defaults.iter())
        .map(|g| g.as_ref().map(|g| conv_form(&mut cx, g, &mut errors)))
        .collect();
    // `type Taken = Slice<Self>`: what an impl that leaves it out has.
    for a in &t.assoc {
        defaults.push(
            a.default
                .as_ref()
                .map(|g| conv_form(&mut cx, g, &mut errors)),
        );
    }
    let first = tgen.len() - t.assoc.len();
    let mut assoc_bounds = Vec::new();
    for (k, a) in t.assoc.iter().enumerate() {
        for b in &a.bounds {
            if let Some(r) = trait_ref(&mut cx, b, &mut errors) {
                assoc_bounds.push((first + k, r));
            }
        }
    }
    let supers: Vec<TraitRef> = (t.supers.iter())
        .filter_map(|s| trait_ref(&mut cx, s, &mut errors))
        .collect();
    d.traits[id.0].defaults = defaults;
    let assoc_bounds = (assoc_bounds.into_iter())
        .map(|(p, r)| (p, d.with_defaults(r, &Ty::Param(p))))
        .collect();
    // A supertrait is in the trait's own parameters, without `Self`.
    let supers = (supers.into_iter())
        .map(|r| d.with_defaults(r, &Ty::Param(0)))
        .map(|r| TraitRef {
            tr: r.tr,
            args: r.args.iter().map(shift_down).collect(),
        })
        .collect();
    d.traits[id.0].assoc = t.assoc.iter().map(|a| d.text(a.name).to_string()).collect();
    d.traits[id.0].assoc_bounds = assoc_bounds;
    d.module_or_program_errors(mid, errors);
    let mut methods = HashMap::new();
    let laws = t.laws.iter().map(|l| (l, true));
    for (m, laws) in t.methods.iter().map(|m| (m, false)).chain(laws) {
        let within = Within {
            owner: Some(Owner::Trait(id)),
            generics: &tgen,
            sets: 0,
            self_ty: Some(Ty::Param(0)),
            sig: None,
        };
        let fid = add_fn(d, mid, item.clone(), m, within);
        // Inside the trait, `Self` has the trait: a default body may call its other fns.
        let own = TraitRef {
            tr: id,
            args: (1..tgen.len()).map(Ty::Param).collect(),
        };
        d.fns[fid.0].bounds.push((0, own));
        if !laws {
            methods.insert(m.name, fid);
        }
    }
    d.traits[id.0].methods = methods;
    d.traits[id.0].supers = supers;
}

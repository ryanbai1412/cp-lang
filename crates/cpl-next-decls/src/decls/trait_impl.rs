//! Pass two for a trait impl, read from its form: its parameters, header, associated types,
//! bounds and methods, checked against the impls already made (R08.1).
use super::forms::{FnForm, Form, ImplForm, TyForm};
use super::impls::{overlap, same_shape};
use super::tyconv::{self, conv_form, trait_ref, TyCx};
use super::Impls;
use super::*;
use cpl_next_ir::ids::ItemId;

/// What became of an impl: made, refused or reported, or waiting on its fields' impls.
pub(super) enum Made {
    Done,
    Wait,
}

/// A trait impl to make, and where.
pub(super) struct Of<'a> {
    pub mid: ModId,
    pub item: &'a ItemId,
    pub form: &'a ImplForm,
    pub tr: &'a TyForm,
    /// The struct a derived impl is made for.
    pub derived_for: Option<Named>,
    /// Whether the derived impl copies its struct's `@cache` fields.
    pub copies: bool,
    /// The first impl this layer makes: those before are std's.
    pub impls_from: usize,
    /// The module's written forms, which an impl's `let` may not read a `var` of.
    pub top: &'a [(ItemId, Form)],
}

pub(super) fn trait_impl(d: &mut Decls, of: Of) -> Made {
    let Of {
        mid,
        item,
        form,
        tr,
        derived_for: id,
        copies,
        impls_from,
        top,
    } = of;
    let view = d.views[mid.0].clone();
    let names = |n: Sym| view.get(&n).copied();
    // The impl's own type parameters: those it declares, then what its header and bounds write
    // that names nothing (`K` of `impl<S: Indexed<K, T>, K, T>`).
    let mut generics: Vec<Sym> = form.generics.clone();
    let written = [tr, &form.self_ty]
        .into_iter()
        .chain(form.bounds.iter().map(|b| &b.tr));
    for t in written {
        if let TyForm::Written(t) = t {
            unnamed(t, &names, &mut d.syms, &mut generics);
        }
    }
    let mut heads: Vec<(Sym, usize, TraitId, usize)> = Vec::new();
    for b in &form.bounds {
        if let Some((n, k)) = bound_trait(&d.syms, &b.tr) {
            if let Some(Named::Trait(t)) = names(n) {
                let j = heads.iter().filter(|h| h.0 == b.param).count();
                heads.push((b.param, j, t, k));
            }
        }
    }
    let hidden = hidden_params(d, &heads, &mut generics);
    // `_..=_`: a parameter of the impl's own that only a range type meets, which its body
    // names `Self`.
    let range_param = super::collect::any_range(&form.self_ty).then(|| {
        generics.push(d.syms.intern("Self"));
        generics.len() - 1
    });
    let doms = super::structs::doms(d, mid);
    let tables = d.tables();
    let mut errors = Vec::new();
    let mut cx = TyCx {
        axes: None,
        names: &names,
        env: &tables,
        item: Some(item.clone()),
        generics: &generics,
        outer: &generics,
        args: None,
        expanding: Vec::new(),
        doms: &doms,
        scoped: &tyconv::unscoped,
        self_ty: None,
        rows: None,
        ret_slots: None,
        next_set: Some(0),
    };
    let for_ = match range_param {
        Some(i) => Ty::Param(i),
        None => conv_form(&mut cx, &form.self_ty, &mut errors),
    };
    cx.self_ty = Some(for_.clone());
    let head = match range_param {
        Some(i) => Ty::AnyRange(i),
        None => for_.clone(),
    };
    let tref = trait_ref(&mut cx, tr, &mut errors).map(|r| d.with_defaults(r, &for_));
    let start = item.at(Span::new(form.span.start, form.span.start));
    // An associated type is the argument the impl's body writes: `type Arc = A`.
    let mut assoc_at = Vec::new();
    let mut missing = Vec::new();
    let tref = tref.map(|mut r| {
        let assoc = d.traits[r.tr.0].assoc.clone();
        let n = d.traits[r.tr.0].defaults.len() - assoc.len();
        if r.args.len() >= n {
            r.args.truncate(n);
            for (k, a) in assoc.iter().enumerate() {
                let written = form.assoc.iter().find(|(name, ..)| d.text(*name) == a);
                assoc_at.push(written.map_or(start.clone(), |(_, sp, _)| {
                    item.at(Span::new(sp.start, sp.start))
                }));
                let default = d.traits[r.tr.0].defaults[n + k].clone();
                match (written, default) {
                    (Some((.., t)), _) => r.args.push(conv_form(&mut cx, t, &mut errors)),
                    (None, Some(t)) => {
                        let mut tys = vec![for_.clone()];
                        tys.extend(r.args.iter().cloned());
                        r.args.push(
                            Subst {
                                tys: &tys,
                                sets: &[],
                            }
                            .ty(&t),
                        );
                    }
                    (None, None) => {
                        let tname = &d.traits[r.tr.0].name;
                        missing.push(format!(
                            "`impl {tname}` does not say its associated type `{a}` (R08.1.9)"
                        ));
                        r.args.push(Ty::Error);
                    }
                }
            }
        }
        r
    });
    let mut ibounds = Vec::new();
    let mut nth: HashMap<Sym, usize> = HashMap::new();
    for b in &form.bounds {
        let i = generics.iter().position(|g| *g == b.param).unwrap_or(0);
        if let Some(r) = trait_ref(&mut cx, &b.tr, &mut errors) {
            let j = nth.entry(b.param).or_insert(0);
            bound_ref(d, r, i, (b.param, *j), &hidden, &mut ibounds);
            *j += 1;
        }
    }
    // A range type is a domain with an order (R04.2.31), so `Self` in a `_..=_` impl's fns
    // meets `Domain` and `Ordered`; the impl itself is not bound by them.
    let mut fn_bounds = ibounds.clone();
    if let Some(i) = range_param {
        for tr in [d.roles.domain, d.roles.order].into_iter().flatten() {
            let r = TraitRef {
                tr,
                args: Vec::new(),
            };
            fn_bounds.push((i, r));
        }
    }
    let n_sets = cx.next_set.unwrap_or(0);
    // A library impl the slice cannot build is said where a program could reach it: through
    // its type when that is a struct, else through its trait.
    if let (Some(r), true) = (&tref, d.is_library(mid) && !errors.is_empty()) {
        match &for_ {
            Ty::Struct(sd, ..) => d.structs[sd.0].errors.append(&mut errors),
            _ => d.traits[r.tr.0].errors.append(&mut errors),
        }
        return Made::Done;
    }
    // No program can reach a library impl of a trait the slice does not declare.
    if tref.is_none() && d.is_library(mid) {
        return Made::Done;
    }
    let Some(tref) = tref.filter(|_| errors.is_empty()) else {
        if errors.is_empty() {
            errors.push(Msg::verdict(
                cpl_diag::codes::UNKNOWN_TYPE,
                "impl of an unknown trait",
            ));
        }
        d.module_or_program_errors(mid, errors);
        return Made::Done;
    };
    if let Some(Named::Struct(sid)) = id.filter(|_| !d.field_trait(tref.tr)) {
        let Some(asked) = Impls::of(d).derived_bounds(sid, &tref, copies) else {
            return Made::Wait;
        };
        for b in asked {
            if !ibounds.contains(&b) {
                fn_bounds.push(b.clone());
                ibounds.push(b);
            }
        }
    }
    let at = start;
    let tname = d.traits[tref.tr.0].name.clone();
    super::collect::impl_names(d, top, item, &tref, form);
    // An impl for every record with some fields is not a blanket one: a record takes it over a
    // blanket impl it also meets (R08.4.5).
    let rank = d.rank_of(&head, &ibounds);
    let clashes = |d: &Decls, im: &ImplDecl| {
        im.trait_.tr == tref.tr
            && overlap(&im.for_, &head)
            && d.impl_rank(im) == rank
            && (d.traits[tref.tr.0].inputs(&im.trait_.args).iter())
                .zip(d.traits[tref.tr.0].inputs(&tref.args))
                .all(|(x, y)| same_shape(x, y))
    };
    // Every written impl is made before any derived one, so it meets none.
    let written = id.is_none();
    if (d.impls.iter()).any(|im| d.live(im) && !(written && im.derived) && clashes(d, im)) {
        // A type's own impl replaces its derived one (R08.1.6).
        if id.is_none() {
            d.misdeclared.push((
                at,
                cpl_diag::codes::DUPLICATE_DEFINITION,
                format!("a second impl of `{tname}` for one type (R08.1.2)"),
            ));
        }
        return Made::Done;
    }
    if d.traits[tref.tr.0].alias {
        d.misdeclared.push((
            at,
            cpl_diag::codes::UNSUPPORTED,
            format!("`{tname}` is a trait alias: impl each trait it names (R08.2.6)"),
        ));
        return Made::Done;
    }
    // An impl lives with its trait, its type or a type among the trait's arguments
    // (`impl Into<E> for Int`), or with its fn for `type_of(f)` (R08.1.11), so it is never an
    // orphan (R08.5.2).
    let local = |t: &Ty| match t {
        Ty::Struct(sd, ..) => d.s(*sd).module == mid,
        Ty::Clo(c) => matches!(&**c, CloRef::Item(f) if d.f(f.decl).module == mid),
        _ => false,
    };
    if d.traits[tref.tr.0].module != mid && !local(&for_) && !tref.args.iter().any(local) {
        d.misdeclared.push((
            at,
            cpl_diag::codes::UNSUPPORTED,
            format!(
                "`{tname}` and the type are both declared elsewhere: an impl lives with one of them"
            ),
        ));
        return Made::Done;
    }
    // ... and replaces one std derived (R08.1.6).
    for k in (0..impls_from).filter(|_| written) {
        if d.impls[k].derived && d.live(&d.impls[k]) && clashes(d, &d.impls[k]) {
            d.impls[k].replaced = true;
        }
    }
    let derived = id.is_some();
    let id = d.impls.len();
    // An impl's `let count = e` is the getter `count` (R04.2.26), and a bodiless `@get` member
    // the impl leaves out is met by the field of that name (R12.11.3).
    let mut filled: Vec<FnForm> = form
        .lets
        .iter()
        .filter_map(crate::synth::value_fn)
        .collect();
    if let Ty::Struct(sid, ..) = &for_ {
        let mut ms: Vec<(Sym, FnId)> = (d.traits[tref.tr.0].methods.iter())
            .map(|(&n, &f)| (n, f))
            .collect();
        ms.sort_by_key(|&(n, _)| d.text(n).to_string());
        let has = |n: Sym| form.methods.iter().chain(&filled).any(|m| m.name == n);
        let getters: Vec<Sym> = (ms.into_iter())
            .filter(|&(n, tm)| {
                let tf = d.f(tm);
                tf.getter
                    && matches!(tf.body, Body::None)
                    && !has(n)
                    && d.field(*sid, d.text(n)).is_some()
            })
            .map(|(n, _)| n)
            .collect();
        for n in getters {
            filled.push(crate::synth::field_getter(n, form.span, &mut d.syms));
        }
    }
    let mut methods = HashMap::new();
    for m in form.methods.iter().chain(&filled) {
        let sig = d.traits[tref.tr.0].methods.get(&m.name).map(|&tm| {
            let td = d.f(tm);
            let mut tys = vec![for_.clone()];
            tys.extend(tref.args.iter().cloned());
            let sub = Subst {
                tys: &tys,
                sets: &[],
            };
            TraitSig {
                self_mode: td.self_mode,
                params: td.params.iter().map(|p| sub.ty(&p.ty)).collect(),
                ret: sub.ty(&td.ret),
            }
        });
        let within = Within {
            owner: Some(Owner::Impl(id)),
            generics: &generics,
            sets: n_sets,
            self_ty: Some(for_.clone()),
            sig,
        };
        let fid = add_fn(d, mid, item.clone(), m, within);
        // The impl's parameters come first in each method's, with its bounds.
        d.fns[fid.0].bounds.extend(fn_bounds.iter().cloned());
        methods.insert(m.name, fid);
    }
    let code = cpl_diag::codes::MISSING_IMPL;
    (d.misdeclared).extend(missing.into_iter().map(|m| (at.clone(), code, m)));
    let generics = generics.iter().map(|g| d.text(*g).to_string()).collect();
    d.push_impl(ImplDecl {
        trait_: tref,
        generics,
        bounds: ibounds,
        for_: head,
        methods,
        at,
        assoc_at,
        module: mid,
        derived,
        replaced: false,
    });
    Made::Done
}

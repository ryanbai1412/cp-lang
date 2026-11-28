//! Collecting declarations: std's once, and each program's as a layer over them.
use super::forms::{Form, ImplForm, Reader, TyForm, UseForm};
use super::impls::supertraits_unmet;
use super::inherent::inherent_impl;
use super::*;
use crate::attrs::Decorator;
use crate::synth::{enum_structs, module_derived};
use cpl_next_ir::ids::{ItemId, SourceId};
use cpl_next_ir::unbuilt::Unbuilt;
use cpl_next_syntax::ast::AstNode as _;

/// Std's declarations, collected once and never written after: the core's and every std
/// module's. A program's declarations are a layer over them.
pub struct Prepared {
    /// Each std module's forms, which a program's layer reads its derives and roles from too.
    pub read: Arc<Vec<Vec<(ItemId, Form)>>>,
    d: Decls,
    own: Layer<HashMap<String, Named>>,
    public: Layer<HashSet<String>>,
    /// The record shapes std writes, sorted.
    shapes: Vec<Vec<String>>,
    /// The row bounds std writes, sorted: each is a core trait (R08.4.1).
    rows: Vec<Vec<String>>,
    /// The `without` forms std writes, sorted: each is a core trait too.
    forms: Vec<crate::records::Form>,
    /// Each record shape std writes and the modules that write it.
    writers: HashMap<Vec<String>, Vec<ModId>>,
}

/// Std's declarations: `mods` are the core and std's modules.
pub fn prepare(mods: &[crate::modules::Module]) -> Prepared {
    let (mut std, _) = layer(None, mods, &[], &HashMap::new());
    let variadics = crate::records::variadics(mods);
    for mi in 0..mods.len() {
        for shape in crate::records::shapes(&mods[mi..=mi], &variadics) {
            std.writers.entry(shape).or_default().push(ModId(mi));
        }
    }
    let d = std::mem::take(&mut std.d);
    std.d = Decls {
        fns: d.fns.frozen(),
        structs: d.structs.frozen(),
        traits: d.traits.frozen(),
        impls: d.impls.frozen(),
        views: d.views.frozen(),
        consts: d.consts.frozen(),
        contexts: d.contexts.frozen(),
        aliases: d.aliases.frozen(),
        ..d
    };
    std.own = std::mem::take(&mut std.own).frozen();
    std.public = std::mem::take(&mut std.public).frozen();
    std
}

/// A program's declarations over std's: `mods` are std's modules in their order, then the
/// program's libraries and the program; `seen` says which of them the program loads.
pub fn collect(
    std: &Prepared,
    mods: &[crate::modules::Module],
    seen: &[bool],
    prelude: &HashMap<String, String>,
) -> (Decls, Vec<ItemId>) {
    let (layer, main) = layer(Some(std), mods, seen, prelude);
    (layer.d, main)
}

/// The declarations of `mods` past `base`'s, over `base`'s; with no base, std's own.
fn layer(
    base: Option<&Prepared>,
    mods: &[crate::modules::Module],
    seen: &[bool],
    prelude: &HashMap<String, String>,
) -> (Prepared, Vec<ItemId>) {
    let from = base.map_or(0, |b| b.d.module_names.len());
    let mut d = base.map_or_else(Decls::default, |b| b.d.clone());
    if base.is_none() {
        d.core_types = Arc::new(super::tyconv::core_types(&mut d.syms));
    }
    d.impls_ranked = None;
    let (fns_from, impls_from, aliases_from) = (d.fns.len(), d.impls.len(), d.aliases.len());
    let n = mods.len();
    d.program = ModId(match base {
        Some(_) => n - 1,
        None => usize::MAX,
    });
    d.module_names = mods.iter().map(|m| m.name.clone()).collect();
    d.within = (mods.iter())
        .map(|m| {
            let w = m.within.as_ref()?;
            mods.iter().position(|x| x.name == *w).map(ModId)
        })
        .collect();
    let within = d.within.clone();
    let in_program: Vec<bool> = (0..n).map(|i| d.of_program(ModId(i))).collect();
    d.module_errors.resize(n, Vec::new());
    d.seen = match base {
        Some(_) => seen.to_vec(),
        None => vec![true; n],
    };
    if d.roles
        .recipe
        .is_some_and(|r| !d.seen(d.structs[r.0].module))
    {
        d.roles.recipe = None;
    }
    let unseen = |d: &Decls, s: &StructId| !d.seen(d.structs[s.0].module);
    if d.member_types.values().any(|s| unseen(&d, s)) {
        let kept = (d.member_types.iter()).filter(|(_, s)| !unseen(&d, s));
        d.member_types = Arc::new(kept.map(|(m, s)| (*m, *s)).collect());
    }
    // Each module's items as their forms, read once (only this layer's modules are read).
    let mut unknown_marks = Vec::new();
    let read: Vec<Vec<(ItemId, Form)>> = (mods.iter().enumerate())
        .map(|(mi, m)| match mi >= from {
            true => {
                let (forms, unknown) = read_forms(&mut d.syms, m);
                unknown_marks.extend(unknown.into_iter().map(|u| (ModId(mi), u)));
                forms
            }
            false => Vec::new(),
        })
        .collect();
    if base.is_some() {
        let values = (read[n - 1].iter())
            .filter_map(|(item, f)| top_value_of(&mut d.syms, item, f))
            .collect();
        d.top_values = Arc::new(values);
    }
    let (mut own, mut public) = match base {
        Some(b) => (b.own.clone(), b.public.clone()),
        None => (Layer::default(), Layer::default()),
    };
    for _ in from..n {
        own.push(HashMap::new());
        public.push(HashSet::new());
    }
    let mut main = Vec::new();
    let memos: Vec<(usize, String)> = Vec::new();
    // Pass 1: names, so signatures can mention later declarations.
    let mut pending: Vec<(ModId, ItemId, &Form, Option<Named>)> = Vec::new();
    let mut op_methods: HashMap<TraitId, Sym> = HashMap::new();
    // The core's `use` lines, for its view made again over a program's records.
    let mut imports: Vec<(ModId, UseForm)> = match from {
        0 => Vec::new(),
        _ => (mods[CORE.0].items.iter())
            .filter_map(|it| match it.item.decl() {
                Some(cpl_next_syntax::ast::Decl::Use(u)) => {
                    Some((CORE, UseForm::read(it.item.is_pub(), &u)))
                }
                _ => None,
            })
            .collect(),
    };
    let mut aliases: Vec<(
        ModId,
        SourceId,
        String,
        Vec<Sym>,
        cpl_next_syntax::ast::Type,
    )> = Vec::new();
    let mut enums: Vec<(usize, String, Vec<String>, Vec<i64>)> = Vec::new();
    // Every record shape the program and its modules write is a core struct (R02.5.2).
    // A program's are those std does not write.
    let mut shapes = base.map_or_else(Vec::new, |b| b.shapes.clone());
    let mut written = crate::records::shapes(&mods[from..], &crate::records::variadics(mods));
    let mut forms = base.map_or_else(Vec::new, |b| b.forms.clone());
    let new_forms: Vec<crate::records::Form> = (crate::records::forms(&mods[from..]).into_iter())
        .filter(|f| forms.binary_search(f).is_err())
        .collect();
    let old_forms = forms.clone();
    forms.extend(new_forms.iter().cloned());
    forms.sort();
    // What a `without` makes of a record is a record too (R08.4.1).
    let known: Vec<Vec<String>> = shapes.iter().chain(&written).cloned().collect();
    written.extend(crate::records::derived(&forms, &known));
    let new_shapes: Vec<Vec<String>> = (written.iter())
        .filter(|s| shapes.binary_search(s).is_err())
        .cloned()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    // A record is the program's when a module it loads writes it.
    for (shape, by) in base.iter().flat_map(|b| &b.writers) {
        if !by.iter().any(|m| d.seen(*m)) && !written.contains(shape) {
            if let Some(Named::Struct(s)) = own[CORE.0].get(&crate::records::name(shape)) {
                d.unseen_records.insert(*s);
            }
        }
    }
    shapes.extend(new_shapes.iter().cloned());
    shapes.sort();
    let mut rows = base.map_or_else(Vec::new, |b| b.rows.clone());
    let new_rows: Vec<Vec<String>> = (crate::records::rows(&mods[from..]).into_iter())
        .filter(|r| rows.binary_search(r).is_err())
        .collect();
    let old_rows = rows.clone();
    rows.extend(new_rows.iter().cloned());
    rows.sort();
    // The core's generated declarations: each new record shape's struct, row trait and
    // `without` trait (R02.5.2, R08.4.1), as forms of the core's first item.
    let at = Span::new(0, 0);
    let records: Vec<(ItemId, Form)> = match mods[CORE.0].items.first() {
        Some(it) => {
            let syms = &mut d.syms;
            let mut fs: Vec<Form> = (crate::records::structs(&new_shapes, syms, at).into_iter())
                .map(Form::Struct)
                .collect();
            fs.extend(
                crate::records::row_traits(&new_rows, syms, at)
                    .into_iter()
                    .map(Form::Trait),
            );
            fs.extend(
                (crate::records::without_traits(&new_forms, syms, at).into_iter()).map(Form::Trait),
            );
            fs.into_iter().map(|f| (it.id.clone(), f)).collect()
        }
        None => Vec::new(),
    };
    let std_read: &[Vec<(ItemId, Form)>] = base.map_or(&[], |b| &b.read[..]);
    let forms_of: Vec<&[(ItemId, Form)]> = (0..mods.len())
        .map(|mi| match mi < from {
            true => std_read.get(mi).map_or(&[][..], |v| &v[..]),
            false => &read[mi][..],
        })
        .collect();
    let all: Vec<&Form> = (forms_of.iter())
        .flat_map(|fs| fs.iter().map(|(_, f)| f))
        .chain(records.iter().map(|(_, f)| f))
        .collect();
    let each = crate::synth::derive_each(&all);
    let cmp = comparisons(&all);
    let convert = crate::synth::convert_marked(&all);
    let positions = crate::records::positions(&all);
    // Only a derive that rebuilds its value copies the `@cache` fields.
    let impl_of = |v: Vec<(ItemId, String, ImplForm)>, caches: bool| {
        v.into_iter()
            .map(move |(i, n, s)| (i, n, Form::Impl(s), caches))
    };
    let mut derived: Vec<Vec<(ItemId, String, Form, bool)>> = Vec::new();
    for (mi, fs) in forms_of.iter().enumerate() {
        // Std's modules get only what a program's records add.
        let ours = mi >= from;
        let made_here = match (mi == CORE.0, ours) {
            (true, _) => module_derived(&records, &each, &cmp, &mut d.syms),
            (false, true) => module_derived(fs, &each, &cmp, &mut d.syms),
            (false, false) => Vec::new(),
        };
        let mut made: Vec<(ItemId, String, Form, bool)> = (made_here.into_iter())
            .map(|(i, n, s, c)| (i, n, Form::Impl(s), c))
            .collect();
        if let (true, Some((id, _))) = (mi == CORE.0, records.first()) {
            let syms = &mut d.syms;
            let impls = crate::records::domain_impls(positions.as_ref(), &new_shapes, syms, at);
            made.extend(
                impls
                    .into_iter()
                    .map(|(n, imp)| (id.clone(), n, Form::Impl(imp), false)),
            );
            // Each new row over every shape, and std's rows over the new shapes.
            let rows_of =
                |rs: &[Vec<String>], syms: &mut crate::sym::Syms| -> Vec<(ItemId, Form)> {
                    (crate::records::row_traits(rs, syms, at).into_iter())
                        .map(|t| (id.clone(), Form::Trait(t)))
                        .collect()
                };
            let rows = rows_of(&new_rows, syms);
            made.extend(impl_of(
                crate::synth::field_trait_impls(&rows, &shapes, syms),
                false,
            ));
            if !new_shapes.is_empty() {
                let old = rows_of(&old_rows, syms);
                made.extend(impl_of(
                    crate::synth::field_trait_impls(&old, &new_shapes, syms),
                    false,
                ));
            }
            let mut without = crate::records::without_impls(&new_forms, &shapes, syms, at);
            without.extend(crate::records::without_impls(
                &old_forms,
                &new_shapes,
                syms,
                at,
            ));
            made.extend(
                without
                    .into_iter()
                    .map(|(n, imp)| (id.clone(), n, Form::Impl(imp), false)),
            );
        }
        if ours || !new_shapes.is_empty() {
            let shapes = if ours { &shapes } else { &new_shapes };
            made.extend(impl_of(
                crate::synth::field_trait_impls(fs, shapes, &mut d.syms),
                false,
            ));
        }
        if let Some(c) = convert.filter(|_| ours) {
            let made_c = crate::synth::enum_conversions(fs, c, positions.as_ref(), &mut d.syms);
            made.extend(impl_of(made_c, false));
        }
        derived.push(made);
    }
    for (w, home) in crate::synth::memo_tables(&forms_of, &mut d.syms) {
        d.memo_homes.insert(w, home);
    }
    // Each enum as structs: its own and one per variant (R02.7).
    let mut synth: Vec<Vec<(ItemId, Form)>> = vec![Vec::new(); mods.len()];
    synth[CORE.0].extend(records.iter().cloned());
    for mi in from..mods.len() {
        for (k, (item, f)) in read[mi].iter().enumerate() {
            let mut placed = Vec::new();
            super::attr_rules::check_form(f, item, &mut placed);
            match ModId(mi) == d.program {
                true => d.errors.extend(placed),
                false => d.module_errors[mi].extend(placed),
            }
            let Form::Enum(e) = f else {
                continue;
            };
            let errors = crate::synth::enum_tags(item, &mods[mi].items[k].item, e, &d.syms);
            match ModId(mi) == d.program {
                true => d.errors.extend(errors),
                false => d.module_errors[mi].extend(errors),
            }
            let vs = e
                .variants
                .iter()
                .map(|v| d.text(v.name).to_string())
                .collect();
            let values = (0..e.variants.len() as i64).collect();
            enums.push((mi, d.text(e.name).to_string(), vs, values));
            let structs = enum_structs(e, &mut d.syms);
            synth[mi].extend(structs.into_iter().map(|s| (item.clone(), Form::Struct(s))));
        }
    }
    for mi in 0..mods.len() {
        let mid = ModId(mi);
        if mi < from && synth[mi].is_empty() {
            continue;
        }
        if mi >= from {
            for h in crate::items::items(&mods[mi]) {
                public[mi].extend(h.exports().map(str::to_string));
            }
        }
        for (_, f) in &synth[mi] {
            if let (true, Some(n)) = (f.public(), f.name()) {
                public[mi].insert(d.text(n).to_string());
            }
        }
        let written = (read[mi].iter()).filter(|(_, f)| !matches!(f, Form::Enum(_)));
        for (item, f) in written.chain(synth[mi].iter()) {
            match f {
                Form::Struct(sd) => {
                    let id = super::names::name_struct(&mut d, &mut own[mi], mid, item, sd);
                    pending.push((mid, item.clone(), f, Some(Named::Struct(id))));
                }
                Form::Trait(t) => {
                    let id = super::names::name_trait(
                        &mut d,
                        &mut own[mi],
                        &mut op_methods,
                        mid,
                        item,
                        t,
                    );
                    pending.push((mid, item.clone(), f, Some(Named::Trait(id))));
                }
                Form::Alias(a) => {
                    let library = d.is_library(mid);
                    for x in a.attrs.iter().filter(|x| {
                        !(library
                            && matches!(
                                x.attr,
                                Decorator::Lint | Decorator::Stress | Decorator::OnUnimplemented
                            ))
                    }) {
                        let e = Msg::at(
                            item.at(super::names::attr_at(x, a.span)),
                            Unbuilt::Decorator {
                                name: x.attr.name().to_string(),
                            },
                        );
                        match library {
                            true => d.module_errors[mi].push(e),
                            false => d.errors.push(e),
                        }
                    }
                    let name = d.text(a.name).to_string();
                    own[mi].insert(
                        name.clone(),
                        Named::Alias(AliasId(aliases_from + aliases.len())),
                    );
                    let generics = a.generics.clone();
                    let at = item.at(Span::new(a.span.start, a.span.start));
                    let TyForm::Written(ty) = &a.ty else {
                        unreachable!("an alias's type is written")
                    };
                    aliases.push((mid, at, name, generics, ty.clone()));
                }
                Form::Fn(_) | Form::Impl(_) => pending.push((mid, item.clone(), f, None)),
                Form::Const {
                    name,
                    ty,
                    value: Some(init),
                    ..
                } if d.items_only(mid) => {
                    let name = d.text(*name).to_string();
                    own[mi].insert(name.clone(), Named::Const(ConstId(d.consts.len())));
                    let ty = ty.as_ref().map(|t| match t {
                        TyForm::Written(t) => t.clone(),
                        _ => unreachable!("a const's type is written"),
                    });
                    d.consts.push(LibConst {
                        module: mid,
                        name,
                        item: item.clone(),
                        ty,
                        init: init.clone(),
                    });
                }
                Form::Use(u) => imports.push((mid, u.clone())),
                Form::Context { name, fields } => {
                    let id = ContextId(d.contexts.len());
                    let name = d.text(*name).to_string();
                    own[mi].insert(name.clone(), Named::Context(id));
                    d.contexts.push(ContextDecl {
                        module: mid,
                        name,
                        fields: fields.clone(),
                    });
                }
                other if d.items_only(mid) => {
                    let e = Unbuilt::LibraryItem {
                        kind: other.kind().to_string(),
                    };
                    match other.name() {
                        Some(n) => {
                            own[mi].insert(d.text(n).to_string(), Named::Refused(d.refused.len()));
                            d.refused.push(e);
                        }
                        None => d.module_errors[mi].push(e.into()),
                    }
                }
                _ => main.push(item.clone()),
            }
        }
    }
    for (mi, name, vs, values) in enums {
        let Some(Named::Struct(id)) = own[mi].get(&name).copied() else {
            continue;
        };
        d.structs[id.0].values = values;
        let vs = (vs.into_iter())
            .filter_map(|v| match own[mi].get(&format!("{name}.{v}")) {
                Some(Named::Struct(p)) => Some((d.syms.intern(&v), *p)),
                _ => None,
            })
            .collect();
        d.structs[id.0].variants = vs;
    }
    // The derived impls that read their struct's `@cache` fields, by the impl they are.
    let mut copying: HashSet<*const Form> = HashSet::new();
    for (mi, made) in derived.iter().enumerate() {
        for (item, name, s, caches) in made {
            if *caches {
                copying.insert(s);
            }
            // A record is core's struct wherever its impl is written.
            let record = |n: &Named| matches!(n, Named::Struct(s) if d.structs[s.0].record);
            let found = own[mi]
                .get(name)
                .or_else(|| own[CORE.0].get(name).filter(|n| record(n)));
            let n = match s {
                Form::Impl(imp) => match &imp.self_ty {
                    TyForm::Named(_, args) => args.len(),
                    _ => 0,
                },
                _ => 0,
            };
            if let Some(Named::Struct(id)) = found.copied() {
                let id = super::tyconv::by_arity(&d.struct_arity, id, n);
                pending.push((ModId(mi), item.clone(), s, Some(Named::Struct(id))));
            }
        }
    }
    let program = d.program.0;
    // A module the program does not load is no module to it.
    let names: Vec<String> = (d.module_names.iter().enumerate())
        .map(|(i, m)| match d.seen(ModId(i)) {
            true => m.clone(),
            false => String::new(),
        })
        .collect();
    let core_too = !records.is_empty() && from > 0;
    let sc = Scopes {
        own: &own,
        public: &public,
        names: &names,
        imports: &imports,
        prelude,
        within: &within,
    };
    remake_views(&mut d, &sc, from, core_too, None);
    // A product's record is a core struct, so the program's domains are placed once the views are.
    if base.is_some() {
        for (id, f) in &read[n - 1] {
            let Form::Domain(dd) = f else {
                continue;
            };
            let Some(name) = dd.name() else {
                continue;
            };
            let name = name.text().to_string();
            let t = dd.size().and_then(|def| product(&mut d, id, &name, &def));
            let k = d.syms.intern(&name);
            if !d.top_doms.contains_key(&k) {
                let t = t.unwrap_or_else(|| d.new_dom(id));
                d.top_doms.insert(k, t);
            }
        }
    }
    let made: Vec<AliasDecl> = aliases
        .into_iter()
        .map(|(m, at, name, generics, ty)| {
            let sc = Scopes {
                own: &own,
                public: &public,
                names: &names,
                imports: &imports,
                prelude,
                within: &within,
            };
            let mut v = outer_view(&sc, m.0, in_program[m.0], None, None);
            let outside = v.get(&name).copied();
            v.extend(own[m.0].iter().map(|(n, x)| (n.clone(), *x)));
            if let Some(x) = outside {
                v.insert(name.clone(), x);
            }
            let v = v.into_iter().map(|(n, x)| (d.syms.intern(&n), x)).collect();
            AliasDecl {
                name,
                module: m,
                at,
                generics,
                ty,
                names: v,
                program: m.0 == program,
            }
        })
        .collect();
    made.into_iter().for_each(|a| d.aliases.push(a));
    for (i, a) in d.aliases.iter().enumerate().skip(aliases_from) {
        if d.is_library(a.module) && alias_cycle(&d, AliasId(i)) {
            d.module_errors[a.module.0].push(Msg::internal(format!(
                "type alias `{}` is defined by itself",
                a.name
            )));
        }
    }
    // Struct fields first, to a fixed point: a struct holding a struct with function fields has those sets too.
    loop {
        let mut changed = false;
        for (mid, _, s, id) in &pending {
            let (Form::Struct(sd), Some(Named::Struct(id))) = (s, id) else {
                continue;
            };
            let n = super::structs::struct_sets(&mut d, *mid, sd);
            if Arc::make_mut(&mut d.struct_sets).insert(*id, n) != Some(n) {
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    // Then their default type arguments, which name its earlier parameters and any struct.
    for (mid, _, s, id) in &pending {
        if let (Form::Struct(sd), Some(Named::Struct(id))) = (s, id) {
            super::structs::struct_defaults(&mut d, *mid, *id, sd);
        }
    }
    // Traits first, for the defaults every use of one reads; impls last, for their trait's fns.
    let pass = |s: &Form, id: &Option<Named>| match s {
        Form::Trait(_) => 0,
        Form::Impl(_) if id.is_some() => 3,
        Form::Impl(_) => 2,
        _ => 1,
    };
    pending.sort_by_key(|(_, _, s, id)| pass(s, id));
    // A derived impl waits for its fields' types' impls, and is never made if one lacks it.
    let mut queue = pending;
    let (mut round, mut stamped, mut made) = (0, d.fns.len(), (0, 0));
    let mut fns_seen = false;
    loop {
        let mut wait = Vec::new();
        let before = queue.len();
        for (mid, item, s, id) in std::mem::take(&mut queue) {
            let mi = mid.0;
            for k in stamped..d.fns.len() {
                d.fns[k].made = made;
            }
            (stamped, made) = (d.fns.len(), (round, pass(s, &id)));
            // An impl head's `type_of(f)` may name a fn another module declares and this one
            // imports, so the views take every fn before the first impl.
            if !fns_seen && pass(s, &id) >= 2 {
                fns_seen = true;
                let sc = Scopes {
                    own: &own,
                    public: &public,
                    names: &names,
                    imports: &imports,
                    prelude,
                    within: &within,
                };
                remake_views(&mut d, &sc, from, core_too, None);
            }
            match (s, id) {
                (Form::Struct(sd), Some(Named::Struct(id))) => {
                    super::structs::struct_body(&mut d, mid, &item, id, sd)
                }
                (Form::Trait(t), Some(Named::Trait(id))) => {
                    super::structs::trait_body(&mut d, mid, &item, id, t)
                }
                (Form::Impl(form), id) => match &form.trait_ {
                    Some(tr) => {
                        let of = super::trait_impl::Of {
                            mid,
                            item: &item,
                            form,
                            tr,
                            derived_for: id,
                            copies: copying.contains(&(s as *const _)),
                            impls_from,
                            top: forms_of[mi],
                        };
                        if let super::trait_impl::Made::Wait =
                            super::trait_impl::trait_impl(&mut d, of)
                        {
                            wait.push((mid, item, s, id));
                        }
                    }
                    None => {
                        // Inherent methods live in their type's module only (R08.1.3).
                        let (name, at) = match &form.self_ty {
                            TyForm::Written(cpl_next_syntax::ast::Type::Path(p)) => {
                                let segs: Vec<String> =
                                    p.segments().map(|t| t.text().to_string()).collect();
                                (segs.join("."), p.syntax().span())
                            }
                            TyForm::Named(n, _) => (d.text(*n).to_string(), form.span),
                            _ => continue,
                        };
                        let Some(t) = d.struct_named(ModId(mi), &name) else {
                            continue;
                        };
                        if d.s(t).module != mid {
                            d.misdeclared.push((
                                item.at(Span::new(at.start, at.start)),
                                cpl_diag::codes::UNSUPPORTED,
                                format!("`{name}` is declared in another module; its inherent methods live there"),
                            ));
                            continue;
                        }
                        inherent_impl(&mut d, mid, item, t, form);
                    }
                },
                (Form::Fn(f), _) => {
                    let fid = add_fn(&mut d, mid, item.clone(), f, FREE);
                    let name = d.text(f.name).to_string();
                    let before = own[mi].insert(name.clone(), Named::Fn(fid));
                    Arc::make_mut(&mut d.views[mi]).insert(f.name, Named::Fn(fid));
                    if public[mi].contains(&name) && d.f(fid).self_ty.is_some() {
                        d.methods.entry(f.name).or_default().push(fid);
                    }
                    if let Some(Named::Fn(first)) = before {
                        // Free fns of one name whose required labels differ are overloads: the
                        // first stands for the name (R06.1.6).
                        let free = d.f(first).self_ty.is_none() && d.f(fid).self_ty.is_none();
                        let rest = d.overloads.get(&first).map_or(&[][..], Vec::as_slice);
                        let clash = (std::iter::once(&first).chain(rest))
                            .any(|o| d.required_labels(*o) == d.required_labels(fid));
                        if free && !clash {
                            own[mi].insert(name, Named::Fn(first));
                            Arc::make_mut(&mut d.views[mi]).insert(f.name, Named::Fn(first));
                            d.overloads.entry(first).or_default().push(fid);
                            continue;
                        }
                        // Two with a `self` are one method's overloads unless one call could take
                        // both (R08.1.4); that test isn't built.
                        if d.f(first).self_ty.is_some() && d.f(fid).self_ty.is_some() {
                            d.errors.push(Unbuilt::MethodsOfOneName { name }.into());
                            continue;
                        }
                        // One name and one set of required labels is one fn: nothing is chosen
                        // by argument type (R02.11.3, R06.1.6).
                        d.misdeclared.push((
                            item.at(Span::new(f.span.start, f.span.start)),
                            cpl_diag::codes::OVERLOAD_BY_TYPE,
                            format!("`{name}` is already a fn; one name is one fn"),
                        ));
                    }
                }
                _ => {}
            }
        }
        if wait.is_empty() || wait.len() == before {
            break;
        }
        round += 1;
        queue = wait;
    }
    for k in stamped..d.fns.len() {
        d.fns[k].made = made;
    }
    let mut errors = Vec::new();
    let sc = Scopes {
        own: &own,
        public: &public,
        names: &names,
        imports: &imports,
        prelude,
        within: &within,
    };
    remake_views(&mut d, &sc, from, core_too, Some(&mut errors));
    d.errors.extend(errors);
    // Each written fn's body, matched to its fn by the item's fns in the order they were made.
    let mut bodies = HashMap::new();
    for it in mods[from..].iter().flat_map(|m| m.items.iter()) {
        for &id in d.fns_of(&it.id) {
            if let (Body::Written, Some(b)) = (&d.f(id).body, body_of(&d, id, &it.item)) {
                bodies.insert(id, b);
            }
        }
    }
    domain_params(&mut d, fns_from, &bodies);
    fresh_results(&mut d);
    let unmet: Vec<_> = (d.impls.iter().skip(impls_from))
        .flat_map(|im| supertraits_unmet(&d, im))
        .collect();
    d.misdeclared.extend(unmet);
    // A program's lookups go through only the impls it sees.
    let live: Vec<bool> = d.impls.iter().map(|im| d.live(im)).collect();
    if base.is_some() {
        for ks in (d.impls_by_trait.values_mut()).chain(d.impls_by_member.values_mut()) {
            ks.retain(|k| live[*k]);
        }
    }
    // `f.clear()` of a `@memo` fn reaches its clear through the fn, not by a spelling (R06.4.1).
    for (mi, f) in memos {
        let clear = crate::synth::memo_clear_name(&f);
        if let (Some(Named::Fn(w)), Some(Named::Fn(c))) = (own[mi].get(&f), own[mi].get(&clear)) {
            d.memo_clears.insert(*w, *c);
        }
    }
    d.rank_impls();
    let layer = Prepared {
        read: Arc::new(read),
        d,
        own,
        public,
        shapes,
        rows,
        forms,
        writers: HashMap::new(),
    };
    (layer, main)
}

/// Each module's view, made again where this layer changes it: every module's for std, and for
/// a program its own modules' and, when its records add to the core, the core's. `errors` takes
/// the names the core's and the program's views miss, the first time each is made.
fn remake_views(
    d: &mut Decls,
    sc: &Scopes,
    from: usize,
    core_too: bool,
    mut errors: Option<&mut Vec<Msg>>,
) {
    for mi in 0..sc.names.len() {
        if mi < from && !(mi == CORE.0 && core_too) {
            continue;
        }
        let e = match d.is_library(ModId(mi)) || mi < from {
            true => None,
            false => errors.as_deref_mut(),
        };
        let mut ambiguous = HashSet::new();
        let v = view(sc, mi, d.of_program(ModId(mi)), e, &mut ambiguous);
        let v = v.into_iter().map(|(n, x)| (d.syms.intern(&n), x)).collect();
        let ambiguous = ambiguous.iter().map(|n| d.syms.intern(n)).collect();
        let exports = export_map(sc, mi).into_iter();
        let exports = Arc::new(exports.map(|(n, x)| (d.syms.intern(&n), x)).collect());
        d.exports
            .resize(d.exports.len().max(mi + 1), Arc::default());
        d.exports[mi] = exports;
        match mi < d.views.len() {
            true => {
                d.views[mi] = Arc::new(v);
                d.ambiguous[mi] = ambiguous;
            }
            false => {
                d.views.push(Arc::new(v));
                d.ambiguous.push(ambiguous);
            }
        }
        for (w, (home, name)) in &d.memo_homes {
            let name = d.syms.text(*name);
            let t = (*home < sc.own.len()).then(|| sc.own[*home].get(name).copied());
            if let Some(t) = t.flatten() {
                let key = d.syms.intern(&crate::synth::memo_key(d.syms.text(*w)));
                Arc::make_mut(&mut d.views[mi]).insert(key, t);
            }
        }
    }
}

/// `domain P = (A, B)` or `(a: A, b: B)` over domains declared above it, or over ranges: the
/// tuple or record of them (R04.2.10, R04.2.11). A range factor is a domain of its own, named `P.i`, which no program can write.
fn product(
    d: &mut Decls,
    item: &ItemId,
    name: &str,
    def: &cpl_next_syntax::ast::Expr,
) -> Option<Ty> {
    use cpl_next_syntax::ast::{AstNode, Expr};
    let (product, xs): (StructId, Vec<Expr>) = match def {
        Expr::Tuple(t) => {
            let xs: Vec<Expr> = t.syntax().children().filter_map(Expr::cast).collect();
            (*d.tuples.get(&xs.len())?, xs)
        }
        Expr::Record(r) => {
            let fs: Vec<_> = r.fields().collect();
            let names = (fs.iter())
                .map(|f| Some(f.name()?.text().to_string()))
                .collect::<Option<Vec<String>>>()?;
            let record = *d.records.get(&names)?;
            (record, fs.iter().map(|f| f.value()).collect::<Option<_>>()?)
        }
        _ => return None,
    };
    let dom = |x: &Expr| match x {
        Expr::Name(n) => Some(n.ident()?.text().to_string()),
        _ => None,
    };
    let fits = |x: &Expr| match x {
        Expr::Range(_) => true,
        _ => (dom(x).and_then(|n| d.syms.get(&n))).is_some_and(|n| d.top_doms.contains_key(&n)),
    };
    if !xs.iter().all(fits) {
        return None;
    }
    let mut parts = Vec::new();
    for (i, x) in xs.iter().enumerate() {
        parts.push(match dom(x).and_then(|n| d.syms.get(&n)) {
            Some(n) => d.top_doms[&n].clone(),
            None => {
                let t = d.new_dom(item);
                let k = d.syms.intern(&factor_name(name, i));
                d.top_doms.insert(k, t.clone());
                t
            }
        });
    }
    Some(Ty::Struct(product, parts, Vec::new()))
}

/// The bodies form `f` writes, in the order its fns are made.
/// The fns an item declares, in the order its form registers them.
fn member_fns(item: &cpl_next_syntax::ast::Item) -> Vec<cpl_next_syntax::ast::FnDecl> {
    use cpl_next_syntax::ast::Decl as D;
    let members = match item.decl() {
        Some(D::Fn(f)) => return vec![f],
        Some(D::Struct(s)) => s.members(),
        Some(D::Enum(e)) => e.members(),
        Some(D::Trait(t)) => t.members(),
        Some(D::Impl(i)) => i.members(),
        _ => None,
    };
    (members.iter())
        .flat_map(|m| m.fns().collect::<Vec<_>>())
        .collect()
}

/// The body `f` checks, read from `item`, the item that declares it: the written body in the
/// place `f` holds among the item's written fns, or the value of the impl's `let` it is the
/// getter of (R04.2.26). A generated or bodiless fn has none.
pub fn body_of(
    d: &Decls,
    f: FnId,
    item: &cpl_next_syntax::ast::Item,
) -> Option<cpl_next_syntax::ast::FnBody> {
    use cpl_next_syntax::ast as next;
    let decl = d.f(f);
    match decl.body {
        Body::Written => {
            let rank = (d.fns_of(&decl.item).iter())
                .take_while(|g| **g != f)
                .filter(|g| matches!(d.f(**g).body, Body::Written))
                .count();
            (member_fns(item).into_iter())
                .filter_map(|g| g.body())
                .nth(rank)
        }
        Body::Value => {
            let name = decl.name.as_str();
            let members = match item.decl() {
                Some(next::Decl::Impl(i)) => i.members(),
                _ => None,
            };
            let one_name = |l: &next::LetStmt| match &l.patterns().collect::<Vec<_>>()[..] {
                [next::Pattern::Path(p)] => match &p.segments().collect::<Vec<_>>()[..] {
                    [n] => n.text() == name,
                    _ => false,
                },
                _ => false,
            };
            (members.iter())
                .flat_map(|m| m.lets().collect::<Vec<_>>())
                .find(|l| one_name(l))
                .and_then(|l| l.value())
                .map(next::FnBody::Expr)
        }
        Body::Synth(_) | Body::None => None,
    }
}

/// The name of product `P`'s `i`th factor when it is written as a range.
pub fn factor_name(product: &str, i: usize) -> String {
    format!("{product}.{i}")
}

/// A marker names one declaration: a second, in any module, is a duplicate.
pub(super) fn again(item: &ItemId, at: Span, n: &str) -> Msg {
    let code = cpl_diag::codes::DUPLICATE_DEFINITION;
    Msg::at(
        item.at(at),
        Msg::verdict(code, format!("a second declaration marked `@{n}`")),
    )
}
pub(super) fn misplaced_role(item: &ItemId, at: SourceId, role: Option<Role>, kind: &str) -> Msg {
    let msg = match role {
        Some(r) => Msg::verdict(
            cpl_diag::codes::TYPE_MISMATCH,
            format!("`@role({})` is not filled by a {kind}", r.name()),
        ),
        None => Msg::verdict(
            cpl_diag::codes::UNRESOLVED_NAME,
            "`@role` names one of the roles of R11.6.1".to_string(),
        ),
    };
    let _ = item;
    Msg::at(at, msg)
}
pub(super) fn once<T>(slot: &mut Option<T>, id: T, dup: Msg, errors: &mut Vec<Msg>) {
    match slot {
        Some(_) => errors.push(dup),
        None => *slot = Some(id),
    }
}

/// The traits `==` and `<` call, with the method each calls and whether it is `==`: the
/// comparisons a struct derives field by field (R05.2.2).
fn comparisons(all: &[&Form]) -> Vec<(Sym, Sym, bool)> {
    (all.iter())
        .filter_map(|f| match f {
            Form::Trait(t) => Some(t),
            _ => None,
        })
        .filter_map(|t| {
            let op = (t.attrs.iter())
                .find(|a| a.attr == crate::attrs::Decorator::Operator)?
                .str_arg()?;
            let eq = match op.as_str() {
                "==" => true,
                "<" => false,
                _ => return None,
            };
            Some((t.name, operator_method(t)?, eq))
        })
        .collect()
}

/// The fn an `@operator` trait's operator calls: its first.
pub(super) fn operator_method(t: &super::forms::TraitForm) -> Option<Sym> {
    t.methods.first().map(|m| m.name)
}

/// What `@on_unimplemented("..", code: C)` says when a type lacks the trait's impl (R08.1.8).
pub(super) fn unimplemented(
    item: &ItemId,
    attrs: &[super::forms::AttrForm],
    errors: &mut Vec<Msg>,
) -> Unimplemented {
    use cpl_next_syntax::literal::{string_parts, StrPart};
    use cpl_next_syntax::{ast as next, TokenKind};
    let mut out = Unimplemented::default();
    let Some(node) = (attrs.iter())
        .find(|a| a.attr == crate::attrs::Decorator::OnUnimplemented)
        .and_then(|a| a.node.as_ref())
    else {
        return out;
    };
    let args: Vec<next::Arg> = node.args().map(|l| l.args().collect()).unwrap_or_default();
    for a in args {
        let label = a.label().map(|l| l.text().to_string());
        let value = a.value();
        // A message with a `{T}` hole is parsed into text pieces around it, so it is read whole.
        let message = match (&label, &value) {
            (None, Some(next::Expr::Literal(l))) => (l.token())
                .filter(|t| matches!(t.kind(), TokenKind::Str | TokenKind::StrText))
                .map(|_| l.syntax().text()),
            _ => None,
        };
        let code = match (label.as_deref(), &value) {
            (Some("code"), Some(next::Expr::Name(n))) => n.ident(),
            _ => None,
        };
        if let Some(text) = message {
            let text = text.as_str();
            let piece = |r: cpl_diag::Span| &text[r.start as usize..r.end as usize];
            let help = string_parts(text).into_iter().map(|p| match p {
                StrPart::Text(r) => piece(r).to_string(),
                StrPart::Expr(r) => {
                    let n = piece(r).split(':').next().unwrap_or("").trim();
                    let ident = n
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_alphabetic() || c == '_')
                        && n.chars().all(|c| c.is_alphanumeric() || c == '_');
                    match ident {
                        true => format!("{{{n}}}"),
                        false => String::new(),
                    }
                }
            });
            out.help = Some(help.collect());
        } else if let Some(info) = code
            .as_ref()
            .and_then(|c| cpl_diag::registry::lookup(c.text()))
        {
            out.code = Some(info.code);
        } else {
            let span = value.map_or(a.syntax().span(), |v| v.syntax().span());
            errors.push(Msg::at(
                item.at(span),
                Msg::verdict(
                    cpl_diag::codes::UNRESOLVED_NAME,
                    "`@on_unimplemented` takes a message and `code:` a diagnostic code",
                ),
            ));
        }
    }
    out
}

/// `_..=_` or `_.._` as an impl's type: every range type.
pub(super) fn any_range(t: &super::forms::TyForm) -> bool {
    use cpl_next_syntax::ast as next;
    let super::forms::TyForm::Written(next::Type::Range(r)) = t else {
        return false;
    };
    let Some(next::Expr::Range(re)) = r.expr() else {
        return false;
    };
    let hole = |e: Option<next::Expr>| matches!(e, Some(next::Expr::Hole(_)));
    hole(re.lo()) && hole(re.hi())
}

/// An impl's members are its trait's names: one under the name a member `@was` is E0605, which
/// says the name now (decision 504). An impl's `let` is frozen where the impl is declared, so it
/// reads no `var` (R04.2.26).
pub(super) fn impl_names(
    d: &mut Decls,
    top: &[(ItemId, Form)],
    item: &ItemId,
    tref: &TraitRef,
    form: &super::forms::ImplForm,
) {
    use cpl_next_syntax::ast as next;
    let vars: HashSet<String> = (top.iter())
        .filter_map(|(_, f)| match f {
            Form::Stmt(next::Stmt::Let(l)) => Some(l.patterns().collect::<Vec<_>>()),
            _ => None,
        })
        .flatten()
        .filter_map(|p| match p {
            next::Pattern::Path(p) if p.is_var() => Some(p.segments().last()?.text().to_string()),
            _ => None,
        })
        .collect();
    let code = cpl_diag::codes::BAD_INDEX_IMPL;
    let tname = d.traits[tref.tr.0].name.clone();
    let members = (form.methods.iter().map(|f| (f.name, f.span, None)))
        .chain(form.lets.iter().map(|l| (l.name, l.span, l.value.as_ref())));
    let members: Vec<_> = members.collect();
    for (n, span, init) in members {
        let n = d.text(n).to_string();
        let now = (d.traits[tref.tr.0].methods.iter())
            .find(|(_, &tm)| d.f(tm).was.contains(&n))
            .map(|(now, _)| d.text(*now).to_string());
        if let Some(now) = now {
            let msg = format!("`impl {tname}` names this member `{now}`; `{n}` was its old name");
            d.misdeclared.push((item.at(span), code, msg));
        }
        let reads = (init.into_iter())
            .flat_map(|e| e.syntax().descendants().filter_map(next::NameRef::cast));
        for r in reads {
            let Some(v) = r.ident().filter(|v| vars.contains(v.text())) else {
                continue;
            };
            let msg = format!("an impl's `let` is frozen where the impl is declared, and `{}` is a `var`: freeze it first with `let`", v.text());
            d.misdeclared.push((item.at(r.syntax().span()), code, msg));
        }
    }
}

/// A module's items as forms, and the decorators on them no attribute names, by item.
type ReadForms = (
    Vec<(ItemId, Form)>,
    Vec<(ItemId, cpl_next_syntax::ast::Decorator)>,
);

/// Each item of `m` as its form, read once; and the decorators on them no attribute names, each
/// with the item it is on, for E0107.
fn read_forms(syms: &mut crate::sym::Syms, m: &crate::modules::Module) -> ReadForms {
    let mut reader = Reader { syms };
    let (mut forms, mut unknown) = (Vec::new(), Vec::new());
    for it in m.items.iter() {
        let (form, other) = reader.item(&it.item);
        unknown.extend(other.into_iter().map(|x| (it.id.clone(), x)));
        forms.push((it.id.clone(), form));
    }
    (forms, unknown)
}

/// What a top-level `let` binds, when it binds one name: what `top_values` holds.
fn top_value_of(syms: &mut crate::sym::Syms, item: &ItemId, form: &Form) -> Option<TopValue> {
    use cpl_next_syntax::ast::{Pattern, Stmt};
    let Form::Stmt(Stmt::Let(l)) = form else {
        return None;
    };
    let pats: Vec<Pattern> = l.patterns().collect();
    match pats.as_slice() {
        [Pattern::Path(p)] if p.args().is_none() => {
            let segs: Vec<_> = p.segments().collect();
            match segs.as_slice() {
                [n] => Some(TopValue {
                    name: syms.intern(n.text()),
                    item: item.clone(),
                    mutable: l.is_var() || p.is_var(),
                    within: declared_within(l.ty()),
                }),
                _ => None,
            }
        }
        _ => None,
    }
}

/// The range a top-level value's written type declares, by its literal ends.
fn declared_within(t: Option<cpl_next_syntax::ast::Type>) -> (i64, i64) {
    use cpl_next_syntax::ast::{Expr, Type};
    let all = (i64::MIN + 1, i64::MAX);
    let Some(Expr::Range(r)) = (match t {
        Some(Type::Range(t)) => t.expr(),
        _ => None,
    }) else {
        return all;
    };
    let lo = r
        .lo()
        .map_or(Some(all.0), |e| super::tyconv::literal_int(&e));
    let hi = match r.hi() {
        None => Some(all.1),
        Some(e) => super::tyconv::literal_int(&e).and_then(|k| {
            if r.is_inclusive() {
                Some(k)
            } else {
                k.checked_sub(1)
            }
        }),
    };
    (lo.unwrap_or(all.0), hi.unwrap_or(all.1))
}

#[cfg(test)]
mod tests {
    /// `lo..hi` reads its struct off the ends' `@role(endpoint)` impl, so std fills the role with
    /// a trait of one associated type and implements it for every domain (decision 456).
    #[test]
    fn std_fills_the_endpoint_role_and_every_domain_has_it() {
        let std = crate::modules::Std::read();
        let d = &std.decls().d;
        let tr = d.roles.endpoint.expect("std fills `@role(endpoint)`");
        assert_eq!(d.traits[tr.0].assoc.len(), 1);
        assert!(d.trait_impls(tr).count() >= 1);
    }
}

//! Collecting declarations: std's once, and each program's as a layer over them.
use super::impls::{overlap, same_shape, supertraits_unmet};
use super::inherent::inherent_impl;
use super::tyconv::input_bound;
use super::*;
use crate::synth::{enum_structs, module_derived};
use cpl_diag::codes::FIELD_BOUND_FROM_INPUT;
use cpl_diag::msgs::check::FIELD_BOUND_FROM_INPUT_A_FIELD_S_BOUNDS;
use cpl_next_ir::ids::{ItemId, SourceId};
use cpl_next_ir::unbuilt::Unbuilt;

/// Std's declarations, collected once and never written after: the core's and every std
/// module's. A program's declarations are a layer over them.
pub struct Prepared {
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
) -> (Decls, Vec<(ItemId, ast::Stmt)>) {
    let (layer, main) = layer(Some(std), mods, seen, prelude);
    (layer.d, main)
}

/// The declarations of `mods` past `base`'s, over `base`'s; with no base, std's own.
fn layer(
    base: Option<&Prepared>,
    mods: &[crate::modules::Module],
    seen: &[bool],
    prelude: &HashMap<String, String>,
) -> (Prepared, Vec<(ItemId, ast::Stmt)>) {
    let from = base.map_or(0, |b| b.d.module_names.len());
    let mut d = base.map_or_else(Decls::default, |b| b.d.clone());
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
    if d.recipe.is_some_and(|r| !d.seen(d.structs[r.0].module)) {
        d.recipe = None;
    }
    let unseen = |d: &Decls, s: &StructId| !d.seen(d.structs[s.0].module);
    if d.member_types.values().any(|s| unseen(&d, s)) {
        let kept = (d.member_types.iter()).filter(|(_, s)| !unseen(&d, s));
        d.member_types = Arc::new(kept.map(|(m, s)| (m.clone(), *s)).collect());
    }
    if base.is_some() {
        d.top_values = Arc::new(
            (mods[n - 1].items.iter().flat_map(|it| &it.stmts))
                .filter_map(|s| match &s.kind {
                    StmtKind::Let { pat, .. } => match &pat.kind {
                        ast::PatternKind::Name(x) => Some(x.clone()),
                        _ => None,
                    },
                    _ => None,
                })
                .collect(),
        );
    }
    let (mut own, mut public) = match base {
        Some(b) => (b.own.clone(), b.public.clone()),
        None => (Layer::default(), Layer::default()),
    };
    for _ in from..n {
        own.push(HashMap::new());
        public.push(HashSet::new());
    }
    let mut axes = Vec::new();
    for s in mods[from..]
        .iter()
        .flat_map(|m| m.items.iter())
        .flat_map(|it| &it.stmts)
    {
        stmt_axes(s, &mut axes);
    }
    for (lo, hi) in axes {
        if !d.top_doms.contains_key(&anon_key(lo, hi)) {
            let t = fresh_dom(&d);
            if let Ty::Dom(i) = t {
                d.anon_doms.push((i, lo, hi));
            }
            d.top_doms.insert(anon_key(lo, hi), t);
        }
    }
    let mut main = Vec::new();
    // Pass 1: names, so signatures can mention later declarations.
    let mut pending: Vec<(ModId, ItemId, &ast::Stmt, Option<Named>)> = Vec::new();
    let mut op_methods: HashMap<TraitId, String> = HashMap::new();
    // The core's `use` lines, for its view made again over a program's records.
    let mut imports: Vec<(ModId, &ast::Stmt)> = match from {
        0 => Vec::new(),
        _ => (mods[CORE.0].items.iter())
            .flat_map(|it| &it.stmts)
            .filter(|s| matches!(s.kind, StmtKind::Use { .. }))
            .map(|s| (CORE, s))
            .collect(),
    };
    let mut aliases: Vec<(ModId, SourceId, String, Vec<String>, ast::Type)> = Vec::new();
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
    let core_first = mods[CORE.0].items.first();
    let records: Vec<crate::modules::Item> = (core_first
        .and_then(|it| Some((it.id.clone(), it.stmts.first()?.span))))
    .map(|(id, span)| {
        let mut stmts = crate::records::structs(&new_shapes, span);
        stmts.extend(crate::records::row_traits(&new_rows, span));
        stmts.extend(crate::records::without_traits(&new_forms, span));
        vec![crate::modules::Item { id, stmts }]
    })
    .unwrap_or_default();
    let all = || {
        mods.iter()
            .flat_map(|m| m.items.iter())
            .flat_map(|it| &it.stmts)
    };
    let each = crate::synth::derive_each(all);
    let convert = crate::synth::convert_marked(all);
    let positions = crate::records::positions(all());
    let derived: Vec<Vec<(ItemId, String, ast::Stmt)>> = (mods.iter().enumerate())
        .map(|(mi, m)| {
            // Std's modules get only what a program's records add.
            let ours = mi >= from;
            let mut made = match (mi == CORE.0, ours) {
                (true, _) => module_derived(&records, &each),
                (false, true) => module_derived(&m.items, &each),
                (false, false) => Vec::new(),
            };
            if let (true, Some(it)) = (mi == CORE.0, records.first()) {
                if let Some(at) = it.stmts.first() {
                    let impls = crate::records::domain_impls(all(), &new_shapes, at.span);
                    made.extend(impls.into_iter().map(|(n, imp)| (it.id.clone(), n, imp)));
                    // Each new row over every shape, and std's rows over the new shapes.
                    let rows_of = |rs: &[Vec<String>]| crate::modules::Item {
                        id: it.id.clone(),
                        stmts: crate::records::row_traits(rs, at.span),
                    };
                    made.extend(crate::synth::field_trait_impls(
                        &[rows_of(&new_rows)],
                        &shapes,
                    ));
                    if !new_shapes.is_empty() {
                        let old = [rows_of(&old_rows)];
                        made.extend(crate::synth::field_trait_impls(&old, &new_shapes));
                    }
                    let without = (crate::records::without_impls(&new_forms, &shapes, at.span))
                        .into_iter()
                        .chain(crate::records::without_impls(
                            &old_forms,
                            &new_shapes,
                            at.span,
                        ));
                    made.extend(without.map(|(n, imp)| (it.id.clone(), n, imp)));
                }
            }
            if ours || !new_shapes.is_empty() {
                let shapes = if ours { &shapes } else { &new_shapes };
                made.extend(crate::synth::field_trait_impls(&m.items, shapes));
            }
            if let Some(c) = convert.as_ref().filter(|_| ours) {
                made.extend(crate::synth::enum_conversions(
                    &m.items,
                    c,
                    positions.as_ref(),
                ));
            }
            made
        })
        .collect();
    let tables = crate::synth::memo_tables(mods);
    d.memo_tables = tables.clone();
    let synth: Vec<Vec<(ItemId, ast::Stmt)>> = (mods.iter().enumerate())
        .map(|(mi, m)| {
            let mut out = Vec::new();
            if mi == CORE.0 {
                out.extend(
                    (records.iter())
                        .flat_map(|it| it.stmts.iter().map(|s| (it.id.clone(), s.clone()))),
                );
            }
            for crate::modules::Item { id: item, stmts } in m.items.iter().filter(|_| mi >= from) {
                for s in stmts {
                    if let Some((f, t)) = crate::synth::memo_table(s, &tables) {
                        let fns = crate::synth::memo_fns(f, s, t);
                        out.extend(fns.into_iter().map(|x| (item.clone(), x)));
                    }
                    if let StmtKind::Enum(e) = &s.kind {
                        let (structs, mut errors, values) = enum_structs(s, e);
                        errors.extend(crate::synth::enum_level_tags(item, s, e));
                        match ModId(mi) == d.program {
                            true => d.errors.extend(errors),
                            false => d.module_errors[mi].extend(errors),
                        }
                        let vs = e.variants.iter().map(|v| v.name.text.clone()).collect();
                        enums.push((mi, e.name.text.clone(), vs, values));
                        out.extend(structs.into_iter().map(|x| (item.clone(), x)));
                    }
                }
            }
            out
        })
        .collect();
    for (mi, m) in mods.iter().enumerate() {
        let mid = ModId(mi);
        if mi < from && synth[mi].is_empty() {
            continue;
        }
        for crate::modules::Item { id: item, stmts } in m.items.iter() {
            let made = synth[mi].iter().filter(|(i, _)| i == item).map(|(_, s)| s);
            for s in stmts
                .iter()
                .filter(|_| mi >= from)
                .filter(|s| !matches!(s.kind, StmtKind::Enum(_)))
                .chain(made)
            {
                if let Some((f, t)) = crate::synth::memo_table(s, &tables).filter(|_| mi >= from) {
                    main.push((item.clone(), crate::synth::memo_var(f, s.span, t)));
                    continue;
                }
                if let (true, Some(name)) = (s.public, item_name(s)) {
                    public[mi].insert(name.to_string());
                }
                match &s.kind {
                    StmtKind::Struct(sd) => {
                        let id = StructId(d.structs.len());
                        own[mi].insert(sd.name.text.clone(), Named::Struct(id));
                        if marked(s, "tuple") {
                            let n = sd.fields.len();
                            match d.tuples.contains_key(&n) {
                                true => d.errors.push(again(item, s, "tuple")),
                                false => _ = Arc::make_mut(&mut d.tuples).insert(n, id),
                            }
                        }
                        if d.is_library(mid) && s.decorators.iter().any(|x| x.name.text == "recipe")
                        {
                            d.recipe = Some(id);
                        }
                        if marked(s, "cuts") {
                            once(&mut d.cut_struct, id, again(item, s, "cuts"), &mut d.errors);
                        }
                        for (n, slot) in [
                            ("list", &mut d.sugar.list),
                            ("str", &mut d.sugar.str),
                            ("optional", &mut d.sugar.optional),
                            ("map", &mut d.sugar.map),
                            ("array", &mut d.sugar.array),
                        ] {
                            if marked(s, n) {
                                once(slot, id, again(item, s, n), &mut d.errors);
                            }
                        }
                        if marked(s, crate::records::RECORD) {
                            let names = sd.fields.iter().map(|f| f.name.text.clone()).collect();
                            Arc::make_mut(&mut d.records).insert(names, id);
                        }
                        let member = s.decorators.iter().find(|x| x.name.text == "member");
                        if let Some(ast::ExprKind::Name(m)) =
                            member.and_then(|x| x.args.first()).map(|a| &a.value.kind)
                        {
                            Arc::make_mut(&mut d.member_types).insert(m.to_string(), id);
                        }
                        let mut consts = HashMap::new();
                        for m in &sd.methods {
                            if let StmtKind::Const {
                                name,
                                generics,
                                ty,
                                init: Some(init),
                                ..
                            } = &m.kind
                            {
                                if generics.is_empty() {
                                    consts.insert(name.text.clone(), ConstId(d.consts.len()));
                                    d.consts.push(LibConst {
                                        module: mid,
                                        name: format!("{}.{}", sd.name.text, name.text),
                                        item: item.clone(),
                                        ty: ty.clone(),
                                        init: init.clone(),
                                    });
                                }
                            }
                        }
                        d.structs.push(StructDecl {
                            name: sd.name.text.clone(),
                            record: marked(s, crate::records::RECORD),
                            generics: sd.generics.iter().map(|g| g.text.clone()).collect(),
                            fields: Vec::new(),
                            var_fields: Vec::new(),
                            cached: Vec::new(),
                            field_at: Vec::new(),
                            defaults: Vec::new(),
                            module: mid,
                            item: item.clone(),
                            n_sets: 0,
                            methods: HashMap::new(),
                            consts,
                            shaped: HashMap::new(),
                            ctor: None,
                            variants: Vec::new(),
                            errors: Vec::new(),
                            values: Vec::new(),
                            bounds: Vec::new(),
                        });
                        pending.push((mid, item.clone(), s, Some(Named::Struct(id))));
                    }
                    StmtKind::Trait { name, alias, .. } => {
                        let id = TraitId(d.traits.len());
                        if marked(s, "positions") {
                            let e = again(item, s, "positions");
                            once(&mut d.domain_trait, id, e, &mut d.errors);
                        }
                        if marked(s, "convert") {
                            once(
                                &mut d.convert_trait,
                                id,
                                again(item, s, "convert"),
                                &mut d.errors,
                            );
                        }
                        if marked(s, "range_bounds") {
                            let e = again(item, s, "range_bounds");
                            once(&mut d.bounds_trait, id, e, &mut d.errors);
                        }
                        let lit = s.decorators.iter().find(|x| x.name.text == "literal");
                        if let Some(ast::ExprKind::Str(text)) =
                            lit.and_then(|x| x.args.first()).map(|a| &a.value.kind)
                        {
                            if let [ast::StrPiece::Text(t)] = text.as_slice() {
                                match d.literals.iter().any(|(o, _)| o == t) {
                                    true => d.errors.push(again(item, s, "literal")),
                                    false => d.literals.push((t.clone(), id)),
                                }
                            }
                        }
                        let op = s.decorators.iter().find(|x| x.name.text == "operator");
                        if let Some(ast::ExprKind::Str(text)) =
                            op.and_then(|x| x.args.first()).map(|a| &a.value.kind)
                        {
                            if let [ast::StrPiece::Text(t)] = text.as_slice() {
                                let m = operator_method(s);
                                let method = |tr: &TraitId| match op_methods.get(tr) {
                                    Some(m) => Some(m.clone()),
                                    None => d.traits[tr.0].methods.keys().next().cloned(),
                                };
                                let taken =
                                    (d.operators.iter()).any(|(o, tr)| o == t && method(tr) == m);
                                match taken {
                                    true => d.errors.push(again(item, s, "operator")),
                                    false => {
                                        d.operators.push((t.clone(), id));
                                        op_methods.extend(m.map(|m| (id, m)));
                                    }
                                }
                            }
                        }
                        own[mi].insert(name.text.clone(), Named::Trait(id));
                        d.traits.push(TraitDecl {
                            name: name.text.clone(),
                            module: mid,
                            alias: *alias,
                            supers: Vec::new(),
                            methods: HashMap::new(),
                            errors: Vec::new(),
                            defaults: Vec::new(),
                            assoc: Vec::new(),
                            assoc_bounds: Vec::new(),
                            unimplemented: unimplemented(item, s, &mut d.errors),
                            row: marked(s, crate::records::RECORD),
                        });
                        pending.push((mid, item.clone(), s, Some(Named::Trait(id))));
                    }
                    StmtKind::TypeAlias {
                        name,
                        generics,
                        ty,
                        bounds,
                    } if bounds.is_empty() => {
                        let library = d.is_library(mid);
                        for x in (s.decorators.iter())
                            .filter(|x| !(library && read_elsewhere(&x.name.text)))
                        {
                            let at = item.at(x.span);
                            let e = Msg::at(
                                at,
                                Unbuilt::Decorator {
                                    name: x.name.text.clone(),
                                },
                            );
                            match d.is_library(mid) {
                                true => d.module_errors[mi].push(e),
                                false => d.errors.push(e),
                            }
                        }
                        own[mi].insert(
                            name.text.clone(),
                            Named::Alias(AliasId(aliases_from + aliases.len())),
                        );
                        let generics = generics.iter().map(|g| g.text.clone()).collect();
                        let at = item.at(Span::new(s.span.start, s.span.start));
                        aliases.push((mid, at, name.text.clone(), generics, ty.clone()));
                    }
                    StmtKind::Fn(_) | StmtKind::Impl { .. } => {
                        pending.push((mid, item.clone(), s, None))
                    }
                    StmtKind::Const {
                        name,
                        generics,
                        ty,
                        init: Some(init),
                        ..
                    } if d.items_only(mid) && generics.is_empty() => {
                        own[mi].insert(name.text.clone(), Named::Const(ConstId(d.consts.len())));
                        d.consts.push(LibConst {
                            module: mid,
                            name: name.text.clone(),
                            item: item.clone(),
                            ty: ty.clone(),
                            init: init.clone(),
                        });
                    }
                    StmtKind::Use { .. } => imports.push((mid, s)),
                    StmtKind::Context { name, fields } => {
                        let id = ContextId(d.contexts.len());
                        own[mi].insert(name.text.clone(), Named::Context(id));
                        d.contexts.push(ContextDecl {
                            module: mid,
                            name: name.text.clone(),
                            fields: fields.clone(),
                        });
                    }
                    other if d.items_only(mid) => {
                        let e = Unbuilt::LibraryItem {
                            kind: crate::kind_name(other),
                        };
                        match item_name(s) {
                            Some(n) => {
                                own[mi].insert(n.to_string(), Named::Refused(d.refused.len()));
                                d.refused.push(e);
                            }
                            None => d.module_errors[mi].push(e.into()),
                        }
                    }
                    _ => main.push((item.clone(), s.clone())),
                }
            }
        }
    }
    for (mi, name, vs, values) in enums {
        let Some(Named::Struct(id)) = own[mi].get(&name).copied() else {
            continue;
        };
        d.structs[id.0].values = values;
        d.structs[id.0].variants = (vs.into_iter())
            .filter_map(|v| match own[mi].get(&format!("{name}.{v}")) {
                Some(Named::Struct(p)) => Some((v, *p)),
                _ => None,
            })
            .collect();
    }
    for (mi, made) in derived.iter().enumerate() {
        for (item, name, s) in made {
            // A record is core's struct wherever its impl is written.
            let record = |n: &Named| matches!(n, Named::Struct(s) if d.structs[s.0].record);
            let found = own[mi]
                .get(name)
                .or_else(|| own[CORE.0].get(name).filter(|n| record(n)));
            if let Some(Named::Struct(id)) = found.copied() {
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
        for s in mods[n - 1].items.iter().flat_map(|it| &it.stmts) {
            if let StmtKind::Domain { name, def } = &s.kind {
                let t = product(&mut d, &name.text, def).unwrap_or_else(|| fresh_dom(&d));
                d.top_doms.entry(name.text.clone()).or_insert(t);
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
    let aliases = d.aliases.clone();
    // Struct fields first, to a fixed point: a struct holding a struct with function fields has those sets too.
    loop {
        let mut changed = false;
        for (mid, _, s, id) in &pending {
            let (StmtKind::Struct(sd), Some(Named::Struct(id))) = (&s.kind, id) else {
                continue;
            };
            let generics: Vec<String> = sd.generics.iter().map(|g| g.text.clone()).collect();
            let doms = d.doms_in(*mid);
            let (names, sets) = (d.views[mid.0].clone(), d.struct_sets.clone());
            let defaults = d.struct_defaults.clone();
            let tuples = d.tuples.clone();
            let (sugar, records) = (d.sugar, d.records.clone());
            let members = d.member_types.clone();
            let mut cx = TyCx {
                names: &names,
                values: d.top_values.clone(),
                ops: d.operator_fns(),
                item: None,
                struct_sets: &sets,
                struct_defaults: &defaults,
                tuples: &tuples,
                sugar,
                records: &records,
                members: &members,
                generics: &generics,
                outer: &generics,
                args: None,
                aliases: &aliases,
                expanding: Vec::new(),
                doms: &doms,
                ret_slots: None,
                next_set: Some(0),
            };
            let mut errors = Vec::new();
            for f in &sd.fields {
                conv(&mut cx, &f.ty, &mut errors);
            }
            let n = cx.next_set.unwrap_or(0);
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
        let (StmtKind::Struct(sd), Some(Named::Struct(id))) = (&s.kind, id) else {
            continue;
        };
        if sd.generic_defaults.iter().all(Option::is_none) {
            continue;
        }
        let generics: Vec<String> = sd.generics.iter().map(|g| g.text.clone()).collect();
        let doms = d.doms_in(*mid);
        let (names, sets) = (d.views[mid.0].clone(), d.struct_sets.clone());
        let (tuples, defaults) = (d.tuples.clone(), d.struct_defaults.clone());
        let (sugar, records) = (d.sugar, d.records.clone());
        let members = d.member_types.clone();
        let mut cx = TyCx {
            names: &names,
            values: d.top_values.clone(),
            ops: d.operator_fns(),
            item: None,
            struct_sets: &sets,
            struct_defaults: &defaults,
            tuples: &tuples,
            sugar,
            records: &records,
            members: &members,
            generics: &generics,
            outer: &generics,
            args: None,
            aliases: &aliases,
            expanding: Vec::new(),
            doms: &doms,
            ret_slots: None,
            next_set: None,
        };
        let mut errors = Vec::new();
        let tys = (sd.generic_defaults.iter())
            .map(|t| t.as_ref().map(|t| conv(&mut cx, t, &mut errors)))
            .collect();
        Arc::make_mut(&mut d.struct_defaults).insert(*id, tys);
        if d.is_library(*mid) {
            d.structs[id.0].errors.extend(errors);
        } else {
            d.errors.extend(errors);
        }
    }
    // Traits first, for the defaults every use of one reads; impls last, for their trait's fns.
    let pass = |s: &ast::Stmt, id: &Option<Named>| match s.kind {
        StmtKind::Trait { .. } => 0,
        StmtKind::Impl { .. } if id.is_some() => 3,
        StmtKind::Impl { .. } => 2,
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
            match (&s.kind, id) {
                (StmtKind::Struct(sd), Some(Named::Struct(id))) => {
                    let generics: Vec<String> =
                        sd.generics.iter().map(|g| g.text.clone()).collect();
                    let doms = d.doms_in(ModId(mi));
                    let sets = d.struct_sets.clone();
                    let defaults = d.struct_defaults.clone();
                    let tuples = d.tuples.clone();
                    let (sugar, records) = (d.sugar, d.records.clone());
                    let members = d.member_types.clone();
                    let mut cx = TyCx {
                        names: &d.views[mi].clone(),
                        values: d.top_values.clone(),
                        ops: d.operator_fns(),
                        item: None,
                        struct_sets: &sets,
                        struct_defaults: &defaults,
                        tuples: &tuples,
                        sugar,
                        records: &records,
                        members: &members,
                        generics: &generics,
                        outer: &generics,
                        args: None,
                        aliases: &aliases,
                        expanding: Vec::new(),
                        doms: &doms,
                        ret_slots: None,
                        next_set: Some(0),
                    };
                    let mut errors = Vec::new();
                    let fields: Vec<(String, Ty)> =
                        sd.fields
                            .iter()
                            .map(|f| {
                                let ty =
                                    match input_bound(&f.ty, &cx.values) {
                                        Some((written, name)) => {
                                            let text = FIELD_BOUND_FROM_INPUT_A_FIELD_S_BOUNDS
                                                .say(&[&f.name.text, &written, &name]);
                                            let at = item.at(f.ty.span);
                                            errors.push(Msg::at(
                                                at,
                                                Msg::verdict(FIELD_BOUND_FROM_INPUT, text),
                                            ));
                                            Ty::Error
                                        }
                                        None => conv(&mut cx, &f.ty, &mut errors),
                                    };
                                (f.name.text.clone(), ty)
                            })
                            .collect();
                    let n_sets = cx.next_set.unwrap_or(0);
                    d.structs[id.0].bounds = (sd.bounds.iter())
                        .filter_map(|b| Some((generics.iter().position(|g| *g == b.name.text)?, b)))
                        .flat_map(|(i, b)| {
                            b.traits.iter().filter_map(move |t| match &t.kind {
                                TypeKind::Named { name, .. } => Some((i, name)),
                                _ => None,
                            })
                        })
                        .filter_map(|(i, name)| match d.views[mi].get(&name.text) {
                            Some(Named::Trait(tr)) => Some((i, *tr)),
                            _ => None,
                        })
                        .collect();
                    if d.is_library(mid) {
                        d.structs[id.0].errors.extend(errors);
                    } else {
                        d.errors.extend(errors);
                    }
                    let mut methods = HashMap::new();
                    for m in &sd.methods {
                        if let StmtKind::Fn(f) = &m.kind {
                            // The struct's bounds hold in each of its methods (`struct Range<D: Domain>`).
                            let mut f = f.clone();
                            f.bounds.splice(0..0, sd.bounds.iter().cloned());
                            let fid = add_fn(
                                &mut d,
                                mid,
                                item.clone(),
                                m,
                                &f,
                                Within {
                                    owner: Some(Owner::Struct(id)),
                                    generics: &generics,
                                    sets: n_sets,
                                    self_ty: Some(Ty::Struct(
                                        id,
                                        (0..generics.len()).map(Ty::Param).collect(),
                                        (0..n_sets).map(SetTy::Param).collect(),
                                    )),
                                    sig: None,
                                },
                            );
                            methods.insert(f.name.text.clone(), fid);
                        }
                    }
                    let sdecl = &mut d.structs[id.0];
                    sdecl.fields = fields;
                    sdecl.var_fields = sd.fields.iter().map(|f| f.mutable).collect();
                    sdecl.cached = sd.fields.iter().map(|f| f.cached).collect();
                    sdecl.defaults = sd.fields.iter().map(|f| f.default.clone()).collect();
                    sdecl.field_at = sd
                        .fields
                        .iter()
                        .map(|f| item.at(Span::new(f.ty.span.start, f.ty.span.start)))
                        .collect();
                    sdecl.n_sets = n_sets;
                    sdecl.ctor = methods.get("new").copied();
                    sdecl.methods = methods;
                }
                (
                    StmtKind::Trait {
                        body,
                        supers,
                        generics,
                        generic_defaults,
                        ..
                    },
                    Some(Named::Trait(id)),
                ) => {
                    let mut tgen = vec!["Self".to_string()];
                    tgen.extend(generics.iter().map(|g| g.text.clone()));
                    let assoc: Vec<(&str, &Vec<ast::Type>, &ast::Type)> = (body.stmts.iter())
                        .filter_map(|m| match &m.kind {
                            StmtKind::TypeAlias {
                                name, bounds, ty, ..
                            } => Some((name.text.as_str(), bounds, ty)),
                            _ => None,
                        })
                        .collect();
                    tgen.extend(assoc.iter().map(|(a, ..)| format!("Self.{a}")));
                    let mut errors = Vec::new();
                    let names = d.views[mi].clone();
                    let sets = d.struct_sets.clone();
                    let defaults = d.struct_defaults.clone();
                    let tuples = d.tuples.clone();
                    let (sugar, records) = (d.sugar, d.records.clone());
                    let members = d.member_types.clone();
                    let doms = d.doms_in(ModId(mi));
                    let mut cx = TyCx {
                        names: &names,
                        values: d.top_values.clone(),
                        ops: d.operator_fns(),
                        item: None,
                        struct_sets: &sets,
                        struct_defaults: &defaults,
                        tuples: &tuples,
                        sugar,
                        records: &records,
                        members: &members,
                        generics: &tgen,
                        outer: &tgen,
                        args: None,
                        aliases: &aliases,
                        expanding: Vec::new(),
                        doms: &doms,
                        ret_slots: None,
                        next_set: None,
                    };
                    d.traits[id.0].defaults = (generic_defaults.iter())
                        .map(|t| t.as_ref().map(|t| conv(&mut cx, t, &mut errors)))
                        .collect();
                    // `type Taken = Slice<Self>`: what an impl that leaves it out has.
                    for (_, _, t) in &assoc {
                        let t = (!matches!(t.kind, TypeKind::Error))
                            .then(|| conv(&mut cx, t, &mut errors));
                        d.traits[id.0].defaults.push(t);
                    }
                    let first = tgen.len() - assoc.len();
                    let mut assoc_bounds = Vec::new();
                    for (k, (_, bs, _)) in assoc.iter().enumerate() {
                        for b in bs.iter() {
                            if let Some(r) = trait_ref(&mut cx, b, &mut errors) {
                                let r = d.with_defaults(r, &Ty::Param(first + k));
                                assoc_bounds.push((first + k, r));
                            }
                        }
                    }
                    d.traits[id.0].assoc = assoc.iter().map(|(a, ..)| a.to_string()).collect();
                    d.traits[id.0].assoc_bounds = assoc_bounds;
                    // A supertrait is in the trait's own parameters, without `Self`.
                    let own_params = |r: TraitRef| TraitRef {
                        tr: r.tr,
                        args: r.args.iter().map(shift_down).collect(),
                    };
                    let sup: Vec<TraitRef> = supers
                        .iter()
                        .filter_map(|t| trait_ref(&mut cx, t, &mut errors))
                        .map(|r| d.with_defaults(r, &Ty::Param(0)))
                        .map(own_params)
                        .collect();
                    d.module_or_program_errors(mid, errors);
                    let mut methods = HashMap::new();
                    for m in &body.stmts {
                        if let StmtKind::Fn(f) = &m.kind {
                            let fid = add_fn(
                                &mut d,
                                mid,
                                item.clone(),
                                m,
                                f,
                                Within {
                                    owner: Some(Owner::Trait(id)),
                                    generics: &tgen,
                                    sets: 0,
                                    self_ty: Some(Ty::Param(0)),
                                    sig: None,
                                },
                            );
                            // Inside the trait, `Self` has the trait: a default body may call its other fns.
                            let own = TraitRef {
                                tr: id,
                                args: (1..tgen.len()).map(Ty::Param).collect(),
                            };
                            d.fns[fid.0].bounds.push((0, own));
                            methods.insert(f.name.text.clone(), fid);
                        }
                    }
                    d.traits[id.0].methods = methods;
                    d.traits[id.0].supers = sup;
                }
                (
                    StmtKind::Impl {
                        trait_name: Some(tr),
                        target,
                        body,
                        bounds,
                    },
                    _,
                ) => {
                    let names = d.views[mi].clone();
                    // The impl's own type parameters: its bounds' names, then what its header and
                    // bounds write that names nothing (`K` of `impl<S: Indexed<K, T>, K, T>`).
                    let mut generics: Vec<String> =
                        bounds.iter().map(|b| b.name.text.clone()).collect();
                    for t in [tr, target]
                        .into_iter()
                        .chain(bounds.iter().flat_map(|b| &b.traits))
                    {
                        unnamed(t, &names, &mut generics);
                    }
                    let hidden = hidden_params(&d, &names, bounds, &mut generics);
                    let sets = d.struct_sets.clone();
                    let defaults = d.struct_defaults.clone();
                    let tuples = d.tuples.clone();
                    let (sugar, records) = (d.sugar, d.records.clone());
                    let members = d.member_types.clone();
                    let doms = d.doms_in(ModId(mi));
                    let mut errors = Vec::new();
                    let mut cx = TyCx {
                        names: &names,
                        values: d.top_values.clone(),
                        ops: d.operator_fns(),
                        item: Some(item.clone()),
                        struct_sets: &sets,
                        struct_defaults: &defaults,
                        tuples: &tuples,
                        sugar,
                        records: &records,
                        members: &members,
                        generics: &generics,
                        outer: &generics,
                        args: None,
                        aliases: &aliases,
                        expanding: Vec::new(),
                        doms: &doms,
                        ret_slots: None,
                        next_set: Some(0),
                    };
                    let for_ = conv(&mut cx, target, &mut errors);
                    let tref =
                        trait_ref(&mut cx, tr, &mut errors).map(|r| d.with_defaults(r, &for_));
                    // An associated type is the argument the impl's body writes: `type Arc = A`.
                    let mut assoc_at = Vec::new();
                    let mut missing = Vec::new();
                    let tref = tref.map(|mut r| {
                        let assoc = d.traits[r.tr.0].assoc.clone();
                        let n = d.traits[r.tr.0].defaults.len() - assoc.len();
                        if r.args.len() >= n {
                            r.args.truncate(n);
                            for (k, a) in assoc.iter().enumerate() {
                                let ty = body.stmts.iter().find_map(|m| match &m.kind {
                                    StmtKind::TypeAlias { name, ty, .. } if name.text == *a => {
                                        Some((ty, item.at(Span::new(m.span.start, m.span.start))))
                                    }
                                    _ => None,
                                });
                                assoc_at.push(ty.as_ref().map_or(item.at(Span::new(s.span.start, s.span.start)), |t| t.1.clone()));
                                let ty = ty.map(|t| t.0);
                                let default = d.traits[r.tr.0].defaults[n + k].clone();
                                match (ty, default) {
                                    (Some(t), _) => r.args.push(conv(&mut cx, t, &mut errors)),
                                    (None, Some(t)) => {
                                        let mut tys = vec![for_.clone()];
                                        tys.extend(r.args.iter().cloned());
                                        let t = Subst {
                                            tys: &tys,
                                            sets: &[],
                                        }
                                        .ty(&t);
                                        r.args.push(t);
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
                    for b in bounds {
                        let i = generics.iter().position(|g| *g == b.name.text).unwrap_or(0);
                        for (j, t) in b.traits.iter().enumerate() {
                            if let Some(r) = trait_ref(&mut cx, t, &mut errors) {
                                let key = (b.name.text.clone(), j);
                                bound_ref(&d, r, i, key, &hidden, &mut ibounds);
                            }
                        }
                    }
                    let n_sets = cx.next_set.unwrap_or(0);
                    // A library impl the slice cannot build is said where a program could reach
                    // it: through its type when that is a struct, else through its trait.
                    if let (Some(r), true) = (&tref, d.is_library(mid) && !errors.is_empty()) {
                        match &for_ {
                            Ty::Struct(sd, ..) => d.structs[sd.0].errors.append(&mut errors),
                            _ => d.traits[r.tr.0].errors.append(&mut errors),
                        }
                        continue;
                    }
                    // No program can reach a library impl of a trait the slice does not declare.
                    if tref.is_none() && d.is_library(mid) {
                        continue;
                    }
                    let Some(tref) = tref.filter(|_| errors.is_empty()) else {
                        if errors.is_empty() {
                            errors.push(Msg::verdict(
                                cpl_diag::codes::UNKNOWN_TYPE,
                                "impl of an unknown trait",
                            ));
                        }
                        d.module_or_program_errors(mid, errors);
                        continue;
                    };
                    if let Some(Named::Struct(sid)) = id {
                        if !d.field_trait(tref.tr) && !d.fields_have(sid, tref.tr) {
                            wait.push((mid, item, s, id));
                            continue;
                        }
                    }
                    let at = item.at(Span::new(s.span.start, s.span.start));
                    let tname = d.traits[tref.tr.0].name.clone();
                    // An impl for every record with some fields is not a blanket one: a record
                    // takes it over a blanket impl it also meets (R08.4.5).
                    let rank = d.rank_of(&for_, &ibounds);
                    let clashes = |d: &Decls, im: &ImplDecl| {
                        im.trait_.tr == tref.tr
                            && overlap(&im.for_, &for_)
                            && d.impl_rank(im) == rank
                            && (d.traits[tref.tr.0].inputs(&im.trait_.args).iter())
                                .zip(d.traits[tref.tr.0].inputs(&tref.args))
                                .all(|(x, y)| same_shape(x, y))
                    };
                    // Every written impl is made before any derived one, so it meets none.
                    let written = id.is_none();
                    if (d.impls.iter())
                        .any(|im| d.live(im) && !(written && im.derived) && clashes(&d, im))
                    {
                        // A type's own impl replaces its derived one (R08.1.6).
                        if id.is_some() {
                            continue;
                        }
                        d.misdeclared.push((
                            at,
                            cpl_diag::codes::DUPLICATE_DEFINITION,
                            format!("a second impl of `{tname}` for one type (R08.1.2)"),
                        ));
                        continue;
                    }
                    if d.traits[tref.tr.0].alias {
                        d.misdeclared.push((
                            at,
                            cpl_diag::codes::UNSUPPORTED,
                            format!(
                                "`{tname}` is a trait alias: impl each trait it names (R08.2.6)"
                            ),
                        ));
                        continue;
                    }
                    // An impl lives with its trait, its type or a type among the trait's arguments
                    // (`impl Into<E> for Int`), or with its fn for `type_of(f)` (R08.1.11), so it
                    // is never an orphan (R08.5.2).
                    let local = |t: &Ty| match t {
                        Ty::Struct(sd, ..) => d.s(*sd).module == mid,
                        Ty::Clo(c) => matches!(&**c, CloRef::Item(f) if d.f(f.decl).module == mid),
                        _ => false,
                    };
                    if !d.is_library(mid)
                        && d.traits[tref.tr.0].module != mid
                        && !local(&for_)
                        && !tref.args.iter().any(local)
                    {
                        d.misdeclared.push((
                            at,
                            cpl_diag::codes::UNSUPPORTED,
                            format!("`{tname}` and the type are both declared elsewhere: an impl lives with one of them"),
                        ));
                        continue;
                    }
                    // ... and replaces one std derived (R08.1.6).
                    for k in (0..impls_from).filter(|_| written) {
                        if d.impls[k].derived && d.live(&d.impls[k]) && clashes(&d, &d.impls[k]) {
                            d.impls[k].replaced = true;
                        }
                    }
                    let derived = id.is_some();
                    let id = d.impls.len();
                    // A bodiless `@get` member the impl leaves out is met by the field of that
                    // name (R12.11.3).
                    let written = |n: &str| {
                        (body.stmts.iter())
                            .any(|m| matches!(&m.kind, StmtKind::Fn(f) if f.name.text == n))
                    };
                    let mut filled: Vec<ast::Stmt> = Vec::new();
                    if let Ty::Struct(sid, ..) = &for_ {
                        let mut ms: Vec<(&String, &FnId)> =
                            d.traits[tref.tr.0].methods.iter().collect();
                        ms.sort();
                        for (n, &tm) in ms {
                            let tf = d.f(tm);
                            if tf.getter
                                && matches!(tf.body, ast::FnBody::None)
                                && !written(n)
                                && d.s(*sid).fields.iter().any(|(f, _)| f == n)
                            {
                                filled.push(crate::synth::field_getter(n, s.span));
                            }
                        }
                    }
                    let mut methods = HashMap::new();
                    for m in body.stmts.iter().chain(&filled) {
                        if let StmtKind::Fn(f) = &m.kind {
                            let sig = d.traits[tref.tr.0].methods.get(&f.name.text).map(|&tm| {
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
                            let fid = add_fn(
                                &mut d,
                                mid,
                                item.clone(),
                                m,
                                f,
                                Within {
                                    owner: Some(Owner::Impl(id)),
                                    generics: &generics,
                                    sets: n_sets,
                                    self_ty: Some(for_.clone()),
                                    sig,
                                },
                            );
                            // The impl's parameters come first in each method's, with its bounds.
                            d.fns[fid.0].bounds.extend(ibounds.iter().cloned());
                            methods.insert(f.name.text.clone(), fid);
                        }
                    }
                    let code = cpl_diag::codes::MISSING_IMPL;
                    (d.misdeclared).extend(missing.into_iter().map(|m| (at.clone(), code, m)));
                    d.push_impl(ImplDecl {
                        trait_: tref,
                        generics,
                        bounds: ibounds,
                        for_,
                        methods,
                        at,
                        assoc_at,
                        module: mid,
                        derived,
                        replaced: false,
                    });
                }
                (StmtKind::Fn(f), _) => {
                    let fid = add_fn(&mut d, mid, item.clone(), s, f, FREE);
                    let before = own[mi].insert(f.name.text.clone(), Named::Fn(fid));
                    Arc::make_mut(&mut d.views[mi]).insert(f.name.text.clone(), Named::Fn(fid));
                    if d.is_library(mid)
                        && public[mi].contains(&f.name.text)
                        && d.f(fid).self_ty.is_some()
                    {
                        d.methods.entry(f.name.text.clone()).or_default().push(fid);
                    }
                    if let Some(Named::Fn(first)) = before {
                        if d.is_library(mid) {
                            let e = Unbuilt::LibraryOverload {
                                name: f.name.text.clone(),
                            };
                            d.fns[fid.0].errors.push(e.into());
                            continue;
                        }
                        // Two with a `self` are one method's overloads unless one call could take
                        // both (R08.1.4); that test isn't built.
                        if d.f(first).self_ty.is_some() && d.f(fid).self_ty.is_some() {
                            d.errors.push(
                                Unbuilt::MethodsOfOneName {
                                    name: f.name.text.clone(),
                                }
                                .into(),
                            );
                            continue;
                        }
                        // One name and one set of labels is one fn: nothing is chosen by
                        // argument type (R02.11.3, R06.1.6).
                        let labels = |f: FnId| -> Vec<&str> {
                            let mut ls: Vec<&str> =
                                d.f(f).labels.iter().map(String::as_str).collect();
                            ls.sort_unstable();
                            ls
                        };
                        if labels(first) != labels(fid) {
                            d.errors.push(Unbuilt::OverloadByLabels.into());
                        }
                        let code = cpl_diag::codes::OVERLOAD_BY_TYPE;
                        d.misdeclared.push((
                            item.at(Span::new(s.span.start, s.span.start)),
                            code,
                            format!("`{}` is already a fn; one name is one fn", f.name.text),
                        ));
                    }
                }
                (
                    StmtKind::Impl {
                        trait_name: None,
                        target,
                        bounds,
                        body,
                    },
                    _,
                ) => {
                    // Inherent methods live in their type's module only (R08.1.3).
                    let TypeKind::Named { name, .. } = &target.kind else {
                        continue;
                    };
                    let Some(t) = struct_named(&d.views[mi], &name.text) else {
                        continue;
                    };
                    if d.s(t).module != mid {
                        d.misdeclared.push((
                            item.at(Span::new(target.span.start, target.span.start)),
                            cpl_diag::codes::UNSUPPORTED,
                            format!("`{}` is declared in another module; its inherent methods live there", name.text),
                        ));
                        continue;
                    }
                    inherent_impl(&mut d, mid, item, t, target, bounds, body);
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
    domain_params(&mut d, fns_from);
    fresh_results(&mut d);
    let unmet = supertraits_unmet(&d, impls_from);
    d.misdeclared.extend(unmet);
    // A program's lookups go through only the impls it sees.
    let live: Vec<bool> = d.impls.iter().map(|im| d.live(im)).collect();
    if base.is_some() {
        for ks in (d.impls_by_trait.values_mut()).chain(d.impls_by_member.values_mut()) {
            ks.retain(|k| live[*k]);
        }
    }
    let layer = Prepared {
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
        let exports = Arc::new(export_map(sc, mi));
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
    }
}

/// A top-level domain's place among the declared ones.
/// The constant ranges arrays are over in the types `s` declares.
fn stmt_axes(s: &ast::Stmt, out: &mut Vec<(i64, i64)>) {
    let mut types: Vec<&ast::Type> = Vec::new();
    let mut inner: &[ast::Stmt] = &[];
    match &s.kind {
        StmtKind::Fn(f) => {
            types.extend(f.params.iter().filter_map(|p| p.ty.as_ref()));
            types.extend(f.ret.iter().chain(f.lends.as_ref().map(|l| &l.ty)));
        }
        StmtKind::Struct(sd) => {
            types.extend(sd.fields.iter().map(|f| &f.ty));
            inner = &sd.methods;
        }
        StmtKind::Enum(e) => {
            types.extend(
                e.variants
                    .iter()
                    .flat_map(|v| v.fields.iter().map(|(_, t)| t)),
            );
            inner = &e.methods;
        }
        StmtKind::TypeAlias { ty, .. } => types.push(ty),
        StmtKind::Trait { body, .. } | StmtKind::Impl { body, .. } | StmtKind::Mod { body, .. } => {
            inner = &body.stmts
        }
        _ => {}
    }
    types
        .into_iter()
        .for_each(|t| super::tyconv::const_axes(t, out));
    inner.iter().for_each(|s| stmt_axes(s, out));
}

pub(super) fn fresh_dom(d: &Decls) -> Ty {
    Ty::Dom(
        (d.top_doms.values())
            .filter(|t| matches!(t, Ty::Dom(_)))
            .count(),
    )
}

/// `domain P = (A, B)` or `(a: A, b: B)` over domains declared above it, or over ranges: the
/// tuple or record of them (R04.2.10, R04.2.11). A range factor is a domain of its own, named `P.i`, which no program can write.
fn product(d: &mut Decls, name: &str, def: &ast::Expr) -> Option<Ty> {
    let (product, xs): (StructId, Vec<&ast::Expr>) = match &def.kind {
        ast::ExprKind::Tuple(xs) => (*d.tuples.get(&xs.len())?, xs.iter().collect()),
        ast::ExprKind::Record(fs) => {
            let names: Vec<String> = fs.iter().map(|(n, _)| n.text.clone()).collect();
            let record = *d.records.get(&names)?;
            (record, fs.iter().map(|(_, x)| x).collect())
        }
        _ => return None,
    };
    let fits = |x: &ast::Expr| match &x.kind {
        ast::ExprKind::Name(n) => d.top_doms.contains_key(n),
        ast::ExprKind::Range { .. } => true,
        _ => false,
    };
    if !xs.iter().all(|x| fits(x)) {
        return None;
    }
    let mut parts = Vec::new();
    for (i, x) in xs.iter().enumerate() {
        parts.push(match &x.kind {
            ast::ExprKind::Name(n) => d.top_doms[n].clone(),
            _ => {
                let t = fresh_dom(d);
                d.top_doms.insert(factor_name(name, i), t.clone());
                t
            }
        });
    }
    Some(Ty::Struct(product, parts, Vec::new()))
}

/// The name of product `P`'s `i`th factor when it is written as a range.
pub fn factor_name(product: &str, i: usize) -> String {
    format!("{product}.{i}")
}

fn marked(s: &ast::Stmt, n: &str) -> bool {
    s.decorators.iter().any(|x| x.name.text == n)
}

/// A marker names one declaration: a second, in any module, is a duplicate.
pub(super) fn again(item: &ItemId, s: &ast::Stmt, n: &str) -> Msg {
    let code = cpl_diag::codes::DUPLICATE_DEFINITION;
    Msg::at(
        item.at(s.span),
        Msg::verdict(code, format!("a second declaration marked `@{n}`")),
    )
}

pub(super) fn once<T>(slot: &mut Option<T>, id: T, dup: Msg, errors: &mut Vec<Msg>) {
    match slot {
        Some(_) => errors.push(dup),
        None => *slot = Some(id),
    }
}

/// An `@operator` trait's method: `-` marks both `Sub.sub` and `Neg.neg`.
fn operator_method(s: &ast::Stmt) -> Option<String> {
    match &s.kind {
        StmtKind::Trait { body, .. } => body.stmts.iter().find_map(|x| match &x.kind {
            StmtKind::Fn(f) => Some(f.name.text.clone()),
            _ => None,
        }),
        _ => None,
    }
}

/// A trait's `@on_unimplemented(code: C, "help")`, each part optional (R08.1.8).
fn unimplemented(item: &ItemId, s: &ast::Stmt, errors: &mut Vec<Msg>) -> Unimplemented {
    let mut out = Unimplemented::default();
    let Some(x) = s
        .decorators
        .iter()
        .find(|x| x.name.text == "on_unimplemented")
    else {
        return out;
    };
    for a in &x.args {
        match (a.name.as_ref().map(|n| n.text.as_str()), &a.value.kind) {
            (None, ast::ExprKind::Str(pieces)) => {
                let text = (pieces.iter()).map(|p| match p {
                    ast::StrPiece::Text(t) => t.clone(),
                    ast::StrPiece::Expr(e) | ast::StrPiece::Spec(e, _) => match &e.kind {
                        ast::ExprKind::Name(n) => format!("{{{n}}}"),
                        _ => String::new(),
                    },
                });
                out.help = Some(text.collect());
            }
            (Some("code"), ast::ExprKind::Name(c)) if cpl_diag::registry::lookup(c).is_some() => {
                out.code = cpl_diag::registry::lookup(c).map(|i| i.code);
            }
            _ => errors.push(Msg::at(
                item.at(a.value.span),
                Msg::verdict(
                    cpl_diag::codes::UNRESOLVED_NAME,
                    "`@on_unimplemented` takes a message and `code:` a diagnostic code",
                ),
            )),
        }
    }
    out
}

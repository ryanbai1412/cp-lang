//! Collecting declarations: std's once, and each program's as a layer over them.
use super::forms::{Form, Reader, UseForm};
use super::impls::{overlap, same_shape, supertraits_unmet};
use super::inherent::inherent_impl;
use super::tyconv::input_bound;
use super::Impls;
use super::*;
use crate::synth::{enum_structs, module_derived};
use cpl_diag::codes::FIELD_BOUND_FROM_INPUT;
use cpl_diag::msgs::check::FIELD_BOUND_FROM_INPUT_A_FIELD_S_BOUNDS;
use cpl_next_ir::ids::{ItemId, SourceId};
use cpl_next_ir::unbuilt::Unbuilt;
use cpl_next_syntax::ast::AstNode as _;
use cpl_syntax::decorators::Mark;

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
            .filter_map(|(_, f)| let_name(&mut d.syms, f))
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
    let mut memos: Vec<(usize, String)> = Vec::new();
    // Pass 1: names, so signatures can mention later declarations.
    let mut pending: Vec<(ModId, ItemId, &ast::Stmt, Option<Named>)> = Vec::new();
    let mut op_methods: HashMap<TraitId, String> = HashMap::new();
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
    // Only a derive that rebuilds its value copies the `@cache` fields.
    let none_copy =
        |v: Vec<(ItemId, String, ast::Stmt)>| v.into_iter().map(|(i, n, s)| (i, n, s, false));
    let derived: Vec<Vec<(ItemId, String, ast::Stmt, bool)>> = (mods.iter().enumerate())
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
                    made.extend(
                        impls
                            .into_iter()
                            .map(|(n, imp)| (it.id.clone(), n, imp, false)),
                    );
                    // Each new row over every shape, and std's rows over the new shapes.
                    let rows_of = |rs: &[Vec<String>]| crate::modules::Item {
                        id: it.id.clone(),
                        stmts: crate::records::row_traits(rs, at.span),
                    };
                    made.extend(none_copy(crate::synth::field_trait_impls(
                        &[rows_of(&new_rows)],
                        &shapes,
                    )));
                    if !new_shapes.is_empty() {
                        let old = [rows_of(&old_rows)];
                        made.extend(none_copy(crate::synth::field_trait_impls(
                            &old,
                            &new_shapes,
                        )));
                    }
                    let without = (crate::records::without_impls(&new_forms, &shapes, at.span))
                        .into_iter()
                        .chain(crate::records::without_impls(
                            &old_forms,
                            &new_shapes,
                            at.span,
                        ));
                    made.extend(without.map(|(n, imp)| (it.id.clone(), n, imp, false)));
                }
            }
            if ours || !new_shapes.is_empty() {
                let shapes = if ours { &shapes } else { &new_shapes };
                made.extend(none_copy(crate::synth::field_trait_impls(&m.items, shapes)));
            }
            if let Some(c) = convert.as_ref().filter(|_| ours) {
                made.extend(none_copy(crate::synth::enum_conversions(
                    &m.items,
                    c,
                    positions.as_ref(),
                )));
            }
            made
        })
        .collect();
    for (w, (home, name)) in crate::synth::memo_tables(mods) {
        let (w, name) = (d.syms.intern(&w), d.syms.intern(&name));
        d.memo_homes.insert(w, (home, name));
    }
    let (syms, words) = (d.syms.clone(), d.memo_homes.clone());
    let tables = |w: &str| syms.get(w).is_some_and(|w| words.contains_key(&w));
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
                        let fns = crate::synth::memo_fns(f, s, &t);
                        out.extend(fns.into_iter().map(|x| (item.clone(), x)));
                    }
                    if let StmtKind::Enum(e) = &s.kind {
                        let (structs, mut errors, values) = enum_structs(s, e);
                        errors.extend(crate::synth::enum_tags(item, s, e));
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
        let heads = match mi >= from {
            true => crate::items::items(m),
            false => Vec::new(),
        };
        for (k, crate::modules::Item { id: item, stmts }) in m.items.iter().enumerate() {
            let made = synth[mi].iter().filter(|(i, _)| i == item).map(|(_, s)| s);
            let memo = |s: &ast::Stmt| crate::synth::memo_table(s, &tables).is_some();
            if let Some(h) = heads.get(k).filter(|_| !stmts.iter().any(memo)) {
                public[mi].extend(h.exports().map(str::to_string));
            }
            for s in made.clone() {
                if let (true, Some(name)) = (s.public, item_name(s)) {
                    public[mi].insert(name.to_string());
                }
            }
            for s in stmts
                .iter()
                .filter(|_| mi >= from)
                .filter(|s| !matches!(s.kind, StmtKind::Enum(_)))
                .chain(made)
            {
                if let Some((f, t)) = crate::synth::memo_table(s, &tables).filter(|_| mi >= from) {
                    memos.push((mi, f.name.text.clone()));
                    main.push((item.clone(), crate::synth::memo_var(f, f.sig_span, &t)));
                    continue;
                }
                match &s.kind {
                    StmtKind::Struct(sd) => {
                        let id = StructId(d.structs.len());
                        let n = sd.generics.len();
                        // One of a name and another arity is a sibling of the first; one of
                        // the same arity is a duplicate (R02.11.4).
                        match own[mi].get(&sd.name.text) {
                            Some(&Named::Struct(f)) => {
                                let first = (f, d.structs[f.0].generics.len());
                                let of_name = (Arc::make_mut(&mut d.struct_arity).entry(f))
                                    .or_insert_with(|| vec![first]);
                                match of_name.iter().any(|(_, k)| *k == n) {
                                    true => d.errors.push(Msg::at(
                                        item.at(s.span),
                                        Msg::verdict(
                                            cpl_diag::codes::DUPLICATE_DEFINITION,
                                            format!(
                                                "a second struct `{}` with {n} type parameters \
                                                 (R02.11.4)",
                                                sd.name.text
                                            ),
                                        ),
                                    )),
                                    false => of_name.push((id, n)),
                                }
                            }
                            _ => _ = own[mi].insert(sd.name.text.clone(), Named::Struct(id)),
                        }
                        for (x, role) in super::role_marks(s) {
                            // The tuple role takes one struct per field count (R02.5.1).
                            if role == Some(Role::Tuple) {
                                let n = sd.fields.len();
                                match d.tuples.contains_key(&n) {
                                    true => d.errors.push(again(item, s, "role(tuple)")),
                                    false => _ = Arc::make_mut(&mut d.tuples).insert(n, id),
                                }
                                continue;
                            }
                            let slot = match role {
                                Some(Role::Option) => &mut d.roles.option,
                                Some(Role::Recipe) => &mut d.roles.recipe,
                                Some(Role::Cuts) => &mut d.roles.cut,
                                Some(Role::List) => &mut d.sugar.list,
                                Some(Role::Str) => &mut d.sugar.str,
                                Some(Role::Map) => &mut d.sugar.map,
                                Some(Role::Array) => &mut d.sugar.array,
                                Some(Role::Pack) => &mut d.roles.pack,
                                _ => {
                                    d.errors.push(misplaced_role(item, x, role, "struct"));
                                    continue;
                                }
                            };
                            let dup =
                                again(item, s, &format!("role({})", role.map_or("", Role::name)));
                            once(slot, id, dup, &mut d.errors);
                        }
                        if marked(s, Mark::Record) {
                            let names = sd.fields.iter().map(|f| f.name.text.clone()).collect();
                            Arc::make_mut(&mut d.records).insert(names, id);
                        }
                        let member = s.decorators.iter().find(|x| x.mark == Some(Mark::Member));
                        if let Some(ast::ExprKind::Name(m)) =
                            member.and_then(|x| x.args.first()).map(|a| &a.value.kind)
                        {
                            let m = d.syms.intern(m);
                            Arc::make_mut(&mut d.member_types).insert(m, id);
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
                                    let n = d.syms.intern(&name.text);
                                    consts.insert(n, ConstId(d.consts.len()));
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
                            record: marked(s, Mark::Record),
                            generics: sd.generics.iter().map(|g| g.text.clone()).collect(),
                            fields: Vec::new(),
                            var_fields: Vec::new(),
                            pub_fields: Vec::new(),
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
                        for (x, role) in super::role_marks(s) {
                            let slot = match role {
                                Some(Role::Iteration) => &mut d.roles.iteration,
                                Some(Role::Positions) => &mut d.roles.domain,
                                Some(Role::Order) => &mut d.roles.order,
                                Some(Role::Convert) => &mut d.roles.convert,
                                Some(Role::Counted) => &mut d.roles.counted,
                                Some(Role::RangeBounds) => &mut d.roles.bounds,
                                Some(Role::DeriveFields) => &mut d.roles.derive_fields,
                                _ => {
                                    d.errors.push(misplaced_role(item, x, role, "trait"));
                                    continue;
                                }
                            };
                            let dup = again(item, s, "role(..)");
                            once(slot, id, dup, &mut d.errors);
                        }
                        let lit = s.decorators.iter().find(|x| x.mark == Some(Mark::Literal));
                        if let Some(ast::ExprKind::Str(text)) =
                            lit.and_then(|x| x.args.first()).map(|a| &a.value.kind)
                        {
                            if let [ast::StrPiece::Text(t)] = text.as_slice() {
                                let t = d.syms.intern(t);
                                match d.literals.iter().any(|(o, _)| *o == t) {
                                    true => d.errors.push(again(
                                        item,
                                        s,
                                        lit.map_or("", |x| &x.name.text),
                                    )),
                                    false => d.literals.push((t, id)),
                                }
                            }
                        }
                        let op = s.decorators.iter().find(|x| x.mark == Some(Mark::Operator));
                        if let Some(ast::ExprKind::Str(text)) =
                            op.and_then(|x| x.args.first()).map(|a| &a.value.kind)
                        {
                            if let [ast::StrPiece::Text(t)] = text.as_slice() {
                                let m = operator_method(s);
                                let t = d.syms.intern(t);
                                let method = |tr: &TraitId| match op_methods.get(tr) {
                                    Some(m) => Some(m.clone()),
                                    None => d.traits[tr.0]
                                        .methods
                                        .keys()
                                        .next()
                                        .map(|n| d.text(*n).to_string()),
                                };
                                let taken =
                                    (d.operators.iter()).any(|(o, tr)| *o == t && method(tr) == m);
                                match taken {
                                    true => d.errors.push(again(
                                        item,
                                        s,
                                        op.map_or("", |x| &x.name.text),
                                    )),
                                    false => {
                                        d.operators.push((t, id));
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
                            row: marked(s, Mark::Record),
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
                        for x in
                            (s.decorators.iter()).filter(|x| !(library && read_elsewhere(x.mark)))
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
        let vs = (vs.into_iter())
            .filter_map(|v| match own[mi].get(&format!("{name}.{v}")) {
                Some(Named::Struct(p)) => Some((d.syms.intern(&v), *p)),
                _ => None,
            })
            .collect();
        d.structs[id.0].variants = vs;
    }
    // The derived impls that read their struct's `@cache` fields, by the impl they are.
    let mut copying: HashSet<*const ast::Stmt> = HashSet::new();
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
            let n = match &s.kind {
                StmtKind::Impl { target, .. } => match &target.kind {
                    TypeKind::Named { args, .. } => args.len(),
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
        for it in mods[n - 1].items.iter() {
            for s in &it.stmts {
                if let StmtKind::Domain { name, def, .. } = &s.kind {
                    let t = product(&mut d, &it.id, &name.text, def);
                    if !d.top_doms.contains_key(&name.text) {
                        let t = t.unwrap_or_else(|| d.new_dom(&it.id));
                        d.top_doms.insert(name.text.clone(), t);
                    }
                }
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
            let (StmtKind::Struct(sd), Some(Named::Struct(id))) = (&s.kind, id) else {
                continue;
            };
            let generics: Vec<String> = sd.generics.iter().map(|g| g.text.clone()).collect();
            let doms = d.doms_in(*mid);
            let names = d.views[mid.0].clone();
            let syms = d.syms.clone();
            let tables = d.tables();
            let mut cx = TyCx {
                names: &|n: &str| look(&syms, &names, n),
                env: &tables,
                item: None,
                generics: &generics,
                outer: &generics,
                args: None,
                expanding: Vec::new(),
                doms: &doms,
                scoped: &tyconv::unscoped,
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
        let names = d.views[mid.0].clone();
        let syms = d.syms.clone();
        let tables = d.tables();
        let mut cx = TyCx {
            names: &|n: &str| look(&syms, &names, n),
            env: &tables,
            item: None,
            generics: &generics,
            outer: &generics,
            args: None,
            expanding: Vec::new(),
            doms: &doms,
            scoped: &tyconv::unscoped,
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
                    let names = d.views[mi].clone();
                    let syms = d.syms.clone();
                    let tables = d.tables();
                    let mut cx = TyCx {
                        names: &|n: &str| look(&syms, &names, n),
                        env: &tables,
                        item: None,
                        generics: &generics,
                        outer: &generics,
                        args: None,
                        expanding: Vec::new(),
                        doms: &doms,
                        scoped: &tyconv::unscoped,
                        ret_slots: None,
                        next_set: Some(0),
                    };
                    let mut errors = Vec::new();
                    let fields: Vec<_> =
                        sd.fields
                            .iter()
                            .map(|f| {
                                let ty =
                                    match input_bound(&f.ty, &tables.values, &tables.syms) {
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
                        .filter_map(|(i, name)| match look(&d.syms, &d.views[mi], &name.text) {
                            Some(Named::Trait(tr)) => Some((i, tr)),
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
                            methods.insert(d.syms.intern(&f.name.text), fid);
                        }
                    }
                    let fields = (fields.into_iter())
                        .map(|(n, t)| (d.syms.intern(&n), t))
                        .collect();
                    let ctor = d.member(&methods, "new");
                    let sdecl = &mut d.structs[id.0];
                    sdecl.fields = fields;
                    sdecl.var_fields = sd.fields.iter().map(|f| f.mutable).collect();
                    sdecl.pub_fields = sd.fields.iter().map(|f| f.public).collect();
                    sdecl.cached = sd.fields.iter().map(|f| f.cached).collect();
                    sdecl.defaults = sd.fields.iter().map(|f| f.default.clone()).collect();
                    sdecl.field_at = sd
                        .fields
                        .iter()
                        .map(|f| item.at(Span::new(f.ty.span.start, f.ty.span.start)))
                        .collect();
                    sdecl.n_sets = n_sets;
                    sdecl.ctor = ctor;
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
                    let syms = d.syms.clone();
                    let doms = d.doms_in(ModId(mi));
                    let tables = d.tables();
                    let mut cx = TyCx {
                        names: &|n: &str| look(&syms, &names, n),
                        env: &tables,
                        item: None,
                        generics: &tgen,
                        outer: &tgen,
                        args: None,
                        expanding: Vec::new(),
                        doms: &doms,
                        scoped: &tyconv::unscoped,
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
                            methods.insert(d.syms.intern(&f.name.text), fid);
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
                    let syms = d.syms.clone();
                    // The impl's own type parameters: its bounds' names, then what its header and
                    // bounds write that names nothing (`K` of `impl<S: Indexed<K, T>, K, T>`).
                    let mut generics: Vec<String> =
                        bounds.iter().map(|b| b.name.text.clone()).collect();
                    for t in [tr, target]
                        .into_iter()
                        .chain(bounds.iter().flat_map(|b| &b.traits))
                    {
                        unnamed(t, &|n: &str| look(&syms, &names, n), &mut generics);
                    }
                    let hidden =
                        hidden_params(&d, &|n: &str| look(&syms, &names, n), bounds, &mut generics);
                    // `_..=_`: a parameter of the impl's own that only a range type meets, which
                    // its body names `Self`.
                    let ranges_only = any_range(target);
                    let range_param = ranges_only.then(|| {
                        generics.push("Self".to_string());
                        generics.len() - 1
                    });
                    let doms = d.doms_in(ModId(mi));
                    let mut errors = Vec::new();
                    let tables = d.tables();
                    let mut cx = TyCx {
                        names: &|n: &str| look(&syms, &names, n),
                        env: &tables,
                        item: Some(item.clone()),
                        generics: &generics,
                        outer: &generics,
                        args: None,
                        expanding: Vec::new(),
                        doms: &doms,
                        scoped: &tyconv::unscoped,
                        ret_slots: None,
                        next_set: Some(0),
                    };
                    let for_ = match range_param {
                        Some(i) => Ty::Param(i),
                        None => conv(&mut cx, target, &mut errors),
                    };
                    let head = match range_param {
                        Some(i) => Ty::AnyRange(i),
                        None => for_.clone(),
                    };
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
                    // A range type is a domain with an order (R04.2.31), so `Self` in a `_..=_`
                    // impl's fns meets `Domain` and `Ordered`; the impl itself is not bound by them.
                    let mut fn_bounds = ibounds.clone();
                    if let Some(i) = range_param {
                        for tr in [d.roles.domain, d.roles.order].into_iter().flatten() {
                            fn_bounds.push((
                                i,
                                TraitRef {
                                    tr,
                                    args: Vec::new(),
                                },
                            ));
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
                    if let Some(Named::Struct(sid)) = id.filter(|_| !d.field_trait(tref.tr)) {
                        let Some(asked) = Impls::of(&d).derived_bounds(
                            sid,
                            &tref,
                            copying.contains(&(s as *const _)),
                        ) else {
                            wait.push((mid, item, s, id));
                            continue;
                        };
                        for b in asked {
                            if !ibounds.contains(&b) {
                                fn_bounds.push(b.clone());
                                ibounds.push(b);
                            }
                        }
                    }
                    let at = item.at(Span::new(s.span.start, s.span.start));
                    let tname = d.traits[tref.tr.0].name.clone();
                    let top = mods[mi].items.iter().flat_map(|it| &it.stmts);
                    impl_names(&mut d, top, &item, &tref, body);
                    // An impl for every record with some fields is not a blanket one: a record
                    // takes it over a blanket impl it also meets (R08.4.5).
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
                    if d.traits[tref.tr.0].module != mid
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
                    let mut filled: Vec<ast::Stmt> = body
                        .stmts
                        .iter()
                        .filter_map(crate::synth::impl_value)
                        .collect();
                    let valued: Vec<String> = (filled.iter())
                        .filter_map(|m| match &m.kind {
                            StmtKind::Fn(f) => Some(f.name.text.clone()),
                            _ => None,
                        })
                        .collect();
                    let written = |n: &str| written(n) || valued.iter().any(|v| v == n);
                    if let Ty::Struct(sid, ..) = &for_ {
                        let mut ms: Vec<(Sym, FnId)> = (d.traits[tref.tr.0].methods.iter())
                            .map(|(&n, &f)| (n, f))
                            .collect();
                        ms.sort_by_key(|&(n, _)| d.text(n));
                        for (n, tm) in ms {
                            let n = d.text(n);
                            let tf = d.f(tm);
                            if tf.getter
                                && matches!(tf.body, ast::FnBody::None)
                                && !written(n)
                                && d.field(*sid, n).is_some()
                            {
                                filled.push(crate::synth::field_getter(n, s.span));
                            }
                        }
                    }
                    let mut methods = HashMap::new();
                    for m in body.stmts.iter().chain(&filled) {
                        if let StmtKind::Fn(f) = &m.kind {
                            let sig =
                                d.member(&d.traits[tref.tr.0].methods, &f.name.text)
                                    .map(|tm| {
                                        let td = d.f(tm);
                                        let mut tys = vec![for_.clone()];
                                        tys.extend(tref.args.iter().cloned());
                                        let sub = Subst {
                                            tys: &tys,
                                            sets: &[],
                                        };
                                        TraitSig {
                                            self_mode: td.self_mode,
                                            params: td
                                                .params
                                                .iter()
                                                .map(|p| sub.ty(&p.ty))
                                                .collect(),
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
                            d.fns[fid.0].bounds.extend(fn_bounds.iter().cloned());
                            methods.insert(d.syms.intern(&f.name.text), fid);
                        }
                    }
                    let code = cpl_diag::codes::MISSING_IMPL;
                    (d.misdeclared).extend(missing.into_iter().map(|m| (at.clone(), code, m)));
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
                }
                (StmtKind::Fn(f), _) => {
                    let fid = add_fn(&mut d, mid, item.clone(), s, f, FREE);
                    let before = own[mi].insert(f.name.text.clone(), Named::Fn(fid));
                    let k = d.syms.intern(&f.name.text);
                    Arc::make_mut(&mut d.views[mi]).insert(k, Named::Fn(fid));
                    if public[mi].contains(&f.name.text) && d.f(fid).self_ty.is_some() {
                        let n = d.syms.intern(&f.name.text);
                        d.methods.entry(n).or_default().push(fid);
                    }
                    if let Some(Named::Fn(first)) = before {
                        // Free fns of one name whose required labels differ are overloads: the
                        // first stands for the name (R06.1.6).
                        let free = d.f(first).self_ty.is_none() && d.f(fid).self_ty.is_none();
                        let rest = d.overloads.get(&first).map_or(&[][..], Vec::as_slice);
                        let clash = (std::iter::once(&first).chain(rest))
                            .any(|o| d.required_labels(*o) == d.required_labels(fid));
                        if free && !clash {
                            own[mi].insert(f.name.text.clone(), Named::Fn(first));
                            let k = d.syms.intern(&f.name.text);
                            Arc::make_mut(&mut d.views[mi]).insert(k, Named::Fn(first));
                            d.overloads.entry(first).or_default().push(fid);
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
                        // One name and one set of required labels is one fn: nothing is chosen
                        // by argument type (R02.11.3, R06.1.6).
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
                    let Some(t) = d.struct_named(ModId(mi), &name.text) else {
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
fn product(d: &mut Decls, item: &ItemId, name: &str, def: &ast::Expr) -> Option<Ty> {
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
                let t = d.new_dom(item);
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

fn marked(s: &ast::Stmt, m: Mark) -> bool {
    s.mark(m).is_some()
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
fn operator_method(t: &super::forms::TraitForm) -> Option<Sym> {
    t.methods.first().map(|m| m.name)
}

/// What `@on_unimplemented("..", code: C)` says when a type lacks the trait's impl (R08.1.8).
fn unimplemented(
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
        let message = match (&label, &value) {
            (None, Some(next::Expr::Literal(l))) => {
                l.token().filter(|t| t.kind() == TokenKind::Str)
            }
            _ => None,
        };
        let code = match (label.as_deref(), &value) {
            (Some("code"), Some(next::Expr::Name(n))) => n.ident(),
            _ => None,
        };
        if let Some(t) = message {
            let text = t.text();
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
fn any_range(t: &super::forms::TyForm) -> bool {
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
fn impl_names(
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
            .find(|(_, &tm)| d.f(tm).was.iter().any(|w| *w == n))
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

/// Each item of `m` as its form, read once; and the decorators on them no attribute names, each
/// with the item it is on, for E0107.
fn read_forms(
    syms: &mut crate::sym::Syms,
    m: &crate::modules::Module,
) -> (
    Vec<(ItemId, Form)>,
    Vec<(ItemId, cpl_next_syntax::ast::Decorator)>,
) {
    let mut reader = Reader { syms };
    let (mut forms, mut unknown) = (Vec::new(), Vec::new());
    for it in m.items.iter() {
        let (form, other) = reader.item(&it.item);
        unknown.extend(other.into_iter().map(|x| (it.id.clone(), x)));
        forms.push((it.id.clone(), form));
    }
    (forms, unknown)
}

/// The name a top-level `let` binds, when it binds one name: what `top_values` holds.
fn let_name(syms: &mut crate::sym::Syms, form: &Form) -> Option<Sym> {
    use cpl_next_syntax::ast::{Pattern, Stmt};
    let Form::Stmt(Stmt::Let(l)) = form else {
        return None;
    };
    let pats: Vec<Pattern> = l.patterns().collect();
    match pats.as_slice() {
        [Pattern::Path(p)] if p.args().is_none() => {
            let segs: Vec<_> = p.segments().collect();
            match segs.as_slice() {
                [n] => Some(syms.intern(n.text())),
                _ => None,
            }
        }
        _ => None,
    }
}

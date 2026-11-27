//! Collecting declarations: std's once, and each program's as a layer over them.
use super::impls::{overlap, same_shape, supertraits_unmet};
use super::inherent::inherent_impl;
use super::*;
use crate::synth::{enum_structs, module_derived};

/// Std's declarations, collected once and never written after: the core's and every std
/// module's. A program's declarations are a layer over them.
pub struct Prepared {
    d: Decls,
    own: Layer<HashMap<String, Named>>,
    public: Layer<HashSet<String>>,
    /// The record shapes std writes, sorted.
    shapes: Vec<Vec<String>>,
    /// Each record shape std writes and the modules that write it.
    writers: HashMap<Vec<String>, Vec<ModId>>,
    /// Each name a std free method answers to, `@aka` and `@was` too, and its modules.
    methods: HashMap<String, Vec<ModId>>,
}

impl Prepared {
    /// The std modules `d` does not load that declare a method `names` calls and found nowhere:
    /// a method call needs no import (R11.3.4), so a check that finds none asks for these.
    pub fn declaring(&self, names: &[String], d: &Decls) -> Vec<String> {
        let mut out: Vec<String> = (names.iter())
            .flat_map(|n| self.methods.get(n).into_iter().flatten())
            .filter(|m| !d.seen(**m))
            .map(|m| d.module_names[m.0].clone())
            .collect();
        out.sort();
        out.dedup();
        out
    }
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
    for fs in std.d.methods.values() {
        for f in fs {
            let fd = std.d.f(*f);
            for n in std::iter::once(&fd.name).chain(&fd.aka).chain(&fd.was) {
                std.methods.entry(n.clone()).or_default().push(fd.module);
            }
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
) -> (Decls, Vec<(u32, ast::Stmt)>) {
    let (layer, main) = layer(Some(std), mods, seen, prelude);
    (layer.d, main)
}

/// The declarations of `mods` past `base`'s, over `base`'s; with no base, std's own.
fn layer(
    base: Option<&Prepared>,
    mods: &[crate::modules::Module],
    seen: &[bool],
    prelude: &HashMap<String, String>,
) -> (Prepared, Vec<(u32, ast::Stmt)>) {
    let from = base.map_or(0, |b| b.d.module_names.len());
    let mut d = base.map_or_else(Decls::default, |b| b.d.clone());
    let (fns_from, impls_from, aliases_from) = (d.fns.len(), d.impls.len(), d.aliases.len());
    let n = mods.len();
    d.program = ModId(match base {
        Some(_) => n - 1,
        None => usize::MAX,
    });
    d.module_names = mods.iter().map(|m| m.name.clone()).collect();
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
        for s in mods[n - 1].items.iter().flat_map(|(_, stmts)| stmts) {
            if let StmtKind::Domain { name, def } = &s.kind {
                let k = (d.top_doms.values())
                    .filter(|t| matches!(t, Ty::Dom(_)))
                    .count();
                let t = product(&d, def).unwrap_or(Ty::Dom(k));
                d.top_doms.entry(name.text.clone()).or_insert(t);
            }
        }
        d.top_values = Arc::new(
            (mods[n - 1].items.iter().flat_map(|(_, stmts)| stmts))
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
    let mut main = Vec::new();
    // Pass 1: names, so signatures can mention later declarations.
    let mut pending: Vec<(ModId, u32, &ast::Stmt, Option<Named>)> = Vec::new();
    // The core's `use` lines, for its view made again over a program's records.
    let mut imports: Vec<(ModId, &ast::Stmt)> = match from {
        0 => Vec::new(),
        _ => (mods[CORE.0].items.iter())
            .flat_map(|(_, ss)| ss)
            .filter(|s| matches!(s.kind, StmtKind::Use { .. }))
            .map(|s| (CORE, s))
            .collect(),
    };
    let mut aliases: Vec<(ModId, usize, String, Vec<String>, ast::Type)> = Vec::new();
    let mut enums: Vec<(usize, String, Vec<String>, Vec<i64>)> = Vec::new();
    // Every record shape the program and its modules write is a core struct (R02.5.2).
    // A program's are those std does not write.
    let mut shapes = base.map_or_else(Vec::new, |b| b.shapes.clone());
    let written = crate::records::shapes(&mods[from..], &crate::records::variadics(mods));
    let new_shapes: Vec<Vec<String>> = (written.iter())
        .filter(|s| shapes.binary_search(s).is_err())
        .cloned()
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
    let core_first = mods[CORE.0].items.first();
    let records: Vec<(u32, Vec<ast::Stmt>)> = (core_first
        .and_then(|(i, ss)| Some((*i, ss.first()?.span))))
    .map(|(i, span)| vec![(i, crate::records::structs(&new_shapes, span))])
    .unwrap_or_default();
    let each = crate::synth::derive_each(&mods[CORE.0].items);
    let convert = crate::synth::convert_marked(&mods[CORE.0].items);
    let derived: Vec<Vec<(u32, String, ast::Stmt)>> = (mods.iter().enumerate())
        .map(|(mi, m)| {
            // Std's modules get only what a program's records add.
            let ours = mi >= from;
            let mut made = match (mi == CORE.0, ours) {
                (true, _) => module_derived(&records, &each),
                (false, true) => module_derived(&m.items, &each),
                (false, false) => Vec::new(),
            };
            if ours || !new_shapes.is_empty() {
                let shapes = if ours { &shapes } else { &new_shapes };
                made.extend(crate::synth::field_trait_impls(&m.items, shapes));
            }
            if let Some(c) = convert.as_ref().filter(|_| ours) {
                made.extend(crate::synth::enum_conversions(&m.items, c));
            }
            made
        })
        .collect();
    let synth: Vec<Vec<(u32, ast::Stmt)>> = (mods.iter().enumerate())
        .map(|(mi, m)| {
            let mut out = Vec::new();
            if mi == CORE.0 {
                out.extend((records.iter()).flat_map(|(i, ss)| ss.iter().map(|s| (*i, s.clone()))));
            }
            for (item, stmts) in m.items.iter().filter(|_| mi >= from) {
                for s in stmts {
                    if let StmtKind::Enum(e) = &s.kind {
                        let (structs, errors, values) = enum_structs(s, e);
                        match ModId(mi) == d.program {
                            true => d.errors.extend(errors),
                            false => d.module_errors[mi].extend(errors),
                        }
                        let vs = e.variants.iter().map(|v| v.name.text.clone()).collect();
                        enums.push((mi, e.name.text.clone(), vs, values));
                        out.extend(structs.into_iter().map(|x| (*item, x)));
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
        for (item, stmts) in m.items.iter() {
            let made = synth[mi].iter().filter(|(i, _)| i == item).map(|(_, s)| s);
            for s in stmts
                .iter()
                .filter(|_| mi >= from)
                .filter(|s| !matches!(s.kind, StmtKind::Enum(_)))
                .chain(made)
            {
                if let (true, Some(name)) = (s.public, item_name(s)) {
                    public[mi].insert(name.to_string());
                }
                match &s.kind {
                    StmtKind::Struct(sd) => {
                        let id = StructId(d.structs.len());
                        own[mi].insert(sd.name.text.clone(), Named::Struct(id));
                        if mi == CORE.0 && s.decorators.iter().any(|x| x.name.text == "tuple") {
                            Arc::make_mut(&mut d.tuples).insert(sd.fields.len(), id);
                        }
                        if d.is_library(mid) && s.decorators.iter().any(|x| x.name.text == "recipe")
                        {
                            d.recipe = Some(id);
                        }
                        if mi == CORE.0 && s.decorators.iter().any(|x| x.name.text == "cuts") {
                            d.cut_struct = Some(id);
                        }
                        let member = s.decorators.iter().find(|x| x.name.text == "member");
                        if let Some(ast::ExprKind::Name(m)) =
                            member.and_then(|x| x.args.first()).map(|a| &a.value.kind)
                        {
                            Arc::make_mut(&mut d.member_types).insert(m.to_string(), id);
                        }
                        d.structs.push(StructDecl {
                            name: sd.name.text.clone(),
                            generics: sd.generics.iter().map(|g| g.text.clone()).collect(),
                            fields: Vec::new(),
                            var_fields: Vec::new(),
                            cached: Vec::new(),
                            field_at: Vec::new(),
                            defaults: Vec::new(),
                            module: mid,
                            item: *item,
                            n_sets: 0,
                            methods: HashMap::new(),
                            shaped: HashMap::new(),
                            ctor: None,
                            variants: Vec::new(),
                            errors: Vec::new(),
                            values: Vec::new(),
                            bounds: Vec::new(),
                        });
                        pending.push((mid, *item, s, Some(Named::Struct(id))));
                    }
                    StmtKind::Trait { name, alias, .. } => {
                        let id = TraitId(d.traits.len());
                        if mi == CORE.0 && s.decorators.iter().any(|x| x.name.text == "positions") {
                            d.domain_trait = Some(id);
                        }
                        if mi == CORE.0 && s.decorators.iter().any(|x| x.name.text == "convert") {
                            d.convert_trait = Some(id);
                        }
                        let op = s.decorators.iter().find(|x| x.name.text == "operator");
                        if let Some(ast::ExprKind::Str(text)) =
                            op.and_then(|x| x.args.first()).map(|a| &a.value.kind)
                        {
                            if let (CORE, [ast::StrPiece::Text(t)]) = (mid, text.as_slice()) {
                                d.operators.push((t.clone(), id));
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
                        });
                        pending.push((mid, *item, s, Some(Named::Trait(id))));
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
                            let at = Span::new(*item + x.span.start, *item + x.span.end);
                            let e = Msg::at(at, format!("slice: decorator `@{}`", x.name.text));
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
                        let at = (*item + s.span.start) as usize;
                        aliases.push((mid, at, name.text.clone(), generics, ty.clone()));
                    }
                    StmtKind::Fn(_) | StmtKind::Impl { .. } => pending.push((mid, *item, s, None)),
                    StmtKind::Const {
                        name,
                        generics,
                        ty,
                        init: Some(init),
                        ..
                    } if d.is_library(mid) && generics.is_empty() => {
                        own[mi].insert(name.text.clone(), Named::Const(ConstId(d.consts.len())));
                        d.consts.push(LibConst {
                            module: mid,
                            name: name.text.clone(),
                            item: *item,
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
                    other if d.is_library(mid) => {
                        let e = format!("slice: library item {}", crate::kind_name(other));
                        match item_name(s) {
                            Some(n) => {
                                own[mi].insert(n.to_string(), Named::Refused(d.refused.len()));
                                d.refused.push(e);
                            }
                            None => d.module_errors[mi].push(e.into()),
                        }
                    }
                    _ => main.push((*item, s.clone())),
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
            let owner = match crate::records::is_record(name) {
                true => CORE.0,
                false => mi,
            };
            if let Some(Named::Struct(id)) = own[owner].get(name).copied() {
                pending.push((ModId(mi), *item, s, Some(Named::Struct(id))));
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
    };
    remake_views(&mut d, &sc, from, core_too, None);
    let made: Vec<AliasDecl> = aliases
        .into_iter()
        .map(|(m, at, name, generics, ty)| {
            let sc = Scopes {
                own: &own,
                public: &public,
                names: &names,
                imports: &imports,
                prelude,
            };
            let mut v = outer_view(&sc, m.0, m.0 == program, None, None);
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
            d.module_errors[a.module.0]
                .push(format!("type alias `{}` is defined by itself", a.name).into());
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
            let members = d.member_types.clone();
            let mut cx = TyCx {
                names: &names,
                values: d.top_values.clone(),
                ops: d.operator_fns(),
                item: None,
                struct_sets: &sets,
                struct_defaults: &defaults,
                tuples: &tuples,
                members: &members,
                generics: &generics,
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
        let members = d.member_types.clone();
        let mut cx = TyCx {
            names: &names,
            values: d.top_values.clone(),
            ops: d.operator_fns(),
            item: None,
            struct_sets: &sets,
            struct_defaults: &defaults,
            tuples: &tuples,
            members: &members,
            generics: &generics,
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
    loop {
        let mut wait = Vec::new();
        let before = queue.len();
        for (mid, item, s, id) in std::mem::take(&mut queue) {
            let mi = mid.0;
            for k in stamped..d.fns.len() {
                d.fns[k].made = made;
            }
            (stamped, made) = (d.fns.len(), (round, pass(s, &id)));
            match (&s.kind, id) {
                (StmtKind::Struct(sd), Some(Named::Struct(id))) => {
                    let generics: Vec<String> =
                        sd.generics.iter().map(|g| g.text.clone()).collect();
                    let doms = d.doms_in(ModId(mi));
                    let sets = d.struct_sets.clone();
                    let defaults = d.struct_defaults.clone();
                    let tuples = d.tuples.clone();
                    let members = d.member_types.clone();
                    let mut cx = TyCx {
                        names: &d.views[mi].clone(),
                        values: d.top_values.clone(),
                        ops: d.operator_fns(),
                        item: None,
                        struct_sets: &sets,
                        struct_defaults: &defaults,
                        tuples: &tuples,
                        members: &members,
                        generics: &generics,
                        args: None,
                        aliases: &aliases,
                        expanding: Vec::new(),
                        doms: &doms,
                        ret_slots: None,
                        next_set: Some(0),
                    };
                    let mut errors = Vec::new();
                    let fields: Vec<(String, Ty)> = sd
                        .fields
                        .iter()
                        .map(|f| (f.name.text.clone(), conv(&mut cx, &f.ty, &mut errors)))
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
                                item,
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
                        .map(|f| (item + f.ty.span.start) as usize)
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
                        members: &members,
                        generics: &tgen,
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
                                item,
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
                    let sets = d.struct_sets.clone();
                    let defaults = d.struct_defaults.clone();
                    let tuples = d.tuples.clone();
                    let members = d.member_types.clone();
                    let doms = d.doms_in(ModId(mi));
                    let mut errors = Vec::new();
                    let mut cx = TyCx {
                        names: &names,
                        values: d.top_values.clone(),
                        ops: d.operator_fns(),
                        item: Some(item),
                        struct_sets: &sets,
                        struct_defaults: &defaults,
                        tuples: &tuples,
                        members: &members,
                        generics: &generics,
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
                                        Some((ty, (item + m.span.start) as usize))
                                    }
                                    _ => None,
                                });
                                assoc_at.push(ty.map_or((item + s.span.start) as usize, |t| t.1));
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
                        for t in &b.traits {
                            if let Some(r) = trait_ref(&mut cx, t, &mut errors) {
                                ibounds.push((i, d.with_defaults(r, &Ty::Param(i))));
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
                            errors.push(
                                format!(
                                    "{}: impl of an unknown trait",
                                    cpl_diag::codes::UNKNOWN_TYPE.0
                                )
                                .into(),
                            );
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
                    let at = (item + s.span.start) as usize;
                    let tname = d.traits[tref.tr.0].name.clone();
                    let clashes = |d: &Decls, im: &ImplDecl| {
                        im.trait_.tr == tref.tr
                            && overlap(&im.for_, &for_)
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
                            cpl_diag::codes::DUPLICATE_DEFINITION.0,
                            format!("a second impl of `{tname}` for one type (R08.1.2)"),
                        ));
                        continue;
                    }
                    if d.traits[tref.tr.0].alias {
                        d.misdeclared.push((
                            at,
                            cpl_diag::codes::UNSUPPORTED.0,
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
                            cpl_diag::codes::UNSUPPORTED.0,
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
                                filled.push(crate::synth::field_getter(n, s));
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
                                item,
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
                    let code = cpl_diag::codes::MISSING_IMPL.0;
                    (d.misdeclared).extend(missing.into_iter().map(|m| (at, code, m)));
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
                    let fid = add_fn(&mut d, mid, item, s, f, FREE);
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
                            let e =
                                format!("slice: a library with two fns named `{}`", f.name.text);
                            d.fns[fid.0].errors.push(e.into());
                            continue;
                        }
                        // Two with a `self` are one method's overloads unless one call could take
                        // both (R08.1.4); that test isn't built.
                        if d.f(first).self_ty.is_some() && d.f(fid).self_ty.is_some() {
                            d.errors.push(
                                format!(
                                    "slice: methods of one name in one module (`{}`)",
                                    f.name.text
                                )
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
                            d.errors
                                .push("slice: fns of one name told apart by their labels".into());
                        }
                        let code = cpl_diag::codes::OVERLOAD_BY_TYPE;
                        d.misdeclared.push((
                            (item + s.span.start) as usize,
                            code.0,
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
                            (item + target.span.start) as usize,
                            cpl_diag::codes::UNSUPPORTED.0,
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
    };
    remake_views(&mut d, &sc, from, core_too, Some(&mut errors));
    d.errors.extend(errors);
    domain_params(&mut d, fns_from);
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
        writers: HashMap::new(),
        methods: HashMap::new(),
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
    let program = d.program.0;
    for mi in 0..sc.names.len() {
        if mi < from && !(mi == CORE.0 && core_too) {
            continue;
        }
        let e = match d.is_library(ModId(mi)) || mi < from {
            true => None,
            false => errors.as_deref_mut(),
        };
        let mut ambiguous = HashSet::new();
        let v = view(sc, mi, mi == program, e, &mut ambiguous);
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

/// `domain P = (A, B)` over domains declared above it: the tuple of them (R04.2.11).
fn product(d: &Decls, def: &ast::Expr) -> Option<Ty> {
    let ast::ExprKind::Tuple(xs) = &def.kind else {
        return None;
    };
    let &tuple = d.tuples.get(&xs.len())?;
    let parts = (xs.iter())
        .map(|x| match &x.kind {
            ast::ExprKind::Name(n) => d.top_doms.get(n).cloned(),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    Some(Ty::Struct(tuple, parts, Vec::new()))
}

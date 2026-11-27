//! Types as written turned into types: names looked up in a module's view, aliases expanded,
//! a struct's left-off arguments filled from its defaults, and a set for each fn type.
use super::*;
use cpl_next_ir::ids::ItemId;
use cpl_next_ir::unbuilt::Unbuilt;

/// Type arguments written but left off the end take their defaults, which may name the
/// arguments before them (R08.2.5).
pub(super) fn fill_defaults(defaults: Option<&Vec<Option<Ty>>>, a: &mut Vec<Ty>) {
    let Some(ds) = defaults else { return };
    while a.len() < ds.len() {
        let Some(t) = &ds[a.len()] else { break };
        let t = Subst { tys: a, sets: &[] }.ty(t);
        a.push(t);
    }
}

pub(super) struct TyCx<'a> {
    pub(super) names: &'a HashMap<String, Named>,
    /// Every `const`, so a range type's end may name one whose value is a literal.
    pub(super) consts: &'a Layer<LibConst>,
    /// What the program's top level binds with `let`: values, not items, to `type_of`.
    pub(super) values: Arc<HashSet<String>>,
    /// Where the item being converted starts in the source, when known: its spans count from it.
    pub(super) item: Option<ItemId>,
    /// Each binary operator's fn, which `type_of(+)` names.
    pub(super) ops: Arc<Vec<(String, FnId)>>,
    /// How many set parameters each struct has, so a struct named in a type gets its own.
    pub(super) struct_sets: &'a HashMap<StructId, usize>,
    pub(super) struct_defaults: &'a HashMap<StructId, Vec<Option<Ty>>>,
    pub(super) tuples: &'a HashMap<usize, StructId>,
    pub(super) sugar: Sugar,
    pub(super) records: &'a HashMap<Vec<String>, StructId>,
    pub(super) members: &'a HashMap<String, StructId>,
    pub(super) generics: &'a [String],
    /// The signature's own generics, where an alias's expansion names them by position.
    pub(super) outer: &'a [String],
    /// What the generics stand for, when converting an alias's type.
    pub(super) args: Option<&'a [Ty]>,
    pub(super) aliases: &'a Layer<AliasDecl>,
    /// The aliases being expanded, to report one that names itself.
    pub(super) expanding: Vec<usize>,
    /// The types a body declares by name: its domains, and its ranges with run-time ends.
    pub(super) doms: &'a HashMap<String, Ty>,
    /// `t.Pre`: a domain a binding in scope owns, found through that binding, not by spelling.
    pub(super) scoped: &'a dyn Fn(&str) -> Option<Ty>,
    /// When converting a result type: the next result slot.
    pub(super) ret_slots: Option<usize>,
    /// Next implicit set parameter; `None` where a fn type may not appear bare.
    pub(super) next_set: Option<usize>,
}

/// The set for one function type written here: a parameter of the signature, a result slot, or a
/// placeholder the checker replaces in a body.
pub(super) fn slot(cx: &mut TyCx) -> SetTy {
    match (&mut cx.next_set, &mut cx.ret_slots) {
        (Some(n), _) => {
            *n += 1;
            SetTy::Param(*n - 1)
        }
        (None, Some(k)) => {
            *k += 1;
            SetTy::RetSlot(*k - 1)
        }
        (None, None) => SetTy::Var(usize::MAX),
    }
}
pub(super) fn alias(cx: &mut TyCx, id: usize, n: &str, a: Vec<Ty>, errors: &mut Vec<Msg>) -> Ty {
    let decl = &cx.aliases[id];
    if a.len() != decl.generics.len() {
        errors.push(Msg::verdict(
            cpl_diag::codes::WRONG_ARG_COUNT,
            format!(
                "`{n}` takes {} type arguments, found {}",
                decl.generics.len(),
                a.len()
            ),
        ));
        return Ty::Error;
    }
    // An alias defined by itself is E0323 where it is declared (`alias_cycle`).
    if cx.expanding.contains(&id) {
        return Ty::Error;
    }
    let mut expanding = cx.expanding.clone();
    expanding.push(id);
    let none = HashMap::new();
    let mut inner = TyCx {
        names: &decl.names,
        consts: cx.consts,
        values: cx.values.clone(),
        item: None,
        ops: cx.ops.clone(),
        struct_sets: cx.struct_sets,
        struct_defaults: cx.struct_defaults,
        tuples: cx.tuples,
        sugar: cx.sugar,
        records: cx.records,
        members: cx.members,
        generics: &decl.generics,
        outer: cx.outer,
        args: Some(&a),
        aliases: cx.aliases,
        expanding,
        doms: if decl.program { cx.doms } else { &none },
        scoped: if decl.program { cx.scoped } else { &unscoped },
        ret_slots: cx.ret_slots,
        next_set: cx.next_set,
    };
    let t = conv(&mut inner, &decl.ty, errors);
    (cx.ret_slots, cx.next_set) = (inner.ret_slots, inner.next_set);
    t
}
/// Whether some type fits both impl headers.
/// A decorator another tool reads (`cpl lint`, `cpl stress`, a diagnostic's suggestions): on a
/// library item it changes nothing a run or a check of a program does (R06.4.11). A program's
/// own is refused until the slice checks it.
pub(super) fn read_elsewhere(name: &str) -> bool {
    matches!(name, "lint" | "stress" | "on_unimplemented")
}

/// `Seq<T>` written where a trait is wanted.
pub(super) fn trait_ref(cx: &mut TyCx, t: &ast::Type, errors: &mut Vec<Msg>) -> Option<TraitRef> {
    // `(u: V, w: W, ..)`: the core trait of that row, over the field types (R08.4.1).
    if let TypeKind::Row(fs) = &t.kind {
        let names: Vec<&str> = fs.iter().map(|(n, _)| n.text.as_str()).collect();
        let Some(Named::Trait(tr)) = cx.names.get(&crate::records::row_name(&names)).copied()
        else {
            errors.push(Unbuilt::BoundNotNamed.into());
            return None;
        };
        let args = fs.iter().map(|(_, a)| conv(cx, a, errors)).collect();
        return Some(TraitRef { tr, args });
    }
    let TypeKind::Named { name, args } = &t.kind else {
        errors.push(Unbuilt::BoundNotNamed.into());
        return None;
    };
    let Some(Named::Trait(tr)) = cx.names.get(&name.text).copied() else {
        errors.push(Msg::verdict(
            cpl_diag::codes::UNKNOWN_TYPE,
            format!("unknown trait `{}`", name.text),
        ));
        return None;
    };
    let args = args.iter().map(|a| conv(cx, a, errors)).collect();
    Some(TraitRef { tr, args })
}

/// The single capitals in an impl header that name nothing: its undeclared type parameters (R02.10.3).
/// `@get`: a member read like a field (R02.6.6).
pub(crate) const GETTER: &str = "get";
/// A range type with an end naming a value the program's top level binds, written out, and that
/// value: a struct field's layout is fixed before input is read, so no field is bounded by one
/// (R03.1.3).
pub(super) fn input_bound(t: &ast::Type, values: &HashSet<String>) -> Option<(String, String)> {
    struct Named<'a>(&'a HashSet<String>, Option<String>);
    impl cpl_syntax::visit::Visitor for Named<'_> {
        fn expr(&mut self, e: &ast::Expr) {
            match &e.kind {
                ast::ExprKind::Name(n) if self.1.is_none() && self.0.contains(n) => {
                    self.1 = Some(n.clone())
                }
                _ => {}
            }
        }
    }
    match &t.kind {
        TypeKind::Range { lo, hi, inclusive } => {
            let mut v = Named(values, None);
            for e in [lo, hi].into_iter().flatten() {
                cpl_syntax::visit::walk_expr(e, &mut v);
            }
            let end = |e: &Option<Box<ast::Expr>>| match e.as_deref().map(|e| &e.kind) {
                Some(ast::ExprKind::Int(k)) => k.to_string(),
                Some(ast::ExprKind::Name(n)) => n.clone(),
                Some(_) => "…".to_string(),
                None => String::new(),
            };
            let dots = if *inclusive { "..=" } else { ".." };
            v.1.map(|n| (format!("{}{dots}{}", end(lo), end(hi)), n))
        }
        TypeKind::Tuple(ts) => ts.iter().find_map(|t| input_bound(t, values)),
        TypeKind::Optional(t) => input_bound(t, values),
        _ => None,
    }
}

pub(crate) fn unnamed(t: &ast::Type, names: &HashMap<String, Named>, out: &mut Vec<String>) {
    let mut all = Vec::new();
    names_in(t, &mut all);
    for n in all {
        let capital = n.len() == 1 && n.chars().all(|c| c.is_ascii_uppercase());
        if capital && !names.contains_key(&n) && !out.contains(&n) {
            out.push(n);
        }
    }
}

/// Every bare name (no type arguments) a type writes, in order.
pub(crate) fn names_in(t: &ast::Type, out: &mut Vec<String>) {
    match &t.kind {
        TypeKind::Named { name, args } => {
            if args.is_empty() {
                out.push(name.text.clone());
            }
            args.iter().for_each(|a| names_in(a, out));
        }
        TypeKind::Optional(e) | TypeKind::Mut(e) | TypeKind::Yields(e) => names_in(e, out),
        TypeKind::Tuple(ts) => ts.iter().for_each(|e| names_in(e, out)),
        TypeKind::Array { index, elem } => {
            index.iter().chain([&**elem]).for_each(|e| names_in(e, out))
        }
        TypeKind::Fn { params, ret } => {
            params.iter().chain([&**ret]).for_each(|e| names_in(e, out))
        }
        _ => {}
    }
}

/// A type written among `Self, A, B..` re-read among `A, B..`.
pub(super) fn shift_down(t: &Ty) -> Ty {
    let tys: Vec<Ty> = std::iter::once(Ty::Error)
        .chain((0..64).map(Ty::Param))
        .collect();
    Subst {
        tys: &tys,
        sets: &[],
    }
    .ty(t)
}

/// A core type written by its name.
pub fn prim_type(n: &str) -> Option<Ty> {
    Some(match n {
        "Int" => Ty::Int,
        "Bool" => Ty::Bool,
        "Char" => Ty::Char,
        "U64" => Ty::U64,
        "F64" => Ty::F64,
        _ => return None,
    })
}

pub(super) fn conv(cx: &mut TyCx, t: &ast::Type, errors: &mut Vec<Msg>) -> Ty {
    match &t.kind {
        TypeKind::Named { name, args } => {
            let n = name.text.as_str();
            let mut a: Vec<Ty> = args.iter().map(|x| conv(cx, x, errors)).collect();
            // `D.m`: the `@member(m)` struct at `D`, as `D.cuts` is `Cut<D>`.
            if let Some((base, m)) = n.rsplit_once('.') {
                if let Some(&id) = cx.members.get(m) {
                    let named = TypeKind::Named {
                        name: ast::Name {
                            text: base.to_string(),
                            span: name.span,
                        },
                        args: Vec::new(),
                    };
                    let base = conv(
                        cx,
                        &ast::Type {
                            kind: named,
                            span: t.span,
                        },
                        errors,
                    );
                    let k = cx.struct_sets.get(&id).copied().unwrap_or(0);
                    let sets = (0..k).map(|_| slot(cx)).collect();
                    return Ty::Struct(id, vec![base], sets);
                }
            }
            if let Some(i) = cx.generics.iter().position(|g| g == n) {
                return match cx.args {
                    Some(args) => args[i].clone(),
                    None => Ty::Param(i),
                };
            }
            if let Some(Named::Alias(id)) = cx.names.get(n) {
                return alias(cx, id.0, n, a, errors);
            }
            if let Some(t) = cx.doms.get(n).cloned().or_else(|| (cx.scoped)(n)) {
                return t;
            }
            if let Some(t) = prim_type(n) {
                return t;
            }
            match n {
                "Raw" if a.len() == 1 => Ty::Raw(Box::new(a.remove(0))),
                _ => match struct_named(cx.names, n) {
                    // Set arguments of a struct named in a signature are fresh parameters.
                    Some(d) => {
                        fill_defaults(cx.struct_defaults.get(&d), &mut a);
                        let k = cx.struct_sets.get(&d).copied().unwrap_or(0);
                        let sets = (0..k).map(|_| slot(cx)).collect();
                        Ty::Struct(d, a, sets)
                    }
                    None => {
                        errors.push(Msg::verdict(
                            cpl_diag::codes::UNKNOWN_TYPE,
                            format!("unknown type `{n}`"),
                        ));
                        Ty::Error
                    }
                },
            }
        }
        TypeKind::Tuple(ts) if ts.is_empty() => Ty::Unit,
        // `(A, B)` is core's `@tuple` struct with two fields.
        TypeKind::Tuple(ts) if cx.tuples.contains_key(&ts.len()) => {
            let d = cx.tuples[&ts.len()];
            let a = ts.iter().map(|x| conv(cx, x, errors)).collect();
            let k = cx.struct_sets.get(&d).copied().unwrap_or(0);
            let sets = (0..k).map(|_| slot(cx)).collect();
            Ty::Struct(d, a, sets)
        }
        // `(lo: A, hi: B)` is the record struct of those field names (R02.5.2).
        TypeKind::Record(fs) if cx.records.contains_key(&field_names(fs)) => {
            let d = cx.records[&field_names(fs)];
            // `domain C` in a result is the fresh domain the fn names `C` (R02.9.3).
            let a = (fs.iter())
                .map(|(n, x)| match x.kind {
                    TypeKind::NewDomain => cx.doms.get(&n.text).cloned().unwrap_or(Ty::Error),
                    _ => conv(cx, x, errors),
                })
                .collect();
            Ty::Struct(d, a, Vec::new())
        }
        TypeKind::RowWithout {
            prefix,
            base,
            removed,
        } => {
            let form = (
                prefix.iter().map(|(n, _)| n.text.clone()).collect(),
                removed.iter().map(|n| n.text.clone()).collect(),
            );
            let base = conv(cx, base, errors);
            without(cx, &form, prefix, base, t.span, errors)
        }
        // `?T` is std's `@optional` struct.
        TypeKind::Optional(inner) if cx.sugar.optional.is_some() => {
            let d = cx.sugar.optional.unwrap_or(StructId(0));
            Ty::Struct(d, vec![conv(cx, inner, errors)], Vec::new())
        }
        // `[D]T` is std's `@array` struct `Array<D, T>`.
        // `0..=CAP` with `const CAP = 2_000_000_000`: a const written as a literal is that literal.
        TypeKind::Range { lo, hi, inclusive } => match static_range_by(lo, hi, *inclusive, |e| {
            match &e.kind {
                ast::ExprKind::Name(n) => match cx.names.get(n.as_str()) {
                    Some(Named::Const(c)) => cx.consts[c.0].init.int_literal(),
                    _ => None,
                },
                _ => e.int_literal(),
            }
        }) {
            Some((lo, hi)) => Ty::Range(lo, hi),
            // `0..=V.count`: its end is the domain's, where a struct's arguments name it.
            None if count_end(lo, hi, *inclusive).is_some_and(|(_, d, ..)| is_type_name(cx, d)) => {
                let (lo, d, m, k) = count_end(lo, hi, *inclusive).unwrap_or_default();
                let d = conv(cx, &ast::Type::named(d, t.span), errors);
                Ty::RangeTo(Box::new(d), m.to_string(), lo, k)
            }
            None => {
                errors.push(Unbuilt::RangeTypeOutsideBody.into());
                Ty::Error
            }
        },
        // `[0..2]T`: an axis over a constant range is keyed by that range type (R04.3.4).
        TypeKind::Array { index, .. }
            if index.len() == 1
                && matches!(index[0].kind, TypeKind::Range { .. })
                && const_axis(&index[0]).is_none() =>
        {
            errors.push(Unbuilt::RangeIndexedArray.into());
            Ty::Error
        }
        // `[R, C]T` is `[(R, C)]T`: one array keyed by the product domain (R04.2.11).
        TypeKind::Array { index, elem } if !index.is_empty() => {
            let i = match &index[..] {
                [i] => conv(cx, i, errors),
                _ => conv(
                    cx,
                    &ast::Type {
                        kind: TypeKind::Tuple(index.clone()),
                        span: t.span,
                    },
                    errors,
                ),
            };
            let e = conv(cx, elem, errors);
            match cx.sugar.array {
                Some(array) => Ty::Struct(array, vec![i, e], Vec::new()),
                None => {
                    errors.push(Msg::internal("`[D]T` needs std's `@array` struct"));
                    Ty::Error
                }
            }
        }
        TypeKind::Fn { params, ret } => {
            let ps = (params.iter())
                .map(|x| match &x.kind {
                    TypeKind::Mut(m) => Ty::Mut(Box::new(conv(cx, m, errors))),
                    _ => conv(cx, x, errors),
                })
                .collect();
            let r = conv(cx, ret, errors);
            let set = slot(cx);
            Ty::Fn(ps, Box::new(r), set)
        }
        // `type_of(f)`: the fn item's own type, one for a generic fn (R08.1.10).
        TypeKind::ItemOf(e) => {
            let item = cx.item.clone();
            let at = |text: Msg| match &item {
                Some(i) => Msg::at(i.at(e.span), text),
                None => text,
            };
            let key = e.fn_item_key();
            match key.as_deref().map(|k| (k, cx.names.get(k))) {
                Some((_, Some(Named::Fn(f)))) => Ty::Clo(Box::new(CloRef::Item(FnRef {
                    decl: *f,
                    targs: Vec::new(),
                    sets: Vec::new(),
                    ctx: 0,
                }))),
                Some((k, None))
                    if matches!(e.kind, ast::ExprKind::Name(_)) && !cx.values.contains(k) =>
                {
                    let code = cpl_diag::codes::UNRESOLVED_NAME;
                    errors.push(at(Msg::verdict(code, format!("unknown name `{k}`"))));
                    Ty::Error
                }
                Some((k, None)) if !matches!(e.kind, ast::ExprKind::Name(_)) => {
                    match cx.ops.iter().find(|(o, _)| o == k) {
                        Some((_, f)) => Ty::Clo(Box::new(CloRef::Item(FnRef {
                            decl: *f,
                            targs: Vec::new(),
                            sets: Vec::new(),
                            ctx: 0,
                        }))),
                        None => {
                            let code = cpl_diag::codes::NOT_A_FN_ITEM;
                            errors.push(at(Msg::verdict(
                                code,
                                format!("no `@operator` trait declares `{k}`, so it is no fn item"),
                            )));
                            Ty::Error
                        }
                    }
                }
                _ => {
                    let code = cpl_diag::codes::NOT_A_FN_ITEM;
                    errors.push(at(Msg::verdict(
                        code,
                        "`type_of` takes a `fn` declared by name, not this value",
                    )));
                    Ty::Error
                }
            }
        }
        // The parser has said what is wrong with it.
        TypeKind::Error => Ty::Error,
        TypeKind::DomainBinder(n) => cx.doms.get(&n.text).cloned().unwrap_or(Ty::Error),
        // A result's `_`, filled in once its struct's domain parameters are known (R02.9.3).
        TypeKind::Value(_) if cx.ret_slots.is_some() && is_hole(t) => Ty::Error,
        other => {
            errors.push(
                Unbuilt::Type {
                    kind: crate::kind_name(other),
                }
                .into(),
            );
            Ty::Error
        }
    }
}

/// Converts a type written in a body: fn types get a placeholder set the checker replaces.
pub fn body_type(
    d: &Decls,
    m: ModId,
    generics: &[String],
    doms: &HashMap<String, Ty>,
    scoped: &dyn Fn(&str) -> Option<Ty>,
    t: &ast::Type,
    errors: &mut Vec<Msg>,
) -> Ty {
    conv(
        &mut TyCx {
            names: &d.views[m.0],
            consts: &d.consts,
            values: d.top_values.clone(),
            item: None,
            ops: d.operator_fns(),
            struct_sets: &d.struct_sets,
            struct_defaults: &d.struct_defaults,
            tuples: &d.tuples,
            sugar: d.sugar,
            records: &d.records,
            members: &d.member_types,
            generics,
            outer: generics,
            args: None,
            aliases: &d.aliases,
            expanding: Vec::new(),
            doms,
            scoped,
            ret_slots: None,
            next_set: None,
        },
        t,
        errors,
    )
}

/// No binding is in scope where decls reads a type.
pub(super) fn unscoped(_: &str) -> Option<Ty> {
    None
}

/// `(to: X, ..B without (u, v))`: of a record, the record of its other fields after the added
/// ones; of a row-bound parameter `R`, the associated type its form's trait names, a parameter of
/// the signature (R08.4.1).
fn without(
    cx: &mut TyCx,
    form: &crate::records::Form,
    prefix: &[(ast::Name, ast::Type)],
    base: Ty,
    span: cpl_diag::Span,
    errors: &mut Vec<Msg>,
) -> Ty {
    let item = cx.item.clone();
    let missing = |what: &str| {
        let m = Msg::verdict(
            cpl_diag::codes::UNKNOWN_FIELD,
            format!("`without` drops a field {what} lacks (R08.4.4)"),
        );
        match &item {
            Some(i) => Msg::at(i.at(span), m),
            None => m,
        }
    };
    match base {
        Ty::Struct(s, args, _) => {
            let Some(fs) = (cx.records.iter())
                .find(|(_, id)| **id == s)
                .map(|(fs, _)| fs.clone())
            else {
                errors.push(missing("a type that is not a record"));
                return Ty::Error;
            };
            let Some(made) = crate::records::without_shape(form, &fs) else {
                errors.push(missing("the record"));
                return Ty::Error;
            };
            let Some(&id) = cx.records.get(&made) else {
                errors.push(
                    Unbuilt::Type {
                        kind: "RowWithout".to_string(),
                    }
                    .into(),
                );
                return Ty::Error;
            };
            let a = (made.iter())
                .map(|f| match prefix.iter().find(|(n, _)| n.text == *f) {
                    Some((_, t)) => conv(cx, t, errors),
                    None => fs
                        .iter()
                        .position(|x| x == f)
                        .map_or(Ty::Error, |k| args[k].clone()),
                })
                .collect();
            Ty::Struct(id, a, Vec::new())
        }
        Ty::Param(i) => {
            let rest = format!("{}.{}", cx.outer[i], crate::records::without_member(form));
            match cx.outer.iter().position(|g| *g == rest) {
                Some(k) => Ty::Param(k),
                None => {
                    errors.push(missing("the row bound"));
                    Ty::Error
                }
            }
        }
        Ty::Error => Ty::Error,
        _ => {
            errors.push(missing("a type that is not a record"));
            Ty::Error
        }
    }
}

/// `_` as a type argument, `Rooted<V, _>`.
fn is_hole(t: &ast::Type) -> bool {
    matches!(&t.kind, TypeKind::Value(e) if matches!(&e.kind, ast::ExprKind::Name(n) if n == "_"))
}

/// `t` leaves some struct's parameter as `_`, directly or in an optional.
pub(super) fn has_hole(t: &ast::Type) -> bool {
    match &t.kind {
        TypeKind::Named { args, .. } => args.iter().any(|a| is_hole(a) || has_hole(a)),
        TypeKind::Optional(e) => has_hole(e),
        TypeKind::Tuple(ts) => ts.iter().any(has_hole),
        TypeKind::Record(fs) => fs.iter().any(|(_, t)| has_hole(t)),
        _ => false,
    }
}

/// Whether `n` names a type where `cx` converts one: a generic or a declared name.
fn is_type_name(cx: &TyCx, n: &str) -> bool {
    cx.generics.iter().any(|g| g == n) || cx.names.contains_key(n)
}

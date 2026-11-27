//! Types as written turned into types: names looked up in a module's view, aliases expanded,
//! a struct's left-off arguments filled from its defaults, and a set for each fn type.
use super::*;

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
    /// How many set parameters each struct has, so a struct named in a type gets its own.
    pub(super) struct_sets: &'a HashMap<StructId, usize>,
    pub(super) struct_defaults: &'a HashMap<StructId, Vec<Option<Ty>>>,
    pub(super) tuples: &'a HashMap<usize, StructId>,
    pub(super) generics: &'a [String],
    /// What the generics stand for, when converting an alias's type.
    pub(super) args: Option<&'a [Ty]>,
    pub(super) aliases: &'a [AliasDecl],
    /// The aliases being expanded, to report one that names itself.
    pub(super) expanding: Vec<usize>,
    /// The types a body declares by name: its domains, and its ranges with run-time ends.
    pub(super) doms: &'a HashMap<String, Ty>,
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
        errors.push(
            format!(
                "{}: `{n}` takes {} type arguments, found {}",
                cpl_diag::codes::WRONG_ARG_COUNT.0,
                decl.generics.len(),
                a.len()
            )
            .into(),
        );
        return Ty::Error;
    }
    // An alias defined by itself is E0323 where it is declared (`alias_cycle`).
    if cx.expanding.contains(&id) {
        return Ty::Error;
    }
    let mut expanding = cx.expanding.clone();
    expanding.push(id);
    let mut inner = TyCx {
        names: &decl.names,
        struct_sets: cx.struct_sets,
        struct_defaults: cx.struct_defaults,
        tuples: cx.tuples,
        generics: &decl.generics,
        args: Some(&a),
        aliases: cx.aliases,
        expanding,
        doms: &HashMap::new(),
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
    let TypeKind::Named { name, args } = &t.kind else {
        errors.push("slice: a bound that is not a named trait".into());
        return None;
    };
    let Some(Named::Trait(tr)) = cx.names.get(&name.text).copied() else {
        errors.push(
            format!(
                "{}: unknown trait `{}`",
                cpl_diag::codes::UNKNOWN_TYPE.0,
                name.text
            )
            .into(),
        );
        return None;
    };
    let args = args.iter().map(|a| conv(cx, a, errors)).collect();
    Some(TraitRef { tr, args })
}

/// The single capitals in an impl header that name nothing: its undeclared type parameters (R02.10.3).
/// `@get`: a member read like a field (R02.6.6).
pub(crate) const GETTER: &str = "get";

pub(crate) fn unnamed(t: &ast::Type, names: &HashMap<String, Named>, out: &mut Vec<String>) {
    if let TypeKind::Named { name, args } = &t.kind {
        let n = name.text.as_str();
        let capital = n.len() == 1 && n.chars().all(|c| c.is_ascii_uppercase());
        if args.is_empty() && capital && !names.contains_key(n) && !out.iter().any(|g| g == n) {
            out.push(n.to_string());
        }
        args.iter().for_each(|a| unnamed(a, names, out));
        return;
    }
    match &t.kind {
        TypeKind::Optional(e) | TypeKind::Mut(e) | TypeKind::Yields(e) => unnamed(e, names, out),
        TypeKind::Tuple(ts) => ts.iter().for_each(|e| unnamed(e, names, out)),
        TypeKind::Array { index, elem } => index
            .iter()
            .chain([&**elem])
            .for_each(|e| unnamed(e, names, out)),
        TypeKind::Fn { params, ret } => params
            .iter()
            .chain([&**ret])
            .for_each(|e| unnamed(e, names, out)),
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
            if let Some(i) = cx.generics.iter().position(|g| g == n) {
                return match cx.args {
                    Some(args) => args[i].clone(),
                    None => Ty::Param(i),
                };
            }
            if let Some(Named::Alias(id)) = cx.names.get(n) {
                return alias(cx, id.0, n, a, errors);
            }
            if let Some(t) = cx.doms.get(n) {
                return t.clone();
            }
            if let Some(t) = prim_type(n) {
                return t;
            }
            match n {
                "Str" if struct_named(cx.names, "Str").is_none() => {
                    errors.push("internal: `Str` needs the library's `Str`".into());
                    Ty::Error
                }
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
                        errors.push(
                            format!("{}: unknown type `{n}`", cpl_diag::codes::UNKNOWN_TYPE.0)
                                .into(),
                        );
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
        // `(lo: A, hi: B)` is the core record struct `(lo:, hi:)<A, B>` (R02.5.2).
        TypeKind::Record(fs)
            if struct_named(cx.names, &crate::records::name(&field_names(fs))).is_some() =>
        {
            let named = ast::Type {
                kind: TypeKind::Named {
                    name: ast::Name {
                        text: crate::records::name(&field_names(fs)),
                        span: t.span,
                    },
                    args: fs.iter().map(|(_, t)| t.clone()).collect(),
                },
                span: t.span,
            };
            conv(cx, &named, errors)
        }
        // `?T` is the library's `Optional<T>`.
        TypeKind::Optional(inner) if struct_named(cx.names, "Optional").is_some() => {
            let named = ast::Type {
                kind: TypeKind::Named {
                    name: ast::Name {
                        text: "Optional".to_string(),
                        span: t.span,
                    },
                    args: vec![(**inner).clone()],
                },
                span: t.span,
            };
            conv(cx, &named, errors)
        }
        // `[D]T` is the library's `Array<D, T>`.
        TypeKind::Range { lo, hi, inclusive } => match static_range(lo, hi, *inclusive) {
            Some((lo, hi)) => Ty::Range(lo, hi),
            None => {
                errors.push(
                    "slice: a range type whose ends are not constants, outside a body".into(),
                );
                Ty::Error
            }
        },
        TypeKind::Array { index, .. }
            if index.len() == 1 && matches!(index[0].kind, TypeKind::Range { .. }) =>
        {
            errors.push("slice: an array indexed by a range outside a `let` or `read`".into());
            Ty::Error
        }
        TypeKind::Array { index, elem } if index.len() == 1 => {
            let i = conv(cx, &index[0], errors);
            let e = conv(cx, elem, errors);
            match struct_named(cx.names, "Array") {
                Some(array) => Ty::Struct(array, vec![i, e], Vec::new()),
                None => {
                    errors.push("internal: `[D]T` needs the library's `Array`".into());
                    Ty::Error
                }
            }
        }
        TypeKind::Fn { params, ret } => {
            let ps = params.iter().map(|x| conv(cx, x, errors)).collect();
            let r = conv(cx, ret, errors);
            let set = slot(cx);
            Ty::Fn(ps, Box::new(r), set)
        }
        // The parser has said what is wrong with it.
        TypeKind::Error => Ty::Error,
        other => {
            errors.push(format!("slice: type {}", crate::kind_name(other)).into());
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
    t: &ast::Type,
    errors: &mut Vec<Msg>,
) -> Ty {
    conv(
        &mut TyCx {
            names: &d.views[m.0],
            struct_sets: &d.struct_sets,
            struct_defaults: &d.struct_defaults,
            tuples: &d.tuples,
            generics,
            args: None,
            aliases: &d.aliases,
            expanding: Vec::new(),
            doms,
            ret_slots: None,
            next_set: None,
        },
        t,
        errors,
    )
}

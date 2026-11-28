//! Types as written turned into types: names looked up in a module's view, aliases expanded,
//! a struct's left-off arguments filled from its defaults, and a set for each fn type.
use super::*;
use crate::views::{Scope, Signatures, TypeTables};
use cpl_next_ir::ids::ItemId;
use cpl_next_ir::unbuilt::Unbuilt;
use cpl_syntax::decorators::Mark;

/// Type arguments written but left off the end take their defaults, which may name the
/// arguments before them (R08.2.5).
/// The struct of `s`'s name with `n` type parameters, else `s` (R02.11.4).
pub(crate) fn by_arity(
    arity: &HashMap<StructId, Vec<(StructId, usize)>>,
    s: StructId,
    n: usize,
) -> StructId {
    (arity.get(&s).into_iter().flatten())
        .find(|(_, k)| *k == n)
        .map_or(s, |(o, _)| *o)
}

pub(super) fn fill_defaults(defaults: Option<&Vec<Option<Ty>>>, a: &mut Vec<Ty>) {
    let Some(ds) = defaults else { return };
    while a.len() < ds.len() {
        let Some(t) = &ds[a.len()] else { break };
        let t = Subst { tys: a, sets: &[] }.ty(t);
        a.push(t);
    }
}

pub(super) struct TyCx<'a> {
    /// What a name means where the type is written.
    pub(super) names: &'a dyn Fn(&str) -> Option<Named>,
    /// The signatures' tables, each read by key.
    pub(super) env: &'a dyn TypeTables,
    /// Where the item being converted starts in the source, when known: its spans count from it.
    pub(super) item: Option<ItemId>,
    pub(super) generics: &'a [String],
    /// The signature's own generics, where an alias's expansion names them by position.
    pub(super) outer: &'a [String],
    /// What the generics stand for, when converting an alias's type.
    pub(super) args: Option<&'a [Ty]>,
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
    let env = cx.env;
    let decl = env.alias(AliasId(id));
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
        names: &|n: &str| env.sym(n).and_then(|s| decl.names.get(&s)).copied(),
        env,
        item: None,
        generics: &decl.generics,
        outer: cx.outer,
        args: Some(&a),
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
pub(super) fn read_elsewhere(m: Option<Mark>) -> bool {
    matches!(m, Some(Mark::Lint | Mark::Stress | Mark::OnUnimplemented))
}

/// Whether `x` is `@stress`, by grammar's decorator table.
pub(super) fn is_stress(x: &ast::Decorator) -> bool {
    use cpl_next_syntax::grammar::Decorator;
    !x.directive && matches!(Decorator::of(&x.name.text), Some(Decorator::Stress))
}

/// What is wrong with a program's `@stress`, if anything: it names one bare kind above a type
/// alias, and `cpl run` reads nothing else of it (R12.13.3).
pub(super) fn stress_misuse(x: &ast::Decorator, alias: bool) -> Option<Msg> {
    use cpl_next_syntax::grammar::StressKind;
    let kind = match x.args.as_slice() {
        [a] => match (&a.name, &a.value.kind) {
            (None, ast::ExprKind::Name(w)) => StressKind::of(w),
            _ => None,
        },
        _ => None,
    };
    let (text, help) = match (alias, kind) {
        (true, Some(_)) => return None,
        (false, _) => (
            "`@stress` decorates a type alias".to_string(),
            "put it on a `type` alias, as in `@stress(tree) type T = [0..n - 1](u: V, v: V)`",
        ),
        (true, None) => {
            let kinds = StressKind::ROWS.map(|r| format!("`{}`", r.1)).join(", ");
            (
                format!("`@stress` takes one of {kinds}"),
                "write one bare kind, as in `@stress(tree)`",
            )
        }
    };
    Some(Msg {
        help: Some(help.to_string()),
        ..Msg::verdict(cpl_diag::codes::BAD_STRESS, text)
    })
}

/// `Seq<T>` written where a trait is wanted.
pub(super) fn trait_ref(cx: &mut TyCx, t: &ast::Type, errors: &mut Vec<Msg>) -> Option<TraitRef> {
    // `(u: V, w: W, ..)`: the core trait of that row, over the field types (R08.4.1).
    if let TypeKind::Row(fs) = &t.kind {
        let names: Vec<&str> = fs.iter().map(|(n, _)| n.text.as_str()).collect();
        let Some(Named::Trait(tr)) = (cx.names)(&crate::records::row_name(&names)) else {
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
    let Some(Named::Trait(tr)) = (cx.names)(&name.text) else {
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
/// A range type with an end naming a value the program's top level binds, written out, and that
/// value: a struct field's layout is fixed before input is read, so no field is bounded by one
/// (R03.1.3).
pub(super) fn input_bound(t: &ast::Type, bound: &dyn Fn(&str) -> bool) -> Option<(String, String)> {
    struct Named<'a>(&'a dyn Fn(&str) -> bool, Option<String>);
    impl cpl_syntax::visit::Visitor for Named<'_> {
        fn expr(&mut self, e: &ast::Expr) {
            match &e.kind {
                ast::ExprKind::Name(n) if self.1.is_none() && (self.0)(n) => {
                    self.1 = Some(n.clone())
                }
                _ => {}
            }
        }
    }
    match &t.kind {
        TypeKind::Range { lo, hi, inclusive } => {
            let mut v = Named(bound, None);
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
        TypeKind::Tuple(ts) => ts.iter().find_map(|t| input_bound(t, bound)),
        TypeKind::Optional(t) => input_bound(t, bound),
        _ => None,
    }
}

pub(crate) fn unnamed(t: &ast::Type, names: &dyn Fn(&str) -> Option<Named>, out: &mut Vec<String>) {
    let mut all = Vec::new();
    names_in(t, &mut all);
    for n in all {
        let capital = n.len() == 1 && n.chars().all(|c| c.is_ascii_uppercase());
        if capital && names(&n).is_none() && !out.contains(&n) {
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
    cpl_contract::Prim::named(n).map(Ty::of_prim)
}

pub(super) fn conv(cx: &mut TyCx, t: &ast::Type, errors: &mut Vec<Msg>) -> Ty {
    match &t.kind {
        TypeKind::Named { name, args } => {
            let n = name.text.as_str();
            let mut a: Vec<Ty> = args.iter().map(|x| conv(cx, x, errors)).collect();
            // `D.m`: the `@member(m)` struct at `D`, as `D.cuts` is `Cut<D>`.
            if let Some((base, m)) = n.rsplit_once('.') {
                if let Some(id) = cx.env.member(m) {
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
                    let k = cx.env.n_sets(id);
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
            if let Some(Named::Alias(id)) = (cx.names)(n) {
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
                _ => match (cx.names)(n).and_then(|x| match x {
                    Named::Struct(s) => Some(s),
                    _ => None,
                }) {
                    // Set arguments of a struct named in a signature are fresh parameters.
                    Some(d) => {
                        let d = cx.env.by_arity(d, a.len());
                        fill_defaults(cx.env.struct_defaults(d), &mut a);
                        let k = cx.env.n_sets(d);
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
        // `(A, B)` is the `@role(tuple)` struct with two fields.
        TypeKind::Tuple(ts) if cx.env.tuple(ts.len()).is_some() => {
            let d = cx.env.tuple(ts.len()).unwrap_or(StructId(0));
            let a = ts.iter().map(|x| conv(cx, x, errors)).collect();
            let k = cx.env.n_sets(d);
            let sets = (0..k).map(|_| slot(cx)).collect();
            Ty::Struct(d, a, sets)
        }
        // `(lo: A, hi: B)` is the record struct of those field names (R02.5.2).
        TypeKind::Record(fs) if cx.env.record(&field_names(fs)).is_some() => {
            let d = cx.env.record(&field_names(fs)).unwrap_or(StructId(0));
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
        // `?T` is the struct filling the `option` role.
        TypeKind::Optional(inner) if cx.env.roles().option.is_some() => {
            let d = cx.env.roles().option.unwrap_or(StructId(0));
            Ty::Struct(d, vec![conv(cx, inner, errors)], Vec::new())
        }
        // `[D]T` is the `@role(array)` struct `Array<D, T>`.
        // `0..=CAP` with `const CAP = 2_000_000_000`: a const written as a literal is that literal.
        TypeKind::Range { lo, hi, inclusive } => match static_range_by(lo, hi, *inclusive, |e| {
            match &e.kind {
                ast::ExprKind::Name(n) => match (cx.names)(n.as_str()) {
                    Some(Named::Const(c)) => cx.env.konst(c).init.int_literal(),
                    _ => None,
                },
                _ => e.int_literal(),
            }
        }) {
            Some((lo, hi)) => Ty::Range(lo, hi),
            // `0..=V.count`: its end is the domain's, where a struct's arguments name it.
            None if count_end(lo, hi, *inclusive)
                .is_some_and(|(_, d, m, _)| is_type_name(cx, d) && cx.env.getter(m).is_some()) =>
            {
                let (lo, d, m, k) = count_end(lo, hi, *inclusive).unwrap_or_default();
                let m = cx.env.getter(m).unwrap_or(FnId(0));
                let d = conv(cx, &ast::Type::named(d, t.span), errors);
                Ty::RangeTo(Box::new(d), m, lo, k)
            }
            None => {
                errors.push(match input_bound(t, &|n| cx.env.top_var(n)) {
                    Some((_, n)) => {
                        let m = Msg {
                            help: Some(format!("freeze it first: `let {n}` in place of `var {n}`")),
                            ..Msg::verdict(
                                cpl_diag::codes::RANGE_END_VAR,
                                format!("a range type can't end at the top-level `var` `{n}`: it may change while the type is in use"),
                            )
                        };
                        match &cx.item {
                            Some(i) => Msg::at(i.at(t.span), m),
                            None => m,
                        }
                    }
                    None => Unbuilt::RangeTypeOutsideBody.into(),
                });
                Ty::Error
            }
        },
        // `[0..2]T`: an axis over a constant range is keyed by that range type (R04.3.4).
        TypeKind::Array { index, .. }
            if index.len() == 1
                && matches!(index[0].kind, TypeKind::Range { .. })
                && const_axis(&index[0]).is_none()
                && !count_axis(cx, &index[0]) =>
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
            match cx.env.sugar().array {
                Some(array) => Ty::Struct(array, vec![i, e], Vec::new()),
                None => {
                    errors.push(Msg::internal("`[D]T` needs the `@role(array)` struct"));
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
            match key.as_deref().map(|k| (k, (cx.names)(k))) {
                Some((_, Some(Named::Fn(f)))) => Ty::Clo(Box::new(CloRef::Item(FnRef {
                    decl: f,
                    targs: Vec::new(),
                    sets: Vec::new(),
                    ctx: 0,
                }))),
                Some((k, None))
                    if matches!(e.kind, ast::ExprKind::Name(_)) && !cx.env.top_value(k) =>
                {
                    let code = cpl_diag::codes::UNRESOLVED_NAME;
                    errors.push(at(Msg::verdict(code, format!("unknown name `{k}`"))));
                    Ty::Error
                }
                Some((k, None)) if !matches!(e.kind, ast::ExprKind::Name(_)) => {
                    match cx.env.operator(k) {
                        Some(f) => Ty::Clo(Box::new(CloRef::Item(FnRef {
                            decl: f,
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

/// Converts a type written in a body: fn types get a placeholder set the checker replaces. It
/// reads the module's scope and the signatures, and gives the type with what is wrong in it.
pub fn body_type(
    sigs: &dyn Signatures,
    scope: &dyn Scope,
    m: ModId,
    generics: &[String],
    doms: &HashMap<String, Ty>,
    scoped: &dyn Fn(&str) -> Option<Ty>,
    t: &ast::Type,
) -> (Ty, Vec<Msg>) {
    let mut errors = Vec::new();
    let mut cx = TyCx {
        names: &|n: &str| scope.named(m, n),
        env: sigs,
        item: None,
        generics,
        outer: generics,
        args: None,
        expanding: Vec::new(),
        doms,
        scoped,
        ret_slots: None,
        next_set: None,
    };
    let t = conv(&mut cx, t, &mut errors);
    (t, errors)
}

/// The tables as decls has built them so far, for the types it converts while building the rest.
pub(super) struct Tables {
    aliases: Layer<AliasDecl>,
    consts: Layer<LibConst>,
    pub(super) values: Arc<HashSet<Sym>>,
    pub(super) vars: Arc<HashSet<Sym>>,
    pub(super) syms: Syms,
    ops: Arc<Vec<(Sym, FnId)>>,
    getters: Arc<HashMap<Sym, FnId>>,
    struct_sets: Arc<HashMap<StructId, usize>>,
    struct_defaults: Arc<HashMap<StructId, Vec<Option<Ty>>>>,
    struct_arity: Arc<HashMap<StructId, Vec<(StructId, usize)>>>,
    tuples: Arc<HashMap<usize, StructId>>,
    sugar: Sugar,
    roles: Roles,
    records: Arc<HashMap<Vec<String>, StructId>>,
    members: Arc<HashMap<Sym, StructId>>,
}

impl Decls {
    /// Its tables as they are now, for the types decls converts while it builds the rest.
    pub(super) fn tables(&self) -> Tables {
        Tables {
            aliases: self.aliases.clone(),
            consts: self.consts.clone(),
            values: self.top_values.clone(),
            vars: self.top_vars.clone(),
            syms: self.syms.clone(),
            ops: self.operator_fns(),
            getters: self.domain_getters(),
            struct_sets: self.struct_sets.clone(),
            struct_defaults: self.struct_defaults.clone(),
            struct_arity: self.struct_arity.clone(),
            tuples: self.tuples.clone(),
            sugar: self.sugar,
            roles: self.roles.clone(),
            records: self.records.clone(),
            members: self.member_types.clone(),
        }
    }
}

impl TypeTables for Tables {
    fn text(&self, s: Sym) -> &str {
        self.syms.text(s)
    }
    fn sym(&self, text: &str) -> Option<Sym> {
        self.syms.get(text)
    }
    fn alias(&self, id: AliasId) -> &AliasDecl {
        &self.aliases[id.0]
    }
    fn konst(&self, id: ConstId) -> &LibConst {
        &self.consts[id.0]
    }
    fn top_value(&self, name: &str) -> bool {
        self.syms
            .get(name)
            .is_some_and(|s| self.values.contains(&s))
    }
    fn top_var(&self, name: &str) -> bool {
        self.syms.get(name).is_some_and(|s| self.vars.contains(&s))
    }
    fn operator(&self, op: &str) -> Option<FnId> {
        let op = self.syms.get(op)?;
        self.ops.iter().find(|(o, _)| *o == op).map(|&(_, f)| f)
    }
    fn getter(&self, m: &str) -> Option<FnId> {
        self.getters.get(&self.syms.get(m)?).copied()
    }
    fn n_sets(&self, s: StructId) -> usize {
        self.struct_sets.get(&s).copied().unwrap_or(0)
    }
    fn struct_defaults(&self, s: StructId) -> Option<&Vec<Option<Ty>>> {
        self.struct_defaults.get(&s)
    }
    fn by_arity(&self, s: StructId, n: usize) -> StructId {
        by_arity(&self.struct_arity, s, n)
    }
    fn tuple(&self, n: usize) -> Option<StructId> {
        self.tuples.get(&n).copied()
    }
    fn widest_tuple(&self) -> usize {
        self.tuples.keys().max().copied().unwrap_or(0)
    }
    fn record(&self, fields: &[String]) -> Option<StructId> {
        self.records.get(fields).copied()
    }
    fn record_fields(&self, s: StructId) -> Option<Vec<String>> {
        (self.records.iter())
            .find(|(_, id)| **id == s)
            .map(|(fs, _)| fs.clone())
    }
    fn member(&self, m: &str) -> Option<StructId> {
        self.members.get(&self.syms.get(m)?).copied()
    }
    fn sugar(&self) -> Sugar {
        self.sugar
    }
    fn roles(&self) -> &Roles {
        &self.roles
    }
}

/// `[0..V.count - 1]T`: an axis ending at a domain's count, keyed by that range type (R04.3.4).
fn count_axis(cx: &TyCx, t: &ast::Type) -> bool {
    let TypeKind::Range { lo, hi, inclusive } = &t.kind else {
        return false;
    };
    count_end(lo, hi, *inclusive)
        .is_some_and(|(_, d, m, _)| is_type_name(cx, d) && cx.env.getter(m).is_some())
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
            let Some(fs) = cx.env.record_fields(s) else {
                errors.push(missing("a type that is not a record"));
                return Ty::Error;
            };
            let Some(made) = crate::records::without_shape(form, &fs) else {
                errors.push(missing("the record"));
                return Ty::Error;
            };
            let Some(id) = cx.env.record(&made) else {
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
    cx.generics.iter().any(|g| g == n) || (cx.names)(n).is_some()
}

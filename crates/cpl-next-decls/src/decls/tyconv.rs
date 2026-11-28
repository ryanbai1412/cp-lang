//! Types as written turned into types: names looked up in a module's view, aliases expanded,
//! a struct's left-off arguments filled from its defaults, and a set for each fn type. Every
//! written type and every form decls makes reaches one conversion, keyed by `Sym`.
use super::forms::TyForm;
use super::*;
use crate::ops::{bin_intr, fold, un_intr};
use crate::views::{Names, RoleStructs, Shapes, Texts, TypeTables};
use cpl_next_ir::ids::ItemId;
use cpl_next_ir::ty::End;
use cpl_next_ir::unbuilt::Unbuilt;
use cpl_next_syntax::ast::{self as next, AstNode, Expr, GenericArg, Type};
use cpl_next_syntax::literal::int_value;
use cpl_next_syntax::ops::BinOp;
use cpl_next_syntax::tree::SyntaxToken;
use cpl_next_syntax::TokenKind;

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

/// The record struct whose fields are named `fields`, in that order.
pub(crate) fn record_of(
    records: &HashMap<Vec<String>, StructId>,
    syms: &Syms,
    fields: &[Sym],
) -> Option<StructId> {
    let names: Vec<String> = fields.iter().map(|f| syms.text(*f).to_string()).collect();
    records.get(&names).copied()
}

/// Record `s`'s field names, or `None` when `s` is not a record.
pub(crate) fn fields_of_record(
    records: &HashMap<Vec<String>, StructId>,
    syms: &Syms,
    s: StructId,
) -> Option<Vec<Sym>> {
    let (names, _) = records.iter().find(|(_, id)| **id == s)?;
    names.iter().map(|n| syms.get(n)).collect()
}

/// Type arguments written but left off the end take their defaults, which may name the
/// arguments before them (R08.2.5).
pub(crate) fn fill_defaults(defaults: Option<&Vec<Option<Ty>>>, a: &mut Vec<Ty>) {
    let Some(ds) = defaults else { return };
    while a.len() < ds.len() {
        let Some(t) = &ds[a.len()] else { break };
        let t = Subst { tys: a, sets: &[] }.ty(t);
        a.push(t);
    }
}

/// A `without` met over a parameter: the parameter, the form, the added fields' types.
pub(super) type Row = (usize, crate::records::Form, Vec<Ty>);

pub(super) struct TyCx<'a> {
    /// What a name means where the type is written.
    pub(super) names: &'a dyn Fn(Sym) -> Option<Named>,
    /// The signatures' tables, each read by key.
    pub(super) env: &'a dyn TypeTables,
    /// The names' ids, for the names the type writes.
    pub(super) syms: &'a dyn crate::sym::SymLookup,
    /// Where the item being converted starts in the source, when known: its spans count from it.
    pub(super) item: Option<ItemId>,
    pub(super) generics: &'a [Sym],
    /// The signature's own generics, where an alias's expansion names them by position.
    pub(super) outer: &'a [Sym],
    /// What the generics stand for, when converting an alias's type.
    pub(super) args: Option<&'a [Ty]>,
    /// The aliases being expanded, to report one that names itself.
    pub(super) expanding: Vec<usize>,
    /// The types a body declares by name: its domains, and its ranges with run-time ends.
    pub(super) doms: &'a HashMap<Sym, Ty>,
    /// `t.Pre`: a domain a binding in scope owns, found through that binding, not by spelling.
    pub(super) scoped: &'a dyn Fn(Sym) -> Option<Ty>,
    /// `Self` of the trait or impl being converted, for a form's [`TyForm::SelfTy`].
    pub(super) self_ty: Option<Ty>,
    /// Each `(to: X, ..R without (u, v))` met over a row-bound parameter `R`: its index, its form
    /// and the added fields' types, for the bound it puts on `R` (R08.4.1).
    pub(super) rows: Option<&'a std::cell::RefCell<Vec<Row>>>,
    /// When converting a result type: the next result slot.
    pub(super) ret_slots: Option<usize>,
    /// Next implicit set parameter; `None` where a fn type may not appear bare.
    pub(super) next_set: Option<usize>,
    /// In a body: the type the checker gave each axis or range type with run-time ends, by its
    /// item-relative span, read in place of converting it.
    pub(super) axes: Option<&'a HashMap<cpl_diag::Span, Ty>>,
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

pub(super) fn alias(cx: &mut TyCx, id: usize, name: &str, a: Vec<Ty>, errors: &mut Vec<Msg>) -> Ty {
    let env = cx.env;
    let decl = env.alias(AliasId(id));
    if a.len() != decl.generics.len() {
        errors.push(Msg::verdict(
            cpl_diag::codes::WRONG_ARG_COUNT,
            format!(
                "`{}` takes {} type arguments, found {}",
                name,
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
        axes: None,
        names: &|n: Sym| decl.names.get(&n).copied(),
        env,
        syms: cx.syms,
        item: None,
        generics: &decl.generics,
        outer: cx.outer,
        args: Some(&a),
        expanding,
        doms: if decl.program { cx.doms } else { &none },
        scoped: if decl.program { cx.scoped } else { &unscoped },
        self_ty: None,
        rows: cx.rows,
        ret_slots: cx.ret_slots,
        next_set: cx.next_set,
    };
    let t = conv(&mut inner, &decl.ty, errors);
    (cx.ret_slots, cx.next_set) = (inner.ret_slots, inner.next_set);
    t
}

/// `Seq<T>` written where a trait is wanted.
pub(super) fn trait_ref(cx: &mut TyCx, t: &TyForm, errors: &mut Vec<Msg>) -> Option<TraitRef> {
    match t {
        TyForm::Named(n, args) => {
            let tr = trait_named(cx, Some(*n), cx.env.text(*n).to_string(), errors)?;
            let args = args.iter().map(|a| conv_form(cx, a, errors)).collect();
            Some(TraitRef { tr, args })
        }
        TyForm::Written(Type::Path(p)) => {
            let text = path_text(p);
            let tr = trait_named(cx, cx.syms.get(&text), text, errors)?;
            let args = generic_args(p)
                .iter()
                .map(|a| conv_arg(cx, a, errors))
                .collect();
            Some(TraitRef { tr, args })
        }
        // `(u: V, w: W, ..)`: the core trait of that row, over the field types (R08.4.1).
        TyForm::Written(Type::Record(r)) if r.fields().any(|f| f.is_rest() && f.ty().is_none()) => {
            let fs: Vec<next::RecordTypeField> = r.fields().filter(|f| !f.is_rest()).collect();
            let names: Vec<String> = (fs.iter())
                .map(|f| f.name().map_or(String::new(), |n| n.text().to_string()))
                .collect();
            let row = cx.syms.get(&crate::records::row_name(&names));
            let Some(Named::Trait(tr)) = row.and_then(|s| (cx.names)(s)) else {
                errors.push(Unbuilt::BoundNotNamed.into());
                return None;
            };
            let args = (fs.iter())
                .map(|f| f.ty().map_or(Ty::Error, |t| conv(cx, &t, errors)))
                .collect();
            Some(TraitRef { tr, args })
        }
        _ => {
            errors.push(Unbuilt::BoundNotNamed.into());
            None
        }
    }
}

fn trait_named(cx: &TyCx, n: Option<Sym>, text: String, errors: &mut Vec<Msg>) -> Option<TraitId> {
    match n.and_then(|n| (cx.names)(n)) {
        Some(Named::Trait(tr)) => Some(tr),
        _ => {
            errors.push(Msg::verdict(
                cpl_diag::codes::UNKNOWN_TYPE,
                format!("unknown trait `{text}`"),
            ));
            None
        }
    }
}

/// A range type with an end naming a value the program's top level binds, as written, and that
/// value: a struct field's layout is fixed before input is read, so no field is bounded by one
/// (R03.1.3).
pub(super) fn input_bound(t: &Type, values: &[TopValue], syms: &Syms) -> Option<(String, String)> {
    match t {
        Type::Range(r) => {
            let n = (r.syntax().descendants())
                .filter_map(next::NameRef::cast)
                .filter_map(|n| n.ident())
                .find(|n| (syms.get(n.text())).is_some_and(|s| top_value(values, s).is_some()))?;
            Some((r.syntax().text().trim().to_string(), n.text().to_string()))
        }
        Type::Tuple(ts) => ts
            .elems()
            .iter()
            .find_map(|(_, t)| input_bound(t, values, syms)),
        Type::Optional(o) => input_bound(&o.inner()?, values, syms),
        _ => None,
    }
}

/// The single capitals a type writes that name nothing: an impl header's undeclared type
/// parameters (R02.10.4).
pub(crate) fn unnamed(
    t: &Type,
    names: &dyn Fn(Sym) -> Option<Named>,
    syms: &mut Syms,
    out: &mut Vec<Sym>,
) {
    let mut all = Vec::new();
    names_in(t, &mut all);
    for n in all {
        if n.len() == 1 && n.chars().all(|c| c.is_ascii_uppercase()) {
            let s = syms.intern(&n);
            if names(s).is_none() && !out.contains(&s) {
                out.push(s);
            }
        }
    }
}

/// Every bare name (no type arguments) a type writes, in order.
pub(crate) fn names_in(t: &Type, out: &mut Vec<String>) {
    match t {
        Type::Path(p) => {
            let args = generic_args(p);
            if args.is_empty() {
                out.push(path_text(p));
            }
            for a in &args {
                if let GenericArg::Type(t) = a {
                    names_in(t, out);
                }
            }
        }
        Type::Optional(o) => o.inner().iter().for_each(|i| names_in(i, out)),
        Type::Tuple(ts) => ts.elems().iter().for_each(|(_, t)| names_in(t, out)),
        Type::Array(a) => {
            for d in a.dims().iter().flat_map(|d| d.args().collect::<Vec<_>>()) {
                if let GenericArg::Type(t) = d {
                    names_in(&t, out);
                }
            }
            a.elem().iter().for_each(|e| names_in(e, out));
        }
        Type::Fn(f) => {
            f.param().iter().for_each(|p| names_in(p, out));
            f.ret()
                .and_then(|r| r.ty())
                .iter()
                .for_each(|r| names_in(r, out));
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
    cpl_next_ir::prim::Prim::named(n).map(Ty::of_prim)
}

/// Each core type by its name, for a first layer's tables.
pub(super) fn core_types(syms: &mut Syms) -> HashMap<Sym, CoreType> {
    let mut out: HashMap<Sym, CoreType> = (cpl_next_ir::prim::Prim::ALL.into_iter())
        .map(|p| (syms.intern(p.name()), CoreType::Prim(p)))
        .collect();
    out.insert(syms.intern("Raw"), CoreType::Raw);
    out
}

/// A type decls makes or reads as a form.
pub(super) fn conv_form(cx: &mut TyCx, t: &TyForm, errors: &mut Vec<Msg>) -> Ty {
    match t {
        TyForm::Written(t) => conv(cx, t, errors),
        TyForm::Param(i) => param(cx, *i),
        TyForm::SelfTy => cx.self_ty.clone().unwrap_or_else(|| {
            errors.push(Msg::internal("`Self` outside a trait or impl"));
            Ty::Error
        }),
        TyForm::SelfAssoc(a) => {
            let s = cx.syms.get(&format!("Self.{}", cx.env.text(*a)));
            match s.and_then(|s| cx.generics.iter().position(|g| *g == s)) {
                Some(i) => param(cx, i),
                None => {
                    errors.push(Msg::internal("`Self.R` outside its trait"));
                    Ty::Error
                }
            }
        }
        TyForm::Named(n, args) => {
            let a = args.iter().map(|x| conv_form(cx, x, errors)).collect();
            named(cx, *n, a, errors)
        }
        TyForm::Record(fs) => {
            let names: Vec<Sym> = fs.iter().map(|(n, _)| *n).collect();
            let Some(d) = cx.env.record(&names) else {
                return unbuilt("Record", errors);
            };
            let a = fs.iter().map(|(_, t)| conv_form(cx, t, errors)).collect();
            Ty::Struct(d, a, Vec::new())
        }
    }
}

fn param(cx: &TyCx, i: usize) -> Ty {
    cx.args.map_or(Ty::Param(i), |a| a[i].clone())
}

pub(super) fn conv(cx: &mut TyCx, t: &Type, errors: &mut Vec<Msg>) -> Ty {
    if let Some(given) = cx.axes.and_then(|a| a.get(&t.syntax().span())) {
        return given.clone();
    }
    match t {
        Type::Path(p) => {
            let segs: Vec<SyntaxToken> = p.segments().collect();
            let a = generic_args(p)
                .iter()
                .map(|x| conv_arg(cx, x, errors))
                .collect();
            conv_path(cx, &segs, a, errors)
        }
        Type::Tuple(ts) => {
            let elems = ts.elems();
            if elems.is_empty() {
                return Ty::Unit;
            }
            if ts.is_grouping() {
                return conv(cx, &elems[0].1, errors);
            }
            // `(A, B)` is the `@role(tuple)` struct with two fields.
            let Some(d) = cx.env.tuple(elems.len()) else {
                return unbuilt("Tuple", errors);
            };
            let a = (elems.iter())
                .map(|(m, x)| match m {
                    true => unbuilt("Mut", errors),
                    false => conv(cx, x, errors),
                })
                .collect();
            let sets = (0..cx.env.n_sets(d)).map(|_| slot(cx)).collect();
            Ty::Struct(d, a, sets)
        }
        Type::Record(r) => conv_record(cx, r, errors),
        // `?T` is the struct filling the `option` role.
        Type::Optional(o) => match (cx.env.role_struct(Role::Option), o.inner()) {
            (_, _) if o.is_mut() => unbuilt("Mut", errors),
            (Some(d), Some(inner)) => Ty::Struct(d, vec![conv(cx, &inner, errors)], Vec::new()),
            (None, _) => unbuilt("`?T` with no `@role(option)` struct", errors),
            (Some(_), None) => Ty::Error,
        },
        Type::Range(r) => match r.expr() {
            Some(Expr::Range(re)) => conv_range(cx, &re, errors),
            Some(_) => unbuilt("Value", errors),
            None => Ty::Error,
        },
        Type::Array(a) => conv_array(cx, a, errors),
        Type::Fn(f) => {
            let ps = match f.param() {
                Some(Type::Tuple(ts)) => (ts.elems().into_iter())
                    .map(|(m, x)| {
                        let x = conv(cx, &x, errors);
                        if m {
                            Ty::Mut(Box::new(x))
                        } else {
                            x
                        }
                    })
                    .collect(),
                Some(p) => vec![conv(cx, &p, errors)],
                None => Vec::new(),
            };
            let r = match f.ret() {
                Some(r) if r.is_yields() => unbuilt("Yields", errors),
                Some(r) => r.ty().map_or(Ty::Unit, |r| conv(cx, &r, errors)),
                None => Ty::Unit,
            };
            let set = slot(cx);
            Ty::Fn(ps, Box::new(r), set)
        }
        Type::TypeOf(o) => item_of(cx, o, errors),
        Type::DomainBinder(b) => (b.name().and_then(|n| cx.syms.get(n.text())))
            .and_then(|s| cx.doms.get(&s).cloned())
            .unwrap_or(Ty::Error),
        // A result's `_`, filled in once its struct's domain parameters are known (R02.9.3).
        Type::Infer(_) if cx.ret_slots.is_some() => Ty::Error,
        Type::Infer(_) => unbuilt("Value", errors),
        Type::Alphabet(_) => unbuilt("Alphabet", errors),
        Type::Literal(_) => unbuilt("Literal", errors),
    }
}

/// A type argument: a type, or a value, which is its singleton range type (`Mod<7>` is
/// `Mod<7..=7>`, R03.5.1).
fn conv_arg(cx: &mut TyCx, a: &GenericArg, errors: &mut Vec<Msg>) -> Ty {
    match a {
        GenericArg::Type(Type::Range(r)) => match r.expr() {
            Some(Expr::Range(re)) => conv_range(cx, &re, errors),
            Some(e) => match const_int(cx, &e) {
                Some(k) => Ty::Range(k, k),
                None => {
                    let text = cpl_diag::msgs::ir::BAD_NUMBER_EXPECTED_A_COMPILE_TIME.say(&[]);
                    let m = Msg::verdict(cpl_diag::codes::BAD_NUMBER, text);
                    errors.push(at(cx, e.syntax().span(), m));
                    Ty::Error
                }
            },
            None => Ty::Error,
        },
        GenericArg::Type(t) => conv(cx, t, errors),
        // `D` of `impl Show for Bitset<D: Show>`, its bounds read with the impl's.
        GenericArg::Param(p) if !p.is_const() => {
            let s = p.name().and_then(|n| cx.syms.get(n.text()));
            match s.and_then(|s| cx.generics.iter().position(|g| *g == s)) {
                Some(i) => param(cx, i),
                None => unbuilt("GenericParam", errors),
            }
        }
        GenericArg::Param(_) => unbuilt("GenericParam", errors),
    }
}

fn at(cx: &TyCx, span: cpl_diag::Span, m: Msg) -> Msg {
    match &cx.item {
        Some(i) => Msg::at(i.at(span), m),
        None => m,
    }
}

fn generic_args(p: &next::PathType) -> Vec<GenericArg> {
    p.generic_args()
        .map(|a| a.args().collect())
        .unwrap_or_default()
}

fn path_text(p: &next::PathType) -> String {
    p.segments()
        .map(|s| s.text().to_string())
        .collect::<Vec<_>>()
        .join(".")
}

/// `D.m`, `T`, `Alias`, `Int`, `Struct<..>`.
fn conv_path(cx: &mut TyCx, segs: &[SyntaxToken], a: Vec<Ty>, errors: &mut Vec<Msg>) -> Ty {
    // `D.m`: the `@member(m)` struct at `D`, as `D.cuts` is `Cut<D>`.
    if let [base @ .., m] = segs {
        if !base.is_empty() {
            if let Some(id) = cx.syms.get(m.text()).and_then(|m| cx.env.member(m)) {
                let base = conv_path(cx, base, Vec::new(), errors);
                let sets = (0..cx.env.n_sets(id)).map(|_| slot(cx)).collect();
                return Ty::Struct(id, vec![base], sets);
            }
        }
    }
    let text = segs.iter().map(|s| s.text()).collect::<Vec<_>>().join(".");
    match cx.syms.get(&text) {
        Some(n) => named(cx, n, a, errors),
        None => unknown(&text, errors),
    }
}

/// The type a resolved name gives with its arguments: a parameter, an alias, a domain, a core
/// type or a struct.
pub(super) fn named(cx: &mut TyCx, n: Sym, mut a: Vec<Ty>, errors: &mut Vec<Msg>) -> Ty {
    if let Some(i) = cx.generics.iter().position(|g| *g == n) {
        return param(cx, i);
    }
    let found = (cx.names)(n);
    if let Some(Named::Alias(id)) = found {
        let env = cx.env;
        return alias(cx, id.0, env.text(n), a, errors);
    }
    if let Some(t) = cx.doms.get(&n).cloned().or_else(|| (cx.scoped)(n)) {
        return t;
    }
    match cx.env.core_type(n) {
        Some(CoreType::Prim(p)) => return Ty::of_prim(p),
        Some(CoreType::Raw) if a.len() == 1 => return Ty::Raw(Box::new(a.remove(0))),
        _ => {}
    }
    match found {
        // Set arguments of a struct named in a signature are fresh parameters.
        Some(Named::Struct(d)) => {
            let (d, a) = struct_args(cx.env, d, a);
            let sets = (0..cx.env.n_sets(d)).map(|_| slot(cx)).collect();
            Ty::Struct(d, a, sets)
        }
        _ => unknown(cx.env.text(n), errors),
    }
}

fn unknown(n: &str, errors: &mut Vec<Msg>) -> Ty {
    errors.push(Msg::verdict(
        cpl_diag::codes::UNKNOWN_TYPE,
        format!("unknown type `{n}`"),
    ));
    Ty::Error
}

fn unbuilt(kind: &str, errors: &mut Vec<Msg>) -> Ty {
    errors.push(
        Unbuilt::Type {
            kind: kind.to_string(),
        }
        .into(),
    );
    Ty::Error
}

/// `(lo: A, hi: B)` is the record struct of those field names (R02.5.2); `(to: V, ..R without
/// (u, v))` is what `without` makes of `R`. An open row `(u: V, ..)` is only a bound.
fn conv_record(cx: &mut TyCx, r: &next::RecordType, errors: &mut Vec<Msg>) -> Ty {
    let fields: Vec<next::RecordTypeField> = r.fields().collect();
    let text = |t: Option<SyntaxToken>| t.map_or(String::new(), |t| t.text().to_string());
    if let Some(rest) = fields.iter().find(|f| f.is_rest()) {
        let Some(base) = rest.ty() else {
            return unbuilt("record row with no base after `..`", errors);
        };
        let prefix: Vec<(String, Option<Type>)> = (fields.iter())
            .filter(|f| !f.is_rest())
            .map(|f| (text(f.name()), f.ty()))
            .collect();
        let form = (
            prefix.iter().map(|(n, _)| n.clone()).collect(),
            rest.removed()
                .into_iter()
                .map(|t| t.text().to_string())
                .collect(),
        );
        let base = conv(cx, &base, errors);
        return without(cx, &form, &prefix, base, r.syntax().span(), errors);
    }
    // `domain C` in a result is the fresh domain the fn names `C` (R02.9.3).
    let names: Vec<String> = (fields.iter())
        .map(|f| match (f.name(), f.ty()) {
            (Some(n), _) => n.text().to_string(),
            (None, Some(Type::DomainBinder(b))) => text(b.name()),
            (None, _) => String::new(),
        })
        .collect();
    let Some(d) = record_named(cx, &names) else {
        return unbuilt("Record", errors);
    };
    let a = (fields.iter())
        .map(|f| f.ty().map_or(Ty::Error, |x| conv(cx, &x, errors)))
        .collect();
    Ty::Struct(d, a, Vec::new())
}

/// `lo..hi` as a type: constant ends give `Range`, an end naming a value parameter gives
/// `RangeEnds` (`0..P`, R03.5.1), and `lo..=D.m + k` a range to a domain's getter.
fn conv_range(cx: &mut TyCx, re: &next::RangeExpr, errors: &mut Vec<Msg>) -> Ty {
    let (lo, hi) = (re.lo(), re.hi());
    let lo_end = match &lo {
        Some(e) => end(cx, e),
        None => Some(End::Lit(i64::MIN + 1)),
    };
    let (hi_end, inclusive) = match &hi {
        Some(e) => (end(cx, e), re.is_inclusive()),
        None => (Some(End::Lit(i64::MAX)), true),
    };
    if let (Some(l), Some(h)) = (lo_end, hi_end) {
        let t = Ty::range_ends(l, h, inclusive);
        let unmade = matches!(
            &t,
            Ty::RangeEnds {
                lo: End::Lit(_),
                hi: End::Lit(_),
                ..
            }
        );
        if !unmade {
            return t;
        }
    }
    // `0..=V.count`: its end is the domain's, where a struct's arguments name it.
    if let Some((lo, d, m, k)) = count_end(cx, lo.as_ref(), hi.as_ref(), re.is_inclusive()) {
        if let (true, Some(m)) = (
            is_type_name(cx, &d),
            cx.syms.get(m.text()).and_then(|m| cx.env.getter(m)),
        ) {
            let d = conv_path(cx, &[d], Vec::new(), errors);
            return Ty::RangeTo(Box::new(d), m, lo, k);
        }
    }
    match var_end(cx, [lo.as_ref(), hi.as_ref()]) {
        Some(m) => errors.push(m),
        None => errors.push(Unbuilt::RangeTypeOutsideBody.into()),
    }
    Ty::Error
}

/// A range type's end: a constant, a value parameter of the declaration, or a value the top
/// level binds with `let` (R02.10.3).
fn end(cx: &TyCx, e: &Expr) -> Option<End> {
    if let Some(k) = const_int(cx, e) {
        return Some(End::Lit(k));
    }
    let Expr::Name(n) = e else {
        return None;
    };
    let k = n.ident()?;
    let s = cx.syms.get(k.text())?;
    if let Some(i) = cx.generics.iter().position(|g| *g == s) {
        return Some(End::Of(Box::new(param(cx, i))));
    }
    let (id, v) = cx.env.top_value(cx.syms.get(k.text())?)?;
    (!v.mutable).then_some(End::Global {
        id,
        within: v.within,
    })
}

/// A range type ending at a top-level `var`: its value may change while the type is in use, so
/// only a `let` may end one (R02.10.3).
fn var_end(cx: &TyCx, ends: [Option<&Expr>; 2]) -> Option<Msg> {
    let k = ends.into_iter().flatten().find_map(|e| match e {
        Expr::Name(n) => n.ident().filter(|k| {
            cx.syms
                .get(k.text())
                .and_then(|s| cx.env.top_value(s))
                .is_some_and(|(_, v)| v.mutable)
        }),
        _ => None,
    })?;
    let msg = format!(
        "a type can't end at `{0}`, a `var` that may change while the type is in use; declare `{0}` with `let`",
        k.text()
    );
    Some(Msg::verdict(cpl_diag::codes::RANGE_END_VAR, msg))
}

/// The integer a constant expression is: literals, `+ - *`, and a `const` written as one
/// (`0..=CAP` with `const CAP = 2_000_000_000`).
pub(super) fn const_int(cx: &TyCx, e: &Expr) -> Option<i64> {
    let konst = |n: Sym| match (cx.names)(n) {
        Some(Named::Const(c)) => const_value(&cx.env.konst(c).init, &|_| None),
        _ => None,
    };
    const_value(e, &|t: &SyntaxToken| konst(cx.syms.get(t.text())?))
}

/// The integer a constant expression of literals is.
pub fn literal_int(e: &Expr) -> Option<i64> {
    const_value(e, &|_| None)
}

/// The integer `e` is, with each name's value from `name`.
fn const_value(e: &Expr, name: &dyn Fn(&SyntaxToken) -> Option<i64>) -> Option<i64> {
    match e {
        Expr::Literal(l) => {
            let t = l.token()?;
            match t.kind() {
                TokenKind::Int => i64::try_from(int_value(t.text())?).ok(),
                _ => None,
            }
        }
        Expr::Paren(p) => const_value(&p.inner()?, name),
        Expr::Prefix(p) => {
            let op = p.op_kind()?;
            fold(un_intr(op)?, &[const_value(&p.operand()?, name)?])
        }
        Expr::Bin(b) => {
            let (x, y) = (const_value(&b.lhs()?, name)?, const_value(&b.rhs()?, name)?);
            fold(bin_intr(b.op_kind()?)?, &[x, y])
        }
        Expr::Name(n) => name(&n.ident()?),
        _ => None,
    }
}

/// `lo..=D.m + k`: a range type with a constant start whose end is a static getter of a type
/// named `D`, give or take a constant (`0..=V.count`, `1..V.count + 1`). Gives `lo`, `D`, `m`
/// and `k`.
fn count_end(
    cx: &TyCx,
    lo: Option<&Expr>,
    hi: Option<&Expr>,
    inclusive: bool,
) -> Option<(i64, SyntaxToken, SyntaxToken, i64)> {
    fn end(cx: &TyCx, e: &Expr) -> Option<(SyntaxToken, SyntaxToken, i64)> {
        match e {
            Expr::Paren(p) => end(cx, &p.inner()?),
            Expr::Field(f) => match f.base()? {
                Expr::Name(d) => Some((d.ident()?, f.name()?, 0)),
                _ => None,
            },
            // An offset from the getter: `+ k` or `- k`.
            Expr::Bin(b) => {
                let op = b
                    .op_kind()
                    .filter(|op| matches!(op, BinOp::Add | BinOp::Sub))?;
                let j = const_int(cx, &b.rhs()?)?;
                let (d, m, k) = end(cx, &b.lhs()?)?;
                Some((d, m, fold(bin_intr(op)?, &[k, j])?))
            }
            _ => None,
        }
    }
    let lo = const_int(cx, lo?)?;
    let (d, m, k) = end(cx, hi?)?;
    let k = if inclusive { k } else { k.checked_sub(1)? };
    Some((lo, d, m, k))
}

/// `[D]T`, `[R, C]T` (one array keyed by the product, R04.2.11), `[domain S]T` (owns a fresh
/// domain), `[0..2]T` keyed by a constant range (R04.3.4).
fn conv_array(cx: &mut TyCx, a: &next::ArrayType, errors: &mut Vec<Msg>) -> Ty {
    let dims: Vec<GenericArg> = a.dims().map(|d| d.args().collect()).unwrap_or_default();
    let Some(elem) = a.elem() else {
        return Ty::Error;
    };
    let axes: Vec<Type> = (dims.iter())
        .filter_map(|d| match d {
            GenericArg::Type(t) => Some(t.clone()),
            GenericArg::Param(_) => None,
        })
        .collect();
    if axes.is_empty() || axes.len() != dims.len() {
        return unbuilt("array whose axes do not match its keys", errors);
    }
    // `[0..2]T`: an axis over a constant range is keyed by that range type (R04.3.4).
    if let [Type::Range(r)] = &axes[..] {
        if let Some(Expr::Range(re)) = r.expr() {
            let given = cx.axes.is_some_and(|a| a.contains_key(&r.syntax().span()));
            if !given && !static_range(cx, &re) {
                errors.push(Unbuilt::RangeIndexedArray.into());
                return Ty::Error;
            }
        }
    }
    let i = match &axes[..] {
        [i] => conv(cx, i, errors),
        // `[R, C]T` is keyed by `(R, C)`, the `@role(tuple)` struct of its axes.
        is => {
            let Some(d) = cx.env.tuple(is.len()) else {
                return unbuilt("Tuple", errors);
            };
            let parts = is.iter().map(|x| conv(cx, x, errors)).collect();
            let sets = (0..cx.env.n_sets(d)).map(|_| slot(cx)).collect();
            Ty::Struct(d, parts, sets)
        }
    };
    let e = conv(cx, &elem, errors);
    match cx.env.role_struct(Role::Array) {
        Some(array) => Ty::Struct(array, vec![i, e], Vec::new()),
        None => {
            errors.push(Msg::internal("`[D]T` needs the `@role(array)` struct"));
            Ty::Error
        }
    }
}

/// `type_of(f)`: the fn item's own type, one for a generic fn (R08.1.10).
fn item_of(cx: &mut TyCx, o: &next::TypeOfType, errors: &mut Vec<Msg>) -> Ty {
    let arg = o.arg();
    let span = arg
        .as_ref()
        .map_or(o.syntax().span(), |e| e.syntax().span());
    let fn_ref = |f: FnId| {
        Ty::Clo(Box::new(CloRef::Item(FnRef {
            decl: f,
            targs: Vec::new(),
            sets: Vec::new(),
            ctx: 0,
        })))
    };
    let not_fn = || {
        Msg::verdict(
            cpl_diag::codes::NOT_A_FN_ITEM,
            "`type_of` takes a `fn` declared by name, not this value",
        )
    };
    let msg = match arg {
        Some(Expr::Name(n)) => {
            let Some(k) = n.ident() else {
                return Ty::Error;
            };
            match cx.syms.get(k.text()).and_then(|s| (cx.names)(s)) {
                Some(Named::Fn(f)) => return fn_ref(f),
                None if cx
                    .syms
                    .get(k.text())
                    .and_then(|s| cx.env.top_value(s))
                    .is_none() =>
                {
                    Msg::verdict(
                        cpl_diag::codes::UNRESOLVED_NAME,
                        format!("unknown name `{}`", k.text()),
                    )
                }
                _ => not_fn(),
            }
        }
        // `type_of(+)`: the fn of the `@operator` trait that declares it.
        Some(Expr::OpValue(op)) => {
            let k: String = (op.syntax().tokens())
                .filter(|t| {
                    !t.kind().is_trivia()
                        && !matches!(t.kind(), TokenKind::LParen | TokenKind::RParen)
                })
                .map(|t| t.text().to_string())
                .collect();
            match cx.syms.get(&k).and_then(|k| cx.env.operator(k)) {
                Some(f) => return fn_ref(f),
                None => Msg::verdict(
                    cpl_diag::codes::NOT_A_FN_ITEM,
                    format!("no `@operator` trait declares `{k}`, so it is no fn item"),
                ),
            }
        }
        _ => not_fn(),
    };
    errors.push(at(cx, span, msg));
    Ty::Error
}

/// Converts a type written in a body: fn types get a placeholder set the checker replaces. It
/// reads the module's scope and the signatures, and gives the type with what is wrong in it.
pub fn body_type(at: &BodyScope, axes: &HashMap<cpl_diag::Span, Ty>, t: &Type) -> (Ty, Vec<Msg>) {
    at.run_at(Some(axes), |cx, errors| conv(cx, t, errors))
}

/// The range type `lo..hi` is when decls makes it (see [`static_axis`]), read in a body.
pub fn range_type(at: &BodyScope, re: &next::RangeExpr) -> (Ty, Vec<Msg>) {
    at.run(|cx, errors| conv_range(cx, re, errors))
}

/// Whether a range axis `[lo..hi]T` is keyed by its range type, which decls makes: both ends
/// constant, or ending at a domain's count. Any other the checker names in a body.
pub fn static_axis(
    tables: &dyn TypeTables,
    names: &dyn Names,
    syms: &dyn crate::sym::SymLookup,
    m: ModId,
    generics: &[Sym],
    doms: &HashMap<Sym, Ty>,
    re: &next::RangeExpr,
) -> bool {
    let at = BodyScope {
        tables,
        names,
        syms,
        m,
        generics,
        doms,
        scoped: &unscoped,
    };
    at.run(|cx, _| static_range(cx, re)).0
}

fn static_range(cx: &TyCx, re: &next::RangeExpr) -> bool {
    let constant = (re.lo().zip(re.hi()))
        .is_some_and(|(l, h)| const_int(cx, &l).is_some() && const_int(cx, &h).is_some());
    constant || count_axis(cx, re)
}

/// Struct `d` written with the arguments `a`: the declaration of that arity, its defaults filled.
pub fn struct_args(sigs: &dyn TypeTables, d: StructId, mut a: Vec<Ty>) -> (StructId, Vec<Ty>) {
    let d = sigs.by_arity(d, a.len());
    fill_defaults(sigs.struct_defaults(d), &mut a);
    (d, a)
}

/// Alias `id` given its arguments as types, as a call writes it: `AliasBox(42)` infers the
/// parameters it doesn't write, so they arrive as the checker's fresh types, not as syntax.
pub fn alias_type(at: &BodyScope, id: AliasId, args: Vec<Ty>) -> (Ty, Vec<Msg>) {
    at.run(|cx, errors| alias(cx, id.0, &at.tables.alias(id).name, args, errors))
}

/// What a type written in a body is read against: its module's names, the body's type
/// parameters, and the domains it has in scope.
pub struct BodyScope<'a> {
    pub tables: &'a dyn TypeTables,
    pub names: &'a dyn Names,
    pub syms: &'a dyn crate::sym::SymLookup,
    pub m: ModId,
    pub generics: &'a [Sym],
    pub doms: &'a HashMap<Sym, Ty>,
    pub scoped: &'a dyn Fn(Sym) -> Option<Ty>,
}

impl BodyScope<'_> {
    fn run<R>(&self, f: impl FnOnce(&mut TyCx, &mut Vec<Msg>) -> R) -> (R, Vec<Msg>) {
        self.run_at(None, f)
    }
    fn run_at<R>(
        &self,
        axes: Option<&HashMap<cpl_diag::Span, Ty>>,
        f: impl FnOnce(&mut TyCx, &mut Vec<Msg>) -> R,
    ) -> (R, Vec<Msg>) {
        let mut errors = Vec::new();
        let names = |n: Sym| self.names.named(self.m, n);
        let mut cx = TyCx {
            axes,
            names: &names,
            env: self.tables,
            syms: self.syms,
            item: None,
            generics: self.generics,
            outer: self.generics,
            args: None,
            expanding: Vec::new(),
            doms: self.doms,
            scoped: self.scoped,
            self_ty: None,
            rows: None,
            ret_slots: None,
            next_set: None,
        };
        let r = f(&mut cx, &mut errors);
        (r, errors)
    }
}

/// The tables as decls has built them so far, for the types it converts while building the rest.
pub(super) struct Tables {
    aliases: Layer<AliasDecl>,
    consts: Layer<LibConst>,
    structs: Layer<StructDecl>,
    pub(super) values: Arc<Vec<TopValue>>,
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
    core: Arc<HashMap<Sym, CoreType>>,
}

impl Decls {
    /// Its tables as they are now, for the types decls converts while it builds the rest.
    pub(super) fn tables(&self) -> Tables {
        Tables {
            aliases: self.aliases.clone(),
            consts: self.consts.clone(),
            structs: self.structs.clone(),
            values: self.top_values.clone(),
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
            core: self.core_types.clone(),
        }
    }
}

impl Texts for Tables {
    fn text(&self, s: Sym) -> &str {
        self.syms.text(s)
    }
}

impl TypeTables for Tables {
    fn alias(&self, id: AliasId) -> &AliasDecl {
        &self.aliases[id.0]
    }
    fn konst(&self, id: ConstId) -> &LibConst {
        &self.consts[id.0]
    }
    fn core_type(&self, n: Sym) -> Option<CoreType> {
        self.core.get(&n).copied()
    }
    fn top_value(&self, name: Sym) -> Option<(TopValueId, TopValue)> {
        let g = top_value(&self.values, name)?;
        Some((g, self.values[g.0].clone()))
    }
    fn operator(&self, op: Sym) -> Option<FnId> {
        self.ops.iter().find(|(o, _)| *o == op).map(|&(_, f)| f)
    }
    fn getter(&self, m: Sym) -> Option<FnId> {
        self.getters.get(&m).copied()
    }
}

impl RoleStructs for Tables {
    fn role_struct(&self, r: Role) -> Option<StructId> {
        crate::views::role_struct_of(&self.roles, &self.sugar, r)
    }
}

impl Shapes for Tables {
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
    fn record(&self, fields: &[Sym]) -> Option<StructId> {
        record_of(&self.records, &self.syms, fields)
    }
    fn record_fields(&self, s: StructId) -> Option<Vec<Sym>> {
        fields_of_record(&self.records, &self.syms, s)
    }
    fn field_names(&self, s: StructId) -> Vec<Sym> {
        self.structs[s.0].fields.iter().map(|(f, _)| *f).collect()
    }
    fn member(&self, m: Sym) -> Option<StructId> {
        self.members.get(&m).copied()
    }
    fn variant(&self, e: StructId, name: Sym) -> Option<usize> {
        (self.structs[e.0].variants.iter()).position(|(v, _)| *v == name)
    }
}

/// `[0..V.count - 1]T`: an axis ending at a domain's count, keyed by that range type (R04.3.4).
fn count_axis(cx: &TyCx, re: &next::RangeExpr) -> bool {
    count_end(cx, re.lo().as_ref(), re.hi().as_ref(), re.is_inclusive()).is_some_and(
        |(_, d, m, _)| {
            is_type_name(cx, &d)
                && cx
                    .syms
                    .get(m.text())
                    .and_then(|m| cx.env.getter(m))
                    .is_some()
        },
    )
}

/// The record struct with fields written `names`: none when a name is one no declaration gave.
fn record_named(cx: &TyCx, names: &[String]) -> Option<StructId> {
    let fields = names
        .iter()
        .map(|n| cx.syms.get(n))
        .collect::<Option<Vec<_>>>()?;
    cx.env.record(&fields)
}

/// No binding is in scope where decls reads a type.
pub(super) fn unscoped(_: Sym) -> Option<Ty> {
    None
}

/// `(to: X, ..B without (u, v))`: of a record, the record of its other fields after the added
/// ones; of a row-bound parameter `R`, the associated type its form's trait names, a parameter of
/// the signature (R08.4.1).
fn without(
    cx: &mut TyCx,
    form: &crate::records::Form,
    prefix: &[(String, Option<Type>)],
    base: Ty,
    span: cpl_diag::Span,
    errors: &mut Vec<Msg>,
) -> Ty {
    let missing = |cx: &TyCx, what: &str| {
        let m = Msg::verdict(
            cpl_diag::codes::UNKNOWN_FIELD,
            format!("`without` drops a field {what} lacks (R08.4.4)"),
        );
        at(cx, span, m)
    };
    match base {
        Ty::Struct(s, args, _) => {
            let Some(fs) = cx.env.record_fields(s) else {
                errors.push(missing(cx, "a type that is not a record"));
                return Ty::Error;
            };
            let fs: Vec<String> = fs.iter().map(|f| cx.env.text(*f).to_string()).collect();
            let Some(made) = crate::records::without_shape(form, &fs) else {
                errors.push(missing(cx, "the record"));
                return Ty::Error;
            };
            let Some(id) = record_named(cx, &made) else {
                return unbuilt("RowWithout", errors);
            };
            let a = (made.iter())
                .map(|f| match prefix.iter().find(|(n, _)| n == f) {
                    Some((_, Some(t))) => conv(cx, t, errors),
                    Some((_, None)) => Ty::Error,
                    None => fs
                        .iter()
                        .position(|x| x == f)
                        .map_or(Ty::Error, |k| args[k].clone()),
                })
                .collect();
            Ty::Struct(id, a, Vec::new())
        }
        Ty::Param(i) => {
            let rest = format!(
                "{}.{}",
                cx.env.text(cx.outer[i]),
                crate::records::without_member(form)
            );
            match (cx.syms.get(&rest)).and_then(|r| cx.outer.iter().position(|g| *g == r)) {
                Some(k) => {
                    if let Some(rows) = cx.rows {
                        let args = (prefix.iter())
                            .map(|(_, t)| t.as_ref().map_or(Ty::Error, |t| conv(cx, t, errors)))
                            .collect();
                        rows.borrow_mut().push((i, form.clone(), args));
                    }
                    Ty::Param(k)
                }
                None => {
                    errors.push(missing(cx, "the row bound"));
                    Ty::Error
                }
            }
        }
        Ty::Error => Ty::Error,
        _ => {
            errors.push(missing(cx, "a type that is not a record"));
            Ty::Error
        }
    }
}

/// `t` leaves some struct's parameter as `_`, directly or in an optional.
pub(super) fn has_hole(t: &Type) -> bool {
    match t {
        Type::Path(p) => generic_args(p).iter().any(|a| match a {
            GenericArg::Type(Type::Infer(_)) => true,
            GenericArg::Type(t) => has_hole(t),
            GenericArg::Param(_) => false,
        }),
        Type::Optional(o) => o.inner().is_some_and(|i| has_hole(&i)),
        Type::Tuple(ts) => ts.elems().iter().any(|(_, t)| has_hole(t)),
        Type::Record(r) => r.fields().any(|f| f.ty().is_some_and(|t| has_hole(&t))),
        _ => false,
    }
}

/// Whether `n` names a type where `cx` converts one: a generic or a declared name.
fn is_type_name(cx: &TyCx, n: &SyntaxToken) -> bool {
    cx.syms
        .get(n.text())
        .is_some_and(|s| cx.generics.contains(&s) || (cx.names)(s).is_some())
}

//! Contract 2: syntax -> declarations. Answers "what does this name mean"
//! without looking inside a body.
use crate::ty::*;
use cpl_syntax::ast::{self, FnBody, StmtKind, TypeKind};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SelfMode {
    None,
    Value,
    Mut,
}

pub struct FnDecl {
    pub name: String,
    /// The struct, trait or impl it sits in.
    pub owner: Option<Owner>,
    pub self_mode: SelfMode,
    /// Type parameters: the owner's first, then its own.
    pub generics: Vec<String>,
    pub bounds: Vec<(usize, TraitId)>,
    /// Without `self`.
    pub params: Vec<Param>,
    pub ret: Ty,
    /// Implicit set parameters: one per fn type in the parameters.
    pub n_sets: usize,
    /// How many function types its result holds.
    pub n_ret_sets: usize,
    /// A fn that runs the caller's code in the middle of its own: a generator (`yields T`,
    /// `false`) or a lender (`lends T`, `true`).
    pub region: Option<(bool, Ty)>,
    pub body: FnBody,
    pub intrinsic: bool,
    pub getter: bool,
    pub item: u32,
}
pub struct Param {
    pub name: String,
    pub ty: Ty,
    pub mutable: bool,
}
pub struct StructDecl {
    pub name: String,
    pub generics: Vec<String>,
    pub fields: Vec<(String, Ty)>,
    /// Per field: declared with `var`, so assignable after construction.
    pub var_fields: Vec<bool>,
    /// Per field: where its type is written, as an offset into the source.
    pub field_at: Vec<usize>,
    pub n_sets: usize,
    pub methods: HashMap<String, FnId>,
}
pub struct TraitDecl {
    pub methods: HashMap<String, FnId>,
}
pub struct ImplDecl {
    pub trait_: TraitId,
    pub for_: StructId,
    pub methods: HashMap<String, FnId>,
}
/// What a top-level name declares.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Named {
    Fn(FnId),
    Struct(StructId),
    Trait(TraitId),
}
/// What a method sits in. A method of an impl sits in the struct the impl is for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Owner {
    Struct(StructId),
    Trait(TraitId),
}

#[derive(Default)]
pub struct Decls {
    pub fns: Vec<FnDecl>,
    pub structs: Vec<StructDecl>,
    pub traits: Vec<TraitDecl>,
    pub impls: Vec<ImplDecl>,
    pub names: HashMap<String, Named>,
    pub struct_sets: HashMap<StructId, usize>,
    pub errors: Vec<String>,
}

/// Where a method call resolved to.
pub enum Member {
    Field(usize, Ty),
    /// A method, with the receiver's type and set arguments.
    Method(FnId, Vec<Ty>, Vec<SetTy>),
}

impl Decls {
    pub fn f(&self, d: FnId) -> &FnDecl {
        &self.fns[d.0]
    }
    pub fn s(&self, d: StructId) -> &StructDecl {
        &self.structs[d.0]
    }
    pub fn fn_named(&self, name: &str) -> Option<FnId> {
        match self.names.get(name) {
            Some(Named::Fn(f)) => Some(*f),
            _ => None,
        }
    }
    pub fn struct_named(&self, name: &str) -> Option<StructId> {
        struct_named(&self.names, name)
    }
    pub fn lookup_member(&self, recv: &Ty, name: &str) -> Option<Member> {
        let Ty::Struct(d, targs, sets) = recv else {
            return None;
        };
        let s = self.s(*d);
        if let Some(i) = s.fields.iter().position(|(n, _)| n == name) {
            let sub = Subst { tys: targs, sets };
            return Some(Member::Field(i, sub.ty(&s.fields[i].1)));
        }
        if let Some(m) = s.methods.get(name) {
            return Some(Member::Method(*m, targs.clone(), sets.clone()));
        }
        let m = self
            .impls
            .iter()
            .filter(|im| im.for_ == *d)
            .find_map(|im| im.methods.get(name))?;
        Some(Member::Method(*m, targs.clone(), sets.clone()))
    }
}

/// The first field of struct `root` that stores a `root` inline, however deep (R03.2.9).
/// Storage (`Raw`), references and function values end the walk: they hold their contents elsewhere.
pub fn field_holding_itself(d: &Decls, root: StructId) -> Option<usize> {
    fn holds(d: &Decls, t: &Ty, root: StructId, stack: &mut Vec<StructId>) -> bool {
        let Ty::Struct(id, tys, sets) = t else {
            return false;
        };
        if *id == root {
            return true;
        }
        if stack.contains(id) {
            return false;
        }
        stack.push(*id);
        let sub = Subst { tys, sets };
        let found = d
            .s(*id)
            .fields
            .iter()
            .any(|(_, f)| holds(d, &sub.ty(f), root, stack));
        stack.pop();
        found
    }
    d.s(root)
        .fields
        .iter()
        .position(|(_, f)| holds(d, f, root, &mut Vec::new()))
}

fn struct_named(names: &HashMap<String, Named>, name: &str) -> Option<StructId> {
    match names.get(name) {
        Some(Named::Struct(s)) => Some(*s),
        _ => None,
    }
}

struct TyCx<'a> {
    names: &'a HashMap<String, Named>,
    /// How many set parameters each struct has, so a struct named in a type gets its own.
    struct_sets: &'a HashMap<StructId, usize>,
    generics: &'a [String],
    /// The types a body declares by name: its domains, and its ranges with run-time ends.
    doms: &'a HashMap<String, Ty>,
    /// When converting a result type: the next result slot.
    ret_slots: Option<usize>,
    /// Next implicit set parameter; `None` where a fn type may not appear bare.
    next_set: Option<usize>,
}

/// The set for one function type written here: a parameter of the signature, a result slot, or a
/// placeholder the checker replaces in a body.
fn slot(cx: &mut TyCx) -> SetTy {
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
fn conv(cx: &mut TyCx, t: &ast::Type, errors: &mut Vec<String>) -> Ty {
    match &t.kind {
        TypeKind::Named { name, args } => {
            let n = name.text.as_str();
            let mut a: Vec<Ty> = args.iter().map(|x| conv(cx, x, errors)).collect();
            if let Some(i) = cx.generics.iter().position(|g| g == n) {
                return Ty::Param(i);
            }
            if let Some(t) = cx.doms.get(n) {
                return t.clone();
            }
            match n {
                "Int" => Ty::Int,
                "Bool" => Ty::Bool,
                "Raw" if a.len() == 1 => Ty::Raw(Box::new(a.remove(0))),
                _ => match struct_named(cx.names, n) {
                    // Set arguments of a struct named in a signature are fresh parameters.
                    Some(d) => {
                        let k = cx.struct_sets.get(&d).copied().unwrap_or(0);
                        let sets = (0..k).map(|_| slot(cx)).collect();
                        Ty::Struct(d, a, sets)
                    }
                    None => {
                        errors.push(format!("unknown type `{n}`"));
                        Ty::Error
                    }
                },
            }
        }
        TypeKind::Tuple(ts) if ts.is_empty() => Ty::Unit,
        // `(A, B)` is the library's `Tuple2<A, B>`. The slice finds it by name; the design wants a marker.
        TypeKind::Tuple(ts)
            if (2..=4).contains(&ts.len())
                && struct_named(cx.names, &format!("Tuple{}", ts.len())).is_some() =>
        {
            let named = ast::Type {
                kind: TypeKind::Named {
                    name: ast::Name {
                        text: format!("Tuple{}", ts.len()),
                        span: t.span,
                    },
                    args: ts.clone(),
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
                    "slice: a range type whose ends are not constants, outside a body".to_string(),
                );
                Ty::Error
            }
        },
        TypeKind::Array { index, .. }
            if index.len() == 1 && matches!(index[0].kind, TypeKind::Range { .. }) =>
        {
            errors.push("slice: an array indexed by a range outside a `let` or `read`".to_string());
            Ty::Error
        }
        TypeKind::Array { index, elem } if index.len() == 1 => {
            let i = conv(cx, &index[0], errors);
            let e = conv(cx, elem, errors);
            match struct_named(cx.names, "Array") {
                Some(array) => Ty::Struct(array, vec![i, e], Vec::new()),
                None => {
                    errors.push("`[D]T` needs the library's `Array`".to_string());
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
        other => {
            errors.push(format!("slice: type {}", crate::kind_name(other)));
            Ty::Error
        }
    }
}

/// Converts a type written in a body: fn types get a placeholder set the checker replaces.
pub fn body_type(
    d: &Decls,
    generics: &[String],
    doms: &HashMap<String, Ty>,
    t: &ast::Type,
    errors: &mut Vec<String>,
) -> Ty {
    conv(
        &mut TyCx {
            names: &d.names,
            struct_sets: &d.struct_sets,
            generics,
            doms,
            ret_slots: None,
            next_set: None,
        },
        t,
        errors,
    )
}

pub fn collect(items: &[(u32, Vec<ast::Stmt>)]) -> (Decls, Vec<(u32, ast::Stmt)>) {
    let mut d = Decls::default();
    let mut main = Vec::new();
    // Pass 1: names, so signatures can mention later declarations.
    let mut pending: Vec<(u32, &ast::Stmt, Option<Named>)> = Vec::new();
    for (item, stmts) in items {
        for s in stmts {
            match &s.kind {
                StmtKind::Struct(sd) => {
                    let id = StructId(d.structs.len());
                    d.names.insert(sd.name.text.clone(), Named::Struct(id));
                    d.structs.push(StructDecl {
                        name: sd.name.text.clone(),
                        generics: sd.generics.iter().map(|g| g.text.clone()).collect(),
                        fields: Vec::new(),
                        var_fields: Vec::new(),
                        field_at: Vec::new(),
                        n_sets: 0,
                        methods: HashMap::new(),
                    });
                    pending.push((*item, s, Some(Named::Struct(id))));
                }
                StmtKind::Trait { name, .. } => {
                    let id = TraitId(d.traits.len());
                    d.names.insert(name.text.clone(), Named::Trait(id));
                    d.traits.push(TraitDecl {
                        methods: HashMap::new(),
                    });
                    pending.push((*item, s, Some(Named::Trait(id))));
                }
                StmtKind::Fn(_) | StmtKind::Impl { .. } => pending.push((*item, s, None)),
                _ => main.push((*item, s.clone())),
            }
        }
    }
    // Struct fields first, to a fixed point: a struct holding a struct with function fields has those sets too.
    loop {
        let mut changed = false;
        for (_, s, id) in &pending {
            let (StmtKind::Struct(sd), Some(Named::Struct(id))) = (&s.kind, id) else {
                continue;
            };
            let generics: Vec<String> = sd.generics.iter().map(|g| g.text.clone()).collect();
            let no_doms = HashMap::new();
            let (names, sets) = (d.names.clone(), d.struct_sets.clone());
            let mut cx = TyCx {
                names: &names,
                struct_sets: &sets,
                generics: &generics,
                doms: &no_doms,
                ret_slots: None,
                next_set: Some(0),
            };
            let mut errors = Vec::new();
            for f in &sd.fields {
                conv(&mut cx, &f.ty, &mut errors);
            }
            let n = cx.next_set.unwrap_or(0);
            if d.struct_sets.insert(*id, n) != Some(n) {
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    for (item, s, id) in pending {
        match (&s.kind, id) {
            (StmtKind::Struct(sd), Some(Named::Struct(id))) => {
                let generics: Vec<String> = sd.generics.iter().map(|g| g.text.clone()).collect();
                let no_doms = HashMap::new();
                let sets = d.struct_sets.clone();
                let mut cx = TyCx {
                    names: &d.names.clone(),
                    struct_sets: &sets,
                    generics: &generics,
                    doms: &no_doms,
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
                d.errors.extend(errors);
                let mut methods = HashMap::new();
                for m in &sd.methods {
                    if let StmtKind::Fn(f) = &m.kind {
                        let fid = add_fn(
                            &mut d,
                            item,
                            m,
                            f,
                            Some(Owner::Struct(id)),
                            &generics,
                            n_sets,
                        );
                        methods.insert(f.name.text.clone(), fid);
                    }
                }
                let sdecl = &mut d.structs[id.0];
                sdecl.fields = fields;
                sdecl.var_fields = sd.fields.iter().map(|f| f.mutable).collect();
                sdecl.field_at = sd
                    .fields
                    .iter()
                    .map(|f| (item + f.ty.span.start) as usize)
                    .collect();
                sdecl.n_sets = n_sets;
                sdecl.methods = methods;
            }
            (StmtKind::Trait { body, .. }, Some(Named::Trait(id))) => {
                let mut methods = HashMap::new();
                for m in &body.stmts {
                    if let StmtKind::Fn(f) = &m.kind {
                        let fid = add_fn(&mut d, item, m, f, Some(Owner::Trait(id)), &[], 0);
                        methods.insert(f.name.text.clone(), fid);
                    }
                }
                d.traits[id.0].methods = methods;
            }
            (
                StmtKind::Impl {
                    trait_name: Some(tr),
                    target,
                    body,
                    ..
                },
                _,
            ) => {
                let name_of = |t: &ast::Type| match &t.kind {
                    TypeKind::Named { name, .. } => d.names.get(&name.text).copied(),
                    _ => None,
                };
                let (Some(Named::Trait(tr)), Some(Named::Struct(for_))) =
                    (name_of(tr), name_of(target))
                else {
                    d.errors
                        .push("impl of an unknown trait or type".to_string());
                    continue;
                };
                if d.impls.iter().any(|im| im.trait_ == tr && im.for_ == for_) {
                    d.errors
                        .push("two impls of one trait for one type".to_string());
                    continue;
                }
                let generics = d.s(for_).generics.clone();
                let n_sets = d.s(for_).n_sets;
                let mut methods = HashMap::new();
                for m in &body.stmts {
                    if let StmtKind::Fn(f) = &m.kind {
                        let fid = add_fn(
                            &mut d,
                            item,
                            m,
                            f,
                            Some(Owner::Struct(for_)),
                            &generics,
                            n_sets,
                        );
                        methods.insert(f.name.text.clone(), fid);
                    }
                }
                d.impls.push(ImplDecl {
                    trait_: tr,
                    for_,
                    methods,
                });
            }
            (StmtKind::Fn(f), _) => {
                let fid = add_fn(&mut d, item, s, f, None, &[], 0);
                d.names.insert(f.name.text.clone(), Named::Fn(fid));
            }
            _ => {}
        }
    }
    (d, main)
}

fn add_fn(
    d: &mut Decls,
    item: u32,
    s: &ast::Stmt,
    f: &ast::FnDecl,
    owner: Option<Owner>,
    outer: &[String],
    outer_sets: usize,
) -> FnId {
    let mut generics = outer.to_vec();
    generics.extend(f.generics.iter().map(|g| g.text.clone()));
    let names = d.names.clone();
    let no_doms = HashMap::new();
    let sets = d.struct_sets.clone();
    let mut cx = TyCx {
        names: &names,
        struct_sets: &sets,
        generics: &generics,
        doms: &no_doms,
        ret_slots: None,
        next_set: Some(outer_sets),
    };
    let mut errors = Vec::new();
    let mut self_mode = SelfMode::None;
    let mut params = Vec::new();
    for p in &f.params {
        if p.name.text == "self" {
            self_mode = if p.mutable {
                SelfMode::Mut
            } else {
                SelfMode::Value
            };
            continue;
        }
        let ty = match &p.ty {
            Some(t) => conv(&mut cx, t, &mut errors),
            None => {
                errors.push(format!("parameter `{}` needs a type", p.name.text));
                Ty::Error
            }
        };
        params.push(Param {
            name: p.name.text.clone(),
            ty,
            mutable: p.mutable,
        });
    }
    let n_sets = cx.next_set.unwrap_or(0);
    // A fn type in a result is "whatever this fn returns": filled in after the body is checked.
    cx.next_set = None;
    cx.ret_slots = Some(0);
    let mut region = None;
    let ret = match (&f.ret, &f.lends) {
        (_, Some(l)) => {
            region = Some((true, conv(&mut cx, &l.ty, &mut errors)));
            Ty::Unit
        }
        (Some(t), None) => match &t.kind {
            TypeKind::Yields(inner) => {
                region = Some((false, conv(&mut cx, inner, &mut errors)));
                Ty::Unit
            }
            _ => conv(&mut cx, t, &mut errors),
        },
        (None, None) => Ty::Unit,
    };
    let n_ret_sets = cx.ret_slots.unwrap_or(0);
    let mut bounds = Vec::new();
    for b in &f.bounds {
        let Some(i) = generics.iter().position(|g| *g == b.name.text) else {
            continue;
        };
        for t in &b.traits {
            if let TypeKind::Named { name, .. } = &t.kind {
                match d.names.get(&name.text) {
                    Some(Named::Trait(tr)) => bounds.push((i, *tr)),
                    _ => errors.push(format!("unknown trait `{}`", name.text)),
                }
            }
        }
    }
    d.errors.extend(errors);
    // A decorator changes meaning, so one the slice does not implement refuses the program; it is never skipped.
    for x in &s.decorators {
        if !matches!(x.name.text.as_str(), "get" | "multitest") {
            d.errors
                .push(format!("slice: decorator `@{}`", x.name.text));
        }
    }
    let has = |n: &str| s.decorators.iter().any(|x| x.name.text == n);
    d.fns.push(FnDecl {
        name: f.name.text.clone(),
        owner,
        self_mode,
        generics,
        bounds,
        params,
        ret,
        n_sets,
        n_ret_sets,
        region,
        body: f.body.clone(),
        intrinsic: matches!(&f.body, FnBody::Expr(e) if matches!(e.kind, cpl_syntax::ast::ExprKind::Intrinsic(_))),
        getter: has("get"),
        item,
    });
    FnId(d.fns.len() - 1)
}

/// The ends of a range type when both are constants, as an inclusive pair.
pub fn static_range(
    lo: &Option<Box<ast::Expr>>,
    hi: &Option<Box<ast::Expr>>,
    inclusive: bool,
) -> Option<(i64, i64)> {
    fn constant(e: &ast::Expr) -> Option<i64> {
        match &e.kind {
            ast::ExprKind::Int(i) => i64::try_from(*i).ok(),
            ast::ExprKind::Paren(x) => constant(x),
            ast::ExprKind::Unary {
                op: ast::UnOp::Neg,
                expr,
            } => constant(expr)?.checked_neg(),
            // `1e9`, `2e5`: a whole number written with an exponent.
            ast::ExprKind::Float(text) => {
                let (m, e) = text.split_once(['e', 'E'])?;
                m.parse::<i64>()
                    .ok()?
                    .checked_mul(10i64.checked_pow(e.parse().ok()?)?)
            }
            _ => None,
        }
    }
    let lo = match lo {
        Some(e) => constant(e)?,
        None => i64::MIN + 1,
    };
    let hi = match hi {
        Some(e) if inclusive => constant(e)?,
        Some(e) => constant(e)?.checked_sub(1)?,
        None => i64::MAX,
    };
    Some((lo, hi))
}

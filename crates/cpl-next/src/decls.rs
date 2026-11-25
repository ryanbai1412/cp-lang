//! Contract 2: syntax -> declarations. Answers "what does this name mean"
//! without looking inside a body.
use crate::ty::*;
use cpl_syntax::ast::{self, FnBody, StmtKind, TypeKind};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// A module: the core, a library file or the program.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord, Default)]
pub struct ModId(pub usize);

/// The core's module: the language's own declarations, seen by every module.
pub const CORE: ModId = ModId(0);

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
    /// What the body calls its receiver: `self`, or what an impl writes in its place
    /// (`fn lt(a, b)` for the trait's `fn lt(self, other: Self)`).
    pub self_name: String,
    /// The type of `self`, in `generics`, when it has one.
    pub self_ty: Option<Ty>,
    /// Type parameters: the owner's first, then its own.
    pub generics: Vec<String>,
    pub bounds: Vec<(usize, TraitRef)>,
    /// Without `self`.
    pub params: Vec<Param>,
    /// The names of the parameters passed only by name.
    pub labels: Vec<String>,
    pub ret: Ty,
    /// Implicit set parameters: one per fn type in the parameters.
    pub n_sets: usize,
    /// How many function types its result holds.
    pub n_ret_sets: usize,
    /// A fn that runs the caller's code in the middle of its own: a generator (`yields T`,
    /// `false`) or a lender (`lends T`, `true`).
    pub region: Option<(bool, Ty)>,
    pub body: FnBody,
    /// The name in `= @intrinsic(name)`, when that is its body.
    pub intrinsic: Option<String>,
    pub getter: bool,
    pub item: u32,
    pub module: ModId,
    /// What its signature uses that the slice does not build, said only when a call reaches it.
    pub errors: Vec<String>,
}
pub struct Param {
    pub name: String,
    /// The name a call gives it by: its label, else its name.
    pub label: String,
    pub ty: Ty,
    pub mutable: bool,
    /// After `;`: passed by name only.
    pub named_only: bool,
    /// Read in the fn's module when a call leaves the parameter out (R06.1.5).
    pub default: Option<ast::Expr>,
}
pub struct StructDecl {
    pub name: String,
    pub generics: Vec<String>,
    pub fields: Vec<(String, Ty)>,
    /// Per field: declared with `var`, so assignable after construction.
    pub var_fields: Vec<bool>,
    /// Per field: `@cache`, so a read-only method may write it (R05.10.10).
    pub cached: Vec<bool>,
    /// Per field: where its type is written, as an offset into the source.
    pub field_at: Vec<usize>,
    /// Per field: the value a constructor that leaves it out stores, read in `module` (R02.6.1).
    pub defaults: Vec<Option<ast::Expr>>,
    pub module: ModId,
    pub n_sets: usize,
    pub methods: HashMap<String, FnId>,
    /// Its static `new`, which `T(args)` calls outside its own body (R02.6.3).
    pub ctor: Option<FnId>,
    /// An enum's variants, each with the struct of its payload, named `Enum.Variant` (R02.7).
    pub variants: Vec<(String, StructId)>,
    /// What its fields use that the slice does not build, said only when the program reaches it.
    pub errors: Vec<String>,
}
pub struct TraitDecl {
    pub name: String,
    pub module: ModId,
    /// `trait Both = Sh + Nm`: met by meeting its supertraits, never implemented (R08.2.6).
    pub alias: bool,
    /// Its supertraits, in its own type parameters (R08.1.1).
    pub supers: Vec<TraitRef>,
    pub methods: HashMap<String, FnId>,
    /// What its library impls use that the slice does not build, said when one of its fns is reached.
    pub errors: Vec<String>,
    /// `trait Add<Rhs = Self>`: each parameter's default, in `Self` and the parameters before it.
    pub defaults: Vec<Option<Ty>>,
}
/// A trait with its arguments: `Seq<T>`.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TraitRef {
    pub tr: TraitId,
    pub args: Vec<Ty>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AliasId(pub usize);
/// `type Name<G> = T`: `T` is read in its module's names, where `Name` itself means what it means
/// outside (R02.6.2).
pub struct AliasDecl {
    pub name: String,
    pub module: ModId,
    /// Where it is declared, as an offset into the source.
    pub at: usize,
    pub generics: Vec<String>,
    pub ty: ast::Type,
    pub names: HashMap<String, Named>,
}
/// `impl Trait<A> for T`: `trait_` and `for_` are in the impl's own type parameters, which are
/// the names it writes that no declaration has (`impl Seq<T> for List<T>`) and its bounds' (R08.1.2).
pub struct ImplDecl {
    pub trait_: TraitRef,
    pub generics: Vec<String>,
    pub bounds: Vec<(usize, TraitRef)>,
    pub for_: Ty,
    /// Its parameters bounded by the domain trait: each fits only a domain.
    pub domains: Vec<usize>,
    pub methods: HashMap<String, FnId>,
    /// Where it is written.
    pub at: usize,
    pub module: ModId,
}
/// What a top-level name declares.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Named {
    Fn(FnId),
    Struct(StructId),
    Trait(TraitId),
    Alias(AliasId),
    /// A library item the slice does not build: an index into `Decls::refused`, said only
    /// when a program names it.
    Refused(usize),
}
/// What a method sits in.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Owner {
    Struct(StructId),
    Trait(TraitId),
    /// An index into `Decls::impls`.
    Impl(usize),
}

#[derive(Default)]
pub struct Decls {
    pub fns: Vec<FnDecl>,
    pub structs: Vec<StructDecl>,
    pub traits: Vec<TraitDecl>,
    pub impls: Vec<ImplDecl>,
    /// Per module, what each name means there: the core's names, then the prelude's (the
    /// program only, R11.3.1), then its own and what its `use` lines import (R11.2.3).
    pub views: Vec<HashMap<String, Named>>,
    pub module_names: Vec<String>,
    pub program: ModId,
    /// Per library module, what it uses outside its fns that the slice does not build.
    pub module_errors: Vec<Vec<String>>,
    /// Why each `Named::Refused` library item is not built.
    pub refused: Vec<String>,
    pub struct_sets: HashMap<StructId, usize>,
    pub aliases: Rc<Vec<AliasDecl>>,
    pub errors: Vec<String>,
    /// Declarations the program gets wrong: where, the code, what is wrong.
    pub misdeclared: Vec<(usize, &'static str, String)>,
    /// Each library's public free fns with a `self`, by name: a method call finds them with no
    /// import (R11.3.4).
    pub methods: HashMap<String, Vec<FnId>>,
    /// The program's top-level domains, which its declarations may name: each is the
    /// `Ty::Dom` of its place among them, filled when its statement runs (R02.9).
    pub top_doms: HashMap<String, Ty>,
    /// The trait core marks `@positions`: every domain's position type has it, and nothing else.
    pub domain_trait: Option<TraitId>,
    /// Core's `@coerce` fns: each converts a value of its parameter's type where its result's
    /// type is wanted (R04.8.10).
    pub coercions: Vec<FnId>,
}

/// Where a method call resolved to.
pub enum Member {
    Field(usize, Ty),
    /// A method, with the receiver's type and set arguments.
    Method(FnId, Vec<Ty>, Vec<SetTy>),
}

impl Decls {
    /// The domains a declaration in module `m` may name.
    fn doms_in(&self, m: ModId) -> HashMap<String, Ty> {
        match m == self.program {
            true => self.top_doms.clone(),
            false => HashMap::new(),
        }
    }
    /// What a declaration uses that the slice does not build: a library's is said when the program
    /// reaches it, the program's at once.
    fn module_or_program_errors(&mut self, m: ModId, errors: Vec<String>) {
        if self.is_library(m) {
            self.module_errors[m.0].extend(errors);
        } else {
            self.errors.extend(errors);
        }
    }
    pub fn f(&self, d: FnId) -> &FnDecl {
        &self.fns[d.0]
    }
    pub fn s(&self, d: StructId) -> &StructDecl {
        &self.structs[d.0]
    }
    pub fn named(&self, m: ModId, name: &str) -> Option<Named> {
        self.views[m.0].get(name).copied()
    }
    pub fn fn_named(&self, m: ModId, name: &str) -> Option<FnId> {
        match self.named(m, name) {
            Some(Named::Fn(f)) => Some(f),
            _ => None,
        }
    }
    pub fn struct_named(&self, m: ModId, name: &str) -> Option<StructId> {
        struct_named(&self.views[m.0], name)
    }
    /// The core's fn of this name, whatever the module calls its own.
    pub fn core_fn(&self, name: &str) -> Option<FnId> {
        self.fn_named(CORE, name)
    }
    pub fn core_struct(&self, name: &str) -> Option<StructId> {
        self.struct_named(CORE, name)
    }
    /// A read-only method that may still write its receiver's `@cache` fields, so a call
    /// lends the receiver for writing (R05.10.10).
    pub fn writes_cache(&self, f: FnId) -> bool {
        let fd = self.f(f);
        fd.self_mode == SelfMode::Value
            && matches!(fd.owner, Some(Owner::Struct(s)) if self.s(s).cached.contains(&true))
    }
    /// A library declaration's place is not the program's: its module's name.
    pub fn is_library(&self, m: ModId) -> bool {
        m != CORE && m != self.program
    }
    pub fn lookup_member(&self, recv: &Ty, name: &str) -> Option<Member> {
        if let Ty::Struct(d, targs, sets) = recv {
            let s = self.s(*d);
            if let Some(i) = s.fields.iter().position(|(n, _)| n == name) {
                let sub = Subst { tys: targs, sets };
                return Some(Member::Field(i, sub.ty(&s.fields[i].1)));
            }
            if let Some(m) = s.methods.get(name) {
                return Some(Member::Method(*m, targs.clone(), sets.clone()));
            }
        }
        self.impl_member(recv, name, None)
    }
    /// `r` with each parameter it leaves out given its default, for `self_ty` as `Self` (R08.1.1).
    fn with_defaults(&self, mut r: TraitRef, self_ty: &Ty) -> TraitRef {
        let defaults = &self.traits[r.tr.0].defaults;
        while let Some(Some(t)) = defaults.get(r.args.len()) {
            let mut tys = vec![self_ty.clone()];
            tys.extend(r.args.iter().cloned());
            let t = Subst {
                tys: &tys,
                sets: &[],
            }
            .ty(t);
            r.args.push(t);
        }
        r
    }
    /// The fn `name` of an impl for `recv`, of the trait named `trait_name` when one is given,
    /// or the trait's default when the impl does not write it (R08.1.2).
    pub fn impl_member(&self, recv: &Ty, name: &str, trait_name: Option<&str>) -> Option<Member> {
        let sets = match recv {
            Ty::Struct(_, _, ss) => ss.clone(),
            _ => Vec::new(),
        };
        self.impls.iter().find_map(|im| {
            let tr = &self.traits[im.trait_.tr.0];
            if trait_name.is_some_and(|n| n != tr.name) {
                return None;
            }
            if let Some(m) = im.methods.get(name) {
                let targs = self.impl_args(im, recv)?;
                return Some(Member::Method(*m, targs, sets.clone()));
            }
            let tm = *tr.methods.get(name)?;
            if matches!(self.f(tm).body, FnBody::None) {
                return None;
            }
            let binds = self.impl_args(im, recv)?;
            let sub = Subst {
                tys: &binds,
                sets: &[],
            };
            let mut targs = vec![recv.clone()];
            targs.extend(im.trait_.args.iter().map(|t| sub.ty(t)));
            Some(Member::Method(tm, targs, Vec::new()))
        })
    }
    /// Whether every field of `s` has an impl of `tr`, as a derived impl needs (R08.1.6).
    fn fields_have(&self, s: StructId, tr: TraitId) -> bool {
        let sd = self.s(s);
        (sd.fields.iter().zip(&sd.cached)).all(|((_, t), cached)| *cached || self.has_impl(t, tr))
    }
    /// Whether `t` has an impl of `tr`, its parameters taken to meet the impl's bounds.
    fn has_impl(&self, t: &Ty, tr: TraitId) -> bool {
        if matches!(t, Ty::Param(_) | Ty::Var(_) | Ty::Error) {
            return true;
        }
        if Some(tr) == self.domain_trait {
            return matches!(t, Ty::Dom(_));
        }
        (self.impls.iter())
            .filter(|im| im.trait_.tr == tr)
            .find_map(|im| Some((im, self.impl_args(im, t)?)))
            .is_some_and(|(im, targs)| {
                (im.bounds.iter())
                    .all(|(i, b)| targs.get(*i).is_none_or(|a| self.has_impl(a, b.tr)))
            })
    }
    /// The library method `name` whose `self` can be `recv` (R11.3.4).
    pub fn method_for(&self, name: &str, recv: &Ty) -> Option<FnId> {
        let fits = |f: &FnId| match (self.f(*f).self_ty.as_ref(), recv) {
            (Some(Ty::Param(_)), _) | (_, Ty::Var(_) | Ty::Error) => true,
            (Some(Ty::Struct(a, ..)), Ty::Struct(b, ..)) => a == b,
            (Some(t), r) => std::mem::discriminant(t) == std::mem::discriminant(r),
            (None, _) => false,
        };
        self.methods.get(name)?.iter().copied().find(|f| fits(f))
    }
    /// The impl's type arguments when it is for `t`; a type not known yet fits any.
    pub fn impl_args(&self, im: &ImplDecl, t: &Ty) -> Option<Vec<Ty>> {
        fn fit(p: &Ty, t: &Ty, binds: &mut [Option<Ty>], doms: &[usize]) -> bool {
            match (p, t) {
                (Ty::Param(i), _)
                    if doms.contains(i) && !matches!(t, Ty::Dom(_) | Ty::Var(_) | Ty::Error) =>
                {
                    false
                }
                // A parameter binds even to a type not known yet, so the method's own
                // arguments follow it once it is.
                (Ty::Param(i), _) => match &binds[*i] {
                    Some(b) => b == t || matches!(b, Ty::Var(_)) || matches!(t, Ty::Var(_)),
                    None => {
                        binds[*i] = Some(t.clone());
                        true
                    }
                },
                (_, Ty::Var(_) | Ty::Error) => true,
                (Ty::Raw(a), Ty::Raw(b)) | (Ty::Ref(a), Ty::Ref(b)) => fit(a, b, binds, doms),
                (Ty::Struct(d1, a, _), Ty::Struct(d2, b, _)) => {
                    d1 == d2
                        && a.len() == b.len()
                        && a.iter().zip(b).all(|(x, y)| fit(x, y, binds, doms))
                }
                (Ty::Range(..) | Ty::RangeOf(_), Ty::Int)
                | (Ty::Int, Ty::Range(..) | Ty::RangeOf(_)) => true,
                _ => p == t,
            }
        }
        let mut binds = vec![None; im.generics.len()];
        fit(&im.for_, t, &mut binds, &im.domains)
            .then(|| binds.into_iter().map(|b| b.unwrap_or(Ty::Error)).collect())
    }
}

/// The first field of struct `root` that stores a `root` inline, however deep (R03.2.9).
/// Storage (`Raw`), references and function values end the walk: they hold their contents elsewhere.
/// Whether alias `root`'s type needs `root` itself, through its own name or other aliases (E0323).
pub fn alias_cycle(d: &Decls, root: AliasId) -> bool {
    fn needs(
        d: &Decls,
        at: AliasId,
        t: &ast::Type,
        root: AliasId,
        seen: &mut Vec<AliasId>,
    ) -> bool {
        let inner: Vec<&ast::Type> = match &t.kind {
            TypeKind::Named { name, args } => {
                if let Some(Named::Alias(b)) = d.aliases[at.0].names.get(&name.text) {
                    if *b == root {
                        return true;
                    }
                    if !seen.contains(b) {
                        seen.push(*b);
                        if needs(d, *b, &d.aliases[b.0].ty, root, seen) {
                            return true;
                        }
                    }
                }
                args.iter().collect()
            }
            TypeKind::Array { index, elem } => index.iter().chain([&**elem]).collect(),
            TypeKind::OwnDomain { elem: t, .. }
            | TypeKind::Optional(t)
            | TypeKind::Mut(t)
            | TypeKind::Yields(t) => vec![&**t],
            TypeKind::Tuple(ts) => ts.iter().collect(),
            TypeKind::Record(fs) | TypeKind::Row(fs) => fs.iter().map(|(_, t)| t).collect(),
            TypeKind::Fn { params, ret } => params.iter().chain([&**ret]).collect(),
            TypeKind::RowWithout { prefix, base, .. } => {
                prefix.iter().map(|(_, t)| t).chain([&**base]).collect()
            }
            _ => Vec::new(),
        };
        inner.into_iter().any(|t| needs(d, at, t, root, seen))
    }
    needs(d, root, &d.aliases[root.0].ty, root, &mut vec![root])
}

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
    /// What the generics stand for, when converting an alias's type.
    args: Option<&'a [Ty]>,
    aliases: &'a [AliasDecl],
    /// The aliases being expanded, to report one that names itself.
    expanding: Vec<usize>,
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
fn alias(cx: &mut TyCx, id: usize, n: &str, a: Vec<Ty>, errors: &mut Vec<String>) -> Ty {
    let decl = &cx.aliases[id];
    if a.len() != decl.generics.len() {
        errors.push(format!(
            "`{n}` takes {} type arguments, found {}",
            decl.generics.len(),
            a.len()
        ));
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
fn read_elsewhere(name: &str) -> bool {
    matches!(name, "lint" | "stress" | "aka" | "was" | "on_unimplemented")
}

/// An impl of a trait for a type needs one of each of its supertraits for that type (R08.1.1).
/// The library's are checked with it; here one the slice refuses would read as missing.
fn supertraits_unmet(d: &Decls) -> Vec<(usize, &'static str, String)> {
    let mut out = Vec::new();
    for im in d.impls.iter().filter(|im| !d.is_library(im.module)) {
        let tr = &d.traits[im.trait_.tr.0];
        for sup in &tr.supers {
            let met = match &im.for_ {
                Ty::Param(i) => im.bounds.iter().any(|(j, r)| j == i && r.tr == sup.tr),
                t => (d.impls.iter()).any(|o| o.trait_.tr == sup.tr && d.impl_args(o, t).is_some()),
            };
            if !met {
                out.push((
                    im.at,
                    cpl_diag::codes::MISSING_IMPL.0,
                    format!(
                        "`impl {}` needs an `impl {}` for the same type (R08.1.1)",
                        tr.name, d.traits[sup.tr.0].name
                    ),
                ));
            }
        }
    }
    out
}
/// Whether one type could fit both impl headers; a parameter in `ad` or `bd` fits only a domain.
fn overlap(a: &Ty, ad: &[usize], b: &Ty, bd: &[usize]) -> bool {
    let only_doms = |t: &Ty, ds: &[usize]| matches!(t, Ty::Param(i) if ds.contains(i));
    match (a, b) {
        _ if only_doms(a, ad) => matches!(b, Ty::Param(_) | Ty::Dom(_)),
        _ if only_doms(b, bd) => matches!(a, Ty::Param(_) | Ty::Dom(_)),
        (Ty::Param(_), _) | (_, Ty::Param(_)) => true,
        (Ty::Raw(x), Ty::Raw(y)) | (Ty::Ref(x), Ty::Ref(y)) => overlap(x, ad, y, bd),
        (Ty::Struct(d1, x, _), Ty::Struct(d2, y, _)) => {
            d1 == d2 && x.iter().zip(y).all(|(x, y)| overlap(x, ad, y, bd))
        }
        _ => a == b,
    }
}

/// `Seq<T>` written where a trait is wanted.
fn trait_ref(cx: &mut TyCx, t: &ast::Type, errors: &mut Vec<String>) -> Option<TraitRef> {
    let TypeKind::Named { name, args } = &t.kind else {
        errors.push("slice: a bound that is not a named trait".to_string());
        return None;
    };
    let Some(Named::Trait(tr)) = cx.names.get(&name.text).copied() else {
        errors.push(format!("unknown trait `{}`", name.text));
        return None;
    };
    let args = args.iter().map(|a| conv(cx, a, errors)).collect();
    Some(TraitRef { tr, args })
}

/// The single capitals in an impl header that name nothing: its undeclared type parameters (R02.10.3).
fn unnamed(t: &ast::Type, names: &HashMap<String, Named>, out: &mut Vec<String>) {
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
fn shift_down(t: &Ty) -> Ty {
    let tys: Vec<Ty> = std::iter::once(Ty::Error)
        .chain((0..64).map(Ty::Param))
        .collect();
    Subst {
        tys: &tys,
        sets: &[],
    }
    .ty(t)
}

fn conv(cx: &mut TyCx, t: &ast::Type, errors: &mut Vec<String>) -> Ty {
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
            match n {
                "Int" => Ty::Int,
                "Bool" => Ty::Bool,
                "Char" => Ty::Char,
                "Str" if struct_named(cx.names, "Str").is_none() => {
                    errors.push("`Str` needs the library's `Str`".to_string());
                    Ty::Error
                }
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
    m: ModId,
    generics: &[String],
    doms: &HashMap<String, Ty>,
    t: &ast::Type,
    errors: &mut Vec<String>,
) -> Ty {
    conv(
        &mut TyCx {
            names: &d.views[m.0],
            struct_sets: &d.struct_sets,
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

/// The name a top-level item declares, if it declares one.
fn item_name(s: &ast::Stmt) -> Option<&str> {
    match &s.kind {
        StmtKind::Struct(sd) => Some(&sd.name.text),
        StmtKind::Trait { name, .. } => Some(&name.text),
        StmtKind::Fn(f) => Some(&f.name.text),
        StmtKind::TypeAlias { name, .. } => Some(&name.text),
        StmtKind::Const { name, .. } => Some(&name.text),
        _ => None,
    }
}

/// What each name means in module `mi`: the prelude's for the program, then the core's, then
/// what its `use` lines import, then its own (R11.2.3). `errors` is `None` while fns are not
/// declared yet, so a name not found is not yet known to be missing.
/// What every module declares and imports, from which each module's view is built.
struct Scopes<'a> {
    own: &'a [HashMap<String, Named>],
    public: &'a [HashSet<String>],
    names: &'a [String],
    imports: &'a [(ModId, &'a ast::Stmt)],
    prelude: &'a HashMap<String, String>,
}

fn view(
    sc: &Scopes,
    mi: usize,
    program: bool,
    errors: Option<&mut Vec<String>>,
) -> HashMap<String, Named> {
    let mut v = outer_view(sc, mi, program, errors);
    v.extend(sc.own[mi].iter().map(|(n, x)| (n.clone(), *x)));
    v
}

/// What each name means in module `mi` apart from what it declares itself.
fn outer_view(
    sc: &Scopes,
    mi: usize,
    program: bool,
    mut errors: Option<&mut Vec<String>>,
) -> HashMap<String, Named> {
    let Scopes {
        own,
        public,
        names,
        imports,
        prelude,
    } = *sc;
    let index = |m: &str| names.iter().position(|n| n == m);
    let export = |t: usize, n: &str| {
        (t == CORE.0 || public[t].contains(n))
            .then(|| own[t].get(n).copied())
            .flatten()
    };
    // The core holds the slice's own forms of some std names; a prelude name never hides one.
    let mut v = HashMap::new();
    if program {
        for (n, m) in prelude {
            if let Some(x) = index(m).and_then(|t| export(t, n)) {
                v.insert(n.clone(), x);
            }
        }
    }
    v.extend(own[CORE.0].iter().map(|(n, x)| (n.clone(), *x)));
    for (m, s) in imports.iter().filter(|(m, _)| m.0 == mi) {
        let _ = m;
        let StmtKind::Use {
            path,
            names: ns,
            glob,
            ..
        } = &s.kind
        else {
            continue;
        };
        let module = path
            .iter()
            .map(|n| n.text.as_str())
            .collect::<Vec<_>>()
            .join(".");
        // `use std.rand.Rng` names one item of `std.rand` (R11.2.1).
        let (module, ns) = match (index(&module), path.split_last()) {
            (None, Some((last, init))) if ns.is_empty() && !*glob => (
                init.iter()
                    .map(|n| n.text.as_str())
                    .collect::<Vec<_>>()
                    .join("."),
                std::slice::from_ref(last),
            ),
            _ => (module, &ns[..]),
        };
        // An unknown module is the loader's to report.
        let Some(t) = index(&module) else { continue };
        if *glob {
            for n in &public[t] {
                if let Some(x) = export(t, n) {
                    v.insert(n.clone(), x);
                }
            }
            continue;
        }
        if ns.is_empty() {
            if let Some(e) = errors.as_deref_mut() {
                e.push(format!("slice: `use {module}` without names"));
            }
            continue;
        }
        for n in ns {
            match export(t, &n.text) {
                Some(_) if own[mi].contains_key(&n.text) => {
                    if let Some(e) = errors.as_deref_mut() {
                        e.push(format!(
                            "slice: `use` of `{}`, which this file also declares",
                            n.text
                        ));
                    }
                }
                Some(x) => {
                    v.insert(n.text.clone(), x);
                }
                None => {
                    if let Some(e) = errors.as_deref_mut() {
                        e.push(format!(
                            "slice: `{module}` has no pub `{}` the slice declares",
                            n.text
                        ));
                    }
                }
            }
        }
    }
    v
}

pub fn collect(
    mods: &[crate::modules::Module],
    prelude: &HashMap<String, String>,
) -> (Decls, Vec<(u32, ast::Stmt)>) {
    let mut d = Decls::default();
    let n = mods.len();
    d.program = ModId(n - 1);
    d.module_names = mods.iter().map(|m| m.name.clone()).collect();
    d.module_errors = vec![Vec::new(); n];
    for s in mods[n - 1].items.iter().flat_map(|(_, stmts)| stmts) {
        if let StmtKind::Domain { name, .. } = &s.kind {
            let k = d.top_doms.len();
            d.top_doms.entry(name.text.clone()).or_insert(Ty::Dom(k));
        }
    }
    let mut own: Vec<HashMap<String, Named>> = vec![HashMap::new(); n];
    let mut public: Vec<HashSet<String>> = vec![HashSet::new(); n];
    let mut main = Vec::new();
    // Pass 1: names, so signatures can mention later declarations.
    let mut pending: Vec<(ModId, u32, &ast::Stmt, Option<Named>)> = Vec::new();
    let mut imports: Vec<(ModId, &ast::Stmt)> = Vec::new();
    let mut aliases: Vec<(ModId, usize, String, Vec<String>, ast::Type)> = Vec::new();
    let mut enums: Vec<(usize, String, Vec<String>)> = Vec::new();
    let derived: Vec<Vec<(u32, String, ast::Stmt)>> = (mods.iter().enumerate())
        .map(|(mi, m)| match mi == CORE.0 {
            true => Vec::new(),
            false => module_derived(&m.items),
        })
        .collect();
    let synth: Vec<Vec<(u32, ast::Stmt)>> = (mods.iter().enumerate())
        .map(|(mi, m)| {
            let mut out = Vec::new();
            for (item, stmts) in &m.items {
                for s in stmts {
                    if let StmtKind::Enum(e) = &s.kind {
                        let (structs, errors) = enum_structs(s, e);
                        match mi == n - 1 {
                            true => d.errors.extend(errors),
                            false => d.module_errors[mi].extend(errors),
                        }
                        let vs = e.variants.iter().map(|v| v.name.text.clone()).collect();
                        enums.push((mi, e.name.text.clone(), vs));
                        out.extend(structs.into_iter().map(|x| (*item, x)));
                    }
                }
            }
            out
        })
        .collect();
    for (mi, m) in mods.iter().enumerate() {
        let mid = ModId(mi);
        for (item, stmts) in &m.items {
            let made = synth[mi].iter().filter(|(i, _)| i == item).map(|(_, s)| s);
            for s in stmts
                .iter()
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
                        d.structs.push(StructDecl {
                            name: sd.name.text.clone(),
                            generics: sd.generics.iter().map(|g| g.text.clone()).collect(),
                            fields: Vec::new(),
                            var_fields: Vec::new(),
                            cached: Vec::new(),
                            field_at: Vec::new(),
                            defaults: Vec::new(),
                            module: mid,
                            n_sets: 0,
                            methods: HashMap::new(),
                            ctor: None,
                            variants: Vec::new(),
                            errors: Vec::new(),
                        });
                        pending.push((mid, *item, s, Some(Named::Struct(id))));
                    }
                    StmtKind::Trait { name, alias, .. } => {
                        let id = TraitId(d.traits.len());
                        if mi == CORE.0 && s.decorators.iter().any(|x| x.name.text == "positions") {
                            d.domain_trait = Some(id);
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
                            let e = format!("slice: decorator `@{}`", x.name.text);
                            match d.is_library(mid) {
                                true => d.module_errors[mi].push(e),
                                false => d.errors.push(e),
                            }
                        }
                        own[mi].insert(name.text.clone(), Named::Alias(AliasId(aliases.len())));
                        let generics = generics.iter().map(|g| g.text.clone()).collect();
                        let at = (*item + s.span.start) as usize;
                        aliases.push((mid, at, name.text.clone(), generics, ty.clone()));
                    }
                    StmtKind::Fn(_) | StmtKind::Impl { .. } => pending.push((mid, *item, s, None)),
                    StmtKind::Use { .. } => imports.push((mid, s)),
                    // A library's context is only reached by a `with`, which the checker refuses.
                    StmtKind::Context { .. } if d.is_library(mid) => {}
                    other if d.is_library(mid) => {
                        let e = format!("slice: library item {}", crate::kind_name(other));
                        match item_name(s) {
                            Some(n) => {
                                own[mi].insert(n.to_string(), Named::Refused(d.refused.len()));
                                d.refused.push(e);
                            }
                            None => d.module_errors[mi].push(e),
                        }
                    }
                    _ => main.push((*item, s.clone())),
                }
            }
        }
    }
    for (mi, name, vs) in enums {
        let Some(Named::Struct(id)) = own[mi].get(&name).copied() else {
            continue;
        };
        d.structs[id.0].variants = (vs.into_iter())
            .filter_map(|v| match own[mi].get(&format!("{name}.{v}")) {
                Some(Named::Struct(p)) => Some((v, *p)),
                _ => None,
            })
            .collect();
    }
    for (mi, made) in derived.iter().enumerate() {
        for (item, name, s) in made {
            if let Some(Named::Struct(id)) = own[mi].get(name).copied() {
                pending.push((ModId(mi), *item, s, Some(Named::Struct(id))));
            }
        }
    }
    let program = d.program.0;
    let names = d.module_names.clone();
    d.views = (0..n)
        .map(|mi| {
            view(
                &Scopes {
                    own: &own,
                    public: &public,
                    names: &names,
                    imports: &imports,
                    prelude,
                },
                mi,
                mi == program,
                None,
            )
        })
        .collect();
    d.aliases = Rc::new(
        aliases
            .into_iter()
            .map(|(m, at, name, generics, ty)| {
                let sc = Scopes {
                    own: &own,
                    public: &public,
                    names: &names,
                    imports: &imports,
                    prelude,
                };
                let mut v = outer_view(&sc, m.0, m.0 == program, None);
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
                }
            })
            .collect(),
    );
    for (i, a) in d.aliases.iter().enumerate() {
        if d.is_library(a.module) && alias_cycle(&d, AliasId(i)) {
            d.module_errors[a.module.0]
                .push(format!("type alias `{}` is defined by itself", a.name));
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
            let mut cx = TyCx {
                names: &names,
                struct_sets: &sets,
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
            if d.struct_sets.insert(*id, n) != Some(n) {
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    // Traits first, for the defaults every use of one reads; impls last, for their trait's fns.
    pending.sort_by_key(|(_, _, s, id)| match s.kind {
        StmtKind::Trait { .. } => 0,
        StmtKind::Impl { .. } if id.is_some() => 3,
        StmtKind::Impl { .. } => 2,
        _ => 1,
    });
    // A derived impl waits for its fields' types' impls, and is never made if one lacks it.
    let mut queue = pending;
    loop {
        let mut wait = Vec::new();
        let before = queue.len();
        for (mid, item, s, id) in std::mem::take(&mut queue) {
            let mi = mid.0;
            match (&s.kind, id) {
                (StmtKind::Struct(sd), Some(Named::Struct(id))) => {
                    let generics: Vec<String> =
                        sd.generics.iter().map(|g| g.text.clone()).collect();
                    let doms = d.doms_in(ModId(mi));
                    let sets = d.struct_sets.clone();
                    let mut cx = TyCx {
                        names: &d.views[mi].clone(),
                        struct_sets: &sets,
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
                    if sd.generic_defaults.iter().any(Option::is_some) {
                        errors.push(format!(
                            "slice: a default type argument of `{}`",
                            sd.name.text
                        ));
                    }
                    let n_sets = cx.next_set.unwrap_or(0);
                    if d.is_library(mid) {
                        d.structs[id.0].errors.extend(errors);
                    } else {
                        d.errors.extend(errors);
                    }
                    let mut methods = HashMap::new();
                    for m in &sd.methods {
                        if let StmtKind::Fn(f) = &m.kind {
                            let fid = add_fn(
                                &mut d,
                                mid,
                                item,
                                m,
                                f,
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
                    let mut errors = Vec::new();
                    let names = d.views[mi].clone();
                    let sets = d.struct_sets.clone();
                    let doms = d.doms_in(ModId(mi));
                    let mut cx = TyCx {
                        names: &names,
                        struct_sets: &sets,
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
                    // The impl's own type parameters: its bounds' names, then what its header writes
                    // that names nothing.
                    let mut generics: Vec<String> =
                        bounds.iter().map(|b| b.name.text.clone()).collect();
                    for t in [tr, target] {
                        unnamed(t, &names, &mut generics);
                    }
                    let sets = d.struct_sets.clone();
                    let doms = d.doms_in(ModId(mi));
                    let mut errors = Vec::new();
                    let mut cx = TyCx {
                        names: &names,
                        struct_sets: &sets,
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
                    if let (Some(r), true) = (&tref, d.is_library(mid) && !errors.is_empty()) {
                        d.traits[r.tr.0].errors.append(&mut errors);
                        continue;
                    }
                    // No program can reach a library impl of a trait the slice does not declare.
                    if tref.is_none() && d.is_library(mid) {
                        continue;
                    }
                    let Some(tref) = tref.filter(|_| errors.is_empty()) else {
                        if errors.is_empty() {
                            errors.push("impl of an unknown trait".to_string());
                        }
                        d.module_or_program_errors(mid, errors);
                        continue;
                    };
                    if let Some(Named::Struct(sid)) = id {
                        if !d.fields_have(sid, tref.tr) {
                            wait.push((mid, item, s, id));
                            continue;
                        }
                    }
                    let at = (item + s.span.start) as usize;
                    let tname = d.traits[tref.tr.0].name.clone();
                    let domains: Vec<usize> = (ibounds.iter())
                        .filter(|(_, r)| Some(r.tr) == d.domain_trait)
                        .map(|(i, _)| *i)
                        .collect();
                    if d.impls.iter().any(|im| {
                        im.trait_.tr == tref.tr && overlap(&im.for_, &im.domains, &for_, &domains)
                    }) {
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
                    // An impl lives with its trait or its type (R08.5.2).
                    let home = match &for_ {
                        Ty::Struct(sd, ..) => Some(d.s(*sd).module),
                        _ => None,
                    };
                    if !d.is_library(mid) && d.traits[tref.tr.0].module != mid && home != Some(mid)
                    {
                        d.misdeclared.push((
                            at,
                            cpl_diag::codes::UNSUPPORTED.0,
                            format!("`{tname}` and the type are both declared elsewhere: an impl lives with one of them"),
                        ));
                        continue;
                    }
                    let id = d.impls.len();
                    let mut methods = HashMap::new();
                    for m in &body.stmts {
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
                    d.impls.push(ImplDecl {
                        trait_: tref,
                        generics,
                        bounds: ibounds,
                        for_,
                        domains,
                        methods,
                        at,
                        module: mid,
                    });
                }
                (StmtKind::Fn(f), _) => {
                    let fid = add_fn(&mut d, mid, item, s, f, FREE);
                    let before = own[mi].insert(f.name.text.clone(), Named::Fn(fid));
                    d.views[mi].insert(f.name.text.clone(), Named::Fn(fid));
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
                            d.fns[fid.0].errors.push(e);
                            continue;
                        }
                        // Two with a `self` are one method's overloads unless one call could take
                        // both (R08.1.4); that test isn't built.
                        if d.f(first).self_ty.is_some() && d.f(fid).self_ty.is_some() {
                            d.errors.push(format!(
                                "slice: methods of one name in one module (`{}`)",
                                f.name.text
                            ));
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
                            d.errors.push(
                                "slice: fns of one name told apart by their labels".to_string(),
                            );
                        }
                        let code = cpl_diag::codes::OVERLOAD_BY_TYPE;
                        d.misdeclared.push((
                            (item + s.span.start) as usize,
                            code.0,
                            format!("`{}` is already a fn; one name is one fn", f.name.text),
                        ));
                    }
                }
                _ => {}
            }
        }
        if wait.is_empty() || wait.len() == before {
            break;
        }
        queue = wait;
    }
    let mut errors = Vec::new();
    d.views = (0..n)
        .map(|mi| {
            let e = if d.is_library(ModId(mi)) {
                None
            } else {
                Some(&mut errors)
            };
            view(
                &Scopes {
                    own: &own,
                    public: &public,
                    names: &names,
                    imports: &imports,
                    prelude,
                },
                mi,
                mi == program,
                e,
            )
        })
        .collect();
    d.errors.extend(errors);
    let unmet = supertraits_unmet(&d);
    d.misdeclared.extend(unmet);
    (d, main)
}

/// The item a method sits in: its type parameters and set parameters come first.
struct Within<'a> {
    owner: Option<Owner>,
    generics: &'a [String],
    sets: usize,
    /// The type of `self` in what it sits in, in `generics`.
    self_ty: Option<Ty>,
    /// For an impl's fn, the trait's signature for it in the impl's terms: what an
    /// untyped parameter or a missing result means (R08.1.2).
    sig: Option<TraitSig>,
}
struct TraitSig {
    self_mode: SelfMode,
    params: Vec<Ty>,
    ret: Ty,
}
const FREE: Within = Within {
    owner: None,
    sig: None,
    generics: &[],
    sets: 0,
    self_ty: None,
};

fn add_fn(
    d: &mut Decls,
    m: ModId,
    item: u32,
    s: &ast::Stmt,
    f: &ast::FnDecl,
    within: Within,
) -> FnId {
    let Within {
        owner,
        generics: outer,
        sets: outer_sets,
        self_ty: outer_self,
        sig,
    } = within;
    let mut generics = outer.to_vec();
    generics.extend(f.generics.iter().map(|g| g.text.clone()));
    let names = d.views[m.0].clone();
    let doms = d.doms_in(m);
    let aliases = d.aliases.clone();
    let sets = d.struct_sets.clone();
    let mut cx = TyCx {
        names: &names,
        struct_sets: &sets,
        generics: &generics,
        args: None,
        aliases: &aliases,
        expanding: Vec::new(),
        doms: &doms,
        ret_slots: None,
        next_set: Some(outer_sets),
    };
    let mut errors = Vec::new();
    let mut self_mode = SelfMode::None;
    let mut self_name = "self".to_string();
    let mut self_ty = None;
    let mut params = Vec::new();
    let sig_takes_self = sig.as_ref().is_some_and(|s| s.self_mode != SelfMode::None);
    for (k, p) in f.params.iter().enumerate() {
        if p.name.text == "self" || (k == 0 && sig_takes_self) {
            self_name = p.name.text.clone();
            self_mode = if p.mutable || sig.as_ref().is_some_and(|s| s.self_mode == SelfMode::Mut) {
                SelfMode::Mut
            } else {
                SelfMode::Value
            };
            // A free fn's `self` says what it extends (R08.1.4).
            self_ty = match (&outer_self, &p.ty) {
                (Some(t), _) => Some(t.clone()),
                (None, Some(t)) => Some(conv(&mut cx, t, &mut errors)),
                (None, None) => {
                    errors.push("`self` of a free fn needs a type".to_string());
                    None
                }
            };
            continue;
        }
        let from_trait = (sig.as_ref()).and_then(|s| s.params.get(params.len()).cloned());
        let ty = match (&p.ty, from_trait) {
            (Some(t), _) => conv(&mut cx, t, &mut errors),
            (None, Some(t)) => t,
            (None, None) => {
                errors.push(format!("parameter `{}` needs a type", p.name.text));
                Ty::Error
            }
        };
        if p.variadic || p.lazy {
            errors.push(format!(
                "slice: a variadic or lazy parameter `{}`",
                p.name.text
            ));
        }
        params.push(Param {
            name: p.name.text.clone(),
            label: p.label.as_ref().unwrap_or(&p.name).text.clone(),
            ty,
            mutable: p.mutable,
            named_only: p.named_only,
            default: p.default.clone(),
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
        (None, None) => sig.as_ref().map_or(Ty::Unit, |s| s.ret.clone()),
    };
    let n_ret_sets = cx.ret_slots.unwrap_or(0);
    let mut bounds = Vec::new();
    for b in &f.bounds {
        let Some(i) = generics.iter().position(|g| *g == b.name.text) else {
            continue;
        };
        for t in &b.traits {
            if let Some(r) = trait_ref(&mut cx, t, &mut errors) {
                bounds.push((i, d.with_defaults(r, &Ty::Param(i))));
            }
        }
    }
    // A decorator changes meaning, so one the slice does not implement refuses the program; it is
    // never skipped. A library's `@lint` declares a rewrite for the linter, which the slice does not
    // run, so the program's own is refused: its expected result may be that lint (R06.4.9).
    for x in &s.decorators {
        let elsewhere = d.is_library(m) && read_elsewhere(&x.name.text);
        let core_only = m == CORE && x.name.text == "coerce";
        if !elsewhere && !core_only && !matches!(x.name.text.as_str(), "get" | "multitest") {
            errors.push(format!("slice: decorator `@{}`", x.name.text));
        }
    }
    let mut errors = if d.is_library(m) {
        errors
    } else {
        d.errors.extend(errors);
        Vec::new()
    };
    let has = |n: &str| s.decorators.iter().any(|x| x.name.text == n);
    // A getter is read like a field, so nothing but `self` may need an argument (R02.6.6).
    let getter = has("get");
    let coerce = m == CORE && has("coerce");
    if coerce {
        d.coercions.push(FnId(d.fns.len()));
    }
    if getter && params.iter().any(|p| p.default.is_none()) {
        let msg = format!(
            "getter `{}` is read like a field, so each parameter but `self` needs a default",
            f.name.text
        );
        if d.is_library(m) {
            errors.push(msg);
        } else {
            let code = cpl_diag::codes::GETTER_NEEDS_DEFAULTS.0;
            d.misdeclared
                .push(((item + s.span.start) as usize, code, msg));
        }
    }
    d.fns.push(FnDecl {
        name: f.name.text.clone(),
        owner,
        self_mode,
        self_name,
        self_ty,
        generics,
        bounds,
        params,
        labels: f
            .params
            .iter()
            .filter(|p| p.named_only)
            .map(|p| p.label.as_ref().unwrap_or(&p.name).text.clone())
            .collect(),
        ret,
        n_sets,
        n_ret_sets,
        region,
        body: f.body.clone(),
        intrinsic: match &f.body {
            FnBody::Expr(e) => match &e.kind {
                cpl_syntax::ast::ExprKind::Intrinsic(n) => Some(n.clone()),
                _ => None,
            },
            _ => None,
        },
        getter,
        item,
        module: m,
        errors,
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

/// An enum as structs: its own, with no fields (the parser makes its methods an `impl`), then one
/// per variant holding that variant's payload, named `Enum.Variant`, which no source name can be
/// (R02.7).
fn enum_structs(s: &ast::Stmt, e: &ast::EnumDecl) -> (Vec<ast::Stmt>, Vec<String>) {
    let mut errors = Vec::new();
    let one = |name: String, fields, methods, decorators| ast::Stmt {
        kind: StmtKind::Struct(ast::StructDecl {
            name: ast::Name {
                text: name,
                span: e.name.span,
            },
            generics: e.generics.clone(),
            generic_defaults: vec![None; e.generics.len()],
            bounds: Vec::new(),
            fields,
            methods,
        }),
        span: s.span,
        decorators,
        public: s.public,
    };
    let mut out = vec![one(
        e.name.text.clone(),
        Vec::new(),
        Vec::new(),
        s.decorators.clone(),
    )];
    for v in &e.variants {
        if v.tag.is_some() || v.value.is_some() {
            errors.push(format!("slice: a tagged variant `{}`", v.name.text));
        }
        let fields = (v.fields.iter().enumerate())
            .map(|(i, (n, t))| ast::FieldDecl {
                name: n.clone().unwrap_or_else(|| ast::Name {
                    text: i.to_string(),
                    span: t.span,
                }),
                ty: t.clone(),
                default: None,
                mutable: false,
                cached: false,
                bits: false,
                public: true,
            })
            .collect();
        out.push(one(
            format!("{}.{}", e.name.text, v.name.text),
            fields,
            Vec::new(),
            Vec::new(),
        ));
    }
    (out, errors)
}

/// Per struct a module declares, the impls it gets structurally (R08.1.6), made after every written
/// impl and only where none is: `Eq` field by field and `Ord` lexicographic in declaration order.
fn module_derived(items: &[(u32, Vec<ast::Stmt>)]) -> Vec<(u32, String, ast::Stmt)> {
    let mut out = Vec::new();
    for (item, stmts) in items {
        for s in stmts {
            let StmtKind::Struct(sd) = &s.kind else {
                continue;
            };
            // The traits `==` and `<` call: the derivable ones.
            for op in [ast::CmpOp::Eq, ast::CmpOp::Lt] {
                let Some((tr, method)) = op.trait_method() else {
                    continue;
                };
                let derived = derived_impl(s, sd, op, tr, method);
                out.push((*item, sd.name.text.clone(), derived));
            }
        }
    }
    out
}

/// `impl<T: Eq> Eq for S<T>: fn eq(a, b) = a.x == b.x and ..`, or `Ord`'s `lt` over the fields in
/// order, bounding the parameters the fields mention and written at the struct.
fn derived_impl(
    s: &ast::Stmt,
    sd: &ast::StructDecl,
    op: ast::CmpOp,
    tr: &str,
    method: &str,
) -> ast::Stmt {
    use ast::{BinOp, CmpOp, ExprKind};
    let span = s.span;
    let name = |text: &str| ast::Name {
        text: text.to_string(),
        span,
    };
    let ty = |text: &str, args| ast::Type {
        kind: TypeKind::Named {
            name: name(text),
            args,
        },
        span,
    };
    let ex = |kind| ast::Expr { kind, span };
    let field = |side: &str, f: &str| {
        ex(ExprKind::Field {
            base: Box::new(ex(ExprKind::Name(side.to_string()))),
            name: name(f),
        })
    };
    let cmp = |op, f: &str| {
        ex(ExprKind::Compare {
            first: Box::new(field("a", f)),
            rest: vec![(op, field("b", f))],
        })
    };
    let bin = |op, lhs, rhs| {
        ex(ExprKind::Binary {
            op,
            op_span: span,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        })
    };
    // A `@cache` field is not part of the value (R05.10.13).
    let compared: Vec<&ast::FieldDecl> = sd.fields.iter().filter(|f| !f.cached).collect();
    let fields: Vec<&str> = compared.iter().map(|f| f.name.text.as_str()).collect();
    let body = match op {
        CmpOp::Eq => (fields.iter().map(|f| cmp(CmpOp::Eq, f)))
            .reduce(|a, b| bin(BinOp::And, a, b))
            .unwrap_or_else(|| ex(ExprKind::Bool(true))),
        _ => (fields.iter().rev())
            .fold(None, |rest, f| {
                Some(match rest {
                    None => cmp(CmpOp::Lt, f),
                    Some(r) => bin(
                        BinOp::Or,
                        cmp(CmpOp::Lt, f),
                        bin(BinOp::And, cmp(CmpOp::Eq, f), r),
                    ),
                })
            })
            .unwrap_or_else(|| ex(ExprKind::Bool(false))),
    };
    let mut mentioned = Vec::new();
    for f in &compared {
        unnamed(&f.ty, &HashMap::new(), &mut mentioned);
    }
    let param = |n: &str| ast::Param {
        mutable: false,
        variadic: false,
        lazy: false,
        named_only: false,
        label: None,
        name: name(n),
        ty: None,
        default: None,
        span,
    };
    let f = ast::FnDecl {
        name: name(method),
        generics: Vec::new(),
        bounds: Vec::new(),
        params: vec![param("a"), param("b")],
        ret: None,
        lends: None,
        uses: Vec::new(),
        body: FnBody::Expr(body),
        sig_span: span,
    };
    let stmt = |kind| ast::Stmt {
        kind,
        span,
        decorators: Vec::new(),
        public: false,
    };
    stmt(StmtKind::Impl {
        trait_name: Some(ty(tr, Vec::new())),
        target: ty(
            &sd.name.text,
            sd.generics
                .iter()
                .map(|g| ty(&g.text, Vec::new()))
                .collect(),
        ),
        bounds: (sd.generics.iter())
            .filter(|g| mentioned.contains(&g.text))
            .map(|g| ast::Bound {
                name: g.clone(),
                traits: vec![ty(tr, Vec::new())],
            })
            .collect(),
        body: ast::Block {
            stmts: vec![stmt(StmtKind::Fn(f))],
            span,
        },
    })
}

//! Contract 2: syntax -> declarations. Answers "what does this name mean"
//! without looking inside a body.
pub use crate::cycles::{alias_cycle, field_holding_itself};
use crate::diag::Msg;
use crate::synth::{enum_structs, module_derived};
use crate::ty::*;
use cpl_diag::Span;
use cpl_syntax::ast::{self, FnBody, StmtKind, TypeKind};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

mod impls;
mod inherent;
mod tyconv;
use impls::{overlap, same_shape, supertraits_unmet};
use inherent::inherent_impl;
use tyconv::*;
pub use tyconv::{body_type, prim_type};
pub(crate) use tyconv::{unnamed, GETTER};

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
    /// `@same_keys`: its arguments that are arrays over domains share one domain (R04b.1.3).
    pub same_keys: bool,
    /// `@aka(..)`: names another language calls it; a call by one names this fn (R06.4.11).
    pub aka: Vec<String>,
    /// `@was(..)`: names it had before a release renamed it; a call by one is E0503.
    pub was: Vec<String>,
    pub item: u32,
    /// Where it is written, in the joined source.
    pub span: Span,
    pub module: ModId,
    /// What its signature uses that the slice does not build, said only when a call reaches it.
    pub errors: Vec<Msg>,
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
    /// Where its declaration starts in the source: the spans of its defaults count from here.
    pub item: u32,
    pub n_sets: usize,
    pub methods: HashMap<String, FnId>,
    /// Its static `new`, which `T(args)` calls outside its own body (R02.6.3).
    pub ctor: Option<FnId>,
    /// An enum's variants, each with the struct of its payload, named `Enum.Variant` (R02.7).
    pub variants: Vec<(String, StructId)>,
    /// What its fields use that the slice does not build, said only when the program reaches it.
    pub errors: Vec<Msg>,
    /// An enum's `Int` of each variant: its written value (`Neg = -1`), else its place from 0.
    pub values: Vec<i64>,
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
    pub errors: Vec<Msg>,
    /// `trait Add<Rhs = Self>`: each parameter's default, in `Self` and the parameters before it.
    pub defaults: Vec<Option<Ty>>,
    /// `type Arc: Target<V>`: its associated types, its last parameters, which an impl writes
    /// in its body and a bound leaves out (R08.2.3).
    pub assoc: Vec<String>,
    /// The traits each associated type has, in `Self` and the trait's parameters.
    pub assoc_bounds: Vec<(usize, TraitRef)>,
}

impl TraitDecl {
    /// The arguments that choose an impl: all but the associated types, which the impl gives.
    pub fn inputs<'a, T>(&self, args: &'a [T]) -> &'a [T] {
        &args[..args.len().saturating_sub(self.assoc.len())]
    }
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
impl ImplDecl {
    /// For a bare type parameter (`impl<D: Domain> Eq for D`): any impl for a written type is
    /// more specific and is chosen over it (R08.1.2).
    pub fn blanket(&self) -> bool {
        matches!(self.for_, Ty::Param(_))
    }
}
pub struct ImplDecl {
    pub trait_: TraitRef,
    pub generics: Vec<String>,
    pub bounds: Vec<(usize, TraitRef)>,
    pub for_: Ty,
    pub methods: HashMap<String, FnId>,
    /// Where it is written.
    pub at: usize,
    /// Where it says each of its trait's associated types, or `at` for a default.
    pub assoc_at: Vec<usize>,
    pub module: ModId,
    /// Made from the fields (R08.1.6): its `S(..)` is the generated constructor, never a `new`.
    pub derived: bool,
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
    pub module_errors: Vec<Vec<Msg>>,
    /// Why each `Named::Refused` library item is not built.
    pub refused: Vec<String>,
    pub struct_sets: HashMap<StructId, usize>,
    /// `struct SegTree<D, T, F = Int>`: each struct's default type arguments, in its parameters.
    pub struct_defaults: HashMap<StructId, Vec<Option<Ty>>>,
    /// Core's `@tuple` structs by arity: `(A, B)` is the one with two fields.
    pub tuples: HashMap<usize, StructId>,
    pub aliases: Rc<Vec<AliasDecl>>,
    pub errors: Vec<Msg>,
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
    /// The trait core marks `@convert`: `T(x)` for a core type `T` is `x`'s impl of it at `T`.
    pub convert_trait: Option<TraitId>,
    /// Per fn, the type parameters that key an array it names: domain parameters (R04.2).
    pub domain_params: HashMap<FnId, Vec<usize>>,
    /// Core's `@coerce` fns: each converts a value of its parameter's type where its result's
    /// type is wanted (R04.8.10).
    pub coercions: Vec<FnId>,
    /// The `@index` fns, in declaration order.
    pub index_fns: Vec<FnId>,
}

/// Where a method call resolved to.
pub enum Member {
    Field(usize, Ty),
    /// A method, with the receiver's type and set arguments.
    Method(FnId, Vec<Ty>, Vec<SetTy>),
}

impl Decls {
    /// `Tagged<Str>` written in a body: its arguments, the defaults filling those left off.
    pub fn with_defaults_of(&self, d: StructId, mut given: Vec<Ty>) -> Vec<Ty> {
        if given.is_empty() {
            return given;
        }
        fill_defaults(self.struct_defaults.get(&d), &mut given);
        given
    }
    /// The domains a declaration in module `m` may name.
    fn doms_in(&self, m: ModId) -> HashMap<String, Ty> {
        match m == self.program {
            true => self.top_doms.clone(),
            false => HashMap::new(),
        }
    }
    /// What a declaration uses that the slice does not build: a library's is said when the program
    /// reaches it, the program's at once.
    fn module_or_program_errors(&mut self, m: ModId, errors: Vec<Msg>) {
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
}

fn field_names(fs: &[(ast::Name, ast::Type)]) -> Vec<&str> {
    fs.iter().map(|(n, _)| n.text.as_str()).collect()
}

fn struct_named(names: &HashMap<String, Named>, name: &str) -> Option<StructId> {
    match names.get(name) {
        Some(Named::Struct(s)) => Some(*s),
        _ => None,
    }
}

/// A type parameter that keys an array in a fn's signature, or in its owner's fields, is a domain
/// parameter (`struct Dsu<V>: parent: [V]V`): its body may use what every domain has (R04.2). A
/// caller is not held to `Domain`, since a list keyed by `Int` passes as `[Int]T`.
fn domain_params(d: &mut Decls) {
    let Some(array) = d.core_struct("Array") else {
        return;
    };
    // Also through a struct whose own parameter in that place is a domain: `self: Dsu<V>`.
    fn keys(d: &Decls, t: &Ty, array: StructId, i: usize, depth: usize) -> bool {
        let go = |t: &Ty| depth < 8 && keys(d, t, array, i, depth + 1);
        match t {
            Ty::Struct(s, ts, _) => {
                (*s == array && ts.first() == Some(&Ty::Param(i)))
                    || ts.iter().any(go)
                    || (ts.iter().enumerate()).any(|(k, a)| {
                        *a == Ty::Param(i)
                            && depth < 8
                            && (d.structs[s.0].fields.iter())
                                .any(|(_, f)| keys(d, f, array, k, depth + 1))
                    })
            }
            Ty::Raw(e) | Ty::Ref(e) => go(e),
            Ty::Fn(ps, r, _) => ps.iter().chain([&**r]).any(go),
            _ => false,
        }
    }
    for k in 0..d.fns.len() {
        let f = &d.fns[k];
        let fields: &[(String, Ty)] = match f.owner {
            Some(Owner::Struct(s)) => &d.structs[s.0].fields,
            _ => &[],
        };
        let added: Vec<usize> = (0..f.generics.len())
            .filter(|&i| {
                (f.params.iter().map(|p| &p.ty))
                    .chain([&f.ret])
                    .chain(&f.self_ty)
                    .chain(fields.iter().map(|(_, t)| t))
                    .any(|t| keys(d, t, array, i, 0))
            })
            .collect();
        if !added.is_empty() {
            d.domain_params.insert(FnId(k), added);
        }
    }
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
    errors: Option<&mut Vec<Msg>>,
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
    mut errors: Option<&mut Vec<Msg>>,
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
    let mut imported: HashMap<String, Named> = HashMap::new();
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
                e.push(format!("slice: `use {module}` without names").into());
            }
            continue;
        }
        for n in ns {
            match export(t, &n.text) {
                Some(_) if own[mi].contains_key(&n.text) => {
                    if let Some(e) = errors.as_deref_mut() {
                        e.push(
                            format!(
                                "slice: `use` of `{}`, which this file also declares",
                                n.text
                            )
                            .into(),
                        );
                    }
                }
                // Two imports of one name stay ambiguous where it is used (R08.1.4).
                Some(x) if imported.get(&n.text).is_some_and(|y| *y != x) => {
                    if let Some(e) = errors.as_deref_mut() {
                        let text = format!("slice: two `use` lines import `{}`", n.text);
                        e.push(text.into());
                    }
                }
                Some(x) => {
                    v.insert(n.text.clone(), x);
                    imported.insert(n.text.clone(), x);
                }
                None => {
                    if let Some(e) = errors.as_deref_mut() {
                        e.push(
                            format!(
                                "slice: `{module}` has no pub `{}` the slice declares",
                                n.text
                            )
                            .into(),
                        );
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
    let mut enums: Vec<(usize, String, Vec<String>, Vec<i64>)> = Vec::new();
    // Every record shape the program and its modules write is a core struct (R02.5.2).
    let shapes = crate::records::shapes(mods);
    let core_first = mods[CORE.0].items.first();
    let records: Vec<(u32, Vec<ast::Stmt>)> = (core_first
        .and_then(|(i, ss)| Some((*i, ss.first()?.span))))
    .map(|(i, span)| vec![(i, crate::records::structs(&shapes, span))])
    .unwrap_or_default();
    let each = crate::synth::derive_each(&mods[CORE.0].items);
    let convert = crate::synth::convert_marked(&mods[CORE.0].items);
    let derived: Vec<Vec<(u32, String, ast::Stmt)>> = (mods.iter().enumerate())
        .map(|(mi, m)| {
            let mut made = match mi == CORE.0 {
                true => module_derived(&records, &each),
                false => module_derived(&m.items, &each),
            };
            made.extend(crate::synth::field_trait_impls(&m.items, &shapes));
            if let Some(c) = &convert {
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
            for (item, stmts) in &m.items {
                for s in stmts {
                    if let StmtKind::Enum(e) = &s.kind {
                        let (structs, errors, values) = enum_structs(s, e);
                        match mi == n - 1 {
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
                        if mi == CORE.0 && s.decorators.iter().any(|x| x.name.text == "tuple") {
                            d.tuples.insert(sd.fields.len(), id);
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
                            ctor: None,
                            variants: Vec::new(),
                            errors: Vec::new(),
                            values: Vec::new(),
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
            let mut cx = TyCx {
                names: &names,
                struct_sets: &sets,
                struct_defaults: &defaults,
                tuples: &tuples,
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
        let mut cx = TyCx {
            names: &names,
            struct_sets: &sets,
            struct_defaults: &defaults,
            tuples: &tuples,
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
        d.struct_defaults.insert(*id, tys);
        if d.is_library(*mid) {
            d.structs[id.0].errors.extend(errors);
        } else {
            d.errors.extend(errors);
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
                    let defaults = d.struct_defaults.clone();
                    let tuples = d.tuples.clone();
                    let mut cx = TyCx {
                        names: &d.views[mi].clone(),
                        struct_sets: &sets,
                        struct_defaults: &defaults,
                        tuples: &tuples,
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
                    let doms = d.doms_in(ModId(mi));
                    let mut cx = TyCx {
                        names: &names,
                        struct_sets: &sets,
                        struct_defaults: &defaults,
                        tuples: &tuples,
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
                    let doms = d.doms_in(ModId(mi));
                    let mut errors = Vec::new();
                    let mut cx = TyCx {
                        names: &names,
                        struct_sets: &sets,
                        struct_defaults: &defaults,
                        tuples: &tuples,
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
                    if d.impls.iter().any(|im| {
                        im.trait_.tr == tref.tr
                            && overlap(&im.for_, &for_)
                            && (d.traits[tref.tr.0].inputs(&im.trait_.args).iter())
                                .zip(d.traits[tref.tr.0].inputs(&tref.args))
                                .all(|(x, y)| same_shape(x, y))
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
                    // An impl lives with its trait, its type or a type among the trait's arguments
                    // (`impl Into<E> for Int`), so it is never an orphan (R08.5.2).
                    let local = |t: &Ty| matches!(t, Ty::Struct(sd, ..) if d.s(*sd).module == mid);
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
                    d.impls.push(ImplDecl {
                        trait_: tref,
                        generics,
                        bounds: ibounds,
                        for_,
                        methods,
                        at,
                        assoc_at,
                        module: mid,
                        derived,
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
                    let TypeKind::Named { name, args } = &target.kind else {
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
                    inherent_impl(&mut d, mid, item, t, args, bounds, body);
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
    domain_params(&mut d);
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

/// A parameter typed by a trait (`b: Big<Int>`) has a type parameter of its own bounded by it, as
/// `<B: Big<Int>>` would be (R08.2.1); its name `b.impl` can't be written.
fn trait_typed(
    f: &ast::FnDecl,
    outer: &[String],
    names: &HashMap<String, Named>,
) -> Option<ast::FnDecl> {
    let mut out: Option<ast::FnDecl> = None;
    for (k, p) in f.params.iter().enumerate() {
        let Some(t) = &p.ty else { continue };
        let TypeKind::Named { name, .. } = &t.kind else {
            continue;
        };
        let shadowed = outer.contains(&name.text) || f.generics.iter().any(|g| g.text == name.text);
        if shadowed || !matches!(names.get(&name.text), Some(Named::Trait(_))) {
            continue;
        }
        let g = out.get_or_insert_with(|| f.clone());
        let n = ast::Name {
            text: format!("{}.impl", p.name.text),
            span: name.span,
        };
        g.generics.push(n.clone());
        g.bounds.push(ast::Bound {
            name: n.clone(),
            traits: vec![t.clone()],
        });
        g.params[k].ty = Some(ast::Type {
            kind: TypeKind::Named {
                name: n,
                args: Vec::new(),
            },
            span: t.span,
        });
    }
    out
}

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
    let names = d.views[m.0].clone();
    let desugared = trait_typed(f, outer, &names);
    let f = desugared.as_ref().unwrap_or(f);
    let mut generics = outer.to_vec();
    generics.extend(f.generics.iter().map(|g| g.text.clone()));
    // A bound that leaves a trait's arguments out (`S: Indexed`) takes each as a parameter of
    // its own, fixed when the bound is met at a call (R08.1.2). Its name `S.1` can't be written.
    let mut hidden = Vec::new();
    for b in &f.bounds {
        for (j, t) in b.traits.iter().enumerate() {
            let TypeKind::Named { name, args } = &t.kind else {
                continue;
            };
            let Some(Named::Trait(tr)) = names.get(&name.text).copied() else {
                continue;
            };
            let n = d.traits[tr.0].defaults.len();
            if args.len() < n && d.traits[tr.0].defaults[args.len()].is_none() {
                let k = generics.len();
                // An associated type the bound leaves out is named through the bounded
                // parameter: `G.Arc` of `G: Next<V>` (R08.2.3).
                let assoc = &d.traits[tr.0].assoc;
                generics.extend(
                    (args.len()..n).map(|p| match p.checked_sub(n - assoc.len()) {
                        Some(a) => format!("{}.{}", b.name.text, assoc[a]),
                        None => format!("{}.hidden", k + p - args.len()),
                    }),
                );
                hidden.push(((b.name.text.clone(), j), k..generics.len()));
            }
        }
    }
    let doms = d.doms_in(m);
    let aliases = d.aliases.clone();
    let sets = d.struct_sets.clone();
    let defaults = d.struct_defaults.clone();
    let tuples = d.tuples.clone();
    let mut cx = TyCx {
        names: &names,
        struct_sets: &sets,
        struct_defaults: &defaults,
        tuples: &tuples,
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
                    errors.push(Msg::at(
                        Span::new(item + p.name.span.start, item + p.name.span.end),
                        format!(
                            "{}: `self` of a free fn needs a type",
                            cpl_diag::codes::UNKNOWN_TYPE.0
                        ),
                    ));
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
                errors.push(Msg::at(
                    Span::new(item + p.name.span.start, item + p.name.span.end),
                    format!(
                        "{}: parameter `{}` needs a type",
                        cpl_diag::codes::UNKNOWN_TYPE.0,
                        p.name.text
                    ),
                ));
                Ty::Error
            }
        };
        if p.variadic || p.lazy {
            errors.push(format!("slice: a variadic or lazy parameter `{}`", p.name.text).into());
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
        for (j, t) in b.traits.iter().enumerate() {
            if let Some(mut r) = trait_ref(&mut cx, t, &mut errors) {
                let key = (b.name.text.clone(), j);
                if let Some((_, ks)) = hidden.iter().find(|(h, _)| *h == key) {
                    r.args.extend(ks.clone().map(Ty::Param));
                }
                let r = d.with_defaults(r, &Ty::Param(i));
                // What the trait says of its associated types holds of the parameters naming them.
                let mut tys = vec![Ty::Param(i)];
                tys.extend(r.args.iter().cloned());
                let sub = Subst {
                    tys: &tys,
                    sets: &[],
                };
                for (a, ab) in &d.traits[r.tr.0].assoc_bounds {
                    if let Ty::Param(ai) = tys[*a] {
                        let ab = TraitRef {
                            tr: ab.tr,
                            args: ab.args.iter().map(|t| sub.ty(t)).collect(),
                        };
                        bounds.push((ai, ab));
                    }
                }
                bounds.push((i, r));
            }
        }
    }
    // A decorator changes meaning, so one the slice does not implement refuses the program; it is
    // never skipped. A library's `@lint` declares a rewrite for the linter, which the slice does not
    // run, so the program's own is refused: its expected result may be that lint (R06.4.9).
    for x in &s.decorators {
        let elsewhere = d.is_library(m) && read_elsewhere(&x.name.text);
        let core_only = m == CORE && matches!(x.name.text.as_str(), "coerce" | "variant_check");
        if !elsewhere
            && !core_only
            && x.name.text != GETTER
            && !matches!(
                x.name.text.as_str(),
                "multitest" | "same_keys" | "index" | "aka" | "was"
            )
        {
            let at = Span::new(item + x.span.start, item + x.span.end);
            errors.push(Msg::at(at, format!("slice: decorator `@{}`", x.name.text)));
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
    let getter = has(GETTER);
    let same_keys = has("same_keys");
    let words = |n: &str| -> Vec<String> {
        (s.decorators.iter().filter(|x| x.name.text == n))
            .flat_map(|x| &x.args)
            .filter_map(|a| match &a.value.kind {
                cpl_syntax::ast::ExprKind::Name(w) => Some(w.clone()),
                _ => None,
            })
            .collect()
    };
    if has("index") {
        d.index_fns.push(FnId(d.fns.len()));
    }
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
            errors.push(msg.into());
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
        same_keys,
        aka: words("aka"),
        was: words("was"),
        item,
        span: Span::new(item + s.span.start, item + s.span.end),
        module: m,
        errors,
    });
    FnId(d.fns.len() - 1)
}

/// A whole-number literal, maybe negated or written with an exponent (`1e9`).
pub fn constant_int(e: &ast::Expr) -> Option<i64> {
    match &e.kind {
        ast::ExprKind::Int(i) => i64::try_from(*i).ok(),
        ast::ExprKind::Paren(x) => constant_int(x),
        ast::ExprKind::Unary {
            op: ast::UnOp::Neg,
            expr,
        } => constant_int(expr)?.checked_neg(),
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

/// The ends of a range type when both are constants, as an inclusive pair.
pub fn static_range(
    lo: &Option<Box<ast::Expr>>,
    hi: &Option<Box<ast::Expr>>,
    inclusive: bool,
) -> Option<(i64, i64)> {
    let lo = match lo {
        Some(e) => constant_int(e)?,
        None => i64::MIN + 1,
    };
    let hi = match hi {
        Some(e) if inclusive => constant_int(e)?,
        Some(e) => constant_int(e)?.checked_sub(1)?,
        None => i64::MAX,
    };
    Some((lo, hi))
}

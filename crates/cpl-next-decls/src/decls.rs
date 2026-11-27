//! Contract 2: syntax -> declarations. Answers "what does this name mean"
//! without looking inside a body.
pub use crate::cycles::{alias_cycle, field_holding_itself};
use crate::diag::Msg;
use crate::ty::*;
use cpl_diag::Span;
use cpl_syntax::ast::{self, FnBody, StmtKind, TypeKind};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

mod collect;
mod impls;
mod inherent;
mod layer;
mod tyconv;
pub use collect::{collect, prepare, Prepared};
pub use layer::Layer;
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

#[derive(Clone)]
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
    /// A read `lends T` result: the parameters it borrows, receiver first (R05.6.1).
    pub lends: Option<Vec<usize>>,
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
    /// Where it is written, in the offset space the loaded modules are placed in.
    pub span: Span,
    pub module: ModId,
    /// When the collector made it: the round, then the pass (traits, fns and types, impls,
    /// derived impls). Std's and a program's together are checked in this order.
    pub made: (u32, u8),
    /// What its signature uses that the slice does not build, said only when a call reaches it.
    pub errors: Vec<Msg>,
}
#[derive(Clone)]
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
#[derive(Clone)]
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
    /// The fns of its impls over a shape of it (`impl Rev<[D]T>:`), each `self` that shape.
    pub shaped: HashMap<String, Vec<FnId>>,
    /// Its static `new`, which `T(args)` calls outside its own body (R02.6.3).
    pub ctor: Option<FnId>,
    /// An enum's variants, each with the struct of its payload, named `Enum.Variant` (R02.7).
    pub variants: Vec<(String, StructId)>,
    /// What its fields use that the slice does not build, said only when the program reaches it.
    pub errors: Vec<Msg>,
    /// An enum's `Int` of each variant: its written value (`Neg = -1`), else its place from 0.
    pub values: Vec<i64>,
    /// The traits its parameters are bounded by (`struct Range<D: Domain>`).
    pub bounds: Vec<(usize, TraitId)>,
}
#[derive(Clone)]
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
pub struct ConstId(pub usize);

/// A library's `const NAME[: T] = init`.
#[derive(Clone, Debug)]
pub struct LibConst {
    pub module: ModId,
    pub name: String,
    pub item: u32,
    pub ty: Option<ast::Type>,
    pub init: ast::Expr,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AliasId(pub usize);
/// `type Name<G> = T`: `T` is read in its module's names, where `Name` itself means what it means
/// outside (R02.6.2).
#[derive(Clone)]
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
#[derive(Clone)]
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
    /// A std-derived impl a program's written one replaces (R08.1.6): no lookup finds it.
    pub replaced: bool,
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
    /// A library's `const`: a global its initializer sets once, at program start (R01.6.1).
    Const(ConstId),
}
/// What a method sits in.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Owner {
    Struct(StructId),
    Trait(TraitId),
    /// An index into `Decls::impls`.
    Impl(usize),
}

#[derive(Default, Clone)]
pub struct Decls {
    pub fns: Layer<FnDecl>,
    pub structs: Layer<StructDecl>,
    pub traits: Layer<TraitDecl>,
    pub impls: Layer<ImplDecl>,
    /// Per module, what each name means there: the core's names, then the prelude's (the
    /// program only, R11.3.1), then its own and what its `use` lines import (R11.2.3).
    pub views: Layer<Arc<HashMap<String, Named>>>,
    /// Per module, the names two of its `use` lines import as different items: a use of one
    /// is ambiguous (R08.1.4).
    pub ambiguous: Vec<HashSet<String>>,
    pub module_names: Vec<String>,
    pub program: ModId,
    /// Per library module, what it uses outside its fns that the slice does not build.
    pub module_errors: Vec<Vec<Msg>>,
    /// Why each `Named::Refused` library item is not built.
    pub refused: Vec<String>,
    pub consts: Layer<LibConst>,
    pub struct_sets: Arc<HashMap<StructId, usize>>,
    /// `struct SegTree<D, T, F = Int>`: each struct's default type arguments, in its parameters.
    pub struct_defaults: Arc<HashMap<StructId, Vec<Option<Ty>>>>,
    /// Core's `@tuple` structs by arity: `(A, B)` is the one with two fields.
    pub tuples: Arc<HashMap<usize, StructId>>,
    /// A library's `@recipe` struct: the value of a `-> yields T` call made outside a `for`
    /// (R07.6.2). Its one field is the closure that walks the call.
    pub recipe: Option<StructId>,
    /// Core's `@member(m)` structs by `m`: `D.m` is the struct at `D` (R06).
    pub member_types: Arc<HashMap<String, StructId>>,
    /// The struct core marks `@cuts`: a half-open literal range of its values, where a
    /// `Range<D>` is wanted, is the piece of positions between them (R04.4.4).
    pub cut_struct: Option<StructId>,
    pub aliases: Layer<AliasDecl>,
    pub errors: Vec<Msg>,
    /// Declarations the program gets wrong: where, the code, what is wrong.
    pub misdeclared: Vec<(usize, &'static str, String)>,
    /// Each library's public free fns with a `self`, by name: a method call finds them with no
    /// import (R11.3.4).
    pub methods: HashMap<String, Vec<FnId>>,
    /// The program's top-level domains, which its declarations may name: each is the
    /// `Ty::Dom` of its place among them, filled when its statement runs (R02.9).
    pub top_doms: HashMap<String, Ty>,
    /// The names the program's top level binds with `let`.
    pub top_values: Arc<HashSet<String>>,
    /// The trait core marks `@positions`: every domain's position type has it, and nothing else.
    pub domain_trait: Option<TraitId>,
    /// Core's `@operator("+")` traits by the operator they give: `a + b` on a type that is not
    /// a number is its impl's fn (R08.1.5).
    pub operators: Vec<(String, TraitId)>,
    /// The trait core marks `@convert`: `T(x)` for a core type `T` is `x`'s impl of it at `T`.
    pub convert_trait: Option<TraitId>,
    /// Per fn, the type parameters that key an array it names: domain parameters (R04.2).
    pub domain_params: HashMap<FnId, Vec<usize>>,
    /// Core's `@coerce` fns: each converts a value of its parameter's type where its result's
    /// type is wanted (R04.8.10).
    pub coercions: Vec<FnId>,
    /// Static fns marked `@from_op`: what a bare operator becomes where their struct is wanted (R08.1.9).
    pub from_op: Vec<FnId>,
    /// Core's `@pattern` fns: `f(p)` as a pattern matches the optional `f` makes when it is
    /// present and its value matches `p` (R01.7.1).
    pub pattern_fns: Vec<FnId>,
    /// The `@index` fns, in declaration order.
    pub index_fns: Vec<FnId>,
    /// Per module, whether the program loads it: std's others are declared but out of sight.
    pub seen: Vec<bool>,
    /// The records std writes and no module the program loads does: out of sight with their impls.
    pub unseen_records: HashSet<StructId>,
    /// Each trait's impls, and each member name's (its trait's or its own), in order.
    pub impls_by_trait: HashMap<TraitId, Vec<usize>>,
    pub impls_by_member: HashMap<String, Vec<usize>>,
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
    /// The fn a bare operator names: its core `@operator` trait's fn of that many operands.
    pub fn operator_fn(&self, text: &str, binary: bool) -> Option<FnId> {
        (self.operators.iter().filter(|(o, _)| o == text)).find_map(|(_, tr)| {
            let m = *self.traits[tr.0].methods.values().next()?;
            (self.f(m).params.len() == usize::from(binary)).then_some(m)
        })
    }
    /// Each binary operator's fn, for `type_of(+)`.
    pub(crate) fn operator_fns(&self) -> Arc<Vec<(String, FnId)>> {
        Arc::new(
            (self.operators.iter())
                .filter_map(|(o, _)| Some((o.clone(), self.operator_fn(o, true)?)))
                .collect(),
        )
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
    /// Whether the program loads module `m`.
    pub fn seen(&self, m: ModId) -> bool {
        self.seen.get(m.0).copied().unwrap_or(true)
    }
    /// Whether a lookup may find impl `im`: its module is loaded and no written impl replaces it.
    pub fn live(&self, im: &ImplDecl) -> bool {
        !im.replaced
            && self.seen(im.module)
            && !matches!(im.for_, Ty::Struct(s, ..) if self.unseen_records.contains(&s))
    }
    /// The impls of `tr` a lookup may find, in order.
    pub fn trait_impls(&self, tr: TraitId) -> impl Iterator<Item = (usize, &ImplDecl)> + Clone {
        (self.impls_by_trait.get(&tr).into_iter().flatten())
            .map(|&k| (k, &self.impls[k]))
            .filter(|(_, im)| self.live(im))
    }
    fn push_impl(&mut self, im: ImplDecl) {
        let k = self.impls.len();
        self.impls_by_trait.entry(im.trait_.tr).or_default().push(k);
        let mut names: Vec<&String> = im.methods.keys().collect();
        names.extend(self.traits[im.trait_.tr.0].methods.keys());
        names.sort();
        names.dedup();
        for n in names {
            self.impls_by_member.entry(n.clone()).or_default().push(k);
        }
        self.impls.push(im);
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

/// A type parameter in a place whose struct bounds it by `Domain`, in a fn's signature or in its
/// owner's fields, is a domain parameter (`struct Dsu<V>: parent: [V]V`, `Range<D>`): its body
/// may use what every domain has (R04.2). So is one passed where a trait bound's parameter is
/// one (`S: Indexed<D, T>`, whose `keys` is a `Range<D>`). A caller is not held to `Domain`,
/// since a list keyed by `Int` passes as `[Int]T`.
fn domain_params(d: &mut Decls, from: usize) {
    let Some(domain) = d.domain_trait else {
        return;
    };
    // Also through a struct whose own parameter in that place is a domain: `self: Dsu<V>`.
    fn keys(d: &Decls, t: &Ty, domain: TraitId, i: usize, depth: usize) -> bool {
        let go = |t: &Ty| depth < 8 && keys(d, t, domain, i, depth + 1);
        match t {
            Ty::Struct(s, ts, _) => {
                ts.iter().any(go)
                    || (ts.iter().enumerate()).any(|(k, a)| {
                        *a == Ty::Param(i)
                            && (d.structs[s.0].bounds.contains(&(k, domain))
                                || (depth < 8
                                    && (d.structs[s.0].fields.iter())
                                        .any(|(_, f)| keys(d, f, domain, k, depth + 1))))
                    })
            }
            Ty::Raw(e) | Ty::Ref(e) => go(e),
            Ty::Fn(ps, r, _) => ps.iter().chain([&**r]).any(go),
            _ => false,
        }
    }
    // A parameter whose body names one of `Domain`'s fns on it (`D.nth(k)`) is one (R04.4.30).
    struct DomUse<'a> {
        generics: &'a [String],
        fns: Vec<&'a String>,
        found: Vec<usize>,
    }
    impl cpl_syntax::visit::Visitor for DomUse<'_> {
        fn expr(&mut self, e: &ast::Expr) {
            let ast::ExprKind::Field { base, name } = &e.kind else {
                return;
            };
            let ast::ExprKind::Name(n) = &base.kind else {
                return;
            };
            let i = self.generics.iter().position(|g| g == n);
            if let Some(i) = i.filter(|i| self.fns.contains(&&name.text) && !self.found.contains(i))
            {
                self.found.push(i);
            }
        }
    }
    let dom_fns: Vec<&String> = d.traits[domain.0].methods.keys().collect();
    let used: Vec<Vec<usize>> = (d.fns.iter().skip(from))
        .map(|f| {
            let mut v = DomUse {
                generics: &f.generics,
                fns: dom_fns.clone(),
                found: Vec::new(),
            };
            match &f.body {
                FnBody::Block(b) => cpl_syntax::visit::walk_block(b, &mut v),
                FnBody::Expr(e) => cpl_syntax::visit::walk_expr(e, &mut v),
                FnBody::None => {}
            }
            v.found
        })
        .collect();
    for k in from..d.fns.len() {
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
                    .any(|t| keys(d, t, domain, i, 0))
                    || used[k - from].contains(&i)
            })
            .collect();
        if !added.is_empty() {
            d.domain_params.insert(FnId(k), added);
        }
    }
    // A struct's parameter that one of its methods treats as a domain is one in all of them.
    let mut of_struct: HashMap<StructId, Vec<usize>> = HashMap::new();
    for (f, ds) in &d.domain_params {
        if let Some(Owner::Struct(s)) = d.fns[f.0].owner {
            let n = d.structs[s.0].generics.len();
            let js = of_struct.entry(s).or_default();
            js.extend(
                ds.iter()
                    .filter(|&&j| j < n && !js.contains(&j))
                    .collect::<Vec<_>>(),
            );
        }
    }
    for k in from..d.fns.len() {
        let Some(Owner::Struct(s)) = d.fns[k].owner else {
            continue;
        };
        let Some(js) = of_struct.get(&s) else {
            continue;
        };
        let ds = d.domain_params.entry(FnId(k)).or_default();
        for &j in js {
            if !ds.contains(&j) {
                ds.push(j);
            }
        }
    }
    // Through bounds, until no trait's or fn's domain parameters grow.
    loop {
        let trait_dom = |d: &Decls, tr: TraitId, p: usize| {
            (d.traits[tr.0].methods.values()).any(|m| {
                d.domain_params
                    .get(m)
                    .is_some_and(|ds| ds.contains(&(p + 1)))
            })
        };
        let mut grew = false;
        for k in 0..d.fns.len() {
            let more: Vec<usize> = (d.fns[k].bounds.iter())
                .flat_map(|(_, r)| {
                    (r.args.iter().enumerate()).filter_map(move |(p, a)| match a {
                        Ty::Param(j) => Some((r.tr, p, *j)),
                        _ => None,
                    })
                })
                .filter(|&(tr, p, _)| trait_dom(d, tr, p))
                .map(|(_, _, j)| j)
                .collect();
            let ds = d.domain_params.entry(FnId(k)).or_default();
            for j in more {
                if !ds.contains(&j) {
                    ds.push(j);
                    grew = true;
                }
            }
        }
        if !grew {
            break;
        }
    }
    d.domain_params.retain(|_, ds| !ds.is_empty());
}

/// The name a top-level item declares, if it declares one.
pub fn item_name(s: &ast::Stmt) -> Option<&str> {
    match &s.kind {
        StmtKind::Context { name, .. } => Some(&name.text),
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
    own: &'a Layer<HashMap<String, Named>>,
    public: &'a Layer<HashSet<String>>,
    names: &'a [String],
    imports: &'a [(ModId, &'a ast::Stmt)],
    prelude: &'a HashMap<String, String>,
}

fn view(
    sc: &Scopes,
    mi: usize,
    program: bool,
    errors: Option<&mut Vec<Msg>>,
    ambiguous: &mut HashSet<String>,
) -> HashMap<String, Named> {
    let mut v = outer_view(sc, mi, program, errors, Some(ambiguous));
    v.extend(sc.own[mi].iter().map(|(n, x)| (n.clone(), *x)));
    v
}

/// What each name means in module `mi` apart from what it declares itself.
fn outer_view(
    sc: &Scopes,
    mi: usize,
    program: bool,
    mut errors: Option<&mut Vec<Msg>>,
    mut ambiguous: Option<&mut HashSet<String>>,
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
                    v.remove(&n.text);
                    if let Some(a) = ambiguous.as_deref_mut() {
                        a.insert(n.text.clone());
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
            // An associated type's default is the impl's to take, never the bound's.
            let first_assoc = n - d.traits[tr.0].assoc.len();
            if args.len() < n
                && (args.len() >= first_assoc || d.traits[tr.0].defaults[args.len()].is_none())
            {
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
    let members = d.member_types.clone();
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
    let mut lends = None;
    let ret = match (&f.ret, &f.lends) {
        (_, Some(l)) if l.mutable => {
            region = Some((true, conv(&mut cx, &l.ty, &mut errors)));
            Ty::Unit
        }
        // A read lend is the fn's value, which borrows every source it names, or every parameter.
        (_, Some(l)) => {
            let receiver = (self_mode != SelfMode::None).then_some(self_name.as_str());
            let names: Vec<&str> = (receiver.into_iter())
                .chain(params.iter().map(|p| p.name.as_str()))
                .collect();
            let mut from = Vec::new();
            for s in &l.sources {
                match names.iter().position(|n| *n == s.text) {
                    Some(i) => from.push(i),
                    None => errors.push(Msg::at(
                        Span::new(item + s.span.start, item + s.span.end),
                        format!(
                            "{}: `{}` is not a parameter",
                            cpl_diag::codes::UNRESOLVED_NAME.0,
                            s.text
                        ),
                    )),
                }
            }
            if l.sources.is_empty() {
                from = (0..names.len()).collect();
            }
            lends = Some(from);
            conv(&mut cx, &l.ty, &mut errors)
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
        let core_only =
            m == CORE && matches!(x.name.text.as_str(), "coerce" | "variant_check" | "pattern");
        if !elsewhere
            && !core_only
            && x.name.text != GETTER
            && !matches!(
                x.name.text.as_str(),
                "multitest" | "same_keys" | "index" | "aka" | "was" | "from_op"
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
    if has("from_op") {
        d.from_op.push(FnId(d.fns.len()));
    }
    let coerce = m == CORE && has("coerce");
    if coerce {
        d.coercions.push(FnId(d.fns.len()));
    }
    if m == CORE && has("pattern") {
        d.pattern_fns.push(FnId(d.fns.len()));
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
        made: (0, 0),
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
        lends,
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

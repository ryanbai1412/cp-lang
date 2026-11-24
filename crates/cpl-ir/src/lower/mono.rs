//! Multi-module collection, generic monomorphization, traits/impls,
//! closures, `lends` projections, and `laws_hold`.

use super::*;
use std::collections::HashSet;
mod bounds;
mod closure;
mod gen_args;
mod gen_call;
mod instantiate;
mod laws;
mod probe;
mod projection;

pub(super) type ModId = usize;
pub(super) type GenStructId = usize;
pub(super) type GenEnumId = usize;
pub(super) type GenAliasId = usize;
pub(super) type GenFnId = usize;
pub(super) type GenConstId = usize;
pub(super) type ModConstId = usize;
pub(super) type TraitId = usize;

/// A bare binary operator or stdlib combinator passed as a value (`+`, `*`, `max`, `min`, `gcd`, `lcm`, etc.).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) enum BareOp {
    Add,
    Mul,
    BitXor,
    BitOr,
    BitAnd,
    Max,
    Min,
    Gcd,
    Lcm,
    And,
    Or,
}

/// One concrete argument to a generic type, struct, function, or const.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(super) enum GenArg {
    Ty(Ty),
    Domain(DomainId),
    Int(i64),
    RuntimeInt(GlobalId),
    Op(BareOp),
    Range(Option<i64>, Option<i64>),
    /// `?lo..hi`: an optional of a range, so a field `List<?F>` keeps `F`'s bounds.
    OptRange(Option<i64>, Option<i64>),
    /// `[3]Int` given to a `T: Read`: the type, and the lengths its
    /// `read<T>()` reads, which the type itself does not keep.
    Sized(Ty, Vec<i64>),
}

pub(super) type GenEnv = HashMap<String, GenArg>;

/// What a memoized instance or view copy is made from. Every kind shares one
/// cache, so cpl.wasm carries one hash table for them, not one per key shape.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) enum InstKey {
    Struct(GenStructId, Vec<GenArg>),
    Enum(GenEnumId, Vec<GenArg>),
    /// The fn, its args with the range ends, its reversed params, and
    /// whether it reads 1-based.
    Fn(GenFnId, Vec<GenArg>, Vec<String>, bool),
    /// The struct, its impl, the method and the impl's open params.
    OpenImpl(StructId, usize, String, Vec<GenArg>),
    /// The fn and the params it takes as views.
    View(FnId, Vec<(usize, Ty)>),
}

/// The env entry saying type parameter `k` is the `Int` key of an indexed
/// parameter, so `[k]T` is a table keyed like that parameter.
pub(super) fn int_key_marker(k: &str) -> String {
    format!("__int_key_{k}")
}

/// The env entry holding the ends of the range generic `k`'s `Int` was inferred from: `T` of
/// `fold` over a `List<1..=9>` is `Int`, so its values widen as a range's do (R03.1.7), while
/// `T.MIN`/`T.MAX`/`T.count` read `1..=9` (decision 544).
pub(super) fn range_ends_marker(k: &str) -> String {
    format!("__range_ends_{k}")
}

/// Marks a fn's own generic `k`: given `Int`, its `[k]T` is the array it was called with,
/// where a struct's `D = Int` makes `[D]T` a list (R07.3.5).
pub(super) fn own_generic_marker(k: &str) -> String {
    format!("__own_generic_{k}")
}

pub(super) struct GenStructTemplate<'a> {
    pub mod_id: ModId,
    pub offset: u32,
    pub decl: &'a ast::StructDecl,
}

pub(super) struct GenEnumTemplate<'a> {
    pub mod_id: ModId,
    pub offset: u32,
    pub decl: &'a ast::EnumDecl,
}

pub(super) struct GenAliasTemplate<'a> {
    pub mod_id: ModId,
    pub offset: u32,
    pub generics: &'a [ast::Name],
    pub ty: &'a ast::Type,
}

pub(super) struct GenFnTemplate<'a> {
    pub mod_id: ModId,
    pub offset: u32,
    pub owner: Option<StructId>,
    pub owner_env: GenEnv,
    pub decl: &'a ast::FnDecl,
}

pub(super) struct GenConstTemplate<'a> {
    pub mod_id: ModId,
    pub offset: u32,
    pub generics: &'a [ast::Name],
    pub ty: Option<&'a ast::Type>,
    pub init: &'a ast::Expr,
}

pub(super) struct ModConstInfo<'a> {
    pub mod_id: ModId,
    pub offset: u32,
    pub name: String,
    pub ty: Option<&'a ast::Type>,
    pub init: &'a ast::Expr,
    pub global: Option<GlobalId>,
}

pub(super) struct TraitInfo<'a> {
    pub mod_id: ModId,
    pub offset: u32,
    pub name: String,
    pub methods: Vec<&'a ast::FnDecl>,
    /// `trait HasW: w: Int`: the fields a record meets it with, which may name its `generics`.
    pub fields: Vec<(&'a str, &'a ast::Type)>,
    pub generics: Vec<&'a str>,
    /// `trait Selective<T>: Idempotent<T>` lists `Idempotent`.
    pub supers: Vec<String>,
    /// The associated types it gives a default: `type Out = ?T`.
    pub assoc_defaults: Vec<(&'a str, &'a ast::Type)>,
    /// `@on_unimplemented("{T} can't ...")`: the trait's own words for a
    /// type with no `impl` of it.
    pub unimplemented: Option<String>,
    /// `@structural trait Hash`: a compound type has it when each part does.
    pub structural: bool,
    /// `trait Both = Sh + Nm`: implemented exactly when each of these is.
    pub alias_of: Option<Vec<String>>,
    /// An alias's traits with their args: `Next<V>` of `trait WNext<V, W> = Next<V>`.
    pub alias_tys: Vec<&'a ast::Type>,
    /// An alias's bounds on associated types, each the associated type's name and one bound:
    /// `where Self.Arc: Weighted<W>`.
    pub alias_where: Vec<(&'a str, &'a ast::Type)>,
}

pub(super) struct ImplInfo<'a> {
    pub mod_id: ModId,
    pub offset: u32,
    pub trait_name: Option<String>,
    pub trait_args: Vec<&'a ast::Type>,
    pub target: &'a ast::Type,
    pub bounds: &'a [ast::Bound],
    pub body: &'a ast::Block,
}

#[derive(Clone)]
pub(super) struct ProjectionInfo<'a> {
    pub mod_id: ModId,
    pub offset: u32,
    pub owner: StructId,
    pub env: GenEnv,
    pub decl: &'a ast::FnDecl,
}

#[derive(Clone)]
pub(super) struct LawInfo<'a> {
    pub mod_id: ModId,
    pub offset: u32,
    pub name: String,
    pub params: &'a [ast::Param],
    pub body: &'a ast::Expr,
    /// A trait's law names the trait's fns without `self` unqualified (R08.3.1);
    /// empty for a struct's own law.
    pub trait_fns: Vec<String>,
}

pub(super) use super::keyed::OwnedDomainInit;

#[derive(Clone)]
pub(super) struct StructConst<'a> {
    pub mod_id: ModId,
    pub offset: u32,
    pub ty: Option<&'a ast::Type>,
    pub init: &'a ast::Expr,
}

impl<'a> Lowerer<'a> {
    /// Collects the transitive module graph starting from the root `Analysis` (`ModId = 0`).
    pub(super) fn load_modules(&mut self, root: &'a Analysis) {
        self.mods.push(root);
        self.mod_syms.push(HashMap::new());
        let mut path_to_mod: HashMap<String, ModId> = HashMap::new();
        let mut queue = vec![0usize];
        let mut head = 0;
        while head < queue.len() {
            let m = queue[head];
            head += 1;
            let analysis = self.mods[m];
            for imported in analysis.modules() {
                if !path_to_mod.contains_key(&imported.path) {
                    let id = self.mods.len();
                    path_to_mod.insert(imported.path.clone(), id);
                    self.mods.push(&imported.analysis);
                    self.mod_syms.push(HashMap::new());
                    if imported.name == "std.mod" {
                        self.std_mod = Some(id);
                    }
                    queue.push(id);
                }
            }
        }
        self.path_to_mod = path_to_mod;
    }

    pub(super) fn mod_items(&self, m: ModId) -> impl Iterator<Item = (u32, &'a ast::Stmt)> {
        let analysis = self.mods[m];
        let judge = self.judge;
        let aside = self.aside;
        analysis.items.iter().flat_map(move |item| {
            let offset = item.offset;
            // A `@check`, an `@interactor` and a `@alt(program)` are
            // each a program of their own: checked, run by `cpl stress`,
            // and no part of the solution a judge gets.
            item.syntax
                .parsed
                .stmts
                .iter()
                .filter(move |s| {
                    !s.decorators.iter().any(|d| match d.name.text.as_str() {
                        "checker" | "check" => aside != Some(Aside::Checker),
                        "interactor" => !judge,
                        "brute" | "alt" => is_whole_brute(d) && aside != Some(Aside::Brute),
                        _ => false,
                    })
                })
                .map(move |s| (offset, s))
        })
    }

    /// Look up a symbol in the current module (or local domains/aliases/gen_env, including enclosing scopes).
    pub(super) fn get_sym(&self, name: &str) -> Option<Sym> {
        if let Some(local_sym) = self
            .cx
            .local_syms
            .iter()
            .rev()
            .find_map(|s| s.get(name).copied())
        {
            return Some(local_sym);
        }
        for ocx in self.outer_cxs.iter().rev() {
            if let Some(local_sym) = ocx
                .local_syms
                .iter()
                .rev()
                .find_map(|s| s.get(name).copied())
            {
                return Some(local_sym);
            }
        }
        let mut m = self.cur_mod;
        loop {
            if let Some(sym) = self.mod_syms[m].get(name) {
                return self.unaliased(*sym);
            }
            m = self.inline_mods.get(&m)?.0;
        }
    }

    pub(super) fn mangle_arg(&self, arg: &GenArg) -> String {
        match arg {
            GenArg::Int(v) => {
                if *v < 0 {
                    format!("n{}", v.unsigned_abs())
                } else {
                    v.to_string()
                }
            }
            GenArg::Domain(d) => format!("D{d}_{}", self.domains[*d]),
            GenArg::Ty(ty) => self.mangle_ty(ty),
            GenArg::RuntimeInt(gid) => format!("RI{gid}"),
            GenArg::Op(op) => format!("Op_{op:?}"),
            GenArg::Range(lo, hi) => format!("R{:?}_{:?}", lo, hi),
            GenArg::OptRange(lo, hi) => format!("OR{:?}_{:?}", lo, hi),
            GenArg::Sized(ty, lens) => format!("S{}_{lens:?}", self.mangle_ty(ty)),
        }
    }

    pub(super) fn mangle_ty(&self, ty: &Ty) -> String {
        match ty {
            Ty::Unit => "Unit".into(),
            Ty::Int => "Int".into(),
            Ty::I128 => "I128".into(),
            Ty::U64 => "U64".into(),
            Ty::Float => "Float".into(),
            Ty::Bool => "Bool".into(),
            Ty::Char => "Char".into(),
            Ty::Str => "Str".into(),
            Ty::StrView => "StrView".into(),
            Ty::Pos(d) => format!("P{d}"),
            Ty::Bound(d) => format!("B{d}"),
            Ty::Range(t) => format!("Rng_{}", self.mangle_ty(t)),
            Ty::DomainVal => "Dom".into(),
            Ty::Opt(t) => format!("Opt_{}", self.mangle_ty(t)),
            Ty::List(t) => format!("Lst_{}", self.mangle_ty(t)),
            Ty::ListView(t, _) | Ty::Keyed(t, _) => format!("LstView_{}", self.mangle_ty(t)),
            Ty::MutView(t, _) => format!("MutView_{}", self.mangle_ty(t)),
            Ty::Map(k, v) => format!("Map_{}_{}", self.mangle_ty(k), self.mangle_ty(v)),
            Ty::HashMap(k, v) => format!("HMap_{}_{}", self.mangle_ty(k), self.mangle_ty(v)),
            Ty::Set(t) => format!("Set_{}", self.mangle_ty(t)),
            Ty::HashSet(t) => format!("HSet_{}", self.mangle_ty(t)),
            Ty::Array { dims, elem } => format!("Arr{}_{}", dims.len(), self.mangle_ty(elem)),
            Ty::Tuple(ts) => {
                let parts: Vec<String> = ts.iter().map(|t| self.mangle_ty(t)).collect();
                format!("Tup{}_{}", ts.len(), parts.join("_"))
            }
            Ty::Record(fs) => {
                let parts: Vec<String> = fs
                    .iter()
                    .map(|(n, t)| format!("{n}_{}", self.mangle_ty(t)))
                    .collect();
                format!("Rec{}_{}", fs.len(), parts.join("_"))
            }
            Ty::Fn(ps, r) => {
                let parts: Vec<String> = ps.iter().map(|t| self.mangle_ty(t)).collect();
                format!("Fn_{}_{}", parts.join("_"), self.mangle_ty(r))
            }
            Ty::Mut(t) => format!("Mut_{}", self.mangle_ty(t)),
            Ty::Struct(id) => format!("S{id}"),
            Ty::Enum(id) => format!("E{id}"),
            Ty::Error => "Err".into(),
        }
    }

    /// Probes the concrete type of an expression without permanently emitting errors or side effects.
    pub(super) fn probe_ty_silent(&mut self, e: &ast::Expr) -> Option<Ty> {
        let val = self.probe_expr(e, None);
        (val.ty != Ty::Error).then_some(val.ty)
    }

    /// Lowers `e` only to learn about it: the statements it would hoist ahead of its
    /// statement and the errors it would report are dropped, so the probe never runs.
    fn probe_expr_ty(
        &mut self,
        e: &ast::Expr,
        param_ty: &ast::Type,
        tpl_mod: ModId,
        tpl_offset: u32,
        gen_names: &[String],
        inferred: &GenEnv,
    ) -> Option<Ty> {
        let diag_len = self.diags.len();
        let hint = if !self.ast_ty_has_unbound(param_ty, gen_names, inferred) {
            let saved_mod = std::mem::replace(&mut self.cur_mod, tpl_mod);
            let saved_offset = std::mem::replace(&mut self.offset, tpl_offset);
            let saved_env = std::mem::replace(&mut self.gen_env, inferred.clone());
            let h = self.resolve_ty(param_ty);
            self.cur_mod = saved_mod;
            self.offset = saved_offset;
            self.gen_env = saved_env;
            (h != Ty::Error).then_some(h)
        } else {
            None
        };
        let saved_cx = self.cx.clone();
        let saved_outer = self.outer_cxs.clone();
        let saved_closures = self.closures.len();
        let probed = self.expr(e, hint.as_ref());
        self.call_hint.1 = None;
        self.cx = saved_cx;
        self.outer_cxs = saved_outer;
        self.diags.truncate(diag_len);
        self.drop_closures_from(saved_closures);
        (probed.ty != Ty::Error).then_some(probed.ty)
    }
}

pub(super) fn split_projection_body(
    decl: &ast::FnDecl,
) -> (&[ast::Stmt], Option<&ast::Expr>, &[ast::Stmt]) {
    let block = match &decl.body {
        ast::FnBody::Block(block) => block,
        ast::FnBody::Expr(e) => return (&[], Some(e), &[]),
        _ => return (&[], None, &[]),
    };
    for (i, stmt) in block.stmts.iter().enumerate() {
        if let ast::StmtKind::Lend { place, .. } = &stmt.kind {
            return (&block.stmts[..i], Some(place), &block.stmts[i + 1..]);
        }
    }
    (&[], None, &[])
}

/// The name a `let` in a struct or impl body gives its constant. A
/// capitalized one, `let MOD = P`, parses as a pattern that could be an
/// enum variant; there it is the name.
pub(super) fn let_const_name(pat: &ast::Pattern) -> Option<&str> {
    match &pat.kind {
        ast::PatternKind::Name(name) => Some(name),
        _ => pat.bare_variant(),
    }
}

fn type_uses_as_index(ty: &ast::Type, name: &str) -> bool {
    match &ty.kind {
        ast::TypeKind::Array { index, elem } => {
            index.iter().any(|i| type_names(i, name)) || type_uses_as_index(elem, name)
        }
        ast::TypeKind::Named { args, .. } => args.iter().any(|a| type_uses_as_index(a, name)),
        ast::TypeKind::Optional(inner)
        | ast::TypeKind::Mut(inner)
        | ast::TypeKind::OwnDomain { elem: inner, .. } => type_uses_as_index(inner, name),
        ast::TypeKind::Tuple(ts) => ts.iter().any(|t| type_uses_as_index(t, name)),
        ast::TypeKind::Record(fs) => fs.iter().any(|(_, t)| type_uses_as_index(t, name)),
        ast::TypeKind::Fn { params, ret } => {
            params.iter().any(|p| type_uses_as_index(p, name)) || type_uses_as_index(ret, name)
        }
        _ => false,
    }
}

/// Whether the written type `ty` names `name` anywhere inside it.
pub(super) fn type_names(ty: &ast::Type, name: &str) -> bool {
    match &ty.kind {
        ast::TypeKind::Named { name: n, args } => {
            n.text == name
                || n.text.split('.').next() == Some(name)
                || args.iter().any(|a| type_names(a, name))
        }
        ast::TypeKind::Array { index, elem } => {
            index.iter().any(|i| type_names(i, name)) || type_names(elem, name)
        }
        ast::TypeKind::OwnDomain { elem, .. }
        | ast::TypeKind::Optional(elem)
        | ast::TypeKind::Mut(elem)
        | ast::TypeKind::Yields(elem) => type_names(elem, name),
        ast::TypeKind::Tuple(items) => items.iter().any(|i| type_names(i, name)),
        ast::TypeKind::Record(fields) => fields.iter().any(|(_, t)| type_names(t, name)),
        ast::TypeKind::Fn { params, ret } => {
            params.iter().any(|p| type_names(p, name)) || type_names(ret, name)
        }
        ast::TypeKind::Value(e) => matches!(&e.kind, ast::ExprKind::Name(n) if n == name),
        ast::TypeKind::Range { .. }
        | ast::TypeKind::NewDomain
        | ast::TypeKind::DomainBinder(_)
        | ast::TypeKind::ConstrainedStr { .. }
        | ast::TypeKind::LiteralUnion(_)
        | ast::TypeKind::ItemOf(_)
        | ast::TypeKind::Infer
        | ast::TypeKind::Error => false,
        ast::TypeKind::Row(fields) => fields.iter().any(|(_, t)| type_names(t, name)),
        ast::TypeKind::RowWithout { prefix, base, .. } => {
            prefix.iter().any(|(_, t)| type_names(t, name)) || type_names(base, name)
        }
    }
}

/// An argument for a `lazy f: () -> T` becomes `() => arg`, unless it is a
/// lambda already.
pub(super) fn thunk_lazy_args(params: &[&ast::Param], args: &[ast::Arg]) -> Option<Vec<ast::Arg>> {
    if !params.iter().any(|p| p.lazy) {
        return None;
    }
    let positional: Vec<&&ast::Param> = params.iter().filter(|p| !p.named_only).collect();
    let mut next = 0;
    let out = args
        .iter()
        .map(|arg| {
            let param = match &arg.name {
                Some(n) => params.iter().find(|p| p.label_text() == n.text),
                None => {
                    next += 1;
                    positional.get(next - 1).copied()
                }
            };
            let mut arg = arg.clone();
            if param.is_some_and(|p| p.lazy)
                && !matches!(arg.value.kind, ast::ExprKind::Lambda { .. })
            {
                let span = arg.value.span;
                arg.value = ast::Expr {
                    kind: ast::ExprKind::Lambda {
                        params: Vec::new(),
                        param_tys: Vec::new(),
                        param_mut: Vec::new(),
                        ret_ty: None,
                        body: Box::new(arg.value),
                    },
                    span,
                };
            }
            arg
        })
        .collect();
    Some(out)
}

/// A variadic `..xs: Ts` takes the positional arguments from its place on
/// as one tuple, passed by name; `Ts` is then that tuple's type.
fn pack_args(params: &[&ast::Param], args: &[ast::Arg], span: Span) -> Option<Vec<ast::Arg>> {
    let at = params
        .iter()
        .filter(|p| !p.named_only)
        .position(|p| p.variadic)?;
    let param = params.iter().filter(|p| !p.named_only).nth(at)?;
    let mut kept = Vec::new();
    let mut items = Vec::new();
    let mut named = Vec::new();
    for arg in args {
        match &arg.name {
            None if kept.iter().filter(|a: &&ast::Arg| a.name.is_none()).count() >= at => {
                items.push(arg.value.clone());
            }
            // A name no parameter has is a field of the named tuple (R06.1.5).
            Some(n) if !params.iter().any(|p| p.label_text() == n.text) => {
                named.push((n.clone(), arg.value.clone()));
            }
            _ => kept.push(arg.clone()),
        }
    }
    let ends = |a: Span, b: Span| Span::new(a.start, b.end);
    let span = match (items.first(), items.last(), named.first(), named.last()) {
        (Some(a), Some(b), _, _) => ends(a.span, b.span),
        (_, _, Some(a), Some(b)) => ends(a.0.span, b.1.span),
        _ => Span::point(span.end),
    };
    let kind = if items.is_empty() && !named.is_empty() {
        ast::ExprKind::Record(named)
    } else {
        ast::ExprKind::Tuple(items)
    };
    kept.push(ast::Arg {
        name: Some(ast::Name {
            text: param.label_text().to_string(),
            span,
        }),
        mutable: false,
        value: ast::Expr { kind, span },
    });
    Some(kept)
}

/// The least and greatest value of `a op b`.
fn interval(op: BinOp, a: (i64, i64), b: (i64, i64)) -> Option<(i64, i64)> {
    match op {
        BinOp::Add => Some((a.0.checked_add(b.0)?, a.1.checked_add(b.1)?)),
        BinOp::Sub => Some((a.0.checked_sub(b.1)?, a.1.checked_sub(b.0)?)),
        BinOp::Mul => {
            let ends = [
                a.0.checked_mul(b.0)?,
                a.0.checked_mul(b.1)?,
                a.1.checked_mul(b.0)?,
                a.1.checked_mul(b.1)?,
            ];
            Some((*ends.iter().min()?, *ends.iter().max()?))
        }
        _ => None,
    }
}

pub(super) fn ty_mentions(t: &ast::Type, name: &str) -> bool {
    match &t.kind {
        ast::TypeKind::Named { name: n, args } => {
            // `M.Fold` mentions `M`.
            n.text == name
                || n.text.split_once('.').is_some_and(|(head, _)| head == name)
                || args.iter().any(|a| ty_mentions(a, name))
        }
        ast::TypeKind::Array { index, elem } => {
            index.iter().any(|i| ty_mentions(i, name)) || ty_mentions(elem, name)
        }
        ast::TypeKind::OwnDomain { elem, .. }
        | ast::TypeKind::Optional(elem)
        | ast::TypeKind::Mut(elem)
        | ast::TypeKind::Yields(elem) => ty_mentions(elem, name),
        ast::TypeKind::Tuple(items) => items.iter().any(|i| ty_mentions(i, name)),
        ast::TypeKind::Record(fields) => fields.iter().any(|(_, f)| ty_mentions(f, name)),
        ast::TypeKind::Fn { params, ret } => {
            params.iter().any(|p| ty_mentions(p, name)) || ty_mentions(ret, name)
        }
        _ => false,
    }
}

/// The generic-env key binding the bare operator passed for the fn-typed parameter `name`, so
/// each operator gets its own instance and the body can tell `+` from `max`.
pub(super) fn op_param_key(name: &str) -> String {
    format!("__op_param_{name}")
}

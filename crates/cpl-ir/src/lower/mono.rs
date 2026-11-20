//! Multi-module collection, generic monomorphization, traits/impls,
//! closures, `lends` projections, and `laws_hold`.

use super::*;
use std::collections::HashSet;

pub(super) type ModId = usize;
pub(super) type GenStructId = usize;
pub(super) type GenEnumId = usize;
pub(super) type GenAliasId = usize;
pub(super) type GenFnId = usize;
pub(super) type GenConstId = usize;
pub(super) type ModConstId = usize;
pub(super) type TraitId = usize;

/// A bare binary operator or stdlib combinator passed as a value (`+`, `*`, `max`, `min`, `gcd`, etc.).
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
}

/// One concrete argument to a generic type, struct, function, or const.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(super) enum GenArg {
    Ty(Ty),
    Domain(DomainId),
    Int(i64),
    RuntimeInt(GlobalId),
    Op(BareOp),
}

pub(super) type GenEnv = HashMap<String, GenArg>;

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
    #[allow(dead_code)]
    pub name: String,
    pub methods: Vec<&'a ast::FnDecl>,
}

pub(super) struct ImplInfo<'a> {
    pub mod_id: ModId,
    pub offset: u32,
    pub trait_name: Option<String>,
    pub target: &'a ast::Type,
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
}

#[derive(Clone)]
pub(super) struct OwnedDomainInit<'a> {
    pub id: DomainId,
    pub def: &'a ast::Expr,
    pub offset: u32,
    pub mod_id: ModId,
}

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
                    // SAFETY: imported.analysis lives inside `root`'s Arc tree for lifetime `'a`.
                    let ptr: *const Analysis = &*imported.analysis;
                    self.mods.push(unsafe { &*ptr });
                    self.mod_syms.push(HashMap::new());
                    queue.push(id);
                }
            }
        }
        self.path_to_mod = path_to_mod;
    }

    pub(super) fn mod_items(&self, m: ModId) -> impl Iterator<Item = (u32, &'a ast::Stmt)> {
        let analysis = self.mods[m];
        analysis.items.iter().flat_map(|item| {
            let offset = item.offset;
            // A `@checker` or `@interactor` is the judge's side of the
            // problem: it is checked, and is no part of the solution.
            item.syntax
                .parsed
                .stmts
                .iter()
                .filter(|s| {
                    !s.decorators
                        .iter()
                        .any(|d| matches!(d.name.text.as_str(), "checker" | "interactor"))
                })
                .map(move |s| (offset, s))
        })
    }

    /// Look up a symbol in the current module (or local domains/aliases/gen_env, including enclosing scopes).
    pub(super) fn get_sym(&self, name: &str) -> Option<Sym> {
        if let Some(local_sym) = self.cx.local_syms.iter().rev().find_map(|s| s.get(name).copied()) {
            return Some(local_sym);
        }
        for ocx in self.outer_cxs.iter().rev() {
            if let Some(local_sym) = ocx.local_syms.iter().rev().find_map(|s| s.get(name).copied()) {
                return Some(local_sym);
            }
        }
        self.mod_syms[self.cur_mod].get(name).copied()
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
        }
    }

    /// Recognizes a bare operator expression (`+`, `*`, `^`, `|`, `&`, `max`, `min`, `gcd`)
    /// or a local variable bound to a `BareOp` generic parameter.
    pub(super) fn detect_bare_op(&self, e: &ast::Expr) -> Option<BareOp> {
        match &e.kind {
            ast::ExprKind::Paren(inner) => self.detect_bare_op(inner),
            ast::ExprKind::Name(n) => {
                if let Some(&op) = self.cx.op_locals.get(n) {
                    return Some(op);
                }
                if self.lookup(n).is_some() {
                    return None;
                }
                match n.as_str() {
                    "max" => Some(BareOp::Max),
                    "min" => Some(BareOp::Min),
                    "gcd" => Some(BareOp::Gcd),
                    _ => None,
                }
            }
            ast::ExprKind::Lambda { params, body } if params.len() == 2 => {
                let ast::PatternKind::Name(p0) = &params[0].kind else {
                    return None;
                };
                let ast::PatternKind::Name(p1) = &params[1].kind else {
                    return None;
                };
                if let ast::ExprKind::Binary { op, lhs, rhs, .. } = &body.kind {
                    let is_direct = matches!(&lhs.kind, ast::ExprKind::Name(a) if a == p0)
                        && matches!(&rhs.kind, ast::ExprKind::Name(b) if b == p1);
                    if is_direct {
                        return match op {
                            ast::BinOp::Add => Some(BareOp::Add),
                            ast::BinOp::Mul => Some(BareOp::Mul),
                            ast::BinOp::BitXor => Some(BareOp::BitXor),
                            ast::BinOp::BitOr => Some(BareOp::BitOr),
                            ast::BinOp::BitAnd => Some(BareOp::BitAnd),
                            _ => None,
                        };
                    }
                }
                None
            }
            _ => None,
        }
    }

    pub(super) fn mangle_ty(&self, ty: &Ty) -> String {
        match ty {
            Ty::Unit => "Unit".into(),
            Ty::Int => "Int".into(),
            Ty::Float => "Float".into(),
            Ty::Bool => "Bool".into(),
            Ty::Char => "Char".into(),
            Ty::Str => "Str".into(),
            Ty::Pos(d) => format!("P{d}"),
            Ty::Bound(d) => format!("B{d}"),
            Ty::Range(t) => format!("Rng_{}", self.mangle_ty(t)),
            Ty::DomainVal => "Dom".into(),
            Ty::Opt(t) => format!("Opt_{}", self.mangle_ty(t)),
            Ty::List(t) => format!("Lst_{}", self.mangle_ty(t)),
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
                let parts: Vec<String> = fs.iter().map(|(n, t)| format!("{n}_{}", self.mangle_ty(t))).collect();
                format!("Rec{}_{}", fs.len(), parts.join("_"))
            }
            Ty::Fn(ps, r) => {
                let parts: Vec<String> = ps.iter().map(|t| self.mangle_ty(t)).collect();
                format!("Fn_{}_{}", parts.join("_"), self.mangle_ty(r))
            }
            Ty::Struct(id) => format!("S{id}"),
            Ty::Enum(id) => format!("E{id}"),
            Ty::Error => "Err".into(),
        }
    }

    /// Evaluates a compile-time integer expression (for `Mod<998244353>`, `Mod<P>`, etc.).
    pub(super) fn eval_const_int(&self, e: &ast::Expr) -> Option<i64> {
        match &e.kind {
            ast::ExprKind::Int(v) => i64::try_from(*v).ok(),
            ast::ExprKind::Paren(inner) => self.eval_const_int(inner),
            ast::ExprKind::Unary { op: UnOp::Neg, expr } => self.eval_const_int(expr)?.checked_neg(),
            ast::ExprKind::Binary { op, lhs, rhs, .. } => {
                let l = self.eval_const_int(lhs)?;
                let r = self.eval_const_int(rhs)?;
                match op {
                    BinOp::Add => l.checked_add(r),
                    BinOp::Sub => l.checked_sub(r),
                    BinOp::Mul => l.checked_mul(r),
                    BinOp::Div if r != 0 => l.checked_div_euclid(r),
                    BinOp::Rem if r != 0 => l.checked_rem_euclid(r),
                    BinOp::Pow if r >= 0 => l.checked_pow(r as u32),
                    _ => None,
                }
            }
            ast::ExprKind::Name(n) => {
                if let Some(GenArg::Int(v)) = self.gen_env.get(n) {
                    return Some(*v);
                }
                if let Some(Sym::ModConst(cid)) = self.get_sym(n) {
                    return self.eval_const_int(self.mod_consts[cid].init);
                }
                if let Some(v) = self.const_ints.get(n) {
                    return Some(*v);
                }
                None
            }
            _ => None,
        }
    }

    /// Resolves a list of AST type arguments into concrete `GenArg`s.
    pub(super) fn resolve_gen_args(&mut self, args: &[ast::Type]) -> Vec<GenArg> {
        args.iter().map(|t| self.resolve_gen_arg(t)).collect()
    }

    pub(super) fn resolve_gen_arg(&mut self, ty: &ast::Type) -> GenArg {
        match &ty.kind {
            ast::TypeKind::Value(expr) => {
                if let Some(v) = self.eval_const_int(expr) {
                    return GenArg::Int(v);
                }
                // Could be a domain or type name written in value position.
                if let ast::ExprKind::Name(n) = &expr.kind {
                    if let Some(a) = self.gen_env.get(n).cloned() {
                        return a;
                    }
                    if let Some(Sym::Domain(d)) = self.get_sym(n) {
                        return GenArg::Domain(d);
                    }
                    if let Some(Sym::Global(gid)) = self.get_sym(n) {
                        if self.globals[gid].ty == Ty::Int {
                            return GenArg::RuntimeInt(gid);
                        }
                    }
                }
                self.error(codes::BAD_NUMBER, expr.span, "expected a compile-time constant integer");
                GenArg::Int(1)
            }
            ast::TypeKind::Named { name, args } if args.is_empty() => {
                if let Some(a) = self.gen_env.get(&name.text).cloned() {
                    return a;
                }
                if let Some(Sym::Domain(d)) = self.get_sym(&name.text) {
                    return GenArg::Domain(d);
                }
                if let Some(Sym::Global(gid)) = self.get_sym(&name.text) {
                    if self.globals[gid].ty == Ty::Int {
                        return GenArg::RuntimeInt(gid);
                    }
                }
                if let Some(v) = self.const_ints.get(&name.text).copied() {
                    return GenArg::Int(v);
                }
                if let Some(Sym::ModConst(cid)) = self.get_sym(&name.text) {
                    if let Some(v) = self.eval_const_int(self.mod_consts[cid].init) {
                        return GenArg::Int(v);
                    }
                }
                let resolved = self.resolve_ty(ty);
                match resolved {
                    Ty::Pos(d) => GenArg::Domain(d),
                    other => GenArg::Ty(other),
                }
            }
            _ => {
                let resolved = self.resolve_ty(ty);
                match resolved {
                    Ty::Pos(d) => GenArg::Domain(d),
                    other => GenArg::Ty(other),
                }
            }
        }
    }

    /// Instantiates a generic struct template with concrete `args`, memoized.
    pub(super) fn instantiate_struct(&mut self, gid: GenStructId, args: Vec<GenArg>) -> StructId {
        let key = (gid, args.clone());
        if let Some(&id) = self.struct_insts.get(&key) {
            return id;
        }
        let tpl_mod = self.gen_structs[gid].mod_id;
        let tpl_offset = self.gen_structs[gid].offset;
        let decl = self.gen_structs[gid].decl;
        let id = self.structs.len();
        let suffix: Vec<String> = args.iter().map(|a| self.mangle_arg(a)).collect();
        let mangled = format!("{}_{}_{id}", decl.name.text, suffix.join("_"));

        let mut env: GenEnv = HashMap::new();
        for (p, a) in decl.generics.iter().zip(&args) {
            env.insert(p.text.clone(), a.clone());
        }

        self.structs.push(StructInfo {
            def: StructDef {
                name: mangled,
                fields: Vec::new(),
                show_fn: None,
                eq_fn: None,
                lt_fn: None,
            },
            decl,
            mod_id: tpl_mod,
            offset: tpl_offset,
            origin_gen: Some((gid, args.clone())),
            env: env.clone(),
            methods: HashMap::new(),
            gen_methods: HashMap::new(),
            getters: HashSet::new(),
            projections: HashMap::new(),
            consts: HashMap::new(),
            owned_domains: HashMap::new(),
            owned_domain_inits: Vec::new(),
            laws: Vec::new(),
            index_domain: None,
        });
        self.struct_insts.insert(key, id);

        // Switch context to the struct's defining module and generic env.
        let saved_mod = std::mem::replace(&mut self.cur_mod, tpl_mod);
        let saved_offset = std::mem::replace(&mut self.offset, tpl_offset);
        let saved_env = std::mem::replace(&mut self.gen_env, env);
        let saved_self = self.self_struct.replace(id);

        // 1. Collect struct-owned domains (e.g. `domain Pre = 0..V.K` in `Rooted<V>`).
        for item in &decl.methods {
            if let ast::StmtKind::Domain { name, def } = &item.kind {
                let dom_id = self.domains.len();
                self.domains.push(format!("{}_{}", self.structs[id].def.name, name.text));
                self.structs[id].owned_domains.insert(name.text.clone(), dom_id);
                self.structs[id].owned_domain_inits.push(OwnedDomainInit {
                    id: dom_id,
                    def,
                    offset: tpl_offset,
                    mod_id: tpl_mod,
                });
                self.gen_env.insert(name.text.clone(), GenArg::Domain(dom_id));
                self.structs[id].env.insert(name.text.clone(), GenArg::Domain(dom_id));
            }
        }

        // 2. Resolve fields.
        let fields: Vec<(String, Ty)> = decl
            .fields
            .iter()
            .map(|f| (f.name.text.clone(), self.resolve_ty(&f.ty)))
            .collect();
        self.structs[id].def.fields = fields;

        // 3. Collect methods, getters, projections, laws, constants.
        let struct_env = self.gen_env.clone();
        for item in &decl.methods {
            let is_get = item.decorators.iter().any(|d| d.name.text == "get");
            match &item.kind {
                ast::StmtKind::Fn(f) => {
                    if is_get {
                        self.structs[id].getters.insert(f.name.text.clone());
                    }
                    if f.lends.is_some() {
                        self.structs[id].projections.insert(
                            f.name.text.clone(),
                            ProjectionInfo {
                                mod_id: tpl_mod,
                                offset: tpl_offset,
                                owner: id,
                                env: struct_env.clone(),
                                decl: f,
                            },
                        );
                    } else if !f.generics.is_empty() {
                        let gfid = self.gen_fns.len();
                        self.gen_fns.push(GenFnTemplate {
                            mod_id: tpl_mod,
                            offset: tpl_offset,
                            owner: Some(id),
                            owner_env: struct_env.clone(),
                            decl: f,
                        });
                        self.structs[id].gen_methods.insert(f.name.text.clone(), gfid);
                    } else {
                        let fn_id = self.declare_fn_in(f, tpl_mod, tpl_offset, Some(id), struct_env.clone());
                        self.resolve_fn_sig(fn_id, None);
                        self.structs[id].methods.insert(f.name.text.clone(), fn_id);
                    }
                }
                ast::StmtKind::Let { pat: ast::Pattern { kind: ast::PatternKind::Name(cname), .. }, ty, init: Some(init), .. } => {
                    self.structs[id].consts.insert(
                        cname.clone(),
                        StructConst {
                            mod_id: tpl_mod,
                            offset: tpl_offset,
                            ty: ty.as_ref(),
                            init,
                        },
                    );
                }
                ast::StmtKind::Const { name, ty, init: Some(init), .. } => {
                    self.structs[id].consts.insert(
                        name.text.clone(),
                        StructConst {
                            mod_id: tpl_mod,
                            offset: tpl_offset,
                            ty: ty.as_ref(),
                            init,
                        },
                    );
                }
                ast::StmtKind::Law { name, params, body } => {
                    self.structs[id].laws.push(LawInfo {
                        mod_id: tpl_mod,
                        offset: tpl_offset,
                        name: name.text.clone(),
                        params,
                        body,
                    });
                }
                _ => {}
            }
        }

        // 4. Attach matching `impl` blocks for this struct.
        self.attach_impls_for_struct(id);

        self.cur_mod = saved_mod;
        self.offset = saved_offset;
        self.gen_env = saved_env;
        self.self_struct = saved_self;
        id
    }

    /// Instantiates a generic enum template with concrete `args`, memoized.
    pub(super) fn instantiate_enum(&mut self, gid: GenEnumId, args: Vec<GenArg>) -> EnumId {
        let key = (gid, args.clone());
        if let Some(&id) = self.enum_insts.get(&key) {
            return id;
        }
        let tpl_mod = self.gen_enums[gid].mod_id;
        let tpl_offset = self.gen_enums[gid].offset;
        let decl = self.gen_enums[gid].decl;
        let id = self.enums.len();
        let suffix: Vec<String> = args.iter().map(|a| self.mangle_arg(a)).collect();
        let mangled = format!("{}_{}_{id}", decl.name.text, suffix.join("_"));
        self.enums.push(EnumDef {
            name: mangled,
            variants: Vec::new(),
        });
        self.enum_insts.insert(key, id);

        let mut env = HashMap::new();
        for (p, a) in decl.generics.iter().zip(&args) {
            env.insert(p.text.clone(), a.clone());
        }
        let saved_mod = std::mem::replace(&mut self.cur_mod, tpl_mod);
        let saved_offset = std::mem::replace(&mut self.offset, tpl_offset);
        let saved_env = std::mem::replace(&mut self.gen_env, env);
        let variants = decl
            .variants
            .iter()
            .map(|v| {
                let fields = v.fields.iter().map(|(_, t)| self.resolve_ty(t)).collect();
                (v.name.text.clone(), fields)
            })
            .collect();
        self.enums[id].variants = variants;
        self.cur_mod = saved_mod;
        self.offset = saved_offset;
        self.gen_env = saved_env;
        id
    }

    /// Finds and attaches all `impl` blocks matching concrete struct `sid`.
    pub(super) fn attach_impls_for_struct(&mut self, sid: StructId) {
        let struct_decl_name = self.structs[sid].decl.name.text.clone();
        let struct_mod = self.structs[sid].mod_id;
        let origin_args: Vec<GenArg> = self.structs[sid]
            .origin_gen
            .as_ref()
            .map(|(_, a)| a.clone())
            .unwrap_or_default();

        let num_impls = self.impls.len();
        for idx in 0..num_impls {
            let imp_mod = self.impls[idx].mod_id;
            let imp_offset = self.impls[idx].offset;
            let imp_trait = self.impls[idx].trait_name.clone();
            let imp_target = self.impls[idx].target;
            let imp_body = self.impls[idx].body;

            let ast::TypeKind::Named { name: tname, args: targs } = &imp_target.kind else {
                continue;
            };
            // Check whether `tname` in `imp_mod` refers to the same struct.
            let target_matches = match self.mod_syms[imp_mod].get(&tname.text).copied() {
                Some(Sym::Struct(id)) => id == sid,
                Some(Sym::GenStruct(gid)) => {
                    self.structs[sid].origin_gen.as_ref().is_some_and(|(g, _)| *g == gid)
                }
                _ => tname.text == struct_decl_name && imp_mod == struct_mod,
            };
            if !target_matches {
                continue;
            }
            if targs.len() != origin_args.len() {
                continue;
            }
            // Bind impl type params from `targs` vs `origin_args`.
            let mut env = self.structs[sid].env.clone();
            let mut ok = true;
            for (targ, actual) in targs.iter().zip(&origin_args) {
                match &targ.kind {
                    ast::TypeKind::Named { name, args } if args.is_empty() => {
                        env.insert(name.text.clone(), actual.clone());
                    }
                    ast::TypeKind::Value(expr) => {
                        if let ast::ExprKind::Name(n) = &expr.kind {
                            env.insert(n.clone(), actual.clone());
                        } else if let Some(v) = self.eval_const_int(expr) {
                            if *actual != GenArg::Int(v) {
                                ok = false;
                            }
                        }
                    }
                    _ => {}
                }
            }
            if !ok {
                continue;
            }

            let saved_mod = std::mem::replace(&mut self.cur_mod, imp_mod);
            let saved_offset = std::mem::replace(&mut self.offset, imp_offset);
            let saved_env = std::mem::replace(&mut self.gen_env, env.clone());
            let saved_self = self.self_struct.replace(sid);

            // Handle `impl Index for S`: allocate backing domain for `let K = ...`.
            if imp_trait.as_deref() == Some("Index") && self.structs[sid].index_domain.is_none() {
                let dom_id = self.domains.len();
                self.domains.push(format!("{}_Index", self.structs[sid].def.name));
                self.structs[sid].index_domain = Some(dom_id);
            }

            for item in &imp_body.stmts {
                let is_get = item.decorators.iter().any(|d| d.name.text == "get");
                match &item.kind {
                    ast::StmtKind::Fn(f) => {
                        if is_get {
                            self.structs[sid].getters.insert(f.name.text.clone());
                        }
                        if f.lends.is_some() {
                            self.structs[sid].projections.insert(
                                f.name.text.clone(),
                                ProjectionInfo {
                                    mod_id: imp_mod,
                                    offset: imp_offset,
                                    owner: sid,
                                    env: env.clone(),
                                    decl: f,
                                },
                            );
                        } else if !f.generics.is_empty() {
                            let gfid = self.gen_fns.len();
                            self.gen_fns.push(GenFnTemplate {
                                mod_id: imp_mod,
                                offset: imp_offset,
                                owner: Some(sid),
                                owner_env: env.clone(),
                                decl: f,
                            });
                            self.structs[sid].gen_methods.insert(f.name.text.clone(), gfid);
                        } else {
                            let fn_id = self.declare_fn_in(f, imp_mod, imp_offset, Some(sid), env.clone());
                            self.resolve_fn_sig(fn_id, imp_trait.as_deref());
                            self.structs[sid].methods.insert(f.name.text.clone(), fn_id);
                            match (imp_trait.as_deref(), f.name.text.as_str()) {
                                (Some("Show"), "show") => self.structs[sid].def.show_fn = Some(fn_id),
                                (Some("Eq"), "eq") => self.structs[sid].def.eq_fn = Some(fn_id),
                                (Some("Ord"), "lt") => self.structs[sid].def.lt_fn = Some(fn_id),
                                _ => {}
                            }
                        }
                    }
                    ast::StmtKind::Let { pat, ty, init: Some(init), .. } => {
                        let cname = match &pat.kind {
                            ast::PatternKind::Name(n) => Some(n.clone()),
                            ast::PatternKind::Variant { path, args }
                                if path.len() == 1 && args.is_empty() =>
                            {
                                Some(path[0].text.clone())
                            }
                            _ => None,
                        };
                        if let Some(cname) = cname {
                            self.structs[sid].consts.insert(
                                cname,
                                StructConst {
                                    mod_id: imp_mod,
                                    offset: imp_offset,
                                    ty: ty.as_ref(),
                                    init,
                                },
                            );
                        }
                    }
                    ast::StmtKind::Const { name, ty, init: Some(init), .. } => {
                        self.structs[sid].consts.insert(
                            name.text.clone(),
                            StructConst {
                                mod_id: imp_mod,
                                offset: imp_offset,
                                ty: ty.as_ref(),
                                init,
                            },
                        );
                    }
                    _ => {}
                }
            }

            // Copy any default trait methods not overridden by this impl.
            if let Some(tname) = &imp_trait {
                if let Some(Sym::Trait(tid)) = self.mod_syms[imp_mod].get(tname).copied() {
                    let t_mod = self.traits[tid].mod_id;
                    let t_offset = self.traits[tid].offset;
                    let t_methods = self.traits[tid].methods.clone();
                    for m in t_methods {
                        if !matches!(m.body, ast::FnBody::None)
                            && !self.structs[sid].methods.contains_key(&m.name.text)
                        {
                            let fn_id = self.declare_fn_in(m, t_mod, t_offset, Some(sid), env.clone());
                            self.resolve_fn_sig(fn_id, Some(tname.as_str()));
                            self.structs[sid].methods.insert(m.name.text.clone(), fn_id);
                        }
                    }
                }
            }

            self.cur_mod = saved_mod;
            self.offset = saved_offset;
            self.gen_env = saved_env;
            self.self_struct = saved_self;
        }
    }

    /// Resolves a generic function instantiation with concrete `args`, memoized.
    pub(super) fn instantiate_fn(&mut self, gfid: GenFnId, args: Vec<GenArg>) -> FnId {
        let key = (gfid, args.clone());
        if let Some(&id) = self.fn_insts.get(&key) {
            return id;
        }
        let tpl_mod = self.gen_fns[gfid].mod_id;
        let tpl_offset = self.gen_fns[gfid].offset;
        let owner = self.gen_fns[gfid].owner;
        let mut env = self.gen_fns[gfid].owner_env.clone();
        let decl = self.gen_fns[gfid].decl;
        for (p, a) in decl.generics.iter().zip(&args) {
            env.insert(p.text.clone(), a.clone());
        }
        let id = self.declare_fn_in(decl, tpl_mod, tpl_offset, owner, env);
        let suffix: Vec<String> = args.iter().map(|a| self.mangle_arg(a)).collect();
        self.sigs[id].name = format!("{}_{}_{id}", self.sigs[id].name, suffix.join("_"));
        self.fn_insts.insert(key, id);
        self.resolve_fn_sig(id, None);
        id
    }

    /// Evaluates a generic `const Sum<T> = ...` with concrete `args`.
    pub(super) fn eval_gen_const(&mut self, gcid: GenConstId, args: Vec<GenArg>) -> Expr {
        let tpl_mod = self.gen_consts[gcid].mod_id;
        let tpl_offset = self.gen_consts[gcid].offset;
        let generics = self.gen_consts[gcid].generics;
        let ty_ast = self.gen_consts[gcid].ty;
        let init = self.gen_consts[gcid].init;
        let mut env = HashMap::new();
        for (p, a) in generics.iter().zip(&args) {
            env.insert(p.text.clone(), a.clone());
        }
        let saved_mod = std::mem::replace(&mut self.cur_mod, tpl_mod);
        let saved_offset = std::mem::replace(&mut self.offset, tpl_offset);
        let saved_env = std::mem::replace(&mut self.gen_env, env);
        let saved_self = self.self_struct.take();
        let hint = ty_ast.map(|t| self.resolve_ty(t));
        let expr = self.expr(init, hint.as_ref());
        self.cur_mod = saved_mod;
        self.offset = saved_offset;
        self.gen_env = saved_env;
        self.self_struct = saved_self;
        expr
    }

    /// Lazily initializes an imported module's top-level `const`/`let` as a global.
    pub(super) fn ensure_mod_const(&mut self, mcid: ModConstId, span: Span) -> Expr {
        if let Some(gid) = self.mod_consts[mcid].global {
            return self.mk(ExprKind::Global(gid), self.globals[gid].ty.clone(), span);
        }
        let tpl_mod = self.mod_consts[mcid].mod_id;
        let tpl_offset = self.mod_consts[mcid].offset;
        let name = self.mod_consts[mcid].name.clone();
        let ty_ast = self.mod_consts[mcid].ty;
        let init = self.mod_consts[mcid].init;

        let saved_mod = std::mem::replace(&mut self.cur_mod, tpl_mod);
        let saved_offset = std::mem::replace(&mut self.offset, tpl_offset);
        let saved_env = std::mem::take(&mut self.gen_env);
        let saved_self = self.self_struct.take();
        let saved_cx = std::mem::replace(
            &mut self.cx,
            FnCx {
                is_main: false,
                scopes: vec![HashMap::new()],
                ..FnCx::default()
            },
        );
        let hint = ty_ast.map(|t| self.resolve_ty(t));
        let val = self.expr(init, hint.as_ref());
        let val_ty = val.ty.clone();
        let temp_cx = std::mem::replace(&mut self.cx, saved_cx);
        self.cur_mod = saved_mod;
        self.offset = saved_offset;
        self.gen_env = saved_env;
        self.self_struct = saved_self;

        // If initializing the const needed temp locals, wrap in a small init fn or run in main preamble.
        let gid = self.globals.len();
        let gname = format!("m{tpl_mod}_{name}_{gid}");
        self.globals.push(Global {
            name: gname,
            ty: val_ty.clone(),
        });
        self.mod_consts[mcid].global = Some(gid);
        let place = Place {
            kind: PlaceKind::Global(gid),
            ty: val_ty.clone(),
            span: val.span,
        };
        if temp_cx.locals.is_empty() {
            self.preamble_stmts.push(Stmt::Assign(place, AssignOp::Set, val));
        } else {
            // Put into a 0-arg helper fn called in preamble so locals stay clean.
            let fid = self.sigs.len();
            let init_fn_name = format!("cpl_init_g{gid}");
            self.sigs.push(FnSig {
                name: init_fn_name.clone(),
                params: Vec::new(),
                ret: Some(val_ty.clone()),
                owner: None,
                receiver: None,
                mod_id: tpl_mod,
                env: HashMap::new(),
                decl: self.dummy_fn_decl(),
                offset: tpl_offset,
                is_generator: false,
                new_domain_slot: None,
                state: State::Done,
            });
            self.bodies.push(Some(Function {
                name: init_fn_name,
                params: Vec::new(),
                ret: val_ty.clone(),
                locals: temp_cx.locals,
                body: Block {
                    stmts: vec![Stmt::Return(Some(val))],
                },
            }));
            let call = Expr {
                kind: ExprKind::Call(fid, Vec::new()),
                ty: val_ty.clone(),
                span: place.span,
            };
            self.preamble_stmts.push(Stmt::Assign(place, AssignOp::Set, call));
        }
        self.mk(ExprKind::Global(gid), val_ty, span)
    }

    /// Calls a generic function or generic struct method, inferring any omitted type arguments
    /// via two-pass unification (non-lambda arguments first, then lambda arguments).
    pub(super) fn call_gen_fn(
        &mut self,
        gfid: GenFnId,
        explicit: Vec<GenArg>,
        receiver: Option<Arg>,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let tpl_mod = self.gen_fns[gfid].mod_id;
        let tpl_offset = self.gen_fns[gfid].offset;
        let owner_env = self.gen_fns[gfid].owner_env.clone();
        let decl = self.gen_fns[gfid].decl;
        let gen_names: Vec<String> = decl.generics.iter().map(|g| g.text.clone()).collect();

        // Map call args to AST param definitions (skipping `self` if method).
        let ast_params: Vec<&ast::Param> = decl
            .params
            .iter()
            .enumerate()
            .filter(|(i, p)| !(*i == 0 && p.name.text == "self" && self.gen_fns[gfid].owner.is_some()))
            .map(|(_, p)| p)
            .collect();

        let mut inferred: GenEnv = owner_env.clone();
        for (name, val) in gen_names.iter().zip(&explicit) {
            inferred.insert(name.clone(), val.clone());
        }

        // Match call args to ast_params.
        let mut matched: Vec<Option<&ast::Arg>> = vec![None; ast_params.len()];
        let mut next = 0;
        for arg in args {
            let idx = match &arg.name {
                Some(n) => ast_params.iter().position(|p| p.name.text == n.text),
                None => {
                    while next < ast_params.len() && matched[next].is_some() {
                        next += 1;
                    }
                    (next < ast_params.len()).then_some(next)
                }
            };
            if let Some(i) = idx {
                matched[i] = Some(arg);
            }
        }

        // If a UFCS call passes a concrete `DomainVal(d)` as receiver (e.g. `D.prefix_range(4)`),
        // bind the first param's generic domain name (`self: D`) to `GenArg::Domain(d)`.
        if let Some(Arg::Value(Expr {
            kind: ExprKind::DomainVal(d),
            ..
        })) = &receiver
        {
            if let Some(first_param) = decl.params.first() {
                if let Some(ast::Type {
                    kind: ast::TypeKind::Named { name: pname, args: targs },
                    ..
                }) = &first_param.ty
                {
                    if targs.is_empty() && gen_names.contains(&pname.text) {
                        inferred.entry(pname.text.clone()).or_insert(GenArg::Domain(*d));
                    }
                }
            }
        }

        // Pass 1: unify from non-lambda arguments (and detect bare operators passed for unbound type params).
        for (param, arg) in ast_params.iter().zip(&matched) {
            let (Some(pty), Some(arg)) = (&param.ty, arg) else {
                continue;
            };
            if let ast::TypeKind::Named { name, args: targs } = &pty.kind {
                if targs.is_empty()
                    && gen_names.contains(&name.text)
                    && !inferred.contains_key(&name.text)
                {
                    if let Some(op) = self.detect_bare_op(&arg.value) {
                        inferred.insert(name.text.clone(), GenArg::Op(op));
                        continue;
                    }
                    if let Some(Sym::Domain(d)) = self.sym(&arg.value) {
                        inferred.insert(name.text.clone(), GenArg::Domain(d));
                        continue;
                    }
                }
            }
            if matches!(arg.value.kind, ast::ExprKind::Lambda { .. }) {
                continue;
            }
            if let Some(concrete) =
                self.probe_expr_ty(&arg.value, pty, tpl_mod, tpl_offset, &gen_names, &inferred)
            {
                if concrete == Ty::DomainVal {
                    if let ast::TypeKind::Named { name, args: targs } = &pty.kind {
                        if targs.is_empty() && gen_names.contains(&name.text) {
                            let saved_err = self.diags.len();
                            let ev = self.expr(&arg.value, None);
                            self.diags.truncate(saved_err);
                            if let ExprKind::DomainVal(d) = ev.kind {
                                inferred.entry(name.text.clone()).or_insert(GenArg::Domain(d));
                                continue;
                            }
                        }
                    }
                }
                self.unify_ast_ty(pty, &concrete, tpl_mod, &gen_names, &mut inferred);
            }
        }

        // Propagate domain param `V` from a bound `Graph<V, E>` (or any struct with a domain param),
        // and arc element type `A` from a bound `next: N` when `N` is a `Graph<V, E>` or fn.
        for g in &gen_names {
            if !inferred.contains_key(g) && (g == "V" || g == "D") {
                let dom_from_struct = inferred.values().find_map(|ga| {
                    if let GenArg::Ty(Ty::Struct(sid)) = ga {
                        self.structs[*sid].origin_gen.as_ref().and_then(|(_, gargs)| {
                            gargs.iter().find_map(|a| {
                                if let GenArg::Domain(d) = a {
                                    Some(*d)
                                } else {
                                    None
                                }
                            })
                        })
                    } else {
                        None
                    }
                });
                if let Some(d) = dom_from_struct {
                    inferred.insert(g.clone(), GenArg::Domain(d));
                }
            }
        }

        // Pass 2 (up to 3 rounds for chained lambdas like `dijkstra(adj, from, w)`):
        // unify from lambda arguments whose parameter types are now known.
        for _ in 0..3 {
            let before = inferred.len();
            // If an unbound generic (e.g. `T` in `fold_path<S, T>(..., ds: S)`) is present in a bound
            // struct argument's generic environment (e.g. `SegTree<Pre, Word>` has `T -> Word`), inherit it.
            for g in &gen_names {
                if !inferred.contains_key(g) {
                    let from_struct = inferred.values().find_map(|ga| match ga {
                        GenArg::Ty(Ty::Struct(sid)) => self.structs[*sid].env.get(g).cloned(),
                        _ => None,
                    });
                    if let Some(ga) = from_struct {
                        inferred.insert(g.clone(), ga);
                    }
                }
            }
            // If an unbound generic `T` is used as a lambda parameter type (e.g. `min_by<S, T, K>(it: S, key: T -> K)`),
            // deduce `T` from a bound container generic `S` (`List<Elem>`, `[D]Elem`, `Range<Elem>`, `Str`).
            for g in &gen_names {
                if !inferred.contains_key(g) && (g == "T" || g == "E" || g == "X") {
                    let elem_ty = inferred.values().find_map(|ga| match ga {
                        GenArg::Ty(Ty::List(e))
                        | GenArg::Ty(Ty::Array { elem: e, .. })
                        | GenArg::Ty(Ty::Range(e))
                        | GenArg::Ty(Ty::Set(e))
                        | GenArg::Ty(Ty::HashSet(e)) => Some((**e).clone()),
                        GenArg::Ty(Ty::Str) => Some(Ty::Char),
                        _ => None,
                    });
                    if let Some(ety) = elem_ty {
                        inferred.insert(g.clone(), GenArg::Ty(ety));
                    }
                }
            }
            // If an unbound generic `A` is used as a lambda parameter type (e.g. `w: A -> W` in `dijkstra`),
            // deduce `A` from a bound `N` (`Graph<V, E>`'s `out` element type, or `Fn(_, List<Elem>)`).
            for g in &gen_names {
                if !inferred.contains_key(g) && g == "A" {
                    let arc_ty = inferred.values().find_map(|ga| match ga {
                        GenArg::Ty(Ty::Struct(sid)) => {
                            let out_fn = self.structs[*sid].methods.get("out").copied()?;
                            if let Some(Ty::List(elem)) = self.sigs[out_fn].ret.clone() {
                                Some((*elem).clone())
                            } else {
                                None
                            }
                        }
                        GenArg::Ty(Ty::Fn(_, ret)) => {
                            if let Ty::List(elem) = ret.as_ref() {
                                Some((**elem).clone())
                            } else {
                                None
                            }
                        }
                        _ => None,
                    });
                    if let Some(aty) = arc_ty {
                        inferred.insert(g.clone(), GenArg::Ty(aty));
                    }
                }
            }
            for (param, arg) in ast_params.iter().zip(&matched) {
                let (Some(pty), Some(arg)) = (&param.ty, arg) else {
                    continue;
                };
                let ast::ExprKind::Lambda { params: lparams, body: lbody } = &arg.value.kind else {
                    continue;
                };
                if let ast::TypeKind::Named { name, args: targs } = &pty.kind {
                    if targs.is_empty()
                        && gen_names.contains(&name.text)
                        && !inferred.contains_key(&name.text)
                        && lparams.len() == 1
                    {
                        let dom_ty = inferred
                            .get("V")
                            .or_else(|| inferred.get("D"))
                            .and_then(|ga| match ga {
                                GenArg::Domain(d) => Some(Ty::Pos(*d)),
                                GenArg::Ty(t) => Some(t.clone()),
                                _ => None,
                            });
                        if let Some(vty) = dom_ty {
                            if let Some(ret_ty) = self.probe_lambda_ret(lparams, std::slice::from_ref(&vty), lbody) {
                                inferred.insert(
                                    name.text.clone(),
                                    GenArg::Ty(Ty::Fn(vec![vty], Box::new(ret_ty))),
                                );
                            }
                        }
                        continue;
                    }
                }
                if let ast::TypeKind::Fn { params: fparams, ret: fret } = &pty.kind {
                    // Check if all parameter types of this lambda can now be resolved.
                    let mut p_tys = Vec::new();
                    let mut all_known = fparams.len() == lparams.len();
                    if all_known {
                        let saved_mod = std::mem::replace(&mut self.cur_mod, tpl_mod);
                        let saved_offset = std::mem::replace(&mut self.offset, tpl_offset);
                        let saved_env = std::mem::replace(&mut self.gen_env, inferred.clone());
                        for fp in fparams {
                            if self.ast_ty_has_unbound(fp, &gen_names, &inferred) {
                                all_known = false;
                                break;
                            }
                            p_tys.push(self.resolve_ty(fp));
                        }
                        self.cur_mod = saved_mod;
                        self.offset = saved_offset;
                        self.gen_env = saved_env;
                    }
                    if all_known {
                        // Probe return type of the lambda body.
                        if let Some(ret_ty) = self.probe_lambda_ret(lparams, &p_tys, lbody) {
                            self.unify_ast_ty(fret, &ret_ty, tpl_mod, &gen_names, &mut inferred);
                        }
                    }
                }
            }
            if inferred.len() == before {
                break;
            }
        }

        // Handle `<T, S>(it: S) -> T` where `T` is the element type of `S` (or `S` is `List<T>` when `it` is `[]`).
        for (pi, p) in ast_params.iter().enumerate() {
            if let Some(arg) = args.get(pi) {
                if matches!(&arg.value.kind, ast::ExprKind::List(items) if items.is_empty()) {
                    if let Some(pty) = p.ty.as_ref() {
                        if let ast::TypeKind::Named { name, args: targs } = &pty.kind {
                            if targs.is_empty()
                                && gen_names.contains(&name.text)
                                && !inferred.contains_key(&name.text)
                            {
                                if let Some(GenArg::Ty(elem_ty)) = decl
                                    .ret
                                    .as_ref()
                                    .and_then(|r| match &r.kind {
                                        ast::TypeKind::Named { name: rn, args: ra } if ra.is_empty() => {
                                            inferred.get(&rn.text)
                                        }
                                        _ => None,
                                    })
                                    .cloned()
                                {
                                    inferred.insert(name.text.clone(), GenArg::Ty(Ty::list(elem_ty)));
                                }
                            }
                        }
                    }
                }
            }
        }
        if let Some(ret_ast) = decl.ret.as_ref() {
            if let ast::TypeKind::Named { name: rn, args: ra } = &ret_ast.kind {
                if ra.is_empty() && gen_names.contains(&rn.text) && !inferred.contains_key(&rn.text) {
                    let elem_from_container = inferred.values().find_map(|ga| match ga {
                        GenArg::Ty(Ty::List(e)) => Some((**e).clone()),
                        GenArg::Ty(Ty::Array { elem, .. }) => Some((**elem).clone()),
                        GenArg::Ty(Ty::Range(e)) => Some((**e).clone()),
                        GenArg::Ty(Ty::Str) => Some(Ty::Char),
                        _ => None,
                    });
                    if let Some(elem_ty) = elem_from_container {
                        inferred.insert(rn.text.clone(), GenArg::Ty(elem_ty));
                    }
                }
            }
        }

        let mut final_args = Vec::with_capacity(gen_names.len());
        for g in &gen_names {
            match inferred.get(g) {
                Some(a) => final_args.push(a.clone()),
                None => final_args.push(GenArg::Ty(Ty::Int)),
            }
        }
        let fn_id = self.instantiate_fn(gfid, final_args);
        self.user_call(fn_id, receiver, args, span)
    }

    /// Evaluates a struct-associated constant (e.g. `Mod<P>.zero`, `Mod<P>.one`).
    pub(super) fn eval_struct_const(
        &mut self,
        sid: StructId,
        sc: StructConst<'a>,
        span: Span,
    ) -> Expr {
        let saved_mod = std::mem::replace(&mut self.cur_mod, sc.mod_id);
        let saved_offset = std::mem::replace(&mut self.offset, sc.offset);
        let saved_self = self.self_struct.replace(sid);
        let mut env = self.gen_env.clone();
        if let Some((gid, gargs)) = self.structs[sid].origin_gen.clone() {
            for (p, a) in self.gen_structs[gid].decl.generics.iter().zip(gargs) {
                env.insert(p.text.clone(), a);
            }
        }
        let saved_env = std::mem::replace(&mut self.gen_env, env);
        let hint = sc.ty.map(|t| self.resolve_ty(t));
        let mut val = self.expr(sc.init, hint.as_ref());
        if let Some(h) = hint {
            val = self.coerce(val, &h, span);
        }
        self.cur_mod = saved_mod;
        self.offset = saved_offset;
        self.self_struct = saved_self;
        self.gen_env = saved_env;
        val
    }

    /// Lower a local `fn name(params) -> R: body` inside a block as a closure bound to `name`.
    pub(super) fn lower_local_fn(&mut self, decl: &ast::FnDecl, out: &mut Vec<Stmt>) {
        let param_tys: Vec<Ty> = decl
            .params
            .iter()
            .map(|p| {
                p.ty.as_ref()
                    .map(|t| self.resolve_ty(t))
                    .unwrap_or(Ty::Int)
            })
            .collect();
        let ret_ty = decl
            .ret
            .as_ref()
            .map(|r| self.resolve_ty(r))
            .unwrap_or(Ty::Unit);
        let fn_ty = Ty::Fn(param_tys.clone(), Box::new(ret_ty.clone()));
        let fn_slot = self.cx.locals.len();
        self.cx.locals.push(Local {
            name: decl.name.text.clone(),
            ty: fn_ty.clone(),
        });
        self.cx
            .scopes
            .last_mut()
            .unwrap()
            .insert(decl.name.text.clone(), fn_slot);

        let mut fresh = FnCx {
            is_closure: true,
            ..FnCx::default()
        };
        fresh.scopes.push(HashMap::new());
        fresh.local_syms.push(HashMap::new());
        fresh.ret = Some(ret_ty.clone());
        let outer_cx = std::mem::replace(&mut self.cx, fresh);
        self.outer_cxs.push(outer_cx);

        let mut param_slots = Vec::new();
        for (p, pty) in decl.params.iter().zip(param_tys) {
            let slot = self.cx.locals.len();
            self.cx.locals.push(Local {
                name: p.name.text.clone(),
                ty: pty,
            });
            if p.mutable {
                self.cx.mut_slots.insert(slot);
            }
            self.cx.scopes[0].insert(p.name.text.clone(), slot);
            param_slots.push(slot);
        }

        let tail = if ret_ty == Ty::Unit {
            Tail::Discard
        } else {
            Tail::Return
        };
        let body = match &decl.body {
            ast::FnBody::Block(block) => self.block(block, tail),
            ast::FnBody::Expr(e) => {
                let mut stmts = Vec::new();
                self.tail(e, tail, &mut stmts);
                Block { stmts }
            }
            ast::FnBody::None => Block::default(),
        };

        let inner_cx = std::mem::replace(&mut self.cx, self.outer_cxs.pop().unwrap());
        let cid = self.closures.len();
        self.closures.push(ClosureDef {
            params: param_slots,
            captures: inner_cx.captures,
            locals: inner_cx.locals,
            ret: ret_ty,
            body,
        });
        let init = self.mk(ExprKind::Closure(cid), fn_ty, decl.name.span);
        out.push(Stmt::Let(fn_slot, Some(init)));
    }

    /// Probes the concrete type of an expression without permanently emitting errors or side effects.
    pub(super) fn probe_ty_silent(&mut self, e: &ast::Expr) -> Option<Ty> {
        let saved_diags = self.diags.len();
        let saved_cx = self.cx.clone();
        let saved_outer = self.outer_cxs.clone();
        let val = self.expr(e, None);
        self.diags.truncate(saved_diags);
        self.cx = saved_cx;
        self.outer_cxs = saved_outer;
        (val.ty != Ty::Error).then_some(val.ty)
    }

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
        let probed = self.expr(e, hint.as_ref());
        self.cx = saved_cx;
        self.outer_cxs = saved_outer;
        self.diags.truncate(diag_len);
        (probed.ty != Ty::Error).then_some(probed.ty)
    }

    fn probe_lambda_ret(&mut self, lparams: &[ast::Pattern], p_tys: &[Ty], body: &ast::Expr) -> Option<Ty> {
        let diag_len = self.diags.len();
        let saved_cx = self.cx.clone();
        let saved_outer = self.outer_cxs.clone();
        self.cx.scopes.push(HashMap::new());
        let mut pre = Vec::new();
        for (pat, ty) in lparams.iter().zip(p_tys) {
            self.loop_var(pat, ty.clone(), &mut pre);
        }
        let probed = self.expr(body, None);
        self.cx = saved_cx;
        self.outer_cxs = saved_outer;
        self.diags.truncate(diag_len);
        (probed.ty != Ty::Error).then_some(probed.ty)
    }

    fn ast_ty_has_unbound(&self, ty: &ast::Type, gen_names: &[String], env: &GenEnv) -> bool {
        match &ty.kind {
            ast::TypeKind::Named { name, args } => {
                if args.is_empty() && gen_names.contains(&name.text) && !env.contains_key(&name.text) {
                    return true;
                }
                args.iter().any(|a| self.ast_ty_has_unbound(a, gen_names, env))
            }
            ast::TypeKind::Array { index, elem } => {
                index.iter().any(|d| self.ast_ty_has_unbound(d, gen_names, env))
                    || self.ast_ty_has_unbound(elem, gen_names, env)
            }
            ast::TypeKind::Optional(t) | ast::TypeKind::Yields(t) => {
                self.ast_ty_has_unbound(t, gen_names, env)
            }
            ast::TypeKind::Tuple(ts) => ts.iter().any(|t| self.ast_ty_has_unbound(t, gen_names, env)),
            ast::TypeKind::Record(fs) => fs.iter().any(|(_, t)| self.ast_ty_has_unbound(t, gen_names, env)),
            ast::TypeKind::Fn { params, ret } => {
                params.iter().any(|t| self.ast_ty_has_unbound(t, gen_names, env))
                    || self.ast_ty_has_unbound(ret, gen_names, env)
            }
            ast::TypeKind::Value(e) => {
                if let ast::ExprKind::Name(n) = &e.kind {
                    gen_names.contains(n) && !env.contains_key(n)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Unifies an AST type pattern `pat` against a concrete `Ty` to bind generic parameters.
    #[allow(clippy::only_used_in_recursion)]
    pub(super) fn unify_ast_ty(
        &self,
        pat: &ast::Type,
        concrete: &Ty,
        tpl_mod: ModId,
        gen_names: &[String],
        env: &mut GenEnv,
    ) {
        if *concrete == Ty::Error {
            return;
        }
        match &pat.kind {
            ast::TypeKind::Named { name, args } if args.is_empty() && gen_names.contains(&name.text) => {
                env.entry(name.text.clone()).or_insert_with(|| match concrete {
                    Ty::Pos(d) => GenArg::Domain(*d),
                    other => GenArg::Ty(other.clone()),
                });
            }
            ast::TypeKind::Named { name, args } => match (name.text.as_str(), concrete) {
                ("List" | "Iter", Ty::List(elem)) if args.len() == 1 => {
                    self.unify_ast_ty(&args[0], elem, tpl_mod, gen_names, env);
                }
                ("List" | "Iter", Ty::Str) if args.len() == 1 => {
                    self.unify_ast_ty(&args[0], &Ty::Char, tpl_mod, gen_names, env);
                }
                ("List" | "Iter", Ty::Array { elem, .. }) if args.len() == 1 => {
                    self.unify_ast_ty(&args[0], elem, tpl_mod, gen_names, env);
                }
                ("Set" | "HashSet", Ty::Set(elem) | Ty::HashSet(elem)) if args.len() == 1 => {
                    self.unify_ast_ty(&args[0], elem, tpl_mod, gen_names, env);
                }
                ("Map" | "HashMap", Ty::Map(k, v) | Ty::HashMap(k, v)) if args.len() == 2 => {
                    self.unify_ast_ty(&args[0], k, tpl_mod, gen_names, env);
                    self.unify_ast_ty(&args[1], v, tpl_mod, gen_names, env);
                }
                ("Range", Ty::Range(inner)) if args.len() == 1 => {
                    self.unify_ast_ty(&args[0], inner, tpl_mod, gen_names, env);
                }
                ("Bounds", Ty::Bound(d)) if args.len() == 1 => {
                    self.unify_ast_ty(&args[0], &Ty::Pos(*d), tpl_mod, gen_names, env);
                }
                (_, Ty::Struct(sid)) => {
                    if let Some((_, actual_args)) = &self.structs[*sid].origin_gen {
                        // Also allow structural subtype `Group<T>` -> `Monoid<T>` -> `Semigroup<T>`:
                        // their first generic arg is `T`!
                        for (pa, actual) in args.iter().zip(actual_args) {
                            match actual {
                                GenArg::Ty(t) => self.unify_ast_ty(pa, t, tpl_mod, gen_names, env),
                                GenArg::Domain(d) => self.unify_ast_ty(pa, &Ty::Pos(*d), tpl_mod, gen_names, env),
                                GenArg::Int(v) => {
                                    if let ast::TypeKind::Named { name: n, args: a } = &pa.kind {
                                        if a.is_empty() && gen_names.contains(&n.text) {
                                            env.entry(n.text.clone()).or_insert(GenArg::Int(*v));
                                        }
                                    } else if let ast::TypeKind::Value(e) = &pa.kind {
                                        if let ast::ExprKind::Name(n) = &e.kind {
                                            if gen_names.contains(n) {
                                                env.entry(n.clone()).or_insert(GenArg::Int(*v));
                                            }
                                        }
                                    }
                                }
                                GenArg::RuntimeInt(gid) => {
                                    if let ast::TypeKind::Named { name: n, args: a } = &pa.kind {
                                        if a.is_empty() && gen_names.contains(&n.text) {
                                            env.entry(n.text.clone()).or_insert(GenArg::RuntimeInt(*gid));
                                        }
                                    }
                                }
                                GenArg::Op(op) => {
                                    if let ast::TypeKind::Named { name: n, args: a } = &pa.kind {
                                        if a.is_empty() && gen_names.contains(&n.text) {
                                            env.entry(n.text.clone()).or_insert(GenArg::Op(*op));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            },
            ast::TypeKind::Array { index, elem } => {
                if let Ty::Array { dims, elem: ce } = concrete {
                    for (ix, d) in index.iter().zip(dims) {
                        match d {
                            Dim::Pos(did) | Dim::Bound(did) => {
                                self.unify_ast_ty(ix, &Ty::Pos(*did), tpl_mod, gen_names, env);
                            }
                            Dim::Int => {}
                        }
                    }
                    self.unify_ast_ty(elem, ce, tpl_mod, gen_names, env);
                }
            }
            ast::TypeKind::Optional(inner) => {
                if let Ty::Opt(ci) = concrete {
                    self.unify_ast_ty(inner, ci, tpl_mod, gen_names, env);
                } else {
                    self.unify_ast_ty(inner, concrete, tpl_mod, gen_names, env);
                }
            }
            ast::TypeKind::Tuple(items) => {
                if let Some(cfields) = concrete.fields() {
                    for (p, c) in items.iter().zip(&cfields) {
                        self.unify_ast_ty(p, c, tpl_mod, gen_names, env);
                    }
                }
            }
            ast::TypeKind::Record(fields) => {
                if let Some(cfields) = concrete.fields() {
                    for ((_, p), c) in fields.iter().zip(&cfields) {
                        self.unify_ast_ty(p, c, tpl_mod, gen_names, env);
                    }
                }
            }
            ast::TypeKind::Fn { params, ret } => {
                if let Ty::Fn(cp, cr) = concrete {
                    for (p, c) in params.iter().zip(cp) {
                        self.unify_ast_ty(p, c, tpl_mod, gen_names, env);
                    }
                    self.unify_ast_ty(ret, cr, tpl_mod, gen_names, env);
                }
            }
            _ => {}
        }
    }

    /// Lowers a lambda expression `(a, b) => body` into a `ClosureDef` with automatic capture resolution.
    pub(super) fn lower_lambda(
        &mut self,
        params: &[ast::Pattern],
        body: &ast::Expr,
        hint: Option<&Ty>,
        span: Span,
    ) -> Expr {
        let (param_tys, ret_hint) = match hint {
            Some(Ty::Fn(ps, r)) if ps.len() == params.len() => {
                let rh = if **r == Ty::Error { None } else { Some((**r).clone()) };
                (ps.clone(), rh)
            }
            _ => (vec![Ty::Int; params.len()], None),
        };

        // Push enclosing FnCx onto `outer_cxs` so lookups inside the lambda can capture variables.
        let mut child_cx = FnCx {
            ret: ret_hint.clone(),
            scopes: vec![HashMap::new()],
            is_closure: true,
            ..FnCx::default()
        };
        let mut param_slots = Vec::with_capacity(params.len());
        let mut pre_stmts = Vec::new();
        for (i, ty) in param_tys.iter().enumerate() {
            let pname = match &params[i].kind {
                ast::PatternKind::Name(n) => n.clone(),
                _ => format!("_p{i}"),
            };
            let slot = child_cx.locals.len();
            child_cx.locals.push(Local {
                name: pname.clone(),
                ty: ty.clone(),
            });
            if matches!(params[i].kind, ast::PatternKind::Name(_)) {
                child_cx.scopes[0].insert(pname, slot);
            }
            param_slots.push(slot);
        }

        let parent_cx = std::mem::replace(&mut self.cx, child_cx);
        self.outer_cxs.push(parent_cx);

        // Destructure any non-name parameter patterns (e.g. `((a, b)) => ...`).
        for (i, pat) in params.iter().enumerate() {
            if !matches!(pat.kind, ast::PatternKind::Name(_)) {
                let pexpr = self.local(param_slots[i], pat.span);
                self.bind(pat, pexpr, &mut pre_stmts);
            }
        }

        let mut body_stmts = pre_stmts;
        self.tail(body, Tail::Return, &mut body_stmts);
        let ret_ty = self.cx.ret.clone().unwrap_or(Ty::Unit);
        if ret_ty != Ty::Unit && !matches!(body_stmts.last(), Some(Stmt::Return(_))) {
            body_stmts.push(Stmt::Trap(
                "closure ended without a value".into(),
                self.span(span),
            ));
        }

        let finished = std::mem::replace(&mut self.cx, self.outer_cxs.pop().expect("parent cx"));
        let cid = self.closures.len();
        self.closures.push(ClosureDef {
            params: param_slots,
            captures: finished.captures,
            locals: finished.locals,
            ret: ret_ty.clone(),
            body: Block { stmts: body_stmts },
        });
        self.mk(
            ExprKind::Closure(cid),
            Ty::Fn(param_tys, Box::new(ret_ty)),
            span,
        )
    }

    /// Wraps a named function `fn_id` as a `Ty::Fn` closure value.
    pub(super) fn fn_as_closure(&mut self, fn_id: FnId, span: Span) -> Expr {
        self.queue_fn(fn_id);
        let params: Vec<Ty> = self.sigs[fn_id].params.iter().map(|p| p.ty.clone()).collect();
        let ret = self.ret_of(fn_id, span);
        let mut locals = Vec::new();
        let mut p_slots = Vec::new();
        let mut call_args = Vec::new();
        for (i, ty) in params.iter().enumerate() {
            locals.push(Local {
                name: format!("a{i}"),
                ty: ty.clone(),
            });
            p_slots.push(i);
            call_args.push(Arg::Value(Expr {
                kind: ExprKind::Local(i),
                ty: ty.clone(),
                span: self.span(span),
            }));
        }
        let call = Expr {
            kind: ExprKind::Call(fn_id, call_args),
            ty: ret.clone(),
            span: self.span(span),
        };
        let stmts = if ret == Ty::Unit {
            vec![Stmt::Expr(call)]
        } else {
            vec![Stmt::Return(Some(call))]
        };
        let cid = self.closures.len();
        self.closures.push(ClosureDef {
            params: p_slots,
            captures: Vec::new(),
            locals,
            ret: ret.clone(),
            body: Block { stmts },
        });
        self.mk(ExprKind::Closure(cid), Ty::Fn(params, Box::new(ret)), span)
    }

    /// Captures a local variable from an enclosing `FnCx` (possibly across multiple nested closures).
    pub(super) fn capture_from_outer(&mut self, name: &str) -> Option<Slot> {
        if !self.cx.is_closure {
            return None;
        }
        // Check if already captured in current closure.
        if let Some(&slot) = self.cx.captured_names.get(name) {
            return Some(slot);
        }
        // Find the innermost enclosing FnCx that defines `name` (either directly or as a field of `self`).
        let found_level = self.outer_cxs.iter().rposition(|cx| {
            cx.scopes.iter().rev().any(|s| s.contains_key(name))
        })?;
        // Thread the capture down from `found_level` to `self.cx`.
        let orig_slot = self.outer_cxs[found_level]
            .scopes
            .iter()
            .rev()
            .find_map(|s| s.get(name).copied())?;
        let ty = self.outer_cxs[found_level].locals[orig_slot].ty.clone();
        let by_ref = self.outer_cxs[found_level].mut_slots.contains(&orig_slot);

        let mut cur_outer_slot = orig_slot;
        for level in (found_level + 1)..self.outer_cxs.len() {
            let cx = &mut self.outer_cxs[level];
            if let Some(&existing) = cx.captured_names.get(name) {
                cur_outer_slot = existing;
            } else {
                let inner = cx.locals.len();
                cx.locals.push(Local {
                    name: name.to_string(),
                    ty: ty.clone(),
                });
                cx.captures.push(Capture {
                    outer: cur_outer_slot,
                    inner,
                    is_global: false,
                    by_ref,
                });
                cx.captured_names.insert(name.to_string(), inner);
                if by_ref {
                    cx.mut_slots.insert(inner);
                }
                cur_outer_slot = inner;
            }
        }
        let inner = self.cx.locals.len();
        self.cx.locals.push(Local {
            name: name.to_string(),
            ty,
        });
        self.cx.captures.push(Capture {
            outer: cur_outer_slot,
            inner,
            is_global: false,
            by_ref,
        });
        self.cx.captured_names.insert(name.to_string(), inner);
        if by_ref {
            self.cx.mut_slots.insert(inner);
        }
        Some(inner)
    }

    /// Captures `self` from an enclosing method into the current closure.
    pub(super) fn capture_self_from_outer(&mut self) -> Option<(Slot, StructId)> {
        if !self.cx.is_closure {
            return None;
        }
        let found_level = self.outer_cxs.iter().rposition(|cx| cx.receiver.is_some())?;
        let (_, sid) = self.outer_cxs[found_level].receiver?;
        let slot = self.capture_from_outer("self")?;
        Some((slot, sid))
    }

    /// Binds a receiver value to an instance method `fn_id`, producing a `Ty::Fn` closure value
    /// (e.g. `g.adj` passed as `V -> Iter<A>`).
    pub(super) fn bind_method_closure(&mut self, fn_id: FnId, recv: Expr, span: Span) -> Expr {
        self.queue_fn(fn_id);
        let recv_ty = recv.ty.clone();
        let params: Vec<Ty> = self.sigs[fn_id].params.iter().map(|p| p.ty.clone()).collect();
        let ret = self.ret_of(fn_id, span);
        let fn_ty = Ty::Fn(params.clone(), Box::new(ret.clone()));
        self.do_expr(Some(&fn_ty), span, |lw, out| {
            let outer_recv = lw.temp(recv_ty.clone());
            out.push(Stmt::Let(outer_recv, Some(recv)));
            let mut locals = Vec::new();
            let mut p_slots = Vec::new();
            let mut call_args = Vec::new();
            for (i, ty) in params.iter().enumerate() {
                locals.push(Local {
                    name: format!("a{i}"),
                    ty: ty.clone(),
                });
                p_slots.push(i);
            }
            let inner_recv = locals.len();
            locals.push(Local {
                name: "self".into(),
                ty: recv_ty.clone(),
            });
            call_args.push(Arg::Value(Expr {
                kind: ExprKind::Local(inner_recv),
                ty: recv_ty.clone(),
                span: lw.span(span),
            }));
            for (i, ty) in params.iter().enumerate() {
                call_args.push(Arg::Value(Expr {
                    kind: ExprKind::Local(i),
                    ty: ty.clone(),
                    span: lw.span(span),
                }));
            }
            let call = Expr {
                kind: ExprKind::Call(fn_id, call_args),
                ty: ret.clone(),
                span: lw.span(span),
            };
            let stmts = if ret == Ty::Unit {
                vec![Stmt::Expr(call)]
            } else {
                vec![Stmt::Return(Some(call))]
            };
            let cid = lw.closures.len();
            lw.closures.push(ClosureDef {
                params: p_slots,
                captures: vec![Capture {
                    outer: outer_recv,
                    inner: inner_recv,
                    is_global: false,
                    by_ref: false,
                }],
                locals,
                ret: ret.clone(),
                body: Block { stmts },
            });
            out.push(Stmt::Give(lw.mk(ExprKind::Closure(cid), fn_ty.clone(), span)));
        })
    }

    /// Looks up a binary or unary operator trait method on a concrete struct (`Add`, `Sub`, `Mul`, `Div`, `Neg`).
    pub(super) fn struct_op_fn(&mut self, sid: StructId, method: &str) -> Option<FnId> {
        let id = self.structs[sid].methods.get(method).copied()?;
        self.queue_fn(id);
        Some(id)
    }

    /// Inlines a `lends` / `lends mut` projection for a **read** expression: `base.proj(args)`.
    pub(super) fn lower_projection_read(
        &mut self,
        proj: ProjectionInfo<'a>,
        base: &ast::Expr,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let recv_place = self.try_place(base);
        let recv_val = if recv_place.is_none() {
            Some(self.expr(base, None))
        } else {
            None
        };
        let (pre_stmts, lend_place_ast, post_stmts) = split_projection_body(proj.decl);
        let Some(lend_place_ast) = lend_place_ast else {
            self.error(codes::UNSUPPORTED, span, "projection has no `lend` statement");
            return self.err_expr(span);
        };

        self.do_expr(None, span, |lw, out| {
            let p_place = match recv_place {
                Some(p) => p,
                None => {
                    let v = recv_val.unwrap();
                    let slot = lw.temp(v.ty.clone());
                    let sp = v.span;
                    out.push(Stmt::Let(slot, Some(v)));
                    Place {
                        kind: PlaceKind::Local(slot),
                        ty: Ty::Struct(proj.owner),
                        span: sp,
                    }
                }
            };
            lw.with_projection_scope(&proj, p_place, args, out, |lw, out| {
                for s in pre_stmts {
                    lw.stmt(s, out);
                }
                let val = lw.expr(lend_place_ast, None);
                let result_slot = lw.temp(val.ty.clone());
                let res_ty = val.ty.clone();
                out.push(Stmt::Let(result_slot, Some(val)));
                for s in post_stmts {
                    lw.stmt(s, out);
                }
                if let Some(frame) = lw.cx.dos.last_mut() {
                    frame.ty = Some(res_ty);
                }
                let res = lw.local(result_slot, span);
                out.push(Stmt::Give(res));
            });
        })
    }

    /// Inlines a `lends mut` projection for an **assignment** (`base.proj(args) op= rhs`).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn lower_projection_assign(
        &mut self,
        proj: ProjectionInfo<'a>,
        base: &ast::Expr,
        args: &[ast::Arg],
        op: ast::AssignOp,
        rhs: &ast::Expr,
        span: Span,
        out: &mut Vec<Stmt>,
    ) {
        let rhs_val = self.expr(rhs, None);
        let rhs_slot = self.temp(rhs_val.ty.clone());
        let rhs_ty = rhs_val.ty.clone();
        out.push(Stmt::Let(rhs_slot, Some(rhs_val)));

        let recv_place = self.place(base);
        let (pre_stmts, lend_place_ast, post_stmts) = split_projection_body(proj.decl);
        let Some(lend_place_ast) = lend_place_ast else {
            self.error(codes::UNSUPPORTED, span, "projection has no `lend` statement");
            return;
        };

        self.with_projection_scope(&proj, recv_place, args, out, |lw, out| {
            for s in pre_stmts {
                lw.stmt(s, out);
            }
            let target_place = lw.place(lend_place_ast);
            let val_expr = lw.local(rhs_slot, rhs.span);
            let coerced = lw.coerce(val_expr, &target_place.ty, rhs.span);
            lw.emit_assign_place(target_place, op, coerced, span, out);
            for s in post_stmts {
                lw.stmt(s, out);
            }
        });
        let _ = rhs_ty;
    }

    /// Inlines a `lends mut` projection passed as a `mut` call argument (`f(mut base.proj(args))`).
    pub(super) fn lower_projection_mut_call(
        &mut self,
        proj: ProjectionInfo<'a>,
        base: &ast::Expr,
        args: &[ast::Arg],
        span: Span,
        out: &mut Vec<Stmt>,
        post_out: &mut Vec<Stmt>,
    ) -> Place {
        let recv_place = self.place(base);
        let (pre_stmts, lend_place_ast, post_stmts) = split_projection_body(proj.decl);
        let Some(lend_place_ast) = lend_place_ast else {
            self.error(codes::UNSUPPORTED, span, "projection has no `lend` statement");
            return Place {
                kind: PlaceKind::Local(0),
                ty: Ty::Error,
                span: self.span(span),
            };
        };
        let mut lent_place = None;
        self.with_projection_scope(&proj, recv_place, args, out, |lw, out| {
            for s in pre_stmts {
                lw.stmt(s, out);
            }
            lent_place = Some(lw.place(lend_place_ast));
            for s in post_stmts {
                lw.stmt(s, post_out);
            }
        });
        lent_place.unwrap()
    }

    fn with_projection_scope(
        &mut self,
        proj: &ProjectionInfo<'a>,
        recv_place: Place,
        call_args: &[ast::Arg],
        out: &mut Vec<Stmt>,
        f: impl FnOnce(&mut Self, &mut Vec<Stmt>),
    ) {
        // Bind arguments in the caller's module/env first.
        let ast_params: Vec<&ast::Param> = proj
            .decl
            .params
            .iter()
            .enumerate()
            .filter(|(i, p)| !(*i == 0 && p.name.text == "self"))
            .map(|(_, p)| p)
            .collect();

        // Resolve parameter types in the projection's module/env.
        let saved_mod = std::mem::replace(&mut self.cur_mod, proj.mod_id);
        let saved_offset = std::mem::replace(&mut self.offset, proj.offset);
        let saved_env = std::mem::replace(&mut self.gen_env, proj.env.clone());
        let saved_self = self.self_struct.replace(proj.owner);
        let param_tys: Vec<(String, Ty)> = ast_params
            .iter()
            .map(|p| {
                let ty = p.ty.as_ref().map(|t| self.resolve_ty(t)).unwrap_or(Ty::Int);
                (p.name.text.clone(), ty)
            })
            .collect();
        self.cur_mod = saved_mod;
        self.offset = saved_offset;
        self.gen_env = saved_env;
        self.self_struct = saved_self;

        // Evaluate arguments in caller context.
        let mut arg_slots = Vec::new();
        for ((pname, pty), arg) in param_tys.iter().zip(call_args) {
            let v = self.expect(&arg.value, pty);
            let slot = self.new_slot(pname, pty.clone());
            out.push(Stmt::Let(slot, Some(v)));
            arg_slots.push((pname.clone(), slot));
        }

        // Now switch into projection's module/env with `inline_self` active.
        let saved_mod = std::mem::replace(&mut self.cur_mod, proj.mod_id);
        let saved_offset = std::mem::replace(&mut self.offset, proj.offset);
        let saved_env = std::mem::replace(&mut self.gen_env, proj.env.clone());
        let saved_self = self.self_struct.replace(proj.owner);
        let saved_inline = self.cx.inline_self.replace((recv_place, proj.owner));
        self.cx.scopes.push(HashMap::new());
        for (pname, slot) in arg_slots {
            self.cx.scopes.last_mut().unwrap().insert(pname, slot);
        }

        f(self, out);

        self.cx.scopes.pop();
        self.cx.inline_self = saved_inline;
        self.cur_mod = saved_mod;
        self.offset = saved_offset;
        self.gen_env = saved_env;
        self.self_struct = saved_self;
    }

    /// Lowers `laws_hold(obj, samples_or_kw...)` into nested assertion loops checking every `law`.
    pub(super) fn lower_laws_hold(&mut self, args: &[ast::Arg], span: Span) -> Expr {
        if args.is_empty() {
            self.error(codes::WRONG_ARG_COUNT, span, "`laws_hold` takes an operation value and samples");
            return self.err_expr(span);
        }
        let obj = self.expr(&args[0].value, None);
        let Ty::Struct(sid) = obj.ty else {
            self.error(codes::TYPE_MISMATCH, args[0].value.span, "`laws_hold` expects a struct with `law`s");
            return self.err_expr(span);
        };
        let laws = self.structs[sid].laws.clone();
        let struct_env = self.structs[sid].env.clone();
        let struct_fields = self.structs[sid].def.fields.clone();

        // Lower positional default samples and named sample lists (e.g. `T: [...], F: [...]`).
        let mut default_samples: Option<Expr> = None;
        let mut named_samples: HashMap<String, Expr> = HashMap::new();
        for arg in &args[1..] {
            let v = self.expr(&arg.value, None);
            if let Some(n) = &arg.name {
                named_samples.insert(n.text.clone(), v);
            } else {
                default_samples = Some(v);
            }
        }

        self.do_expr(Some(&Ty::Bool), span, |lw, out| {
            let obj_slot = lw.temp(Ty::Struct(sid));
            out.push(Stmt::Let(obj_slot, Some(obj)));

            // Materialize sample lists in locals once.
            let def_slot = default_samples.map(|e| {
                let s = lw.temp(e.ty.clone());
                out.push(Stmt::Let(s, Some(e)));
                s
            });
            let mut kw_slots: HashMap<String, Slot> = HashMap::new();
            for (k, e) in named_samples {
                let s = lw.temp(e.ty.clone());
                out.push(Stmt::Let(s, Some(e)));
                kw_slots.insert(k, s);
            }

            // Bind fields of `obj` into locals so `op`, `id`, `inv`, `lt`, `compose`, `apply` work in laws.
            lw.cx.scopes.push(HashMap::new());
            for (fi, (fname, fty)) in struct_fields.iter().enumerate() {
                let fslot = lw.new_slot(fname, fty.clone());
                let fexpr = Expr {
                    kind: ExprKind::Field(Box::new(lw.local(obj_slot, span)), fi),
                    ty: fty.clone(),
                    span: lw.span(span),
                };
                out.push(Stmt::Let(fslot, Some(fexpr)));
                lw.cx.scopes.last_mut().unwrap().insert(fname.clone(), fslot);
            }
            let saved_recv = lw.cx.receiver.replace((obj_slot, sid));

            for law in &laws {
                let saved_mod = std::mem::replace(&mut lw.cur_mod, law.mod_id);
                let saved_offset = std::mem::replace(&mut lw.offset, law.offset);
                let saved_env = std::mem::replace(&mut lw.gen_env, struct_env.clone());
                let saved_self = lw.self_struct.replace(sid);

                // Resolve each law parameter's sample list slot.
                let mut pools: Vec<(String, Slot, Ty)> = Vec::new();
                for p in law.params {
                    let tn = match p.ty.as_ref().map(|t| &t.kind) {
                        Some(ast::TypeKind::Named { name, args }) if args.is_empty() => {
                            Some(name.text.as_str())
                        }
                        _ => None,
                    };
                    let slot = tn
                        .and_then(|n| kw_slots.get(n).copied())
                        .or(def_slot);
                    if let Some(s) = slot {
                        let elem_ty = match &lw.cx.locals[s].ty {
                            Ty::List(e) => (**e).clone(),
                            Ty::Array { elem, .. } => (**elem).clone(),
                            other => other.clone(),
                        };
                        pools.push((p.name.text.clone(), s, elem_ty));
                    }
                }
                if pools.len() == law.params.len() {
                    let stmts = lw.emit_law_loops(&pools, 0, law, span);
                    out.extend(stmts);
                }

                lw.cur_mod = saved_mod;
                lw.offset = saved_offset;
                lw.gen_env = saved_env;
                lw.self_struct = saved_self;
            }

            lw.cx.receiver = saved_recv;
            lw.cx.scopes.pop();
            let ok = lw.mk(ExprKind::Bool(true), Ty::Bool, span);
            out.push(Stmt::Give(ok));
        })
    }

    fn emit_law_loops(
        &mut self,
        pools: &[(String, Slot, Ty)],
        idx: usize,
        law: &LawInfo<'a>,
        span: Span,
    ) -> Vec<Stmt> {
        if idx == pools.len() {
            let cond = self.expect(law.body, &Ty::Bool);
            return vec![Stmt::Assert {
                cond,
                message: None,
                text: format!("law {}", law.name),
                line: self.line(law.body.span),
                span: self.span(span),
            }];
        }
        let (pname, pool_slot, elem_ty) = &pools[idx];
        let var_slot = self.new_slot(pname, elem_ty.clone());
        let lid = self.open_loop(None);
        self.cx.scopes.push(HashMap::new());
        self.cx.scopes.last_mut().unwrap().insert(pname.clone(), var_slot);
        let body = self.emit_law_loops(pools, idx + 1, law, span);
        self.cx.scopes.pop();
        self.cx.loops.pop();
        let seq = self.local(*pool_slot, span);
        vec![Stmt::For(
            lid,
            var_slot,
            Iter::Seq(seq, false),
            Block { stmts: body },
        )]
    }

    /// Checks whether a struct `have_id` structurally contains every field of `want_id` (e.g., `Group<T>` -> `Monoid<T>`).
    pub(super) fn coerce_struct_projection(&mut self, e: Expr, have_id: StructId, want_id: StructId) -> Option<Expr> {
        let want_fields = self.structs[want_id].def.fields.clone();
        let have_fields = &self.structs[have_id].def.fields;
        let mut mapping = Vec::with_capacity(want_fields.len());
        for (wname, wty) in &want_fields {
            let pos = have_fields.iter().position(|(hname, hty)| hname == wname && hty == wty)?;
            mapping.push((pos, wty.clone()));
        }
        let span = e.span;
        let tmp = self.temp(Ty::Struct(have_id));
        let mut stmts = vec![Stmt::Let(tmp, Some(e))];
        let mut vals = Vec::with_capacity(mapping.len());
        for (pos, fty) in mapping {
            vals.push(Expr {
                kind: ExprKind::Field(
                    Box::new(Expr {
                        kind: ExprKind::Local(tmp),
                        ty: Ty::Struct(have_id),
                        span,
                    }),
                    pos,
                ),
                ty: fty,
                span,
            });
        }
        stmts.push(Stmt::Give(Expr {
            kind: ExprKind::Tuple(vals),
            ty: Ty::Struct(want_id),
            span,
        }));
        Some(Expr {
            kind: ExprKind::Do(Block { stmts }),
            ty: Ty::Struct(want_id),
            span,
        })
    }

    fn dummy_fn_decl(&self) -> &'a ast::FnDecl {
        // Only used for synthetic preamble init fns whose `state` is already `State::Done`.
        for m in &self.mods {
            for item in &m.items {
                for s in &item.syntax.parsed.stmts {
                    if let ast::StmtKind::Fn(f) = &s.kind {
                        return f;
                    }
                    if let ast::StmtKind::Struct(st) = &s.kind {
                        for m in &st.methods {
                            if let ast::StmtKind::Fn(f) = &m.kind {
                                return f;
                            }
                        }
                    }
                }
            }
        }
        unreachable!("mod const requires at least one fn in loaded modules")
    }
}

fn split_projection_body(
    decl: &ast::FnDecl,
) -> (&[ast::Stmt], Option<&ast::Expr>, &[ast::Stmt]) {
    let ast::FnBody::Block(block) = &decl.body else {
        return (&[], None, &[]);
    };
    for (i, stmt) in block.stmts.iter().enumerate() {
        if let ast::StmtKind::Lend { place, .. } = &stmt.kind {
            return (&block.stmts[..i], Some(place), &block.stmts[i + 1..]);
        }
    }
    (&[], None, &[])
}

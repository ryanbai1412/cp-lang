//! From the front end's syntax trees to a typed [`Program`].
//!
//! The front end's checker is deliberately forgiving: what it can't type is
//! `Unknown` and passes. A back end can't run `Unknown`, so this pass types
//! every expression for real, loads imported modules, monomorphizes generic
//! structs/enums/fns/consts, lowers traits and `impl`s, desugars `lends`
//! projections and generators, and emits one concrete [`Program`] shared by
//! `cpl-interp` and `cpl-codegen`.

mod cache;
mod call;
mod expr;
mod mono;
mod stmt;

use crate::*;
use cpl_check::Analysis;
use cpl_diag::{codes, Diagnostic, Span};
use cpl_syntax::ast;
use mono::*;
use std::collections::{HashMap, HashSet};

/// Lists the names of all `@test` functions declared at the top level of `analysis`.
pub fn list_tests(analysis: &Analysis) -> Vec<String> {
    let mut out = Vec::new();
    for item in &analysis.items {
        for stmt in &item.syntax.parsed.stmts {
            if stmt.decorators.iter().any(|d| d.name.text == "test") {
                if let ast::StmtKind::Fn(decl) = &stmt.kind {
                    out.push(decl.name.text.clone());
                }
            }
        }
    }
    out
}

/// Lowers a file that the front end reported no errors for.
pub fn lower(analysis: &Analysis) -> Result<Program, Vec<Diagnostic>> {
    lower_impl(analysis, None)
}

/// Lowers a single `@test` function of `analysis` as the program's entrypoint.
pub fn lower_test(analysis: &Analysis, test_name: &str) -> Result<Program, Vec<Diagnostic>> {
    lower_impl(analysis, Some(test_name))
}

fn lower_impl(analysis: &Analysis, test_name: Option<&str>) -> Result<Program, Vec<Diagnostic>> {
    let mut lw = Lowerer::new(analysis);
    lw.load_modules(analysis);
    lw.collect_all(test_name);
    lw.signatures();
    if let Some(tname) = test_name {
        lw.main_for_test(tname);
    } else {
        lw.main();
    }
    // Queue root-module top-level functions (for non-test runs) plus any
    // demand-driven functions referenced from `main`, closures, preamble, or transitive callees.
    if test_name.is_none() {
        for id in 0..lw.sigs.len() {
            // A fn with no body is the judge's: there is nothing to lower
            // until something calls it.
            let bodyless = matches!(lw.sigs[id].decl.body, ast::FnBody::None);
            if lw.sigs[id].mod_id == 0 && lw.sigs[id].owner.is_none() && !bodyless {
                lw.queue_fn(id);
            }
        }
    }
    loop {
        while let Some(id) = lw.queued_fns.pop() {
            lw.function(id);
        }
        let mut newly_called = Vec::new();
        collect_block_calls(&lw.main_body, &mut newly_called);
        for s in &lw.preamble_stmts {
            collect_stmt_calls(s, &mut newly_called);
        }
        for c in &lw.closures {
            collect_block_calls(&c.body, &mut newly_called);
        }
        for body in lw.bodies.iter().flatten() {
            collect_block_calls(&body.body, &mut newly_called);
        }
        for s in &lw.structs {
            for hook in [s.def.eq_fn, s.def.lt_fn, s.def.show_fn].into_iter().flatten() {
                newly_called.push(hook);
            }
        }
        let mut added = false;
        for fid in newly_called {
            if lw.queued_set.insert(fid) {
                lw.queued_fns.push(fid);
                added = true;
            }
        }
        if !added {
            break;
        }
    }
    // Prepend any lazy imported module const initializers to `main`.
    if !lw.preamble_stmts.is_empty() {
        let mut combined = std::mem::take(&mut lw.preamble_stmts);
        combined.extend(lw.main_body.stmts);
        lw.main_body.stmts = combined;
    }
    if !lw.diags.is_empty() {
        lw.diags.sort_by_key(|d| (d.span.start, d.span.end));
        lw.diags.dedup();
        return Err(lw.diags);
    }
    // Ensure any unlowered non-generic fns from imported modules (unused) don't panic:
    // keep only lowered fns or fill unused bodies with empty blocks, and deduplicate names.
    let mut used_fn_names: HashSet<String> = HashSet::new();
    let fns = lw
        .bodies
        .into_iter()
        .enumerate()
        .map(|(i, body)| {
            let mut f = body.unwrap_or_else(|| Function {
                name: lw.sigs[i].name.clone(),
                params: Vec::new(),
                ret: Ty::Unit,
                locals: Vec::new(),
                body: Block::default(),
            });
            if !used_fn_names.insert(f.name.clone()) {
                f.name = format!("{}_{i}", f.name);
                used_fn_names.insert(f.name.clone());
            }
            f
        })
        .collect();
    Ok(Program {
        structs: lw.structs.iter().map(|s| s.def.clone()).collect(),
        enums: lw.enums,
        domains: lw.domains,
        globals: lw.globals,
        fns,
        closures: lw.closures,
        main: lw.main_body,
        main_locals: lw.main_locals,
        multitest: lw.multitest,
        interactive: lw.interactive,
    })
}

#[derive(Clone, Copy, Debug)]
enum Sym {
    Global(GlobalId),
    Fn(FnId),
    GenFn(GenFnId),
    Struct(StructId),
    GenStruct(GenStructId),
    Enum(EnumId),
    GenEnum(GenEnumId),
    Variant(EnumId, usize),
    Domain(DomainId),
    Alias(usize),
    GenAlias(GenAliasId),
    GenConst(GenConstId),
    ModConst(ModConstId),
    Trait(TraitId),
    Mod(ModId),
}

struct StructInfo<'a> {
    def: StructDef,
    decl: &'a ast::StructDecl,
    mod_id: ModId,
    offset: u32,
    origin_gen: Option<(GenStructId, Vec<GenArg>)>,
    env: GenEnv,
    methods: HashMap<String, FnId>,
    gen_methods: HashMap<String, GenFnId>,
    getters: HashSet<String>,
    projections: HashMap<String, ProjectionInfo<'a>>,
    consts: HashMap<String, StructConst<'a>>,
    owned_domains: HashMap<String, DomainId>,
    owned_domain_inits: Vec<OwnedDomainInit<'a>>,
    laws: Vec<LawInfo<'a>>,
    /// Backing domain id when `impl Index for S` is defined.
    index_domain: Option<DomainId>,
}

#[derive(Clone)]
struct SigParam<'a> {
    name: String,
    ty: Ty,
    mutable: bool,
    default: Option<&'a ast::Expr>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Pending,
    InProgress,
    Done,
}

struct FnSig<'a> {
    name: String,
    /// Without `self`.
    params: Vec<SigParam<'a>>,
    /// `None` until the body has been read, for a fn without `->`.
    ret: Option<Ty>,
    /// The struct whose method this is, and whether it takes `self`
    /// (`Some(mutable)`).
    owner: Option<StructId>,
    receiver: Option<bool>,
    mod_id: ModId,
    env: GenEnv,
    decl: &'a ast::FnDecl,
    offset: u32,
    is_generator: bool,
    /// If the return type is `(domain C, ...)`, the allocated DomainId + name.
    new_domain_slot: Option<(String, DomainId)>,
    state: State,
}

/// What a block is for, which decides what happens to its last expression.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Tail {
    Discard,
    Return,
    Give,
}

#[derive(Clone)]
struct DoFrame {
    ty: Option<Ty>,
    /// Loops opened before the block: `break` can't reach them.
    loops: usize,
}

#[derive(Clone, Default)]
struct FnCx {
    locals: Vec<Local>,
    scopes: Vec<HashMap<String, Slot>>,
    /// Block-scoped type aliases, local domains, etc. (e.g. `domain V = 1..=6`, `type M = Mod<7>`).
    local_syms: Vec<HashMap<String, Sym>>,
    /// Parameters bound to compile-time `BareOp` values (`+`, `min`, `max`, `gcd`, etc.).
    op_locals: HashMap<String, BareOp>,
    /// Block-scoped `type Alias = ...` resolved to concrete types (e.g. `type Cell = (Row, Col)`).
    local_type_aliases: HashMap<String, Ty>,
    /// Local slots declared `var` or mutated, so closures capture them by reference.
    mut_slots: HashSet<Slot>,
    ret: Option<Ty>,
    /// `self`'s slot and struct inside a method.
    receiver: Option<(Slot, StructId)>,
    /// Inlined `self` place when desugaring a `lends` / `lends mut` projection.
    inline_self: Option<(Place, StructId)>,
    /// Hidden list accumulator when lowering a `-> yields T` generator.
    yield_slot: Option<(Slot, Ty)>,
    loops: Vec<(Option<String>, LoopId)>,
    dos: Vec<DoFrame>,
    /// The `x ?? return` that is the whole right side of the statement
    /// being lowered, and the test it puts ahead of that statement.
    way_out: Option<Span>,
    pending: Vec<Stmt>,
    is_main: bool,
    is_closure: bool,
    captures: Vec<Capture>,
    captured_names: HashMap<String, Slot>,
}

struct Lowerer<'a> {
    mods: Vec<&'a Analysis>,
    path_to_mod: HashMap<String, ModId>,
    mod_syms: Vec<HashMap<String, Sym>>,
    cur_mod: ModId,
    diags: Vec<Diagnostic>,

    structs: Vec<StructInfo<'a>>,
    gen_structs: Vec<GenStructTemplate<'a>>,
    struct_insts: HashMap<(GenStructId, Vec<GenArg>), StructId>,

    enums: Vec<EnumDef>,
    enum_decls: Vec<(ModId, u32, &'a ast::EnumDecl)>,
    gen_enums: Vec<GenEnumTemplate<'a>>,
    enum_insts: HashMap<(GenEnumId, Vec<GenArg>), EnumId>,

    domains: Vec<String>,
    aliases: Vec<(ModId, &'a ast::Type, u32)>,
    gen_aliases: Vec<GenAliasTemplate<'a>>,

    globals: Vec<Global>,
    const_ints: HashMap<String, i64>,
    mod_consts: Vec<ModConstInfo<'a>>,
    gen_consts: Vec<GenConstTemplate<'a>>,

    traits: Vec<TraitInfo<'a>>,
    impls: Vec<ImplInfo<'a>>,

    sigs: Vec<FnSig<'a>>,
    bodies: Vec<Option<Function>>,
    gen_fns: Vec<GenFnTemplate<'a>>,
    fn_insts: HashMap<(GenFnId, Vec<GenArg>), FnId>,

    closures: Vec<ClosureDef>,
    queued_fns: Vec<FnId>,
    queued_set: HashSet<FnId>,
    preamble_stmts: Vec<Stmt>,
    main_body: Block,
    main_locals: Vec<Local>,
    multitest: bool,
    interactive: bool,

    cx: FnCx,
    outer_cxs: Vec<FnCx>,
    gen_env: GenEnv,
    self_struct: Option<StructId>,
    /// Start of the item being lowered; its spans are relative to it.
    offset: u32,
    next_loop: LoopId,
}

impl<'a> Lowerer<'a> {
    fn new(_analysis: &'a Analysis) -> Lowerer<'a> {
        Lowerer {
            mods: Vec::new(),
            path_to_mod: HashMap::new(),
            mod_syms: Vec::new(),
            cur_mod: 0,
            diags: Vec::new(),
            structs: Vec::new(),
            gen_structs: Vec::new(),
            struct_insts: HashMap::new(),
            enums: Vec::new(),
            enum_decls: Vec::new(),
            gen_enums: Vec::new(),
            enum_insts: HashMap::new(),
            domains: Vec::new(),
            aliases: Vec::new(),
            gen_aliases: Vec::new(),
            globals: Vec::new(),
            const_ints: HashMap::new(),
            mod_consts: Vec::new(),
            gen_consts: Vec::new(),
            traits: Vec::new(),
            impls: Vec::new(),
            sigs: Vec::new(),
            bodies: Vec::new(),
            gen_fns: Vec::new(),
            fn_insts: HashMap::new(),
            closures: Vec::new(),
            queued_fns: Vec::new(),
            queued_set: HashSet::new(),
            preamble_stmts: Vec::new(),
            main_body: Block::default(),
            main_locals: Vec::new(),
            multitest: false,
            interactive: false,
            cx: FnCx::default(),
            outer_cxs: Vec::new(),
            gen_env: HashMap::new(),
            self_struct: None,
            offset: 0,
            next_loop: 0,
        }
    }

    // ----- reporting -----

    fn span(&self, span: Span) -> Span {
        if self.cur_mod == 0 {
            span.shift(self.offset)
        } else {
            Span::default()
        }
    }

    fn error(&mut self, code: cpl_diag::Code, span: Span, message: impl Into<String>) {
        let span = self.span(span);
        self.diags.push(Diagnostic::error(code, span, message));
    }

    fn help(&mut self, help: impl Into<String>) {
        if let Some(last) = self.diags.last_mut() {
            last.help = Some(help.into());
        }
    }

    /// Reports something the language has but the back ends don't yet.
    fn unsupported(&mut self, span: Span, what: &str) {
        self.error(
            codes::UNSUPPORTED,
            span,
            format!("`cpl run` and `cpl build` can't handle {what} yet"),
        );
    }

    fn mismatch(&mut self, span: Span, want: &Ty, found: &Ty) {
        let message = format!(
            "expected `{}`, found `{}`",
            self.show(want),
            self.show(found)
        );
        self.error(codes::TYPE_MISMATCH, span, message);
    }

    fn show(&self, ty: &Ty) -> String {
        let structs: Vec<StructDef> = self.structs.iter().map(|s| s.def.clone()).collect();
        crate::show::ty_name(ty, &self.domains, &structs, &self.enums)
    }

    fn err_expr(&self, span: Span) -> Expr {
        Expr {
            kind: ExprKind::Error,
            ty: Ty::Error,
            span: self.span(span),
        }
    }

    // ----- declarations across all modules -----

    fn collect_all(&mut self, active_test: Option<&str>) {
        let num_mods = self.mods.len();
        // Pass 1: collect declarations in every module.
        for m in 0..num_mods {
            self.cur_mod = m;
            let items: Vec<(u32, &'a ast::Stmt)> = self.mod_items(m).collect();
            for (offset, stmt) in items {
                self.offset = offset;
                if m == 0 {
                    for decorator in &stmt.decorators {
                        match decorator.name.text.as_str() {
                            "multitest" => self.multitest = true,
                            "get" | "brute" | "test" | "solution" | "tags" | "memo" | "law" | "gen" | "checker"
                            | "interactor" | "grader" | "" => {}
                            other => self.unsupported(
                                decorator.span,
                                &format!("the `@{other}` decorator"),
                            ),
                        }
                    }
                }
                match &stmt.kind {
                    ast::StmtKind::Struct(decl) => {
                        if !decl.generics.is_empty() {
                            let gid = self.gen_structs.len();
                            self.gen_structs.push(GenStructTemplate {
                                mod_id: m,
                                offset,
                                decl,
                            });
                            self.mod_syms[m]
                                .insert(decl.name.text.clone(), Sym::GenStruct(gid));
                        } else {
                            let id = self.structs.len();
                            self.mod_syms[m].insert(decl.name.text.clone(), Sym::Struct(id));
                            let sname = if m == 0 {
                                decl.name.text.clone()
                            } else {
                                format!("m{m}_{}", decl.name.text)
                            };
                            self.structs.push(StructInfo {
                                def: StructDef {
                                    name: sname,
                                    fields: Vec::new(),
                                    show_fn: None,
                                    eq_fn: None,
                                    lt_fn: None,
                                },
                                decl,
                                mod_id: m,
                                offset,
                                origin_gen: None,
                                env: HashMap::new(),
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
                                                    mod_id: m,
                                                    offset,
                                                    owner: id,
                                                    env: HashMap::new(),
                                                    decl: f,
                                                },
                                            );
                                        } else if !f.generics.is_empty() {
                                            let gfid = self.gen_fns.len();
                                            self.gen_fns.push(GenFnTemplate {
                                                mod_id: m,
                                                offset,
                                                owner: Some(id),
                                                owner_env: HashMap::new(),
                                                decl: f,
                                            });
                                            self.structs[id]
                                                .gen_methods
                                                .insert(f.name.text.clone(), gfid);
                                        } else {
                                            let fn_id = self.declare_fn_in(
                                                f,
                                                m,
                                                offset,
                                                Some(id),
                                                HashMap::new(),
                                            );
                                            self.structs[id]
                                                .methods
                                                .insert(f.name.text.clone(), fn_id);
                                        }
                                    }
                                    ast::StmtKind::Let {
                                        pat:
                                            ast::Pattern {
                                                kind: ast::PatternKind::Name(cname),
                                                ..
                                            },
                                        ty,
                                        init: Some(init),
                                        ..
                                    } => {
                                        self.structs[id].consts.insert(
                                            cname.clone(),
                                            StructConst {
                                                mod_id: m,
                                                offset,
                                                ty: ty.as_ref(),
                                                init,
                                            },
                                        );
                                    }
                                    ast::StmtKind::Const {
                                        name,
                                        ty,
                                        init: Some(init),
                                        ..
                                    } => {
                                        self.structs[id].consts.insert(
                                            name.text.clone(),
                                            StructConst {
                                                mod_id: m,
                                                offset,
                                                ty: ty.as_ref(),
                                                init,
                                            },
                                        );
                                    }
                                    ast::StmtKind::Law { name, params, body } => {
                                        self.structs[id].laws.push(LawInfo {
                                            mod_id: m,
                                            offset,
                                            name: name.text.clone(),
                                            params,
                                            body,
                                        });
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    ast::StmtKind::Enum(decl) => {
                        if !decl.generics.is_empty() {
                            let gid = self.gen_enums.len();
                            self.gen_enums.push(GenEnumTemplate {
                                mod_id: m,
                                offset,
                                decl,
                            });
                            self.mod_syms[m]
                                .insert(decl.name.text.clone(), Sym::GenEnum(gid));
                        } else {
                            let id = self.enums.len();
                            self.mod_syms[m].insert(decl.name.text.clone(), Sym::Enum(id));
                            for (tag, variant) in decl.variants.iter().enumerate() {
                                self.mod_syms[m]
                                    .insert(variant.name.text.clone(), Sym::Variant(id, tag));
                            }
                            let ename = if m == 0 {
                                decl.name.text.clone()
                            } else {
                                format!("m{m}_{}", decl.name.text)
                            };
                            self.enums.push(EnumDef {
                                name: ename,
                                variants: Vec::new(),
                            });
                            self.enum_decls.push((m, offset, decl));
                        }
                    }
                    ast::StmtKind::Domain { name, .. } => {
                        let d = self.domains.len();
                        let dname = if m == 0 {
                            name.text.clone()
                        } else {
                            format!("m{m}_{}", name.text)
                        };
                        self.domains.push(dname);
                        self.mod_syms[m].insert(name.text.clone(), Sym::Domain(d));
                    }
                    ast::StmtKind::TypeAlias { name, generics, ty } => {
                        if !generics.is_empty() {
                            let gaid = self.gen_aliases.len();
                            self.gen_aliases.push(GenAliasTemplate {
                                mod_id: m,
                                offset,
                                generics,
                                ty,
                            });
                            self.mod_syms[m]
                                .insert(name.text.clone(), Sym::GenAlias(gaid));
                        } else {
                            let aid = self.aliases.len();
                            self.aliases.push((m, ty, offset));
                            self.mod_syms[m].insert(name.text.clone(), Sym::Alias(aid));
                        }
                    }
                    ast::StmtKind::Fn(decl) => {
                        if stmt.decorators.iter().any(|d| d.name.text == "test")
                            && (m != 0 || active_test != Some(decl.name.text.as_str()))
                        {
                            continue;
                        }
                        if !decl.generics.is_empty() {
                            let gfid = self.gen_fns.len();
                            self.gen_fns.push(GenFnTemplate {
                                mod_id: m,
                                offset,
                                owner: None,
                                owner_env: HashMap::new(),
                                decl,
                            });
                            self.mod_syms[m].insert(decl.name.text.clone(), Sym::GenFn(gfid));
                        } else {
                            let id = self.declare_fn_in(decl, m, offset, None, HashMap::new());
                            self.mod_syms[m].insert(decl.name.text.clone(), Sym::Fn(id));
                        }
                    }
                    ast::StmtKind::Const {
                        name,
                        generics,
                        ty,
                        init: Some(init),
                        ..
                    } => {
                        if !generics.is_empty() {
                            let gcid = self.gen_consts.len();
                            self.gen_consts.push(GenConstTemplate {
                                mod_id: m,
                                offset,
                                generics,
                                ty: ty.as_ref(),
                                init,
                            });
                            self.mod_syms[m]
                                .insert(name.text.clone(), Sym::GenConst(gcid));
                        } else {
                            if let Some(v) = self.eval_const_int(init) {
                                self.const_ints.insert(name.text.clone(), v);
                            }
                            if m > 0 {
                                let mcid = self.mod_consts.len();
                                self.mod_consts.push(ModConstInfo {
                                    mod_id: m,
                                    offset,
                                    name: name.text.clone(),
                                    ty: ty.as_ref(),
                                    init,
                                    global: None,
                                });
                                self.mod_syms[m]
                                    .insert(name.text.clone(), Sym::ModConst(mcid));
                            }
                        }
                    }
                    ast::StmtKind::Trait { name, body, .. } => {
                        let tid = self.traits.len();
                        let mut methods = Vec::new();
                        for s in &body.stmts {
                            if let ast::StmtKind::Fn(f) = &s.kind {
                                methods.push(f);
                            }
                        }
                        self.traits.push(TraitInfo {
                            mod_id: m,
                            offset,
                            name: name.text.clone(),
                            methods,
                        });
                        self.mod_syms[m].insert(name.text.clone(), Sym::Trait(tid));
                    }
                    ast::StmtKind::Impl {
                        trait_name,
                        target,
                        body,
                        ..
                    } => {
                        let tname = trait_name.as_ref().and_then(|t| match &t.kind {
                            ast::TypeKind::Named { name, .. } => Some(name.text.clone()),
                            _ => None,
                        });
                        self.impls.push(ImplInfo {
                            mod_id: m,
                            offset,
                            trait_name: tname,
                            target,
                            body,
                        });
                    }
                    ast::StmtKind::Use { .. } => {}
                    _ => {}
                }
            }
        }

        // Pass 2: wire `use` imports and module aliases across all modules.
        // Run two rounds in case a module re-exports or chains imports.
        for _ in 0..2 {
            for m in 0..num_mods {
                let analysis = self.mods[m];
                let mods_list = analysis.modules();
                for (local_name, imp) in analysis.imports() {
                    if let Some(target_mod) = self.path_to_mod.get(&mods_list[imp.module].path).copied() {
                        if let Some(&sym) = self.mod_syms[target_mod].get(&imp.name) {
                            self.mod_syms[m].entry(local_name.clone()).or_insert(sym);
                        }
                    }
                }
                for (alias, &mod_idx) in analysis.aliases() {
                    if let Some(target_mod) = self.path_to_mod.get(&mods_list[mod_idx].path).copied() {
                        self.mod_syms[m].entry(alias.clone()).or_insert(Sym::Mod(target_mod));
                    }
                }
            }
        }
        self.cur_mod = 0;
    }

    pub(super) fn declare_fn_in(
        &mut self,
        decl: &'a ast::FnDecl,
        mod_id: ModId,
        offset: u32,
        owner: Option<StructId>,
        env: GenEnv,
    ) -> FnId {
        let base = match owner {
            Some(id) => format!("{}.{}", self.structs[id].def.name, decl.name.text),
            None if mod_id == 0 => decl.name.text.clone(),
            None => format!("m{mod_id}_{}", decl.name.text),
        };
        let is_generator = matches!(
            decl.ret.as_ref().map(|t| &t.kind),
            Some(ast::TypeKind::Yields(_))
        );
        self.sigs.push(FnSig {
            name: base,
            params: Vec::new(),
            ret: None,
            owner,
            receiver: None,
            mod_id,
            env,
            decl,
            offset,
            is_generator,
            new_domain_slot: None,
            state: State::Pending,
        });
        self.bodies.push(None);
        self.sigs.len() - 1
    }

    /// Second pass: field, payload and parameter types for non-generic declarations.
    fn signatures(&mut self) {
        let num_structs = self.structs.len();
        for id in 0..num_structs {
            let (decl, m, offset) = (
                self.structs[id].decl,
                self.structs[id].mod_id,
                self.structs[id].offset,
            );
            self.cur_mod = m;
            self.offset = offset;
            self.self_struct = Some(id);
            let fields = decl
                .fields
                .iter()
                .map(|f| (f.name.text.clone(), self.resolve_ty(&f.ty)))
                .collect();
            self.structs[id].def.fields = fields;
            self.self_struct = None;
        }
        // Attach `impl` blocks for non-generic structs.
        for id in 0..num_structs {
            self.attach_impls_for_struct(id);
        }
        for id in 0..self.enum_decls.len() {
            let (m, offset, decl) = self.enum_decls[id];
            self.cur_mod = m;
            self.offset = offset;
            let variants = decl
                .variants
                .iter()
                .map(|v| {
                    let fields = v.fields.iter().map(|(_, t)| self.resolve_ty(t)).collect();
                    (v.name.text.clone(), fields)
                })
                .collect();
            self.enums[id].variants = variants;
        }
        let num_sigs = self.sigs.len();
        for id in 0..num_sigs {
            if self.sigs[id].params.is_empty() && !self.sigs[id].decl.params.is_empty()
                || self.sigs[id].ret.is_none()
            {
                self.resolve_fn_sig(id, None);
            }
        }
        self.cur_mod = 0;
        self.offset = 0;
    }

    pub(super) fn resolve_fn_sig(&mut self, id: FnId, trait_name: Option<&str>) {
        let decl = self.sigs[id].decl;
        let saved_mod = std::mem::replace(&mut self.cur_mod, self.sigs[id].mod_id);
        let saved_offset = std::mem::replace(&mut self.offset, self.sigs[id].offset);
        let saved_env = std::mem::replace(&mut self.gen_env, self.sigs[id].env.clone());
        let saved_self = std::mem::replace(&mut self.self_struct, self.sigs[id].owner);

        let owner_ty = self.sigs[id].owner.map(Ty::Struct);
        let mut params = Vec::new();
        for (i, param) in decl.params.iter().enumerate() {
            if i == 0 && param.name.text == "self" && self.sigs[id].owner.is_some() {
                // A method that grows a cache works on the caller's struct.
                let owner = self.sigs[id].owner.expect("checked above");
                let cached = cache::grows_cache(self.structs[owner].decl, decl);
                self.sigs[id].receiver = Some(param.mutable || cached);
                continue;
            }
            if param.variadic {
                self.unsupported(param.span, "variadic parameters");
            }
            let ty = match &param.ty {
                Some(ty) => self.resolve_ty(ty),
                None => {
                    // Inherit parameter type from operator/standard trait impls (`impl Add for Mod<P>: fn add(a, b)`).
                    match (trait_name, owner_ty.clone()) {
                        (
                            Some("Add" | "Sub" | "Mul" | "Div" | "Neg" | "Show" | "Eq" | "Ord"),
                            Some(ot),
                        ) => ot,
                        (_, Some(ot)) => ot,
                        _ => {
                            self.error(
                                codes::UNKNOWN_TYPE,
                                param.span,
                                format!("parameter `{}` needs a type", param.name.text),
                            );
                            self.help(format!(
                                "write `{}: Int`, or the type it has",
                                param.name.text
                            ));
                            Ty::Error
                        }
                    }
                }
            };
            params.push(SigParam {
                name: param.name.text.clone(),
                ty,
                mutable: param.mutable,
                default: param.default.as_ref(),
            });
        }
        self.sigs[id].params = params;
        self.sigs[id].ret = match (&decl.ret, &decl.body) {
            (
                Some(ast::Type {
                    kind: ast::TypeKind::Yields(inner),
                    ..
                }),
                _,
            ) => {
                self.sigs[id].is_generator = true;
                Some(Ty::list(self.resolve_ty(inner)))
            }
            (Some(ty), _) => {
                // Check for `-> (domain C, comp: [V]C)` returning a fresh domain.
                if let ast::TypeKind::Record(fields) = &ty.kind {
                    if let Some((dname, _)) = fields
                        .iter()
                        .find(|(_, ft)| matches!(ft.kind, ast::TypeKind::NewDomain))
                    {
                        let dom_id = self.domains.len();
                        self.domains.push(format!("{}_{}", self.sigs[id].name, dname.text));
                        self.gen_env
                            .insert(dname.text.clone(), GenArg::Domain(dom_id));
                        self.sigs[id]
                            .env
                            .insert(dname.text.clone(), GenArg::Domain(dom_id));
                        self.sigs[id].new_domain_slot = Some((dname.text.clone(), dom_id));
                    }
                }
                Some(self.resolve_ty(ty))
            }
            (None, ast::FnBody::Block(block)) => match block.stmts.last() {
                // A body that ends by calling its own fn gives what that
                // call gives, which is nothing unless a `return` says so.
                Some(ast::Stmt {
                    kind: ast::StmtKind::Expr(last),
                    ..
                }) if calls_fn(last, &decl.name.text) && !returns_a_value(block) => Some(Ty::Unit),
                Some(ast::Stmt {
                    kind: ast::StmtKind::Expr(_),
                    ..
                }) => None,
                _ => match (trait_name, owner_ty) {
                    (Some("Show"), _) => Some(Ty::Str),
                    (Some("Eq" | "Ord"), _) => Some(Ty::Bool),
                    (Some("Add" | "Sub" | "Mul" | "Div" | "Neg"), Some(ot)) => Some(ot),
                    _ => Some(Ty::Unit),
                },
            },
            (None, _) => match (trait_name, owner_ty) {
                (Some("Show"), _) => Some(Ty::Str),
                (Some("Eq" | "Ord"), _) => Some(Ty::Bool),
                (Some("Add" | "Sub" | "Mul" | "Div" | "Neg"), Some(ot)) => Some(ot),
                _ => None,
            },
        };

        self.cur_mod = saved_mod;
        self.offset = saved_offset;
        self.gen_env = saved_env;
        self.self_struct = saved_self;
    }

    // ----- types -----

    fn resolve_ty(&mut self, ty: &ast::Type) -> Ty {
        self.resolve(ty, false).ty()
    }

    /// The type `ty` names. With `values`, also the lengths and ranges
    /// written in it, lowered where the type stands.
    fn resolve(&mut self, ty: &ast::Type, values: bool) -> Spec {
        match &ty.kind {
            ast::TypeKind::Named { name, args } => self.named(name, args, values),
            ast::TypeKind::Array { index, elem } => {
                let dims: Vec<Option<DimSpec>> =
                    index.iter().map(|d| self.dim(d, values)).collect();
                let elem = self.resolve(elem, values);
                match dims.into_iter().collect::<Option<Vec<_>>>() {
                    Some(dims) => Spec::Array {
                        dims,
                        elem: Box::new(elem),
                    },
                    None => Spec::Plain(Ty::Error),
                }
            }
            ast::TypeKind::Optional(inner) => Spec::Opt(Box::new(self.resolve(inner, values))),
            ast::TypeKind::Tuple(items) if items.is_empty() => Spec::Plain(Ty::Unit),
            ast::TypeKind::Tuple(items) => {
                let specs: Vec<Spec> = items.iter().map(|t| self.resolve(t, values)).collect();
                Spec::Fields(Ty::Tuple(specs.iter().map(Spec::ty).collect()), specs)
            }
            ast::TypeKind::Record(fields) => {
                let specs: Vec<Spec> = fields
                    .iter()
                    .map(|(_, t)| self.resolve(t, values))
                    .collect();
                let ty = Ty::Record(
                    fields
                        .iter()
                        .zip(&specs)
                        .map(|((name, _), spec)| (name.text.clone(), spec.ty()))
                        .collect(),
                );
                Spec::Fields(ty, specs)
            }
            ast::TypeKind::Range { lo, hi, inclusive } => {
                if !values {
                    return Spec::Plain(Ty::Int);
                }
                let lo = self.expect(lo, &Ty::Int);
                let mut hi = self.expect(hi, &Ty::Int);
                if !inclusive {
                    hi = self.int_add(hi, -1);
                }
                Spec::Int {
                    lo: Some(Box::new(lo)),
                    hi: Some(Box::new(hi)),
                    text: self.source(ty.span),
                }
            }
            ast::TypeKind::OwnDomain { name, elem } => {
                let dom_id = self.domains.len();
                self.domains.push(name.text.clone());
                if let Some(scope) = self.cx.local_syms.last_mut() {
                    scope.insert(name.text.clone(), Sym::Domain(dom_id));
                } else {
                    self.mod_syms[self.cur_mod].insert(name.text.clone(), Sym::Domain(dom_id));
                }
                let elem = self.resolve(elem, values);
                Spec::Array {
                    dims: vec![DimSpec::Pos(dom_id)],
                    elem: Box::new(elem),
                }
            }
            ast::TypeKind::Fn { params, ret } => {
                let ps: Vec<Ty> = params.iter().map(|p| self.resolve_ty(p)).collect();
                let r = self.resolve_ty(ret);
                Spec::Plain(Ty::Fn(ps, Box::new(r)))
            }
            ast::TypeKind::Value(_) => Spec::Plain(Ty::Int),
            ast::TypeKind::NewDomain => Spec::Plain(Ty::DomainVal),
            ast::TypeKind::Yields(inner) => Spec::Plain(Ty::list(self.resolve_ty(inner))),
            ast::TypeKind::Error => Spec::Plain(Ty::Error),
        }
    }

    fn named(&mut self, name: &ast::Name, args: &[ast::Type], values: bool) -> Spec {
        let arg = |lw: &mut Self, i: usize| match args.get(i) {
            Some(ty) => lw.resolve_ty(ty),
            None => {
                lw.error(
                    codes::UNKNOWN_TYPE,
                    name.span,
                    format!(
                        "`{}` needs its element type: `{}<Int>`",
                        name.text, name.text
                    ),
                );
                Ty::Error
            }
        };
        // Check `Self` and `gen_env` first.
        if name.text == "Self" {
            if let Some(sid) = self.self_struct {
                return if values {
                    self.struct_spec(sid)
                } else {
                    Spec::Plain(Ty::Struct(sid))
                };
            }
        }
        if args.is_empty() {
            if let Some(ga) = self.gen_env.get(&name.text).cloned() {
                return Spec::Plain(match ga {
                    GenArg::Ty(t) => t,
                    GenArg::Domain(d) => Ty::Pos(d),
                    GenArg::Int(_) | GenArg::RuntimeInt(_) => Ty::Int,
                    GenArg::Op(_) => Ty::Unit,
                });
            }
            if let Some(ty) = self.cx.local_type_aliases.get(&name.text).cloned() {
                return Spec::Plain(ty);
            }
            for ocx in self.outer_cxs.iter().rev() {
                if let Some(ty) = ocx.local_type_aliases.get(&name.text).cloned() {
                    return Spec::Plain(ty);
                }
            }
        }
        // Check qualified module type `mod_alias.Type` or `D.bounds` or `Rooted.Pre`.
        if let Some((prefix, rest)) = name.text.split_once('.') {
            if rest == "bounds" {
                if let Some(GenArg::Domain(d)) = self.gen_env.get(prefix).cloned() {
                    return Spec::Plain(Ty::Bound(d));
                }
                if let Some(Sym::Domain(d)) = self.get_sym(prefix) {
                    return Spec::Plain(Ty::Bound(d));
                }
            }
            if let Some(Sym::Mod(target_mod)) = self.get_sym(prefix) {
                let saved_mod = std::mem::replace(&mut self.cur_mod, target_mod);
                let inner_name = ast::Name {
                    text: rest.to_string(),
                    span: name.span,
                };
                let res = self.named(&inner_name, args, values);
                self.cur_mod = saved_mod;
                return res;
            }
            if let Some(Sym::GenStruct(gid)) = self.get_sym(prefix) {
                let gargs = self.resolve_gen_args(args);
                let sid = self.instantiate_struct(gid, gargs);
                if let Some(&dom_id) = self.structs[sid].owned_domains.get(rest) {
                    return Spec::Plain(Ty::Pos(dom_id));
                }
            }
            if let Some(Sym::Struct(sid)) = self.get_sym(prefix) {
                if let Some(&dom_id) = self.structs[sid].owned_domains.get(rest) {
                    return Spec::Plain(Ty::Pos(dom_id));
                }
            }
        }

        let ty = match name.text.as_str() {
            "Int" | "I64" => Ty::Int,
            "Float" | "F64" => Ty::Float,
            "Bool" => Ty::Bool,
            "Char" => Ty::Char,
            "Str" => Ty::Str,
            "List" | "Iter" => Ty::list(arg(self, 0)),
            "Opt" => Ty::opt(arg(self, 0)),
            "Set" => Ty::Set(Box::new(arg(self, 0))),
            "HashSet" => Ty::HashSet(Box::new(arg(self, 0))),
            "Map" => {
                let key = arg(self, 0);
                Ty::Map(Box::new(key), Box::new(arg(self, 1)))
            }
            "HashMap" => {
                let key = arg(self, 0);
                Ty::HashMap(Box::new(key), Box::new(arg(self, 1)))
            }
            "Range" => Ty::Range(Box::new(arg(self, 0))),
            "Bounds" => match arg(self, 0) {
                Ty::Pos(d) => Ty::Bound(d),
                other => other,
            },
            "I32" | "U8" | "U32" | "U64" | "I128" => Ty::Int,
            _ => match self.get_sym(&name.text) {
                Some(Sym::Struct(id)) if values => return self.struct_spec(id),
                Some(Sym::Struct(id)) => Ty::Struct(id),
                Some(Sym::GenStruct(gid)) => {
                    let gargs = self.resolve_gen_args(args);
                    let sid = self.instantiate_struct(gid, gargs);
                    if values {
                        return self.struct_spec(sid);
                    }
                    Ty::Struct(sid)
                }
                Some(Sym::Enum(id)) => Ty::Enum(id),
                Some(Sym::GenEnum(gid)) => {
                    let gargs = self.resolve_gen_args(args);
                    Ty::Enum(self.instantiate_enum(gid, gargs))
                }
                Some(Sym::Domain(id)) => Ty::Pos(id),
                Some(Sym::Alias(id)) => {
                    let (m, ty, offset) = self.aliases[id];
                    let saved_mod = std::mem::replace(&mut self.cur_mod, m);
                    let saved_offset = std::mem::replace(&mut self.offset, offset);
                    let spec = self.resolve(ty, values);
                    self.cur_mod = saved_mod;
                    self.offset = saved_offset;
                    return spec;
                }
                Some(Sym::GenAlias(gaid)) => {
                    let gargs = self.resolve_gen_args(args);
                    let tpl_mod = self.gen_aliases[gaid].mod_id;
                    let tpl_offset = self.gen_aliases[gaid].offset;
                    let generics = self.gen_aliases[gaid].generics;
                    let target_ty = self.gen_aliases[gaid].ty;
                    let mut env = HashMap::new();
                    for (p, a) in generics.iter().zip(gargs) {
                        env.insert(p.text.clone(), a);
                    }
                    let saved_mod = std::mem::replace(&mut self.cur_mod, tpl_mod);
                    let saved_offset = std::mem::replace(&mut self.offset, tpl_offset);
                    let saved_env = std::mem::replace(&mut self.gen_env, env);
                    let spec = self.resolve(target_ty, values);
                    self.cur_mod = saved_mod;
                    self.offset = saved_offset;
                    self.gen_env = saved_env;
                    return spec;
                }
                _ => {
                    self.error(
                        codes::UNKNOWN_TYPE,
                        name.span,
                        format!("`{}` is not a type `cpl run` knows", name.text),
                    );
                    Ty::Error
                }
            },
        };
        Spec::Plain(ty)
    }

    /// A struct's fields as they are read from input: with their ranges.
    fn struct_spec(&mut self, id: StructId) -> Spec {
        let (decl, m, offset, env) = (
            self.structs[id].decl,
            self.structs[id].mod_id,
            self.structs[id].offset,
            self.structs[id].env.clone(),
        );
        let saved_mod = std::mem::replace(&mut self.cur_mod, m);
        let saved_offset = std::mem::replace(&mut self.offset, offset);
        let saved_env = std::mem::replace(&mut self.gen_env, env);
        let fields = decl
            .fields
            .iter()
            .map(|f| self.resolve(&f.ty, true))
            .collect();
        self.cur_mod = saved_mod;
        self.offset = saved_offset;
        self.gen_env = saved_env;
        Spec::Fields(Ty::Struct(id), fields)
    }

    /// One dimension of an array type: a domain, its bounds, a custom `Index` struct, or a count.
    fn dim(&mut self, ty: &ast::Type, values: bool) -> Option<DimSpec> {
        if let ast::TypeKind::Named { name, args } = &ty.kind {
            if args.is_empty() {
                if let Some(GenArg::Domain(id)) = self.gen_env.get(&name.text).cloned() {
                    return Some(DimSpec::Pos(id));
                }
                if let Some(Sym::Domain(id)) = self.get_sym(&name.text) {
                    return Some(DimSpec::Pos(id));
                }
                if let Some(Sym::Struct(sid)) = self.get_sym(&name.text) {
                    if let Some(dom_id) = self.structs[sid].index_domain {
                        return Some(DimSpec::Pos(dom_id));
                    }
                }
                if let Some(domain) = name.text.strip_suffix(".bounds") {
                    if let Some(GenArg::Domain(id)) = self.gen_env.get(domain).cloned() {
                        return Some(DimSpec::Bound(id));
                    }
                    if let Some(Sym::Domain(id)) = self.get_sym(domain) {
                        return Some(DimSpec::Bound(id));
                    }
                }
            } else if name.text == "Bounds" && args.len() == 1 {
                if let Ty::Pos(id) = self.resolve_ty(&args[0]) {
                    return Some(DimSpec::Bound(id));
                }
            } else if let Some(Sym::GenStruct(gid)) = self.get_sym(&name.text) {
                let gargs = self.resolve_gen_args(args);
                let sid = self.instantiate_struct(gid, gargs);
                if let Some(dom_id) = self.structs[sid].index_domain {
                    return Some(DimSpec::Pos(dom_id));
                }
            } else if let Some((prefix, rest)) = name.text.split_once('.') {
                if let Some(Sym::GenStruct(gid)) = self.get_sym(prefix) {
                    let gargs = self.resolve_gen_args(args);
                    let sid = self.instantiate_struct(gid, gargs);
                    if let Some(&dom_id) = self.structs[sid].owned_domains.get(rest) {
                        return Some(DimSpec::Pos(dom_id));
                    }
                }
                if let Some(Sym::Struct(sid)) = self.get_sym(prefix) {
                    if let Some(&dom_id) = self.structs[sid].owned_domains.get(rest) {
                        return Some(DimSpec::Pos(dom_id));
                    }
                }
                if let Some(slot) = self.lookup(prefix) {
                    if let Ty::Struct(sid) = self.cx.locals[slot].ty {
                        match self.structs[sid].env.get(rest).cloned() {
                            Some(GenArg::Domain(dom_id)) | Some(GenArg::Ty(Ty::Pos(dom_id))) => {
                                return Some(DimSpec::Pos(dom_id));
                            }
                            _ => {}
                        }
                        if let Some(&dom_id) = self.structs[sid].owned_domains.get(rest) {
                            return Some(DimSpec::Pos(dom_id));
                        }
                    }
                }
            }
        }
        // Also handle `[xs.D]T` parsed as `TypeKind::Value(Field { base: Name(prefix), name: rest })`.
        if let ast::TypeKind::Value(e) = &ty.kind {
            if let ast::ExprKind::Field { base, name } = &e.kind {
                if let ast::ExprKind::Name(prefix) = &base.kind {
                    if let Some(slot) = self.lookup(prefix) {
                        if let Ty::Struct(sid) = self.cx.locals[slot].ty {
                            match self.structs[sid].env.get(&name.text).cloned() {
                                Some(GenArg::Domain(dom_id)) | Some(GenArg::Ty(Ty::Pos(dom_id))) => {
                                    return Some(DimSpec::Pos(dom_id));
                                }
                                _ => {}
                            }
                            if let Some(&dom_id) = self.structs[sid].owned_domains.get(&name.text) {
                                return Some(DimSpec::Pos(dom_id));
                            }
                        }
                    }
                    if let Some(Sym::Struct(sid)) = self.get_sym(prefix) {
                        if let Some(&dom_id) = self.structs[sid].owned_domains.get(&name.text) {
                            return Some(DimSpec::Pos(dom_id));
                        }
                    }
                }
            }
        }
        // Anything else is a length: `[n]Int`, `[2 * n]Int`, `[0..n]Int`, `[V.bounds]Int`.
        let text = self.source(ty.span);
        if let Some(domain) = text.strip_suffix(".bounds") {
            let dname = domain.trim();
            if let Some(GenArg::Domain(id)) = self.gen_env.get(dname).cloned() {
                return Some(DimSpec::Bound(id));
            }
            if let Some(Sym::Domain(id)) = self.get_sym(dname) {
                return Some(DimSpec::Bound(id));
            }
        }
        match &ty.kind {
            ast::TypeKind::Range { lo, hi, inclusive } => {
                if !matches!(lo.kind, ast::ExprKind::Int(0)) {
                    self.unsupported(lo.span, "array dimensions that don't start at 0");
                    return None;
                }
                if !*inclusive {
                    let saved_diags = self.diags.len();
                    let saved_cx = self.cx.clone();
                    let saved_outer = self.outer_cxs.clone();
                    let probed = self.expr(hi, None);
                    self.diags.truncate(saved_diags);
                    self.cx = saved_cx;
                    self.outer_cxs = saved_outer;
                    if let ExprKind::DomainVal(dom_id) = probed.kind {
                        return Some(DimSpec::Pos(dom_id));
                    }
                }
                if !values {
                    return Some(DimSpec::Int(Box::new(self.int_lit(0, ty.span))));
                }
                let mut len = self.expect(hi, &Ty::Int);
                if *inclusive {
                    len = self.int_add(len, 1);
                }
                Some(DimSpec::Int(Box::new(len)))
            }
            ast::TypeKind::Named { name, args } if args.is_empty() => {
                if !values {
                    return Some(DimSpec::Int(Box::new(self.int_lit(0, ty.span))));
                }
                let expr = ast::Expr {
                    kind: ast::ExprKind::Name(name.text.clone()),
                    span: name.span,
                };
                Some(DimSpec::Int(Box::new(self.expect(&expr, &Ty::Int))))
            }
            ast::TypeKind::Value(expr) => {
                if !values {
                    let saved_diags = self.diags.len();
                    let saved_cx = self.cx.clone();
                    let saved_outer = self.outer_cxs.clone();
                    let v = self.expr(expr, None);
                    self.diags.truncate(saved_diags);
                    self.cx = saved_cx;
                    self.outer_cxs = saved_outer;
                    if let ExprKind::DomainVal(dom_id) = v.kind {
                        return Some(DimSpec::Pos(dom_id));
                    }
                    return Some(DimSpec::Int(Box::new(self.int_lit(0, ty.span))));
                }
                let v = self.expr(expr, None);
                if let ExprKind::DomainVal(dom_id) = v.kind {
                    return Some(DimSpec::Pos(dom_id));
                }
                Some(DimSpec::Int(Box::new(self.coerce(v, &Ty::Int, expr.span))))
            }
            _ => {
                self.error(
                    codes::UNKNOWN_TYPE,
                    ty.span,
                    "an array is indexed by a domain or a length: `[V]Int`, `[n]Int`",
                );
                None
            }
        }
    }

    /// The source text of a span of the current item.
    fn source(&self, span: Span) -> String {
        let abs = span.shift(self.offset);
        self.mods[self.cur_mod]
            .text
            .get(abs.start as usize..abs.end as usize)
            .unwrap_or("")
            .to_string()
    }

    fn line(&self, span: Span) -> u32 {
        let abs = span.shift(self.offset);
        self.mods[self.cur_mod].lines.line_of(abs.start) + 1
    }

    // ----- scopes -----

    fn new_slot(&mut self, name: &str, ty: Ty) -> Slot {
        self.cx.locals.push(Local {
            name: name.to_string(),
            ty,
        });
        self.cx.locals.len() - 1
    }

    /// A local the user didn't name.
    fn temp(&mut self, ty: Ty) -> Slot {
        self.new_slot("", ty)
    }

    fn local(&self, slot: Slot, span: Span) -> Expr {
        Expr {
            kind: ExprKind::Local(slot),
            ty: self.cx.locals[slot].ty.clone(),
            span: self.span(span),
        }
    }

    fn lookup(&self, name: &str) -> Option<Slot> {
        self.cx
            .scopes
            .iter()
            .rev()
            .find_map(|s| s.get(name).copied())
    }

    /// Declares `name` with the value `init`: a global at the top level of
    /// the file, a local anywhere else.
    fn declare(&mut self, name: &str, init: Expr, mutable: bool, out: &mut Vec<Stmt>) {
        if self.cx.is_main && self.cx.scopes.len() == 1 {
            let id = self.globals.len();
            self.globals.push(Global {
                name: name.to_string(),
                ty: init.ty.clone(),
            });
            self.mod_syms[self.cur_mod].insert(name.to_string(), Sym::Global(id));
            let place = Place {
                kind: PlaceKind::Global(id),
                ty: init.ty.clone(),
                span: init.span,
            };
            out.push(Stmt::Assign(place, AssignOp::Set, init));
        } else {
            let slot = self.new_slot(name, init.ty.clone());
            if mutable {
                self.cx.mut_slots.insert(slot);
            }
            self.cx
                .scopes
                .last_mut()
                .expect("a scope")
                .insert(name.to_string(), slot);
            out.push(Stmt::Let(slot, Some(init)));
        }
    }

    // ----- fns -----

    fn main(&mut self) {
        self.cur_mod = 0;
        self.cx = FnCx {
            is_main: true,
            ret: Some(Ty::Unit),
            scopes: vec![HashMap::new()],
            local_syms: vec![HashMap::new()],
            ..FnCx::default()
        };
        let mut out = Vec::new();
        let items: Vec<(u32, &'a ast::Stmt)> = self.mod_items(0).collect();
        for (offset, stmt) in items {
            self.offset = offset;
            // Skip `@test` functions in normal `cpl run`/`cpl build`.
            if stmt.decorators.iter().any(|d| d.name.text == "test") {
                continue;
            }
            self.stmt(stmt, &mut out);
        }
        let cx = std::mem::take(&mut self.cx);
        self.main_body = Block { stmts: out };
        self.main_locals = cx.locals;
    }

    fn main_for_test(&mut self, test_name: &str) {
        self.cur_mod = 0;
        self.cx = FnCx {
            is_main: true,
            ret: Some(Ty::Unit),
            scopes: vec![HashMap::new()],
            local_syms: vec![HashMap::new()],
            ..FnCx::default()
        };
        let mut out = Vec::new();
        let items: Vec<(u32, &'a ast::Stmt)> = self.mod_items(0).collect();
        for (offset, stmt) in items {
            self.offset = offset;
            // Run top-level constants/lets/domains/impl Index initializers before the test.
            match &stmt.kind {
                ast::StmtKind::Const { generics, .. } if generics.is_empty() => {
                    self.stmt(stmt, &mut out);
                }
                ast::StmtKind::Let { .. }
                | ast::StmtKind::Domain { .. }
                | ast::StmtKind::Impl { .. } => {
                    self.stmt(stmt, &mut out);
                }
                _ => {}
            }
        }
        if let Some(Sym::Fn(fn_id)) = self.mod_syms[0].get(test_name).copied() {
            self.queue_fn(fn_id);
            let ret = self.ret_of(fn_id, Span::default());
            let call = Expr {
                kind: ExprKind::Call(fn_id, Vec::new()),
                ty: ret,
                span: Span::default(),
            };
            out.push(Stmt::Expr(call));
        } else {
            self.error(
                codes::UNRESOLVED_NAME,
                Span::default(),
                format!("no `@test` fn named `{test_name}`"),
            );
        }
        let cx = std::mem::take(&mut self.cx);
        self.main_body = Block { stmts: out };
        self.main_locals = cx.locals;
    }

    pub(super) fn queue_fn(&mut self, id: FnId) {
        if self.queued_set.insert(id) {
            self.queued_fns.push(id);
        }
    }

    /// Lowers the body of fn `id`, once.
    fn function(&mut self, id: FnId) {
        if self.sigs[id].state != State::Pending {
            return;
        }
        // Skip uncalled `@test` fns in non-test runs unless explicitly referenced.
        self.sigs[id].state = State::InProgress;
        let decl = self.sigs[id].decl;
        let is_gen = self.sigs[id].is_generator;
        let new_dom = self.sigs[id].new_domain_slot.clone();
        let saved_mod = std::mem::replace(&mut self.cur_mod, self.sigs[id].mod_id);
        let saved_offset = std::mem::replace(&mut self.offset, self.sigs[id].offset);
        let saved_env = std::mem::replace(&mut self.gen_env, self.sigs[id].env.clone());
        let saved_self = std::mem::replace(&mut self.self_struct, self.sigs[id].owner);

        let mut cx = FnCx {
            ret: self.sigs[id].ret.clone(),
            scopes: vec![HashMap::new()],
            local_syms: vec![HashMap::new()],
            ..FnCx::default()
        };
        let mut params = Vec::new();
        if let (Some(mutable), Some(owner)) = (self.sigs[id].receiver, self.sigs[id].owner) {
            cx.locals.push(Local {
                name: "self".into(),
                ty: Ty::Struct(owner),
            });
            cx.scopes[0].insert("self".into(), 0);
            if mutable {
                cx.mut_slots.insert(0);
            }
            cx.receiver = Some((0, owner));
            params.push(ParamDef { mutable });
        }
        for param in &self.sigs[id].params {
            let slot = cx.locals.len();
            cx.scopes[0].insert(param.name.clone(), slot);
            if param.mutable {
                cx.mut_slots.insert(slot);
            }
            cx.locals.push(Local {
                name: param.name.clone(),
                ty: param.ty.clone(),
            });
            params.push(ParamDef {
                mutable: param.mutable,
            });
        }
        for ast_p in &decl.params {
            if let Some(ast::Type {
                kind: ast::TypeKind::Named { name, args },
                ..
            }) = &ast_p.ty
            {
                if args.is_empty() {
                    if let Some(GenArg::Op(op)) = self.gen_env.get(&name.text) {
                        cx.op_locals.insert(ast_p.name.text.clone(), *op);
                    }
                }
            }
        }
        let saved_cx = std::mem::replace(&mut self.cx, cx);
        let mut out = Vec::new();

        // If this is a constructor/static method of a struct with owned domains (e.g. `Rooted<V>.Pre`),
        // initialize those owned domains at the start of the method.
        if let Some(owner) = self.sigs[id].owner {
            if self.sigs[id].receiver.is_none() {
                let inits = self.structs[owner].owned_domain_inits.clone();
                for init in inits {
                    let sm = std::mem::replace(&mut self.cur_mod, init.mod_id);
                    let so = std::mem::replace(&mut self.offset, init.offset);
                    self.init_domain_id(init.id, init.def, &mut out);
                    self.cur_mod = sm;
                    self.offset = so;
                }
            }
        }

        // If this fn is a generator (`-> yields T`), allocate the output `List<T>` accumulator.
        if is_gen {
            let list_ty = self.cx.ret.clone().unwrap_or_else(|| Ty::list(Ty::Error));
            let elem_ty = match &list_ty {
                Ty::List(e) => (**e).clone(),
                _ => Ty::Error,
            };
            let acc = self.new_slot("__yields", list_ty.clone());
            self.cx.mut_slots.insert(acc);
            out.push(Stmt::Let(
                acc,
                Some(self.mk(ExprKind::Tuple(Vec::new()), list_ty, decl.name.span)),
            ));
            self.cx.yield_slot = Some((acc, elem_ty));
        }

        // If this fn returns `(domain C, ...)`, pre-bind `C` as a local domain symbol.
        if let Some((ref dname, dom_id)) = new_dom {
            self.cx.local_syms[0].insert(dname.clone(), Sym::Domain(dom_id));
        }

        match &decl.body {
            ast::FnBody::Block(block) => {
                let tail = if is_gen { Tail::Discard } else { Tail::Return };
                let body = self.block(block, tail);
                out.extend(body.stmts);
            }
            ast::FnBody::Expr(expr) => {
                let tail = if is_gen { Tail::Discard } else { Tail::Return };
                self.tail(expr, tail, &mut out);
            }
            ast::FnBody::None => {
                let name = &decl.name.text;
                self.error(
                    codes::UNSUPPORTED,
                    decl.name.span,
                    format!("`{name}` has no body: the judge's grader supplies it, so there is nothing to run here"),
                );
                self.help(format!(
                    "give `{name}` a body to try the program locally, and take it out to submit"
                ));
            }
        }

        if let Some((acc, _)) = self.cx.yield_slot {
            let res = self.local(acc, decl.name.span);
            out.push(Stmt::Return(Some(res)));
        }

        let ret = self.cx.ret.clone().unwrap_or(Ty::Unit);
        if ret != Ty::Unit && !matches!(out.last(), Some(Stmt::Return(_))) {
            out.push(Stmt::Trap(
                format!("fn `{}` ended without a value", decl.name.text),
                self.span(decl.name.span),
            ));
        }
        let cx = std::mem::replace(&mut self.cx, saved_cx);
        self.cur_mod = saved_mod;
        self.offset = saved_offset;
        self.gen_env = saved_env;
        self.self_struct = saved_self;
        self.sigs[id].ret = Some(ret.clone());
        self.sigs[id].state = State::Done;
        self.bodies[id] = Some(Function {
            name: self.sigs[id].name.clone(),
            params,
            ret,
            locals: cx.locals,
            body: Block { stmts: out },
        });
    }

    /// The return type of fn `id`, reading its body now if it has no `->`.
    fn ret_of(&mut self, id: FnId, span: Span) -> Ty {
        if let Some(ty) = &self.sigs[id].ret {
            return ty.clone();
        }
        if self.sigs[id].state == State::InProgress {
            let name = self.sigs[id].decl.name.text.clone();
            self.error(
                codes::UNKNOWN_TYPE,
                span,
                format!("`{name}` calls itself, so it needs its return type written"),
            );
            self.help(format!(
                "add `-> Int`, or the type it returns, to `fn {name}`"
            ));
            return Ty::Error;
        }
        self.function(id);
        self.sigs[id].ret.clone().unwrap_or(Ty::Error)
    }

    // ----- small builders -----

    fn int_lit(&self, value: i64, span: Span) -> Expr {
        Expr {
            kind: ExprKind::Int(value),
            ty: Ty::Int,
            span: self.span(span),
        }
    }

    /// `e + by`, folded when `e` is a literal.
    fn int_add(&self, e: Expr, by: i64) -> Expr {
        if let ExprKind::Int(v) = e.kind {
            if let Some(sum) = v.checked_add(by) {
                return Expr {
                    kind: ExprKind::Int(sum),
                    ..e
                };
            }
        }
        let span = e.span;
        let rhs = Expr {
            kind: ExprKind::Int(by),
            ty: Ty::Int,
            span,
        };
        Expr {
            kind: ExprKind::Binary(BinOp::Add, Box::new(e), Box::new(rhs)),
            ty: Ty::Int,
            span,
        }
    }

    fn mk(&self, kind: ExprKind, ty: Ty, span: Span) -> Expr {
        Expr {
            kind,
            ty,
            span: self.span(span),
        }
    }

    /// `e` as a `want`, converting where the language does it silently.
    fn coerce(&mut self, e: Expr, want: &Ty, span: Span) -> Expr {
        match self.try_coerce(e, want) {
            Ok(e) => e,
            Err(e) => {
                self.mismatch(span, want, &e.ty);
                self.err_expr(span)
            }
        }
    }

    fn try_coerce(&mut self, e: Expr, want: &Ty) -> Result<Expr, Expr> {
        if e.ty == *want || e.ty == Ty::Error || *want == Ty::Error {
            return Ok(e);
        }
        let span = e.span;
        match (want, &e.ty) {
            (Ty::Pos(d), Ty::DomainVal) => Ok(Expr {
                kind: ExprKind::Cast(
                    Cast::Trust,
                    Box::new(Expr {
                        kind: ExprKind::Int(0),
                        ty: Ty::Int,
                        span,
                    }),
                ),
                ty: Ty::Pos(*d),
                span,
            }),
            (Ty::Float, Ty::Int) => Ok(match e.kind {
                ExprKind::Int(v) => Expr {
                    kind: ExprKind::Float(v as f64),
                    ty: Ty::Float,
                    span,
                },
                _ => Expr {
                    kind: ExprKind::Cast(Cast::IntToFloat, Box::new(e)),
                    ty: Ty::Float,
                    span,
                },
            }),
            (Ty::Opt(inner), _) if !matches!(e.kind, ExprKind::None) => {
                let inner = self.try_coerce(e, inner)?;
                Ok(Expr {
                    kind: ExprKind::Some(Box::new(inner)),
                    ty: want.clone(),
                    span,
                })
            }
            (Ty::List(elem), Ty::Str) if **elem == Ty::Char => Ok(Expr {
                kind: ExprKind::Builtin(Builtin::StrToList, vec![e]),
                ty: want.clone(),
                span,
            }),
            (Ty::Str, Ty::List(elem)) if **elem == Ty::Char => Ok(Expr {
                kind: ExprKind::Builtin(Builtin::ListToStr, vec![e]),
                ty: Ty::Str,
                span,
            }),
            (Ty::List(elem), Ty::Array { elem: aelem, .. }) if *elem == *aelem => Ok(Expr {
                kind: ExprKind::Builtin(Builtin::ArrayToList, vec![e]),
                ty: want.clone(),
                span,
            }),
            (Ty::HashMap(..), Ty::Map(..)) if matches!(&e.kind, ExprKind::Tuple(v) if v.is_empty()) => {
                Ok(Expr {
                    ty: want.clone(),
                    ..e
                })
            }
            (Ty::HashSet(_), Ty::Set(_)) if matches!(&e.kind, ExprKind::Tuple(v) if v.is_empty()) => {
                Ok(Expr {
                    ty: want.clone(),
                    ..e
                })
            }
            (Ty::List(w_elem), Ty::List(e_elem)) if w_elem != e_elem => {
                // Element-wise list coercion (e.g. `List<Int>` -> `List<?Int>`, `List<(Char, Int)>` -> `List<(val: Char, len: Int)>`).
                let can_elem = matches!(
                    (&**w_elem, &**e_elem),
                    (Ty::Opt(inner), other) if **inner == *other
                ) || matches!(
                    (&**w_elem, &**e_elem),
                    (Ty::Record(wf), Ty::Tuple(et)) if wf.len() == et.len() && wf.iter().zip(et).all(|((_, a), b)| a == b)
                ) || matches!(
                    (&**w_elem, &**e_elem),
                    (Ty::Tuple(wt), Ty::Record(ef)) if wt.len() == ef.len() && wt.iter().zip(ef).all(|(a, (_, b))| a == b)
                );
                if !can_elem {
                    return Err(e);
                }
                let src_ty = e.ty.clone();
                let dst_ty = want.clone();
                let w_elem = (**w_elem).clone();
                let e_elem = (**e_elem).clone();
                Ok(self.do_expr(Some(&dst_ty), span, |lw, out| {
                    let src_slot = lw.temp(src_ty);
                    out.push(Stmt::Let(src_slot, Some(e)));
                    let acc_slot = lw.temp(dst_ty.clone());
                    out.push(Stmt::Let(
                        acc_slot,
                        Some(lw.mk(ExprKind::Tuple(Vec::new()), dst_ty.clone(), span)),
                    ));
                    let item_slot = lw.temp(e_elem.clone());
                    let item_expr = lw.local(item_slot, span);
                    let coerced = lw.try_coerce(item_expr, &w_elem).unwrap_or_else(|x| x);
                    let push = lw.mk(
                        ExprKind::Mutate(
                            Mutation::Push,
                            Box::new(Place {
                                kind: PlaceKind::Local(acc_slot),
                                ty: dst_ty.clone(),
                                span,
                            }),
                            vec![coerced],
                        ),
                        Ty::Unit,
                        span,
                    );
                    let lid = lw.next_loop;
                    lw.next_loop += 1;
                    let src_ref = lw.local(src_slot, span);
                    out.push(Stmt::For(
                        lid,
                        item_slot,
                        Iter::Seq(src_ref, false),
                        Block {
                            stmts: vec![Stmt::Expr(push)],
                        },
                    ));
                    out.push(Stmt::Give(lw.local(acc_slot, span)));
                }))
            }
            (
                Ty::Array {
                    dims: w_dims,
                    elem: w_elem,
                },
                Ty::Array {
                    dims: e_dims,
                    elem: e_elem,
                },
            ) if w_dims == e_dims && w_dims.len() == 1 => {
                // Structural record projection for 1D arrays, e.g. `[E](u: V, v: V, w: Int)` -> `[E](u: V, v: V)`.
                if let (Ty::Record(w_fields), Ty::Record(e_fields)) = (&**w_elem, &**e_elem) {
                    let indices: Option<Vec<usize>> = w_fields
                        .iter()
                        .map(|(wn, wt)| {
                            e_fields
                                .iter()
                                .position(|(en, et)| en == wn && et == wt)
                        })
                        .collect();
                    if let Some(indices) = indices {
                        let dim = w_dims[0];
                        let src_ty = e.ty.clone();
                        let dst_ty = want.clone();
                        let w_rec = (**w_elem).clone();
                        let e_rec = (**e_elem).clone();
                        return Ok(self.do_expr(Some(&dst_ty), span, |lw, out| {
                            let src_slot = lw.temp(src_ty);
                            out.push(Stmt::Let(src_slot, Some(e)));
                            let dim_spec = match dim {
                                Dim::Pos(d) => DimSpec::Pos(d),
                                Dim::Bound(d) => DimSpec::Bound(d),
                                Dim::Int => {
                                    let sref = lw.local(src_slot, span);
                                    DimSpec::Int(Box::new(lw.len_of(sref, span)))
                                }
                            };
                            let arr_slot = lw.temp(dst_ty.clone());
                            out.push(Stmt::Let(
                                arr_slot,
                                Some(lw.mk(ExprKind::Fill(vec![dim_spec], None), dst_ty.clone(), span)),
                            ));
                            let idx_ty = match dim {
                                Dim::Pos(d) => Ty::Pos(d),
                                Dim::Bound(d) => Ty::Bound(d),
                                Dim::Int => Ty::Int,
                            };
                            let idx_slot = lw.temp(idx_ty.clone());
                            let idx_expr = lw.local(idx_slot, span);
                            let sref = lw.local(src_slot, span);
                            let elem_expr = lw.mk(
                                ExprKind::Index(Box::new(sref.clone()), vec![idx_expr.clone()]),
                                e_rec.clone(),
                                span,
                            );
                            let elem_slot = lw.temp(e_rec);
                            let el_ref = lw.local(elem_slot, span);
                            let proj_fields: Vec<Expr> = indices
                                .iter()
                                .zip(w_fields)
                                .map(|(&fi, (_, ft))| {
                                    lw.mk(ExprKind::Field(Box::new(el_ref.clone()), fi), ft.clone(), span)
                                })
                                .collect();
                            let proj_val = lw.mk(ExprKind::Tuple(proj_fields), w_rec.clone(), span);
                            let set_stmt = Stmt::Assign(
                                Place {
                                    kind: PlaceKind::Index(
                                        Box::new(Place {
                                            kind: PlaceKind::Local(arr_slot),
                                            ty: dst_ty.clone(),
                                            span,
                                        }),
                                        vec![idx_expr],
                                    ),
                                    ty: w_rec,
                                    span,
                                },
                                AssignOp::Set,
                                proj_val,
                            );
                            let iter = match dim {
                                Dim::Pos(d) => Iter::Range {
                                    lo: lw.int_lit(0, span),
                                    hi: lw.mk(ExprKind::DomainLen(d), Ty::Int, span),
                                    step: None,
                                    rev: false,
                                },
                                Dim::Bound(d) => Iter::Range {
                                    lo: lw.int_lit(0, span),
                                    hi: lw.int_add(lw.mk(ExprKind::DomainLen(d), Ty::Int, span), 1),
                                    step: None,
                                    rev: false,
                                },
                                Dim::Int => Iter::Range {
                                    lo: lw.int_lit(0, span),
                                    hi: lw.len_of(sref, span),
                                    step: None,
                                    rev: false,
                                },
                            };
                            let lid = lw.next_loop;
                            lw.next_loop += 1;
                            out.push(Stmt::For(
                                lid,
                                idx_slot,
                                iter,
                                Block {
                                    stmts: vec![Stmt::Let(elem_slot, Some(elem_expr)), set_stmt],
                                },
                            ));
                            out.push(Stmt::Give(lw.local(arr_slot, span)));
                        }));
                    }
                }
                Err(e)
            }
            (Ty::Record(fields), Ty::Tuple(items))
                if fields.len() == items.len()
                    && fields.iter().zip(items).all(|((_, f), i)| f == i) =>
            {
                Ok(Expr {
                    ty: want.clone(),
                    ..e
                })
            }
            (Ty::Tuple(items), Ty::Record(fields))
                if fields.len() == items.len()
                    && fields.iter().zip(items).all(|((_, f), i)| f == i) =>
            {
                Ok(Expr {
                    ty: want.clone(),
                    ..e
                })
            }
            (Ty::Record(f1), Ty::Record(f2))
                if f1.len() == f2.len()
                    && f1.iter().zip(f2).all(|((_, a), (_, b))| a == b) =>
            {
                Ok(Expr {
                    ty: want.clone(),
                    ..e
                })
            }
            // Structural struct subtyping (e.g. `Group<T>` -> `Monoid<T>` -> `Semigroup<T>`).
            (Ty::Struct(want_id), Ty::Struct(have_id)) => {
                if let Some(projected) = self.coerce_struct_projection(e.clone(), *have_id, *want_id) {
                    Ok(projected)
                } else {
                    Err(e)
                }
            }
            _ => Err(e),
        }
    }
}

fn collect_block_calls(block: &Block, out: &mut Vec<FnId>) {
    for stmt in &block.stmts {
        collect_stmt_calls(stmt, out);
    }
}

fn collect_stmt_calls(stmt: &Stmt, out: &mut Vec<FnId>) {
    match stmt {
        Stmt::Let(_, init) => {
            if let Some(e) = init {
                collect_expr_calls(e, out);
            }
        }
        Stmt::Assign(place, _, value) => {
            collect_place_calls(place, out);
            collect_expr_calls(value, out);
        }
        Stmt::Expr(e) | Stmt::Give(e) => collect_expr_calls(e, out),
        Stmt::Return(value) => {
            if let Some(e) = value {
                collect_expr_calls(e, out);
            }
        }
        Stmt::If(cond, then, otherwise) => {
            collect_expr_calls(cond, out);
            collect_block_calls(then, out);
            collect_block_calls(otherwise, out);
        }
        Stmt::While(_, cond, body) => {
            collect_expr_calls(cond, out);
            collect_block_calls(body, out);
        }
        Stmt::For(_, _, iter, body) => {
            match iter {
                Iter::Range { lo, hi, step, .. } => {
                    collect_expr_calls(lo, out);
                    collect_expr_calls(hi, out);
                    if let Some(s) = step {
                        collect_expr_calls(s, out);
                    }
                }
                Iter::RangeVal(e, _) | Iter::Seq(e, _) => collect_expr_calls(e, out),
            }
            collect_block_calls(body, out);
        }
        Stmt::Assert { cond, message, .. } => {
            collect_expr_calls(cond, out);
            if let Some(m) = message {
                collect_expr_calls(m, out);
            }
        }
        Stmt::DefDomain { lo, len, .. } => {
            collect_expr_calls(lo, out);
            collect_expr_calls(len, out);
        }
        Stmt::DefDomainFromVal { val, .. } => collect_expr_calls(val, out),
        Stmt::Break(_) | Stmt::Continue(_) | Stmt::Trap(..) => {}
    }
}

fn collect_place_calls(place: &Place, out: &mut Vec<FnId>) {
    match &place.kind {
        PlaceKind::Local(_) | PlaceKind::Global(_) => {}
        PlaceKind::Field(base, _) => collect_place_calls(base, out),
        PlaceKind::Index(base, indices) => {
            collect_place_calls(base, out);
            for idx in indices {
                collect_expr_calls(idx, out);
            }
        }
    }
}

fn collect_spec_calls(spec: &Spec, out: &mut Vec<FnId>) {
    match spec {
        Spec::Plain(_) => {}
        Spec::Int { lo, hi, .. } => {
            if let Some(e) = lo {
                collect_expr_calls(e, out);
            }
            if let Some(e) = hi {
                collect_expr_calls(e, out);
            }
        }
        Spec::Fields(_, fields) => {
            for f in fields {
                collect_spec_calls(f, out);
            }
        }
        Spec::Array { dims, elem } => {
            for d in dims {
                if let DimSpec::Int(len) = d {
                    collect_expr_calls(len, out);
                }
            }
            collect_spec_calls(elem, out);
        }
        Spec::Opt(inner) => collect_spec_calls(inner, out),
    }
}

fn collect_expr_calls(e: &Expr, out: &mut Vec<FnId>) {
    match &e.kind {
        ExprKind::Unit
        | ExprKind::Int(_)
        | ExprKind::Float(_)
        | ExprKind::Bool(_)
        | ExprKind::Char(_)
        | ExprKind::Str(_)
        | ExprKind::None
        | ExprKind::Local(_)
        | ExprKind::Global(_)
        | ExprKind::DomainLen(_)
        | ExprKind::DomainLo(_)
        | ExprKind::DomainVal(_)
        | ExprKind::Closure(_)
        | ExprKind::Error => {}
        ExprKind::Do(block) => collect_block_calls(block, out),
        ExprKind::Some(a)
        | ExprKind::Unary(_, a)
        | ExprKind::Cast(_, a)
        | ExprKind::Field(a, _)
        | ExprKind::IsVariant(a, _)
        | ExprKind::Payload(a, _, _)
        | ExprKind::IsSome(a)
        | ExprKind::Unwrap(a, _) => collect_expr_calls(a, out),
        ExprKind::Binary(_, a, b)
        | ExprKind::Cmp(_, a, b)
        | ExprKind::Range { lo: a, hi: b } => {
            collect_expr_calls(a, out);
            collect_expr_calls(b, out);
        }
        ExprKind::If(a, b, c) => {
            collect_expr_calls(a, out);
            collect_expr_calls(b, out);
            collect_expr_calls(c, out);
        }
        ExprKind::Index(base, indices) => {
            collect_expr_calls(base, out);
            for i in indices {
                collect_expr_calls(i, out);
            }
        }
        ExprKind::Tuple(items) | ExprKind::Variant(_, items) | ExprKind::Builtin(_, items) => {
            for i in items {
                collect_expr_calls(i, out);
            }
        }
        ExprKind::Call(fid, args) => {
            out.push(*fid);
            for a in args {
                match a {
                    Arg::Value(v) => collect_expr_calls(v, out),
                    Arg::Mut(p) => collect_place_calls(p, out),
                }
            }
        }
        ExprKind::CallExpr(callee, args) => {
            collect_expr_calls(callee, out);
            for a in args {
                collect_expr_calls(a, out);
            }
        }
        ExprKind::Mutate(_, place, args) => {
            collect_place_calls(place, out);
            for a in args {
                collect_expr_calls(a, out);
            }
        }
        ExprKind::Sort(place, sort_by) => {
            collect_place_calls(place, out);
            match sort_by {
                SortBy::Natural => {}
                SortBy::Key(_, key) => collect_expr_calls(key, out),
                SortBy::Fn { func, .. } => collect_expr_calls(func, out),
            }
        }
        ExprKind::Read(spec) => collect_spec_calls(spec, out),
        ExprKind::Print(args) => {
            for a in args {
                collect_expr_calls(&a.value, out);
            }
        }
        ExprKind::Format(pieces) => {
            for p in pieces {
                if let FmtPiece::Value(a) = p {
                    collect_expr_calls(&a.value, out);
                }
            }
        }
        ExprKind::Fill(dims, value) => {
            for d in dims {
                if let DimSpec::Int(len) = d {
                    collect_expr_calls(len, out);
                }
            }
            if let Some(v) = value {
                collect_expr_calls(v, out);
            }
        }
    }
}

/// Whether `e` is a call of `name`: `name(...)` or `x.name(...)`.
fn calls_fn(e: &ast::Expr, name: &str) -> bool {
    let ast::ExprKind::Call { callee, .. } = &e.kind else {
        return false;
    };
    match &callee.kind {
        ast::ExprKind::Name(called) => called == name,
        ast::ExprKind::Field { name: called, .. } => called.text == name,
        _ => false,
    }
}

/// Whether a `return` under `block` gives a value. A nested fn's returns
/// are its own.
fn returns_a_value(block: &ast::Block) -> bool {
    struct Returns(bool);
    impl cpl_syntax::visit::Visitor for Returns {
        fn stmt(&mut self, stmt: &ast::Stmt) {
            if matches!(&stmt.kind, ast::StmtKind::Return(Some(_))) {
                self.0 = true;
            }
        }
    }
    let mut returns = Returns(false);
    for stmt in &block.stmts {
        if !matches!(stmt.kind, ast::StmtKind::Fn(_)) {
            cpl_syntax::visit::walk_stmt(stmt, &mut returns);
        }
    }
    returns.0
}

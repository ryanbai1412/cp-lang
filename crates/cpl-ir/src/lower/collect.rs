//! Pass 1 of collecting a program: what each top-level statement declares.

use super::*;

impl<'a> Lowerer<'a> {
    /// Registers what `stmt`, a statement of module `m` at `offset`, declares.
    pub(super) fn collect_item(&mut self, m: ModId, offset: u32, stmt: &'a ast::Stmt) {
        match &stmt.kind {
            ast::StmtKind::Struct(decl) => {
                if !decl.generics.is_empty() {
                    let gid = self.gen_structs.len();
                    self.gen_structs.push(GenStructTemplate {
                        mod_id: m,
                        offset,
                        decl,
                    });
                    self.mod_syms[m].insert(decl.name.text.clone(), Sym::GenStruct(gid));
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
                            fixed: Default::default(),
                            narrow_ints: Default::default(),
                            show_fn: None,
                            eq_fn: None,
                            ord: None,
                            cached_fields: Default::default(),
                        },
                        decl: Some(decl),
                        mod_id: m,
                        offset,
                        origin_gen: None,
                        env: HashMap::new(),
                        methods: HashMap::new(),
                        trait_methods: HashMap::new(),
                        op_methods: Vec::new(),
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
                                if is_projection(f) {
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
                                } else if !f.generics.is_empty() || self.takes_trait(f, m) {
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
                                    let fn_id =
                                        self.declare_fn_in(f, m, offset, Some(id), HashMap::new());
                                    self.structs[id].methods.insert(f.name.text.clone(), fn_id);
                                }
                            }
                            ast::StmtKind::Let {
                                pat,
                                ty,
                                init: Some(init),
                                ..
                            } if mono::let_const_name(pat).is_some() => {
                                let cname = mono::let_const_name(pat).expect("guarded");
                                self.structs[id].consts.insert(
                                    cname.to_string(),
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
                    self.mod_syms[m].insert(decl.name.text.clone(), Sym::GenEnum(gid));
                } else {
                    let id = self.enums.len();
                    self.mod_syms[m].insert(decl.name.text.clone(), Sym::Enum(id));
                    for (tag, variant) in decl.variants.iter().enumerate() {
                        self.mod_syms[m].insert(variant.name.text.clone(), Sym::Variant(id, tag));
                    }
                    let ename = if m == 0 {
                        decl.name.text.clone()
                    } else {
                        format!("m{m}_{}", decl.name.text)
                    };
                    self.enums.push(EnumDef {
                        name: ename,
                        variants: Vec::new(),
                        values: Vec::new(),
                        tags: Vec::new(),
                        show_fn: None,
                    });
                    self.enum_discriminants.push(Vec::new());
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
            ast::StmtKind::Let { pat, .. } => {
                fn register_pat_domains(
                    pat: &ast::Pattern,
                    m: usize,
                    domains: &mut Vec<String>,
                    mod_syms: &mut [HashMap<String, Sym>],
                ) {
                    match &pat.kind {
                        ast::PatternKind::Domain(name) => {
                            if !mod_syms[m].contains_key(&name.text) {
                                let d = domains.len();
                                let dname = if m == 0 {
                                    name.text.clone()
                                } else {
                                    format!("m{m}_{}", name.text)
                                };
                                domains.push(dname);
                                mod_syms[m].insert(name.text.clone(), Sym::Domain(d));
                            }
                        }
                        ast::PatternKind::Tuple(pats) => {
                            for p in pats {
                                register_pat_domains(p, m, domains, mod_syms);
                            }
                        }
                        _ => {}
                    }
                }
                register_pat_domains(pat, m, &mut self.domains, &mut self.mod_syms);
            }
            ast::StmtKind::TypeAlias {
                name, generics, ty, ..
            } => {
                if !generics.is_empty() {
                    let gaid = self.gen_aliases.len();
                    self.gen_aliases.push(GenAliasTemplate {
                        mod_id: m,
                        offset,
                        generics,
                        ty,
                    });
                    self.mod_syms[m].insert(name.text.clone(), Sym::GenAlias(gaid));
                } else {
                    let aid = self.aliases.len();
                    self.aliases.push((m, ty, offset));
                    self.mod_syms[m].insert(name.text.clone(), Sym::Alias(aid));
                }
            }
            ast::StmtKind::Fn(decl) => {
                if stmt.decorators.iter().any(|d| d.name.text == "test") && m != 0 {
                    return;
                }
                let decl = self.with_implicit_generics(decl, m, None);
                if !decl.generics.is_empty() || decl.params.iter().any(super::declare::is_opt_mut) {
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
                    if stmt.decorators.iter().any(|d| d.name.text == "memo") {
                        self.memo_fns.insert(id);
                        if let Some(table) = memo::into_table(stmt) {
                            self.memo_into.insert(id, table.to_string());
                        }
                    }
                }
            }
            ast::StmtKind::Context { name, fields } => {
                if m != 0 && name.text == "io" {
                    return;
                }
                let prev = self.cur_mod;
                self.cur_mod = m;
                let field_tys: Vec<(String, Ty)> = fields
                    .iter()
                    .map(|f| (f.name.text.clone(), self.resolve_ty(&f.ty)))
                    .collect();
                self.cur_mod = prev;
                let rec_ty = Ty::Record(field_tys.clone());
                let gid = self.globals.len();
                self.globals.push(Global {
                    name: format!("_ctx_{}", name.text),
                    ty: rec_ty,
                    span: name.span,
                });
                self.mut_globals.insert(gid);
                self.mod_syms[m].insert(name.text.clone(), Sym::Global(gid));
                self.context_defs
                    .insert(name.text.clone(), (gid, field_tys));
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
                    self.mod_syms[m].insert(name.text.clone(), Sym::GenConst(gcid));
                } else {
                    if let Some(v) = self.eval_const_int(init) {
                        self.const_ints.insert(name.text.clone(), v);
                    }
                    if let Some(v) = self.eval_const_bool(init) {
                        self.const_bools.insert(name.text.clone(), v);
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
                        self.mod_syms[m].insert(name.text.clone(), Sym::ModConst(mcid));
                    }
                }
            }
            ast::StmtKind::Trait {
                name,
                generics,
                body,
                supers,
                alias,
                ..
            } => {
                let tid = self.traits.len();
                let alias_of = alias.then(|| {
                    supers
                        .iter()
                        .filter_map(|t| match &t.kind {
                            ast::TypeKind::Named { name, .. } => Some(name.text.clone()),
                            _ => None,
                        })
                        .collect()
                });
                let (methods, fields) = fn_value::trait_members(body);
                let unimplemented = stmt
                    .decorators
                    .iter()
                    .filter(|d| d.name.text == "on_unimplemented")
                    .find_map(|d| {
                        let arg = d.args.first()?;
                        let ast::ExprKind::Str(_) = &arg.value.kind else {
                            return None;
                        };
                        let abs = arg.value.span.shift(offset);
                        let raw = self.mods[m]
                            .text
                            .get(abs.start as usize..abs.end as usize)?;
                        Some(raw.trim_matches('"').to_string())
                    });
                self.traits.push(TraitInfo {
                    mod_id: m,
                    offset,
                    name: name.text.clone(),
                    methods,
                    fields,
                    generic: !generics.is_empty(),
                    unimplemented,
                    alias_of,
                });
                self.mod_syms[m].insert(name.text.clone(), Sym::Trait(tid));
            }
            ast::StmtKind::Mod { name, body } => {
                // An inline `mod` is a module of its own over its parent's source.
                let sub_mod = self.mod_syms.len();
                self.mods.push(self.mods[m]);
                self.mod_syms.push(HashMap::new());
                self.inline_mods.insert(sub_mod, (m, &body.stmts));
                self.mod_syms[m].insert(name.text.clone(), Sym::Mod(sub_mod));
                let saved = std::mem::replace(&mut self.cur_mod, sub_mod);
                for s in &body.stmts {
                    self.collect_item(sub_mod, offset, s);
                }
                self.cur_mod = saved;
            }
            ast::StmtKind::Impl {
                trait_name,
                target,
                bounds,
                body,
            } => {
                let (tname, targs) = match trait_name.as_ref().map(|t| &t.kind) {
                    Some(ast::TypeKind::Named { name, args }) => {
                        (Some(name.text.clone()), args.iter().collect())
                    }
                    _ => (None, Vec::new()),
                };
                self.impls.push(ImplInfo {
                    mod_id: m,
                    offset,
                    trait_name: tname,
                    trait_args: targs,
                    target,
                    bounds,
                    body,
                });
            }
            ast::StmtKind::Use {
                path, names, glob, ..
            } => self.use_sibling(m, path, names, *glob),
            _ => {}
        }
    }

    /// Whether `name` in `m` is a type whose methods hang on a struct; an
    /// enum's, a sum-domain enum's included, are fns on the type instead.
    pub(super) fn names_struct_like(&self, m: ModId, name: &str) -> bool {
        match self.mod_syms[m].get(name) {
            None | Some(Sym::Enum(_)) => false,
            Some(Sym::Domain(d)) => !self.sum_enums.contains_key(d),
            Some(_) => true,
        }
    }

    /// An enum's values: `Ty::Enum`, or the positions of a sum-domain enum.
    pub(super) fn is_enum_ty(&self, ty: &Ty) -> bool {
        match ty {
            Ty::Enum(_) => true,
            Ty::Pos(d) => self.sum_enums.contains_key(d),
            _ => false,
        }
    }

    /// `use a.f` inside `mod b:`, `a` an inline module beside `b`: `f` names
    /// `a`'s item in `b` (R11.2.2).
    fn use_sibling(&mut self, m: ModId, path: &[ast::Name], names: &[ast::Name], glob: bool) {
        let Some(&(parent, _)) = self.inline_mods.get(&m) else {
            return;
        };
        let (module, picked) = match (names.is_empty() && !glob, path) {
            (true, [module @ .., last]) => (module, std::slice::from_ref(last)),
            _ => (path, names),
        };
        let mut sib = parent;
        for part in module {
            match self.mod_syms[sib].get(&part.text) {
                Some(&Sym::Mod(next)) if self.inline_mods.contains_key(&next) => sib = next,
                _ => return,
            }
        }
        if sib == parent {
            return;
        }
        let found: Vec<(String, Sym)> = match glob {
            true => self.mod_syms[sib]
                .iter()
                .map(|(n, s)| (n.clone(), *s))
                .collect(),
            false => picked
                .iter()
                .filter_map(|n| Some((n.text.clone(), *self.mod_syms[sib].get(&n.text)?)))
                .collect(),
        };
        self.mod_syms[m].extend(found);
    }
}

/// A `lends` fn that takes `self` lends part of its receiver; a static
/// one is an ordinary fn whose result is lent (R05.5.2).
pub(super) fn is_projection(f: &ast::FnDecl) -> bool {
    f.lends.is_some() && f.params.first().is_some_and(|p| p.name.text == "self")
}

/// `fn f(..) = @cpp("...")`: a body written in C++ (Rbundle.5).
pub(super) fn is_cpp(e: &ast::Expr) -> bool {
    matches!(e.kind, ast::ExprKind::Cpp(_))
}

impl<'a> Lowerer<'a> {
    /// `fn new(next: Next<V>)`: a trait-typed parameter makes the fn generic in
    /// that argument's type, as an explicit `<N: Next<V>>` would.
    /// A `?mut T` param does too: each call shape is its own instance.
    pub(super) fn takes_trait(&self, f: &ast::FnDecl, m: usize) -> bool {
        f.params.iter().any(|p| {
            matches!(
                &p.ty,
                Some(ast::Type { kind: ast::TypeKind::Named { name, .. }, .. })
                    if matches!(self.mod_syms[m].get(&name.text), Some(Sym::Trait(_)))
            ) || declare::is_opt_mut(p)
        })
    }

    /// The fns no engine but the C++ can run: the judge's grader's, and
    /// the `@cpp` ones, which are in `judge_fns` too, with their C++.
    #[allow(clippy::type_complexity)]
    pub(super) fn opaque_fns(
        &mut self,
        analysis: &'a Analysis,
    ) -> (Vec<FnId>, Vec<FnId>, Vec<(FnId, String)>) {
        let (mut judge_fns, entry_fns) = self.grader_fns(analysis);
        let mut cpp = Vec::new();
        for id in 0..self.sigs.len() {
            if let ast::FnBody::Expr(ast::Expr {
                kind: ast::ExprKind::Cpp(code),
                ..
            }) = &self.sigs[id].decl.body
            {
                if self.bodies[id].is_some() {
                    judge_fns.push(id);
                    cpp.push((id, code.clone()));
                }
            }
        }
        (judge_fns, entry_fns, cpp)
    }
}

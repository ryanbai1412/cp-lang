//! Instantiating generic structs and enums, attaching their impls, and
//! evaluating generic, module and struct constants.

use super::*;

impl<'a> Lowerer<'a> {
    /// Instantiates a generic struct template with concrete `args`, memoized.
    pub(in crate::lower) fn instantiate_struct(
        &mut self,
        gid: GenStructId,
        mut args: Vec<GenArg>,
    ) -> StructId {
        let tpl_mod = self.gen_structs[gid].mod_id;
        let tpl_offset = self.gen_structs[gid].offset;
        let decl = self.gen_structs[gid].decl;
        if args.len() < decl.generics.len() {
            let saved_mod = std::mem::replace(&mut self.cur_mod, tpl_mod);
            let saved_offset = std::mem::replace(&mut self.offset, tpl_offset);
            let saved_env = self.gen_env.clone();
            let mut env: GenEnv = HashMap::new();
            for (p, a) in decl.generics.iter().zip(&args) {
                env.insert(p.text.clone(), a.clone());
            }
            for i in args.len()..decl.generics.len() {
                if let Some(Some(default_ty)) = decl.generic_defaults.get(i) {
                    self.gen_env = env.clone();
                    let arg = self.resolve_gen_arg(default_ty);
                    env.insert(decl.generics[i].text.clone(), arg.clone());
                    args.push(arg);
                } else {
                    break;
                }
            }
            self.gen_env = saved_env;
            self.cur_mod = saved_mod;
            self.offset = saved_offset;
        }
        let key = InstKey::Struct(gid, args.clone());
        if let Some(&id) = self.insts.get(&key) {
            return id;
        }
        let id = self.structs.len();
        let suffix: Vec<String> = args.iter().map(|a| self.mangle_arg(a)).collect();
        let mangled = format!(
            "{}_{}_{id}",
            decl.name.text.replace('\'', "_a"),
            suffix.join("_")
        );

        let mut env: GenEnv = HashMap::new();
        for (p, a) in decl.generics.iter().zip(&args) {
            env.insert(p.text.clone(), a.clone());
        }
        // `struct Box<T, M: Op<T>>` at `M = Z`: its methods' `op.lift(x)` needs the `T`
        // solving the bound gave `impl<T> Op<T> for Z`, as a generic fn's body does.
        let mut solved = env.clone();
        self.bind_impl_bounds(&decl.bounds, tpl_mod, &mut solved);
        env.extend(
            solved
                .into_iter()
                .filter(|(k, _)| k.starts_with("impl") && k.contains('.')),
        );

        self.structs.push(StructInfo {
            def: StructDef {
                name: mangled,
                source: decl.name.text.clone(),
                targs: shown_args(&args),
                fields: Vec::new(),
                ..Default::default()
            },
            decl: Some(decl),
            mod_id: tpl_mod,
            offset: tpl_offset,
            origin_gen: Some((gid, args.clone())),
            env: env.clone(),
            methods: HashMap::new(),
            trait_methods: HashMap::new(),
            op_methods: Vec::new(),
            gen_methods: HashMap::new(),
            open_impls: Vec::new(),
            getters: HashSet::new(),
            projections: HashMap::new(),
            consts: HashMap::new(),
            owned_domains: HashMap::new(),
            owned_domain_inits: Vec::new(),
            laws: Vec::new(),
            field_ranges: Vec::new(),
            index_domain: None,
        });
        self.insts.insert(key, id);

        // Switch context to the struct's defining module and generic env.
        let saved_mod = std::mem::replace(&mut self.cur_mod, tpl_mod);
        let saved_offset = std::mem::replace(&mut self.offset, tpl_offset);
        let saved_env = std::mem::replace(&mut self.gen_env, env);
        let saved_self = self.self_struct.replace(id);

        // 1. Collect struct-owned domains (e.g. `domain Pre = 0..V.K` in `Rooted<V>`).
        let field_names: Vec<&str> = decl.fields.iter().map(|f| f.name.text.as_str()).collect();
        for item in &decl.methods {
            if let ast::StmtKind::Domain { name, def } = &item.kind {
                let dom_id = self.domains.len();
                self.domains
                    .push(format!("{}_{}", self.structs[id].def.name, name.text));
                self.structs[id]
                    .owned_domains
                    .insert(name.text.clone(), dom_id);
                self.structs[id].owned_domain_inits.push(OwnedDomainInit {
                    id: dom_id,
                    def,
                    offset: tpl_offset,
                    mod_id: tpl_mod,
                    per_value: cpl_syntax::visit::mentions_any(def, &field_names),
                });
                self.gen_env
                    .insert(name.text.clone(), GenArg::Domain(dom_id));
                self.structs[id]
                    .env
                    .insert(name.text.clone(), GenArg::Domain(dom_id));
            }
        }

        // 2. Resolve fields.
        let fields: Vec<(String, Ty)> = decl
            .fields
            .iter()
            .map(|f| (f.name.text.clone(), self.resolve_ty(&f.ty)))
            .collect();
        self.structs[id].def.fields = fields;
        self.structs[id].def.cached_fields = decl
            .fields
            .iter()
            .enumerate()
            .filter(|(_, f)| f.cached)
            .map(|(i, _)| i)
            .collect();
        self.structs[id].def.bits_fields = decl
            .fields
            .iter()
            .enumerate()
            .filter(|(_, f)| f.bits)
            .map(|(i, _)| i)
            .collect();
        self.structs[id].def.hidden = self.hidden_fields(self.structs[id].mod_id, decl);
        self.structs[id].def.fixed = self.fixed_rows(decl);
        self.structs[id].def.narrow_ints = self.narrow_int_fields(decl);
        self.structs[id].def.int_ranges = self.int_field_ranges(decl);
        self.collect_struct_field_ranges(id, decl);

        // 3. Collect methods, getters, projections, laws, constants.
        let struct_env = self.gen_env.clone();
        for item in &decl.methods {
            let is_get = item.decorators.iter().any(|d| d.name.text == "get");
            match &item.kind {
                ast::StmtKind::Fn(f) => {
                    if is_get {
                        self.structs[id].getters.insert(f.name.text.clone());
                    }
                    if collect::is_projection(f) {
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
                    } else if !f.generics.is_empty() || self.takes_trait(f, tpl_mod) {
                        let gfid = self.gen_fns.len();
                        self.gen_fns.push(GenFnTemplate {
                            mod_id: tpl_mod,
                            offset: tpl_offset,
                            owner: Some(id),
                            owner_env: struct_env.clone(),
                            decl: f,
                        });
                        self.structs[id]
                            .gen_methods
                            .insert(f.name.text.clone(), gfid);
                    } else {
                        let fn_id = self.declare_fn_in(
                            f,
                            tpl_mod,
                            tpl_offset,
                            Some(id),
                            struct_env.clone(),
                        );
                        // Read on first use: `Rev<S>.take -> Rev<Slice<S>>` would
                        // otherwise make instances without end.
                        self.quiet_sigs.insert(fn_id);
                        self.structs[id].methods.insert(f.name.text.clone(), fn_id);
                    }
                }
                ast::StmtKind::Let {
                    pat,
                    ty,
                    init: Some(init),
                    ..
                } if let Some(cname) = let_const_name(pat) => {
                    self.structs[id].consts.insert(
                        cname.to_string(),
                        StructConst {
                            mod_id: tpl_mod,
                            offset: tpl_offset,
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
                        trait_fns: Vec::new(),
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
    pub(in crate::lower) fn instantiate_enum(
        &mut self,
        gid: GenEnumId,
        args: Vec<GenArg>,
    ) -> EnumId {
        let key = InstKey::Enum(gid, args.clone());
        if let Some(&id) = self.insts.get(&key) {
            return id;
        }
        let tpl_mod = self.gen_enums[gid].mod_id;
        let tpl_offset = self.gen_enums[gid].offset;
        let decl = self.gen_enums[gid].decl;
        let id = self.enums.len();
        let suffix: Vec<String> = args.iter().map(|a| self.mangle_arg(a)).collect();
        let mangled = format!(
            "{}_{}_{id}",
            decl.name.text.replace('\'', "_a"),
            suffix.join("_")
        );
        self.enums.push(EnumDef {
            name: mangled,
            source: decl.name.text.clone(),
            targs: shown_args(&args),
            variants: Vec::new(),
            values: Vec::new(),
            tags: Vec::new(),
            show_fn: None,
        });
        self.insts.insert(key, id);

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
        self.enums[id].tags = decl.variants.iter().map(|v| self.variant_tag(v)).collect();
        self.cur_mod = saved_mod;
        self.offset = saved_offset;
        self.gen_env = saved_env;
        id
    }

    /// Finds and attaches all `impl` blocks matching concrete struct `sid`.
    pub(in crate::lower) fn attach_impls_for_struct(&mut self, sid: StructId) {
        let item = self
            .item_types
            .iter()
            .find(|(_, s)| *s == sid)
            .map(|(k, _)| k.clone());
        let struct_decl_name = match self.structs[sid].decl {
            Some(sdecl) => sdecl.name.text.clone(),
            None if item.is_some() => String::new(),
            None => return,
        };
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
            let imp_trait_args = self.impls[idx].trait_args.clone();
            let imp_target = self.impls[idx].target;
            let imp_body = self.impls[idx].body;

            let (target_matches, targs): (bool, &[ast::Type]) = match &imp_target.kind {
                ast::TypeKind::Named {
                    name: tname,
                    args: targs,
                } if item.is_none() => {
                    // Check whether `tname` in `imp_mod` refers to the same struct.
                    let keyed = cpl_check::arity_key(&tname.text, targs.len());
                    let syms = &self.mod_syms[imp_mod];
                    let hit = match syms.get(&keyed).or_else(|| syms.get(&tname.text)).copied() {
                        Some(Sym::Struct(id)) => id == sid,
                        Some(Sym::GenStruct(gid)) => self.structs[sid]
                            .origin_gen
                            .as_ref()
                            .is_some_and(|(g, _)| *g == gid),
                        _ => tname.text == struct_decl_name && imp_mod == struct_mod,
                    };
                    (hit, targs)
                }
                ast::TypeKind::ItemOf(of) => (item.is_some() && of.fn_item_ident() == item, &[]),
                _ => continue,
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
            if !ok || !self.bind_impl_bounds(self.impls[idx].bounds, imp_mod, &mut env) {
                continue;
            }
            if !self.open_impl_params(idx, &env).is_empty() {
                self.structs[sid].open_impls.push(idx);
                continue;
            }

            let saved_mod = std::mem::replace(&mut self.cur_mod, imp_mod);
            let saved_offset = std::mem::replace(&mut self.offset, imp_offset);
            let saved_env = std::mem::replace(&mut self.gen_env, env.clone());
            let saved_self = self.self_struct.replace(sid);

            // Handle `impl Index for S`: allocate backing domain for `let count = ...`.
            if imp_trait.as_deref() == Some("Index") && self.structs[sid].index_domain.is_none() {
                let dom_id = self.domains.len();
                self.domains.push(self.structs[sid].def.name.clone());
                self.structs[sid].index_domain = Some(dom_id);
            }

            let imp_rhs_ty: Option<Ty> = imp_trait_args.first().map(|t| self.resolve_ty(t));
            let imp_out_ty: Option<Ty> = imp_body.stmts.iter().find_map(|s| match &s.kind {
                ast::StmtKind::TypeAlias { name, ty, .. } if name.text == "Out" => {
                    Some(self.resolve_ty(ty))
                }
                _ => None,
            });
            if let Some(ref o) = imp_out_ty {
                env.insert("Out".to_string(), GenArg::Ty(o.clone()));
                self.gen_env
                    .insert("Out".to_string(), GenArg::Ty(o.clone()));
            }

            for item in &imp_body.stmts {
                let is_get = item.decorators.iter().any(|d| d.name.text == "get");
                match &item.kind {
                    ast::StmtKind::Fn(f) => {
                        if is_get {
                            self.structs[sid].getters.insert(f.name.text.clone());
                        }
                        if collect::is_projection(f) {
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
                        } else if !f.generics.is_empty() || self.takes_trait(f, imp_mod) {
                            let gfid = self.gen_fns.len();
                            self.gen_fns.push(GenFnTemplate {
                                mod_id: imp_mod,
                                offset: imp_offset,
                                owner: Some(sid),
                                owner_env: env.clone(),
                                decl: f,
                            });
                            self.structs[sid]
                                .gen_methods
                                .insert(f.name.text.clone(), gfid);
                        } else {
                            let fn_id =
                                self.declare_fn_in(f, imp_mod, imp_offset, Some(sid), env.clone());
                            self.resolve_fn_sig_with(
                                fn_id,
                                imp_trait.as_deref(),
                                imp_rhs_ty.as_ref(),
                                imp_out_ty.as_ref(),
                            );
                            let (tr, rhs) = (imp_trait.as_deref(), imp_rhs_ty.clone());
                            if self.register_op_impl(sid, fn_id, tr, &f.name.text, rhs) {
                                // Dispatched by its operand's type, not by name.
                            } else if let Some(tname) = imp_trait
                                .as_ref()
                                .filter(|_| self.structs[sid].methods.contains_key(&f.name.text))
                            {
                                // An inherent method of the same name wins the lookup (R08.1.4).
                                let key = (tname.clone(), f.name.text.clone());
                                self.structs[sid].trait_methods.insert(key, fn_id);
                            } else {
                                if let Some(tname) = &imp_trait {
                                    let key = (tname.clone(), f.name.text.clone());
                                    self.structs[sid].trait_methods.insert(key, fn_id);
                                }
                                self.structs[sid].methods.insert(f.name.text.clone(), fn_id);
                            }
                            let def = &mut self.structs[sid].def;
                            match (imp_trait.as_deref(), f.name.text.as_str()) {
                                (Some("Show"), "show") => def.show_fn = Some(fn_id),
                                (Some("Eq"), "eq") => def.eq_fn = Some(fn_id),
                                (Some("Ord"), "lt") => def.ord = Some(OrdHook::Lt(fn_id)),
                                (Some("Ord"), "cmp") => def.ord = Some(OrdHook::Cmp(fn_id)),
                                _ => {}
                            }
                        }
                    }
                    ast::StmtKind::Let {
                        pat,
                        ty,
                        init: Some(init),
                        ..
                    } => {
                        let cname = let_const_name(pat).map(str::to_string);
                        if imp_trait.as_deref() == Some("Index")
                            && cname.as_deref() == Some("count")
                        {
                            let generic = self.structs[sid].origin_gen.is_some();
                            if let (true, Some(dom_id)) = (generic, self.structs[sid].index_domain)
                            {
                                let len = self.expr_as(init, &Ty::Int);
                                self.size_library_domain(dom_id, len);
                            }
                        }
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
                    ast::StmtKind::Const {
                        name,
                        ty,
                        init: Some(init),
                        ..
                    } => {
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
                    let t_env = self.default_fn_env(tid, &imp_trait_args, imp_body, &env);
                    let t_fns: Vec<String> =
                        t_methods.iter().map(|m| m.name.text.clone()).collect();
                    for m in t_methods {
                        if !matches!(m.body, ast::FnBody::None)
                            && !self.structs[sid].methods.contains_key(&m.name.text)
                        {
                            let fn_id =
                                self.declare_fn_in(m, t_mod, t_offset, Some(sid), t_env.clone());
                            self.default_trait_fns.insert(fn_id, t_fns.clone());
                            self.resolve_fn_sig(fn_id, Some(tname.as_str()));
                            self.structs[sid].methods.insert(m.name.text.clone(), fn_id);
                            let key = (tname.clone(), m.name.text.clone());
                            self.structs[sid].trait_methods.insert(key, fn_id);
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

    /// The names a trait's default fn sees inside an impl: the impl's own,
    /// the trait's parameters as the impl's trait args, and `Self.Out` as the
    /// type the impl chooses or else the trait's default (decision 469).
    fn default_fn_env(
        &mut self,
        tid: usize,
        trait_args: &[&ast::Type],
        imp_body: &ast::Block,
        env: &HashMap<String, GenArg>,
    ) -> HashMap<String, GenArg> {
        let mut t_env = env.clone();
        let generics = self.traits[tid].generics.clone();
        for (g, a) in generics.iter().zip(trait_args) {
            t_env.insert(g.to_string(), GenArg::Ty(self.resolve_ty(a)));
        }
        for s in &imp_body.stmts {
            if let ast::StmtKind::TypeAlias { name, ty, .. } = &s.kind {
                let chosen = self.resolve_ty(ty);
                t_env.insert(format!("Self.{}", name.text), GenArg::Ty(chosen));
            }
        }
        let defaults = self.traits[tid].assoc_defaults.clone();
        let (t_mod, t_offset) = (self.traits[tid].mod_id, self.traits[tid].offset);
        let saved_mod = std::mem::replace(&mut self.cur_mod, t_mod);
        let saved_offset = std::mem::replace(&mut self.offset, t_offset);
        let saved_env = std::mem::replace(&mut self.gen_env, t_env.clone());
        for (name, ty) in defaults {
            let key = format!("Self.{name}");
            if let std::collections::hash_map::Entry::Vacant(slot) = t_env.entry(key) {
                slot.insert(GenArg::Ty(self.resolve_ty(ty)));
            }
        }
        self.cur_mod = saved_mod;
        self.offset = saved_offset;
        self.gen_env = saved_env;
        t_env
    }

    /// Evaluates a generic `const Sum<T> = ...` with concrete `args`.
    pub(in crate::lower) fn eval_gen_const(&mut self, gcid: GenConstId, args: Vec<GenArg>) -> Expr {
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
    pub(in crate::lower) fn ensure_mod_const(&mut self, mcid: ModConstId, span: Span) -> Expr {
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
            span: Span::default(),
        });
        self.mod_consts[mcid].global = Some(gid);
        let place = Place {
            kind: PlaceKind::Global(gid),
            ty: val_ty.clone(),
            span: val.span,
        };
        if temp_cx.locals.is_empty() {
            self.preamble_stmts
                .push(Stmt::Assign(place, AssignOp::Set, val));
        } else {
            // Put into a 0-arg helper fn called in preamble so locals stay clean.
            let fid = self.sigs.len();
            let init_fn_name = format!("cpl_init_g{gid}");
            self.sigs.push(FnSig {
                name: init_fn_name.clone(),
                params: Vec::new(),
                ret: Some(val_ty.clone()),
                assumed_unit: false,
                owner: None,
                receiver: None,
                mod_id: tpl_mod,
                env: HashMap::new(),
                decl: self.dummy_fn_decl(),
                offset: tpl_offset,
                is_generator: false,
                new_domain_slots: Vec::new(),
                returns_fresh_domain: false,
                used_fresh_call: false,
                state: State::Done,
                sig: SigState::Resolved,
                judge_ranges: Default::default(),
                asked_at: self.asked_at.callee(self.cur_mod),
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
            self.preamble_stmts
                .push(Stmt::Assign(place, AssignOp::Set, call));
        }
        self.mk(ExprKind::Global(gid), val_ty, span)
    }

    /// Evaluates a struct-associated constant (e.g. `Mod<P>.zero`, `Mod<P>.one`).
    pub(in crate::lower) fn eval_struct_const(
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
}

/// A generic instance's arguments as types, for showing `Pair<Int>`; none
/// when one is a number or an operator, which shows as the bare name.
fn shown_args(args: &[GenArg]) -> Vec<Ty> {
    args.iter()
        .map(|a| match a {
            GenArg::Ty(t) => Some(t.clone()),
            GenArg::Domain(d) => Some(Ty::Pos(*d)),
            _ => None,
        })
        .collect::<Option<_>>()
        .unwrap_or_default()
}

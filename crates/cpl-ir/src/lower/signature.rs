//! Fn signatures: declaring fns, implicit generics, associated types, impls
//! for built-in types, and reading parameter and return types on demand.

use super::*;
use cpl_diag::msgs;

impl<'a> Lowerer<'a> {
    /// `fn size(s: List<T>)`: an undeclared one-letter type in the signature
    /// is a type parameter of the fn (R02.10.3).
    /// `inline` is the body of an inline `mod m:`, whose names live in no
    /// `self.mods` entry of their own.
    pub(in crate::lower) fn with_implicit_generics(
        &self,
        decl: &'a ast::FnDecl,
        m: usize,
        inline: Option<&'a [ast::Stmt]>,
    ) -> &'a ast::FnDecl {
        if !decl.generics.is_empty() {
            return decl;
        }
        let mut names = Vec::new();
        for p in &decl.params {
            if let Some(t) = &p.ty {
                implicit_type_names(t, &mut names);
            }
        }
        if let Some(t) = &decl.ret {
            implicit_type_names(t, &mut names);
        }
        if names.is_empty() {
            return decl;
        }
        let inline = inline.or_else(|| self.inline_mod(m).map(|(_, body)| body));
        let items: Box<dyn Iterator<Item = &'a ast::Stmt> + '_> = match inline {
            Some(stmts) => Box::new(stmts.iter()),
            None => Box::new(self.mod_items(m).map(|(_, stmt)| stmt)),
        };
        let declared: HashSet<&str> = items
            .flat_map(|stmt| -> Vec<&str> {
                match &stmt.kind {
                    ast::StmtKind::Struct(d) => vec![d.name.text.as_str()],
                    ast::StmtKind::Enum(d) => vec![d.name.text.as_str()],
                    ast::StmtKind::Domain { name, .. }
                    | ast::StmtKind::TypeAlias { name, .. }
                    | ast::StmtKind::Trait { name, .. } => vec![name.text.as_str()],
                    ast::StmtKind::Use { names, .. } => {
                        names.iter().map(|n| n.text.as_str()).collect()
                    }
                    _ => vec![],
                }
            })
            .collect();
        names.retain(|n| !declared.contains(n.as_str()) && !self.mod_syms[m].contains_key(n));
        if names.is_empty() {
            return decl;
        }
        let mut implicit = decl.clone();
        implicit.generics = names
            .into_iter()
            .map(|text| ast::Name {
                text,
                span: decl.name.span,
            })
            .collect();
        Box::leak(Box::new(implicit))
    }

    /// `type Fold = T` in the trait of impl `idx`, for an impl that leaves `item` out (decision 469).
    fn assoc_default(&self, idx: usize, item: &str) -> Option<AssocSrc<'a>> {
        let name = self.impls[idx].trait_name.as_deref()?;
        let tid = self.traits.iter().position(|t| t.name == name)?;
        let (_, ty) = self.traits[tid]
            .assoc_defaults
            .iter()
            .find(|(n, _)| *n == item)?;
        Some(AssocSrc::Default(tid, ty))
    }

    /// The type an impl's associated type names, in the impl's env; a trait default sees the
    /// trait's generics as the impl's trait arguments.
    fn resolve_assoc_src(&mut self, idx: usize, src: AssocSrc<'a>) -> Ty {
        let (tid, default) = match src {
            AssocSrc::Impl(ty) => return self.resolve_ty(ty),
            AssocSrc::Default(tid, ty) => (tid, ty),
        };
        let trait_args = self.impls[idx].trait_args.clone();
        let args: Vec<Ty> = trait_args.iter().map(|a| self.resolve_ty(a)).collect();
        let tr = &self.traits[tid];
        let env: GenEnv = (tr.generics.iter().zip(args))
            .map(|(g, ty)| (g.to_string(), GenArg::Ty(ty)))
            .collect();
        let saved = (
            std::mem::replace(&mut self.cur_mod, tr.mod_id),
            std::mem::replace(&mut self.offset, tr.offset),
            std::mem::replace(&mut self.gen_env, env),
        );
        let found = self.resolve_ty(default);
        (self.cur_mod, self.offset, self.gen_env) = saved;
        found
    }

    /// `B.Item` for `B` bound to `t`: the `type Item = ..` of an impl for `t`.
    pub(in crate::lower) fn assoc_type(&mut self, t: &Ty, item: &str) -> Option<Ty> {
        for idx in 0..self.impls.len() {
            let (m, offset, body) = (
                self.impls[idx].mod_id,
                self.impls[idx].offset,
                self.impls[idx].body,
            );
            let alias = body.stmts.iter().find_map(|s| match &s.kind {
                ast::StmtKind::TypeAlias {
                    name, generics, ty, ..
                } if name.text == item && generics.is_empty() => Some(ty),
                _ => None,
            });
            let alias = match alias {
                Some(ty) => AssocSrc::Impl(ty),
                None => match self.assoc_default(idx, item) {
                    Some(found) => found,
                    None => continue,
                },
            };
            let opened = match t {
                Ty::Struct(sid) => self.opened_impl_env(idx, *sid),
                _ => None,
            };
            let Some(env) = opened.or_else(|| self.impl_env(idx, t)) else {
                continue;
            };
            let saved = (
                std::mem::replace(&mut self.cur_mod, m),
                std::mem::replace(&mut self.offset, offset),
                std::mem::replace(&mut self.gen_env, env),
            );
            let found = self.resolve_assoc_src(idx, alias);
            (self.cur_mod, self.offset, self.gen_env) = saved;
            return Some(found);
        }
        // Structural trait associated types: if a trait method returns `Self.<item>`
        // or `yields Self.<item>`, recover `<item>` from the concrete type's matching
        // method (or sequence-valued adjacency array / function).
        let self_item = format!("Self.{item}");
        let unwrap_seq = |seq_ty: &Ty| -> Option<Ty> {
            match seq_ty {
                Ty::List(e) | Ty::ListView(e, _) | Ty::MutView(e, _) | Ty::Range(e) => {
                    Some((**e).clone())
                }
                Ty::Str | Ty::StrView => Some(Ty::Char),
                Ty::Array { elem, .. } => Some((**elem).clone()),
                _ => None,
            }
        };
        let mut cands = Vec::new();
        for tr in &self.traits {
            for m in &tr.methods {
                let Some(ret_ast) = &m.ret else { continue };
                let (matched, yielded) = match &ret_ast.kind {
                    ast::TypeKind::Named { name, args }
                        if args.is_empty() && name.text == self_item =>
                    {
                        (true, false)
                    }
                    ast::TypeKind::Yields(inner) => match &inner.kind {
                        ast::TypeKind::Named { name, args }
                            if args.is_empty() && name.text == self_item =>
                        {
                            (true, true)
                        }
                        _ => (false, false),
                    },
                    _ => (false, false),
                };
                if matched {
                    cands.push((m.name.text.clone(), yielded));
                }
            }
        }
        for (method, yielded) in cands {
            {
                if let Ty::Struct(sid) = t {
                    if let Some(fn_id) = self.own_method(*sid, &method) {
                        if let Some(ret_ty) = &self.sigs[fn_id].ret {
                            if yielded {
                                if let Some(elem) = unwrap_seq(ret_ty) {
                                    return Some(elem);
                                }
                            } else {
                                return Some(ret_ty.clone());
                            }
                        }
                    }
                }
                if yielded {
                    match t {
                        Ty::Array { elem, .. } => {
                            if let Some(arc) = unwrap_seq(elem) {
                                return Some(arc);
                            }
                        }
                        Ty::Fn(_, ret) => {
                            if let Some(arc) = unwrap_seq(ret) {
                                return Some(arc);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        if item == "Item" {
            if let Some(elem) = unwrap_seq(t) {
                return Some(elem);
            }
        }
        None
    }

    /// `impl T for Int`: a built-in type has no struct to hang methods
    /// on, so each method is a fn whose `self` is the type, found by
    /// `x.method()` on a value of it.
    pub(in crate::lower) fn attach_impls_for_builtins(&mut self) {
        for idx in 0..self.impls.len() {
            let imp = &self.impls[idx];
            let (m, offset, target, body) = (imp.mod_id, imp.offset, imp.target, imp.body);
            if self.domain_blanket_param(idx).is_some() || self.row_param(idx).is_some() {
                continue;
            }
            // `impl Pick for Int -> Bool`: fn values of that type get its methods.
            let open = self.impl_params(m, target);
            let head = match &target.kind {
                ast::TypeKind::Named { name, args }
                    if args.is_empty()
                        && !self.names_struct_like(m, &name.text)
                        && !open.contains(&name.text) =>
                {
                    name.text.clone()
                }
                ast::TypeKind::Fn { .. } if open.is_empty() => format!("fn{idx}"),
                _ => continue,
            };
            let trait_name = self.impls[idx].trait_name.clone();
            let saved = (
                std::mem::replace(&mut self.cur_mod, m),
                std::mem::replace(&mut self.offset, offset),
            );
            let ty = self.resolve_ty(target);
            if !matches!(ty, Ty::Struct(_) | Ty::Error) {
                for item in &body.stmts {
                    if let ast::StmtKind::Fn(f) = &item.kind {
                        if !f.generics.is_empty() || f.lends.is_some() {
                            continue;
                        }
                        let fn_id = self.declare_fn_in(f, m, offset, None, GenEnv::default());
                        self.sigs[fn_id].name = format!(
                            "{}_{}_{}",
                            trait_name.as_deref().unwrap_or("impl"),
                            head,
                            f.name.text
                        );
                        self.prim_self.insert(fn_id, ty.clone());
                        self.prim_methods
                            .push((ty.clone(), f.name.text.clone(), fn_id));
                        self.resolve_fn_sig(fn_id, trait_name.as_deref());
                        if let (Ty::Enum(e), Some("Show"), "show") =
                            (&ty, trait_name.as_deref(), f.name.text.as_str())
                        {
                            self.enums[*e].show_fn = Some(fn_id);
                        }
                    }
                }
            }
            (self.cur_mod, self.offset) = saved;
        }
    }

    pub(crate) fn declare_fn_in(
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
            assumed_unit: false,
            owner,
            receiver: None,
            mod_id,
            env,
            decl,
            offset,
            is_generator,
            new_domain_slots: Vec::new(),
            returns_fresh_domain: false,
            used_fresh_call: false,
            state: State::Pending,
            sig: SigState::Unresolved,
            judge_ranges: Default::default(),
            asked_at: self.asked_at.callee(self.cur_mod),
        });
        self.bodies.push(None);
        self.sigs.len() - 1
    }

    /// Second pass: field, payload and parameter types for non-generic declarations.
    pub(in crate::lower) fn signatures(&mut self) {
        for id in 0..self.enum_decls.len() {
            let (m, offset, decl) = self.enum_decls[id];
            self.cur_mod = m;
            self.offset = offset;
            let mut next_disc: i64 = 0;
            let mut discs = Vec::with_capacity(decl.variants.len());
            let variants: Vec<(String, Vec<Ty>)> = decl
                .variants
                .iter()
                .map(|v| {
                    let disc = if let Some(val) = &v.value {
                        self.eval_const_int(val).unwrap_or(next_disc)
                    } else {
                        next_disc
                    };
                    discs.push(disc);
                    next_disc = disc.saturating_add(1);
                    let fields = v.fields.iter().map(|(_, t)| self.resolve_ty(t)).collect();
                    (v.name.text.clone(), fields)
                })
                .collect();
            self.enum_discriminants[id] = discs;
            let values = decl
                .variants
                .iter()
                .map(|v| v.value.as_ref().and_then(|e| self.eval_const_int(e)))
                .collect();
            self.enums[id].variants = variants;
            self.enums[id].values = values;
            self.enums[id].tags = decl.variants.iter().map(|v| self.variant_tag(v)).collect();
            // §2.16: a payload enum whose every variant carries only finite-index
            // payloads is a finite index domain (sum domain), lowered as a flat `DomainId`.
            let has_payload = self.enums[id].variants.iter().any(|(_, fs)| !fs.is_empty());
            if has_payload && !self.enums[id].variants.is_empty() {
                let sum_variants: Option<Vec<(String, Vec<ProductFactor>)>> = self.enums[id]
                    .variants
                    .iter()
                    .map(|(vname, fs)| {
                        let pfs: Option<Vec<ProductFactor>> =
                            fs.iter().map(|t| self.ty_to_product_factor(t)).collect();
                        pfs.map(|pfs| (vname.clone(), pfs))
                    })
                    .collect();
                if let Some(sum_variants) = sum_variants {
                    let d = self.domains.len();
                    self.domains.push(self.enums[id].name.clone());
                    self.sum_enums.insert(
                        d,
                        SumEnumInfo {
                            enum_id: id,
                            variants: sum_variants,
                        },
                    );
                    self.enum_to_sum_domain.insert(id, d);
                    self.mod_syms[m].insert(decl.name.text.clone(), Sym::Domain(d));
                }
            }
        }
        let num_structs = self.structs.len();
        for id in 0..num_structs {
            let Some(decl) = self.structs[id].decl else {
                continue;
            };
            let (m, offset) = (self.structs[id].mod_id, self.structs[id].offset);
            self.cur_mod = m;
            self.offset = offset;
            self.self_struct = Some(id);
            let fields = decl
                .fields
                .iter()
                .map(|f| {
                    let ty = match (&f.ty.kind, &f.default) {
                        (ast::TypeKind::Infer, Some(default)) => literal_ty(default),
                        _ => self.resolve_ty(&f.ty),
                    };
                    (f.name.text.clone(), ty)
                })
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
            self.self_struct = None;
        }
        // Attach `impl` blocks for non-generic structs.
        for id in 0..num_structs {
            self.attach_impls_for_struct(id);
        }
        self.attach_impls_for_builtins();
        let num_sigs = self.sigs.len();
        for id in 0..num_sigs {
            self.ensure_sig(id, None);
        }
        self.cur_mod = 0;
        self.offset = 0;
    }

    pub(crate) fn resolve_fn_sig(&mut self, id: FnId, trait_name: Option<&str>) {
        self.resolve_fn_sig_with(id, trait_name, &[], None);
    }

    /// A struct's own method, its signature read now if it was left for
    /// first use.
    pub(crate) fn own_method(&mut self, sid: StructId, name: &str) -> Option<FnId> {
        let id = *self.structs[sid].methods.get(name)?;
        if self.quiet_sigs.contains(&id) {
            self.ensure_sig(id, None);
        }
        Some(id)
    }

    /// Reads `id`'s signature the first time anything needs it: a call made
    /// while other signatures are still being read (a domain sized from a
    /// parameter type) gets the callee's types, not an empty list.
    /// `false` when it is already being read, and the caller should give
    /// up: a `call` closes a definition cycle and reports it (E0209); a
    /// probe (`None`) leaves the report to the call.
    pub(crate) fn ensure_sig(&mut self, id: FnId, call: Option<Span>) -> bool {
        match self.sigs[id].sig {
            SigState::Resolved => true,
            SigState::Resolving => {
                if let Some(call) = call {
                    self.definition_cycle(id, call);
                }
                false
            }
            SigState::Unresolved if self.quiet_sigs.remove(&id) => {
                // An instance's method whose types don't work out for these
                // arguments has no signature; the checker reports the template.
                let dlen = self.diags.len();
                self.resolve_fn_sig(id, None);
                if self.diags.len() > dlen {
                    self.diags.truncate(dlen);
                    self.sigs[id].params.clear();
                    self.sigs[id].ret = None;
                }
                true
            }
            SigState::Unresolved => {
                self.resolve_fn_sig(id, None);
                true
            }
        }
    }

    pub(in crate::lower) fn definition_cycle(&mut self, id: FnId, call: Span) {
        let sig = &self.sigs[id];
        let name = sig.decl.name.text.clone();
        let at = self
            .sig_param_at
            .get(&id)
            .copied()
            .unwrap_or(sig.decl.name.span);
        let at = if sig.mod_id == 0 {
            at.shift(sig.offset)
        } else {
            self.asked_at.at
        };
        let diag = cpl_diag::msgs::check::PARAM_TYPES_CYCLE
            .error(at, &[&name, &name])
            .with_label(
            self.span(call),
            format!("this call needs `{name}`'s parameter types while they are being worked out"),
        )
        .with_help(
            "give the dependent value an explicit type, or move the call out of the definition it sizes",
        );
        self.diags.push(diag);
    }

    pub(crate) fn resolve_fn_sig_with(
        &mut self,
        id: FnId,
        trait_name: Option<&str>,
        trait_args: &[Ty],
        out_ty: Option<&Ty>,
    ) {
        let decl = self.sigs[id].decl;
        self.sigs[id].sig = SigState::Resolving;
        let saved_mod = std::mem::replace(&mut self.cur_mod, self.sigs[id].mod_id);
        let saved_offset = std::mem::replace(&mut self.offset, self.sigs[id].offset);
        let saved_env = std::mem::replace(&mut self.gen_env, self.sigs[id].env.clone());
        let saved_self = std::mem::replace(&mut self.self_struct, self.sigs[id].owner);

        let owner_ty = self.sigs[id]
            .owner
            .map(Ty::Struct)
            .or_else(|| self.prim_self.get(&id).cloned());
        let mut params = Vec::new();
        for (i, param) in decl.params.iter().enumerate() {
            if let (0, "self", Some(owner)) = (i, param.name.text.as_str(), self.sigs[id].owner) {
                // A method that grows a cache works on the caller's struct.
                let cached = self.stress != call::Stress::ColdCache
                    && self.structs[owner]
                        .decl
                        .is_some_and(|sd| cache::grows_cache(sd, decl));
                self.sigs[id].receiver = Some(param.mutable || cached);
                continue;
            }
            let ty = if let Some(GenArg::Ty(t)) = self.gen_env.get(&format!("__trait_param_{i}")) {
                t.clone()
            } else {
                match &param.ty {
                    Some(ty) => {
                        self.sig_param_at.insert(id, ty.span);
                        self.resolve_ty(ty)
                    }
                    None => {
                        let declared = self.trait_declared_ty(
                            trait_name,
                            &decl.name.text,
                            Some(i),
                            trait_args,
                            owner_ty.as_ref(),
                        );
                        match declared.or_else(|| owner_ty.clone()) {
                            Some(ty) => ty,
                            None => {
                                self.report(
                                    param.span,
                                    msgs::ir::UNKNOWN_TYPE_PARAMETER_NEEDS_A_TYPE,
                                    &[&param.name.text],
                                );
                                self.help(format!(
                                    "write `{}: Int`, or the type it has",
                                    param.name.text
                                ));
                                Ty::Error
                            }
                        }
                    }
                }
            };
            params.push(SigParam {
                label: param.label_text().to_string(),
                name: param.name.text.clone(),
                ty,
                mutable: param.mutable,
                default: param.default.as_ref(),
            });
        }
        self.sigs[id].params = params;
        // An unwritten result is the impl's `type Out`, else what its trait declares.
        let declared_ret = match (&decl.ret, out_ty) {
            (Some(_), _) => None,
            (None, Some(o)) => Some(o.clone()),
            (None, None) => self.trait_declared_ty(
                trait_name,
                &decl.name.text,
                None,
                trait_args,
                owner_ty.as_ref(),
            ),
        };
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
            // `-> Rooted<V, _>`: the body's value says what `_` is, a fresh domain made there.
            (Some(ty), _) if type_has_hole(ty) => None,
            (Some(ty), _) => {
                // Check for `-> (domain C, comp: [V]C)` or `-> (domain P, [P]Int)` returning fresh domain(s).
                let mut binders: Vec<String> = Vec::new();
                match &ty.kind {
                    ast::TypeKind::Record(fields) => {
                        for (dname, ft) in fields {
                            if matches!(ft.kind, ast::TypeKind::NewDomain) {
                                binders.push(dname.text.clone());
                            }
                        }
                    }
                    ast::TypeKind::Tuple(items) => {
                        for item in items {
                            if let ast::TypeKind::DomainBinder(dname) = &item.kind {
                                binders.push(dname.text.clone());
                            }
                        }
                    }
                    ast::TypeKind::DomainBinder(dname) => {
                        binders.push(dname.text.clone());
                    }
                    _ => {}
                }
                for dname in binders {
                    let dom_id = self.domains.len();
                    self.domains
                        .push(format!("{}_{}", self.sigs[id].name, dname));
                    self.gen_env.insert(dname.clone(), GenArg::Domain(dom_id));
                    self.sigs[id]
                        .env
                        .insert(dname.clone(), GenArg::Domain(dom_id));
                    self.sigs[id].new_domain_slots.push((dname, dom_id));
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
                _ => Some(declared_ret.unwrap_or(Ty::Unit)),
            },
            (None, _) => declared_ret,
        };

        self.cur_mod = saved_mod;
        self.offset = saved_offset;
        self.gen_env = saved_env;
        self.self_struct = saved_self;
        self.sigs[id].sig = SigState::Resolved;
    }
}

impl<'a> Lowerer<'a> {
    /// The type a trait declares for its fn `method`'s parameter `at`, or
    /// its result when `at` is `None`, with `Self` the impl's target and the
    /// trait's arguments put in, a missing one being `Self` (R08.1.5). A user
    /// trait's declaration and a core trait's in `core_traits.cp` are read
    /// the same way, as the checker reads them.
    pub(crate) fn trait_declared_ty(
        &mut self,
        trait_name: Option<&str>,
        method: &str,
        at: Option<usize>,
        trait_args: &[Ty],
        owner: Option<&Ty>,
    ) -> Option<Ty> {
        let (trait_name, owner) = (trait_name?, owner?);
        let mut env = self.gen_env.clone();
        env.insert("Self".into(), GenArg::Ty(owner.clone()));
        let mut bind = |k: usize, g: &str| {
            env.insert(
                g.into(),
                GenArg::Ty(trait_args.get(k).unwrap_or(owner).clone()),
            );
        };
        let (f, m, o): (&'a ast::FnDecl, _, _) =
            match self.traits.iter().find(|t| t.name == trait_name) {
                Some(t) => {
                    t.generics.iter().enumerate().for_each(|(k, g)| bind(k, g));
                    let f = t.methods.iter().find(|f| f.name.text == method)?;
                    (f, t.mod_id, t.offset)
                }
                None => {
                    let (generics, f) = cpl_check::core_trait_fn(trait_name, method)?;
                    generics
                        .iter()
                        .enumerate()
                        .for_each(|(k, g)| bind(k, &g.text));
                    (f, self.cur_mod, self.offset)
                }
            };
        let ty = match at {
            Some(i) => f.params.get(i)?.ty.as_ref(),
            None => f.ret.as_ref(),
        }?;
        Some(self.quietly_in(m, o, env, |lw| lw.resolve_ty(ty)))
    }
}

/// Where an impl's associated type comes from: its own `type X = ..`, or its trait's default.
#[derive(Clone, Copy)]
enum AssocSrc<'a> {
    Impl(&'a ast::Type),
    Default(usize, &'a ast::Type),
}

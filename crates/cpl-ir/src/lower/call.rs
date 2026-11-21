//! Calls, methods and members: the user's fns and the ones the language
//! provides.

use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Reduce {
    Sum,
    Prod,
    Count,
    All,
    Any,
    Min,
    Max,
}

impl<'a> Lowerer<'a> {
    /// The symbol `e` names, if it is a bare name no local hides.
    pub(super) fn sym(&self, e: &ast::Expr) -> Option<Sym> {
        match &e.kind {
            ast::ExprKind::Name(name) if self.lookup(name).is_none() => {
                if let Some(ga) = self.gen_env.get(name) {
                    return match ga {
                        GenArg::Domain(d) => Some(Sym::Domain(*d)),
                        GenArg::Ty(Ty::Struct(id)) => Some(Sym::Struct(*id)),
                        GenArg::Ty(Ty::Enum(id)) => Some(Sym::Enum(*id)),
                        _ => None,
                    };
                }
                self.get_sym(name)
            }
            _ => None,
        }
    }

    fn call_fn_val(&mut self, callee: Expr, args: &[ast::Arg], span: Span) -> Expr {
        let Ty::Fn(param_tys, ret_ty) = callee.ty.clone() else {
            if callee.ty != Ty::Error {
                self.error(
                    codes::NOT_CALLABLE,
                    span,
                    format!("a `{}` can't be called", self.show(&callee.ty)),
                );
            }
            return self.err_expr(span);
        };
        let mut vals = Vec::with_capacity(args.len());
        for (i, arg) in args.iter().enumerate() {
            let v = match (param_tys.get(i), arg.mutable) {
                // The fn changes the caller's variable, as a top-level
                // fn with a `mut` parameter does.
                (Some(Ty::Mut(want)), true) => {
                    let place = self.place(&arg.value);
                    if let PlaceKind::Local(slot) = place.kind {
                        self.cx.mut_slots.insert(slot);
                    }
                    if place.ty == Ty::Bool && matches!(place.kind, PlaceKind::Index(..)) {
                        self.unsupported(arg.value.span, "a `Bool` element passed as `mut`");
                    } else if place.ty != **want && place.ty != Ty::Error && **want != Ty::Error {
                        self.mismatch(arg.value.span, want, &place.ty);
                    }
                    Arg::Mut(place)
                }
                (Some(Ty::Mut(_)), false) => {
                    let message = "this parameter is `mut`: the fn changes what it is given".to_string();
                    self.error(codes::ASSIGN_TO_LET, arg.value.span, message);
                    self.help("write `mut` before it, as for a top-level fn");
                    Arg::Value(self.err_expr(arg.value.span))
                }
                (Some(_), true) => {
                    let message = "this parameter is not `mut`: the fn takes a copy".to_string();
                    self.error(codes::ASSIGN_TO_LET, arg.value.span, message);
                    self.help("drop the `mut`, or declare the parameter `mut`");
                    Arg::Value(self.err_expr(arg.value.span))
                }
                (Some(want), false) => Arg::Value(self.expect(&arg.value, want)),
                (None, _) => Arg::Value(self.expr(&arg.value, None)),
            };
            vals.push(v);
        }
        self.mk(ExprKind::CallExpr(Box::new(callee), vals), *ret_ty, span)
    }

    pub(super) fn call(
        &mut self,
        whole: &ast::Expr,
        callee: &ast::Expr,
        args: &[ast::Arg],
        hint: Option<&Ty>,
    ) -> Expr {
        let span = whole.span;
        match &callee.kind {
            ast::ExprKind::Paren(inner) => self.call(whole, inner, args, hint),
            ast::ExprKind::Name(name) => {
                // Local variable, receiver field, or captured fn value?
                if self.lookup(name).is_some() {
                    let fv = self.name(name, None, callee.span);
                    return self.call_fn_val(fv, args, span);
                }
                if let Some((slot, owner)) = self.cx.receiver {
                    if let Some((index, ty)) = self.field_of(&Ty::Struct(owner), name) {
                        if matches!(ty, Ty::Fn(..)) {
                            let base = self.local(slot, callee.span);
                            let fv = self.mk(ExprKind::Field(Box::new(base), index), ty, callee.span);
                            return self.call_fn_val(fv, args, span);
                        }
                    }
                }
                if let Some(slot) = self.capture_from_outer(name) {
                    let fv = self.local(slot, callee.span);
                    return self.call_fn_val(fv, args, span);
                }
                if let Some((slot, owner)) = self.capture_self_from_outer() {
                    if let Some((index, ty)) = self.field_of(&Ty::Struct(owner), name) {
                        if matches!(ty, Ty::Fn(..)) {
                            let base = self.local(slot, callee.span);
                            let fv = self.mk(ExprKind::Field(Box::new(base), index), ty, callee.span);
                            return self.call_fn_val(fv, args, span);
                        }
                    }
                }
                if let Some(ga) = self.gen_env.get(name).cloned() {
                    match ga {
                        GenArg::Domain(id) => {
                            let Some(value) = self.one_arg(name, args, &Ty::Int, span) else {
                                return self.err_expr(span);
                            };
                            return self.mk(
                                ExprKind::Cast(Cast::FromLabel(id), Box::new(value)),
                                Ty::Pos(id),
                                span,
                            );
                        }
                        GenArg::Ty(Ty::Struct(id)) => {
                            return self.construct_or_new(id, args, span);
                        }
                        GenArg::Ty(Ty::Int) => return self.prelude("Int", args, hint, span),
                        GenArg::Ty(Ty::Float) => return self.prelude("F64", args, hint, span),
                        GenArg::Ty(Ty::Char) => return self.prelude("Char", args, hint, span),
                        GenArg::Ty(Ty::Str) => return self.prelude("Str", args, hint, span),
                        GenArg::Ty(Ty::Bool) => return self.prelude("Bool", args, hint, span),
                        _ => {}
                    }
                }
                if name == "same" && args.len() == 2 {
                    let op_a = self.detect_bare_op(&args[0].value);
                    let op_b = self.detect_bare_op(&args[1].value);
                    if op_a.is_some() || op_b.is_some() {
                        return self.mk(ExprKind::Bool(op_a.is_some() && op_a == op_b), Ty::Bool, span);
                    }
                }
                match self.get_sym(name) {
                    Some(Sym::Fn(id)) => self.user_call(id, None, args, span),
                    Some(Sym::GenFn(gfid)) => {
                        self.call_hint = hint.cloned();
                        self.call_gen_fn(gfid, Vec::new(), None, args, span)
                    }
                    Some(Sym::Struct(id)) => self.construct_or_new(id, args, span),
                    Some(Sym::GenStruct(gid)) => self.call_gen_struct(gid, Vec::new(), args, hint, span),
                    Some(Sym::Variant(id, tag)) => self.variant(id, tag, args, span),
                    Some(Sym::Enum(eid))
                        if !self.enums[eid].variants.is_empty()
                            && self.enums[eid].variants.iter().all(|(_, fs)| fs.is_empty()) =>
                    {
                        let Some(exprs) = self.positional_any(name, args, 1, span) else {
                            return self.err_expr(span);
                        };
                        let v = self.expr(exprs[0], None);
                        let iv = self.coerce(v, &Ty::Int, exprs[0].span);
                        let loc = self.temp(Ty::Int);
                        let init = Stmt::Let(loc, Some(iv));
                        let discs = self.enum_discriminants.get(eid).cloned().unwrap_or_default();
                        let n = self.enums[eid].variants.len();
                        let mut acc = self.mk(
                            ExprKind::Variant(n - 1, Vec::new()),
                            Ty::Enum(eid),
                            span,
                        );
                        for tag in (0..n.saturating_sub(1)).rev() {
                            let disc = discs.get(tag).copied().unwrap_or(tag as i64);
                            let cond = self.mk(
                                ExprKind::Cmp(
                                    CmpOp::Eq,
                                    Box::new(self.mk(ExprKind::Local(loc), Ty::Int, span)),
                                    Box::new(self.int_lit(disc, span)),
                                ),
                                Ty::Bool,
                                span,
                            );
                            let then = self.mk(
                                ExprKind::Variant(tag, Vec::new()),
                                Ty::Enum(eid),
                                span,
                            );
                            acc = self.mk(
                                ExprKind::If(Box::new(cond), Box::new(then), Box::new(acc)),
                                Ty::Enum(eid),
                                span,
                            );
                        }
                        self.mk(
                            ExprKind::Do(Block {
                                stmts: vec![init, Stmt::Give(acc)],
                            }),
                            Ty::Enum(eid),
                            span,
                        )
                    }
                    Some(Sym::Domain(id)) => {
                        if let Some(factors) = self.product_domains.get(&id).cloned() {
                            if args.len() == factors.len() {
                                let mut parts = Vec::with_capacity(factors.len());
                                let named = args.iter().any(|a| a.name.is_some());
                                if named {
                                    let mut slots: Vec<Option<Expr>> = vec![None; factors.len()];
                                    for (i, arg) in args.iter().enumerate() {
                                        let idx = arg
                                            .name
                                            .as_ref()
                                            .and_then(|n| {
                                                factors.iter().position(|(fnm, _)| {
                                                    fnm.as_deref() == Some(n.text.as_str())
                                                })
                                            })
                                            .unwrap_or(i);
                                        let want = self.product_factor_ty(&factors[idx].1);
                                        slots[idx] = Some(self.expect(&arg.value, &want));
                                    }
                                    if slots.iter().all(|s| s.is_some()) {
                                        for s in slots {
                                            parts.push(s.unwrap());
                                        }
                                        return self.encode_product(id, parts, span);
                                    }
                                } else {
                                    for ((_, f), arg) in factors.iter().zip(args) {
                                        let want = self.product_factor_ty(f);
                                        parts.push(self.expect(&arg.value, &want));
                                    }
                                    return self.encode_product(id, parts, span);
                                }
                            }
                            if args.len() == 1 && args[0].name.is_none() {
                                let v = self.expr(&args[0].value, None);
                                if matches!(v.ty, Ty::Tuple(_) | Ty::Record(_)) {
                                    return self.coerce(v, &Ty::Pos(id), span);
                                }
                                let value = if v.ty == Ty::Char {
                                    self.mk(
                                        ExprKind::Cast(Cast::CharToInt, Box::new(v)),
                                        Ty::Int,
                                        args[0].value.span,
                                    )
                                } else {
                                    self.coerce(v, &Ty::Int, args[0].value.span)
                                };
                                return self.mk(
                                    ExprKind::Cast(Cast::FromLabel(id), Box::new(value)),
                                    Ty::Pos(id),
                                    span,
                                );
                            }
                        }
                        let Some(exprs) = self.positional_any(name, args, 1, span) else {
                            return self.err_expr(span);
                        };
                        let v = self.expr(exprs[0], None);
                        let value = if v.ty == Ty::Char {
                            self.mk(ExprKind::Cast(Cast::CharToInt, Box::new(v)), Ty::Int, exprs[0].span)
                        } else {
                            self.coerce(v, &Ty::Int, exprs[0].span)
                        };
                        self.mk(
                            ExprKind::Cast(Cast::FromLabel(id), Box::new(value)),
                            Ty::Pos(id),
                            span,
                        )
                    }
                    Some(Sym::Alias(aid)) => {
                        let (m, ty_ast, offset) = self.aliases[aid];
                        let sm = std::mem::replace(&mut self.cur_mod, m);
                        let so = std::mem::replace(&mut self.offset, offset);
                        let resolved = self.resolve_ty(ty_ast);
                        self.cur_mod = sm;
                        self.offset = so;
                        match resolved {
                            Ty::Struct(id) => self.construct_or_new(id, args, span),
                            Ty::Pos(id) => {
                                let Some(value) = self.one_arg(name, args, &Ty::Int, span) else {
                                    return self.err_expr(span);
                                };
                                self.mk(
                                    ExprKind::Cast(Cast::FromLabel(id), Box::new(value)),
                                    Ty::Pos(id),
                                    span,
                                )
                            }
                            _ => {
                                self.error(
                                    codes::NOT_CALLABLE,
                                    callee.span,
                                    format!("`{name}` is a type and can't be called"),
                                );
                                self.err_expr(span)
                            }
                        }
                    }
                    Some(Sym::Global(id)) => {
                        let fv = self.mk(ExprKind::Global(id), self.globals[id].ty.clone(), callee.span);
                        self.call_fn_val(fv, args, span)
                    }
                    Some(Sym::ModConst(mcid)) => {
                        let fv = self.ensure_mod_const(mcid, callee.span);
                        self.call_fn_val(fv, args, span)
                    }
                    Some(Sym::Enum(_) | Sym::GenEnum(_) | Sym::GenAlias(_) | Sym::Trait(_) | Sym::Mod(_) | Sym::GenConst(_)) => {
                        self.error(
                            codes::NOT_CALLABLE,
                            callee.span,
                            format!("`{name}` is a type and can't be called"),
                        );
                        self.err_expr(span)
                    }
                    None => self.prelude(name, args, hint, span),
                }
            }
            ast::ExprKind::Generic { base, args: types } => {
                self.call_generic(base, types, args, hint, span)
            }
            ast::ExprKind::Field { base, name } => self.method(whole, base, name, args, hint),
            _ => {
                let fv = self.expr(callee, None);
                self.call_fn_val(fv, args, span)
            }
        }
    }

    fn call_generic(
        &mut self,
        base: &ast::Expr,
        types: &[ast::Type],
        args: &[ast::Arg],
        hint: Option<&Ty>,
        span: Span,
    ) -> Expr {
        // Check `base.method::<T>(...)`.
        if let ast::ExprKind::Field { base: recv, name } = &base.kind {
            if let Some(Sym::Mod(target_mod)) = self.sym(recv) {
                let saved_mod = std::mem::replace(&mut self.cur_mod, target_mod);
                let sym = self.mod_syms[target_mod].get(&name.text).copied();
                self.cur_mod = saved_mod;
                match sym {
                    Some(Sym::GenFn(gfid)) => {
                        let gargs = self.resolve_gen_args(types);
                        return self.call_gen_fn(gfid, gargs, None, args, span);
                    }
                    Some(Sym::GenStruct(gid)) => {
                        let gargs = self.resolve_gen_args(types);
                        return self.call_gen_struct(gid, gargs, args, hint, span);
                    }
                    _ => {}
                }
            }
            let val = self.expr(recv, None);
            if let Ty::Struct(sid) = val.ty {
                if let Some(&gfid) = self.structs[sid].gen_methods.get(&name.text) {
                    let gargs = self.resolve_gen_args(types);
                    return self.call_gen_fn(gfid, gargs, Some(Arg::Value(val)), args, span);
                }
            }
        }

        let name = match &base.kind {
            ast::ExprKind::Name(name) => name.as_str(),
            _ => "",
        };
        // `read::<Int>()`, `read<1..=3>()`, `read<[n]1..=3>()`: a read that says its own type.
        if name == "read" && self.get_sym(name).is_none() {
            if let [ty] = types {
                if args.is_empty() {
                    let spec = self.resolve(ty, true);
                    return self.read(spec, span);
                }
                let want = self.resolve_ty(ty);
                return self.prelude(name, args, Some(&want), span);
            }
        }
        if matches!(name, "List" | "Set" | "HashSet" | "Map" | "HashMap") {
            let ty = ast::Type {
                kind: ast::TypeKind::Named {
                    name: ast::Name {
                        text: name.to_string(),
                        span: base.span,
                    },
                    args: types.to_vec(),
                },
                span: base.span,
            };
            let ty = self.resolve_ty(&ty);
            if args.is_empty() {
                return self.mk(ExprKind::Tuple(Vec::new()), ty, span);
            }
            if args.len() == 1 && matches!(name, "List" | "Set" | "HashSet") {
                let seq = self.expr(&args[0].value, None);
                if seq.ty == ty {
                    return seq;
                }
                if let Ok(c) = self.try_coerce(seq.clone(), &ty) {
                    return c;
                }
                // Copy elements into a fresh List/Set.
                let is_set = matches!(ty, Ty::Set(_) | Ty::HashSet(_));
                return self.do_expr(Some(&ty), span, |lw, out| {
                    let acc = lw.temp(ty.clone());
                    out.push(Stmt::Let(acc, Some(lw.mk(ExprKind::Tuple(Vec::new()), ty.clone(), span))));
                    let (iter, elem) = lw.iter(&args[0].value);
                    let lid = lw.open_loop(None);
                    let slot = lw.temp(elem);
                    let op = if is_set { Mutation::Insert } else { Mutation::Push };
                    let rty = if is_set { Ty::Bool } else { Ty::Unit };
                    let item = lw.local(slot, span);
                    let m = lw.mk(
                        ExprKind::Mutate(
                            op,
                            Box::new(Place { kind: PlaceKind::Local(acc), ty: ty.clone(), span: lw.span(span) }),
                            vec![item],
                        ),
                        rty,
                        span,
                    );
                    lw.cx.loops.pop();
                    out.push(Stmt::For(lid, slot, iter, Block { stmts: vec![Stmt::Expr(m)] }));
                    let res = lw.local(acc, span);
                    out.push(Stmt::Give(res));
                });
            }
        }
        let lookup = prefix_shape_struct(name, types.len()).unwrap_or(name);
        match self.get_sym(lookup) {
            Some(Sym::GenStruct(gid)) => {
                let gargs = self.resolve_gen_args(types);
                self.call_gen_struct(gid, gargs, args, hint, span)
            }
            Some(Sym::GenFn(gfid)) => {
                let gargs = self.resolve_gen_args(types);
                self.call_gen_fn(gfid, gargs, None, args, span)
            }
            Some(Sym::GenAlias(gaid)) => {
                let gargs = self.resolve_gen_args(types);
                let tpl_mod = self.gen_aliases[gaid].mod_id;
                let tpl_offset = self.gen_aliases[gaid].offset;
                let generics = self.gen_aliases[gaid].generics;
                let target_ty = self.gen_aliases[gaid].ty;
                let mut env = HashMap::new();
                for (p, a) in generics.iter().zip(gargs) {
                    env.insert(p.text.clone(), a);
                }
                let sm = std::mem::replace(&mut self.cur_mod, tpl_mod);
                let so = std::mem::replace(&mut self.offset, tpl_offset);
                let se = std::mem::replace(&mut self.gen_env, env);
                let resolved = self.resolve_ty(target_ty);
                self.cur_mod = sm;
                self.offset = so;
                self.gen_env = se;
                if let Ty::Struct(sid) = resolved {
                    return self.construct_or_new(sid, args, span);
                }
                self.err_expr(span)
            }
            None => {
                self.error(
                    codes::UNSUPPORTED,
                    span,
                    format!("`{name}` is not in scope during lowering"),
                );
                self.err_expr(span)
            }
            Some(sym) => {
                let kind = match sym {
                    Sym::Fn(_) => "non-generic function",
                    Sym::Struct(_) => "non-generic struct",
                    Sym::Enum(_) => "non-generic enum",
                    Sym::GenEnum(_) => "generic enum",
                    Sym::Variant(_, _) => "enum variant",
                    Sym::Domain(_) => "domain",
                    Sym::Alias(_) => "non-generic type alias",
                    Sym::Global(_) | Sym::GenConst(_) | Sym::ModConst(_) => "value",
                    Sym::Trait(_) => "trait",
                    Sym::Mod(_) => "module",
                    Sym::GenFn(_) | Sym::GenStruct(_) | Sym::GenAlias(_) => unreachable!(),
                };
                self.error(
                    codes::UNSUPPORTED,
                    span,
                    format!("`{name}` is a {kind} and cannot be called with explicit type arguments"),
                );
                self.err_expr(span)
            }
        }
    }

    /// Calls a struct `S(...)`: outside `S`'s own body, routes to `S.new(...)` if `new` exists;
    /// inside `S`'s own body, constructs `S` directly from its fields.
    fn construct_or_new(&mut self, id: StructId, args: &[ast::Arg], span: Span) -> Expr {
        let inside_self = self.self_struct == Some(id)
            || self.self_struct.is_some_and(|sid| {
                matches!(
                    (&self.structs[sid].origin_gen, &self.structs[id].origin_gen),
                    (Some((a, _)), Some((b, _))) if a == b
                )
            });
        if !inside_self {
            if let Some(&new_fn) = self.structs[id].methods.get("new") {
                return self.user_call(new_fn, None, args, span);
            }
            if let Some(&new_gfn) = self.structs[id].gen_methods.get("new") {
                return self.call_gen_fn(new_gfn, Vec::new(), None, args, span);
            }
        }
        self.construct(id, args, span)
    }

    /// Instantiates a generic struct (inferring omitted type args from `self_struct`, `hint`, or call args)
    /// and constructs it via `construct_or_new`.
    fn call_gen_struct(
        &mut self,
        gid: GenStructId,
        explicit: Vec<GenArg>,
        args: &[ast::Arg],
        hint: Option<&Ty>,
        span: Span,
    ) -> Expr {
        let decl = self.gen_structs[gid].decl;
        let tpl_mod = self.gen_structs[gid].mod_id;
        let gen_names: Vec<String> = decl.generics.iter().map(|g| g.text.clone()).collect();
        let min_explicit = decl
            .generic_defaults
            .iter()
            .take_while(|d| d.is_none())
            .count();
        if !explicit.is_empty() && explicit.len() >= min_explicit && explicit.len() <= gen_names.len() {
            self.emit_anon_product_defs(&explicit, span);
            let sid = self.instantiate_struct(gid, explicit);
            return self.construct_or_new(sid, args, span);
        }
        // Inside the struct's own body (`Mod(x % P)`, `Dsu(...)`, `Order(...)`), reuse `self_struct`!
        if explicit.is_empty() {
            if let Some(sid) = self.self_struct {
                if self.structs[sid].origin_gen.as_ref().is_some_and(|(g, _)| *g == gid) {
                    return self.construct_or_new(sid, args, span);
                }
            }
        }
        // Try inferring from `hint` (e.g. `Monoid(m.id, ...)` when return type is `Monoid<T>`).
        if let Some(Ty::Struct(hsid)) = hint {
            if let Some((hgid, _)) = &self.structs[*hsid].origin_gen {
                if *hgid == gid {
                    return self.construct_or_new(*hsid, args, span);
                }
            }
        }
        // Otherwise infer generic params from `fn new(...)` (if present) or constructor fields (`Lit(Road(a), true)`).
        let mut inferred: GenEnv = HashMap::new();
        for (n, a) in gen_names.iter().zip(&explicit) {
            inferred.insert(n.clone(), a.clone());
        }
        let new_params: Option<&[ast::Param]> = decl.methods.iter().find_map(|s| match &s.kind {
            ast::StmtKind::Fn(f) if f.name.text == "new" => Some(f.params.as_slice()),
            _ => None,
        });
        if let Some(params) = new_params {
            let mut pos_idx = 0usize;
            for arg in args {
                let param = if let Some(aname) = &arg.name {
                    params.iter().find(|p| p.name.text == aname.text)
                } else {
                    let p = params.get(pos_idx);
                    pos_idx += 1;
                    p
                };
                if let Some(p) = param {
                    if let Some(pat_ty) = &p.ty {
                        self.bind_range_domain(pat_ty, &arg.value, &gen_names, &mut inferred, span, None);
                    }
                    if !matches!(arg.value.kind, ast::ExprKind::Lambda { .. }) {
                        if let Some(pty) = self.probe_ty_silent(&arg.value) {
                            if let Some(pat_ty) = &p.ty {
                                self.unify_ast_ty(pat_ty, &pty, tpl_mod, &gen_names, &mut inferred);
                            }
                        }
                    }
                }
            }
        }
        for (f, arg) in decl.fields.iter().zip(args) {
            if new_params.is_none() {
                self.bind_range_domain(&f.ty, &arg.value, &gen_names, &mut inferred, span, None);
            }
            if !matches!(arg.value.kind, ast::ExprKind::Lambda { .. }) {
                if let Some(pty) = self.probe_ty_silent(&arg.value) {
                    self.unify_ast_ty(&f.ty, &pty, tpl_mod, &gen_names, &mut inferred);
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
        self.emit_anon_product_defs(&final_args, span);
        let sid = self.instantiate_struct(gid, final_args);
        self.construct_or_new(sid, args, span)
    }

    fn emit_anon_product_defs(&mut self, gargs: &[GenArg], span: Span) {
        let mut defs = Vec::new();
        for ga in gargs {
            if let GenArg::Domain(d) = ga {
                if self.anon_product_domain_set.contains(d) {
                    self.def_anon_product_domain(*d, span, &mut defs);
                }
            }
        }
        self.cx.pending.extend(defs);
    }

    /// Positional arguments only, and exactly `n` of them.
    fn positional_any<'b>(
        &mut self,
        what: &str,
        args: &'b [ast::Arg],
        n: usize,
        span: Span,
    ) -> Option<Vec<&'b ast::Expr>> {
        if let Some(named) = args.iter().find_map(|a| a.name.as_ref()) {
            let message = format!("`{what}` has no argument named `{}`", named.text);
            self.error(codes::UNKNOWN_ARG_NAME, named.span, message);
            return None;
        }
        if args.len() != n {
            let message = format!(
                "`{what}` takes {n} argument{}, but {} {} given",
                if n == 1 { "" } else { "s" },
                args.len(),
                if args.len() == 1 { "was" } else { "were" }
            );
            self.error(codes::WRONG_ARG_COUNT, span, message);
            return None;
        }
        Some(args.iter().map(|a| &a.value).collect())
    }

    fn one_arg(&mut self, what: &str, args: &[ast::Arg], want: &Ty, span: Span) -> Option<Expr> {
        let exprs = self.positional_any(what, args, 1, span)?;
        Some(self.expect(exprs[0], want))
    }

    /// Calls the user's fn `id`. Arguments are matched by position, then
    /// by name, then from defaults.
    pub(super) fn user_call(
        &mut self,
        mut id: FnId,
        receiver: Option<Arg>,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        if self.sigs[id].returns_fresh_domain && self.sigs[id].used_fresh_call {
            id = self.clone_fresh_domain_fn(id);
        }
        self.queue_fn(id);
        let thunked: Vec<ast::Arg>;
        let decl_params: Vec<&ast::Param> =
            self.sigs[id].decl.params.iter().filter(|p| p.name.text != "self").collect();
        let args = match super::mono::thunk_lazy_args(&decl_params, args) {
            Some(t) => {
                thunked = t;
                thunked.as_slice()
            }
            None => args,
        };
        let params = self.sigs[id].params.clone();
        let name = self.sigs[id].decl.name.text.clone();
        let mut given: Vec<Option<&ast::Arg>> = vec![None; params.len()];
        let mut next = 0;
        let mut ok = true;
        for arg in args {
            let index = match &arg.name {
                Some(arg_name) => match params.iter().position(|p| p.name == arg_name.text) {
                    Some(index) => index,
                    None => {
                        let message =
                            format!("`{name}` has no parameter named `{}`", arg_name.text);
                        self.error(codes::UNKNOWN_ARG_NAME, arg_name.span, message);
                        ok = false;
                        continue;
                    }
                },
                None => {
                    while next < params.len() && given[next].is_some() {
                        next += 1;
                    }
                    next
                }
            };
            if index >= params.len() {
                let message = format!(
                    "`{name}` takes {} argument{}, but {} were given",
                    params.len(),
                    if params.len() == 1 { "" } else { "s" },
                    args.len()
                );
                self.error(codes::WRONG_ARG_COUNT, span, message);
                ok = false;
                break;
            }
            given[index] = Some(arg);
        }
        let mut lowered: Vec<Arg> = receiver.into_iter().collect();
        let mut pre_stmts: Vec<Stmt> = Vec::new();
        let mut post_stmts: Vec<Stmt> = Vec::new();
        // A domain the fn shares between its calls stands for this call's
        // array until it returns; then it is what it was, for a caller
        // further out that is the same fn.
        let mut put_back: Vec<Stmt> = Vec::new();
        for param in &params {
            let Ty::Array { dims, .. } = &param.ty else {
                continue;
            };
            let [Dim::Pos(d)] = dims.as_slice() else {
                continue;
            };
            let d = *d;
            let known = put_back.iter().any(|s| matches!(s, Stmt::DefDomain { id, .. } if *id == d));
            if !self.range_domains_shared.contains(&d) || known {
                continue;
            }
            let (lo, len) = (self.temp(Ty::Int), self.temp(Ty::Int));
            pre_stmts.push(Stmt::Let(lo, Some(self.mk(ExprKind::DomainLo(d), Ty::Int, span))));
            pre_stmts.push(Stmt::Let(len, Some(self.mk(ExprKind::DomainLen(d), Ty::Int, span))));
            put_back.push(Stmt::DefDomain {
                id: d,
                lo: self.local(lo, span),
                len: self.local(len, span),
            });
        }
        // The domains a `mut` array sizes, known before the loop: its
        // size is set ahead of every argument.
        let mut sized = HashSet::new();
        for (param, arg) in params.iter().zip(&given) {
            let Some(arg) = arg.filter(|_| param.mutable) else {
                continue;
            };
            if let Some((d, _)) = self
                .probe_ty_silent(&arg.value)
                .and_then(|ty| self.range_domain_of(&param.ty, &ty))
            {
                sized.insert(d);
            }
        }
        let outer_sized = self.call_sized.replace(sized);
        let mut mut_sized = HashSet::new();
        for (at, (param, arg)) in params.iter().zip(given).enumerate() {
            match arg {
                Some(arg) if param.mutable => {
                    if !arg.mutable {
                        let message =
                            format!("`{name}` changes `{}`, so pass it with `mut`", param.name);
                        self.error(codes::MISSING_MUT_AT_CALL, arg.value.span, message);
                        ok = false;
                    }
                    let place = if let ast::ExprKind::Call { callee, args: pargs } = &arg.value.kind {
                        if let ast::ExprKind::Field { base, name: pname } = &callee.kind {
                            if let Some(proj) = self.find_projection_on(base, &pname.text) {
                                self.lower_projection_mut_call(
                                    proj,
                                    base,
                                    pargs,
                                    arg.value.span,
                                    &mut pre_stmts,
                                    &mut post_stmts,
                                )
                            } else {
                                self.place(&arg.value)
                            }
                        } else {
                            self.place(&arg.value)
                        }
                    } else {
                        self.place(&arg.value)
                    };
                    if let PlaceKind::Local(s) = match &place.kind {
                        PlaceKind::Local(s) => PlaceKind::Local(*s),
                        _ => PlaceKind::Global(usize::MAX),
                    } {
                        self.cx.mut_slots.insert(s);
                    }
                    if place.ty == Ty::Bool && matches!(place.kind, PlaceKind::Index(..)) {
                        self.unsupported(arg.value.span, "a `Bool` element passed as `mut`");
                        ok = false;
                    }
                    let mut place = place;
                    if let Some((d, lo)) = self.range_domain_of(&param.ty, &place.ty) {
                        let len = self.mk(
                            ExprKind::Builtin(Builtin::Len, vec![super::stmt::place_to_expr(&place)]),
                            Ty::Int,
                            arg.value.span,
                        );
                        // Before any argument is worked out, and so before
                        // those given by value: they come second.
                        let size = match mut_sized.insert(d) {
                            true => Stmt::DefDomain {
                                id: d,
                                lo: self.int_lit(lo, arg.value.span),
                                len,
                            },
                            false => self.size_domain(d, lo, len, arg.value.span),
                        };
                        pre_stmts.push(size);
                        place.ty = param.ty.clone();
                    }
                    if self.list_given(&param.ty).is_some_and(|list| list == place.ty) {
                        let message = format!(
                            "`{name}` changes `{}` in place, and a list is given to a `[D]T` as a copy; pass an array such as a `[0..n]T`",
                            param.name
                        );
                        self.error(codes::TYPE_MISMATCH, arg.value.span, message);
                        ok = false;
                    } else if place.ty != param.ty && place.ty != Ty::Error && param.ty != Ty::Error {
                        self.mismatch(arg.value.span, &param.ty, &place.ty);
                        ok = false;
                    }
                    lowered.push(Arg::Mut(place));
                }
                Some(arg) => {
                    if param.ty == Ty::Unit && self.detect_bare_op(&arg.value).is_some() {
                        lowered.push(Arg::Value(self.mk(ExprKind::Unit, Ty::Unit, arg.value.span)));
                    } else if let Some(list) = self.list_given(&param.ty) {
                        // A list where a `[D]T` is asked for is made as
                        // the list it is; passing it sizes `D`.
                        let value = self.expect(&arg.value, &list);
                        lowered.push(Arg::Value(self.coerce(value, &param.ty, arg.value.span)));
                    } else if let Some((d, first, other)) = self.other_first(&param.ty, &arg.value)
                    {
                        let message = format!(
                            "`{name}` takes `{}` indexed by the same `{}` as an argument before it, which starts at {first}; this one starts at {other}",
                            param.name,
                            self.domain_said(d)
                        );
                        self.error(codes::TYPE_MISMATCH, arg.value.span, message);
                        ok = false;
                    } else {
                        let declared = self.sigs[id].decl.params.get(at).and_then(|p| p.ty.as_ref());
                        lowered.push(Arg::Value(self.expect_declared(&arg.value, &param.ty, declared)));
                    }
                }
                None => match param.default {
                    Some(default) if !param.mutable => {
                        let sm = std::mem::replace(&mut self.cur_mod, self.sigs[id].mod_id);
                        let saved = std::mem::replace(&mut self.offset, self.sigs[id].offset);
                        let se = std::mem::replace(&mut self.gen_env, self.sigs[id].env.clone());
                        let value = self.expect(default, &param.ty);
                        self.cur_mod = sm;
                        self.offset = saved;
                        self.gen_env = se;
                        lowered.push(Arg::Value(value));
                    }
                    _ => {
                        if ok {
                            let message = format!("`{name}` needs the argument `{}`", param.name);
                            self.error(codes::WRONG_ARG_COUNT, span, message);
                        }
                        ok = false;
                    }
                },
            }
        }
        self.call_sized = outer_sized;
        let ret = self.ret_of(id, span);
        if self.sigs[id].returns_fresh_domain {
            self.sigs[id].used_fresh_call = true;
        }
        if !ok {
            return self.err_expr(span);
        }
        post_stmts.extend(put_back);
        let call_expr = self.mk(ExprKind::Call(id, lowered), ret.clone(), span);
        let call = if pre_stmts.is_empty() && post_stmts.is_empty() {
            call_expr
        } else {
            let mut stmts = pre_stmts;
            if ret == Ty::Unit {
                stmts.push(Stmt::Expr(call_expr));
                stmts.extend(post_stmts);
                stmts.push(Stmt::Give(self.mk(ExprKind::Unit, Ty::Unit, span)));
            } else {
                let res_slot = self.temp(ret.clone());
                stmts.push(Stmt::Let(res_slot, Some(call_expr)));
                stmts.extend(post_stmts);
                stmts.push(Stmt::Give(self.local(res_slot, span)));
            }
            self.mk(ExprKind::Do(Block { stmts }), ret, span)
        };
        // What the fn gives in terms of an array's plain range is the
        // caller's plain `Int`s.
        let plain = self.plain_ty(&call.ty);
        self.try_coerce(call, &plain).unwrap_or_else(|call| call)
    }

    /// `Edge(u: a, v: b, w: 5)` or `Edge(a, b, 5)`.
    /// `P(read() for _ in fields(P))`, `P(x + y for (x, y) in fields(a, b))`:
    /// the body once per field, giving that field (decision 28.9).
    fn fields_ctor(&mut self, id: StructId, args: &[ast::Arg], span: Span) -> Option<Expr> {
        let [arg] = args else {
            return None;
        };
        let ast::ExprKind::Comprehension { body, clauses } = &arg.value.kind else {
            return None;
        };
        let [clause] = clauses.as_slice() else {
            return None;
        };
        if arg.name.is_some() {
            return None;
        }
        let values = self.reflected_values(&clause.iter)?;
        let fields = self.structs[id].def.fields.clone();
        let mut stmts = Vec::new();
        let mut temps = Vec::new();
        for a in values {
            if let ast::ExprKind::Name(n) = &a.value.kind {
                if self.lookup(n).is_none() && n.starts_with(|c: char| c.is_ascii_uppercase()) {
                    let own = match self.gen_env.get(n) {
                        Some(GenArg::Ty(Ty::Struct(sid))) => *sid == id,
                        _ => matches!(self.get_sym(n), Some(Sym::Struct(sid)) if sid == id),
                    };
                    if !own {
                        let message = format!("`fields({n})` here names the type it builds, `{}`", self.structs[id].def.name);
                        self.error(codes::TYPE_MISMATCH, a.value.span, message);
                        return Some(self.err_expr(span));
                    }
                    continue;
                }
            }
            let v = self.expr(&a.value, None);
            if v.ty != Ty::Struct(id) && v.ty != Ty::Error {
                let message = format!(
                    "`fields` here walks values of the type it builds, `{}`; this is a `{}`",
                    self.structs[id].def.name,
                    self.show(&v.ty)
                );
                self.error(codes::TYPE_MISMATCH, a.value.span, message);
                return Some(self.err_expr(span));
            }
            let slot = self.temp(v.ty.clone());
            stmts.push(Stmt::Let(slot, Some(v)));
            temps.push((slot, a.value.span));
        }
        let mut items = Vec::new();
        for (i, (_, fty)) in fields.iter().enumerate() {
            self.cx.scopes.push(HashMap::new());
            self.cx.local_syms.push(HashMap::new());
            if !temps.is_empty() {
                let mut vals: Vec<Expr> = temps
                    .iter()
                    .map(|(slot, sp)| {
                        let base = self.local(*slot, *sp);
                        self.mk(ExprKind::Field(Box::new(base), i), fty.clone(), *sp)
                    })
                    .collect();
                let value = if vals.len() == 1 {
                    vals.pop().unwrap()
                } else {
                    let ty = Ty::Tuple(vec![fty.clone(); vals.len()]);
                    self.mk(ExprKind::Tuple(vals), ty, temps[0].1)
                };
                self.bind(&clause.pat, value, &mut stmts);
            }
            let v = self.expect(body, fty);
            let v = match self.struct_field_range(id, i).and_then(|r| r.scalar().cloned()) {
                Some(b) => self.check_range(v, &b, body.span),
                None => v,
            };
            let slot = self.temp(fty.clone());
            stmts.push(Stmt::Let(slot, Some(v)));
            items.push(self.local(slot, body.span));
            self.cx.local_syms.pop();
            self.cx.scopes.pop();
        }
        let value = self.mk(ExprKind::Tuple(items), Ty::Struct(id), span);
        stmts.push(Stmt::Give(value));
        Some(self.mk(ExprKind::Do(Block { stmts }), Ty::Struct(id), span))
    }

    fn construct(&mut self, id: StructId, args: &[ast::Arg], span: Span) -> Expr {
        if let Some(e) = self.fields_ctor(id, args, span) {
            return e;
        }
        let fields = self.structs[id].def.fields.clone();
        let Some(decl) = self.structs[id].decl else {
            self.error(
                codes::NOT_CALLABLE,
                span,
                format!("`{}` cannot be constructed directly", self.structs[id].def.name),
            );
            return self.err_expr(span);
        };
        let name = decl.name.text.clone();
        let mut given: Vec<Option<&ast::Arg>> = vec![None; fields.len()];
        for (i, arg) in args.iter().enumerate() {
            let index = match &arg.name {
                Some(arg_name) => fields.iter().position(|(f, _)| *f == arg_name.text),
                None => (i < fields.len()).then_some(i),
            };
            match index {
                Some(index) => given[index] = Some(arg),
                None => {
                    let message = match &arg.name {
                        Some(arg_name) => format!("`{name}` has no field `{}`", arg_name.text),
                        None => format!(
                            "`{name}` has {} fields, but {} values were given",
                            fields.len(),
                            args.len()
                        ),
                    };
                    self.error(codes::WRONG_ARG_COUNT, arg.value.span, message);
                    return self.err_expr(span);
                }
            }
        }
        let mut values = Vec::new();
        for (i, ((field, ty), arg)) in fields.iter().zip(given).enumerate() {
            match (arg, &decl.fields[i].default) {
                (Some(arg), _) => {
                    let v = self.expect_declared(&arg.value, ty, Some(&decl.fields[i].ty));
                    let v = match self.struct_field_range(id, i).and_then(|r| r.scalar().cloned()) {
                        Some(b) => self.check_range(v, &b, arg.value.span),
                        None => v,
                    };
                    values.push(v);
                }
                (None, Some(default)) => {
                    let saved = std::mem::replace(&mut self.offset, self.structs[id].offset);
                    let v = if matches!(ty, Ty::Array { .. }) {
                        self.let_value(Some(&decl.fields[i].ty), Some(default), default.span)
                            .unwrap_or_else(|| self.err_expr(default.span))
                    } else {
                        self.expect(default, ty)
                    };
                    let v = match self.struct_field_range(id, i).and_then(|r| r.scalar().cloned()) {
                        Some(b) => self.check_range(v, &b, default.span),
                        None => v,
                    };
                    values.push(v);
                    self.offset = saved;
                }
                (None, None) => {
                    let message = format!("`{name}` needs a value for its field `{field}`");
                    self.error(codes::WRONG_ARG_COUNT, span, message);
                    return self.err_expr(span);
                }
            }
        }
        self.mk(ExprKind::Tuple(values), Ty::Struct(id), span)
    }

    /// Lowers `e` for a field or parameter of type `want`, declared as
    /// `declared`. `Node([none, none], 0)`: a list written out where a
    /// one-index array is declared is the cells of that array, as in
    /// `let kid: [0..2]?Int = [none, none]`, with the same count check.
    /// An array given for a `D` that an argument before bound to arrays
    /// starting elsewhere: the domain, its first index, and this one's.
    fn other_first(&mut self, want: &Ty, arg: &ast::Expr) -> Option<(DomainId, i64, i64)> {
        let Ty::Array { dims, .. } = want else {
            return None;
        };
        let [Dim::Pos(d)] = dims.as_slice() else {
            return None;
        };
        let first = *self.range_domains.get(d)?;
        let other = match self.probe_ty_silent(arg)? {
            Ty::Array { dims, .. } => match dims.as_slice() {
                [Dim::Int] => 0,
                [Dim::IntFrom(lo)] => *lo,
                _ => return None,
            },
            _ => return None,
        };
        (other != first).then_some((*d, first, other))
    }

    fn expect_declared(&mut self, e: &ast::Expr, want: &Ty, declared: Option<&ast::Type>) -> Expr {
        let cells = matches!(&e.kind, ast::ExprKind::List(items) if !items.is_empty())
            && matches!(want, Ty::Array { dims, .. } if matches!(dims[..], [Dim::Int | Dim::IntFrom(_)]));
        let ranged_comp = matches!(&e.kind, ast::ExprKind::Table { .. })
            && declared.is_some_and(|d| {
                self.range_info_from_ast(d)
                    .and_then(|r| r.elem().and_then(|e| e.scalar()).cloned())
                    .is_some()
            });
        match declared.filter(|_| cells || ranged_comp) {
            Some(declared) => match self.let_value(Some(declared), Some(e), e.span) {
                Some(value) => self.coerce(value, want, e.span),
                None => self.err_expr(e.span),
            },
            None => self.expect(e, want),
        }
    }

    fn variant(&mut self, id: EnumId, tag: usize, args: &[ast::Arg], span: Span) -> Expr {
        let (name, fields) = self.enums[id].variants[tag].clone();
        let Some(exprs) = self.positional_any(&name, args, fields.len(), span) else {
            return self.err_expr(span);
        };
        let values: Vec<Expr> = exprs
            .into_iter()
            .zip(&fields)
            .map(|(e, ty)| self.expect(e, ty))
            .collect();
        if let Some(&d) = self.enum_to_sum_domain.get(&id) {
            return self.encode_sum_variant(d, tag, values, span);
        }
        self.mk(ExprKind::Variant(tag, values), Ty::Enum(id), span)
    }

    /// Fns every file can call without declaring them.
    fn prelude(&mut self, name: &str, args: &[ast::Arg], hint: Option<&Ty>, span: Span) -> Expr {
        match name {
            "print" => {
                let mut digits = crate::FLOAT_DIGITS;
                let mut values = Vec::new();
                let mut sep = None;
                let mut none_text = None;
                let mut end = None;
                for arg in args {
                    match &arg.name {
                        // `end: " "` is written in place of the newline.
                        Some(arg_name) if arg_name.text == "end" => {
                            end = Some(Box::new(self.expect(&arg.value, &Ty::Str)));
                        }
                        // `sep: "\n"` goes between the arguments and between
                        // the items of a list; `none: "-1"` is what a `none`
                        // is written as.
                        Some(arg_name) if arg_name.text == "sep" => {
                            sep = Some(self.expect(&arg.value, &Ty::Str));
                        }
                        Some(arg_name) if arg_name.text == "none" => {
                            none_text = Some(self.expect(&arg.value, &Ty::Str));
                        }
                        Some(arg_name) if arg_name.text == "digits" => match arg.value.kind {
                            ast::ExprKind::Int(d) if (0..=18).contains(&d) => digits = d as u32,
                            _ => self.error(
                                codes::TYPE_MISMATCH,
                                arg.value.span,
                                "`digits` is a whole number from 0 to 18",
                            ),
                        },
                        Some(arg_name) => {
                            let message =
                                format!("`print` has no argument named `{}`", arg_name.text);
                            self.error(codes::UNKNOWN_ARG_NAME, arg_name.span, message);
                        }
                        None => values.push((self.expr(&arg.value, None), arg.value.span)),
                    }
                }
                // Every argument is worked out before anything is written:
                // one that prints (`print("A", side())`) has its output
                // ahead of this line, in both back ends. Arguments after
                // the last call are read where they are.
                let calls = |e: &Expr| {
                    let mut found = false;
                    crate::walk::deep_expr(e, &mut |inner| {
                        found |= matches!(inner.kind, ExprKind::Call(..) | ExprKind::CallExpr(..));
                    });
                    found
                };
                let mut ahead = Vec::new();
                if let Some(last) = values.iter().rposition(|(value, _)| calls(value)) {
                    for (value, _) in &mut values[..=last] {
                        let literal = matches!(
                            value.kind,
                            ExprKind::Int(_) | ExprKind::Float(_) | ExprKind::Bool(_) | ExprKind::Char(_) | ExprKind::Str(_)
                        );
                        if literal || value.ty == Ty::Unit || value.ty == Ty::Error {
                            continue;
                        }
                        let slot = self.temp(value.ty.clone());
                        let held = Expr {
                            kind: ExprKind::Local(slot),
                            ty: value.ty.clone(),
                            span: value.span,
                        };
                        ahead.push(Stmt::Let(slot, Some(std::mem::replace(value, held))));
                    }
                }
                if sep.is_some() || none_text.is_some() {
                    values = values
                        .into_iter()
                        .map(|(value, at)| {
                            (self.print_piece(value, sep.as_ref(), none_text.as_ref(), at), at)
                        })
                        .collect();
                }
                let mut values: Vec<FmtArg> = values
                    .into_iter()
                    .filter_map(|(value, at)| self.fmt_arg(value, digits, at))
                    .collect();
                if let Some(sep) = sep {
                    // One text, so the separator is the only thing between.
                    let mut pieces = Vec::new();
                    for (i, arg) in values.drain(..).enumerate() {
                        if i > 0 {
                            pieces.push(FmtPiece::Value(FmtArg { value: sep.clone(), digits }));
                        }
                        pieces.push(FmtPiece::Value(arg));
                    }
                    let line = self.mk(ExprKind::Format(pieces), Ty::Str, span);
                    values.push(FmtArg { value: line, digits });
                }
                let print = self.mk(ExprKind::Print(values, end), Ty::Unit, span);
                if ahead.is_empty() {
                    return print;
                }
                ahead.push(Stmt::Expr(print));
                ahead.push(Stmt::Give(self.mk(ExprKind::Unit, Ty::Unit, span)));
                self.mk(ExprKind::Do(Block { stmts: ahead }), Ty::Unit, span)
            }
            // `debug(i, best)`: shown on a local run, and a checkpoint
            // of its trace. A string literal names the checkpoint.
            "debug" => {
                let mut values = Vec::new();
                for arg in args {
                    if let Some(arg_name) = &arg.name {
                        let message = format!("`debug` has no argument named `{}`", arg_name.text);
                        self.error(codes::UNKNOWN_ARG_NAME, arg_name.span, message);
                        continue;
                    }
                    let text = self.source(arg.value.span);
                    let label = matches!(arg.value.kind, ast::ExprKind::Str(_));
                    let value = self.expr(&arg.value, None);
                    if value.ty == Ty::Unit {
                        self.error(codes::TYPE_MISMATCH, arg.value.span, "this has no value to show");
                        continue;
                    }
                    values.push(DebugArg { text, value, label });
                }
                let line = self.line(span);
                self.mk(ExprKind::Debug(line, values), Ty::Unit, span)
            }
            // `let k: Int = read_output()` in a `@checker`: the next
            // thing the solution printed.
            "read_output" => {
                if self.aside != Some(Aside::Checker) {
                    self.error(
                        codes::UNRESOLVED_NAME,
                        span,
                        "`read_output()` reads what the solution printed, which only a `@checker` fn sees",
                    );
                    return self.err_expr(span);
                }
                if !args.is_empty() {
                    self.error(codes::WRONG_ARG_COUNT, span, "`read_output` takes no arguments");
                    return self.err_expr(span);
                }
                let Some(want) = hint.filter(|want| **want != Ty::Unit) else {
                    self.error(codes::TYPE_MISMATCH, span, "`read_output` can't tell what to read");
                    self.help("say what the solution printed: `let k: Int = read_output()`");
                    return self.err_expr(span);
                };
                let Some(spec) = self.spec_of(want) else {
                    let message = format!("`read_output` can't tell how much a `{}` is", self.show(want));
                    self.error(codes::TYPE_MISMATCH, span, message);
                    return self.err_expr(span);
                };
                self.read_printed(spec, span)
            }
            // `let r: Int = ask("?", x)`: the query goes out, the answer
            // comes back as the type says. `ask("!", x)` only prints.
            "ask" => {
                self.interactive = true;
                let mut values = Vec::new();
                let mut limit = None;
                for arg in args {
                    match &arg.name {
                        // `limit: K`: the judge allows `K` queries, and
                        // one more fails here as it would there.
                        Some(name) if name.text == "limit" => {
                            limit = Some(self.expect(&arg.value, &Ty::Int));
                        }
                        Some(name) => {
                            let message = format!("`ask` has no argument named `{}`", name.text);
                            self.error(codes::UNKNOWN_ARG_NAME, name.span, message);
                        }
                        None => values.push((self.expr(&arg.value, None), arg.value.span)),
                    }
                }
                let values: Vec<FmtArg> = values
                    .into_iter()
                    .filter_map(|(value, at)| self.fmt_arg(value, crate::FLOAT_DIGITS, at))
                    .collect();
                let query = self.mk(ExprKind::Print(values, None), Ty::Unit, span);
                let counted = limit.map(|limit| {
                    self.mk(ExprKind::Builtin(Builtin::CountQuery, vec![limit]), Ty::Unit, span)
                });
                let Some(want) = hint.filter(|want| **want != Ty::Unit) else {
                    let Some(counted) = counted else {
                        return query;
                    };
                    return self.do_expr(Some(&Ty::Unit), span, |lw, out| {
                        out.push(Stmt::Expr(counted));
                        out.push(Stmt::Expr(query));
                        out.push(Stmt::Give(lw.mk(ExprKind::Unit, Ty::Unit, span)));
                    });
                };
                let Some(spec) = self.spec_of(want) else {
                    let message = format!("`ask` can't tell how much a `{}` is", self.show(want));
                    self.error(codes::TYPE_MISMATCH, span, message);
                    self.help("give the answer its full type: `let r: Int = ask(\"?\", x)`");
                    return self.err_expr(span);
                };
                let want = want.clone();
                self.do_expr(Some(&want), span, |lw, out| {
                    out.extend(counted.map(Stmt::Expr));
                    out.push(Stmt::Expr(query));
                    let answer = lw.read(spec, span);
                    out.push(Stmt::Give(answer));
                })
            }
            "read" => {
                // `read(1..=n)` reads an `Int` and checks it is in the range.
                if let [ast::Arg { name: None, mutable: false, value }] = args {
                    if let ast::ExprKind::Range {
                        lo: Some(lo),
                        hi: Some(hi),
                        inclusive,
                        step: None,
                    } = &value.kind
                    {
                        let lo = self.expect(lo, &Ty::Int);
                        let mut hi = self.expect(hi, &Ty::Int);
                        if !inclusive {
                            hi = self.int_add(hi, -1);
                        }
                        let spec = Spec::Int {
                            lo: Some(Box::new(lo)),
                            hi: Some(Box::new(hi)),
                            text: self.source(value.span),
                        };
                        return self.read(spec, span);
                    }
                }
                // `read(none: 0)`: the number that a `?T` reads as `none`.
                let absent = args
                    .iter()
                    .find(|a| a.name.as_ref().is_some_and(|n| n.text == "none"))
                    .map(|a| self.expect(&a.value, &Ty::Int));
                if !args
                    .iter()
                    .all(|a| a.name.as_ref().is_some_and(|n| n.text == "gen" || n.text == "none"))
                    && self.positional_any("read", args, 0, span).is_none()
                {
                    return self.err_expr(span);
                }
                let Some(want) = hint else {
                    self.error(codes::TYPE_MISMATCH, span, "can't tell what to read here");
                    self.help("`read()` reads what the type says: `let n: Int = read()`");
                    return self.err_expr(span);
                };
                match self.spec_of(want) {
                    Some(mut spec) => {
                        if let Some(absent) = absent {
                            if !reads_none_as(&mut spec, &absent) {
                                let message = format!(
                                    "`none:` is for reading a `?T`, and a `{}` has none",
                                    self.show(want)
                                );
                                self.error(codes::UNKNOWN_ARG_NAME, span, message);
                            }
                        }
                        self.read(spec, span)
                    }
                    None => {
                        let message =
                            format!("`read()` can't tell how much a `{}` is", self.show(want));
                        self.error(codes::TYPE_MISMATCH, span, message);
                        self.help(
                            "declare the variable with its full type: `let a: [n]Int = read()`",
                        );
                        self.err_expr(span)
                    }
                }
            }
            "min" | "max" => self.min_max(name == "max", args, span),
            "first" => {
                if let Some(walk) = self.tested_walk(name, args, true) {
                    return self.lower_first(&walk, span);
                }
                let Some(exprs) = self.positional_any(name, args, 1, span) else {
                    return self.err_expr(span);
                };
                self.lower_first(exprs[0], span)
            }
            "sum" | "prod" | "count" | "all" | "any" => {
                let kind = match name {
                    "sum" => Reduce::Sum,
                    "prod" => Reduce::Prod,
                    "count" => Reduce::Count,
                    "all" => Reduce::All,
                    _ => Reduce::Any,
                };
                if let Some(walk) = self.tested_walk(name, args, false) {
                    return self.reduce(kind, &walk, span);
                }
                let Some(exprs) = self.positional_any(name, args, 1, span) else {
                    return self.err_expr(span);
                };
                self.reduce(kind, exprs[0], span)
            }
            "abs" => {
                let Some(exprs) = self.positional_any(name, args, 1, span) else {
                    return self.err_expr(span);
                };
                let value = self.expr(exprs[0], None);
                match value.ty {
                    Ty::Int | Ty::I128 | Ty::Float => {
                        let ty = value.ty.clone();
                        self.mk(ExprKind::Builtin(Builtin::Abs, vec![value]), ty, span)
                    }
                    Ty::Error => value,
                    _ => {
                        self.mismatch(exprs[0].span, &Ty::Int, &value.ty);
                        self.err_expr(span)
                    }
                }
            }
            "sqrt" => match self.one_arg(name, args, &Ty::Float, span) {
                Some(value) => self.mk(
                    ExprKind::Builtin(Builtin::Sqrt, vec![value]),
                    Ty::Float,
                    span,
                ),
                None => self.err_expr(span),
            },
            // A program that flushes is read by someone who answers: it
            // reads as it goes.
            "flush" => match self.positional_any(name, args, 0, span) {
                Some(_) => {
                    self.interactive = true;
                    self.mk(ExprKind::Builtin(Builtin::Flush, Vec::new()), Ty::Unit, span)
                }
                None => self.err_expr(span),
            },
            "isqrt" => match self.one_arg(name, args, &Ty::Int, span) {
                Some(value) => self.mk(ExprKind::Builtin(Builtin::Isqrt, vec![value]), Ty::Int, span),
                None => self.err_expr(span),
            },
            "atan2" => match self.positional_any(name, args, 2, span) {
                Some(exprs) => {
                    let y = self.expect(exprs[0], &Ty::Float);
                    let x = self.expect(exprs[1], &Ty::Float);
                    self.mk(ExprKind::Builtin(Builtin::Atan2, vec![y, x]), Ty::Float, span)
                }
                None => self.err_expr(span),
            },
            "popcount" | "ctz" | "clz" | "bit_width" | "bit_ceil" => {
                let op = match name {
                    "popcount" => Builtin::Popcount,
                    "ctz" => Builtin::Ctz,
                    "clz" => Builtin::Clz,
                    "bit_width" => Builtin::BitWidth,
                    _ => Builtin::BitCeil,
                };
                // The bits of a `U64` are counted as they are.
                let value = match self.positional_any(name, args, 1, span) {
                    Some(exprs) if op != Builtin::BitCeil => {
                        let value = self.expr(exprs[0], None);
                        match value.ty {
                            Ty::U64 => value,
                            _ => self.coerce(value, &Ty::Int, exprs[0].span),
                        }
                    }
                    Some(exprs) => self.expect(exprs[0], &Ty::Int),
                    None => return self.err_expr(span),
                };
                self.mk(ExprKind::Builtin(op, vec![value]), Ty::Int, span)
            }
            "bit" => match self.one_arg(name, args, &Ty::Int, span) {
                Some(k) => {
                    let one = self.int_lit(1, span);
                    self.mk(
                        ExprKind::Binary(BinOp::Shl, Box::new(one), Box::new(k)),
                        Ty::Int,
                        span,
                    )
                }
                None => self.err_expr(span),
            },
            "swap" => {
                if args.len() != 2 {
                    self.error(codes::WRONG_ARG_COUNT, span, "`swap` takes 2 arguments");
                    return self.err_expr(span);
                }
                let pa = self.place(&args[0].value);
                let pb = self.place(&args[1].value);
                let ty = pa.ty.clone();
                if pb.ty != ty && ty != Ty::Error && pb.ty != Ty::Error {
                    self.mismatch(args[1].value.span, &ty, &pb.ty);
                    return self.err_expr(span);
                }
                // Three moves, so two lists trade places in O(1).
                self.do_expr(Some(&Ty::Unit), span, |lw, out| {
                    let pa = lw.settle_indexes(pa, out);
                    let pb = lw.settle_indexes(pb, out);
                    let tmp = lw.temp(ty.clone());
                    let held = Place {
                        kind: PlaceKind::Local(tmp),
                        ty: ty.clone(),
                        span: lw.span(span),
                    };
                    let take = |lw: &mut Self, place: &Place| {
                        lw.mk(
                            ExprKind::Mutate(Mutation::Take, Box::new(place.clone()), Vec::new()),
                            ty.clone(),
                            span,
                        )
                    };
                    let from_a = take(lw, &pa);
                    out.push(Stmt::Let(tmp, Some(from_a)));
                    let from_b = take(lw, &pb);
                    out.push(Stmt::Assign(pa, AssignOp::Set, from_b));
                    let from_tmp = take(lw, &held);
                    out.push(Stmt::Assign(pb, AssignOp::Set, from_tmp));
                    out.push(Stmt::Give(lw.mk(ExprKind::Unit, Ty::Unit, span)));
                })
            }
            "rand" => match self.positional_any(name, args, 2, span) {
                Some(exprs) => {
                    let lo = self.expect(exprs[0], &Ty::Int);
                    let hi = self.expect(exprs[1], &Ty::Int);
                    self.mk(ExprKind::Builtin(Builtin::Rand, vec![lo, hi]), Ty::Int, span)
                }
                None => self.err_expr(span),
            },
            "shuffle" => {
                if args.len() != 1 {
                    self.error(codes::WRONG_ARG_COUNT, span, "`shuffle` takes 1 list argument");
                    return self.err_expr(span);
                }
                let place = self.place(&args[0].value);
                self.mk(
                    ExprKind::Mutate(Mutation::Shuffle, Box::new(place), Vec::new()),
                    Ty::Unit,
                    span,
                )
            }
            "copy" => match self.positional_any(name, args, 1, span) {
                Some(exprs) => {
                    let val = self.expr(exprs[0], hint);
                    let ty = val.ty.clone();
                    self.mk(ExprKind::Builtin(Builtin::Copy, vec![val]), ty, span)
                }
                None => self.err_expr(span),
            },
            "some" => match self.positional_any(name, args, 1, span) {
                Some(exprs) => {
                    let inner_h = match hint {
                        Some(Ty::Opt(inner)) => Some(&**inner),
                        _ => None,
                    };
                    let val = self.expr(exprs[0], inner_h);
                    let ty = Ty::opt(val.ty.clone());
                    self.mk(ExprKind::Some(Box::new(val)), ty, span)
                }
                None => self.err_expr(span),
            },
            "laws_hold" => self.lower_laws_hold(args, span),
            "sort" => {
                if args.is_empty() {
                    self.error(codes::WRONG_ARG_COUNT, span, "`sort` takes a list or array");
                    return self.err_expr(span);
                }
                let target = &args[0].value;
                let place = self.place(target);
                let elem = match &place.ty {
                    Ty::List(e) | Ty::Array { elem: e, .. } => (**e).clone(),
                    Ty::Error => return self.err_expr(span),
                    other => {
                        let message = format!("`sort` orders a list or an array, not a `{}`", self.show(other));
                        self.error(codes::BAD_OPERANDS, target.span, message);
                        if matches!(other, Ty::Range(_)) {
                            self.help("a range has no cells to move; `sorted(r, by: ..)` gives the list in order, and `[x for x in r]` the list");
                        }
                        return self.err_expr(span);
                    }
                };
                self.sort_place(place, &elem, &args[1..], span)
            }
            // Toward zero: what `Int(x)` used to mean without saying so.
            "trunc" => match self.one_arg(name, args, &Ty::Float, span) {
                Some(value) => self.cast(Cast::FloatToInt, value, Ty::Int),
                None => self.err_expr(span),
            },
            "floor" | "ceil" | "round" => {
                let op = match name {
                    "floor" => Builtin::Floor,
                    "ceil" => Builtin::Ceil,
                    _ => Builtin::Round,
                };
                match self.one_arg(name, args, &Ty::Float, span) {
                    Some(value) => self.mk(ExprKind::Builtin(op, vec![value]), Ty::Int, span),
                    None => self.err_expr(span),
                }
            }
            "len" => {
                let Some(exprs) = self.positional_any(name, args, 1, span) else {
                    return self.err_expr(span);
                };
                let value = self.expr(exprs[0], None);
                self.len_of(value, span)
            }
            "choose2" => {
                let Some(exprs) = self.positional_any("choose2", args, 1, span) else {
                    return self.err_expr(span);
                };
                let n = self.expect(exprs[0], &Ty::Int);
                self.do_expr(Some(&Ty::Int), span, |lw, out| {
                    let v = lw.hold(n, out);
                    let prev = lw.int_add(v.clone(), -1);
                    let prod = lw.mk(
                        ExprKind::Binary(BinOp::Mul, Box::new(v), Box::new(prev)),
                        Ty::Int,
                        span,
                    );
                    let two = lw.int_lit(2, span);
                    let res = lw.mk(
                        ExprKind::Binary(BinOp::Div, Box::new(prod), Box::new(two)),
                        Ty::Int,
                        span,
                    );
                    out.push(Stmt::Give(res));
                })
            }
            "minmax" => {
                let Some(exprs) = self.positional_any("minmax", args, 2, span) else {
                    return self.err_expr(span);
                };
                let a = self.expr(exprs[0], None);
                let ty = a.ty.clone();
                let b = self.expect(exprs[1], &ty);
                let pair_ty = Ty::Tuple(vec![ty.clone(), ty.clone()]);
                self.do_expr(Some(&pair_ty), span, |lw, out| {
                    let av = lw.hold(a, out);
                    let bv = lw.hold(b, out);
                    let cmp = lw.mk(
                        ExprKind::Cmp(CmpOp::Le, Box::new(av.clone()), Box::new(bv.clone())),
                        Ty::Bool,
                        span,
                    );
                    let ab = lw.mk(
                        ExprKind::Tuple(vec![av.clone(), bv.clone()]),
                        pair_ty.clone(),
                        span,
                    );
                    let ba = lw.mk(ExprKind::Tuple(vec![bv, av]), pair_ty.clone(), span);
                    let res = lw.mk(
                        ExprKind::If(Box::new(cmp), Box::new(ab), Box::new(ba)),
                        pair_ty.clone(),
                        span,
                    );
                    out.push(Stmt::Give(res));
                })
            }
            "adjacency" => self.lower_adjacency(args, span),
            "sorted" => {
                if args.is_empty() {
                    self.error(codes::WRONG_ARG_COUNT, span, "`sorted` takes a sequence");
                    return self.err_expr(span);
                }
                let value = match self.walked(&args[0].value) {
                    Some(values) => self.expr(&values, None),
                    None => self.expr(&args[0].value, None),
                };
                if args.len() == 1 {
                    self.sorted(value, span)
                } else {
                    self.sorted_by(value, &args[1..], span)
                }
            }
            "argmax" | "argmin" if args.len() == 1 && self.lookup(name).is_none() => {
                self.arg_extreme(name == "argmax", &args[0].value, span)
            }
            "runs" => self.lower_runs(args, span),
            "span" => self.lower_span(args, span),
            "counts" => self.lower_counts(args, span),
            "prefix" => self.lower_prefix(args, span),
            "List" => {
                if args.is_empty() {
                    let ty = match hint {
                        Some(ty @ Ty::List(_)) => ty.clone(),
                        _ => Ty::list(Ty::Int),
                    };
                    return self.mk(ExprKind::Tuple(Vec::new()), ty, span);
                }
                let src = &args[0].value;
                let collect_directly = matches!(
                    &src.kind,
                    ast::ExprKind::Range { .. } | ast::ExprKind::Generic { .. }
                ) || matches!(
                    &src.kind,
                    ast::ExprKind::Field { name, .. }
                        if matches!(name.text.as_str(), "bounds" | "subsets")
                ) || matches!(
                    &src.kind,
                    ast::ExprKind::Name(n)
                        if self.lookup(n).is_none()
                            && (self.resolve_domain_name(n).is_some()
                                || matches!(self.get_sym(n), Some(Sym::Struct(sid)) if self.structs[sid].index_domain.is_some()))
                );
                if collect_directly {
                    return self.collect_iter_to_list(src, span);
                }
                let val = self.expr(src, None);
                match val.ty.clone() {
                    Ty::List(_) => val,
                    Ty::ListView(elem) | Ty::MutView(elem) => self.mk(
                        ExprKind::Builtin(Builtin::ToList, vec![val]),
                        Ty::List(elem),
                        span,
                    ),
                    Ty::Str | Ty::StrView => self.mk(
                        ExprKind::Builtin(Builtin::StrToList, vec![val]),
                        Ty::list(Ty::Char),
                        span,
                    ),
                    Ty::Array { elem, .. } => self.mk(
                        ExprKind::Builtin(Builtin::ArrayToList, vec![val]),
                        Ty::List(elem),
                        span,
                    ),
                    Ty::Set(elem) | Ty::HashSet(elem) => {
                        self.mk(ExprKind::Builtin(Builtin::Sorted, vec![val]), Ty::List(elem), span)
                    }
                    Ty::DomainVal | Ty::Range(_) | Ty::Map(..) | Ty::HashMap(..) => {
                        self.collect_iter_to_list(src, span)
                    }
                    Ty::Pos(d) if self.subset_base(d).is_some() => {
                        self.collect_iter_to_list(src, span)
                    }
                    _ => val,
                }
            }
            "Set" | "HashSet" => {
                let hashed = name == "HashSet";
                if args.is_empty() {
                    let ty = match hint {
                        Some(ty @ (Ty::Set(_) | Ty::HashSet(_))) => ty.clone(),
                        _ if hashed => Ty::HashSet(Box::new(Ty::Int)),
                        _ => Ty::Set(Box::new(Ty::Int)),
                    };
                    return self.mk(ExprKind::Tuple(Vec::new()), ty, span);
                }
                let (iter, elem) = self.iter(&args[0].value);
                let ty = if hashed {
                    Ty::HashSet(Box::new(elem.clone()))
                } else {
                    Ty::Set(Box::new(elem.clone()))
                };
                self.do_expr(Some(&ty), span, |lw, out| {
                    let acc = lw.temp(ty.clone());
                    out.push(Stmt::Let(acc, Some(lw.mk(ExprKind::Tuple(Vec::new()), ty.clone(), span))));
                    let lid = lw.open_loop(None);
                    let slot = lw.temp(elem);
                    let item = lw.local(slot, span);
                    let ins = lw.mk(
                        ExprKind::Mutate(
                            Mutation::Insert,
                            Box::new(Place { kind: PlaceKind::Local(acc), ty: ty.clone(), span: lw.span(span) }),
                            vec![item],
                        ),
                        Ty::Bool,
                        span,
                    );
                    lw.cx.loops.pop();
                    out.push(Stmt::For(lid, slot, iter, Block { stmts: vec![Stmt::Expr(ins)] }));
                    let res = lw.local(acc, span);
                    out.push(Stmt::Give(res));
                })
            }
            "Map" | "HashMap" => {
                let hashed = name == "HashMap";
                let ty = match hint {
                    Some(ty @ (Ty::Map(..) | Ty::HashMap(..))) => ty.clone(),
                    _ if hashed => Ty::HashMap(Box::new(Ty::Int), Box::new(Ty::Int)),
                    _ => Ty::Map(Box::new(Ty::Int), Box::new(Ty::Int)),
                };
                self.mk(ExprKind::Tuple(Vec::new()), ty, span)
            }
            "Int" | "I128" | "U64" | "F64" | "Char" | "Str" | "Bool" => {
                if name == "Str" && args.is_empty() {
                    return self.mk(ExprKind::Str(Vec::new()), Ty::Str, span);
                }
                let Some(exprs) = self.positional_any(name, args, 1, span) else {
                    return self.err_expr(span);
                };
                let value = self.expr(exprs[0], None);
                self.convert(name, value, span)
            }
            _ => {
                let message = format!("`cpl run` doesn't know a fn named `{name}`");
                self.error(codes::UNRESOLVED_NAME, span, message);
                self.err_expr(span)
            }
        }
    }

    /// `sep.join(items)`, with `none` written as `none_text` when given.
    fn join(&mut self, sep: Expr, items: Expr, none_text: Option<&Expr>, span: Span) -> Expr {
        let flat = match &items.ty {
            Ty::List(_) => true,
            Ty::Array { dims, .. } => dims.len() == 1,
            Ty::Error => return items,
            _ => false,
        };
        if !flat {
            let message = format!("`join` takes a list or a 1-D array, not a `{}`", self.show(&items.ty));
            self.error(codes::TYPE_MISMATCH, span, message);
            return self.err_expr(span);
        }
        let mut args = vec![sep, items];
        args.extend(none_text.cloned());
        self.mk(ExprKind::Builtin(Builtin::Join, args), Ty::Str, span)
    }

    /// One `print` argument under `sep:` / `none:`, as the text it prints.
    fn print_piece(&mut self, value: Expr, sep: Option<&Expr>, none_text: Option<&Expr>, span: Span) -> Expr {
        let flat = match &value.ty {
            Ty::List(_) => true,
            Ty::Array { dims, .. } => dims.len() == 1,
            _ => false,
        };
        if flat {
            let sep = match sep {
                Some(sep) => sep.clone(),
                None => self.mk(ExprKind::Str(b" ".to_vec()), Ty::Str, span),
            };
            return self.join(sep, value, none_text, span);
        }
        let (Ty::Opt(inner), Some(none_text)) = (value.ty.clone(), none_text) else {
            return value;
        };
        let none_text = none_text.clone();
        self.do_expr(Some(&Ty::Str), span, |lw, out| {
            let slot = lw.temp(value.ty.clone());
            out.push(Stmt::Let(slot, Some(value)));
            let held = lw.local(slot, span);
            let some = lw.mk(ExprKind::IsSome(Box::new(held.clone())), Ty::Bool, span);
            let inside = lw.mk(ExprKind::Unwrap(Box::new(held), false), *inner, span);
            let shown = lw.mk(ExprKind::Builtin(Builtin::ShowStr, vec![inside]), Ty::Str, span);
            let text = lw.mk(
                ExprKind::If(Box::new(some), Box::new(shown), Box::new(none_text)),
                Ty::Str,
                span,
            );
            out.push(Stmt::Give(text));
        })
    }

    fn convert(&mut self, to: &str, value: Expr, span: Span) -> Expr {
        let from = value.ty.clone();
        let (cast, ty) = match (to, &from) {
            (_, Ty::Error) => return value,
            ("Int", Ty::Int)
            | ("I128", Ty::I128)
            | ("U64", Ty::U64)
            | ("F64", Ty::Float)
            | ("Char", Ty::Char)
            | ("Str", Ty::Str)
            | ("Bool", Ty::Bool) => return value,
            ("Int", Ty::Float) => {
                self.error(
                    codes::TYPE_MISMATCH,
                    span,
                    "`Int` of an F64 does not say how to round",
                );
                self.help("write `floor(x)`, `ceil(x)`, `round(x)` or `trunc(x)`");
                return self.err_expr(span);
            }
            ("Int", Ty::Char) => (Cast::CharToInt, Ty::Int),
            ("Int", Ty::Str) => {
                return self.mk(ExprKind::Builtin(Builtin::ParseInt, vec![value]), Ty::Int, span);
            }
            ("Int", Ty::Pos(d)) => (Cast::Label(*d), Ty::Int),
            ("Int", Ty::Bound(_)) => (Cast::Ord, Ty::Int),
            ("Int", Ty::Enum(eid)) if self.enums[*eid].variants.iter().all(|(_, f)| f.is_empty()) => {
                (Cast::EnumVal(*eid), Ty::Int)
            }
            ("Int", Ty::Bool) => {
                let one = self.int_lit(1, span);
                let zero = self.int_lit(0, span);
                return self.mk(
                    ExprKind::If(Box::new(value), Box::new(one), Box::new(zero)),
                    Ty::Int,
                    span,
                );
            }
            ("Int", Ty::Enum(eid))
                if !self.enums[*eid].variants.is_empty()
                    && self.enums[*eid].variants.iter().all(|(_, fs)| fs.is_empty()) =>
            {
                let eid = *eid;
                let loc = self.temp(Ty::Enum(eid));
                let init = Stmt::Let(loc, Some(value));
                let discs = self.enum_discriminants.get(eid).cloned().unwrap_or_default();
                let n = self.enums[eid].variants.len();
                let mut acc = self.int_lit(discs.get(n - 1).copied().unwrap_or((n - 1) as i64), span);
                for tag in (0..n.saturating_sub(1)).rev() {
                    let disc = discs.get(tag).copied().unwrap_or(tag as i64);
                    let cond = self.mk(
                        ExprKind::IsVariant(
                            Box::new(self.mk(ExprKind::Local(loc), Ty::Enum(eid), span)),
                            tag,
                        ),
                        Ty::Bool,
                        span,
                    );
                    let then = self.int_lit(disc, span);
                    acc = self.mk(
                        ExprKind::If(Box::new(cond), Box::new(then), Box::new(acc)),
                        Ty::Int,
                        span,
                    );
                }
                return self.mk(
                    ExprKind::Do(Block {
                        stmts: vec![init, Stmt::Give(acc)],
                    }),
                    Ty::Int,
                    span,
                );
            }
            ("F64", Ty::Int) => (Cast::IntToFloat, Ty::Float),
            ("I128", Ty::Int) => (Cast::IntToI128, Ty::I128),
            ("Int", Ty::I128) => (Cast::I128ToInt, Ty::Int),
            ("U64", Ty::Int) => (Cast::IntToU64, Ty::U64),
            ("Int", Ty::U64) => (Cast::U64ToInt, Ty::Int),
            ("U64", Ty::I128) => (Cast::I128ToU64, Ty::U64),
            ("I128", Ty::U64) => (Cast::U64ToI128, Ty::I128),
            ("F64", Ty::I128) => (Cast::I128ToFloat, Ty::Float),
            ("F64", Ty::U64) => (Cast::U64ToFloat, Ty::Float),
            ("Char", Ty::Int) => (Cast::IntToChar, Ty::Char),
            ("Str", Ty::List(elem)) if **elem == Ty::Char => {
                return self.mk(ExprKind::Builtin(Builtin::ListToStr, vec![value]), Ty::Str, span);
            }
            ("Str", _) => {
                return self.mk(ExprKind::Builtin(Builtin::ShowStr, vec![value]), Ty::Str, span);
            }
            ("Bool", Ty::Int) => {
                let zero = self.int_lit(0, span);
                return self.mk(
                    ExprKind::Cmp(CmpOp::Ne, Box::new(value), Box::new(zero)),
                    Ty::Bool,
                    span,
                );
            }
            _ => {
                let message = format!("a `{}` can't be turned into a `{to}`", self.show(&from));
                self.error(codes::TYPE_MISMATCH, span, message);
                return self.err_expr(span);
            }
        };
        self.mk(ExprKind::Cast(cast, Box::new(value)), ty, span)
    }

    pub(super) fn len_of(&mut self, value: Expr, span: Span) -> Expr {
        match value.ty {
            Ty::Array { .. }
            | Ty::List(_)
            | Ty::ListView(_) | Ty::MutView(_)
            | Ty::Str
            | Ty::StrView
            | Ty::Map(..)
            | Ty::HashMap(..)
            | Ty::Set(_)
            | Ty::HashSet(_)
            | Ty::Range(_) => {
                self.mk(ExprKind::Builtin(Builtin::Len, vec![value]), Ty::Int, span)
            }
            Ty::Error => value,
            _ => {
                let message = format!("a `{}` has no length", self.show(&value.ty));
                self.error(codes::UNKNOWN_FIELD, span, message);
                self.err_expr(span)
            }
        }
    }

    fn sorted(&mut self, value: Expr, span: Span) -> Expr {
        match &value.ty {
            Ty::List(elem) | Ty::ListView(elem) | Ty::MutView(elem) => {
                if !self.ordered(elem) {
                    let message = format!("a `{}` has no order to sort by", self.show(elem));
                    self.error(codes::BAD_OPERANDS, span, message);
                    return self.err_expr(span);
                }
                let ty = Ty::list((**elem).clone());
                self.mk(ExprKind::Builtin(Builtin::Sorted, vec![value]), ty, span)
            }
            Ty::Array { elem, .. } => {
                let elem = elem.clone();
                let list = self.mk(
                    ExprKind::Builtin(Builtin::ArrayToList, vec![value]),
                    Ty::List(elem.clone()),
                    span,
                );
                self.mk(ExprKind::Builtin(Builtin::Sorted, vec![list]), Ty::List(elem), span)
            }
            Ty::Set(elem) | Ty::HashSet(elem) => {
                // A set is walked in order already.
                let ty = Ty::List(elem.clone());
                self.mk(ExprKind::Builtin(Builtin::Sorted, vec![value]), ty, span)
            }
            ty if ty.is_seq() => {
                let elem = ty.seq_elem().unwrap();
                if !self.ordered(&elem) {
                    let message = format!("a `{}` has no order to sort by", self.show(&elem));
                    self.error(codes::BAD_OPERANDS, span, message);
                    return self.err_expr(span);
                }
                let list = self.walked_list(Iter::Seq(value, false), elem.clone(), span);
                self.mk(ExprKind::Builtin(Builtin::Sorted, vec![list]), Ty::list(elem), span)
            }
            Ty::Error => value,
            other => {
                let message = format!("`sorted` takes a list, not a `{}`", self.show(other));
                self.error(codes::TYPE_MISMATCH, span, message);
                self.err_expr(span)
            }
        }
    }

    /// Every position of a domain, first to last, as a list.
    fn positions(&mut self, d: DomainId, span: Span) -> Expr {
        let list_ty = Ty::list(Ty::Pos(d));
        self.do_expr(Some(&list_ty.clone()), span, |lw, out| {
            let all = lw.temp(list_ty.clone());
            let empty = lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span);
            out.push(Stmt::Let(all, Some(empty)));
            let at = lw.temp(Ty::Pos(d));
            let item = lw.local(at, span);
            let push = lw.mk(
                ExprKind::Mutate(
                    Mutation::Push,
                    Box::new(Place {
                        kind: PlaceKind::Local(all),
                        ty: list_ty.clone(),
                        span,
                    }),
                    vec![item],
                ),
                Ty::Unit,
                span,
            );
            let lid = lw.next_loop;
            lw.next_loop += 1;
            let iter = Iter::Range {
                lo: lw.int_lit(0, span),
                hi: lw.mk(ExprKind::DomainLen(d), Ty::Int, span),
                step: None,
                rev: false,
                closed: false,
            };
            out.push(Stmt::For(lid, at, iter, Block { stmts: vec![Stmt::Expr(push)] }));
            out.push(Stmt::Give(lw.local(all, span)));
        })
    }

    /// The list of what `lo..hi` goes over, in order.
    pub(super) fn range_list(&mut self, lo: Expr, hi: Expr, elem: Ty, span: Span) -> Expr {
        self.walked_list(
            Iter::Range {
                lo,
                hi,
                step: None,
                rev: false,
                closed: false,
            },
            elem,
            span,
        )
    }

    /// The list of what `iter` goes over, in the order it goes.
    pub(super) fn walked_list(&mut self, iter: Iter, elem: Ty, span: Span) -> Expr {
        self.collected(elem.clone(), span, |lw, out, push| {
            let item = lw.temp(elem);
            let lid = lw.fresh_loop();
            let at = lw.local(item, span);
            let body = Block { stmts: vec![push(lw, at)] };
            out.push(Stmt::For(lid, item, iter, body));
        })
    }

    /// `r.rev` kept as a value: the list of what the range goes over, last
    /// first. A loop over `r.rev` never comes here and builds no list.
    fn reversed_range(&mut self, range: Expr, span: Span) -> Expr {
        let Ty::Range(elem) = &range.ty else { unreachable!("only a range is asked for") };
        let elem = (**elem).clone();
        let iter = match range.kind {
            ExprKind::Range { lo, hi } => Iter::Range {
                lo: *lo,
                hi: *hi,
                step: None,
                rev: true,
                closed: false,
            },
            _ => Iter::RangeVal(range, true),
        };
        self.walked_list(iter, elem, span)
    }

    /// `sorted(xs, by: ...)`
    fn sorted_by(&mut self, value: Expr, args: &[ast::Arg], span: Span) -> Expr {
        // `sorted(V, by: ...)`: the positions of a domain, in another order.
        let value = match (value.kind, &value.ty) {
            (ExprKind::DomainVal(d), _) => self.positions(d, span),
            // `sorted(D.all, by: ...)`, `sorted(0..n, by: ...)`: a range
            // written out is the list of what it goes over.
            (ExprKind::Range { lo, hi }, Ty::Range(of)) => self.range_list(*lo, *hi, (**of).clone(), span),
            (kind, _) => Expr { kind, ..value },
        };
        let list_ty = match value.ty.clone() {
            Ty::List(e) | Ty::ListView(e) | Ty::MutView(e) => Ty::List(e),
            Ty::Array { elem, .. } => Ty::List(elem),
            Ty::Set(e) | Ty::HashSet(e) => Ty::List(e),
            ty if ty.is_seq() => Ty::list(ty.seq_elem().unwrap()),
            Ty::Error => return value,
            other => {
                self.error(
                    codes::TYPE_MISMATCH,
                    span,
                    format!("`sorted` takes a sequence, not a `{}`", self.show(&other)),
                );
                return self.err_expr(span);
            }
        };
        let elem = match &list_ty {
            Ty::List(e) => (**e).clone(),
            _ => unreachable!(),
        };
        self.do_expr(Some(&list_ty), span, |lw, out| {
            let init = match value.ty {
                Ty::List(_) => lw.mk(ExprKind::Builtin(Builtin::Copy, vec![value]), list_ty.clone(), span),
                Ty::ListView(_) | Ty::MutView(_) => lw.mk(ExprKind::Builtin(Builtin::ToList, vec![value]), list_ty.clone(), span),
                Ty::Array { .. } => lw.mk(
                    ExprKind::Builtin(Builtin::ArrayToList, vec![value]),
                    list_ty.clone(),
                    span,
                ),
                Ty::Set(_) | Ty::HashSet(_) => lw.mk(
                    ExprKind::Builtin(Builtin::Sorted, vec![value]),
                    list_ty.clone(),
                    span,
                ),
                _ => lw.walked_list(Iter::Seq(value, false), elem.clone(), span),
            };
            let slot = lw.temp(list_ty.clone());
            out.push(Stmt::Let(slot, Some(init)));
            let place = Place {
                kind: PlaceKind::Local(slot),
                ty: list_ty.clone(),
                span: lw.span(span),
            };
            let sort_expr = lw.sort_place(place, &elem, args, span);
            out.push(Stmt::Expr(sort_expr));
            let res = lw.local(slot, span);
            out.push(Stmt::Give(res));
        })
    }

    /// `runs(seq)` -> `List<(T, Int)>` grouping adjacent equal elements.
    fn lower_runs(&mut self, args: &[ast::Arg], span: Span) -> Expr {
        let Some(exprs) = self.positional_any("runs", args, 1, span) else {
            return self.err_expr(span);
        };
        let seq = self.expr(exprs[0], None);
        let elem = match seq.ty.seq_elem() {
            Some(e) => e,
            None if seq.ty == Ty::Error => return self.err_expr(span),
            None => {
                self.error(
                    codes::TYPE_MISMATCH,
                    exprs[0].span,
                    format!("`runs` takes a sequence, not `{}`", self.show(&seq.ty)),
                );
                return self.err_expr(span);
            }
        };
        let pair_ty = Ty::Tuple(vec![elem.clone(), Ty::Int]);
        let res_ty = Ty::list(pair_ty.clone());
        self.do_expr(Some(&res_ty), span, |lw, out| {
            let acc = lw.temp(res_ty.clone());
            out.push(Stmt::Let(
                acc,
                Some(lw.mk(ExprKind::Tuple(Vec::new()), res_ty.clone(), span)),
            ));
            let lid = lw.open_loop(None);
            let item_slot = lw.temp(elem.clone());
            let acc_expr = lw.local(acc, span);
            let acc_len = lw.mk(ExprKind::Builtin(Builtin::Len, vec![acc_expr.clone()]), Ty::Int, span);
            let zero = lw.int_lit(0, span);
            let nonempty = lw.mk(
                ExprKind::Cmp(CmpOp::Gt, Box::new(acc_len.clone()), Box::new(zero)),
                Ty::Bool,
                span,
            );
            let last_idx = lw.int_add(acc_len, -1);
            let last_elem = lw.mk(
                ExprKind::Index(Box::new(acc_expr.clone()), vec![last_idx.clone()]),
                pair_ty.clone(),
                span,
            );
            let last_val = lw.mk(ExprKind::Field(Box::new(last_elem), 0), elem.clone(), span);
            let eq = lw.mk(
                ExprKind::Cmp(CmpOp::Eq, Box::new(last_val), Box::new(lw.local(item_slot, span))),
                Ty::Bool,
                span,
            );
            let cond = lw.mk(
                ExprKind::Binary(BinOp::And, Box::new(nonempty), Box::new(eq)),
                Ty::Bool,
                span,
            );
            let count_place = Place {
                kind: PlaceKind::Field(
                    Box::new(Place {
                        kind: PlaceKind::Index(
                            Box::new(Place {
                                kind: PlaceKind::Local(acc),
                                ty: res_ty.clone(),
                                span: lw.span(span),
                            }),
                            vec![last_idx],
                        ),
                        ty: pair_ty.clone(),
                        span: lw.span(span),
                    }),
                    1,
                ),
                ty: Ty::Int,
                span: lw.span(span),
            };
            let inc = Stmt::Assign(
                count_place,
                AssignOp::Binary(BinOp::Add),
                lw.int_lit(1, span),
            );
            let pair = lw.mk(
                ExprKind::Tuple(vec![lw.local(item_slot, span), lw.int_lit(1, span)]),
                pair_ty.clone(),
                span,
            );
            let push = Stmt::Expr(lw.mk(
                ExprKind::Mutate(
                    Mutation::Push,
                    Box::new(Place {
                        kind: PlaceKind::Local(acc),
                        ty: res_ty.clone(),
                        span: lw.span(span),
                    }),
                    vec![pair],
                ),
                Ty::Unit,
                span,
            ));
            lw.cx.loops.pop();
            let seq_iter = if matches!(seq.ty, Ty::Range(_)) {
                Iter::RangeVal(seq, false)
            } else {
                Iter::Seq(seq, false)
            };
            out.push(Stmt::For(
                lid,
                item_slot,
                seq_iter,
                Block {
                    stmts: vec![Stmt::If(
                        cond,
                        Block { stmts: vec![inc] },
                        Block { stmts: vec![push] },
                    )],
                },
            ));
            let res = lw.local(acc, span);
            out.push(Stmt::Give(res));
        })
    }

    /// The spec of a type that says everything `read()` needs on its own.
    /// `read_output()` of a value laid out as `spec`: a `read()` from
    /// what the solution printed, then back to the input.
    pub(super) fn read_printed(&mut self, spec: Spec, span: Span) -> Expr {
        let want = spec.ty();
        self.do_expr(Some(&want), span, |lw, out| {
            let swap = lw.mk(ExprKind::Builtin(Builtin::SwapInput, Vec::new()), Ty::Unit, span);
            out.push(Stmt::Expr(swap.clone()));
            let printed = lw.read(spec, span);
            let slot = lw.temp(printed.ty.clone());
            out.push(Stmt::Let(slot, Some(printed)));
            out.push(Stmt::Expr(swap));
            out.push(Stmt::Give(lw.local(slot, span)));
        })
    }

    fn spec_of(&mut self, ty: &Ty) -> Option<Spec> {
        Some(match ty {
            Ty::Struct(id) => self.struct_spec(*id),
            Ty::Tuple(_) | Ty::Record(_) => {
                let fields = ty.fields().expect("a tuple or record");
                let specs: Option<Vec<Spec>> = fields.iter().map(|f| self.spec_of(f)).collect();
                Spec::Fields(ty.clone(), specs?)
            }
            Ty::Array { dims, elem } => {
                let dims: Option<Vec<DimSpec>> = dims
                    .iter()
                    .map(|d| match d {
                        Dim::Pos(d) => Some(DimSpec::Pos(*d)),
                        Dim::Bound(d) => Some(DimSpec::Bound(*d)),
                        Dim::Sym(d) => Some(DimSpec::Sym(*d)),
                        Dim::Enum(e) => Some(DimSpec::Enum(*e)),
                        Dim::Int | Dim::IntFrom(_) => None,
                    })
                    .collect();
                Spec::Array {
                    dims: dims?,
                    elem: Box::new(self.spec_of(elem)?),
                }
            }
            Ty::Opt(inner) => Spec::Opt(Box::new(self.spec_of(inner)?), None),
            other => Spec::Plain(other.clone()),
        })
    }

    /// `read()` of a value shaped like `spec`.
    pub(super) fn read(&mut self, spec: Spec, span: Span) -> Expr {
        fn unreadable(spec: &Spec) -> Option<Ty> {
            match spec {
                Spec::Plain(ty) => match ty {
                    Ty::Int | Ty::I128 | Ty::U64 | Ty::Float | Ty::Char | Ty::Str | Ty::Bool | Ty::Pos(_) | Ty::Error => None,
                    other => Some(other.clone()),
                },
                Spec::Int { .. } | Spec::Call(_) => None,
                Spec::Fields(_, fields) => fields.iter().find_map(unreadable),
                Spec::Array { elem, .. } => unreadable(elem),
                Spec::Opt(inner, absent) => match (&**inner, absent) {
                    (Spec::Plain(Ty::Int | Ty::Pos(_)) | Spec::Int { .. }, Some(_)) => None,
                    _ => Some(spec.ty()),
                },
            }
        }
        if let Some(ty) = unreadable(&spec) {
            let message = format!("`read()` has no input format for a `{}`", self.show(&ty));
            self.error(codes::UNSUPPORTED, span, message);
            if let Ty::Opt(inner) = &ty {
                if matches!(**inner, Ty::Int | Ty::Pos(_)) {
                    self.help("say which number in the input stands for `none`: `read(none: 0)`");
                }
            }
            return self.err_expr(span);
        }
        if let Spec::Array { dims, elem } = &spec {
            if let [DimSpec::Pos(d)] = dims[..] {
                if self.own_domains.contains(&d) {
                    return self.read_own(d, elem, spec.ty(), span);
                }
            }
        }
        // Deeper in, a domain this `read()` names has no size yet: reading
        // would take no values at all and go on.
        fn names_domain(spec: &Spec, own: &std::collections::HashSet<DomainId>) -> bool {
            match spec {
                Spec::Fields(_, fields) => fields.iter().any(|f| names_domain(f, own)),
                Spec::Opt(inner, _) => names_domain(inner, own),
                Spec::Array { dims, elem } => {
                    dims.iter().any(|d| matches!(d, DimSpec::Pos(d) if own.contains(d)))
                        || names_domain(elem, own)
                }
                _ => false,
            }
        }
        if names_domain(&spec, &self.own_domains) {
            let shown = self.show(&spec.ty());
            self.error(
                codes::UNSUPPORTED,
                span,
                format!("`read()` can't size a domain named inside a `{shown}`"),
            );
            self.help("read it on its own: `let s: [domain S]Char = read()`, then the rest".to_string());
            return self.err_expr(span);
        }
        let ty = spec.ty();
        self.mk(ExprKind::Read(Box::new(spec)), ty, span)
    }

    /// `let s: [domain S]Char = read()`: the token read is the array,
    /// and its length is the size of `S`.
    fn read_own(&mut self, d: DomainId, elem: &Spec, ty: Ty, span: Span) -> Expr {
        if elem.ty() != Ty::Char {
            let shown = self.show(&ty);
            self.error(
                codes::UNSUPPORTED,
                span,
                format!("`read()` can't tell how many values a `{shown}` has"),
            );
            self.help("read the count first, name a domain of that size, and read a `[D]T`".to_string());
            return self.err_expr(span);
        }
        self.do_expr(Some(&ty.clone()), span, |lw, out| {
            let word = lw.temp(Ty::Str);
            let token = lw.mk(ExprKind::Read(Box::new(Spec::Plain(Ty::Str))), Ty::Str, span);
            out.push(Stmt::Let(word, Some(token)));
            let len = |lw: &Self| {
                lw.mk(
                    ExprKind::Builtin(Builtin::Len, vec![lw.local(word, span)]),
                    Ty::Int,
                    span,
                )
            };
            out.push(Stmt::DefDomain {
                id: d,
                lo: lw.int_lit(0, span),
                len: len(lw),
            });
            let cells = lw.temp(ty.clone());
            let blank = lw.mk(ExprKind::Fill(vec![DimSpec::Pos(d)], None), ty.clone(), span);
            out.push(Stmt::Let(cells, Some(blank)));
            let at = lw.temp(Ty::Int);
            let pos = lw.mk(
                ExprKind::Cast(Cast::Trust, Box::new(lw.local(at, span))),
                Ty::Pos(d),
                span,
            );
            let cell = Place {
                kind: PlaceKind::Index(
                    Box::new(Place {
                        kind: PlaceKind::Local(cells),
                        ty: ty.clone(),
                        span: lw.span(span),
                    }),
                    vec![pos],
                ),
                ty: Ty::Char,
                span: lw.span(span),
            };
            let byte = lw.mk(
                ExprKind::Index(Box::new(lw.local(word, span)), vec![lw.local(at, span)]),
                Ty::Char,
                span,
            );
            let lid = lw.next_loop;
            lw.next_loop += 1;
            let iter = Iter::Range {
                lo: lw.int_lit(0, span),
                hi: len(lw),
                step: None,
                rev: false,
                closed: false,
            };
            let copy = Stmt::Assign(cell, AssignOp::Set, byte);
            out.push(Stmt::For(lid, at, iter, Block { stmts: vec![copy] }));
            out.push(Stmt::Give(lw.local(cells, span)));
        })
    }

    fn apply_key(
        &mut self,
        key_ast: &ast::Expr,
        elem_expr: Expr,
        out: &mut Vec<Stmt>,
        span: Span,
    ) -> Option<Expr> {
        // `by: f` where `f` is a top-level fn / builtin: wrap as `x => f(x)`.
        if let ast::ExprKind::Name(fn_name) = &key_ast.kind {
            if self.lookup(fn_name).is_none() {
                let at = key_ast.span;
                let mk_name = |text: &str| ast::Expr {
                    kind: ast::ExprKind::Name(text.to_string()),
                    span: at,
                };
                let lambda = ast::Expr {
                    kind: ast::ExprKind::Lambda {
                        params: vec![ast::Pattern {
                            kind: ast::PatternKind::Name("x".to_string()),
                            span: at,
                        }],
                        param_tys: Vec::new(),
                        body: Box::new(ast::Expr {
                            kind: ast::ExprKind::Call {
                                callee: Box::new(mk_name(fn_name)),
                                args: vec![ast::Arg {
                                    name: None,
                                    mutable: false,
                                    value: mk_name("x"),
                                }],
                            },
                            span: at,
                        }),
                    },
                    span: at,
                };
                return self.apply_key(&lambda, elem_expr, out, span);
            }
        }
        if let ast::ExprKind::Lambda { params, body, .. } = &key_ast.kind {
            if params.len() == 1 {
                self.cx.scopes.push(HashMap::new());
                let mut pre = Vec::new();
                let slot = self.loop_var(&params[0], elem_expr.ty.clone(), &mut pre);
                out.push(Stmt::Let(slot, Some(elem_expr)));
                out.extend(pre);
                let key = self.expr(body, None);
                self.cx.scopes.pop();
                return Some(key);
            }
        }
        let fv = self.expr(key_ast, None);
        match &fv.ty {
            Ty::Fn(ps, ret) if ps.len() == 1 => {
                let ret = (**ret).clone();
                Some(self.mk(
                    ExprKind::CallExpr(Box::new(fv), vec![Arg::Value(elem_expr)]),
                    ret,
                    span,
                ))
            }
            Ty::Error => None,
            _ => {
                self.error(
                    codes::BAD_OPERANDS,
                    key_ast.span,
                    "`by:` expects a one-argument key function",
                );
                None
            }
        }
    }

    fn min_max_by(
        &mut self,
        is_max: bool,
        seq_ast: &ast::Expr,
        key_ast: &ast::Expr,
        span: Span,
    ) -> Expr {
        if let Some(values) = self.walked(seq_ast) {
            return self.min_max_by(is_max, &values, key_ast, span);
        }
        let name = if is_max { "max" } else { "min" };
        let at = self.span(span);
        let (frames, value) = match &seq_ast.kind {
            ast::ExprKind::Comprehension { body, clauses } => {
                let frames = self.open_loops(clauses);
                let value = self.expr(body, None);
                (frames, value)
            }
            _ => {
                let seq = self.expr(seq_ast, None);
                let elem = match seq.ty.seq_elem().or_else(|| match &seq.ty {
                    Ty::Array { elem, .. } | Ty::Set(elem) | Ty::HashSet(elem) => {
                        Some((**elem).clone())
                    }
                    _ => None,
                }) {
                    Some(e) => e,
                    None if seq.ty == Ty::Error => return self.err_expr(span),
                    None => {
                        let message = format!(
                            "this needs a sequence of values, not a `{}`",
                            self.show(&seq.ty)
                        );
                        self.error(codes::TYPE_MISMATCH, seq_ast.span, message);
                        return self.err_expr(span);
                    }
                };
                self.open_seq(seq, elem)
            }
        };
        let elem_ty = value.ty.clone();
        if elem_ty == Ty::Error {
            self.close_loops(frames, Vec::new());
            return self.err_expr(span);
        }
        let local = |slot: Slot, ty: &Ty| Expr {
            kind: ExprKind::Local(slot),
            ty: ty.clone(),
            span: at,
        };
        let place = |slot: Slot, ty: &Ty| Place {
            kind: PlaceKind::Local(slot),
            ty: ty.clone(),
            span: at,
        };
        let mut inner = Vec::new();
        let x_slot = self.temp(elem_ty.clone());
        inner.push(Stmt::Let(x_slot, Some(value)));
        let Some(key_expr) = self.apply_key(key_ast, local(x_slot, &elem_ty), &mut inner, span) else {
            self.close_loops(frames, Vec::new());
            return self.err_expr(span);
        };
        let key_ty = key_expr.ty.clone();
        if key_ty == Ty::Error {
            self.close_loops(frames, Vec::new());
            return self.err_expr(span);
        }
        if !self.ordered(&key_ty) {
            let message = format!("a `{}` has no order for `{name}`", self.show(&key_ty));
            self.error(codes::BAD_OPERANDS, key_ast.span, message);
            self.close_loops(frames, Vec::new());
            return self.err_expr(span);
        }
        let out_ty = Ty::opt(elem_ty.clone());
        let opt_key_ty = Ty::opt(key_ty.clone());
        let best_slot = self.temp(out_ty.clone());
        let best_key_slot = self.temp(opt_key_ty.clone());
        let k_slot = self.temp(key_ty.clone());
        inner.push(Stmt::Let(k_slot, Some(key_expr)));

        let bk_is_some = self.mk(
            ExprKind::IsSome(Box::new(local(best_key_slot, &opt_key_ty))),
            Ty::Bool,
            span,
        );
        let bk_none = self.mk(ExprKind::Unary(UnOp::Not, Box::new(bk_is_some)), Ty::Bool, span);
        let bk_val = self.mk(
            ExprKind::Unwrap(Box::new(local(best_key_slot, &opt_key_ty)), false),
            key_ty.clone(),
            span,
        );
        let k_val = local(k_slot, &key_ty);
        let better_cmp = if is_max {
            self.mk(ExprKind::Cmp(CmpOp::Lt, Box::new(bk_val), Box::new(k_val)), Ty::Bool, span)
        } else {
            self.mk(ExprKind::Cmp(CmpOp::Lt, Box::new(k_val), Box::new(bk_val)), Ty::Bool, span)
        };
        let take = self.mk(
            ExprKind::If(
                Box::new(bk_none),
                Box::new(self.mk(ExprKind::Bool(true), Ty::Bool, span)),
                Box::new(better_cmp),
            ),
            Ty::Bool,
            span,
        );
        let set_best = Stmt::Assign(
            place(best_slot, &out_ty),
            AssignOp::Set,
            self.mk(ExprKind::Some(Box::new(local(x_slot, &elem_ty))), out_ty.clone(), span),
        );
        let set_key = Stmt::Assign(
            place(best_key_slot, &opt_key_ty),
            AssignOp::Set,
            self.mk(ExprKind::Some(Box::new(local(k_slot, &key_ty))), opt_key_ty.clone(), span),
        );
        inner.push(Stmt::If(
            take,
            Block { stmts: vec![set_best, set_key] },
            Block::default(),
        ));
        let (loops, _) = self.close_loops(frames, inner);
        let mut stmts = vec![
            Stmt::Let(best_slot, Some(self.mk(ExprKind::None, out_ty.clone(), span))),
            Stmt::Let(best_key_slot, Some(self.mk(ExprKind::None, opt_key_ty, span))),
        ];
        stmts.extend(loops);
        stmts.push(Stmt::Give(local(best_slot, &out_ty)));
        self.mk(ExprKind::Do(Block { stmts }), out_ty, span)
    }

    fn min_max(&mut self, is_max: bool, args: &[ast::Arg], span: Span) -> Expr {
        let name = if is_max { "max" } else { "min" };
        if let [seq, by_arg] = args {
            if seq.name.is_none()
                && !seq.mutable
                && by_arg.name.as_ref().is_some_and(|n| n.text == "by")
                && !by_arg.mutable
            {
                return self.min_max_by(is_max, &seq.value, &by_arg.value, span);
            }
        }
        if let Some(named) = args.iter().find_map(|a| a.name.as_ref()) {
            let message = format!("`{name}` has no argument named `{}`", named.text);
            self.error(codes::UNKNOWN_ARG_NAME, named.span, message);
            return self.err_expr(span);
        }
        match args {
            [] => {
                self.error(
                    codes::WRONG_ARG_COUNT,
                    span,
                    format!("`{name}` needs something to compare"),
                );
                self.err_expr(span)
            }
            [one] => self.reduce_opt(
                if is_max { Reduce::Max } else { Reduce::Min },
                &one.value,
                false,
                span,
            ),
            _ => {
                let mut values: Vec<Expr> =
                    args.iter().map(|a| self.expr(&a.value, None)).collect();
                if values.iter().any(|v| v.ty == Ty::Error) {
                    return self.err_expr(span);
                }
                // `min(5, opt)`: a plain value among optionals makes the
                // result plain, and each `none` is skipped.
                let plain = values.iter().find(|v| !matches!(v.ty, Ty::Opt(_))).map(|v| v.ty.clone());
                if let Some(plain) = plain.filter(|plain| {
                    self.ordered(plain)
                        && values.iter().any(|v| matches!(v.ty, Ty::Opt(_)))
                        && values.iter().all(|v| v.ty == *plain || v.ty == Ty::opt(plain.clone()))
                }) {
                    let op = if is_max { Builtin::Max } else { Builtin::Min };
                    return self.do_expr(Some(&plain), span, |lw, out| {
                        let mut slots = Vec::with_capacity(values.len());
                        for value in values {
                            let slot = lw.temp(value.ty.clone());
                            slots.push((slot, value.ty.clone()));
                            out.push(Stmt::Let(slot, Some(value)));
                        }
                        let best = lw.temp(plain.clone());
                        let first = slots
                            .iter()
                            .position(|(_, ty)| *ty == plain)
                            .expect("one value is plain");
                        let start = lw.local(slots[first].0, span);
                        out.push(Stmt::Let(best, Some(start)));
                        for (i, (slot, ty)) in slots.iter().enumerate() {
                            if i == first {
                                continue;
                            }
                            let held = lw.local(*slot, span);
                            let place = Place {
                                kind: PlaceKind::Local(best),
                                ty: plain.clone(),
                                span: lw.span(span),
                            };
                            let so_far = lw.local(best, span);
                            if *ty == plain {
                                let better = lw.mk(ExprKind::Builtin(op, vec![so_far, held]), plain.clone(), span);
                                out.push(Stmt::Assign(place, AssignOp::Set, better));
                            } else {
                                let some = lw.mk(ExprKind::IsSome(Box::new(held.clone())), Ty::Bool, span);
                                let inside = lw.mk(ExprKind::Unwrap(Box::new(held), false), plain.clone(), span);
                                let better = lw.mk(ExprKind::Builtin(op, vec![so_far, inside]), plain.clone(), span);
                                out.push(Stmt::If(
                                    some,
                                    Block {
                                        stmts: vec![Stmt::Assign(place, AssignOp::Set, better)],
                                    },
                                    Block::default(),
                                ));
                            }
                        }
                        let result = lw.local(best, span);
                        out.push(Stmt::Give(result));
                    });
                }
                let ty = if values.iter().any(|v| v.ty == Ty::Float) {
                    Ty::Float
                } else {
                    values[0].ty.clone()
                };
                if let Ty::Opt(inner) = ty.clone() {
                    if !self.ordered(&inner) {
                        let message = format!("a `{}` has no order for `{name}`", self.show(&ty));
                        self.error(codes::BAD_OPERANDS, span, message);
                        return self.err_expr(span);
                    }
                    // Optional min/max: `none` is the identity element.
                    let mut acc_expr = self.coerce(
                        std::mem::replace(&mut values[0], self.err_expr(span)),
                        &ty,
                        args[0].value.span,
                    );
                    for i in 1..values.len() {
                        let rhs_expr = self.coerce(
                            std::mem::replace(&mut values[i], self.err_expr(span)),
                            &ty,
                            args[i].value.span,
                        );
                        let lhs_ty = ty.clone();
                        let inner_c = (*inner).clone();
                        acc_expr = self.do_expr(Some(&ty), span, |lw, out| {
                            let a_slot = lw.temp(lhs_ty.clone());
                            let b_slot = lw.temp(lhs_ty.clone());
                            out.push(Stmt::Let(a_slot, Some(acc_expr)));
                            out.push(Stmt::Let(b_slot, Some(rhs_expr)));
                            let a = lw.local(a_slot, span);
                            let b = lw.local(b_slot, span);
                            let a_some = lw.mk(ExprKind::IsSome(Box::new(a.clone())), Ty::Bool, span);
                            let b_some = lw.mk(ExprKind::IsSome(Box::new(b.clone())), Ty::Bool, span);
                            let av = lw.mk(ExprKind::Unwrap(Box::new(a.clone()), false), inner_c.clone(), span);
                            let bv = lw.mk(ExprKind::Unwrap(Box::new(b.clone()), false), inner_c.clone(), span);
                            let op = if is_max { Builtin::Max } else { Builtin::Min };
                            let both = lw.mk(
                                ExprKind::Some(Box::new(lw.mk(
                                    ExprKind::Builtin(op, vec![av, bv]),
                                    inner_c,
                                    span,
                                ))),
                                lhs_ty.clone(),
                                span,
                            );
                            let inner_if = lw.mk(
                                ExprKind::If(Box::new(b_some), Box::new(both), Box::new(a)),
                                lhs_ty.clone(),
                                span,
                            );
                            let res = lw.mk(
                                ExprKind::If(Box::new(a_some), Box::new(inner_if), Box::new(b)),
                                lhs_ty,
                                span,
                            );
                            out.push(Stmt::Give(res));
                        });
                    }
                    return acc_expr;
                }
                if !self.ordered(&ty) {
                    let message = format!("a `{}` has no order for `{name}`", self.show(&ty));
                    self.error(codes::BAD_OPERANDS, span, message);
                    return self.err_expr(span);
                }
                for (i, arg) in args.iter().enumerate() {
                    let value = std::mem::replace(&mut values[i], self.err_expr(span));
                    values[i] = self.coerce(value, &ty, arg.value.span);
                }
                let op = if is_max { Builtin::Max } else { Builtin::Min };
                self.mk(ExprKind::Builtin(op, values), ty, span)
            }
        }
    }

    /// `first(x for x in seq if cond)` or `first(seq)` -> `?T` (first yielded item, or `none`).
    fn lower_first(&mut self, arg: &ast::Expr, span: Span) -> Expr {
        let at = self.span(span);
        let (frames, value) = match &arg.kind {
            ast::ExprKind::Comprehension { body, clauses } => {
                let frames = self.open_loops(clauses);
                let value = self.expr(body, None);
                (frames, value)
            }
            _ => {
                let seq = self.expr(arg, None);
                let elem = match seq.ty.seq_elem().or_else(|| match &seq.ty {
                    Ty::Array { elem, .. } | Ty::Set(elem) | Ty::HashSet(elem) => Some((**elem).clone()),
                    _ => None,
                }) {
                    Some(e) => e,
                    None if seq.ty == Ty::Error => return self.err_expr(span),
                    None => {
                        let message = format!(
                            "this needs a sequence of values, not a `{}`",
                            self.show(&seq.ty)
                        );
                        self.error(codes::TYPE_MISMATCH, arg.span, message);
                        return self.err_expr(span);
                    }
                };
                self.open_seq(seq, elem)
            }
        };
        let elem_ty = value.ty.clone();
        if elem_ty == Ty::Error {
            self.close_loops(frames, Vec::new());
            return self.err_expr(span);
        }
        let res_ty = Ty::opt(elem_ty);
        let acc = self.temp(res_ty.clone());
        let mut stmts = vec![Stmt::Let(
            acc,
            Some(Expr {
                kind: ExprKind::None,
                ty: res_ty.clone(),
                span: at,
            }),
        )];
        let inner = vec![
            Stmt::Assign(
                Place {
                    kind: PlaceKind::Local(acc),
                    ty: res_ty.clone(),
                    span: at,
                },
                AssignOp::Set,
                Expr {
                    kind: ExprKind::Some(Box::new(value)),
                    ty: res_ty.clone(),
                    span: at,
                },
            ),
            Stmt::Break(None),
        ];
        let (mut loops, outer) = self.close_loops(frames, inner);
        retarget_break(&mut loops, outer, 0);
        stmts.extend(loops);
        stmts.push(Stmt::Give(Expr {
            kind: ExprKind::Local(acc),
            ty: res_ty.clone(),
            span: at,
        }));
        self.mk(ExprKind::Do(Block { stmts }), res_ty, span)
    }

    /// Lowers `e`, returning `?T` when `e` is a 1-arg `min(...)`, `max(...)`, or
    /// binary-function `xs.fold(f)` reduction (for `... ?? default` and `...!`).
    pub(super) fn expr_maybe_opt_reduce(&mut self, e: &ast::Expr) -> Expr {
        if let ast::ExprKind::Call { callee, args } = &e.kind {
            if args.len() == 1 && args[0].name.is_none() {
                if let ast::ExprKind::Name(n) = &callee.kind {
                    if (n == "min" || n == "max") && self.lookup(n).is_none() {
                        let kind = if n == "max" { Reduce::Max } else { Reduce::Min };
                        return self.reduce_opt(kind, &args[0].value, true, e.span);
                    }
                }
            }
            if let ast::ExprKind::Field { base, name } = &callee.kind {
                if args.is_empty()
                    && matches!(name.text.as_str(), "min" | "max")
                    && matches!(
                        self.probe_ty_silent(base),
                        Some(
                            Ty::List(_)
                                | Ty::ListView(_) | Ty::MutView(_)
                                | Ty::Array { .. }
                                | Ty::Range(_)
                                | Ty::Str
                                | Ty::StrView
                        )
                    )
                {
                    let kind = if name.text == "max" { Reduce::Max } else { Reduce::Min };
                    return self.reduce_opt(kind, base, true, e.span);
                }
                if args.len() == 1
                    && args[0].name.is_none()
                    && !args[0].mutable
                    && name.text == "fold"
                    && matches!(
                        self.probe_ty_silent(base),
                        Some(
                            Ty::List(_)
                                | Ty::ListView(_) | Ty::MutView(_)
                                | Ty::Array { .. }
                                | Ty::Range(_)
                                | Ty::Set(_)
                                | Ty::HashSet(_)
                                | Ty::Str
                                | Ty::StrView
                        )
                    )
                {
                    return self.fold_seq(base, &args[0], true, name.span, e.span);
                }
            }
        }
        self.expr(e, None)
    }

    fn fold_seq(
        &mut self,
        base: &ast::Expr,
        op: &ast::Arg,
        optional: bool,
        name_span: Span,
        span: Span,
    ) -> Expr {
        if let ast::ExprKind::Name(op_name) = &op.value.kind {
            if matches!(op_name.as_str(), "min" | "max") {
                let elem = self.probe_ty_silent(base).and_then(|t| t.seq_elem());
                if elem.as_ref().is_some_and(|e| !matches!(e, Ty::Int | Ty::I128 | Ty::Float)) {
                    let kind = if op_name == "min" { Reduce::Min } else { Reduce::Max };
                    return self.reduce_opt(kind, base, optional, span);
                }
            }
        }
        let use_monoid = !optional && self.detect_bare_op(&op.value).is_some();
        if !use_monoid {
            let is_binary_fn = match &op.value.kind {
                ast::ExprKind::Lambda { params, .. } => params.len() == 2,
                ast::ExprKind::Name(fn_name) if self.lookup(fn_name).is_none() => true,
                _ => matches!(
                    self.probe_ty_silent(&op.value),
                    Some(Ty::Fn(ps, _)) if ps.len() == 2
                ),
            };
            if is_binary_fn {
                return self.reduce_fold_fn(base, &op.value, optional, span);
            }
        }
        let seq = ast::Arg {
            name: None,
            mutable: false,
            value: base.clone(),
        };
        let callee = ast::Expr {
            kind: ast::ExprKind::Name("fold".into()),
            span: name_span,
        };
        let call_args = vec![op.clone(), seq];
        let whole = ast::Expr {
            kind: ast::ExprKind::Call {
                callee: Box::new(callee.clone()),
                args: call_args.clone(),
            },
            span,
        };
        self.call(&whole, &callee, &call_args, None)
    }

    fn apply_fold_fn(
        &mut self,
        fn_ast: &ast::Expr,
        acc_expr: Expr,
        cur_expr: Expr,
        out: &mut Vec<Stmt>,
        span: Span,
    ) -> Option<Expr> {
        if let ast::ExprKind::Name(fn_name) = &fn_ast.kind {
            if self.lookup(fn_name).is_none() {
                let at = fn_ast.span;
                let mk_name = |text: &str| ast::Expr {
                    kind: ast::ExprKind::Name(text.to_string()),
                    span: at,
                };
                let lambda = ast::Expr {
                    kind: ast::ExprKind::Lambda {
                        params: vec![
                            ast::Pattern {
                                kind: ast::PatternKind::Name("__fold_a".to_string()),
                                span: at,
                            },
                            ast::Pattern {
                                kind: ast::PatternKind::Name("__fold_b".to_string()),
                                span: at,
                            },
                        ],
                        param_tys: Vec::new(),
                        body: Box::new(ast::Expr {
                            kind: ast::ExprKind::Call {
                                callee: Box::new(mk_name(fn_name)),
                                args: vec![
                                    ast::Arg {
                                        name: None,
                                        mutable: false,
                                        value: mk_name("__fold_a"),
                                    },
                                    ast::Arg {
                                        name: None,
                                        mutable: false,
                                        value: mk_name("__fold_b"),
                                    },
                                ],
                            },
                            span: at,
                        }),
                    },
                    span: at,
                };
                return self.apply_fold_fn(&lambda, acc_expr, cur_expr, out, span);
            }
        }
        if let ast::ExprKind::Lambda { params, body, .. } = &fn_ast.kind {
            if params.len() == 2 {
                self.cx.scopes.push(HashMap::new());
                let mut pre = Vec::new();
                let a_slot = self.loop_var(&params[0], acc_expr.ty.clone(), &mut pre);
                out.push(Stmt::Let(a_slot, Some(acc_expr)));
                let b_slot = self.loop_var(&params[1], cur_expr.ty.clone(), &mut pre);
                out.push(Stmt::Let(b_slot, Some(cur_expr)));
                out.extend(pre);
                let res = self.expr(body, None);
                self.cx.scopes.pop();
                return Some(res);
            }
        }
        let fv = self.expr(fn_ast, None);
        match &fv.ty {
            Ty::Fn(ps, ret) if ps.len() == 2 => {
                let ret = (**ret).clone();
                Some(self.mk(
                    ExprKind::CallExpr(
                        Box::new(fv),
                        vec![Arg::Value(acc_expr), Arg::Value(cur_expr)],
                    ),
                    ret,
                    span,
                ))
            }
            Ty::Error => None,
            _ => {
                self.error(
                    codes::BAD_OPERANDS,
                    fn_ast.span,
                    "`fold` expects a monoid or a two-argument function `(T, T) -> T`",
                );
                None
            }
        }
    }

    fn reduce_fold_fn(
        &mut self,
        base: &ast::Expr,
        fn_ast: &ast::Expr,
        optional: bool,
        span: Span,
    ) -> Expr {
        if let Some(values) = self.walked(base) {
            return self.reduce_fold_fn(&values, fn_ast, optional, span);
        }
        let at = self.span(span);
        let (frames, value) = match &base.kind {
            ast::ExprKind::Comprehension { body, clauses } => {
                let frames = self.open_loops(clauses);
                let value = self.expr(body, None);
                (frames, value)
            }
            _ => {
                let seq = self.expr(base, None);
                let elem = match seq.ty.seq_elem().or_else(|| match &seq.ty {
                    Ty::Array { elem, .. } | Ty::Set(elem) | Ty::HashSet(elem) => {
                        Some((**elem).clone())
                    }
                    _ => None,
                }) {
                    Some(e) => e,
                    None if seq.ty == Ty::Error => return self.err_expr(span),
                    None => {
                        let message = format!(
                            "this needs a sequence of values, not a `{}`",
                            self.show(&seq.ty)
                        );
                        self.error(codes::TYPE_MISMATCH, base.span, message);
                        return self.err_expr(span);
                    }
                };
                self.open_seq(seq, elem)
            }
        };
        let ty = value.ty.clone();
        if ty == Ty::Error {
            self.close_loops(frames, Vec::new());
            return self.err_expr(span);
        }
        let local = |slot: Slot, t: &Ty| Expr {
            kind: ExprKind::Local(slot),
            ty: t.clone(),
            span: at,
        };
        let place = |slot: Slot, t: &Ty| Place {
            kind: PlaceKind::Local(slot),
            ty: t.clone(),
            span: at,
        };
        let lit = |kind: ExprKind, t: Ty| Expr { kind, ty: t, span: at };
        let mut stmts = Vec::new();
        let acc = self.temp(ty.clone());
        let have = self.temp(Ty::Bool);
        let cur = self.temp(ty.clone());
        stmts.push(Stmt::Let(acc, None));
        stmts.push(Stmt::Let(have, Some(lit(ExprKind::Bool(false), Ty::Bool))));

        let mut combine_stmts = Vec::new();
        let Some(combined) = self.apply_fold_fn(
            fn_ast,
            local(acc, &ty),
            local(cur, &ty),
            &mut combine_stmts,
            span,
        ) else {
            self.close_loops(frames, Vec::new());
            return self.err_expr(span);
        };
        if combined.ty == Ty::Error {
            self.close_loops(frames, Vec::new());
            return self.err_expr(span);
        }
        if combined.ty != ty {
            let message = format!(
                "`fold` expects a function returning `{}`, got `{}`",
                self.show(&ty),
                self.show(&combined.ty)
            );
            self.error(codes::TYPE_MISMATCH, fn_ast.span, message);
            self.close_loops(frames, Vec::new());
            return self.err_expr(span);
        }
        combine_stmts.push(Stmt::Assign(place(acc, &ty), AssignOp::Set, combined));

        let inner = vec![
            Stmt::Let(cur, Some(value)),
            Stmt::If(
                local(have, &Ty::Bool),
                Block { stmts: combine_stmts },
                Block {
                    stmts: vec![
                        Stmt::Assign(place(acc, &ty), AssignOp::Set, local(cur, &ty)),
                        Stmt::Assign(
                            place(have, &Ty::Bool),
                            AssignOp::Set,
                            lit(ExprKind::Bool(true), Ty::Bool),
                        ),
                    ],
                },
            ),
        ];
        let (loops, _) = self.close_loops(frames, inner);
        stmts.extend(loops);

        let (out_slot, out_ty) = if optional {
            let res_ty = Ty::opt(ty.clone());
            let out_slot = self.temp(res_ty.clone());
            stmts.push(Stmt::Let(
                out_slot,
                Some(lit(
                    ExprKind::If(
                        Box::new(local(have, &Ty::Bool)),
                        Box::new(lit(ExprKind::Some(Box::new(local(acc, &ty))), res_ty.clone())),
                        Box::new(lit(ExprKind::None, res_ty.clone())),
                    ),
                    res_ty.clone(),
                )),
            ));
            (out_slot, res_ty)
        } else {
            stmts.push(Stmt::If(
                lit(
                    ExprKind::Unary(UnOp::Not, Box::new(local(have, &Ty::Bool))),
                    Ty::Bool,
                ),
                Block {
                    stmts: vec![Stmt::Trap("`fold` of an empty sequence".into(), at)],
                },
                Block::default(),
            ));
            (acc, ty)
        };
        stmts.push(Stmt::Give(local(out_slot, &out_ty)));
        self.mk(ExprKind::Do(Block { stmts }), out_ty, span)
    }

    /// `count(xs, x => x > 2)` is `count(x > 2 for x in xs)`, and
    /// `first(xs, p)` is `first(x for x in xs if p(x))`: a test after the
    /// sequence is the body of the walk, so every fold that takes a walk
    /// takes a test the same way. Only `count`, `all`, `any` and `first`
    /// take one: they fold what the test says, where a `sum` of a
    /// function is `sum(f(x) for x in xs)`.
    fn tested_walk(&mut self, name: &str, args: &[ast::Arg], filter: bool) -> Option<ast::Expr> {
        if !matches!(name, "count" | "all" | "any" | "first") {
            return None;
        }
        let [seq, test] = args else { return None };
        if seq.name.is_some() || test.name.is_some() || !self.is_test(&test.value) {
            return None;
        }
        let at = test.value.span;
        let mk = |kind: ast::ExprKind| ast::Expr { kind, span: at };
        let (pat, body) = match &test.value.kind {
            ast::ExprKind::Lambda { params, body, .. } if params.len() == 1 => (params[0].clone(), (**body).clone()),
            _ => {
                let var = "it\u{b7}".to_string();
                let call = mk(ast::ExprKind::Call {
                    callee: Box::new(test.value.clone()),
                    args: vec![ast::Arg { name: None, mutable: false, value: mk(ast::ExprKind::Name(var.clone())) }],
                });
                (ast::Pattern { kind: ast::PatternKind::Name(var), span: at }, call)
            }
        };
        let (body, conds) = if filter {
            let ast::PatternKind::Name(var) = &pat.kind else { return None };
            (mk(ast::ExprKind::Name(var.clone())), vec![ast::Cond::Expr(body)])
        } else {
            (body, Vec::new())
        };
        Some(mk(ast::ExprKind::Comprehension {
            body: Box::new(body),
            clauses: vec![ast::Clause { pat, iter: seq.value.clone(), conds }],
        }))
    }

    /// `sum(a[v] for v in V)`, `max(xs)`, `any(...)`: a loop that folds
    /// the values into one.
    fn reduce(&mut self, kind: Reduce, arg: &ast::Expr, span: Span) -> Expr {
        self.reduce_opt(kind, arg, false, span)
    }

    /// `argmax(f(v) for v in V)` is the `v` with the largest `f(v)`: `none`
    /// when there is nothing to pick from, the first one on a tie, and a `none`
    /// among the values is skipped as `max` skips it. It is `max_by` of
    /// `std.seq` over the walk, so the two can't drift apart.
    fn arg_extreme(&mut self, max: bool, arg: &ast::Expr, span: Span) -> Expr {
        let at = arg.span;
        let mk = |kind: ast::ExprKind| ast::Expr { kind, span: at };
        let name = |text: &str| mk(ast::ExprKind::Name(text.to_string()));
        let (body, clauses) = match &arg.kind {
            ast::ExprKind::Comprehension { body, clauses } => ((**body).clone(), clauses.clone()),
            _ => {
                // `argmax(a)` of a list or an array is `a.argmax` of
                // `std.seq`: the first index of the largest value.
                let method = if max { "argmax" } else { "argmin" };
                if let Some(found) = self.try_ufcs_call(method, arg, &[], span) {
                    return found;
                }
                self.unsupported(span, "`argmax` / `argmin` of a sequence without a `for`");
                self.help("name the position: `argmax(a[i] for i in D)`");
                return self.err_expr(span);
            }
        };
        let [clause] = clauses.as_slice() else {
            self.unsupported(span, "`argmax` / `argmin` over more than one `for`");
            return self.err_expr(span);
        };
        let ast::PatternKind::Name(var) = &clause.pat.kind else {
            self.unsupported(span, "`argmax` / `argmin` over a pattern");
            return self.err_expr(span);
        };
        // Are the values optionals? Then the ones that are `none` are left out.
        let values = mk(ast::ExprKind::Comprehension {
            body: Box::new(body.clone()),
            clauses: clauses.clone(),
        });
        let (diags, pending) = (self.diags.len(), self.cx.pending.len());
        let optional = matches!(self.expr(&values, None).ty, Ty::List(elem) if matches!(*elem, Ty::Opt(_)));
        self.diags.truncate(diags);
        self.cx.pending.truncate(pending);
        let mut clause = clause.clone();
        let mut key = body;
        if optional {
            let test = mk(ast::ExprKind::Compare {
                first: Box::new(key.clone()),
                rest: vec![(CmpOp::Ne, mk(ast::ExprKind::None))],
            });
            clause.conds.push(ast::Cond::Expr(test));
            key = mk(ast::ExprKind::Unwrap(Box::new(key)));
        }
        let picks = mk(ast::ExprKind::Comprehension {
            body: Box::new(name(var)),
            clauses: vec![clause],
        });
        let by = mk(ast::ExprKind::Lambda {
            params: vec![ast::Pattern { kind: ast::PatternKind::Name(var.clone()), span: at }],
            param_tys: Vec::new(),
            body: Box::new(key),
        });
        let call = mk(ast::ExprKind::Call {
            callee: Box::new(name(if max { "max" } else { "min" })),
            args: vec![
                ast::Arg { name: None, mutable: false, value: picks },
                ast::Arg {
                    name: Some(ast::Name { text: "by".to_string(), span: at }),
                    mutable: false,
                    value: by,
                },
            ],
        });
        self.expr(&call, None)
    }

    /// O(1) closed-form arithmetic-series sum for `Range<Int>` and
    /// `(Range<Int>).step(k)`. Returns `0` for empty ranges and traps on
    /// `k <= 0` just like `step` iteration.
    fn try_range_int_sum(&mut self, arg: &ast::Expr, span: Span) -> Option<Expr> {
        let mut cur = arg;
        while let ast::ExprKind::Paren(inner) = &cur.kind {
            cur = inner;
        }
        if let ast::ExprKind::Call { callee, args } = &cur.kind {
            if args.len() == 1 && args[0].name.is_none() && !args[0].mutable {
                if let ast::ExprKind::Field { base, name } = &callee.kind {
                    if name.text == "step"
                        && matches!(self.probe_ty_silent(base), Some(Ty::Range(inner)) if *inner == Ty::Int)
                    {
                        let range_val = self.expr(base, None);
                        let step_val = self.expr(&args[0].value, Some(&Ty::Int));
                        let step_val = self.coerce(step_val, &Ty::Int, span);
                        return Some(self.range_int_sum_closed(range_val, Some(step_val), span));
                    }
                }
            }
        }
        if matches!(self.probe_ty_silent(cur), Some(Ty::Range(inner)) if *inner == Ty::Int) {
            let range_val = self.expr(cur, None);
            return Some(self.range_int_sum_closed(range_val, None, span));
        }
        None
    }

    fn range_int_sum_closed(&mut self, range_val: Expr, step_val: Option<Expr>, span: Span) -> Expr {
        let mut args = vec![range_val];
        if let Some(sv) = step_val {
            args.push(sv);
        }
        self.mk(ExprKind::Builtin(Builtin::RangeSum, args), Ty::Int, span)
    }

    fn reduce_opt(&mut self, kind: Reduce, arg: &ast::Expr, optional: bool, span: Span) -> Expr {
        if matches!(kind, Reduce::Sum) && !optional {
            if let Some(closed) = self.try_range_int_sum(arg, span) {
                return closed;
            }
        }
        if let Some(values) = self.walked(arg) {
            return self.reduce_opt(kind, &values, optional, span);
        }
        let at = self.span(span);
        let (frames, value) = match &arg.kind {
            ast::ExprKind::Comprehension { body, clauses } => {
                let frames = self.open_loops(clauses);
                let value = match kind {
                    Reduce::Count | Reduce::All | Reduce::Any => self.expect(body, &Ty::Bool),
                    _ => self.expr(body, None),
                };
                (frames, value)
            }
            _ => {
                let seq = self.expr(arg, None);
                let elem = match seq.ty.seq_elem().or_else(|| match &seq.ty {
                    Ty::Array { elem, .. } | Ty::Set(elem) | Ty::HashSet(elem) => Some((**elem).clone()),
                    _ => None,
                }) {
                    Some(e) => e,
                    None if seq.ty == Ty::Error => return self.err_expr(span),
                    None => {
                        let message = format!(
                            "this needs a sequence of values, not a `{}`",
                            self.show(&seq.ty)
                        );
                        self.error(codes::TYPE_MISMATCH, arg.span, message);
                        return self.err_expr(span);
                    }
                };
                self.open_seq(seq, elem)
            }
        };
        let local = |slot: Slot, ty: &Ty| Expr {
            kind: ExprKind::Local(slot),
            ty: ty.clone(),
            span: at,
        };
        let place = |slot: Slot, ty: &Ty| Place {
            kind: PlaceKind::Local(slot),
            ty: ty.clone(),
            span: at,
        };
        let lit = |kind: ExprKind, ty: Ty| Expr { kind, ty, span: at };
        let mut stmts = Vec::new();
        let (acc, ty, inner, after): (Slot, Ty, Vec<Stmt>, Vec<Stmt>) = match kind {
            Reduce::Sum => 'sum: {
                let ty = value.ty.clone();
                // A struct with a `zero` and a `+` adds with its own `+`,
                // from its own zero: a `Mod<P>`, a matrix, a polynomial.
                if let Ty::Struct(sid) = ty {
                    let zero = self.structs[sid].consts.get("zero").cloned();
                    if let (Some(zero), Some(add)) = (zero, self.struct_op_fn(sid, "add")) {
                        let zero = self.eval_struct_const(sid, zero, span);
                        let zero = self.coerce(zero, &ty, span);
                        let acc = self.temp(ty.clone());
                        stmts.push(Stmt::Let(acc, Some(zero)));
                        let total = lit(ExprKind::Local(acc), ty.clone());
                        let next = lit(ExprKind::Call(add, vec![Arg::Value(total), Arg::Value(value)]), ty.clone());
                        let add = Stmt::Assign(place(acc, &ty), AssignOp::Set, next);
                        break 'sum (acc, ty, vec![add], Vec::new());
                    }
                }
                let zero = match ty {
                    Ty::Int => ExprKind::Int(0),
                    Ty::I128 => ExprKind::Cast(Cast::IntToI128, Box::new(lit(ExprKind::Int(0), Ty::Int))),
                    Ty::U64 => ExprKind::Cast(Cast::IntToU64, Box::new(lit(ExprKind::Int(0), Ty::Int))),
                    Ty::Float => ExprKind::Float(0.0),
                    Ty::Error => {
                        self.close_loops(frames, Vec::new());
                        return self.err_expr(span);
                    }
                    _ => {
                        let message = format!("`sum` adds numbers, not `{}`", self.show(&ty));
                        self.error(codes::BAD_OPERANDS, arg.span, message);
                        if matches!(ty, Ty::Opt(_)) {
                            self.help("`sum` does not skip or absorb `none`: add `x ?? 0`, or filter with `if let`");
                        }
                        if ty == Ty::Str {
                            self.help("to put the pieces of text one after another, write `xs.join(\"\")`");
                        }
                        self.close_loops(frames, Vec::new());
                        return self.err_expr(span);
                    }
                };
                let acc = self.temp(ty.clone());
                stmts.push(Stmt::Let(acc, Some(lit(zero, ty.clone()))));
                let add = Stmt::Assign(place(acc, &ty), AssignOp::Binary(BinOp::Add), value);
                (acc, ty, vec![add], Vec::new())
            }
            Reduce::Prod => 'prod: {
                let ty = value.ty.clone();
                // A struct with a `zero` and a `+` adds with its own `+`,
                // from its own zero: a `Mod<P>`, a matrix, a polynomial.
                if let Ty::Struct(sid) = ty {
                    let one = self.structs[sid].consts.get("one").cloned();
                    if let (Some(one), Some(add)) = (one, self.struct_op_fn(sid, "mul")) {
                        let one = self.eval_struct_const(sid, one, span);
                        let one = self.coerce(one, &ty, span);
                        let acc = self.temp(ty.clone());
                        stmts.push(Stmt::Let(acc, Some(one)));
                        let total = lit(ExprKind::Local(acc), ty.clone());
                        let next = lit(ExprKind::Call(add, vec![Arg::Value(total), Arg::Value(value)]), ty.clone());
                        let add = Stmt::Assign(place(acc, &ty), AssignOp::Set, next);
                        break 'prod (acc, ty, vec![add], Vec::new());
                    }
                }
                let one = match ty {
                    Ty::Int => ExprKind::Int(1),
                    Ty::I128 => ExprKind::Cast(Cast::IntToI128, Box::new(lit(ExprKind::Int(1), Ty::Int))),
                    Ty::U64 => ExprKind::Cast(Cast::IntToU64, Box::new(lit(ExprKind::Int(1), Ty::Int))),
                    Ty::Float => ExprKind::Float(1.0),
                    Ty::Error => {
                        self.close_loops(frames, Vec::new());
                        return self.err_expr(span);
                    }
                    _ => {
                        let message = format!("`prod` multiplies numbers, not `{}`", self.show(&ty));
                        self.error(codes::BAD_OPERANDS, arg.span, message);
                        if matches!(ty, Ty::Opt(_)) {
                            self.help("`prod` does not skip or absorb `none`: add `x ?? 1`, or filter with `if let`");
                        }
                        self.close_loops(frames, Vec::new());
                        return self.err_expr(span);
                    }
                };
                let acc = self.temp(ty.clone());
                stmts.push(Stmt::Let(acc, Some(lit(one, ty.clone()))));
                let add = Stmt::Assign(place(acc, &ty), AssignOp::Binary(BinOp::Mul), value);
                (acc, ty, vec![add], Vec::new())
            }
            Reduce::Count => {
                let acc = self.temp(Ty::Int);
                stmts.push(Stmt::Let(acc, Some(lit(ExprKind::Int(0), Ty::Int))));
                let add = Stmt::Assign(
                    place(acc, &Ty::Int),
                    AssignOp::Binary(BinOp::Add),
                    lit(ExprKind::Int(1), Ty::Int),
                );
                let test = Stmt::If(value, Block { stmts: vec![add] }, Block::default());
                (acc, Ty::Int, vec![test], Vec::new())
            }
            Reduce::All | Reduce::Any => {
                let all = kind == Reduce::All;
                let acc = self.temp(Ty::Bool);
                stmts.push(Stmt::Let(acc, Some(lit(ExprKind::Bool(all), Ty::Bool))));
                let set = Stmt::Assign(
                    place(acc, &Ty::Bool),
                    AssignOp::Set,
                    lit(ExprKind::Bool(!all), Ty::Bool),
                );
                // Leave every loop at the first value that decides it.
                let test = if all {
                    lit(ExprKind::Unary(UnOp::Not, Box::new(value)), Ty::Bool)
                } else {
                    value
                };
                (
                    acc,
                    Ty::Bool,
                    vec![Stmt::If(
                        test,
                        Block {
                            stmts: vec![set, Stmt::Break(None)],
                        },
                        Block::default(),
                    )],
                    Vec::new(),
                )
            }
            Reduce::Min | Reduce::Max => {
                let ty = value.ty.clone();
                if ty == Ty::Error {
                    self.close_loops(frames, Vec::new());
                    return self.err_expr(span);
                }
                // Over optionals `none` is skipped, as `max=` skips it, and
                // the answer is `none` when nothing is left.
                let (ty, held) = match ty {
                    Ty::Opt(inner) => {
                        let held = self.temp(Ty::Opt(inner.clone()));
                        (*inner, Some(held))
                    }
                    ty => (ty, None),
                };
                let optional = optional || held.is_some();
                if matches!(ty, Ty::Opt(_)) || !self.ordered(&ty) {
                    let message = format!("a `{}` has no order to compare by", self.show(&ty));
                    self.error(codes::BAD_OPERANDS, arg.span, message);
                    self.close_loops(frames, Vec::new());
                    return self.err_expr(span);
                }
                let acc = self.temp(ty.clone());
                let have = self.temp(Ty::Bool);
                let cur = self.temp(ty.clone());
                stmts.push(Stmt::Let(acc, None));
                stmts.push(Stmt::Let(have, Some(lit(ExprKind::Bool(false), Ty::Bool))));
                let op = if kind == Reduce::Max {
                    AssignOp::Max
                } else {
                    AssignOp::Min
                };
                let (read, value) = match held {
                    Some(held) => {
                        let opt = Ty::opt(ty.clone());
                        let inside = ExprKind::Unwrap(Box::new(local(held, &opt)), false);
                        (Some((held, value, opt)), lit(inside, ty.clone()))
                    }
                    None => (None, value),
                };
                let inner = vec![
                    Stmt::Let(cur, Some(value)),
                    Stmt::If(
                        local(have, &Ty::Bool),
                        Block {
                            stmts: vec![Stmt::Assign(place(acc, &ty), op, local(cur, &ty))],
                        },
                        Block {
                            stmts: vec![
                                Stmt::Assign(place(acc, &ty), AssignOp::Set, local(cur, &ty)),
                                Stmt::Assign(
                                    place(have, &Ty::Bool),
                                    AssignOp::Set,
                                    lit(ExprKind::Bool(true), Ty::Bool),
                                ),
                            ],
                        },
                    ),
                ];
                let inner = match read {
                    Some((held, value, opt)) => vec![
                        Stmt::Let(held, Some(value)),
                        Stmt::If(
                            lit(ExprKind::IsSome(Box::new(local(held, &opt))), Ty::Bool),
                            Block { stmts: inner },
                            Block::default(),
                        ),
                    ],
                    None => inner,
                };
                if optional {
                    let res_ty = Ty::opt(ty.clone());
                    let out_slot = self.temp(res_ty.clone());
                    let choose = Stmt::Let(
                        out_slot,
                        Some(lit(
                            ExprKind::If(
                                Box::new(local(have, &Ty::Bool)),
                                Box::new(lit(ExprKind::Some(Box::new(local(acc, &ty))), res_ty.clone())),
                                Box::new(lit(ExprKind::None, res_ty.clone())),
                            ),
                            res_ty.clone(),
                        )),
                    );
                    (out_slot, res_ty, inner, vec![choose])
                } else {
                    let word = if kind == Reduce::Max { "max" } else { "min" };
                    let empty = Stmt::If(
                        lit(
                            ExprKind::Unary(UnOp::Not, Box::new(local(have, &Ty::Bool))),
                            Ty::Bool,
                        ),
                        Block {
                            stmts: vec![Stmt::Trap(format!("`{word}` of an empty sequence"), at)],
                        },
                        Block::default(),
                    );
                    (acc, ty, inner, vec![empty])
                }
            }
        };
        let (mut loops, outer) = self.close_loops(frames, inner);
        if matches!(kind, Reduce::All | Reduce::Any) {
            retarget_break(&mut loops, outer, 0);
        }
        stmts.extend(loops);
        stmts.extend(after);
        stmts.push(Stmt::Give(local(acc, &ty)));
        self.mk(ExprKind::Do(Block { stmts }), ty, span)
    }

    // ----- members -----

    /// `base.name` that is not being called.
    /// A number a generic struct was given, read off the type by the
    /// parameter's name: with `struct Mod<P>` and `type Z = Mod<7>`,
    /// `Z.P` is 7.
    fn struct_number_param(&mut self, sid: StructId, name: &str, span: Span) -> Option<Expr> {
        let (gid, gargs) = self.structs[sid].origin_gen.clone()?;
        let at = self.gen_structs[gid].decl.generics.iter().position(|p| p.text == name)?;
        match gargs.get(at)? {
            GenArg::Int(value) => Some(self.int_lit(*value, span)),
            GenArg::RuntimeInt(id) => Some(self.mk(ExprKind::Global(*id), Ty::Int, span)),
            _ => None,
        }
    }

    pub(super) fn field(
        &mut self,
        whole: &ast::Expr,
        base: &ast::Expr,
        name: &ast::Name,
        hint: Option<&Ty>,
    ) -> Expr {
        let span = whole.span;
        if matches!(name.text.as_str(), "first" | "last" | "len" | "is_empty")
            && self.is_sorted_range_expr(base)
        {
            return self.sorted_view_field(base, &name.text, span);
        }
        if let ast::ExprKind::Field { name: bounds, .. } = &base.kind {
            if bounds.text == "bounds" && matches!(name.text.as_str(), "first" | "last") {
                let cuts = self.expr(base, None);
                if let (ExprKind::Range { lo, hi }, Ty::Range(cut)) = (&cuts.kind, &cuts.ty) {
                    let end = if name.text == "first" {
                        (**lo).clone()
                    } else {
                        self.int_add((**hi).clone(), -1)
                    };
                    return self.cast(Cast::Trust, end, (**cut).clone());
                }
            }
        }
        // Handle `GenericBase<...>.member` (e.g., `Mod<7>.zero`, `Sum<Int>.id`, `Less<Int>.rev`).
        if let ast::ExprKind::Generic { base: gbase, args: gtypes } = &base.kind {
            if let ast::ExprKind::Name(gname) = &gbase.kind {
                let lookup = prefix_shape_struct(gname, gtypes.len()).unwrap_or(gname);
                if let Some(Sym::GenStruct(gid)) = self.get_sym(lookup) {
                    let gargs = self.resolve_gen_args(gtypes);
                    let sid = self.instantiate_struct(gid, gargs);
                    if let Some(sc) = self.structs[sid].consts.get(&name.text).cloned() {
                        return self.eval_struct_const(sid, sc, span);
                    }
                    if let Some(&fn_id) = self.structs[sid].methods.get(&name.text) {
                        return self.method_call(fn_id, None, &[], span);
                    }
                    if let Some(number) = self.struct_number_param(sid, &name.text, span) {
                        return number;
                    }
                    if name.text == "K" {
                        if let Some(d) = self.structs[sid].index_domain {
                            return self.mk(ExprKind::DomainLen(d), Ty::Int, span);
                        }
                    }
                }
            }
        }
        // `Subset<D>.full` / `Subset<D>.empty` / `Subset<D>.len`, etc.
        if let ast::ExprKind::Generic { base: gb, args: gargs } = &base.kind {
            if matches!(&gb.kind, ast::ExprKind::Name(n) if n == "Subset") && gargs.len() == 1 {
                if let Ty::Pos(d) = self.resolve_ty(&gargs[0]) {
                    let sub = self.subset_domain(d);
                    return self.domain_field(sub, name, hint, span);
                }
            }
        }
        // Local type alias pointing to a domain (e.g. `type Group = Subset<Rabbit>` inside a block).
        if let ast::ExprKind::Name(n) = &base.kind {
            if self.lookup(n).is_none() {
                if let Some(&Ty::Pos(id)) = self.cx.local_type_aliases.get(n) {
                    return self.domain_field(id, name, hint, span);
                }
            }
        }
        // Generic parameter bound to a `.bounds` domain (`D = V.bounds`):
        // `D.K` / `D.len` / `D.count` is `V.K + 1`, `D.first` is `0`, `D.last` is `V.K`.
        if let ast::ExprKind::Name(n) = &base.kind {
            if self.lookup(n).is_none() {
                if let Some(GenArg::Ty(Ty::Bound(d))) = self.gen_env.get(n).cloned() {
                    match name.text.as_str() {
                        "K" | "len" | "size" | "count" => {
                            let len = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
                            return self.int_add(len, 1);
                        }
                        "first" => {
                            let zero = self.int_lit(0, span);
                            return self.cast(Cast::Trust, zero, Ty::Bound(d));
                        }
                        "last" => {
                            let len = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
                            return self.cast(Cast::Trust, len, Ty::Bound(d));
                        }
                        _ => {}
                    }
                }
            }
        }
        match self.sym(base) {
            Some(Sym::Mod(target_mod)) => {
                let sym = self.mod_syms[target_mod].get(&name.text).copied();
                match sym {
                    Some(Sym::ModConst(mcid)) => return self.ensure_mod_const(mcid, span),
                    Some(Sym::Global(id)) => {
                        return self.mk(ExprKind::Global(id), self.globals[id].ty.clone(), span);
                    }
                    Some(Sym::Fn(id)) => return self.fn_as_closure(id, span),
                    _ => {
                        self.error(
                            codes::UNKNOWN_FIELD,
                            name.span,
                            format!("module has no member `{}`", name.text),
                        );
                        return self.err_expr(span);
                    }
                }
            }
            Some(Sym::Struct(id)) => {
                if name.text == "K" {
                    if let Some(d) = self.structs[id].index_domain {
                        return self.mk(ExprKind::DomainLen(d), Ty::Int, span);
                    }
                }
                if let Some(sc) = self.structs[id].consts.get(&name.text).cloned() {
                    return self.eval_struct_const(id, sc, span);
                }
                if let Some(&fn_id) = self.structs[id].methods.get(&name.text) {
                    return self.method_call(fn_id, None, &[], span);
                }
                if let Some(number) = self.struct_number_param(id, &name.text, span) {
                    return number;
                }
            }
            Some(Sym::Alias(aid)) => {
                let (m, ty_ast, offset) = self.aliases[aid];
                let sm = std::mem::replace(&mut self.cur_mod, m);
                let so = std::mem::replace(&mut self.offset, offset);
                let resolved = self.resolve_ty(ty_ast);
                self.cur_mod = sm;
                self.offset = so;
                match resolved {
                    Ty::Struct(id) => {
                        if let Some(sc) = self.structs[id].consts.get(&name.text).cloned() {
                            return self.eval_struct_const(id, sc, span);
                        }
                        if let Some(&fn_id) = self.structs[id].methods.get(&name.text) {
                            return self.method_call(fn_id, None, &[], span);
                        }
                        if let Some(number) = self.struct_number_param(id, &name.text, span) {
                            return number;
                        }
                    }
                    Ty::Pos(id) => return self.domain_field(id, name, hint, span),
                    _ => {}
                }
            }
            Some(Sym::GenStruct(gid)) => {
                if let Some(Ty::Struct(sid)) = hint {
                    if self.structs[*sid]
                        .origin_gen
                        .as_ref()
                        .is_some_and(|(g, _)| *g == gid)
                    {
                        if let Some(sc) = self.structs[*sid].consts.get(&name.text).cloned() {
                            return self.eval_struct_const(*sid, sc, span);
                        }
                        if self.structs[*sid].getters.contains(&name.text) {
                            if let Some(&fid) = self.structs[*sid].methods.get(&name.text) {
                                self.queue_fn(fid);
                                let ret = self.ret_of(fid, span);
                                return self.mk(ExprKind::Call(fid, Vec::new()), ret, span);
                            }
                        }
                    }
                }
            }
            Some(Sym::Domain(id)) => return self.domain_field(id, name, hint, span),
            Some(Sym::Enum(id)) => {
                return match self.enums[id]
                    .variants
                    .iter()
                    .position(|(v, _)| *v == name.text)
                {
                    Some(tag) if self.enums[id].variants[tag].1.is_empty() => {
                        self.mk(ExprKind::Variant(tag, Vec::new()), Ty::Enum(id), span)
                    }
                    Some(_) => {
                        let message = format!(
                            "the variant `{}` holds values: write `{}(...)`",
                            name.text, name.text
                        );
                        self.error(codes::WRONG_ARG_COUNT, span, message);
                        self.err_expr(span)
                    }
                    None => {
                        let message =
                            format!("`{}` has no variant `{}`", self.enums[id].name, name.text);
                        self.error(codes::UNKNOWN_FIELD, name.span, message);
                        self.err_expr(span)
                    }
                };
            }
            _ => {}
        }
        if let ast::ExprKind::Name(type_name) = &base.kind {
            if self.lookup(type_name).is_none() {
                if type_name == "Int"
                    || matches!(self.gen_env.get(type_name), Some(GenArg::Ty(Ty::Int)))
                {
                    match name.text.as_str() {
                        "MAX" => return self.mk(ExprKind::Int(i64::MAX), Ty::Int, span),
                        "MIN" => return self.mk(ExprKind::Int(INT_MIN), Ty::Int, span),
                        "zero" => return self.mk(ExprKind::Int(0), Ty::Int, span),
                        "one" => return self.mk(ExprKind::Int(1), Ty::Int, span),
                        _ => {}
                    }
                }
                // `T.zero` and `T.one` where `T` is one of the wide integers.
                let wide = match (type_name.as_str(), self.gen_env.get(type_name)) {
                    ("U64", _) | (_, Some(GenArg::Ty(Ty::U64))) => Some((Cast::IntToU64, Ty::U64)),
                    ("I128", _) | (_, Some(GenArg::Ty(Ty::I128))) => Some((Cast::IntToI128, Ty::I128)),
                    _ => None,
                };
                if let (Some((cast, ty)), Some(unit)) = (wide, ["zero", "one"].iter().position(|c| *c == name.text)) {
                    let unit = self.mk(ExprKind::Int(unit as i64), Ty::Int, span);
                    return self.mk(ExprKind::Cast(cast, Box::new(unit)), ty, span);
                }
                if let Some(GenArg::Ty(Ty::Struct(sid))) = self.gen_env.get(type_name).cloned() {
                    if let Some(sc) = self.structs[sid].consts.get(&name.text).cloned() {
                        return self.eval_struct_const(sid, sc, span);
                    }
                    if let Some(&fn_id) = self.structs[sid].methods.get(&name.text) {
                        return self.method_call(fn_id, None, &[], span);
                    }
                }
                if let Some(GenArg::Ty(Ty::Opt(inner))) = self.gen_env.get(type_name).cloned() {
                    if matches!(name.text.as_str(), "MAX" | "MIN") {
                        return self.mk(ExprKind::None, Ty::Opt(inner), span);
                    }
                }
                if type_name == "F64"
                    || matches!(self.gen_env.get(type_name), Some(GenArg::Ty(Ty::Float)))
                {
                    match name.text.as_str() {
                        "zero" => return self.mk(ExprKind::Float(0.0), Ty::Float, span),
                        "one" => return self.mk(ExprKind::Float(1.0), Ty::Float, span),
                        _ => {}
                    }
                }
            }
        }
        let value = self.expr(base, None);
        if let Some((index, ty)) = self.field_of(&value.ty, &name.text) {
            return self.mk(ExprKind::Field(Box::new(value), index), ty, span);
        }
        let ty = value.ty.clone();
        let builtin = |lw: &mut Self, op: Builtin, args: Vec<Expr>, ty: Ty| {
            lw.mk(ExprKind::Builtin(op, args), ty, span)
        };
        match (&ty, name.text.as_str()) {
            (Ty::Error, _) => value,
            (Ty::Struct(id), method) if self.structs[*id].methods.contains_key(method) => {
                let fn_id = self.structs[*id].methods[method];
                if self.structs[*id].getters.contains(method)
                    || self.sigs[fn_id].params.is_empty()
                {
                    self.method_call(fn_id, Some((base, value)), &[], span)
                } else {
                    self.bind_method_closure(fn_id, value, span)
                }
            }
            (Ty::Pos(_) | Ty::Bound(_), "ord" | "to_idx" | "idx") => {
                self.cast(Cast::Ord, value, Ty::Int)
            }
            (Ty::Pos(d), "label") => self.cast(Cast::Label(*d), value, Ty::Int),
            (Ty::Pos(_), "rank") => {
                let ord = self.cast(Cast::Ord, value, Ty::Int);
                self.int_add(ord, 1)
            }
            (Ty::Pos(d), "next") => {
                let len = self.mk(ExprKind::DomainLen(*d), Ty::Int, span);
                builtin(
                    self,
                    Builtin::PosNext,
                    vec![value, len],
                    Ty::opt(ty.clone()),
                )
            }
            (Ty::Pos(_), "prev") => {
                builtin(self, Builtin::PosPrev, vec![value], Ty::opt(ty.clone()))
            }
            (Ty::Pos(d), "before" | "l") => self.cast(Cast::Trust, value, Ty::Bound(*d)),
            (Ty::Pos(d), "after" | "r") => {
                let ord = self.cast(Cast::Ord, value, Ty::Int);
                let next = self.int_add(ord, 1);
                self.cast(Cast::Trust, next, Ty::Bound(*d))
            }
            (Ty::Pos(d), field) if self.product_domains.contains_key(d) => {
                let d = *d;
                let factors = self.product_domains[&d].clone();
                if let Some(idx) = factors
                    .iter()
                    .position(|(fnm, _)| fnm.as_deref() == Some(field))
                    .or_else(|| field.parse::<usize>().ok().filter(|&i| i < factors.len()))
                {
                    self.decode_product_field(d, value, idx, span)
                } else {
                    self.error(
                        codes::UNKNOWN_FIELD,
                        name.span,
                        format!("`{}` has no field `{}`", self.show(&ty), field),
                    );
                    self.err_expr(span)
                }
            }
            (Ty::Bound(d), "left") => {
                builtin(self, Builtin::PosPrev, vec![value], Ty::opt(Ty::Pos(*d)))
            }
            (Ty::Bound(d), "right") => {
                let ord = self.cast(Cast::Ord, value, Ty::Int);
                let before = self.int_add(ord, -1);
                let len = self.mk(ExprKind::DomainLen(*d), Ty::Int, span);
                builtin(
                    self,
                    Builtin::PosNext,
                    vec![before, len],
                    Ty::opt(Ty::Pos(*d)),
                )
            }
            (Ty::Range(inner), "start") => {
                let inner = (**inner).clone();
                let res_ty = if inner == Ty::I128 { Ty::I128 } else { Ty::Int };
                let lo = builtin(self, Builtin::RangeLo, vec![value], res_ty);
                match inner {
                    Ty::Pos(d) | Ty::Bound(d) => self.cast(Cast::Trust, lo, Ty::Bound(d)),
                    _ => lo,
                }
            }
            (Ty::Range(inner), "end") => {
                let inner = (**inner).clone();
                let res_ty = if inner == Ty::I128 { Ty::I128 } else { Ty::Int };
                let hi = builtin(self, Builtin::RangeHi, vec![value], res_ty);
                match inner {
                    Ty::Pos(d) | Ty::Bound(d) => self.cast(Cast::Trust, hi, Ty::Bound(d)),
                    _ => hi,
                }
            }
            (Ty::Struct(id), field_name)
                if matches!(
                    self.structs[*id].env.get(field_name),
                    Some(GenArg::Domain(_))
                ) =>
            {
                let Some(GenArg::Domain(d)) = self.structs[*id].env.get(field_name).cloned() else {
                    unreachable!()
                };
                self.mk(ExprKind::DomainVal(d), Ty::DomainVal, span)
            }
            (Ty::Range(inner), "mid") => {
                let inner_ty = (**inner).clone();
                let res_ty = Ty::opt(inner_ty.clone());
                let r_ty = value.ty.clone();
                self.do_expr(Some(&res_ty.clone()), span, |lw, out| {
                    let r_slot = lw.temp(r_ty);
                    out.push(Stmt::Let(r_slot, Some(value)));
                    let r_ref = lw.local(r_slot, span);
                    let len = lw.mk(ExprKind::Builtin(Builtin::Len, vec![r_ref.clone()]), Ty::Int, span);
                    let empty = lw.mk(
                        ExprKind::Cmp(CmpOp::Eq, Box::new(len.clone()), Box::new(lw.int_lit(0, span))),
                        Ty::Bool,
                        span,
                    );
                    let lo = lw.mk(ExprKind::Builtin(Builtin::RangeLo, vec![r_ref]), Ty::Int, span);
                    let half = lw.mk(
                        ExprKind::Binary(
                            BinOp::Div,
                            Box::new(lw.int_add(len, -1)),
                            Box::new(lw.int_lit(2, span)),
                        ),
                        Ty::Int,
                        span,
                    );
                    let mid_int = lw.mk(
                        ExprKind::Binary(BinOp::Add, Box::new(lo), Box::new(half)),
                        Ty::Int,
                        span,
                    );
                    let mid_val = match inner_ty {
                        Ty::Pos(d) => lw.mk(ExprKind::Cast(Cast::FromOrd(d), Box::new(mid_int)), Ty::Pos(d), span),
                        _ => mid_int,
                    };
                    let res = lw.mk(
                        ExprKind::If(
                            Box::new(empty),
                            Box::new(lw.mk(ExprKind::None, res_ty.clone(), span)),
                            Box::new(lw.mk(ExprKind::Some(Box::new(mid_val)), res_ty.clone(), span)),
                        ),
                        res_ty,
                        span,
                    );
                    out.push(Stmt::Give(res));
                })
            }
            (Ty::Range(inner), "first") => {
                let inner = (**inner).clone();
                builtin(self, Builtin::First, vec![value], Ty::opt(inner))
            }
            (Ty::Range(inner), "last") => {
                let inner = (**inner).clone();
                builtin(self, Builtin::Last, vec![value], Ty::opt(inner))
            }
            (Ty::DomainVal, "len" | "size" | "K" | "count") => {
                // If this is `xs.D.K` where `xs` is a struct holding a `[D]`-indexed array field,
                // read the instance's own array length so multiple instances of `compress` don't alias one global domain length.
                if let ast::ExprKind::Field { base: owner_ast, name: dom_field } = &base.kind {
                    if let Some(Ty::Struct(sid)) = self.probe_ty_silent(owner_ast) {
                        if let Some(GenArg::Domain(d)) = self.structs[sid].env.get(&dom_field.text).cloned() {
                            if let Some((fidx, fty)) = self.structs[sid]
                                .def
                                .fields
                                .iter()
                                .enumerate()
                                .find(|(_, (_, ty))| {
                                    matches!(ty, Ty::Array { dims, .. } if dims.first() == Some(&Dim::Pos(d)))
                                })
                                .map(|(i, (_, ty))| (i, ty.clone()))
                            {
                                let owner_val = self.expr(owner_ast, None);
                                let arr = self.mk(ExprKind::Field(Box::new(owner_val), fidx), fty, span);
                                return builtin(self, Builtin::Len, vec![arr], Ty::Int);
                            }
                        }
                    }
                }
                builtin(self, Builtin::Len, vec![value], Ty::Int)
            }
            (
                Ty::Array { .. }
                | Ty::List(_)
                | Ty::ListView(_) | Ty::MutView(_)
                | Ty::Str
                | Ty::StrView
                | Ty::Map(..)
                | Ty::HashMap(..)
                | Ty::Set(_)
                | Ty::HashSet(_)
                | Ty::Range(_),
                "len",
            ) => self.len_of(value, span),
            (Ty::Range(inner), "is_empty") => {
                let bound_ty = if **inner == Ty::I128 { Ty::I128 } else { Ty::Int };
                let r_ref = if matches!(value.kind, ExprKind::Local(_)) {
                    value
                } else {
                    let slot = self.new_slot("r", value.ty.clone());
                    self.cx.pending.push(Stmt::Let(slot, Some(value)));
                    self.local(slot, span)
                };
                let lo = builtin(self, Builtin::RangeLo, vec![r_ref.clone()], bound_ty.clone());
                let hi = builtin(self, Builtin::RangeHi, vec![r_ref], bound_ty);
                self.mk(ExprKind::Cmp(CmpOp::Ge, Box::new(lo), Box::new(hi)), Ty::Bool, span)
            }
            (
                Ty::Array { .. }
                | Ty::List(_)
                | Ty::ListView(_) | Ty::MutView(_)
                | Ty::Str
                | Ty::StrView
                | Ty::Map(..)
                | Ty::HashMap(..)
                | Ty::Set(_)
                | Ty::HashSet(_),
                "is_empty",
            ) => {
                let len = self.len_of(value, span);
                let zero = self.int_lit(0, span);
                self.mk(
                    ExprKind::Cmp(CmpOp::Eq, Box::new(len), Box::new(zero)),
                    Ty::Bool,
                    span,
                )
            }
            // The ends of a sequence are optionals, `none` when it is
            // empty: a list, and an array over one domain or count.
            (Ty::Array { dims, elem }, "first" | "last") if dims.len() == 1 => {
                let opt = Ty::opt((**elem).clone());
                let end = if name.text == "last" { Builtin::Last } else { Builtin::First };
                builtin(self, end, vec![value], opt)
            }
            // Of a grid they are `g[R.first, C.first]` and `g[R.last, C.last]`.
            (Ty::Array { dims, elem }, "first" | "last")
                if dims.iter().all(|d| matches!(d, Dim::Pos(_) | Dim::Bound(_))) =>
            {
                let last = name.text == "last";
                let mut filled = self.mk(ExprKind::Bool(true), Ty::Bool, span);
                let mut indices = Vec::new();
                for dim in dims {
                    let (d, extra, ty) = match dim {
                        Dim::Bound(d) => (*d, 0, Ty::Bound(*d)),
                        Dim::Pos(d) => (*d, -1, Ty::Pos(*d)),
                        _ => unreachable!("guarded above"),
                    };
                    let len = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
                    let zero = self.int_lit(0, span);
                    let some = self.mk(
                        ExprKind::Cmp(CmpOp::Gt, Box::new(len.clone()), Box::new(zero.clone())),
                        Ty::Bool,
                        span,
                    );
                    filled = self.mk(
                        ExprKind::Binary(BinOp::And, Box::new(filled), Box::new(some)),
                        Ty::Bool,
                        span,
                    );
                    let at = if last { self.int_add(len, extra) } else { zero };
                    indices.push(self.mk(ExprKind::Cast(Cast::Trust, Box::new(at)), ty, span));
                }
                let opt = Ty::opt((**elem).clone());
                let at = self.mk(ExprKind::Index(Box::new(value), indices), (**elem).clone(), span);
                let some = self.mk(ExprKind::Some(Box::new(at)), opt.clone(), span);
                let none = self.mk(ExprKind::None, opt.clone(), span);
                self.mk(ExprKind::If(Box::new(filled), Box::new(some), Box::new(none)), opt, span)
            }
            // `xs.items` kept as a value: the list of `(index, element)`.
            (Ty::Map(..) | Ty::HashMap(..), "items") => self.pairs_of_map(value, span),
            (Ty::Map(..) | Ty::HashMap(..), "keys") => self.proj_of_map(value, 0, span),
            (Ty::Map(..) | Ty::HashMap(..), "values") => self.proj_of_map(value, 1, span),
            (Ty::Array { dims, .. }, "items")
                if dims.len() > 1 && dims.iter().all(|d| !matches!(d, Dim::Bound(_))) =>
            {
                self.cells_of_grid(value, span)
            }
            (Ty::List(_) | Ty::ListView(_) | Ty::MutView(_) | Ty::Array { .. } | Ty::Str | Ty::StrView, "items") if self.items_of(&value, span).is_some() => {
                let pair = |items: &Items| Ty::Tuple(vec![items.index.clone(), items.elem.clone()]);
                let held_ty = value.ty.clone();
                let Some(shape) = self.items_of(&value, span) else { unreachable!("checked above") };
                let list_ty = Ty::List(Box::new(pair(&shape)));
                self.do_expr(Some(&list_ty.clone()), span, |lw, out| {
                    let held = lw.temp(held_ty);
                    out.push(Stmt::Let(held, Some(value)));
                    let seq = lw.local(held, span);
                    let items = lw.items_of(&seq, span).expect("the same type");
                    let acc = lw.temp(list_ty.clone());
                    out.push(Stmt::Let(acc, Some(lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span))));
                    let k = lw.temp(Ty::Int);
                    let (at, cell) = lw.item_at(&items, k, span);
                    let both = lw.mk(ExprKind::Tuple(vec![at, cell]), pair(&items), span);
                    let place = Place { kind: PlaceKind::Local(acc), ty: list_ty.clone(), span: lw.span(span) };
                    let push = lw.mk(ExprKind::Mutate(Mutation::Push, Box::new(place), vec![both]), Ty::Unit, span);
                    let lid = lw.fresh_loop();
                    out.push(Stmt::For(lid, k, items.range(), Block { stmts: vec![Stmt::Expr(push)] }));
                    out.push(Stmt::Give(lw.local(acc, span)));
                })
            }
            (Ty::List(elem) | Ty::ListView(elem) | Ty::MutView(elem), "first") => {
                let opt = Ty::opt((**elem).clone());
                builtin(self, Builtin::First, vec![value], opt)
            }
            (Ty::List(elem) | Ty::ListView(elem) | Ty::MutView(elem), "last") => {
                let opt = Ty::opt((**elem).clone());
                builtin(self, Builtin::Last, vec![value], opt)
            }
            (Ty::Str | Ty::StrView, "first") => {
                builtin(self, Builtin::First, vec![value], Ty::opt(Ty::Char))
            }
            (Ty::Str | Ty::StrView, "last") => {
                builtin(self, Builtin::Last, vec![value], Ty::opt(Ty::Char))
            }
            (Ty::Str | Ty::StrView, "rev") => {
                let value = self.hoist_view_base(value, span);
                builtin(self, Builtin::Reversed, vec![value], Ty::StrView)
            }
            (Ty::List(elem) | Ty::ListView(elem) | Ty::MutView(elem), "rev") => {
                let elem = (**elem).clone();
                let value = self.hoist_view_base(value, span);
                builtin(self, Builtin::Reversed, vec![value], Ty::ListView(Box::new(elem)))
            }
            (Ty::Array { dims, elem }, "rev") if dims.len() == 1 => {
                let elem = (**elem).clone();
                let value = self.hoist_view_base(value, span);
                builtin(self, Builtin::Reversed, vec![value], Ty::ListView(Box::new(elem)))
            }
            (Ty::Range(_), "rev") => self.reversed_range(value, span),
            (Ty::List(_) | Ty::ListView(_) | Ty::MutView(_) | Ty::Str | Ty::StrView, "adjacent") => {
                self.lower_adjacent(value, span)
            }
            (Ty::Array { dims, .. }, "adjacent") if dims.len() == 1 => {
                self.lower_adjacent(value, span)
            }
            (Ty::Struct(sid), "len" | "distinct") if self.counter_key.contains_key(sid) => {
                let map_ty = self.structs[*sid].def.fields[0].1.clone();
                let map_expr = self.mk(ExprKind::Field(Box::new(value), 0), map_ty, span);
                self.len_of(map_expr, span)
            }
            (Ty::Char, "ord") => self.cast(Cast::CharToInt, value, Ty::Int),
            (Ty::Char, "is_digit") => {
                let ord = self.cast(Cast::CharToInt, value, Ty::Int);
                let lo = self.int_lit(b'0' as i64, span);
                let hi = self.int_lit(b'9' as i64, span);
                let ge = self.mk(ExprKind::Cmp(CmpOp::Ge, Box::new(ord.clone()), Box::new(lo)), Ty::Bool, span);
                let le = self.mk(ExprKind::Cmp(CmpOp::Le, Box::new(ord), Box::new(hi)), Ty::Bool, span);
                self.mk(ExprKind::Binary(BinOp::And, Box::new(ge), Box::new(le)), Ty::Bool, span)
            }
            (Ty::Char, "is_lower") => {
                let ord = self.cast(Cast::CharToInt, value, Ty::Int);
                let lo = self.int_lit(b'a' as i64, span);
                let hi = self.int_lit(b'z' as i64, span);
                let ge = self.mk(ExprKind::Cmp(CmpOp::Ge, Box::new(ord.clone()), Box::new(lo)), Ty::Bool, span);
                let le = self.mk(ExprKind::Cmp(CmpOp::Le, Box::new(ord), Box::new(hi)), Ty::Bool, span);
                self.mk(ExprKind::Binary(BinOp::And, Box::new(ge), Box::new(le)), Ty::Bool, span)
            }
            (Ty::Char, "is_upper") => {
                let ord = self.cast(Cast::CharToInt, value, Ty::Int);
                let lo = self.int_lit(b'A' as i64, span);
                let hi = self.int_lit(b'Z' as i64, span);
                let ge = self.mk(ExprKind::Cmp(CmpOp::Ge, Box::new(ord.clone()), Box::new(lo)), Ty::Bool, span);
                let le = self.mk(ExprKind::Cmp(CmpOp::Le, Box::new(ord), Box::new(hi)), Ty::Bool, span);
                self.mk(ExprKind::Binary(BinOp::And, Box::new(ge), Box::new(le)), Ty::Bool, span)
            }
            (Ty::Char, "is_alpha") => {
                let ord = self.cast(Cast::CharToInt, value, Ty::Int);
                let a = self.int_lit(b'a' as i64, span);
                let z = self.int_lit(b'z' as i64, span);
                let cap_a = self.int_lit(b'A' as i64, span);
                let cap_z = self.int_lit(b'Z' as i64, span);
                let lo1 = self.mk(ExprKind::Cmp(CmpOp::Ge, Box::new(ord.clone()), Box::new(a)), Ty::Bool, span);
                let hi1 = self.mk(ExprKind::Cmp(CmpOp::Le, Box::new(ord.clone()), Box::new(z)), Ty::Bool, span);
                let is_lo = self.mk(ExprKind::Binary(BinOp::And, Box::new(lo1), Box::new(hi1)), Ty::Bool, span);
                let lo2 = self.mk(ExprKind::Cmp(CmpOp::Ge, Box::new(ord.clone()), Box::new(cap_a)), Ty::Bool, span);
                let hi2 = self.mk(ExprKind::Cmp(CmpOp::Le, Box::new(ord), Box::new(cap_z)), Ty::Bool, span);
                let is_hi = self.mk(ExprKind::Binary(BinOp::And, Box::new(lo2), Box::new(hi2)), Ty::Bool, span);
                self.mk(ExprKind::Binary(BinOp::Or, Box::new(is_lo), Box::new(is_hi)), Ty::Bool, span)
            }
            (Ty::Char, "is_space") => {
                let ord = self.cast(Cast::CharToInt, value, Ty::Int);
                let sp = self.int_lit(b' ' as i64, span);
                let tab = self.int_lit(b'\t' as i64, span);
                let nl = self.int_lit(b'\n' as i64, span);
                let cr = self.int_lit(b'\r' as i64, span);
                let e1 = self.mk(ExprKind::Cmp(CmpOp::Eq, Box::new(ord.clone()), Box::new(sp)), Ty::Bool, span);
                let e2 = self.mk(ExprKind::Cmp(CmpOp::Eq, Box::new(ord.clone()), Box::new(tab)), Ty::Bool, span);
                let e3 = self.mk(ExprKind::Cmp(CmpOp::Eq, Box::new(ord.clone()), Box::new(nl)), Ty::Bool, span);
                let e4 = self.mk(ExprKind::Cmp(CmpOp::Eq, Box::new(ord), Box::new(cr)), Ty::Bool, span);
                let o1 = self.mk(ExprKind::Binary(BinOp::Or, Box::new(e1), Box::new(e2)), Ty::Bool, span);
                let o2 = self.mk(ExprKind::Binary(BinOp::Or, Box::new(e3), Box::new(e4)), Ty::Bool, span);
                self.mk(ExprKind::Binary(BinOp::Or, Box::new(o1), Box::new(o2)), Ty::Bool, span)
            }
            (Ty::Pos(d), "len") if self.subset_base(*d).is_some() => {
                let ord = self.cast(Cast::Ord, value, Ty::Int);
                builtin(self, Builtin::Popcount, vec![ord], Ty::Int)
            }
            (Ty::Pos(d), "is_empty") if self.subset_base(*d).is_some() => {
                let ord = self.cast(Cast::Ord, value, Ty::Int);
                let zero = self.int_lit(0, span);
                self.mk(
                    ExprKind::Cmp(CmpOp::Eq, Box::new(ord), Box::new(zero)),
                    Ty::Bool,
                    span,
                )
            }
            (Ty::Pos(d), "first") if self.subset_base(*d).is_some() => {
                let base_d = self.subset_base(*d).unwrap();
                let opt_ty = Ty::opt(Ty::Pos(base_d));
                let ord = self.cast(Cast::Ord, value, Ty::Int);
                self.do_expr(Some(&opt_ty), span, |lw, out| {
                    let s = lw.temp(Ty::Int);
                    out.push(Stmt::Let(s, Some(ord)));
                    let sv = lw.local(s, span);
                    let zero = lw.int_lit(0, span);
                    let is_empty = lw.mk(
                        ExprKind::Cmp(CmpOp::Eq, Box::new(sv.clone()), Box::new(zero)),
                        Ty::Bool,
                        span,
                    );
                    let bit = lw.mk(ExprKind::Builtin(Builtin::Ctz, vec![sv]), Ty::Int, span);
                    let pos = lw.cast(Cast::Trust, bit, Ty::Pos(base_d));
                    let some = lw.mk(ExprKind::Some(Box::new(pos)), opt_ty.clone(), span);
                    let none = lw.mk(ExprKind::None, opt_ty.clone(), span);
                    let res = lw.mk(
                        ExprKind::If(Box::new(is_empty), Box::new(none), Box::new(some)),
                        opt_ty.clone(),
                        span,
                    );
                    out.push(Stmt::Give(res));
                })
            }
            (Ty::Pos(d), "K" | "len" | "count") => self.mk(ExprKind::DomainLen(*d), Ty::Int, span),
            (Ty::DomainVal, "first") => {
                let dom_id = match value.kind {
                    ExprKind::DomainVal(id) => Some(id),
                    _ => self.gen_env.values().find_map(|ga| match ga {
                        GenArg::Domain(id) => Some(*id),
                        _ => None,
                    }),
                };
                if let Some(id) = dom_id {
                    self.mk(
                        ExprKind::Cast(Cast::Trust, Box::new(self.int_lit(0, span))),
                        Ty::Pos(id),
                        span,
                    )
                } else {
                    self.err_expr(span)
                }
            }
            (Ty::DomainVal, "last") => {
                let dom_id = match value.kind {
                    ExprKind::DomainVal(id) => Some(id),
                    _ => self.gen_env.values().find_map(|ga| match ga {
                        GenArg::Domain(id) => Some(*id),
                        _ => None,
                    }),
                };
                if let Some(id) = dom_id {
                    let len = self.mk(ExprKind::DomainLen(id), Ty::Int, span);
                    let last = self.int_add(len, -1);
                    self.mk(
                        ExprKind::Cast(Cast::Trust, Box::new(last)),
                        Ty::Pos(id),
                        span,
                    )
                } else {
                    self.err_expr(span)
                }
            }
            // The smallest and largest key of an ordered set (v2.2; `.min()`
            // and `.max()` are the older spelling).
            (Ty::Set(_) | Ty::Map(..), "first" | "last") => {
                let op = if name.text == "first" {
                    Builtin::SetMin
                } else {
                    Builtin::SetMax
                };
                let ty = Ty::opt(match &value.ty {
                    Ty::Map(key, val) => Ty::Tuple(vec![(**key).clone(), (**val).clone()]),
                    Ty::Set(elem) => (**elem).clone(),
                    _ => Ty::Error,
                });
                self.mk(ExprKind::Builtin(op, vec![value]), ty, span)
            }
            (Ty::Opt(_), "is_some") => self.mk(ExprKind::IsSome(Box::new(value)), Ty::Bool, span),
            (Ty::Opt(_), "is_none") => {
                let some = self.mk(ExprKind::IsSome(Box::new(value)), Ty::Bool, span);
                self.mk(ExprKind::Unary(UnOp::Not, Box::new(some)), Ty::Bool, span)
            }
            // Default `Target::to` trait getter on plain values (e.g. vertices yielded by `u => adj[u]`).
            (_, "to") => value,
            // Default `Hashable::hash_token` trait getter (`Int(self)`).
            (Ty::Int, "hash_token") => value,
            (Ty::Char, "hash_token") => self.cast(Cast::CharToInt, value, Ty::Int),
            (Ty::Bool, "hash_token") => self.mk(
                ExprKind::If(
                    Box::new(value),
                    Box::new(self.int_lit(1, span)),
                    Box::new(self.int_lit(0, span)),
                ),
                Ty::Int,
                span,
            ),
            (Ty::Pos(_), "hash_token") => self.cast(Cast::Ord, value, Ty::Int),
            // A number's math reads either way: `sqrt(x)` or `x.sqrt`.
            (Ty::Float, "sqrt") => self.mk(ExprKind::Builtin(Builtin::Sqrt, vec![value]), Ty::Float, span),
            (Ty::Int, "sqrt") => {
                let value = self.coerce(value, &Ty::Float, span);
                self.mk(ExprKind::Builtin(Builtin::Sqrt, vec![value]), Ty::Float, span)
            }
            (Ty::Int, "isqrt") => self.mk(ExprKind::Builtin(Builtin::Isqrt, vec![value]), Ty::Int, span),
            (Ty::Float, "floor" | "ceil" | "round") => {
                let op = match name.text.as_str() {
                    "floor" => Builtin::Floor,
                    "ceil" => Builtin::Ceil,
                    _ => Builtin::Round,
                };
                self.mk(ExprKind::Builtin(op, vec![value]), Ty::Int, span)
            }
            (Ty::Float, "trunc") => self.cast(Cast::FloatToInt, value, Ty::Int),
            (Ty::Int | Ty::I128 | Ty::Float, "abs") => {
                self.mk(ExprKind::Builtin(Builtin::Abs, vec![value]), ty.clone(), span)
            }
            _ => {
                // UFCS `@get` free function (e.g. `p.adj4`, `g.transposed`, `g.flipped`).
                if let Some(res) = self.try_ufcs_call(&name.text, base, &[], span) {
                    return res;
                }
                let message = format!(
                    "a `{}` has no member `{}` that `cpl run` knows",
                    self.show(&ty),
                    name.text
                );
                self.error(codes::UNKNOWN_FIELD, name.span, message);
                match (&ty, name.text.as_str()) {
                    // An arc of a graph built from a list of pairs is the
                    // neighbour, with no record around it.
                    (Ty::Pos(_), "e") => self.help(
                        "this is the neighbour itself, and a graph built from a list of pairs numbers no edges; \
                         for `.e`, give the edges a domain: `let edges: [E](u: V, v: V) = ...`, then `Graph.directed(edges)`",
                    ),
                    (Ty::List(_), "indices") => self.help(
                        "a list is indexed by `Int` from 0: write `0..xs.len`; \
                         an array `[D]T` has `.indices`, the positions of `D`",
                    ),
                    (Ty::Tuple(parts), _) => self.help(format!(
                        "the parts of a tuple have numbers, not names: `.0` to `.{}`",
                        parts.len().saturating_sub(1)
                    )),
                    _ => {}
                }
                self.err_expr(span)
            }
        }
    }

    /// The outer shape of a type, for telling whose method a call means:
    /// a list and an array are one shape, as a map and a hash map are.
    fn shape(&self, ty: &Ty) -> Option<String> {
        let plain = match ty {
            Ty::Int | Ty::I128 | Ty::U64 => "Int",
            Ty::Float => "F64",
            Ty::Bool => "Bool",
            Ty::Char => "Char",
            Ty::Str => "Str",
            Ty::List(_) | Ty::Array { .. } => "List",
            Ty::Map(..) | Ty::HashMap(..) => "Map",
            Ty::Set(_) | Ty::HashSet(_) => "Set",
            Ty::Opt(_) => "?",
            Ty::Tuple(_) | Ty::Record(_) => "()",
            // A struct or enum is its own shape: `foo(self: A)` is not
            // for a `B`.
            Ty::Struct(id) => {
                return Some(match &self.structs[*id].origin_gen {
                    Some((gid, _)) => format!("struct<> {gid}"),
                    None => format!("struct {id}"),
                })
            }
            Ty::Enum(id) => return Some(format!("enum {id}")),
            _ => return None,
        };
        Some(plain.to_string())
    }

    /// The shape of a type as a fn's `self` writes it; `None` for one of
    /// the fn's own type parameters, which takes any.
    fn written_shape(&self, ty: &ast::Type, generics: &[ast::Name]) -> Option<String> {
        let plain = match &ty.kind {
            ast::TypeKind::Array { .. } | ast::TypeKind::OwnDomain { .. } => "List",
            ast::TypeKind::Optional(_) => "?",
            ast::TypeKind::Tuple(_) | ast::TypeKind::Record(_) => "()",
            ast::TypeKind::Range { .. } => "Int",
            ast::TypeKind::Named { name, .. } => match name.text.as_str() {
                "Int" | "I128" | "U64" => "Int",
                "F64" => "F64",
                "Bool" => "Bool",
                "Char" => "Char",
                "Str" => "Str",
                "List" => "List",
                "Map" | "HashMap" => "Map",
                "Set" | "HashSet" => "Set",
                other if generics.iter().any(|g| g.text == other) => return None,
                other => {
                    return match self.get_sym(other)? {
                        Sym::Struct(id) => Some(format!("struct {id}")),
                        Sym::GenStruct(gid) => Some(format!("struct<> {gid}")),
                        Sym::Enum(id) => Some(format!("enum {id}")),
                        _ => None,
                    }
                }
            },
            _ => return None,
        };
        Some(plain.to_string())
    }

    /// Tries calling a free function or generic free function whose first parameter is `self`
    /// using UFCS (`base.name(args...)` -> `name(base, args...)`).
    fn try_ufcs_call(
        &mut self,
        name: &str,
        base: &ast::Expr,
        args: &[ast::Arg],
        span: Span,
    ) -> Option<Expr> {
        let theirs = self.get_sym(&cpl_check::std_method(name));
        let mut sym = self.get_sym(name).or(theirs)?;
        // The program's own `find(self: List<Int>, ..)` is not what
        // `s.find(..)` on a `Str` means: that is the library's.
        if let Some(theirs) = theirs {
            let own = match sym {
                Sym::Fn(fid) => Some(self.sigs[fid].decl),
                Sym::GenFn(gfid) => Some(self.gen_fns[gfid].decl),
                _ => None,
            };
            let takes = own.and_then(|decl| {
                let first = decl.params.first().filter(|p| p.name.text == "self")?;
                self.written_shape(first.ty.as_ref()?, &decl.generics)
            });
            let given = self.probe_ty_silent(base).and_then(|ty| self.shape(&ty));
            if own.is_none() || matches!((takes, given), (Some(takes), Some(given)) if takes != given) {
                sym = theirs;
            }
        }
        let (has_self, self_mut) = match sym {
            Sym::Fn(fid) => match self.sigs[fid].decl.params.first() {
                Some(p) if p.name.text == "self" => (true, p.mutable),
                _ => (false, false),
            },
            Sym::GenFn(gfid) => match self.gen_fns[gfid].decl.params.first() {
                Some(p) if p.name.text == "self" => (true, p.mutable),
                _ => (false, false),
            },
            _ => (false, false),
        };
        if !has_self {
            return None;
        }
        let mut all_args = Vec::with_capacity(args.len() + 1);
        all_args.push(ast::Arg {
            name: None,
            mutable: self_mut,
            value: base.clone(),
        });
        all_args.extend(args.iter().cloned());
        match sym {
            Sym::Fn(fid) => Some(self.user_call(fid, None, &all_args, span)),
            Sym::GenFn(gfid) => Some(self.call_gen_fn(gfid, Vec::new(), None, &all_args, span)),
            _ => None,
        }
    }
    /// `xs.items` in a `for` or a comprehension, when `xs` names a
    /// variable, a field or a cell: the loop counts through the indices
    /// and reads each element in place, so no list of pairs is built.
    pub(super) fn items_head(&mut self, e: &ast::Expr) -> Option<Items> {
        let ast::ExprKind::Field { base, name } = &e.kind else {
            return None;
        };
        if name.text != "items" {
            return None;
        }
        let is_view_expr = matches!(&base.kind, ast::ExprKind::Index { .. })
            || matches!(&base.kind, ast::ExprKind::Field { name, .. } if name.text == "rev");
        if !is_place(base) && !is_view_expr {
            return None;
        }
        if !matches!(
            self.probe_ty_silent(base),
            Some(Ty::List(_) | Ty::ListView(_) | Ty::MutView(_) | Ty::Array { .. } | Ty::Str | Ty::StrView)
        ) {
            return None;
        }
        let seq = self.expr(base, None);
        let seq = if matches!(seq.ty, Ty::StrView | Ty::ListView(_) | Ty::MutView(_))
            && !matches!(seq.kind, ExprKind::Local(_) | ExprKind::Global(_))
        {
            let held = self.temp(seq.ty.clone());
            self.cx.pending.push(Stmt::Let(held, Some(seq)));
            self.local(held, e.span)
        } else {
            seq
        };
        self.items_of(&seq, e.span)
    }

    /// The indices and element type `seq.items` goes through; `None`
    /// when `seq` is not a list or an array of one dimension.
    fn items_of(&mut self, seq: &Expr, span: Span) -> Option<Items> {
        let (dim, elem) = match &seq.ty {
            Ty::List(elem) | Ty::ListView(elem) | Ty::MutView(elem) => (Dim::Int, (**elem).clone()),
            Ty::Str | Ty::StrView => (Dim::Int, Ty::Char),
            Ty::Array { dims, elem } if dims.len() == 1 => (dims[0], (**elem).clone()),
            _ => return None,
        };
        // The loop counts the cells from 0; `[1..=9]T` names them from 1.
        let (len, index, first) = match dim {
            Dim::Pos(d) => (self.mk(ExprKind::DomainLen(d), Ty::Int, span), Ty::Pos(d), 0),
            Dim::Int => (self.len_of(seq.clone(), span), Ty::Int, 0),
            Dim::IntFrom(first) => (self.len_of(seq.clone(), span), Ty::Int, first),
            _ => return None,
        };
        Some(Items { seq: seq.clone(), len, index, first, elem })
    }

    /// `m.items` kept as a value: the list of `(key, value)`, in the
    /// order a `for` over the map goes.
    fn pairs_of_map(&mut self, map: Expr, span: Span) -> Expr {
        let (Ty::Map(key, held) | Ty::HashMap(key, held)) = &map.ty else {
            unreachable!("the caller matched a map")
        };
        let pair = Ty::Tuple(vec![(**key).clone(), (**held).clone()]);
        self.collected(pair.clone(), span, |lw, out, push| {
            let map = lw.hold(map, out);
            let each = lw.temp(pair);
            let body = Block { stmts: vec![push(lw, lw.local(each, span))] };
            let lid = lw.fresh_loop();
            out.push(Stmt::For(lid, each, Iter::Seq(map, false), body));
        })
    }

    /// `m.keys` / `m.values` kept as a value: the list of keys (`idx == 0`) or
    /// values (`idx == 1`), in the order a `for` over the map goes.
    fn proj_of_map(&mut self, map: Expr, idx: usize, span: Span) -> Expr {
        let (Ty::Map(key, held) | Ty::HashMap(key, held)) = &map.ty else {
            unreachable!("the caller matched a map")
        };
        let pair = Ty::Tuple(vec![(**key).clone(), (**held).clone()]);
        let elem = if idx == 0 { (**key).clone() } else { (**held).clone() };
        self.collected(elem.clone(), span, |lw, out, push| {
            let map = lw.hold(map, out);
            let each = lw.temp(pair);
            let proj = lw.mk(ExprKind::Field(Box::new(lw.local(each, span)), idx), elem.clone(), span);
            let body = Block { stmts: vec![push(lw, proj)] };
            let lid = lw.fresh_loop();
            out.push(Stmt::For(lid, each, Iter::Seq(map, false), body));
        })
    }

    /// `m.keys` / `m.values` (and `.rev` on an ordered `Map`) in a `for` or a
    /// comprehension: walks the map in place with no copied list.
    pub(super) fn map_proj_head(&mut self, e: &ast::Expr) -> Option<MapProj> {
        let (field_expr, rev) = match &e.kind {
            ast::ExprKind::Field { base, name } if name.text == "rev" => (base.as_ref(), true),
            _ => (e, false),
        };
        let ast::ExprKind::Field { base, name } = &field_expr.kind else {
            return None;
        };
        let idx = match name.text.as_str() {
            "keys" => 0,
            "values" => 1,
            _ => return None,
        };
        let base_ty = self.probe_ty_silent(base)?;
        match &base_ty {
            Ty::Map(..) => {}
            Ty::HashMap(..) if !rev => {}
            _ => return None,
        }
        let map = self.expr(base, None);
        let (Ty::Map(key, held) | Ty::HashMap(key, held)) = &map.ty else {
            return None;
        };
        let pair_ty = Ty::Tuple(vec![(**key).clone(), (**held).clone()]);
        let elem_ty = if idx == 0 { (**key).clone() } else { (**held).clone() };
        Some(MapProj { map, pair_ty, elem_ty, idx, rev })
    }

    pub(super) fn map_proj_bind(&mut self, pat: &ast::Pattern, proj: &MapProj, body: &mut Vec<Stmt>) -> Slot {
        let each = self.temp(proj.pair_ty.clone());
        let val = self.mk(
            ExprKind::Field(Box::new(self.local(each, pat.span)), proj.idx),
            proj.elem_ty.clone(),
            pat.span,
        );
        self.bind(pat, val, body);
        each
    }

    /// `g.items` on an array of two or more dimensions: every cell with
    /// the tuple of its indices, `((r, c), g[r, c])`, row by row.
    fn cells_of_grid(&mut self, grid: Expr, span: Span) -> Expr {
        let Ty::Array { dims, elem } = &grid.ty else {
            unreachable!("the caller matched an array")
        };
        let (dims, elem) = (dims.clone(), (**elem).clone());
        let index_tys: Vec<Ty> = dims
            .iter()
            .map(|dim| match dim {
                Dim::Pos(d) => Ty::Pos(*d),
                _ => Ty::Int,
            })
            .collect();
        let at_ty = Ty::Tuple(index_tys);
        let pair = Ty::Tuple(vec![at_ty.clone(), elem.clone()]);
        self.collected(pair.clone(), span, |lw, out, push| {
            let grid = lw.hold(grid, out);
            let counters: Vec<Slot> = dims.iter().map(|_| lw.temp(Ty::Int)).collect();
            let mut named = Vec::new();
            let mut stored = Vec::new();
            for (dim, &k) in dims.iter().zip(&counters) {
                let k = lw.local(k, span);
                match *dim {
                    Dim::Pos(d) => {
                        let at = lw.mk(ExprKind::Cast(Cast::FromOrd(d), Box::new(k)), Ty::Pos(d), span);
                        named.push(at.clone());
                        stored.push(at);
                    }
                    Dim::IntFrom(first) => {
                        named.push(lw.int_add(k.clone(), first));
                        stored.push(k);
                    }
                    _ => {
                        named.push(k.clone());
                        stored.push(k);
                    }
                }
            }
            let at = lw.mk(ExprKind::Tuple(named), at_ty, span);
            let cell = lw.mk(ExprKind::Index(Box::new(grid.clone()), stored), elem, span);
            let both = lw.mk(ExprKind::Tuple(vec![at, cell]), pair, span);
            let mut body = vec![push(lw, both)];
            for (axis, &k) in counters.iter().enumerate().rev() {
                let len = lw.mk(
                    ExprKind::Builtin(Builtin::AxisLen, vec![grid.clone(), lw.int_lit(axis as i64, span)]),
                    Ty::Int,
                    span,
                );
                let iter = Iter::Range {
                    lo: lw.int_lit(0, span),
                    hi: len,
                    step: None,
                    rev: false,
                    closed: false,
                };
                let lid = lw.fresh_loop();
                body = vec![Stmt::For(lid, k, iter, Block { stmts: body })];
            }
            out.extend(body);
        })
    }

    /// `g.items` in a `for`, when `g` names a grid: one loop over the
    /// cells, with a counter per axis stepped at the top of each round, so
    /// no list of pairs is built. The counters are set up in `out`.
    pub(super) fn grid_items_head(&mut self, e: &ast::Expr, out: &mut Vec<Stmt>) -> Option<GridItems> {
        let ast::ExprKind::Field { base, name } = &e.kind else {
            return None;
        };
        if name.text != "items" || !is_place(base) {
            return None;
        }
        match self.probe_ty_silent(base) {
            Some(Ty::Array { dims, .. }) if dims.len() >= 2 && !dims.iter().any(|d| matches!(d, Dim::Bound(_))) => {}
            _ => return None,
        }
        let span = e.span;
        let grid = self.expr(base, None);
        let Ty::Array { dims, elem } = grid.ty.clone() else {
            return None;
        };
        let mut lens = Vec::new();
        let mut total = self.int_lit(1, span);
        for axis in 0..dims.len() {
            let len = self.mk(
                ExprKind::Builtin(Builtin::AxisLen, vec![grid.clone(), self.int_lit(axis as i64, span)]),
                Ty::Int,
                span,
            );
            let len = self.hold(len, out);
            total = self.mk(ExprKind::Binary(BinOp::Mul, Box::new(total), Box::new(len.clone())), Ty::Int, span);
            lens.push(len);
        }
        let total = self.hold(total, out);
        // The last counter starts one before the first cell.
        let last = dims.len() - 1;
        let counters: Vec<Slot> = (0..dims.len())
            .map(|axis| {
                let k = self.temp(Ty::Int);
                let start = self.int_lit(if axis == last { -1 } else { 0 }, span);
                out.push(Stmt::Let(k, Some(start)));
                k
            })
            .collect();
        Some(GridItems { grid, dims, elem: *elem, lens, total, counters })
    }

    /// Steps the counters of `for .. in g.items` and binds its pattern;
    /// gives the slot the loop itself counts in.
    pub(super) fn grid_items_bind(&mut self, pat: &ast::Pattern, items: &GridItems, body: &mut Vec<Stmt>) -> Slot {
        let span = pat.span;
        let place = |lw: &Self, k: Slot| Place { kind: PlaceKind::Local(k), ty: Ty::Int, span: lw.span(span) };
        // `c += 1`, and at the end of the row `c = 0` and the same one axis up.
        let mut step: Vec<Stmt> = Vec::new();
        for (axis, &k) in items.counters.iter().enumerate() {
            let bump = Stmt::Assign(place(self, k), AssignOp::Binary(BinOp::Add), self.int_lit(1, span));
            if axis == 0 {
                step = vec![bump];
                continue;
            }
            let full = self.mk(
                ExprKind::Cmp(CmpOp::Eq, Box::new(self.local(k, span)), Box::new(items.lens[axis].clone())),
                Ty::Bool,
                span,
            );
            let mut wrap = vec![Stmt::Assign(place(self, k), AssignOp::Set, self.int_lit(0, span))];
            wrap.extend(step);
            step = vec![bump, Stmt::If(full, Block { stmts: wrap }, Block::default())];
        }
        body.extend(step);
        let mut named = Vec::new();
        let mut stored = Vec::new();
        for (dim, &k) in items.dims.iter().zip(&items.counters) {
            let k = self.local(k, span);
            match *dim {
                Dim::Pos(d) => {
                    // The counter stays under the axis's length.
                    let at = self.mk(ExprKind::Cast(Cast::Trust, Box::new(k)), Ty::Pos(d), span);
                    named.push(at.clone());
                    stored.push(at);
                }
                Dim::IntFrom(first) => {
                    named.push(self.int_add(k.clone(), first));
                    stored.push(k);
                }
                _ => {
                    named.push(k.clone());
                    stored.push(k);
                }
            }
        }
        let at_ty = Ty::Tuple(named.iter().map(|e| e.ty.clone()).collect());
        let at = self.mk(ExprKind::Tuple(named), at_ty.clone(), span);
        let cell = self.mk(ExprKind::Index(Box::new(items.grid.clone()), stored), items.elem.clone(), span);
        match &pat.kind {
            ast::PatternKind::Tuple(parts) if parts.len() == 2 => {
                self.bind(&parts[0], at, body);
                self.bind(&parts[1], cell, body);
            }
            _ => {
                let ty = Ty::Tuple(vec![at_ty, items.elem.clone()]);
                let both = self.mk(ExprKind::Tuple(vec![at, cell]), ty, span);
                self.bind(pat, both, body);
            }
        }
        self.temp(Ty::Int)
    }

    /// The index and the element `items` has at the counter `k`.
    fn item_at(&mut self, items: &Items, k: Slot, span: Span) -> (Expr, Expr) {
        let k = self.local(k, span);
        let (at, stored) = match items.index {
            Ty::Pos(d) => {
                let at = self.mk(ExprKind::Cast(Cast::FromOrd(d), Box::new(k)), Ty::Pos(d), span);
                (at.clone(), at)
            }
            _ if items.first == 0 => (k.clone(), k),
            _ => (self.int_add(k.clone(), items.first), k),
        };
        let cell = self.mk(ExprKind::Index(Box::new(items.seq.clone()), vec![stored]), items.elem.clone(), span);
        (at, cell)
    }

    /// Binds the pattern of `for (i, x) in xs.items` and gives the
    /// loop's counter.
    pub(super) fn items_bind(&mut self, pat: &ast::Pattern, items: &Items, body: &mut Vec<Stmt>) -> Slot {
        let k = self.temp(Ty::Int);
        let (at, cell) = self.item_at(items, k, pat.span);
        match &pat.kind {
            ast::PatternKind::Tuple(parts) if parts.len() == 2 => {
                self.bind(&parts[0], at, body);
                self.bind(&parts[1], cell, body);
            }
            _ => {
                let ty = Ty::Tuple(vec![items.index.clone(), items.elem.clone()]);
                let both = self.mk(ExprKind::Tuple(vec![at, cell]), ty, pat.span);
                self.bind(pat, both, body);
            }
        }
        k
    }

    /// `xs.find(x => ..)`: the first of a range, list or array the test
    /// holds for, `none` when there is none.
    fn find_first(&mut self, value: Expr, elem: Ty, test: &ast::Expr, span: Span) -> Expr {
        let test_ty = Ty::Fn(vec![elem.clone()], Box::new(Ty::Bool));
        let test = self.expect(test, &test_ty);
        let found_ty = Ty::opt(elem.clone());
        self.do_expr(Some(&found_ty), span, |lw, out| {
            let holds = lw.temp(test_ty);
            out.push(Stmt::Let(holds, Some(test)));
            let found = lw.temp(found_ty.clone());
            out.push(Stmt::Let(
                found,
                Some(lw.mk(ExprKind::None, found_ty.clone(), span)),
            ));
            let item = lw.temp(elem.clone());
            let lid = lw.fresh_loop();
            let asked = vec![Arg::Value(lw.local(item, span))];
            let fits = lw.mk(
                ExprKind::CallExpr(Box::new(lw.local(holds, span)), asked),
                Ty::Bool,
                span,
            );
            let some = lw.mk(
                ExprKind::Some(Box::new(lw.local(item, span))),
                found_ty.clone(),
                span,
            );
            let place = Place {
                kind: PlaceKind::Local(found),
                ty: found_ty.clone(),
                span: lw.span(span),
            };
            let hit = Block {
                stmts: vec![Stmt::Assign(place, AssignOp::Set, some), Stmt::Break(None)],
            };
            let body = Block {
                stmts: vec![Stmt::If(fits, hit, Block::default())],
            };
            let iter = match value.kind {
                ExprKind::Range { lo, hi } => Iter::Range {
                    lo: *lo,
                    hi: *hi,
                    step: None,
                    rev: false,
                    closed: false,
                },
                _ if matches!(value.ty, Ty::Range(_)) => Iter::RangeVal(value, false),
                _ => Iter::Seq(value, false),
            };
            out.push(Stmt::For(lid, item, iter, body));
            out.push(Stmt::Give(lw.local(found, span)));
        })
    }

    /// Whether `e` is a test to search with (`x => x > 3`, or a fn value)
    /// rather than a value to search for.
    fn is_test(&mut self, e: &ast::Expr) -> bool {
        matches!(e.kind, ast::ExprKind::Lambda { .. }) || matches!(self.probe_ty_silent(e), Some(Ty::Fn(..)))
    }


    /// Lowers `xs.first_ge(val, [by:])`, `xs.last_le(val, [by:])`, or `xs.find(val)` on a `List<T>` or 1D `[D]T`.
    fn seq_search_method(
        &mut self,
        method: &str,
        container: Expr,
        elem: Ty,
        dom: Option<DomainId>,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let by_key: Option<&ast::Expr> = if method != "find" {
            match args {
                [first, second]
                    if first.name.is_none()
                        && !first.mutable
                        && second.name.as_ref().is_some_and(|n| n.text == "by")
                        && !second.mutable =>
                {
                    Some(&second.value)
                }
                _ => None,
            }
        } else {
            None
        };
        let target = if by_key.is_some() {
            let t = self.expr(&args[0].value, None);
            if t.ty == Ty::Error {
                return self.err_expr(span);
            }
            t
        } else {
            let Some(t) = self.one_arg(method, args, &elem, span) else {
                return self.err_expr(span);
            };
            t
        };
        let target_ty = target.ty.clone();
        let idx_ty = match dom {
            Some(d) => Ty::Pos(d),
            None => Ty::Int,
        };
        let out_ty = Ty::opt(idx_ty.clone());
        let out_ty_hint = out_ty.clone();
        self.do_expr(Some(&out_ty_hint), span, |lw, out| {
            // `vals.first_ge(x)` searches `vals` where it is: holding a
            // field would copy the whole list on every call.
            let c_ref = lw.hold(container, out);
            let t_slot = lw.temp(target_ty.clone());
            out.push(Stmt::Let(t_slot, Some(target)));
            let t_ref = lw.local(t_slot, span);
            let n_expr = match dom {
                Some(d) => lw.mk(ExprKind::DomainLen(d), Ty::Int, span),
                None => lw.len_of(c_ref.clone(), span),
            };
            let n_slot = lw.temp(Ty::Int);
            out.push(Stmt::Let(n_slot, Some(n_expr)));
            let n_ref = lw.local(n_slot, span);
            let make_idx = |lw: &mut Self, i: Expr| -> Expr {
                match dom {
                    Some(d) => lw.mk(ExprKind::Cast(Cast::FromOrd(d), Box::new(i)), Ty::Pos(d), span),
                    None => i,
                }
            };
            if method == "find" {
                let res_slot = lw.temp(out_ty.clone());
                out.push(Stmt::Let(res_slot, Some(lw.mk(ExprKind::None, out_ty.clone(), span))));
                let i_slot = lw.temp(Ty::Int);
                let i_ref = lw.local(i_slot, span);
                let pos_e = make_idx(lw, i_ref);
                let cur = lw.mk(ExprKind::Index(Box::new(c_ref), vec![pos_e.clone()]), elem, span);
                let eq = lw.mk(ExprKind::Cmp(CmpOp::Eq, Box::new(cur), Box::new(t_ref)), Ty::Bool, span);
                let lid = lw.next_loop;
                lw.next_loop += 1;
                out.push(Stmt::For(
                    lid,
                    i_slot,
                    Iter::Range {
                        lo: lw.int_lit(0, span),
                        hi: n_ref,
                        step: None,
                        rev: false,
                        closed: false,
                    },
                    Block {
                        stmts: vec![Stmt::If(
                            eq,
                            Block {
                                stmts: vec![
                                    Stmt::Assign(
                                        Place {
                                            kind: PlaceKind::Local(res_slot),
                                            ty: out_ty.clone(),
                                            span,
                                        },
                                        AssignOp::Set,
                                        lw.mk(ExprKind::Some(Box::new(pos_e)), out_ty.clone(), span),
                                    ),
                                    Stmt::Break(None),
                                ],
                            },
                            Block::default(),
                        )],
                    },
                ));
                out.push(Stmt::Give(lw.local(res_slot, span)));
            } else {
                // Binary search: `first_ge` finds smallest i in 0..n with c[i] >= t;
                // `last_le` finds largest i in 0..n with c[i] <= t (i.e. first_gt - 1).
                let is_ge = method == "first_ge";
                let lo_slot = lw.temp(Ty::Int);
                let hi_slot = lw.temp(Ty::Int);
                out.push(Stmt::Let(lo_slot, Some(lw.int_lit(0, span))));
                out.push(Stmt::Let(hi_slot, Some(n_ref.clone())));
                let lo_ref = lw.local(lo_slot, span);
                let hi_ref = lw.local(hi_slot, span);
                let cond = lw.mk(
                    ExprKind::Cmp(CmpOp::Lt, Box::new(lo_ref.clone()), Box::new(hi_ref.clone())),
                    Ty::Bool,
                    span,
                );
                let sum = lw.mk(
                    ExprKind::Binary(BinOp::Add, Box::new(lo_ref.clone()), Box::new(hi_ref.clone())),
                    Ty::Int,
                    span,
                );
                let mid = lw.mk(
                    ExprKind::Binary(BinOp::Div, Box::new(sum), Box::new(lw.int_lit(2, span))),
                    Ty::Int,
                    span,
                );
                let mid_slot = lw.temp(Ty::Int);
                let mid_ref = lw.local(mid_slot, span);
                let mid_pos = make_idx(lw, mid_ref.clone());
                let cur = lw.mk(ExprKind::Index(Box::new(c_ref), vec![mid_pos]), elem, span);
                let mut loop_stmts = vec![Stmt::Let(mid_slot, Some(mid))];
                let cur_val = if let Some(key_ast) = by_key {
                    let Some(k) = lw.apply_key(key_ast, cur, &mut loop_stmts, span) else {
                        out.push(Stmt::Give(lw.err_expr(span)));
                        return;
                    };
                    k
                } else {
                    cur
                };
                let t_cmp = lw.coerce(t_ref, &cur_val.ty, args[0].value.span);
                let go_left = if is_ge {
                    lw.mk(ExprKind::Cmp(CmpOp::Ge, Box::new(cur_val), Box::new(t_cmp)), Ty::Bool, span)
                } else {
                    lw.mk(ExprKind::Cmp(CmpOp::Gt, Box::new(cur_val), Box::new(t_cmp)), Ty::Bool, span)
                };
                loop_stmts.push(Stmt::If(
                    go_left,
                    Block {
                        stmts: vec![Stmt::Assign(
                            Place { kind: PlaceKind::Local(hi_slot), ty: Ty::Int, span },
                            AssignOp::Set,
                            mid_ref.clone(),
                        )],
                    },
                    Block {
                        stmts: vec![Stmt::Assign(
                            Place { kind: PlaceKind::Local(lo_slot), ty: Ty::Int, span },
                            AssignOp::Set,
                            lw.int_add(mid_ref, 1),
                        )],
                    },
                ));
                let lid = lw.next_loop;
                lw.next_loop += 1;
                out.push(Stmt::While(lid, cond, Block { stmts: loop_stmts }));
                let res = if is_ge {
                    let ok = lw.mk(
                        ExprKind::Cmp(CmpOp::Lt, Box::new(lo_ref.clone()), Box::new(n_ref)),
                        Ty::Bool,
                        span,
                    );
                    let pos = make_idx(lw, lo_ref);
                    lw.mk(
                        ExprKind::If(
                            Box::new(ok),
                            Box::new(lw.mk(ExprKind::Some(Box::new(pos)), out_ty.clone(), span)),
                            Box::new(lw.mk(ExprKind::None, out_ty.clone(), span)),
                        ),
                        out_ty,
                        span,
                    )
                } else {
                    let ok = lw.mk(
                        ExprKind::Cmp(CmpOp::Gt, Box::new(lo_ref.clone()), Box::new(lw.int_lit(0, span))),
                        Ty::Bool,
                        span,
                    );
                    let pos = make_idx(lw, lw.int_add(lo_ref, -1));
                    lw.mk(
                        ExprKind::If(
                            Box::new(ok),
                            Box::new(lw.mk(ExprKind::Some(Box::new(pos)), out_ty.clone(), span)),
                            Box::new(lw.mk(ExprKind::None, out_ty.clone(), span)),
                        ),
                        out_ty,
                        span,
                    )
                };
                out.push(Stmt::Give(res));
            }
        })
    }

    /// Calls method `fn_id`. With a receiver, `self` is the caller's own
    /// variable when the method changes it.
    pub(super) fn method_call(
        &mut self,
        fn_id: FnId,
        receiver: Option<(&ast::Expr, Expr)>,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let name = self.sigs[fn_id].decl.name.text.clone();
        let declared_mut = self.sigs[fn_id]
            .decl
            .params
            .first()
            .is_some_and(|p| p.mutable);
        let receiver = match (self.sigs[fn_id].receiver, receiver) {
            // A method that grows a cache, called on a value with no home
            // (`Squares([]).at(4)`): the value is held for the call.
            (Some(true), Some((base, value))) if !declared_mut && !is_place(base) => {
                let mut ty = Ty::Unit;
                let mut call = self.do_expr(None, span, |lw, out| {
                    let held = lw.temp(value.ty.clone());
                    let place = Place {
                        kind: PlaceKind::Local(held),
                        ty: value.ty.clone(),
                        span,
                    };
                    out.push(Stmt::Let(held, Some(value)));
                    let call = lw.user_call(fn_id, Some(Arg::Mut(place)), args, span);
                    ty = call.ty.clone();
                    out.push(if ty == Ty::Unit {
                        Stmt::Expr(call)
                    } else {
                        Stmt::Give(call)
                    });
                });
                call.ty = ty;
                return call;
            }
            (Some(true), Some((base, _))) => Some(Arg::Mut(self.place(base))),
            (Some(false), Some((_, value))) => Some(Arg::Value(value)),
            (None, None) => None,
            (Some(_), None) => {
                let message = format!("`{name}` is called on a value: `x.{name}(...)`");
                self.error(codes::WRONG_ARG_COUNT, span, message);
                return self.err_expr(span);
            }
            (None, Some(_)) => {
                let message = format!("`{name}` takes no `self`: call it on the type");
                self.error(codes::WRONG_ARG_COUNT, span, message);
                return self.err_expr(span);
            }
        };
        self.user_call(fn_id, receiver, args, span)
    }

    /// The judge's side of the conversation: `sol.send(a, b)` is a line
    /// the solution reads, and `sol.recv()` is the next thing it printed.
    fn talk(&mut self, name: &ast::Name, args: &[ast::Arg], hint: Option<&Ty>, span: Span) -> Expr {
        if name.text == "send" {
            let values: Vec<(Expr, Span)> = args
                .iter()
                .map(|arg| (self.expr(&arg.value, None), arg.value.span))
                .collect();
            let values: Vec<FmtArg> = values
                .into_iter()
                .filter_map(|(value, at)| self.fmt_arg(value, crate::FLOAT_DIGITS, at))
                .collect();
            return self.mk(ExprKind::Print(values, None), Ty::Unit, span);
        }
        if !args.is_empty() {
            self.error(codes::WRONG_ARG_COUNT, span, "`recv` takes no arguments");
            return self.err_expr(span);
        }
        let Some(want) = hint.filter(|want| **want != Ty::Unit) else {
            self.error(codes::TYPE_MISMATCH, span, "`recv` can't tell what to read");
            self.help("say what the solution printed: `let x: Int = sol.recv()`");
            return self.err_expr(span);
        };
        let Some(spec) = self.spec_of(want) else {
            let message = format!("`recv` can't tell how much a `{}` is", self.show(want));
            self.error(codes::TYPE_MISMATCH, span, message);
            return self.err_expr(span);
        };
        let want = want.clone();
        self.do_expr(Some(&want), span, |lw, out| {
            let swap = lw.mk(ExprKind::Builtin(Builtin::SwapInput, Vec::new()), Ty::Unit, span);
            out.push(Stmt::Expr(swap.clone()));
            let heard = lw.read(spec, span);
            let slot = lw.temp(heard.ty.clone());
            out.push(Stmt::Let(slot, Some(heard)));
            out.push(Stmt::Expr(swap));
            out.push(Stmt::Give(lw.local(slot, span)));
        })
    }

    /// `base.name(args)`.
    fn method(
        &mut self,
        whole: &ast::Expr,
        base: &ast::Expr,
        name: &ast::Name,
        args: &[ast::Arg],
        hint: Option<&Ty>,
    ) -> Expr {
        let span = whole.span;
        // `x.inv()`: a `@get` method reads the same with empty parentheses.
        if args.is_empty() {
            if let Some(Ty::Struct(id)) = self.probe_ty_silent(base) {
                if self.structs[id].getters.contains(&name.text) {
                    return self.field(whole, base, name, hint);
                }
            }
        }
        // `opt.map(f)` / `opt.filter(p)` on `?T`.
        if let ([arg], op @ ("map" | "filter")) = (args, name.text.as_str()) {
            if arg.name.is_none() && matches!(self.probe_ty_silent(base), Some(Ty::Opt(_))) {
                return self.lower_opt_map_or_filter(base, &arg.value, op == "map", span);
            }
        }
        // `a.inverse()`: partial injection `[D]?E -> [E]?D` or bijection `[D]E -> [E]D`.
        if args.is_empty() && name.text == "inverse" {
            if let Some(Ty::Array { dims, .. }) = self.probe_ty_silent(base) {
                if dims.len() == 1 {
                    return self.lower_array_inverse(base, span);
                }
            }
        }
        let is_seq_like = |ty: &Ty| ty.is_seq() || matches!(ty, Ty::Set(_) | Ty::HashSet(_));
        let base_is_seq = self.probe_ty_silent(base).as_ref().is_some_and(&is_seq_like)
            || matches!(self.sym(base), Some(Sym::Domain(_)));
        // `xs.map(x => f(x))` returns `[D...]U` on fixed arrays `[D...]T`
        // (preserving index domains), and `List<U>` across every other sequence.
        if let ([func], "map") = (args, name.text.as_str()) {
            if func.name.is_none() && !func.mutable {
                if matches!(self.probe_ty_silent(base), Some(Ty::Array { .. })) {
                    return self.lower_array_map(base, &func.value, span);
                }
                if base_is_seq {
                    return self.lower_map(base, &func.value, span);
                }
            }
        }
        // `xs.filter(x => cond)` returns a `List<T>` of elements satisfying `cond`.
        if let ([test], "filter") = (args, name.text.as_str()) {
            if base_is_seq {
                return self.lower_filter(base, &test.value, span);
            }
        }
        // `xs.partition(x => cond)` returns `(List<T>, List<T>)`.
        if let ([test], "partition") = (args, name.text.as_str()) {
            if base_is_seq {
                return self.lower_partition(base, &test.value, span);
            }
        }
        // `xs.group(by: x => key)` returns `List<List<T>>` in first-seen key order.
        if let ([key], "group") = (args, name.text.as_str()) {
            if key.name.as_ref().is_none_or(|n| n.text == "by") && base_is_seq {
                return self.lower_group_by(base, &key.value, span);
            }
        }
        // `xs.first(x => cond)` returns `?T`.
        if let ([test], "first") = (args, name.text.as_str()) {
            if test.name.is_none() && base_is_seq {
                return self.lower_first_where(base, &test.value, span);
            }
        }
        // `xs.reach(while: (a, b) => ..., cyclic: Bool = false)` returns `List<Int>`.
        if name.text == "reach" && base_is_seq {
            return self.lower_reach(base, args, span);
        }
        // `D.sort(...)`: sort all positions of domain `D` into a `List<D>`.
        if name.text == "sort" {
            if let Some(Sym::Domain(id)) = self.sym(base) {
                let list = self.positions(id, span);
                let list_ty = list.ty.clone();
                let elem = Ty::Pos(id);
                return self.do_expr(Some(&list_ty), span, |lw, out| {
                    let slot = lw.temp(list_ty.clone());
                    out.push(Stmt::Let(slot, Some(list)));
                    let place = Place {
                        kind: PlaceKind::Local(slot),
                        ty: list_ty.clone(),
                        span: lw.span(span),
                    };
                    let sort_expr = lw.sort_place(place, &elem, args, span);
                    out.push(Stmt::Expr(sort_expr));
                    out.push(Stmt::Give(lw.local(slot, span)));
                });
            }
        }
        // `V.all.without(v)`: lift `Range<V>` to `Bitset<V>` and call `without(v)`.
        if name.text == "without" && args.len() == 1 {
            if let Some(Ty::Range(of)) = self.probe_ty_silent(base) {
                if let Ty::Pos(dom) = of.as_ref() {
                    let dom = *dom;
                    if let Some(sid) = self.instantiate_bitset(dom, span) {
                        let base_expr = self.expr(base, Some(&Ty::Struct(sid)));
                        let bs = self.coerce(base_expr, &Ty::Struct(sid), base.span);
                        if let Some(&without_fn) = self.structs[sid].methods.get("without") {
                            self.queue_fn(without_fn);
                            let v = self.expect(&args[0].value, &Ty::Pos(dom));
                            return self.mk(
                                ExprKind::Call(without_fn, vec![Arg::Value(bs), Arg::Value(v)]),
                                Ty::Struct(sid),
                                span,
                            );
                        }
                    }
                }
            }
        }
        // A fold over a sequence reads as its method: `xs.sum()` is
        // `sum(xs)`, `xs.count(x => x > 2)` is `count(xs, x => x > 2)`.
        let seq_ty = self.probe_ty_silent(base);
        if self.is_sorted_range_expr(base)
            || matches!(
                seq_ty,
                Some(
                    Ty::List(_)
                        | Ty::ListView(_) | Ty::MutView(_)
                        | Ty::Array { .. }
                        | Ty::Range(_)
                        | Ty::Set(_)
                        | Ty::HashSet(_)
                        | Ty::Str
                        | Ty::StrView
                )
            )
        {
            let seq = ast::Arg { name: None, mutable: false, value: base.clone() };
            match (args, name.text.as_str()) {
                ([], "sum") => return self.reduce(Reduce::Sum, base, span),
                ([], "prod") => return self.reduce(Reduce::Prod, base, span),
                ([op], "fold") if op.name.is_none() && !op.mutable => {
                    return self.fold_seq(base, op, false, name.span, span);
                }
                (_, "sorted")
                    if !matches!(
                        seq_ty,
                        Some(Ty::List(_) | Ty::ListView(_) | Ty::MutView(_) | Ty::Array { .. })
                    ) =>
                {
                    let value = match self.walked(base) {
                        Some(values) => self.expr(&values, None),
                        None => self.expr(base, None),
                    };
                    return if args.is_empty() { self.sorted(value, span) } else { self.sorted_by(value, args, span) };
                }
                ([], "min" | "max") if !matches!(seq_ty, Some(Ty::Set(_))) => {
                    return self.min_max(name.text == "max", &[seq], span);
                }
                ([by_arg], "min" | "max")
                    if by_arg.name.as_ref().is_some_and(|n| n.text == "by") && !by_arg.mutable =>
                {
                    return self.min_max_by(name.text == "max", base, &by_arg.value, span);
                }
                ([test], "first") => {
                    if let Some(walk) = self.tested_walk("first", &[seq, test.clone()], true) {
                        return self.lower_first(&walk, span);
                    }
                }
                ([test], "count" | "all" | "any") => {
                    if let Some(walk) = self.tested_walk(&name.text, &[seq, test.clone()], false) {
                        let kind = match name.text.as_str() {
                            "count" => Reduce::Count,
                            "all" => Reduce::All,
                            _ => Reduce::Any,
                        };
                        return self.reduce(kind, &walk, span);
                    }
                }
                _ => {}
            }
        }
        // `sol.send(x)` and `sol.recv()` in an `@interactor`.
        if self.judge && matches!(name.text.as_str(), "send" | "recv") {
            if let ast::ExprKind::Name(n) = &base.kind {
                if self.lookup(n).is_some_and(|slot| self.cx.locals[slot].ty == Ty::Unit) {
                    return self.talk(name, args, hint, span);
                }
            }
        }
        // `List.filled(n, val)`
        if let ast::ExprKind::Name(n) = &base.kind {
            if n == "List" && self.lookup(n).is_none() && name.text == "filled" {
                let Some(exprs) = self.positional_any("filled", args, 2, span) else {
                    return self.err_expr(span);
                };
                let count = self.expect(exprs[0], &Ty::Int);
                let val_hint = match hint {
                    Some(Ty::List(e)) => Some(&**e),
                    _ => None,
                };
                // Under a known element type the value is one of it:
                // `let a: List<Z> = List.filled(n, 0)` holds `Z(0)`.
                let val = match val_hint {
                    Some(elem) if !matches!(elem, Ty::Error) => {
                        let elem = elem.clone();
                        self.expect(exprs[1], &elem)
                    }
                    _ => self.expr(exprs[1], None),
                };
                let list_ty = Ty::list(val.ty.clone());
                return self.mk(
                    ExprKind::Builtin(Builtin::ListFilled, vec![count, val]),
                    list_ty,
                    span,
                );
            }
        }
        // `D.nth(k)` where `D` is bound to `V.bounds`.
        if name.text == "nth" {
            if let ast::ExprKind::Name(n) = &base.kind {
                if self.lookup(n).is_none() {
                    if let Some(GenArg::Ty(Ty::Bound(id))) = self.gen_env.get(n).cloned() {
                        if let Some(val) = self.one_arg("nth", args, &Ty::Int, span) {
                            return self.mk(
                                ExprKind::Cast(Cast::Trust, Box::new(val)),
                                Ty::Bound(id),
                                span,
                            );
                        }
                        return self.err_expr(span);
                    }
                }
            }
        }
        // `D.bounds.nth(k)`
        if let ast::ExprKind::Field {
            base: dbase,
            name: fname,
        } = &base.kind
        {
            if fname.text == "bounds" && name.text == "nth" {
                let dom_id = if let Some(Sym::Domain(id)) = self.sym(dbase) {
                    Some(id)
                } else if let ast::ExprKind::Name(n) = &dbase.kind {
                    if self.lookup(n).is_none() {
                        match self.gen_env.get(n) {
                            Some(GenArg::Domain(id)) | Some(GenArg::Ty(Ty::Bound(id))) => Some(*id),
                            _ => None,
                        }
                    } else {
                        None
                    }
                } else {
                    let dv = self.expr(dbase, None);
                    match dv.kind {
                        ExprKind::DomainVal(id) => Some(id),
                        _ => match dv.ty {
                            Ty::Pos(id) | Ty::Bound(id) => Some(id),
                            Ty::DomainVal => self.gen_env.values().find_map(|ga| match ga {
                                GenArg::Domain(id) => Some(*id),
                                _ => None,
                            }),
                            _ => None,
                        },
                    }
                };
                if let Some(id) = dom_id {
                    if let Some(val) = self.one_arg("nth", args, &Ty::Int, span) {
                        return self.mk(
                            ExprKind::Cast(Cast::Trust, Box::new(val)),
                            Ty::Bound(id),
                            span,
                        );
                    }
                    return self.err_expr(span);
                }
            }
        }
        // `GenericStruct<T>.method(...)` (e.g. `SegTree<V, Int>.from(...)`, `Graph<V, Int>.directed(...)`).
        if let ast::ExprKind::Generic {
            base: gbase,
            args: gtypes,
        } = &base.kind
        {
            if let ast::ExprKind::Name(gname) = &gbase.kind {
                let lookup = prefix_shape_struct(gname, gtypes.len()).unwrap_or(gname);
                if let Some(Sym::GenStruct(gid)) = self.get_sym(lookup) {
                    if let Some(call) = self.list_form_call(gid, name, gtypes, args, span) {
                        return call;
                    }
                    if let Some(call) = self.list_form_call(gid, name, gtypes, args, span) {
                        return call;
                    }
                    let gargs = self.resolve_gen_args(gtypes);
                    let sid = self.instantiate_struct(gid, gargs);
                    if let Some(&fn_id) = self.structs[sid].methods.get(&name.text) {
                        return self.method_call(fn_id, None, args, span);
                    }
                    if let Some(&gfid) = self.structs[sid].gen_methods.get(&name.text) {
                        return self.call_gen_fn(gfid, Vec::new(), None, args, span);
                    }
                    self.error(
                        codes::UNKNOWN_FIELD,
                        name.span,
                        format!("`{}` has no fn `{}`", self.structs[sid].def.name, name.text),
                    );
                    return self.err_expr(span);
                }
            }
        }
        // `T.gen(r)` for a type parameter filled by a built-in type: the
        // fn its `impl` declares.
        if let ast::ExprKind::Name(tname) = &base.kind {
            if let (None, Some(GenArg::Ty(ty))) = (self.lookup(tname), self.gen_env.get(tname)) {
                let ty = ty.clone();
                let found = self
                    .prim_methods
                    .iter()
                    .find(|(t, m, _)| *t == ty && *m == name.text)
                    .map(|p| p.2);
                if let Some(fn_id) = found {
                    return self.user_call(fn_id, None, args, span);
                }
            }
        }
        match self.sym(base) {
            Some(Sym::Mod(target_mod)) => {
                let sym = self.mod_syms[target_mod].get(&name.text).copied();
                return match sym {
                    Some(Sym::Fn(id)) => self.user_call(id, None, args, span),
                    Some(Sym::GenFn(gfid)) => self.call_gen_fn(gfid, Vec::new(), None, args, span),
                    Some(Sym::Struct(id)) => self.construct_or_new(id, args, span),
                    Some(Sym::GenStruct(gid)) => self.call_gen_struct(gid, Vec::new(), args, hint, span),
                    Some(Sym::Variant(id, tag)) => self.variant(id, tag, args, span),
                    _ => {
                        self.error(
                            codes::UNKNOWN_FIELD,
                            name.span,
                            format!("module has no fn `{}`", name.text),
                        );
                        self.err_expr(span)
                    }
                };
            }
            Some(Sym::Struct(id)) => {
                if let Some(&fn_id) = self.structs[id].methods.get(&name.text) {
                    return self.method_call(fn_id, None, args, span);
                }
                if let Some(&gfid) = self.structs[id].gen_methods.get(&name.text) {
                    return self.call_gen_fn(gfid, Vec::new(), None, args, span);
                }
                self.error(
                    codes::UNKNOWN_FIELD,
                    name.span,
                    format!("`{}` has no fn `{}`", self.structs[id].def.name, name.text),
                );
                return self.err_expr(span);
            }
            Some(Sym::GenStruct(gid)) => {
                // Static method on a generic struct without explicit type args (`Order.by(...)`, `Rooted.weighted(...)`, `Monoid.new(...)`)
                let decl = self.gen_structs[gid].decl;
                let tpl_mod = self.gen_structs[gid].mod_id;
                let gen_names: Vec<String> = decl.generics.iter().map(|g| g.text.clone()).collect();
                if let Some(call) = self.list_form_call(gid, name, &[], args, span) {
                    return call;
                }
                let mut inferred: GenEnv = HashMap::new();
                if let Some(Ty::Struct(hsid)) = hint {
                    if let Some((hgid, hargs)) = &self.structs[*hsid].origin_gen {
                        if *hgid == gid {
                            for (g, a) in gen_names.iter().zip(hargs) {
                                inferred.insert(g.clone(), a.clone());
                            }
                        }
                    }
                }
                if inferred.len() < gen_names.len() {
                    if let Some(mdecl) = decl.methods.iter().find_map(|m| match &m.kind {
                        ast::StmtKind::Fn(f) if f.name.text == name.text => Some(f),
                        _ => None,
                    }) {
                        for (p, arg) in mdecl.params.iter().zip(args) {
                            if !matches!(arg.value.kind, ast::ExprKind::Lambda { .. }) {
                                if let Some(pty) = &p.ty {
                                    let dlen = self.diags.len();
                                    let probed = self.expr(&arg.value, None);
                                    self.diags.truncate(dlen);
                                    if probed.ty != Ty::Error {
                                        self.unify_ast_ty(
                                            pty,
                                            &probed.ty,
                                            tpl_mod,
                                            &gen_names,
                                            &mut inferred,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                let final_args: Vec<GenArg> = gen_names
                    .iter()
                    .map(|g| inferred.get(g).cloned().unwrap_or(GenArg::Ty(Ty::Int)))
                    .collect();
                let sid = self.instantiate_struct(gid, final_args);
                if let Some(&fn_id) = self.structs[sid].methods.get(&name.text) {
                    return self.method_call(fn_id, None, args, span);
                }
                if let Some(&gfid) = self.structs[sid].gen_methods.get(&name.text) {
                    return self.call_gen_fn(gfid, Vec::new(), None, args, span);
                }
                self.error(
                    codes::UNKNOWN_FIELD,
                    name.span,
                    format!("`{}` has no fn `{}`", decl.name.text, name.text),
                );
                return self.err_expr(span);
            }
            Some(Sym::Alias(aid)) => {
                let (m, ty_ast, offset) = self.aliases[aid];
                let sm = std::mem::replace(&mut self.cur_mod, m);
                let so = std::mem::replace(&mut self.offset, offset);
                let resolved = self.resolve_ty(ty_ast);
                self.cur_mod = sm;
                self.offset = so;
                if let Ty::Struct(id) = resolved {
                    if let Some(&fn_id) = self.structs[id].methods.get(&name.text) {
                        return self.method_call(fn_id, None, args, span);
                    }
                    if let Some(&gfid) = self.structs[id].gen_methods.get(&name.text) {
                        return self.call_gen_fn(gfid, Vec::new(), None, args, span);
                    }
                }
            }
            Some(Sym::Domain(id))
                if self.sum_enums.get(&id).is_some_and(|info| {
                    info.variants.iter().any(|(v, _)| *v == name.text)
                }) =>
            {
                let info = self.sum_enums[&id].clone();
                let tag = info
                    .variants
                    .iter()
                    .position(|(v, _)| *v == name.text)
                    .unwrap();
                return self.variant(info.enum_id, tag, args, span);
            }
            Some(Sym::Enum(id)) => {
                return match self.enums[id]
                    .variants
                    .iter()
                    .position(|(v, _)| *v == name.text)
                {
                    Some(tag) => self.variant(id, tag, args, span),
                    None => {
                        let message =
                            format!("`{}` has no variant `{}`", self.enums[id].name, name.text);
                        self.error(codes::UNKNOWN_FIELD, name.span, message);
                        self.err_expr(span)
                    }
                };
            }
            Some(Sym::Domain(id)) => match name.text.as_str() {
                "at" | "nth" => {
                    return match self.one_arg(&name.text, args, &Ty::Int, span) {
                        Some(value) => self.mk(
                            ExprKind::Cast(Cast::FromOrd(id), Box::new(value)),
                            Ty::Pos(id),
                            span,
                        ),
                        None => self.err_expr(span),
                    };
                }
                // `V.from_idx(i)` is the inverse of `v.to_idx`, and `none`
                // outside the domain.
                "try_at" | "from_idx" => {
                    return match self.one_arg(&name.text, args, &Ty::Int, span) {
                        Some(value) => self.mk(
                            ExprKind::Builtin(Builtin::PosTryOrd(id), vec![value]),
                            Ty::opt(Ty::Pos(id)),
                            span,
                        ),
                        None => self.err_expr(span),
                    };
                }
                "after" => {
                    return match self.one_arg("after", args, &Ty::Pos(id), span) {
                        Some(pos) => {
                            let lo = self.int_add(self.cast(Cast::Ord, pos, Ty::Int), 1);
                            let hi = self.mk(ExprKind::DomainLen(id), Ty::Int, span);
                            self.mk(
                                ExprKind::Range {
                                    lo: Box::new(lo),
                                    hi: Box::new(hi),
                                },
                                Ty::Range(Box::new(Ty::Pos(id))),
                                span,
                            )
                        }
                        None => self.err_expr(span),
                    };
                }
                "before" => {
                    return match self.one_arg("before", args, &Ty::Pos(id), span) {
                        Some(pos) => {
                            let lo = self.int_lit(0, span);
                            let hi = self.cast(Cast::Ord, pos, Ty::Int);
                            self.mk(
                                ExprKind::Range {
                                    lo: Box::new(lo),
                                    hi: Box::new(hi),
                                },
                                Ty::Range(Box::new(Ty::Pos(id))),
                                span,
                            )
                        }
                        None => self.err_expr(span),
                    };
                }
                "pairs" if args.is_empty() => {
                    return self.domain_pairs_list(id, span);
                }
                _ => {}
            },
            _ => {}
        }
        // Check `lends` projection read before lowering `base` as a plain value.
        if let Some(proj) = self.find_projection_on(base, &name.text) {
            return self.lower_projection_read(proj, base, args, span);
        }
        let value = self.expr(base, None);
        let ty = value.ty.clone();
        let method = name.text.as_str();
        // Calling a field of function type (`m.op(a, b)`, `self.lt(a, b)`).
        if let Some((index, fty)) = self.field_of(&ty, method) {
            if matches!(fty, Ty::Fn(..)) {
                let fv = self.mk(ExprKind::Field(Box::new(value), index), fty, span);
                return self.call_fn_val(fv, args, span);
            }
        }
        if let Ty::Struct(id) = &ty {
            if method == "get" {
                if let Some(key_ty) = self.counter_key.get(id).cloned() {
                    let Some(key) = self.one_arg(method, args, &key_ty, span) else {
                        return self.err_expr(span);
                    };
                    let map_ty = self.structs[*id].def.fields[0].1.clone();
                    let map_expr = self.mk(ExprKind::Field(Box::new(value), 0), map_ty, span);
                    let found = self.mk(
                        ExprKind::Builtin(Builtin::MapGet, vec![map_expr, key]),
                        Ty::opt(Ty::Int),
                        span,
                    );
                    let zero = self.int_lit(0, span);
                    return self.mk(
                        ExprKind::Binary(BinOp::Coalesce, Box::new(found), Box::new(zero)),
                        Ty::Int,
                        span,
                    );
                }
            }
            if let Some(fn_id) = self.structs[*id].methods.get(method).copied() {
                return self.method_call(fn_id, Some((base, value)), args, span);
            }
            if let Some(gfid) = self.structs[*id].gen_methods.get(method).copied() {
                let is_mut = self.gen_fns[gfid]
                    .decl
                    .params
                    .first()
                    .is_some_and(|p| p.name.text == "self" && p.mutable)
                    || self.structs[*id]
                        .decl
                        .is_some_and(|sd| super::cache::grows_cache(sd, self.gen_fns[gfid].decl));
                let recv = if is_mut {
                    Arg::Mut(self.place(base))
                } else {
                    Arg::Value(value)
                };
                return self.call_gen_fn(gfid, Vec::new(), Some(recv), args, span);
            }
        }
        // Members that read like fields can be called too: `xs.len()`.
        if args.is_empty()
            && matches!(method, "len" | "is_empty" | "first" | "last" | "items" | "keys" | "values")
            && !matches!(ty, Ty::Struct(_))
        {
            return self.field(whole, base, name, hint);
        }
        let elem = match &ty {
            Ty::List(elem) | Ty::ListView(elem) | Ty::MutView(elem) | Ty::Set(elem) | Ty::HashSet(elem) | Ty::Range(elem) => (**elem).clone(),
            Ty::Array { elem, .. } => (**elem).clone(),
            Ty::Map(key, _) | Ty::HashMap(key, _) => (**key).clone(),
            Ty::Str | Ty::StrView => Ty::Char,
            _ => Ty::Error,
        };
        let mutate = |lw: &mut Self, op: Mutation, args: Vec<Expr>, result: Ty| {
            let place = lw.place(base);
            lw.mk(ExprKind::Mutate(op, Box::new(place), args), result, span)
        };
        match (&ty, method) {
            (Ty::Pos(d), "with") if self.subset_base(*d).is_some() => {
                let base_d = self.subset_base(*d).unwrap();
                let Some(v) = self.one_arg(method, args, &Ty::Pos(base_d), span) else {
                    return self.err_expr(span);
                };
                let s_ord = self.cast(Cast::Ord, value, Ty::Int);
                let v_ord = self.cast(Cast::Ord, v, Ty::Int);
                let one = self.int_lit(1, span);
                let bit = self.mk(
                    ExprKind::Binary(BinOp::Shl, Box::new(one), Box::new(v_ord)),
                    Ty::Int,
                    span,
                );
                let res = self.mk(
                    ExprKind::Binary(BinOp::BitOr, Box::new(s_ord), Box::new(bit)),
                    Ty::Int,
                    span,
                );
                self.cast(Cast::Trust, res, Ty::Pos(*d))
            }
            // `s.set(v, on)` is `s.with(v)` when `on` holds and `s.without(v)`
            // when it does not.
            (Ty::Pos(d), "set") if self.subset_base(*d).is_some() => {
                let base_d = self.subset_base(*d).unwrap();
                let Some(exprs) = self.positional_any(method, args, 2, span) else {
                    return self.err_expr(span);
                };
                let v = self.expect(exprs[0], &Ty::Pos(base_d));
                let on = self.expect(exprs[1], &Ty::Bool);
                let s_ord = self.cast(Cast::Ord, value, Ty::Int);
                let v_ord = self.cast(Cast::Ord, v, Ty::Int);
                let one = self.int_lit(1, span);
                let bit = self.mk(
                    ExprKind::Binary(BinOp::Shl, Box::new(one), Box::new(v_ord)),
                    Ty::Int,
                    span,
                );
                let with = self.mk(
                    ExprKind::Binary(BinOp::BitOr, Box::new(s_ord.clone()), Box::new(bit.clone())),
                    Ty::Int,
                    span,
                );
                let mask = self.mk(ExprKind::Unary(UnOp::BitNot, Box::new(bit)), Ty::Int, span);
                let without = self.mk(
                    ExprKind::Binary(BinOp::BitAnd, Box::new(s_ord), Box::new(mask)),
                    Ty::Int,
                    span,
                );
                // One side runs, so `s` and `v` are each worked out once.
                let res = self.mk(
                    ExprKind::If(Box::new(on), Box::new(with), Box::new(without)),
                    Ty::Int,
                    span,
                );
                self.cast(Cast::Trust, res, Ty::Pos(*d))
            }
            (Ty::Pos(d), "without") if self.subset_base(*d).is_some() => {
                let base_d = self.subset_base(*d).unwrap();
                let Some(v) = self.one_arg(method, args, &Ty::Pos(base_d), span) else {
                    return self.err_expr(span);
                };
                let s_ord = self.cast(Cast::Ord, value, Ty::Int);
                let v_ord = self.cast(Cast::Ord, v, Ty::Int);
                let one = self.int_lit(1, span);
                let bit = self.mk(
                    ExprKind::Binary(BinOp::Shl, Box::new(one), Box::new(v_ord)),
                    Ty::Int,
                    span,
                );
                let mask = self.mk(ExprKind::Unary(UnOp::BitNot, Box::new(bit)), Ty::Int, span);
                let res = self.mk(
                    ExprKind::Binary(BinOp::BitAnd, Box::new(s_ord), Box::new(mask)),
                    Ty::Int,
                    span,
                );
                self.cast(Cast::Trust, res, Ty::Pos(*d))
            }
            (Ty::Error, _) => {
                for arg in args {
                    self.expr(&arg.value, None);
                }
                self.err_expr(span)
            }
            (Ty::Pos(d), "l") => {
                let d = *d;
                match self.one_arg("l", args, &Ty::Int, span) {
                    Some(k) => self.mk(
                        ExprKind::Builtin(Builtin::BoundStepL(d), vec![value, k]),
                        Ty::Bound(d),
                        span,
                    ),
                    None => self.err_expr(span),
                }
            }
            (Ty::Pos(d), "r") => {
                let d = *d;
                match self.one_arg("r", args, &Ty::Int, span) {
                    Some(k) => self.mk(
                        ExprKind::Builtin(Builtin::BoundStepR(d), vec![value, k]),
                        Ty::Bound(d),
                        span,
                    ),
                    None => self.err_expr(span),
                }
            }
            (Ty::Pos(d), "offset" | "ahead") => {
                let d = *d;
                match self.one_arg(method, args, &Ty::Int, span) {
                    Some(k) => self.mk(
                        ExprKind::Builtin(Builtin::PosOffset(d), vec![value, k]),
                        Ty::opt(Ty::Pos(d)),
                        span,
                    ),
                    None => self.err_expr(span),
                }
            }
            (Ty::Pos(d), "back") => {
                let d = *d;
                match self.one_arg("back", args, &Ty::Int, span) {
                    Some(k) => {
                        let neg_k = self.mk(ExprKind::Unary(UnOp::Neg, Box::new(k)), Ty::Int, span);
                        self.mk(
                            ExprKind::Builtin(Builtin::PosOffset(d), vec![value, neg_k]),
                            Ty::opt(Ty::Pos(d)),
                            span,
                        )
                    }
                    None => self.err_expr(span),
                }
            }
            (Ty::Pos(d), "cyc") => {
                let d = *d;
                match self.one_arg("cyc", args, &Ty::Int, span) {
                    Some(k) => self.mk(
                        ExprKind::Builtin(Builtin::PosCyc(d), vec![value, k]),
                        Ty::Pos(d),
                        span,
                    ),
                    None => self.err_expr(span),
                }
            }
            (Ty::Str, "join") => {
                let Some(exprs) = self.positional_any("join", args, 1, span) else {
                    return self.err_expr(span);
                };
                let items = self.expr(exprs[0], None);
                self.join(value, items, None, exprs[0].span)
            }
            (ty, "get") if ty.is_indexed() => {
                let elem = ty.seq_elem().unwrap();
                let opt = Ty::opt(elem);
                match self.one_arg("get", args, &Ty::Int, span) {
                    Some(at) => self.mk(ExprKind::Builtin(Builtin::SeqGet, vec![value, at]), opt, span),
                    None => self.err_expr(span),
                }
            }
            (ty, "at") if ty.is_indexed() => {
                let elem = ty.seq_elem().unwrap();
                match self.one_arg("at", args, &Ty::Int, span) {
                    Some(at) => {
                        let indices = self.vouched(ty, vec![at]);
                        self.mk(ExprKind::Index(Box::new(value), indices), elem, span)
                    }
                    None => self.err_expr(span),
                }
            }
            (ty, "set_at") if ty.is_indexed_mut() => {
                let elem = ty.seq_elem().unwrap();
                let ty_owned = ty.clone();
                match self.positional_any(method, args, 2, span) {
                    Some(exprs) => {
                        let at = self.expect(exprs[0], &Ty::Int);
                        let val = self.expect(exprs[1], &elem);
                        let val_span = exprs[1].span;
                        self.do_expr(Some(&Ty::Unit), span, |lw, out| {
                            let place = lw.place(base);
                            let val = match lw
                                .place_range(&place)
                                .and_then(|r| r.elem().and_then(|e| e.scalar()).cloned())
                            {
                                Some(b) => lw.check_range(val, &b, val_span),
                                None => val,
                            };
                            let indices = lw.vouched(&ty_owned, vec![at]);
                            let cell = Place {
                                kind: PlaceKind::Index(Box::new(place), indices),
                                ty: elem,
                                span: lw.span(span),
                            };
                            out.push(Stmt::Assign(cell, AssignOp::Set, val));
                            out.push(Stmt::Give(lw.mk(ExprKind::Unit, Ty::Unit, span)));
                        })
                    }
                    None => self.err_expr(span),
                }
            }
            (ty, "swap_idx") if ty.is_indexed_mut() => {
                match self.positional_any(method, args, 2, span) {
                    Some(exprs) => {
                        let a = self.expect(exprs[0], &Ty::Int);
                        let b = self.expect(exprs[1], &Ty::Int);
                        mutate(self, Mutation::Swap, vec![a, b], Ty::Unit)
                    }
                    None => self.err_expr(span),
                }
            }
            (Ty::List(_) | Ty::Str, "push") => match self.one_arg(method, args, &elem, span) {
                Some(arg) => {
                    let place = self.place(base);
                    let arg = match self.place_range(&place).and_then(|r| r.elem().and_then(|e| e.scalar()).cloned()) {
                        Some(b) => self.check_range(arg, &b, span),
                        None => arg,
                    };
                    self.mk(ExprKind::Mutate(Mutation::Push, Box::new(place), vec![arg]), Ty::Unit, span)
                }
                None => self.err_expr(span),
            },
            (Ty::List(_), "push_id") => match self.one_arg(method, args, &elem, span) {
                // The index the new element lands on.
                Some(arg) => self.do_expr(Some(&Ty::Int), span, |lw, out| {
                    let place = lw.place(base);
                    let place = lw.settle_indexes(place, out);
                    let arg = match lw.place_range(&place).and_then(|r| r.elem().and_then(|e| e.scalar()).cloned()) {
                        Some(b) => lw.check_range(arg, &b, span),
                        None => arg,
                    };
                    let held = lw.temp(arg.ty.clone());
                    out.push(Stmt::Let(held, Some(arg)));
                    let id = lw.temp(Ty::Int);
                    let list = super::stmt::place_to_expr(&place);
                    let len = lw.mk(ExprKind::Builtin(Builtin::Len, vec![list]), Ty::Int, span);
                    out.push(Stmt::Let(id, Some(len)));
                    let arg = lw.local(held, span);
                    let push = lw.mk(
                        ExprKind::Mutate(Mutation::Push, Box::new(place), vec![arg]),
                        Ty::Unit,
                        span,
                    );
                    out.push(Stmt::Expr(push));
                    let id = lw.local(id, span);
                    out.push(Stmt::Give(id));
                }),
                None => self.err_expr(span),
            },
            (
                Ty::List(_) | Ty::Str | Ty::Set(_) | Ty::HashSet(_) | Ty::Map(..) | Ty::HashMap(..),
                "take",
            ) => match self.positional_any(method, args, 0, span) {
                Some(_) => mutate(self, Mutation::Take, Vec::new(), ty.clone()),
                None => self.err_expr(span),
            },
            (Ty::Set(_) | Ty::Map(..), "pop_first" | "pop_last") => {
                let found = match &ty {
                    Ty::Map(key, val) => Ty::Tuple(vec![(**key).clone(), (**val).clone()]),
                    _ => elem.clone(),
                };
                let op = if method == "pop_first" { Mutation::PopFirst } else { Mutation::PopLast };
                match self.positional_any(method, args, 0, span) {
                    Some(_) => mutate(self, op, Vec::new(), Ty::opt(found)),
                    None => self.err_expr(span),
                }
            }
            (Ty::List(_) | Ty::Str, "pop") => match self.positional_any(method, args, 0, span) {
                Some(_) => {
                    if matches!(elem, Ty::Opt(_)) {
                        mutate(self, Mutation::PopVal, Vec::new(), elem)
                    } else {
                        mutate(self, Mutation::Pop, Vec::new(), Ty::opt(elem))
                    }
                }
                None => self.err_expr(span),
            },
            (Ty::List(_) | Ty::MutView(_) | Ty::Array { .. } | Ty::Str, "swap") => {
                match self.positional_any(method, args, 2, span) {
                    Some(exprs) => {
                        let idx_ty = match &ty {
                            Ty::Array { dims, .. } if dims.len() == 1 => match dims[0] {
                                Dim::Pos(d) => Ty::Pos(d),
                                Dim::Bound(d) => Ty::Bound(d),
                                Dim::Enum(e) => Ty::Enum(e),
                                Dim::Sym(_) | Dim::Int | Dim::IntFrom(_) => Ty::Int,
                            },
                            _ => Ty::Int,
                        };
                        let a = self.expect(exprs[0], &idx_ty);
                        let b = self.expect(exprs[1], &idx_ty);
                        let a = if matches!(a.ty, Ty::Pos(_) | Ty::Bound(_)) {
                            self.cast(Cast::Ord, a, Ty::Int)
                        } else {
                            a
                        };
                        let b = if matches!(b.ty, Ty::Pos(_) | Ty::Bound(_)) {
                            self.cast(Cast::Ord, b, Ty::Int)
                        } else {
                            b
                        };
                        mutate(self, Mutation::Swap, vec![a, b], Ty::Unit)
                    }
                    None => self.err_expr(span),
                }
            }
            (Ty::List(_) | Ty::Str, "resize") => match self.positional_any(method, args, 2, span) {
                Some(exprs) => {
                    let n = self.expect(exprs[0], &Ty::Int);
                    let v = self.expect(exprs[1], &elem);
                    let place = self.place(base);
                    let v = match self.place_range(&place).and_then(|r| r.elem().and_then(|e| e.scalar()).cloned()) {
                        Some(b) => self.check_range(v, &b, exprs[1].span),
                        None => v,
                    };
                    self.mk(ExprKind::Mutate(Mutation::Resize, Box::new(place), vec![n, v]), Ty::Unit, span)
                }
                None => self.err_expr(span),
            },
            (Ty::List(_) | Ty::Str, "truncate") => match self.one_arg(method, args, &Ty::Int, span) {
                Some(n) => mutate(self, Mutation::Truncate, vec![n], Ty::Unit),
                None => self.err_expr(span),
            },
            (Ty::List(_), "reserve") => match self.one_arg(method, args, &Ty::Int, span) {
                Some(n) => mutate(self, Mutation::Reserve, vec![n], Ty::Unit),
                None => self.err_expr(span),
            },
            (Ty::List(_), "extend") => match self.positional_any(method, args, 1, span) {
                Some(exprs) => {
                    let other = self.expect(exprs[0], &ty);
                    mutate(self, Mutation::Extend, vec![other], Ty::Unit)
                }
                None => self.err_expr(span),
            },
            (Ty::List(_) | Ty::MutView(_) | Ty::Array { .. } | Ty::Str, "fill") => match self.one_arg(method, args, &elem, span) {
                Some(v) => {
                    let place = self.place(base);
                    let v = match self.place_range(&place).and_then(|r| r.elem().and_then(|e| e.scalar()).cloned()) {
                        Some(b) => self.check_range(v, &b, span),
                        None => v,
                    };
                    self.mk(ExprKind::Mutate(Mutation::Fill, Box::new(place), vec![v]), Ty::Unit, span)
                }
                None => self.err_expr(span),
            },
            (Ty::Str | Ty::StrView, "to_str") => match self.positional_any(method, args, 0, span) {
                Some(_) => self.mk(ExprKind::Builtin(Builtin::ToStr, vec![value]), Ty::Str, span),
                None => self.err_expr(span),
            },
            (Ty::List(e) | Ty::ListView(e) | Ty::MutView(e), "to_list") => {
                let out_ty = Ty::list((**e).clone());
                match self.positional_any(method, args, 0, span) {
                    Some(_) => self.mk(ExprKind::Builtin(Builtin::ToList, vec![value]), out_ty, span),
                    None => self.err_expr(span),
                }
            }
            (Ty::StrView, "copy") => match self.positional_any(method, args, 0, span) {
                Some(_) => self.mk(ExprKind::Builtin(Builtin::ToStr, vec![value]), Ty::Str, span),
                None => self.err_expr(span),
            },
            (Ty::ListView(e) | Ty::MutView(e), "copy") => {
                let out_ty = Ty::list((**e).clone());
                match self.positional_any(method, args, 0, span) {
                    Some(_) => self.mk(ExprKind::Builtin(Builtin::ToList, vec![value]), out_ty, span),
                    None => self.err_expr(span),
                }
            }
            (Ty::List(_) | Ty::Array { .. } | Ty::Map(..) | Ty::HashMap(..) | Ty::Set(_) | Ty::HashSet(_) | Ty::Str, "copy") => {
                match self.positional_any(method, args, 0, span) {
                    Some(_) => self.mk(ExprKind::Builtin(Builtin::Copy, vec![value]), ty.clone(), span),
                    None => self.err_expr(span),
                }
            }
            (Ty::Set(_) | Ty::HashSet(_), "insert" | "add") => match self.one_arg(method, args, &elem, span) {
                Some(arg) => mutate(self, Mutation::Insert, vec![arg], Ty::Bool),
                None => self.err_expr(span),
            },
            // `m.remove(k)` gives the value that was there, or `none`.
            (Ty::Map(_, val) | Ty::HashMap(_, val), "remove") => {
                let found_ty = Ty::Opt(val.clone());
                match self.one_arg(method, args, &elem, span) {
                    Some(arg) => self.do_expr(Some(&found_ty), span, |lw, out| {
                        let place = lw.place(base);
                        let place = lw.settle_indexes(place, out);
                        let key = lw.temp(arg.ty.clone());
                        out.push(Stmt::Let(key, Some(arg)));
                        let map = super::stmt::place_to_expr(&place);
                        let key_expr = lw.local(key, span);
                        let found = lw.mk(
                            ExprKind::Builtin(Builtin::MapGet, vec![map, key_expr.clone()]),
                            found_ty.clone(),
                            span,
                        );
                        let held = lw.temp(found_ty.clone());
                        out.push(Stmt::Let(held, Some(found)));
                        let remove = lw.mk(
                            ExprKind::Mutate(Mutation::Remove, Box::new(place), vec![key_expr]),
                            Ty::Bool,
                            span,
                        );
                        // Whether it was there is already known.
                        out.push(Stmt::If(remove, Block::default(), Block::default()));
                        let held = lw.local(held, span);
                        out.push(Stmt::Give(held));
                    }),
                    None => self.err_expr(span),
                }
            }
            (Ty::Set(_) | Ty::HashSet(_), "remove") => {
                match self.one_arg(method, args, &elem, span) {
                    Some(arg) => mutate(self, Mutation::Remove, vec![arg], Ty::Bool),
                    None => self.err_expr(span),
                }
            }
            (Ty::Set(_) | Ty::HashSet(_), "set") => {
                let mut key_ast = None;
                let mut on_ast = None;
                let mut ok = args.len() == 2;
                for a in args {
                    match a.name.as_ref().map(|n| n.text.as_str()) {
                        None if key_ast.is_none() => key_ast = Some(&a.value),
                        Some("on") | None if on_ast.is_none() => on_ast = Some(&a.value),
                        _ => ok = false,
                    }
                }
                match (ok, key_ast, on_ast) {
                    (true, Some(k_ast), Some(b_ast)) => {
                        let k_val = self.expect(k_ast, &elem);
                        let b_val = self.expect(b_ast, &Ty::Bool);
                        self.do_expr(Some(&Ty::Unit), span, |lw, out| {
                            let place = lw.place(base);
                            let place = lw.settle_indexes(place, out);
                            let k_slot = lw.temp(k_val.ty.clone());
                            out.push(Stmt::Let(k_slot, Some(k_val)));
                            let k_ref = lw.local(k_slot, span);
                            let ins = lw.mk(
                                ExprKind::Mutate(Mutation::Insert, Box::new(place.clone()), vec![k_ref.clone()]),
                                Ty::Bool,
                                span,
                            );
                            let rem = lw.mk(
                                ExprKind::Mutate(Mutation::Remove, Box::new(place), vec![k_ref]),
                                Ty::Bool,
                                span,
                            );
                            out.push(Stmt::If(
                                b_val,
                                Block { stmts: vec![Stmt::Expr(ins)] },
                                Block { stmts: vec![Stmt::Expr(rem)] },
                            ));
                            out.push(Stmt::Give(lw.mk(ExprKind::Unit, Ty::Unit, span)));
                        })
                    }
                    _ => self.err_expr(span),
                }
            }
            (Ty::List(_) | Ty::Set(_) | Ty::HashSet(_) | Ty::Map(..) | Ty::HashMap(..) | Ty::Str, "clear") => {
                match self.positional_any(method, args, 0, span) {
                    Some(_) => mutate(self, Mutation::Clear, Vec::new(), Ty::Unit),
                    None => self.err_expr(span),
                }
            }
            (Ty::List(_) | Ty::MutView(_) | Ty::Array { .. } | Ty::Str, "reverse") => match self.positional_any(method, args, 0, span) {
                Some(_) => mutate(self, Mutation::Reverse, Vec::new(), Ty::Unit),
                None => self.err_expr(span),
            },
            (Ty::List(_) | Ty::MutView(_) | Ty::Array { .. } | Ty::Str, "sort") => self.sort(base, &elem, args, span),
            (ty, "sorted") if ty.is_seq() || matches!(ty, Ty::Set(_) | Ty::HashSet(_)) => {
                if args.is_empty() {
                    self.sorted(value, span)
                } else {
                    self.sorted_by(value, args, span)
                }
            }
            (Ty::Str | Ty::StrView, "reversed") => match self.positional_any(method, args, 0, span) {
                Some(_) => {
                    let rev = self.mk(ExprKind::Builtin(Builtin::Reversed, vec![value]), Ty::StrView, span);
                    self.mk(ExprKind::Builtin(Builtin::ToStr, vec![rev]), Ty::Str, span)
                }
                None => self.err_expr(span),
            },
            (Ty::List(e) | Ty::ListView(e) | Ty::MutView(e), "reversed") => {
                let out_ty = Ty::list((**e).clone());
                let view_ty = Ty::ListView(e.clone());
                match self.positional_any(method, args, 0, span) {
                    Some(_) => {
                        let rev = self.mk(ExprKind::Builtin(Builtin::Reversed, vec![value]), view_ty, span);
                        self.mk(ExprKind::Builtin(Builtin::ToList, vec![rev]), out_ty, span)
                    }
                    None => self.err_expr(span),
                }
            }
            (Ty::Array { dims, elem: e }, "reversed") if dims.len() == 1 => {
                let out_ty = Ty::list((**e).clone());
                let view_ty = Ty::ListView(e.clone());
                match self.positional_any(method, args, 0, span) {
                    Some(_) => {
                        let rev = self.mk(ExprKind::Builtin(Builtin::Reversed, vec![value]), view_ty, span);
                        self.mk(ExprKind::Builtin(Builtin::ToList, vec![rev]), out_ty, span)
                    }
                    None => self.err_expr(span),
                }
            }
            (
                Ty::List(_)
                | Ty::ListView(_) | Ty::MutView(_)
                | Ty::Set(_)
                | Ty::HashSet(_)
                | Ty::Map(..)
                | Ty::HashMap(..)
                | Ty::Array { .. }
                | Ty::Str
                | Ty::StrView
                | Ty::Range(_),
                "contains",
            ) => match self.one_arg(method, args, &elem, span) {
                Some(arg) => self.mk(
                    ExprKind::Builtin(Builtin::Contains, vec![value, arg]),
                    Ty::Bool,
                    span,
                ),
                None => self.err_expr(span),
            },
            // How many cells equal `x`: `[1, 2, 1].count(1)` is 2,
            // `s[0..k].count('R')` scans the view directly without copying.
            (Ty::List(_) | Ty::ListView(_) | Ty::MutView(_) | Ty::Array { .. }, "count")
            | (Ty::Str | Ty::StrView, "count")
                if args.len() == 1
                    && (matches!(&ty, Ty::List(_) | Ty::ListView(_) | Ty::MutView(_) | Ty::Array { .. })
                        || self.probe_ty_silent(&args[0].value) == Some(Ty::Char)) =>
            {
                match self.one_arg(method, args, &elem, span) {
                    Some(arg) => self.do_expr(Some(&Ty::Int), span, |lw, out| {
                        let (seq, arg) = (lw.hold(value, out), lw.hold(arg, out));
                        let (acc, each) = (lw.temp(Ty::Int), lw.temp(elem.clone()));
                        out.push(Stmt::Let(acc, Some(lw.int_lit(0, span))));
                        let same = lw.mk(
                            ExprKind::Cmp(CmpOp::Eq, Box::new(lw.local(each, span)), Box::new(arg)),
                            Ty::Bool,
                            span,
                        );
                        let place = Place { kind: PlaceKind::Local(acc), ty: Ty::Int, span: lw.span(span) };
                        let inc = Stmt::Assign(place, AssignOp::Binary(BinOp::Add), lw.int_lit(1, span));
                        let body = Block { stmts: vec![Stmt::If(same, Block { stmts: vec![inc] }, Block::default())] };
                        let lid = lw.fresh_loop();
                        out.push(Stmt::For(lid, each, Iter::Seq(seq, false), body));
                        out.push(Stmt::Give(lw.local(acc, span)));
                    }),
                    None => self.err_expr(span),
                }
            }
            (Ty::Map(_, val) | Ty::HashMap(_, val), "get") => {
                match self.one_arg(method, args, &elem, span) {
                    Some(arg) => self.mk(
                        ExprKind::Builtin(Builtin::MapGet, vec![value, arg]),
                        Ty::Opt(val.clone()),
                        span,
                    ),
                    None => self.err_expr(span),
                }
            }
            (Ty::Set(key) | Ty::Map(key, _), "first_ge" | "first_gt" | "last_le" | "last_lt") => {
                let op = match method {
                    "first_ge" => Builtin::FirstGe,
                    "first_gt" => Builtin::FirstGt,
                    "last_le" => Builtin::LastLe,
                    _ => Builtin::LastLt,
                };
                let found = match &ty {
                    Ty::Map(key, val) => Ty::Tuple(vec![(**key).clone(), (**val).clone()]),
                    _ => (**key).clone(),
                };
                match self.one_arg(method, args, key, span) {
                    Some(bound) => self.mk(ExprKind::Builtin(op, vec![value, bound]), Ty::opt(found), span),
                    None => self.err_expr(span),
                }
            }
            (Ty::Set(_), "min" | "max") => match self.positional_any(method, args, 0, span) {
                Some(_) => {
                    let op = if method == "min" {
                        Builtin::SetMin
                    } else {
                        Builtin::SetMax
                    };
                    self.mk(ExprKind::Builtin(op, vec![value]), Ty::opt(elem), span)
                }
                None => self.err_expr(span),
            },
            (Ty::DomainVal | Ty::Pos(_), "nth" | "at" | "from_idx") => {
                let Some(idx_expr) = self.one_arg(method, args, &Ty::Int, span) else {
                    return self.err_expr(span);
                };
                let dom_id = match value.kind {
                    ExprKind::DomainVal(d) => Some(d),
                    _ => match value.ty {
                        Ty::Pos(d) => Some(d),
                        _ => self.gen_env.values().find_map(|ga| match ga {
                            GenArg::Domain(d) => Some(*d),
                            _ => None,
                        }),
                    },
                };
                match dom_id {
                    Some(d) => self.mk(
                        ExprKind::Cast(Cast::Trust, Box::new(idx_expr)),
                        Ty::Pos(d),
                        span,
                    ),
                    None => self.err_expr(span),
                }
            }
            (Ty::DomainVal, "windows") => {
                let Some(k_expr) = self.one_arg(method, args, &Ty::Int, span) else {
                    return self.err_expr(span);
                };
                let dom_id = match value.kind {
                    ExprKind::DomainVal(d) => Some(d),
                    _ => None,
                };
                let range_ty = match dom_id {
                    Some(d) => Ty::Range(Box::new(Ty::Pos(d))),
                    None => Ty::Range(Box::new(Ty::Int)),
                };
                let list_ty = Ty::list(range_ty.clone());
                let list_ty_hint = list_ty.clone();
                self.do_expr(Some(&list_ty_hint), span, |lw, out| {
                    let k_slot = lw.temp(Ty::Int);
                    out.push(Stmt::Let(k_slot, Some(k_expr)));
                    let k_ref = lw.local(k_slot, span);
                    let n_expr = match dom_id {
                        Some(d) => lw.mk(ExprKind::DomainLen(d), Ty::Int, span),
                        None => lw.mk(ExprKind::Builtin(Builtin::Len, vec![value]), Ty::Int, span),
                    };
                    let n_slot = lw.temp(Ty::Int);
                    out.push(Stmt::Let(n_slot, Some(n_expr)));
                    let n_ref = lw.local(n_slot, span);
                    let acc_slot = lw.temp(list_ty.clone());
                    out.push(Stmt::Let(
                        acc_slot,
                        Some(lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span)),
                    ));
                    let lim = lw.mk(
                        ExprKind::Binary(BinOp::Sub, Box::new(n_ref), Box::new(k_ref.clone())),
                        Ty::Int,
                        span,
                    );
                    let hi_loop = lw.int_add(lim, 1);
                    let i_slot = lw.temp(Ty::Int);
                    let i_ref = lw.local(i_slot, span);
                    let end_e = lw.mk(
                        ExprKind::Binary(BinOp::Add, Box::new(i_ref.clone()), Box::new(k_ref)),
                        Ty::Int,
                        span,
                    );
                    let rng = lw.mk(
                        ExprKind::Range {
                            lo: Box::new(i_ref),
                            hi: Box::new(end_e),
                        },
                        range_ty,
                        span,
                    );
                    let push = lw.mk(
                        ExprKind::Mutate(
                            Mutation::Push,
                            Box::new(Place {
                                kind: PlaceKind::Local(acc_slot),
                                ty: list_ty.clone(),
                                span,
                            }),
                            vec![rng],
                        ),
                        Ty::Unit,
                        span,
                    );
                    let lid = lw.next_loop;
                    lw.next_loop += 1;
                    out.push(Stmt::For(
                        lid,
                        i_slot,
                        Iter::Range {
                            lo: lw.int_lit(0, span),
                            hi: hi_loop,
                            step: None,
                            rev: false,
                            closed: false,
                        },
                        Block {
                            stmts: vec![Stmt::Expr(push)],
                        },
                    ));
                    out.push(Stmt::Give(lw.local(acc_slot, span)));
                })
            }
            // `xs.find(x => x > 3)` is the first element the test holds
            // for; `xs.find(3)` is where the value 3 is. Handled uniformly across
            // every `Indexed` sequence (`List`, `ListView`, 1D `[D]T`, `Str`, `StrView`, `Range`).
            (ty, "find") if ty.is_indexed() && args.len() == 1 && self.is_test(&args[0].value) => {
                let elem = ty.seq_elem().unwrap();
                self.find_first(value, elem, &args[0].value, span)
            }
            (ty, "first_ge" | "last_le" | "find")
                if ty.is_indexed()
                    && !(method == "find"
                        && matches!(ty, Ty::Str | Ty::StrView)
                        && args.len() == 1
                        && matches!(self.probe_ty_silent(&args[0].value), Some(Ty::Str | Ty::StrView))) =>
            {
                let dom = match &ty {
                    Ty::Array { dims, .. } if dims.len() == 1 => match dims[0] {
                        Dim::Pos(d) => Some(d),
                        _ => None,
                    },
                    _ => None,
                };
                let elem = ty.seq_elem().unwrap();
                self.seq_search_method(method, value, elem, dom, args, span)
            }
            (Ty::Fn(_, _), "out") => self.call_fn_val(value, args, span),
            // `Neighbours` for a table of neighbour lists: `adj.out(u)` is
            // `adj[u]`, as it is `adj(u)` for a fn.
            (Ty::Array { dims, elem }, "out")
                if dims.len() == 1 && args.len() == 1 && matches!(**elem, Ty::List(_)) =>
            {
                let at = ast::Expr {
                    kind: ast::ExprKind::Index {
                        base: Box::new(base.clone()),
                        indices: vec![args[0].value.clone()],
                    },
                    span,
                };
                self.expr(&at, None)
            }
            // `(lo..hi).clamp(D)`: the part of an Int range that lies in
            // `D`, as a range of D's positions; empty when none does.
            (Ty::Range(inner), "clamp") if **inner == Ty::Int && args.len() == 1 => {
                let domain = match &args[0].value.kind {
                    ast::ExprKind::Name(name) if self.lookup(name).is_none() => {
                        match (self.gen_env.get(name.as_str()), self.get_sym(name)) {
                            (Some(GenArg::Domain(id)), _) => Some(*id),
                            (_, Some(Sym::Domain(id))) => Some(id),
                            _ => None,
                        }
                    }
                    _ => None,
                };
                let Some(d) = domain else {
                    self.error(codes::TYPE_MISMATCH, args[0].value.span, "`clamp` takes a domain");
                    self.help("write the domain the range is cut to: `(l..=r).clamp(V)`");
                    return self.err_expr(span);
                };
                let range_ty = Ty::Range(Box::new(Ty::Pos(d)));
                self.do_expr(Some(&range_ty.clone()), span, |lw, out| {
                    let range = lw.new_slot("range", value.ty.clone());
                    out.push(Stmt::Let(range, Some(value)));
                    // An end of the range as an ordinal of `D`, kept in 0..=len.
                    let end = |lw: &mut Self, which| {
                        let range = lw.local(range, span);
                        let at = lw.mk(ExprKind::Builtin(which, vec![range]), Ty::Int, span);
                        let first = lw.mk(ExprKind::DomainLo(d), Ty::Int, span);
                        let ord = lw.mk(
                            ExprKind::Binary(BinOp::Sub, Box::new(at), Box::new(first)),
                            Ty::Int,
                            span,
                        );
                        let len = lw.mk(ExprKind::DomainLen(d), Ty::Int, span);
                        let zero = lw.int_lit(0, span);
                        let capped = lw.mk(ExprKind::Builtin(Builtin::Min, vec![ord, len]), Ty::Int, span);
                        lw.mk(ExprKind::Builtin(Builtin::Max, vec![capped, zero]), Ty::Int, span)
                    };
                    let lo = lw.new_slot("lo", Ty::Int);
                    let from = end(lw, Builtin::RangeLo);
                    out.push(Stmt::Let(lo, Some(from)));
                    let to = end(lw, Builtin::RangeHi);
                    // An empty range stays empty: its end is not before its start.
                    let lo_again = lw.local(lo, span);
                    let hi = lw.mk(ExprKind::Builtin(Builtin::Max, vec![to, lo_again]), Ty::Int, span);
                    let lo = lw.local(lo, span);
                    let cut = lw.mk(
                        ExprKind::Range {
                            lo: Box::new(lo),
                            hi: Box::new(hi),
                        },
                        range_ty.clone(),
                        span,
                    );
                    out.push(Stmt::Give(cut));
                })
            }
            // `r.drop(k)` / `r.take(k)` on `Range<Int>` or `Range<D>`: sub-range.
            (Ty::Range(inner), "drop" | "take") if **inner != Ty::I128 => {
                let is_drop = method == "drop";
                let range_ty = value.ty.clone();
                let Some(k) = self.one_arg(method, args, &Ty::Int, span) else {
                    return self.err_expr(span);
                };
                self.do_expr(Some(&range_ty), span, |lw, out| {
                    let range = lw.new_slot("range", range_ty.clone());
                    out.push(Stmt::Let(range, Some(value)));
                    let lo_s = lw.new_slot("lo", Ty::Int);
                    let hi_s = lw.new_slot("hi", Ty::Int);
                    let r1 = lw.local(range, span);
                    let r2 = lw.local(range, span);
                    out.push(Stmt::Let(
                        lo_s,
                        Some(lw.mk(ExprKind::Builtin(Builtin::RangeLo, vec![r1]), Ty::Int, span)),
                    ));
                    out.push(Stmt::Let(
                        hi_s,
                        Some(lw.mk(ExprKind::Builtin(Builtin::RangeHi, vec![r2]), Ty::Int, span)),
                    ));
                    let k_s = lw.new_slot("k", Ty::Int);
                    let zero = lw.mk(ExprKind::Int(0), Ty::Int, span);
                    let k_clamped =
                        lw.mk(ExprKind::Builtin(Builtin::Max, vec![k, zero]), Ty::Int, span);
                    out.push(Stmt::Let(k_s, Some(k_clamped)));
                    let diff = lw.mk(
                        ExprKind::Binary(
                            BinOp::Sub,
                            Box::new(lw.local(hi_s, span)),
                            Box::new(lw.local(lo_s, span)),
                        ),
                        Ty::Int,
                        span,
                    );
                    let zero2 = lw.mk(ExprKind::Int(0), Ty::Int, span);
                    let avail = lw.mk(
                        ExprKind::Builtin(Builtin::Max, vec![diff, zero2]),
                        Ty::Int,
                        span,
                    );
                    let shift = lw.mk(
                        ExprKind::Builtin(Builtin::Min, vec![avail, lw.local(k_s, span)]),
                        Ty::Int,
                        span,
                    );
                    let (new_lo, new_hi) = if is_drop {
                        (
                            lw.mk(
                                ExprKind::Binary(
                                    BinOp::Add,
                                    Box::new(lw.local(lo_s, span)),
                                    Box::new(shift),
                                ),
                                Ty::Int,
                                span,
                            ),
                            lw.local(hi_s, span),
                        )
                    } else {
                        (
                            lw.local(lo_s, span),
                            lw.mk(
                                ExprKind::Binary(
                                    BinOp::Add,
                                    Box::new(lw.local(lo_s, span)),
                                    Box::new(shift),
                                ),
                                Ty::Int,
                                span,
                            ),
                        )
                    };
                    let res = lw.mk(
                        ExprKind::Range {
                            lo: Box::new(new_lo),
                            hi: Box::new(new_hi),
                        },
                        range_ty.clone(),
                        span,
                    );
                    out.push(Stmt::Give(res));
                })
            }
            // `(lo..hi).step(k)`: lo, lo + k, ... below hi, as a list.
            (Ty::Range(inner), "step") if **inner == Ty::Int && args.len() == 1 => {
                let by = self.expr(&args[0].value, Some(&Ty::Int));
                let by = self.coerce(by, &Ty::Int, span);
                let list_ty = Ty::list(Ty::Int);
                self.do_expr(Some(&list_ty.clone()), span, |lw, out| {
                    let range = lw.new_slot("range", value.ty.clone());
                    out.push(Stmt::Let(range, Some(value)));
                    let acc = lw.new_slot("steps", list_ty.clone());
                    let empty = lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span);
                    out.push(Stmt::Let(acc, Some(empty)));
                    let i = lw.new_slot("i", Ty::Int);
                    let bound = |lw: &mut Self, which| {
                        let range = lw.local(range, span);
                        lw.mk(ExprKind::Builtin(which, vec![range]), Ty::Int, span)
                    };
                    let (lo, hi) = (bound(lw, Builtin::RangeLo), bound(lw, Builtin::RangeHi));
                    let item = lw.local(i, span);
                    let push = lw.mk(
                        ExprKind::Mutate(
                            Mutation::Push,
                            Box::new(Place {
                                kind: PlaceKind::Local(acc),
                                ty: list_ty.clone(),
                                span: lw.span(span),
                            }),
                            vec![item],
                        ),
                        Ty::Unit,
                        span,
                    );
                    let lid = lw.next_loop;
                    lw.next_loop += 1;
                    out.push(Stmt::For(
                        lid,
                        i,
                        Iter::Range {
                            lo,
                            hi,
                            step: Some(by),
                            rev: false,
                            closed: false,
                        },
                        Block {
                            stmts: vec![Stmt::Expr(push)],
                        },
                    ));
                    out.push(Stmt::Give(lw.local(acc, span)));
                })
            }
            // `xs.last(k)`: the element `k` back from the last one, so
            // `xs.last(1)` is the one before the last; `none` when there
            // are not that many.
            (ty, "last") if ty.is_indexed() && args.len() == 1 => {
                let elem = ty.seq_elem().unwrap();
                // An array of positions is read at the position that far in.
                let of = match &ty {
                    Ty::Array { dims, .. } => match dims[0] {
                        Dim::Pos(d) => Some(d),
                        _ => None,
                    },
                    _ => None,
                };
                let back = self.expect(&args[0].value, &Ty::Int);
                self.do_expr(Some(&Ty::opt(elem.clone())), span, |lw, out| {
                    // A list that is named is read where it is; one that is
                    // computed is kept for the two reads.
                    let list = if matches!(value.kind, ExprKind::Local(_) | ExprKind::Global(_)) {
                        value
                    } else {
                        let kept = lw.temp(value.ty.clone());
                        out.push(Stmt::Let(kept, Some(value)));
                        lw.local(kept, span)
                    };
                    let k = lw.temp(Ty::Int);
                    out.push(Stmt::Let(k, Some(back)));
                    let len = lw.mk(ExprKind::Builtin(Builtin::Len, vec![list.clone()]), Ty::Int, span);
                    let last = lw.int_add(len, -1);
                    let at = lw.temp(Ty::Int);
                    let from_end = lw.mk(
                        ExprKind::Binary(BinOp::Sub, Box::new(last), Box::new(lw.local(k, span))),
                        Ty::Int,
                        span,
                    );
                    out.push(Stmt::Let(at, Some(from_end)));
                    let cmp = |lw: &mut Self, slot| {
                        let zero = lw.int_lit(0, span);
                        lw.mk(ExprKind::Cmp(CmpOp::Ge, Box::new(lw.local(slot, span)), Box::new(zero)), Ty::Bool, span)
                    };
                    let (ahead, inside) = (cmp(lw, k), cmp(lw, at));
                    let there = lw.mk(ExprKind::Binary(BinOp::And, Box::new(ahead), Box::new(inside)), Ty::Bool, span);
                    let cell = match of {
                        Some(d) => lw.mk(ExprKind::Cast(Cast::FromOrd(d), Box::new(lw.local(at, span))), Ty::Pos(d), span),
                        None => lw.local(at, span),
                    };
                    let item = lw.mk(ExprKind::Index(Box::new(list), vec![cell]), elem.clone(), span);
                    let opt = Ty::opt(elem);
                    let some = lw.mk(ExprKind::Some(Box::new(item)), opt.clone(), span);
                    let none = lw.mk(ExprKind::None, opt.clone(), span);
                    out.push(Stmt::Give(lw.mk(ExprKind::If(Box::new(there), Box::new(some), Box::new(none)), opt, span)));
                })
            }
            (Ty::Str | Ty::StrView, "rev") if args.is_empty() => {
                let value = self.hoist_view_base(value, span);
                self.mk(ExprKind::Builtin(Builtin::Reversed, vec![value]), Ty::StrView, span)
            }
            (Ty::List(e) | Ty::ListView(e) | Ty::MutView(e), "rev") if args.is_empty() => {
                let elem = (**e).clone();
                let value = self.hoist_view_base(value, span);
                self.mk(
                    ExprKind::Builtin(Builtin::Reversed, vec![value]),
                    Ty::ListView(Box::new(elem)),
                    span,
                )
            }
            (Ty::Range(_), "rev") if args.is_empty() => self.reversed_range(value, span),
            // `r.halves()`: split `r` into `(left, right)` with `left.len == (r.len + 1) / 2`
            // and `right.len == r.len / 2` (both non-empty whenever `r.len >= 2`).
            (Ty::Range(_), "halves") if args.is_empty() => {
                let r_ty = value.ty.clone();
                let tup_ty = Ty::Tuple(vec![r_ty.clone(), r_ty.clone()]);
                self.do_expr(Some(&tup_ty.clone()), span, |lw, out| {
                    let r_slot = lw.temp(r_ty.clone());
                    out.push(Stmt::Let(r_slot, Some(value)));
                    let r_ref = lw.local(r_slot, span);
                    let lo_slot = lw.temp(Ty::Int);
                    let hi_slot = lw.temp(Ty::Int);
                    let lo_expr = lw.mk(ExprKind::Builtin(Builtin::RangeLo, vec![r_ref.clone()]), Ty::Int, span);
                    let raw_hi = lw.mk(ExprKind::Builtin(Builtin::RangeHi, vec![r_ref]), Ty::Int, span);
                    out.push(Stmt::Let(lo_slot, Some(lo_expr)));
                    let hi_expr = lw.mk(
                        ExprKind::Builtin(Builtin::Max, vec![raw_hi, lw.local(lo_slot, span)]),
                        Ty::Int,
                        span,
                    );
                    out.push(Stmt::Let(hi_slot, Some(hi_expr)));
                    let diff = lw.mk(
                        ExprKind::Binary(
                            BinOp::Sub,
                            Box::new(lw.local(hi_slot, span)),
                            Box::new(lw.local(lo_slot, span)),
                        ),
                        Ty::Int,
                        span,
                    );
                    let left_len = lw.mk(
                        ExprKind::Binary(
                            BinOp::Div,
                            Box::new(lw.int_add(diff, 1)),
                            Box::new(lw.int_lit(2, span)),
                        ),
                        Ty::Int,
                        span,
                    );
                    let mid_slot = lw.temp(Ty::Int);
                    let mid_expr = lw.mk(
                        ExprKind::Binary(BinOp::Add, Box::new(lw.local(lo_slot, span)), Box::new(left_len)),
                        Ty::Int,
                        span,
                    );
                    out.push(Stmt::Let(mid_slot, Some(mid_expr)));
                    let left = lw.mk(
                        ExprKind::Range {
                            lo: Box::new(lw.local(lo_slot, span)),
                            hi: Box::new(lw.local(mid_slot, span)),
                        },
                        r_ty.clone(),
                        span,
                    );
                    let right = lw.mk(
                        ExprKind::Range {
                            lo: Box::new(lw.local(mid_slot, span)),
                            hi: Box::new(lw.local(hi_slot, span)),
                        },
                        r_ty.clone(),
                        span,
                    );
                    let pair = lw.mk(ExprKind::Tuple(vec![left, right]), tup_ty.clone(), span);
                    out.push(Stmt::Give(pair));
                })
            }
            // `r.find(i => ..)`: the first of the range the test holds
            // for, `none` when there is none. `(0..).find(..)` has no end
            // and stops there.
            (Ty::Range(elem), "find") if args.len() == 1 => {
                let elem = (**elem).clone();
                self.find_first(value, elem, &args[0].value, span)
            }
            (Ty::Array { dims, elem }, "rev") if dims.len() == 1 && args.is_empty() => {
                let elem = (**elem).clone();
                let value = self.hoist_view_base(value, span);
                self.mk(
                    ExprKind::Builtin(Builtin::Reversed, vec![value]),
                    Ty::ListView(Box::new(elem)),
                    span,
                )
            }
            // A sequence walked backwards is something to loop over. As a
            // value it would be the sequence unchanged, which is not what
            // `.rev()` says.
            (
                Ty::Array { .. } | Ty::Set(_) | Ty::Map(..) | Ty::HashSet(_) | Ty::HashMap(..),
                "rev",
            ) if args.is_empty() => {
                let message = format!(
                    "a reversed `{}` is something to loop over, not a value: \
                     `for x in xs.rev`, or `[x for x in xs.rev]` for the list",
                    self.show(&ty)
                );
                self.error(codes::UNSUPPORTED, name.span, message);
                self.err_expr(span)
            }
            // `Reversible.rev`: a value that is not a sequence and has no
            // `rev` of its own is its own reverse.
            (_, "rev") if args.is_empty() => value,
            _ => {
                if let Some(&(_, _, fid)) = self.prim_methods.iter().find(|(t, m, _)| *t == ty && m == method) {
                    let mut all_args = vec![ast::Arg { name: None, mutable: false, value: base.clone() }];
                    all_args.extend(args.iter().cloned());
                    return self.user_call(fid, None, &all_args, span);
                }
                // UFCS free function with `self` first parameter (e.g. `p.step(d)`, `p.ray(d)`).
                if let Some(res) = self.try_ufcs_call(method, base, args, span) {
                    return res;
                }
                let message = format!(
                    "a `{}` has no method `{method}` that `cpl run` knows",
                    self.show(&ty)
                );
                self.error(codes::UNKNOWN_FIELD, name.span, message);
                for arg in args {
                    self.expr(&arg.value, None);
                }
                self.err_expr(span)
            }
        }
    }

    /// `xs.sort()`, `xs.sort(by: x => key)`, `xs.sort(by: (a, b) => a < b)`, `xs.sort(by: ord)`.
    fn sort(&mut self, base: &ast::Expr, elem: &Ty, args: &[ast::Arg], span: Span) -> Expr {
        let place = self.place(base);
        self.sort_place(place, elem, args, span)
    }

    fn sort_place(&mut self, place: Place, elem: &Ty, args: &[ast::Arg], span: Span) -> Expr {
        // `desc: true`: the same sort asking its comparison the other way
        // round, so the largest comes first and equal keys stay in the
        // order they had.
        if let Some(at) = args.iter().position(|arg| arg.name.as_ref().is_some_and(|name| name.text == "desc")) {
            let rest: Vec<ast::Arg> = args.iter().enumerate().filter(|(k, _)| *k != at).map(|(_, arg)| arg.clone()).collect();
            let elem = elem.clone();
            return self.do_expr(Some(&Ty::Unit), span, |lw, out| {
                let desc = lw.expect(&args[at].value, &Ty::Bool);
                let desc = lw.hold(desc, out);
                let place = lw.settle_indexes(place, out);
                let mut sort = lw.sort_place(place, &elem, &rest, span);
                if let ExprKind::Sort(_, _, turned) = &mut sort.kind {
                    *turned = Some(Box::new(desc));
                }
                out.push(Stmt::Expr(sort));
                out.push(Stmt::Give(lw.mk(ExprKind::Unit, Ty::Unit, span)));
            });
        }
        // `by: arr` where `arr: [D]T`: an array indexed by the element is the key `x => arr[x]`.
        if let [ast::Arg {
            name: Some(by),
            value,
            mutable,
        }] = args
        {
            if by.text == "by"
                && !matches!(value.kind, ast::ExprKind::Lambda { .. })
                && matches!(
                    self.probe_ty_silent(value),
                    Some(Ty::Array { dims, .. }) if dims.len() == 1
                )
            {
                let at = value.span;
                let idx_expr = ast::Expr {
                    kind: ast::ExprKind::Index {
                        base: Box::new(value.clone()),
                        indices: vec![ast::Expr {
                            kind: ast::ExprKind::Name("__k".to_string()),
                            span: at,
                        }],
                    },
                    span: at,
                };
                let lambda = ast::Expr {
                    kind: ast::ExprKind::Lambda {
                        params: vec![ast::Pattern {
                            kind: ast::PatternKind::Name("__k".to_string()),
                            span: at,
                        }],
                        param_tys: vec![None],
                        body: Box::new(idx_expr),
                    },
                    span: at,
                };
                let keyed = [ast::Arg {
                    name: Some(by.clone()),
                    mutable: *mutable,
                    value: lambda,
                }];
                return self.sort_place(place, elem, &keyed, span);
            }
        }
        // `by: desc`: a fn by its name is the key `x => desc(x)`.
        if let [ast::Arg {
            name: Some(by),
            value:
                value @ ast::Expr {
                    kind: ast::ExprKind::Name(key),
                    ..
                },
            mutable,
        }] = args
        {
            if by.text == "by" && self.lookup(key).is_none() {
                let at = value.span;
                let name = |text: &str| ast::Expr {
                    kind: ast::ExprKind::Name(text.to_string()),
                    span: at,
                };
                let call = ast::Expr {
                    kind: ast::ExprKind::Call {
                        callee: Box::new(name(key)),
                        args: vec![ast::Arg {
                            name: None,
                            mutable: false,
                            value: name("x"),
                        }],
                    },
                    span: at,
                };
                let lambda = ast::Expr {
                    kind: ast::ExprKind::Lambda {
                        params: vec![ast::Pattern {
                            kind: ast::PatternKind::Name("x".to_string()),
                            span: at,
                        }],
                        param_tys: Vec::new(),
                        body: Box::new(call),
                    },
                    span: at,
                };
                let keyed = [ast::Arg {
                    name: Some(by.clone()),
                    mutable: *mutable,
                    value: lambda,
                }];
                return self.sort_place(place, elem, &keyed, span);
            }
        }
        let has_by = args.iter().any(|a| a.name.as_ref().is_some_and(|n| n.text == "by"));
        let has_cmp = args.iter().any(|a| a.name.as_ref().is_some_and(|n| n.text == "cmp"));
        if has_by && has_cmp {
            self.error(codes::BAD_OPERANDS, span, "`sort` takes either `by:` or `cmp:`, not both");
            return self.err_expr(span);
        }
        let key = match args {
            [] => {
                if !self.ordered(elem) {
                    let message = format!("a `{}` has no order to sort by", self.show(elem));
                    self.error(codes::BAD_OPERANDS, span, message);
                    self.help("sort by a key (`xs.sort(by: x => x.w)`) or comparator (`xs.sort(cmp: (a, b) => a < b)`)");
                    return self.err_expr(span);
                }
                SortBy::Natural
            }
            [ast::Arg {
                name: Some(cmp),
                value,
                ..
            }] if cmp.text == "cmp" => {
                let cmp_hint = Ty::Fn(vec![elem.clone(), elem.clone()], Box::new(Ty::Bool));
                let fv = self.expr(value, Some(&cmp_hint));
                match &fv.ty {
                    Ty::Fn(ps, ret) if ps.len() == 2 && matches!(**ret, Ty::Bool | Ty::Error) => SortBy::Fn {
                        func: Box::new(fv),
                        is_order: true,
                    },
                    Ty::Error => return self.err_expr(span),
                    _ => {
                        self.error(
                            codes::BAD_OPERANDS,
                            value.span,
                            "`cmp:` expects a comparator `(a, b) -> Bool`",
                        );
                        return self.err_expr(span);
                    }
                }
            }
            [ast::Arg {
                name: Some(by),
                value:
                    ast::Expr {
                        kind: ast::ExprKind::Lambda { params, body, .. },
                        ..
                    },
                ..
            }] if by.text == "by" && params.len() == 1 => {
                self.cx.scopes.push(HashMap::new());
                let mut pre = Vec::new();
                let slot = self.loop_var(&params[0], elem.clone(), &mut pre);
                let key = self.expr(body, None);
                self.cx.scopes.pop();
                if !pre.is_empty() {
                    self.unsupported(params[0].span, "a pattern as the parameter of a sort key");
                    return self.err_expr(span);
                }
                if key.ty != Ty::Error && (matches!(key.ty, Ty::Opt(_)) || !self.ordered(&key.ty)) {
                    let message = format!("a `{}` has no order to sort by", self.show(&key.ty));
                    self.error(codes::BAD_OPERANDS, body.span, message);
                    return self.err_expr(span);
                }
                SortBy::Key(slot, Box::new(key))
            }
            [ast::Arg {
                name: Some(by),
                value,
                ..
            }] if by.text == "by" => {
                let cmp_hint = Ty::Fn(vec![elem.clone(), elem.clone()], Box::new(Ty::Bool));
                let fv = self.expr(value, Some(&cmp_hint));
                match &fv.ty {
                    Ty::Fn(ps, _) if ps.len() == 2 => SortBy::Fn {
                        func: Box::new(fv),
                        is_order: true,
                    },
                    Ty::Fn(ps, _) if ps.len() == 1 => SortBy::Fn {
                        func: Box::new(fv),
                        is_order: false,
                    },
                    Ty::Struct(sid) => {
                        if let Some((idx, fty @ Ty::Fn(..))) = self.field_of(&Ty::Struct(*sid), "lt") {
                            let lt_fn = self.mk(ExprKind::Field(Box::new(fv), idx), fty, value.span);
                            SortBy::Fn {
                                func: Box::new(lt_fn),
                                is_order: true,
                            }
                        } else {
                            self.error(codes::BAD_OPERANDS, value.span, "`by:` expects a key fn or Order");
                            return self.err_expr(span);
                        }
                    }
                    _ => {
                        self.error(codes::BAD_OPERANDS, value.span, "`by:` expects a key fn or Order");
                        return self.err_expr(span);
                    }
                }
            }
            _ => {
                self.error(
                    codes::WRONG_ARG_COUNT,
                    span,
                    "`sort` takes nothing, or a key: `xs.sort(by: x => x.w)`",
                );
                return self.err_expr(span);
            }
        };
        self.mk(ExprKind::Sort(Box::new(place), key, None), Ty::Unit, span)
    }

    fn domain_field(&mut self, id: DomainId, name: &ast::Name, hint: Option<&Ty>, span: Span) -> Expr {
        if let Some(info) = self.sum_enums.get(&id).cloned() {
            if let Some(tag) = info.variants.iter().position(|(v, _)| *v == name.text) {
                if info.variants[tag].1.is_empty() {
                    return self.encode_sum_variant(id, tag, Vec::new(), span);
                }
                let message = format!(
                    "the variant `{}` holds values: write `{}(...)`",
                    name.text, name.text
                );
                self.error(codes::WRONG_ARG_COUNT, span, message);
                return self.err_expr(span);
            }
        }
        let len = self.mk(ExprKind::DomainLen(id), Ty::Int, span);
        // An empty domain has no first or last position: asking stops the
        // program in the built C++ as it does in `cpl run`.
        if matches!(name.text.as_str(), "first" | "last") && self.subset_base(id).is_none() {
            let zero = self.int_lit(0, span);
            let empty = self.mk(
                ExprKind::Cmp(CmpOp::Le, Box::new(len.clone()), Box::new(zero)),
                Ty::Bool,
                span,
            );
            let message = format!(
                "`{}` is empty: it has no {} position",
                self.domains[id], name.text
            );
            let trap = Stmt::Trap(message, self.span(span));
            self.cx.pending.push(Stmt::If(empty, Block { stmts: vec![trap] }, Block::default()));
        }
        match name.text.as_str() {
            "len" | "size" | "K" | "count" => len,
            "empty" if self.subset_base(id).is_some() => {
                let zero = self.int_lit(0, span);
                self.cast(Cast::Trust, zero, Ty::Pos(id))
            }
            "full" if self.subset_base(id).is_some() => {
                let full = self.int_add(len, -1);
                self.cast(Cast::Trust, full, Ty::Pos(id))
            }
            "first" => {
                let zero = self.int_lit(0, span);
                self.mk(
                    ExprKind::Cast(Cast::FromOrd(id), Box::new(zero)),
                    Ty::Pos(id),
                    span,
                )
            }
            "pairs" => self.domain_pairs_list(id, span),
            "last" => {
                let last = self.int_add(len, -1);
                self.mk(
                    ExprKind::Cast(Cast::FromOrd(id), Box::new(last)),
                    Ty::Pos(id),
                    span,
                )
            }
            "start" => {
                let zero = self.int_lit(0, span);
                self.mk(
                    ExprKind::Cast(Cast::Trust, Box::new(zero)),
                    Ty::Bound(id),
                    span,
                )
            }
            "end" => self.mk(
                ExprKind::Cast(Cast::Trust, Box::new(len)),
                Ty::Bound(id),
                span,
            ),
            "all" => {
                if let Some(Ty::Struct(sid)) = hint {
                    if self.is_bitset_of(*sid, id) {
                        return self.domain_bitset_full(id, span);
                    }
                }
                if let Some(Ty::Pos(sub)) = hint {
                    if self.subset_base(*sub) == Some(id) {
                        let sub_len = self.mk(ExprKind::DomainLen(*sub), Ty::Int, span);
                        let full = self.int_add(sub_len, -1);
                        return self.cast(Cast::Trust, full, Ty::Pos(*sub));
                    }
                }
                self.mk(
                    ExprKind::Range {
                        lo: Box::new(self.int_lit(0, span)),
                        hi: Box::new(len),
                    },
                    Ty::Range(Box::new(Ty::Pos(id))),
                    span,
                )
            }
            "rev" => {
                let range = self.mk(
                    ExprKind::Range {
                        lo: Box::new(self.int_lit(0, span)),
                        hi: Box::new(len),
                    },
                    Ty::Range(Box::new(Ty::Pos(id))),
                    span,
                );
                self.reversed_range(range, span)
            }
            "bounds" => {
                let hi = self.int_add(len, 1);
                self.mk(
                    ExprKind::Range {
                        lo: Box::new(self.int_lit(0, span)),
                        hi: Box::new(hi),
                    },
                    Ty::Range(Box::new(Ty::Bound(id))),
                    span,
                )
            }
            other => {
                self.unsupported(name.span, &format!("`.{other}` of a domain as a value"));
                self.err_expr(span)
            }
        }
    }

    /// Recognizes `V.pairs` or `V.pairs()` where `V` is a domain, returning its `DomainId`.
    pub(super) fn domain_pairs_iter(&self, iter: &ast::Expr) -> Option<DomainId> {
        match &iter.kind {
            ast::ExprKind::Field { base, name } if name.text == "pairs" => {
                if let Some(Sym::Domain(id)) = self.sym(base) {
                    return Some(id);
                }
            }
            ast::ExprKind::Call { callee, args } if args.is_empty() => {
                if let ast::ExprKind::Field { base, name } = &callee.kind {
                    if name.text == "pairs" {
                        if let Some(Sym::Domain(id)) = self.sym(base) {
                            return Some(id);
                        }
                    }
                }
            }
            _ => {}
        }
        None
    }

    pub(super) fn is_bitset_of(&self, sid: StructId, dom: DomainId) -> bool {
        let Some((gid, gargs)) = self.structs[sid].origin_gen.as_ref() else {
            return false;
        };
        self.gen_structs[*gid].decl.name.text == "Bitset"
            && matches!(
                gargs.as_slice(),
                [GenArg::Domain(d) | GenArg::Ty(Ty::Pos(d))] if *d == dom
            )
    }

    pub(super) fn instantiate_bitset(&mut self, dom: DomainId, span: Span) -> Option<StructId> {
        let Some(Sym::GenStruct(gid)) = self.get_sym("Bitset") else {
            self.error(
                codes::UNKNOWN_TYPE,
                span,
                "`Bitset` is not in scope (import `std.bitset.{Bitset}`)",
            );
            return None;
        };
        Some(self.instantiate_struct(gid, vec![GenArg::Domain(dom)]))
    }

    pub(super) fn domain_bitset_full(&mut self, id: DomainId, span: Span) -> Expr {
        let Some(sid) = self.instantiate_bitset(id, span) else {
            return self.err_expr(span);
        };
        let Some(&full_fn) = self.structs[sid].methods.get("full") else {
            return self.err_expr(span);
        };
        self.queue_fn(full_fn);
        self.mk(ExprKind::Call(full_fn, Vec::new()), Ty::Struct(sid), span)
    }

    /// Lowers `a.inverse()` on a 1D array:
    /// - `[D]?E` (partial injection) -> `[E]?D`, trapping if two positions map to the same value;
    /// - `[D]E` (total bijection) -> `[E]D`, trapping if a value is repeated or missing.
    fn lower_opt_map_or_filter(
        &mut self,
        base: &ast::Expr,
        fn_ast: &ast::Expr,
        is_map: bool,
        span: Span,
    ) -> Expr {
        let opt_val = self.expr(base, None);
        let Ty::Opt(inner_ty) = opt_val.ty.clone() else {
            return self.err_expr(span);
        };
        let opt_slot = self.temp(opt_val.ty.clone());
        let held = self.local(opt_slot, span);
        let is_some = self.mk(ExprKind::IsSome(Box::new(held.clone())), Ty::Bool, span);
        let unwrapped = self.mk(
            ExprKind::Unwrap(Box::new(held), false),
            (*inner_ty).clone(),
            span,
        );
        let val_slot = self.temp((*inner_ty).clone());
        let val_expr = self.local(val_slot, span);

        let mut then_stmts = vec![Stmt::Let(val_slot, Some(unwrapped))];
        let applied = if let ast::ExprKind::Lambda { params, body, .. } = &fn_ast.kind {
            if params.len() == 1 {
                self.cx.scopes.push(HashMap::new());
                let p_slot = self.loop_var(&params[0], (*inner_ty).clone(), &mut then_stmts);
                let init_p = self.local(val_slot, span);
                then_stmts.push(Stmt::Let(p_slot, Some(init_p)));
                let res = if is_map {
                    self.expr(body, None)
                } else {
                    self.expect(body, &Ty::Bool)
                };
                self.cx.scopes.pop();
                res
            } else {
                self.err_expr(span)
            }
        } else {
            let callee_val = self.expr(fn_ast, None);
            match callee_val.ty.clone() {
                Ty::Fn(_, ret_ty) => self.mk(
                    ExprKind::CallExpr(Box::new(callee_val), vec![Arg::Value(val_expr.clone())]),
                    *ret_ty,
                    span,
                ),
                Ty::Array { elem, .. } => self.mk(
                    ExprKind::Index(Box::new(callee_val), vec![val_expr.clone()]),
                    *elem,
                    span,
                ),
                _ => {
                    self.error(
                        codes::TYPE_MISMATCH,
                        fn_ast.span,
                        format!(
                            "`{}` expects a function, lambda, or array",
                            if is_map { "map" } else { "filter" }
                        ),
                    );
                    self.err_expr(span)
                }
            }
        };

        let out_ty = if is_map {
            Ty::Opt(Box::new(applied.ty.clone()))
        } else {
            Ty::Opt(inner_ty.clone())
        };
        let res_slot = self.temp(out_ty.clone());
        let none_expr = self.mk(ExprKind::None, out_ty.clone(), span);
        let res_place = Place {
            kind: PlaceKind::Local(res_slot),
            ty: out_ty.clone(),
            span,
        };
        if is_map {
            let some_expr = self.mk(ExprKind::Some(Box::new(applied)), out_ty.clone(), span);
            then_stmts.push(Stmt::Assign(res_place, AssignOp::Set, some_expr));
        } else {
            let some_val = self.mk(ExprKind::Some(Box::new(val_expr)), out_ty.clone(), span);
            then_stmts.push(Stmt::If(
                applied,
                Block {
                    stmts: vec![Stmt::Assign(res_place, AssignOp::Set, some_val)],
                },
                Block::default(),
            ));
        }
        let res_expr = self.local(res_slot, span);
        let stmts = vec![
            Stmt::Let(opt_slot, Some(opt_val)),
            Stmt::Let(res_slot, Some(none_expr)),
            Stmt::If(is_some, Block { stmts: then_stmts }, Block::default()),
            Stmt::Give(res_expr),
        ];
        self.mk(ExprKind::Do(Block { stmts }), out_ty, span)
    }

    fn lower_array_inverse(&mut self, base: &ast::Expr, span: Span) -> Expr {
        let base_ty_ast = if let ast::ExprKind::Name(n) = &base.kind {
            if let Some(slot) = self.lookup(n) {
                self.cx.local_specs.get(&slot).cloned()
            } else if let Some(Sym::Global(gid)) = self.sym(base) {
                self.global_specs.get(&gid).cloned()
            } else {
                None
            }
        } else {
            None
        };
        let arr = self.expr(base, None);
        let Ty::Array { dims, elem } = arr.ty.clone() else {
            return self.err_expr(span);
        };
        if dims.len() != 1 {
            return self.err_expr(span);
        }
        let src_dim = dims[0];
        let (is_partial, val_ty) = match *elem {
            Ty::Opt(inner) => (true, *inner),
            other => (false, other),
        };
        let target_dim_info: Option<(DimSpec, Expr, Expr)> = match &val_ty {
            Ty::Pos(d) => Some((
                DimSpec::Pos(*d),
                self.int_lit(0, span),
                self.mk(ExprKind::DomainLen(*d), Ty::Int, span),
            )),
            Ty::Bound(d) => {
                let len = self.mk(ExprKind::DomainLen(*d), Ty::Int, span);
                let one = self.int_lit(1, span);
                let hi = self.mk(
                    ExprKind::Binary(BinOp::Add, Box::new(len), Box::new(one)),
                    Ty::Int,
                    span,
                );
                Some((DimSpec::Bound(*d), self.int_lit(0, span), hi))
            }
            Ty::Enum(e) => Some((
                DimSpec::Enum(*e),
                self.int_lit(0, span),
                self.int_lit(self.enums[*e].variants.len() as i64, span),
            )),
            Ty::Int => {
                let base_spec = base_ty_ast.as_ref().map(|ty_ast| self.resolve(ty_ast, true));
                let range_spec = base_spec.and_then(|s| match s {
                    Spec::Array { elem: es, .. } => match *es {
                        Spec::Opt(inner, _) if is_partial => Some(*inner),
                        other if !is_partial => Some(other),
                        _ => None,
                    },
                    _ => None,
                });
                match range_spec {
                    Some(Spec::Int {
                        lo: Some(lo_e),
                        hi: Some(hi_e),
                        ..
                    }) => {
                        let one = self.int_lit(1, span);
                        let diff = self.mk(
                            ExprKind::Binary(BinOp::Sub, hi_e.clone(), lo_e.clone()),
                            Ty::Int,
                            span,
                        );
                        let len = self.mk(
                            ExprKind::Binary(BinOp::Add, Box::new(diff), Box::new(one)),
                            Ty::Int,
                            span,
                        );
                        match lo_e.kind {
                            ExprKind::Int(0) => Some((
                                DimSpec::Int(Box::new(len.clone())),
                                self.int_lit(0, span),
                                len,
                            )),
                            ExprKind::Int(c) => {
                                let hi_excl = self.mk(
                                    ExprKind::Binary(
                                        BinOp::Add,
                                        Box::new(self.int_lit(c, span)),
                                        Box::new(len.clone()),
                                    ),
                                    Ty::Int,
                                    span,
                                );
                                Some((
                                    DimSpec::IntFrom(c, Box::new(len)),
                                    self.int_lit(c, span),
                                    hi_excl,
                                ))
                            }
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        };
        let Some((target_dim, target_lo, target_hi)) = target_dim_info else {
            self.unsupported(
                span,
                "`inverse` on an array whose element type is not an index domain or bounded range",
            );
            return self.err_expr(span);
        };
        let src_idx_ty = match &src_dim {
            Dim::Pos(d) => Ty::Pos(*d),
            Dim::Bound(d) => Ty::Bound(*d),
            Dim::Enum(e) => Ty::Enum(*e),
            Dim::Int | Dim::IntFrom(_) => Ty::Int,
            Dim::Sym(_) => {
                self.unsupported(span, "`inverse` on a symmetric array");
                return self.err_expr(span);
            }
        };
        let inv_ty = Ty::Array {
            dims: vec![target_dim.dim()],
            elem: Box::new(Ty::opt(src_idx_ty.clone())),
        };
        let ret_ty = if is_partial {
            inv_ty.clone()
        } else {
            Ty::Array {
                dims: vec![target_dim.dim()],
                elem: Box::new(src_idx_ty.clone()),
            }
        };
        self.do_expr(Some(&ret_ty), span, |lw, out| {
            let arr_ty = arr.ty.clone();
            let arr_slot = lw.temp(arr_ty.clone());
            out.push(Stmt::Let(arr_slot, Some(arr)));
            let inv_slot = lw.temp(inv_ty.clone());
            out.push(Stmt::Let(
                inv_slot,
                Some(lw.mk(ExprKind::Fill(vec![target_dim.clone()], None), inv_ty.clone(), span)),
            ));
            let (src_lo, src_hi) = match &src_dim {
                Dim::Pos(d) => (
                    lw.int_lit(0, span),
                    lw.mk(ExprKind::DomainLen(*d), Ty::Int, span),
                ),
                Dim::Bound(d) => {
                    let len = lw.mk(ExprKind::DomainLen(*d), Ty::Int, span);
                    let one = lw.int_lit(1, span);
                    (
                        lw.int_lit(0, span),
                        lw.mk(
                            ExprKind::Binary(BinOp::Add, Box::new(len), Box::new(one)),
                            Ty::Int,
                            span,
                        ),
                    )
                }
                Dim::Enum(e) => (
                    lw.int_lit(0, span),
                    lw.int_lit(lw.enums[*e].variants.len() as i64, span),
                ),
                Dim::Int => {
                    let len = lw.mk(
                        ExprKind::Builtin(Builtin::Len, vec![lw.local(arr_slot, span)]),
                        Ty::Int,
                        span,
                    );
                    (lw.int_lit(0, span), len)
                }
                Dim::IntFrom(c) => {
                    let len = lw.mk(
                        ExprKind::Builtin(Builtin::Len, vec![lw.local(arr_slot, span)]),
                        Ty::Int,
                        span,
                    );
                    let hi = lw.mk(
                        ExprKind::Binary(BinOp::Add, Box::new(lw.int_lit(*c, span)), Box::new(len)),
                        Ty::Int,
                        span,
                    );
                    (lw.int_lit(*c, span), hi)
                }
                Dim::Sym(_) => unreachable!(),
            };
            let i_slot = lw.temp(src_idx_ty.clone());
            let lid = lw.open_loop(None);
            let mut body = Vec::new();
            let cell_ty = if is_partial {
                Ty::opt(val_ty.clone())
            } else {
                val_ty.clone()
            };
            let src_idx_for_arr = |lw: &mut Self, slot: Slot| {
                let raw = lw.local(slot, span);
                match src_dim {
                    Dim::IntFrom(c) => lw.mk(
                        ExprKind::Binary(BinOp::Sub, Box::new(raw), Box::new(lw.int_lit(c, span))),
                        Ty::Int,
                        span,
                    ),
                    Dim::Enum(e) => {
                        let ord = lw.cast(Cast::EnumOrd(e), raw, Ty::Int);
                        lw.cast(Cast::Raw, ord, Ty::Int)
                    }
                    _ => raw,
                }
            };
            let target_dim_copy = target_dim.dim();
            let tgt_idx_for_arr = |lw: &mut Self, slot: Slot| {
                let raw = lw.local(slot, span);
                match target_dim_copy {
                    Dim::IntFrom(c) => lw.mk(
                        ExprKind::Binary(BinOp::Sub, Box::new(raw), Box::new(lw.int_lit(c, span))),
                        Ty::Int,
                        span,
                    ),
                    Dim::Enum(e) => {
                        let ord = lw.cast(Cast::EnumOrd(e), raw, Ty::Int);
                        lw.cast(Cast::Raw, ord, Ty::Int)
                    }
                    _ => raw,
                }
            };
            let i_idx = src_idx_for_arr(lw, i_slot);
            let cell = lw.mk(
                ExprKind::Index(Box::new(lw.local(arr_slot, span)), vec![i_idx]),
                cell_ty.clone(),
                span,
            );
            let record_value = |lw: &mut Self, v_slot: Slot, stmts: &mut Vec<Stmt>| {
                let v_idx = tgt_idx_for_arr(lw, v_slot);
                let existing = lw.mk(
                    ExprKind::Index(
                        Box::new(lw.local(inv_slot, span)),
                        vec![v_idx.clone()],
                    ),
                    Ty::opt(src_idx_ty.clone()),
                    span,
                );
                let is_dup = lw.mk(ExprKind::IsSome(Box::new(existing)), Ty::Bool, span);
                let msg = if is_partial {
                    "inverse requires unique values (duplicate image)"
                } else {
                    "inverse requires a bijection (duplicate image)"
                };
                stmts.push(Stmt::If(
                    is_dup,
                    Block {
                        stmts: vec![Stmt::Trap(msg.to_string(), lw.span(span))],
                    },
                    Block::default(),
                ));
                let inv_place = Place {
                    kind: PlaceKind::Index(
                        Box::new(Place {
                            kind: PlaceKind::Local(inv_slot),
                            ty: inv_ty.clone(),
                            span: lw.span(span),
                        }),
                        vec![v_idx],
                    ),
                    ty: Ty::opt(src_idx_ty.clone()),
                    span: lw.span(span),
                };
                let some_i = lw.mk(
                    ExprKind::Some(Box::new(lw.local(i_slot, span))),
                    Ty::opt(src_idx_ty.clone()),
                    span,
                );
                stmts.push(Stmt::Assign(inv_place, AssignOp::Set, some_i));
            };
            if is_partial {
                let cell_slot = lw.temp(cell_ty);
                body.push(Stmt::Let(cell_slot, Some(cell)));
                let has_val = lw.mk(
                    ExprKind::IsSome(Box::new(lw.local(cell_slot, span))),
                    Ty::Bool,
                    span,
                );
                let mut then_stmts = Vec::new();
                let v_slot = lw.temp(val_ty.clone());
                let unwrapped = lw.mk(
                    ExprKind::Unwrap(Box::new(lw.local(cell_slot, span)), false),
                    val_ty.clone(),
                    span,
                );
                then_stmts.push(Stmt::Let(v_slot, Some(unwrapped)));
                record_value(lw, v_slot, &mut then_stmts);
                body.push(Stmt::If(
                    has_val,
                    Block { stmts: then_stmts },
                    Block::default(),
                ));
            } else {
                let v_slot = lw.temp(val_ty.clone());
                body.push(Stmt::Let(v_slot, Some(cell)));
                record_value(lw, v_slot, &mut body);
            }
            lw.cx.loops.pop();
            out.push(Stmt::For(
                lid,
                i_slot,
                Iter::Range {
                    lo: src_lo,
                    hi: src_hi,
                    step: None,
                    rev: false,
                    closed: false,
                },
                Block { stmts: body },
            ));
            if is_partial {
                out.push(Stmt::Give(lw.local(inv_slot, span)));
            } else {
                let out_slot = lw.temp(ret_ty.clone());
                out.push(Stmt::Let(
                    out_slot,
                    Some(lw.mk(ExprKind::Fill(vec![target_dim], None), ret_ty.clone(), span)),
                ));
                let v_slot = lw.temp(val_ty.clone());
                let lid2 = lw.open_loop(None);
                let v_idx = tgt_idx_for_arr(lw, v_slot);
                let existing = lw.mk(
                    ExprKind::Index(
                        Box::new(lw.local(inv_slot, span)),
                        vec![v_idx.clone()],
                    ),
                    Ty::opt(src_idx_ty.clone()),
                    span,
                );
                let unwrapped = lw.mk(ExprKind::Unwrap(Box::new(existing), false), src_idx_ty.clone(), span);
                let out_place = Place {
                    kind: PlaceKind::Index(
                        Box::new(Place {
                            kind: PlaceKind::Local(out_slot),
                            ty: ret_ty.clone(),
                            span: lw.span(span),
                        }),
                        vec![v_idx],
                    ),
                    ty: src_idx_ty.clone(),
                    span: lw.span(span),
                };
                lw.cx.loops.pop();
                out.push(Stmt::For(
                    lid2,
                    v_slot,
                    Iter::Range {
                        lo: target_lo,
                        hi: target_hi,
                        step: None,
                        rev: false,
                        closed: false,
                    },
                    Block {
                        stmts: vec![Stmt::Assign(out_place, AssignOp::Set, unwrapped)],
                    },
                ));
                out.push(Stmt::Give(lw.local(out_slot, span)));
            }
        })
    }

    fn lower_reach(&mut self, base: &ast::Expr, args: &[ast::Arg], span: Span) -> Expr {
        let mut pred_ast: Option<&ast::Expr> = None;
        let mut cyclic_ast: Option<&ast::Expr> = None;
        for arg in args {
            if arg.mutable {
                self.error(codes::WRONG_ARG_COUNT, arg.value.span, "`reach` takes no `mut` arguments");
                return self.err_expr(span);
            }
            match arg.name.as_ref().map(|n| n.text.as_str()) {
                None | Some("while") if pred_ast.is_none() => pred_ast = Some(&arg.value),
                Some("cyclic") if cyclic_ast.is_none() => cyclic_ast = Some(&arg.value),
                Some(other) => {
                    self.error(
                        codes::UNKNOWN_ARG_NAME,
                        arg.value.span,
                        format!("`reach` has no `{other}:` option (expected `while:` and optional `cyclic:`)"),
                    );
                    return self.err_expr(span);
                }
                None => {
                    self.error(codes::WRONG_ARG_COUNT, arg.value.span, "`reach` takes one `while:` predicate and optional `cyclic: Bool`");
                    return self.err_expr(span);
                }
            }
        }
        let Some(pred_ast) = pred_ast else {
            self.error(codes::WRONG_ARG_COUNT, span, "`reach` requires a `while: (T, T) -> Bool` predicate");
            return self.err_expr(span);
        };
        let (seq_val, elem_ty) = if let Some(Sym::Domain(d)) = self.sym(base) {
            (self.positions(d, base.span), Ty::Pos(d))
        } else {
            let v = self.expr(base, None);
            if matches!(v.ty, Ty::Error) {
                return self.err_expr(span);
            }
            match &v.ty {
                Ty::List(e) | Ty::ListView(e) | Ty::MutView(e) => {
                    let e = (**e).clone();
                    (v, e)
                }
                Ty::Array { dims, elem } if dims.len() == 1 => {
                    let e = (**elem).clone();
                    (v, e)
                }
                Ty::Str | Ty::StrView => (v, Ty::Char),
                Ty::Range(e) => {
                    let e = (**e).clone();
                    let list = self.walked_list(Iter::RangeVal(v, false), e.clone(), base.span);
                    (list, e)
                }
                Ty::Set(e) | Ty::HashSet(e) => {
                    let e = (**e).clone();
                    let list = self.mk(ExprKind::Builtin(Builtin::Sorted, vec![v]), Ty::list(e.clone()), base.span);
                    (list, e)
                }
                other => {
                    self.error(
                        codes::TYPE_MISMATCH,
                        base.span,
                        format!("`reach` expects a sequence, not `{}`", self.show(other)),
                    );
                    return self.err_expr(span);
                }
            }
        };
        let fn_hint = Ty::Fn(vec![elem_ty.clone(), elem_ty], Box::new(Ty::Bool));
        let pred = self.expect(pred_ast, &fn_hint);
        let cyclic = match cyclic_ast {
            Some(c) => self.expect(c, &Ty::Bool),
            None => self.mk(ExprKind::Bool(false), Ty::Bool, span),
        };
        self.mk(
            ExprKind::Builtin(Builtin::Reach, vec![seq_val, pred, cyclic]),
            Ty::list(Ty::Int),
            span,
        )
    }

    fn lower_array_map(&mut self, base: &ast::Expr, func_ast: &ast::Expr, span: Span) -> Expr {
        let arr_val = self.expr(base, None);
        let Ty::Array { dims, elem } = arr_val.ty.clone() else {
            return self.err_expr(span);
        };
        let elem_ty = (*elem).clone();
        if matches!(elem_ty, Ty::Error) {
            return self.err_expr(span);
        }
        let arr_slot = self.temp(arr_val.ty.clone());
        let arr = self.local(arr_slot, span);

        let mut axis_lens = Vec::with_capacity(dims.len());
        let mut dim_specs = Vec::with_capacity(dims.len());
        for (axis, dim) in dims.iter().enumerate() {
            let len_expr = if dims.len() == 1 {
                self.mk(ExprKind::Builtin(Builtin::Len, vec![arr.clone()]), Ty::Int, span)
            } else {
                self.mk(
                    ExprKind::Builtin(
                        Builtin::AxisLen,
                        vec![arr.clone(), self.int_lit(axis as i64, span)],
                    ),
                    Ty::Int,
                    span,
                )
            };
            let spec = match *dim {
                Dim::Pos(d) => DimSpec::Pos(d),
                Dim::Bound(d) => DimSpec::Bound(d),
                Dim::Sym(d) => DimSpec::Sym(d),
                Dim::Enum(e) => DimSpec::Enum(e),
                Dim::Int => DimSpec::Int(Box::new(len_expr.clone())),
                Dim::IntFrom(lo) => DimSpec::IntFrom(lo, Box::new(len_expr.clone())),
            };
            axis_lens.push(len_expr);
            dim_specs.push(spec);
        }

        let counters: Vec<Slot> = dims.iter().map(|_| self.temp(Ty::Int)).collect();
        let stored: Vec<Expr> = dims
            .iter()
            .zip(&counters)
            .map(|(dim, &k)| {
                let k_expr = self.local(k, span);
                match *dim {
                    Dim::Pos(d) => {
                        self.mk(ExprKind::Cast(Cast::FromOrd(d), Box::new(k_expr)), Ty::Pos(d), span)
                    }
                    _ => k_expr,
                }
            })
            .collect();
        let cell = self.mk(
            ExprKind::Index(Box::new(arr.clone()), stored.clone()),
            elem_ty.clone(),
            span,
        );

        let mut pre_stmts = vec![Stmt::Let(arr_slot, Some(arr_val))];
        let (out_elem, mut inner_stmts, mapped) = if let ast::ExprKind::Lambda { params, body, .. } =
            &func_ast.kind
        {
            if params.len() == 1 {
                self.cx.scopes.push(HashMap::new());
                let mut bind_stmts = Vec::new();
                let item = self.loop_var(&params[0], elem_ty.clone(), &mut bind_stmts);
                let mut inner = vec![Stmt::Let(item, Some(cell))];
                inner.extend(bind_stmts);
                let mapped = self.expr(body, None);
                self.cx.scopes.pop();
                if matches!(mapped.ty, Ty::Error) {
                    return self.err_expr(span);
                }
                (mapped.ty.clone(), inner, mapped)
            } else {
                return self.err_expr(span);
            }
        } else {
            let fn_hint = Ty::Fn(vec![elem_ty.clone()], Box::new(Ty::Error));
            let func = self.expr(func_ast, Some(&fn_hint));
            let out_elem = match &func.ty {
                Ty::Fn(_, ret) if !matches!(ret.as_ref(), Ty::Error) => (**ret).clone(),
                Ty::Error => return self.err_expr(span),
                other => {
                    self.error(
                        codes::TYPE_MISMATCH,
                        func_ast.span,
                        format!("`map` expects a function, not `{}`", self.show(other)),
                    );
                    return self.err_expr(span);
                }
            };
            let func_slot = self.temp(func.ty.clone());
            pre_stmts.push(Stmt::Let(func_slot, Some(func)));
            let func_val = self.local(func_slot, span);
            let mapped = self.mk(
                ExprKind::CallExpr(Box::new(func_val), vec![Arg::Value(cell)]),
                out_elem.clone(),
                span,
            );
            (out_elem, Vec::new(), mapped)
        };

        let out_ty = Ty::Array {
            dims,
            elem: Box::new(out_elem.clone()),
        };
        let acc = self.temp(out_ty.clone());
        let blank = self.mk(ExprKind::Fill(dim_specs, None), out_ty.clone(), span);
        pre_stmts.push(Stmt::Let(acc, Some(blank)));

        let whole = Place {
            kind: PlaceKind::Local(acc),
            ty: out_ty.clone(),
            span: self.span(span),
        };
        let dst = Place {
            kind: PlaceKind::Index(Box::new(whole), stored),
            ty: out_elem,
            span: self.span(span),
        };
        inner_stmts.push(Stmt::Assign(dst, AssignOp::Set, mapped));

        let mut body = inner_stmts;
        for (len_expr, &k) in axis_lens.into_iter().zip(&counters).rev() {
            let iter = Iter::Range {
                lo: self.int_lit(0, span),
                hi: len_expr,
                step: None,
                rev: false,
                closed: false,
            };
            let lid = self.fresh_loop();
            body = vec![Stmt::For(lid, k, iter, Block { stmts: body })];
        }
        pre_stmts.extend(body);
        pre_stmts.push(Stmt::Give(self.local(acc, span)));
        self.mk(ExprKind::Do(Block { stmts: pre_stmts }), out_ty, span)
    }

    fn lower_map(&mut self, base: &ast::Expr, func_ast: &ast::Expr, span: Span) -> Expr {
        let (iter, elem_ty) = self.iter(base);
        if matches!(elem_ty, Ty::Error) {
            return self.err_expr(span);
        }
        if let ast::ExprKind::Lambda { params, body, .. } = &func_ast.kind {
            if params.len() == 1 {
                self.cx.scopes.push(HashMap::new());
                let mut bind_stmts = Vec::new();
                let item = self.loop_var(&params[0], elem_ty.clone(), &mut bind_stmts);
                let mapped = self.expr(body, None);
                self.cx.scopes.pop();
                if matches!(mapped.ty, Ty::Error) {
                    return self.err_expr(span);
                }
                let out_elem = mapped.ty.clone();
                let list_ty = Ty::list(out_elem);
                return self.do_expr(Some(&list_ty), span, |lw, out| {
                    let acc = lw.temp(list_ty.clone());
                    let empty = lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span);
                    out.push(Stmt::Let(acc, Some(empty)));
                    let push = Stmt::Expr(lw.mk(
                        ExprKind::Mutate(
                            Mutation::Push,
                            Box::new(Place {
                                kind: PlaceKind::Local(acc),
                                ty: list_ty.clone(),
                                span: lw.span(span),
                            }),
                            vec![mapped],
                        ),
                        Ty::Unit,
                        span,
                    ));
                    let mut stmts = bind_stmts;
                    stmts.push(push);
                    let lid = lw.fresh_loop();
                    out.push(Stmt::For(lid, item, iter, Block { stmts }));
                    out.push(Stmt::Give(lw.local(acc, span)));
                });
            }
        }
        let fn_hint = Ty::Fn(vec![elem_ty.clone()], Box::new(Ty::Error));
        let func = self.expr(func_ast, Some(&fn_hint));
        let out_elem = match &func.ty {
            Ty::Fn(_, ret) if !matches!(ret.as_ref(), Ty::Error) => (**ret).clone(),
            Ty::Error => return self.err_expr(span),
            other => {
                self.error(
                    codes::TYPE_MISMATCH,
                    func_ast.span,
                    format!("`map` expects a function, not `{}`", self.show(other)),
                );
                return self.err_expr(span);
            }
        };
        self.collected(out_elem.clone(), span, |lw, out, push| {
            let func_val = lw.hold(func, out);
            let item = lw.temp(elem_ty.clone());
            let lid = lw.fresh_loop();
            let item_expr = lw.local(item, span);
            let mapped = lw.mk(
                ExprKind::CallExpr(Box::new(func_val), vec![Arg::Value(item_expr)]),
                out_elem.clone(),
                span,
            );
            let push_stmt = push(lw, mapped);
            out.push(Stmt::For(lid, item, iter, Block { stmts: vec![push_stmt] }));
        })
    }

    fn lower_filter(&mut self, base: &ast::Expr, pred_ast: &ast::Expr, span: Span) -> Expr {
        let (iter, elem_ty) = self.iter(base);
        if matches!(elem_ty, Ty::Error) {
            return self.err_expr(span);
        }
        if let ast::ExprKind::Lambda { params, body, .. } = &pred_ast.kind {
            if params.len() == 1 {
                return self.collected(elem_ty.clone(), span, |lw, out, push| {
                    lw.cx.scopes.push(HashMap::new());
                    let mut bind_stmts = Vec::new();
                    let item = lw.loop_var(&params[0], elem_ty.clone(), &mut bind_stmts);
                    let keep = lw.expect(body, &Ty::Bool);
                    lw.cx.scopes.pop();
                    let lid = lw.fresh_loop();
                    let item_expr = lw.local(item, span);
                    let push_stmt = push(lw, item_expr);
                    bind_stmts.push(Stmt::If(
                        keep,
                        Block { stmts: vec![push_stmt] },
                        Block::default(),
                    ));
                    out.push(Stmt::For(lid, item, iter, Block { stmts: bind_stmts }));
                });
            }
        }
        let fn_ty = Ty::Fn(vec![elem_ty.clone()], Box::new(Ty::Bool));
        let pred = self.expect(pred_ast, &fn_ty);
        self.collected(elem_ty.clone(), span, |lw, out, push| {
            let pred_val = lw.hold(pred, out);
            let item = lw.temp(elem_ty.clone());
            let lid = lw.fresh_loop();
            let item_expr = lw.local(item, span);
            let keep = lw.mk(
                ExprKind::CallExpr(Box::new(pred_val), vec![Arg::Value(item_expr.clone())]),
                Ty::Bool,
                span,
            );
            let push_stmt = push(lw, item_expr);
            let body = Block {
                stmts: vec![Stmt::If(keep, Block { stmts: vec![push_stmt] }, Block::default())],
            };
            out.push(Stmt::For(lid, item, iter, body));
        })
    }

    fn lower_partition(&mut self, base: &ast::Expr, pred_ast: &ast::Expr, span: Span) -> Expr {
        let (iter, elem_ty) = self.iter(base);
        if matches!(elem_ty, Ty::Error) {
            return self.err_expr(span);
        }
        let fn_ty = Ty::Fn(vec![elem_ty.clone()], Box::new(Ty::Bool));
        let pred = self.expect(pred_ast, &fn_ty);
        let list_ty = Ty::list(elem_ty.clone());
        let pair_ty = Ty::Tuple(vec![list_ty.clone(), list_ty.clone()]);
        self.do_expr(Some(&pair_ty), span, |lw, out| {
            let pred_val = lw.hold(pred, out);
            let yes = lw.temp(list_ty.clone());
            let no = lw.temp(list_ty.clone());
            let empty_yes = lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span);
            let empty_no = lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span);
            out.push(Stmt::Let(yes, Some(empty_yes)));
            out.push(Stmt::Let(no, Some(empty_no)));
            let item = lw.temp(elem_ty.clone());
            let lid = lw.fresh_loop();
            let item_expr = lw.local(item, span);
            let cond = lw.mk(
                ExprKind::CallExpr(Box::new(pred_val), vec![Arg::Value(item_expr.clone())]),
                Ty::Bool,
                span,
            );
            let push_yes = lw.mk(
                ExprKind::Mutate(
                    Mutation::Push,
                    Box::new(Place {
                        kind: PlaceKind::Local(yes),
                        ty: list_ty.clone(),
                        span: lw.span(span),
                    }),
                    vec![item_expr.clone()],
                ),
                Ty::Unit,
                span,
            );
            let push_no = lw.mk(
                ExprKind::Mutate(
                    Mutation::Push,
                    Box::new(Place {
                        kind: PlaceKind::Local(no),
                        ty: list_ty.clone(),
                        span: lw.span(span),
                    }),
                    vec![item_expr],
                ),
                Ty::Unit,
                span,
            );
            let body = Block {
                stmts: vec![Stmt::If(
                    cond,
                    Block { stmts: vec![Stmt::Expr(push_yes)] },
                    Block { stmts: vec![Stmt::Expr(push_no)] },
                )],
            };
            out.push(Stmt::For(lid, item, iter, body));
            let res = lw.mk(
                ExprKind::Tuple(vec![lw.local(yes, span), lw.local(no, span)]),
                pair_ty.clone(),
                span,
            );
            out.push(Stmt::Give(res));
        })
    }

    fn lower_first_where(&mut self, base: &ast::Expr, pred_ast: &ast::Expr, span: Span) -> Expr {
        let (iter, elem_ty) = self.iter(base);
        if matches!(elem_ty, Ty::Error) {
            return self.err_expr(span);
        }
        let fn_ty = Ty::Fn(vec![elem_ty.clone()], Box::new(Ty::Bool));
        let pred = self.expect(pred_ast, &fn_ty);
        let opt_ty = Ty::opt(elem_ty.clone());
        self.do_expr(Some(&opt_ty), span, |lw, out| {
            let pred_val = lw.hold(pred, out);
            let found = lw.temp(opt_ty.clone());
            let init = lw.mk(ExprKind::None, opt_ty.clone(), span);
            out.push(Stmt::Let(found, Some(init)));
            let item = lw.temp(elem_ty.clone());
            let lid = lw.fresh_loop();
            let item_expr = lw.local(item, span);
            let cond = lw.mk(
                ExprKind::CallExpr(Box::new(pred_val), vec![Arg::Value(item_expr.clone())]),
                Ty::Bool,
                span,
            );
            let some_val = lw.mk(ExprKind::Some(Box::new(item_expr)), opt_ty.clone(), span);
            let set_found = Stmt::Assign(
                Place {
                    kind: PlaceKind::Local(found),
                    ty: opt_ty.clone(),
                    span: lw.span(span),
                },
                AssignOp::Set,
                some_val,
            );
            let body = Block {
                stmts: vec![Stmt::If(
                    cond,
                    Block {
                        stmts: vec![set_found, Stmt::Break(Some(lid))],
                    },
                    Block::default(),
                )],
            };
            out.push(Stmt::For(lid, item, iter, body));
            out.push(Stmt::Give(lw.local(found, span)));
        })
    }

    fn lower_group_by(&mut self, base: &ast::Expr, key_ast: &ast::Expr, span: Span) -> Expr {
        let (iter, elem_ty) = self.iter(base);
        if matches!(elem_ty, Ty::Error) {
            return self.err_expr(span);
        }
        let mut item = self.temp(elem_ty.clone());
        // Evaluate the key expression for `item`: supports `x => expr`, a 1D array `a` (`a[x]`), or a callable `f` (`f(x)`).
        let (pre_stmts, key_expr): (Vec<Stmt>, Expr) = match &key_ast.kind {
            ast::ExprKind::Lambda { params, body, .. } if params.len() == 1 => {
                self.cx.scopes.push(HashMap::new());
                let mut bind_stmts = Vec::new();
                item = self.loop_var(&params[0], elem_ty.clone(), &mut bind_stmts);
                let ke = self.expr(body, None);
                self.cx.scopes.pop();
                (bind_stmts, ke)
            }
            _ => {
                let item_expr = self.local(item, span);
                if matches!(self.probe_ty_silent(key_ast), Some(Ty::Array { dims, .. }) if dims.len() == 1)
                {
                    let arr = self.expr(key_ast, None);
                    let arr_elem = match &arr.ty {
                        Ty::Array { elem, .. } => (**elem).clone(),
                        _ => Ty::Error,
                    };
                    let idx = match &arr.ty {
                        Ty::Array { dims, .. } => match dims[0] {
                            Dim::Enum(e) => {
                                let ord = self.cast(Cast::EnumOrd(e), item_expr.clone(), Ty::Int);
                                self.cast(Cast::Raw, ord, Ty::Int)
                            }
                            _ => item_expr.clone(),
                        },
                        _ => item_expr.clone(),
                    };
                    (
                        Vec::new(),
                        self.mk(ExprKind::Index(Box::new(arr), vec![idx]), arr_elem, span),
                    )
                } else {
                    let f = self.expr(key_ast, None);
                    let ret = match &f.ty {
                        Ty::Fn(_, r) => (**r).clone(),
                        _ => Ty::Error,
                    };
                    (
                        Vec::new(),
                        self.mk(
                            ExprKind::CallExpr(Box::new(f), vec![Arg::Value(item_expr.clone())]),
                            ret,
                            span,
                        ),
                    )
                }
            }
        };
        let item_expr = self.local(item, span);
        let key_ty = key_expr.ty.clone();
        if matches!(key_ty, Ty::Error) {
            return self.err_expr(span);
        }
        let group_ty = Ty::list(elem_ty.clone());
        let groups_ty = Ty::list(group_ty.clone());
        let map_ty = Ty::Map(Box::new(key_ty.clone()), Box::new(Ty::Int));
        let opt_int = Ty::opt(Ty::Int);
        self.do_expr(Some(&groups_ty), span, |lw, out| {
            let groups = lw.temp(groups_ty.clone());
            let seen = lw.temp(map_ty.clone());
            out.push(Stmt::Let(
                groups,
                Some(lw.mk(ExprKind::Tuple(Vec::new()), groups_ty.clone(), span)),
            ));
            out.push(Stmt::Let(
                seen,
                Some(lw.mk(ExprKind::Tuple(Vec::new()), map_ty.clone(), span)),
            ));
            let lid = lw.fresh_loop();
            let mut body_stmts = pre_stmts;
            let k_slot = lw.temp(key_ty.clone());
            body_stmts.push(Stmt::Let(k_slot, Some(key_expr)));
            let k_expr = lw.local(k_slot, span);
            let got = lw.mk(
                ExprKind::Builtin(Builtin::MapGet, vec![lw.local(seen, span), k_expr.clone()]),
                opt_int.clone(),
                span,
            );
            let got_slot = lw.temp(opt_int.clone());
            body_stmts.push(Stmt::Let(got_slot, Some(got)));
            let is_some = lw.mk(
                ExprKind::IsSome(Box::new(lw.local(got_slot, span))),
                Ty::Bool,
                span,
            );
            let idx = lw.mk(
                ExprKind::Unwrap(Box::new(lw.local(got_slot, span)), false),
                Ty::Int,
                span,
            );
            let groups_place = Place {
                kind: PlaceKind::Local(groups),
                ty: groups_ty.clone(),
                span: lw.span(span),
            };
            let cell_place = Place {
                kind: PlaceKind::Index(Box::new(groups_place.clone()), vec![idx]),
                ty: group_ty.clone(),
                span: lw.span(span),
            };
            let push_existing = lw.mk(
                ExprKind::Mutate(Mutation::Push, Box::new(cell_place), vec![item_expr.clone()]),
                Ty::Unit,
                span,
            );
            // Else branch: seen[k] = groups.len; groups.push([item])
            let cur_len = lw.mk(
                ExprKind::Builtin(Builtin::Len, vec![lw.local(groups, span)]),
                Ty::Int,
                span,
            );
            let seen_place = Place {
                kind: PlaceKind::Index(
                    Box::new(Place {
                        kind: PlaceKind::Local(seen),
                        ty: map_ty.clone(),
                        span: lw.span(span),
                    }),
                    vec![k_expr],
                ),
                ty: Ty::Int,
                span: lw.span(span),
            };
            let set_seen = Stmt::Assign(seen_place, AssignOp::Set, cur_len);
            let new_group = lw.mk(ExprKind::Tuple(vec![item_expr]), group_ty.clone(), span);
            let push_new = lw.mk(
                ExprKind::Mutate(Mutation::Push, Box::new(groups_place), vec![new_group]),
                Ty::Unit,
                span,
            );
            body_stmts.push(Stmt::If(
                is_some,
                Block { stmts: vec![Stmt::Expr(push_existing)] },
                Block { stmts: vec![set_seen, Stmt::Expr(push_new)] },
            ));
            out.push(Stmt::For(lid, item, iter, Block { stmts: body_stmts }));
            out.push(Stmt::Give(lw.local(groups, span)));
        })
    }

    /// `adjacency(edges)` -> `[V]List<V>` and `adjacency(edges, into: Bitset)` -> `[V]Bitset<V>`.
    fn lower_adjacency(&mut self, args: &[ast::Arg], span: Span) -> Expr {
        if args.is_empty() || args.len() > 2 {
            self.error(
                codes::WRONG_ARG_COUNT,
                span,
                "`adjacency` takes `(edges)` or `(edges, into: Bitset)`",
            );
            return self.err_expr(span);
        }
        let mut into_bitset = false;
        if let Some(opt) = args.get(1) {
            if opt.name.as_ref().map(|n| n.text.as_str()) == Some("into")
                && matches!(&opt.value.kind, ast::ExprKind::Name(n) if n == "Bitset")
            {
                into_bitset = true;
            } else {
                self.error(
                    codes::TYPE_MISMATCH,
                    opt.value.span,
                    "`adjacency` expects `into: Bitset`",
                );
                return self.err_expr(span);
            }
        }
        // Fuse `edges.filter(e => cond)` to avoid materializing a temporary edge list.
        let (edges_ast, filter_lambda) = match &args[0].value.kind {
            ast::ExprKind::Call { callee, args: fargs } => match (&callee.kind, fargs.as_slice()) {
                (
                    ast::ExprKind::Field { base, name },
                    [ast::Arg {
                        name: None,
                        value:
                            ast::Expr {
                                kind: ast::ExprKind::Lambda { params, body, .. },
                                ..
                            },
                        ..
                    }],
                ) if name.text == "filter" && params.len() == 1 => {
                    (base.as_ref(), Some((&params[0], body.as_ref())))
                }
                _ => (&args[0].value, None),
            },
            _ => (&args[0].value, None),
        };
        let (iter, elem_ty) = self.iter(edges_ast);
        let (dom, u_idx, v_idx) = match &elem_ty {
            Ty::Record(fields) => {
                let u = fields.iter().position(|(n, _)| n == "u");
                let v = fields.iter().position(|(n, _)| n == "v");
                match (u, v) {
                    (Some(ui), Some(vi)) => match (&fields[ui].1, &fields[vi].1) {
                        (Ty::Pos(du), Ty::Pos(dv)) if du == dv => (*du, ui, vi),
                        _ => {
                            self.error(
                                codes::TYPE_MISMATCH,
                                args[0].value.span,
                                "`adjacency` edges must have `u` and `v` in the same vertex domain",
                            );
                            return self.err_expr(span);
                        }
                    },
                    _ => {
                        self.error(
                            codes::TYPE_MISMATCH,
                            args[0].value.span,
                            "`adjacency` edges must be `(u: V, v: V, ...)` or `(V, V)`",
                        );
                        return self.err_expr(span);
                    }
                }
            }
            Ty::Tuple(parts) if parts.len() >= 2 => match (&parts[0], &parts[1]) {
                (Ty::Pos(du), Ty::Pos(dv)) if du == dv => (*du, 0, 1),
                _ => {
                    self.error(
                        codes::TYPE_MISMATCH,
                        args[0].value.span,
                        "`adjacency` edges must have endpoints in the same vertex domain",
                    );
                    return self.err_expr(span);
                }
            },
            Ty::Error => return self.err_expr(span),
            _ => {
                self.error(
                    codes::TYPE_MISMATCH,
                    args[0].value.span,
                    "`adjacency` expects a sequence of `(u: V, v: V, ...)` or `(V, V)` edges",
                );
                return self.err_expr(span);
            }
        };
        let (cell_ty, init_expr, add_fn_opt) = if into_bitset {
            let Some(sid) = self.instantiate_bitset(dom, span) else {
                return self.err_expr(span);
            };
            let Some(&new_fn) = self.structs[sid].methods.get("new") else {
                return self.err_expr(span);
            };
            let Some(&add_fn) = self.structs[sid].methods.get("add") else {
                return self.err_expr(span);
            };
            self.queue_fn(new_fn);
            self.queue_fn(add_fn);
            let init = self.mk(ExprKind::Call(new_fn, Vec::new()), Ty::Struct(sid), span);
            (Ty::Struct(sid), init, Some(add_fn))
        } else {
            let list_ty = Ty::list(Ty::Pos(dom));
            let init = self.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span);
            (list_ty, init, None)
        };
        let arr_ty = Ty::Array {
            dims: vec![Dim::Pos(dom)],
            elem: Box::new(cell_ty.clone()),
        };
        self.do_expr(Some(&arr_ty), span, |lw, out| {
            let acc = lw.temp(arr_ty.clone());
            let fill = lw.mk(
                ExprKind::Fill(vec![DimSpec::Pos(dom)], Some(Box::new(init_expr))),
                arr_ty.clone(),
                span,
            );
            out.push(Stmt::Let(acc, Some(fill)));
            let mut loop_pre = Vec::new();
            let (edge, filter_cond) = if let Some((pat, cond_ast)) = filter_lambda {
                lw.cx.scopes.push(HashMap::new());
                let slot = lw.loop_var(pat, elem_ty.clone(), &mut loop_pre);
                let cond = lw.expect(cond_ast, &Ty::Bool);
                lw.cx.scopes.pop();
                (slot, Some(cond))
            } else {
                (lw.temp(elem_ty.clone()), None)
            };
            let edge_expr = lw.local(edge, span);
            let u_expr = lw.mk(ExprKind::Field(Box::new(edge_expr.clone()), u_idx), Ty::Pos(dom), span);
            let v_expr = lw.mk(ExprKind::Field(Box::new(edge_expr), v_idx), Ty::Pos(dom), span);
            let acc_place = Place {
                kind: PlaceKind::Local(acc),
                ty: arr_ty.clone(),
                span: lw.span(span),
            };
            let u_cell = Place {
                kind: PlaceKind::Index(Box::new(acc_place.clone()), vec![u_expr.clone()]),
                ty: cell_ty.clone(),
                span: lw.span(span),
            };
            let v_cell = Place {
                kind: PlaceKind::Index(Box::new(acc_place), vec![v_expr.clone()]),
                ty: cell_ty.clone(),
                span: lw.span(span),
            };
            let mut body_stmts = Vec::new();
            if let Some(add_fn) = add_fn_opt {
                let c1 = lw.mk(
                    ExprKind::Call(add_fn, vec![Arg::Mut(u_cell), Arg::Value(v_expr)]),
                    Ty::Unit,
                    span,
                );
                let c2 = lw.mk(
                    ExprKind::Call(add_fn, vec![Arg::Mut(v_cell), Arg::Value(u_expr)]),
                    Ty::Unit,
                    span,
                );
                body_stmts.push(Stmt::Expr(c1));
                body_stmts.push(Stmt::Expr(c2));
            } else {
                let p1 = lw.mk(
                    ExprKind::Mutate(Mutation::Push, Box::new(u_cell), vec![v_expr]),
                    Ty::Unit,
                    span,
                );
                let p2 = lw.mk(
                    ExprKind::Mutate(Mutation::Push, Box::new(v_cell), vec![u_expr]),
                    Ty::Unit,
                    span,
                );
                body_stmts.push(Stmt::Expr(p1));
                body_stmts.push(Stmt::Expr(p2));
            }
            let loop_body = if let Some(cond) = filter_cond {
                loop_pre.push(Stmt::If(
                    cond,
                    Block { stmts: body_stmts },
                    Block::default(),
                ));
                Block { stmts: loop_pre }
            } else {
                Block { stmts: body_stmts }
            };
            let lid = lw.fresh_loop();
            out.push(Stmt::For(lid, edge, iter, loop_body));
            out.push(Stmt::Give(lw.local(acc, span)));
        })
    }

    /// Materializes `V.pairs` as `List<(V, V)>` for all `0 <= i < j < V.count`.
    pub(super) fn domain_pairs_list(&mut self, id: DomainId, span: Span) -> Expr {
        let pair_ty = Ty::Tuple(vec![Ty::Pos(id), Ty::Pos(id)]);
        self.collected(pair_ty.clone(), span, |lw, out, push| {
            let len = lw.mk(ExprKind::DomainLen(id), Ty::Int, span);
            let i_slot = lw.temp(Ty::Pos(id));
            let j_slot = lw.temp(Ty::Pos(id));
            let i_var = lw.mk(ExprKind::Local(i_slot), Ty::Pos(id), span);
            let j_var = lw.mk(ExprKind::Local(j_slot), Ty::Pos(id), span);
            let i_ord = lw.cast(Cast::Ord, i_var.clone(), Ty::Int);
            let j_lo = lw.int_add(i_ord, 1);
            let inner_id = lw.fresh_loop();
            let outer_id = lw.fresh_loop();
            let pair = lw.mk(ExprKind::Tuple(vec![i_var, j_var]), pair_ty, span);
            let push_stmt = push(lw, pair);
            let inner = Stmt::For(
                inner_id,
                j_slot,
                Iter::Range {
                    lo: j_lo,
                    hi: len.clone(),
                    step: None,
                    rev: false,
                    closed: false,
                },
                Block {
                    stmts: vec![push_stmt],
                },
            );
            out.push(Stmt::For(
                outer_id,
                i_slot,
                Iter::Range {
                    lo: lw.int_lit(0, span),
                    hi: len,
                    step: None,
                    rev: false,
                    closed: false,
                },
                Block { stmts: vec![inner] },
            ));
        })
    }

    /// A `[]` list local, the loop that pushes `value` for each `item` of
    /// `iter`, and the list.
    pub(super) fn collected(
        &mut self,
        elem: Ty,
        span: Span,
        build: impl FnOnce(&mut Self, &mut Vec<Stmt>, &dyn Fn(&mut Self, Expr) -> Stmt),
    ) -> Expr {
        let ty = Ty::list(elem);
        self.do_expr(Some(&ty), span, |lw, out| {
            let acc = lw.temp(ty.clone());
            out.push(Stmt::Let(
                acc,
                Some(lw.mk(ExprKind::Tuple(Vec::new()), ty.clone(), span)),
            ));
            let list_ty = ty.clone();
            let push = move |lw: &mut Self, value: Expr| {
                let place = Place {
                    kind: PlaceKind::Local(acc),
                    ty: list_ty.clone(),
                    span: lw.span(span),
                };
                Stmt::Expr(lw.mk(
                    ExprKind::Mutate(Mutation::Push, Box::new(place), vec![value]),
                    Ty::Unit,
                    span,
                ))
            };
            build(lw, out, &push);
            out.push(Stmt::Give(lw.local(acc, span)));
        })
    }

    fn collect_iter_to_list(&mut self, src: &ast::Expr, span: Span) -> Expr {
        let (iter, elem) = self.iter(src);
        self.collected(elem.clone(), span, |lw, out, push| {
            let item = lw.temp(elem);
            let lid = lw.fresh_loop();
            let value = lw.local(item, span);
            let body = Block {
                stmts: vec![push(lw, value)],
            };
            out.push(Stmt::For(lid, item, iter, body));
        })
    }

    /// `xs.adjacent`: the pairs `(xs[i], xs[i + 1])`, as a list.
    fn lower_adjacent(&mut self, value: Expr, span: Span) -> Expr {
        let (elem_ty, dim) = match &value.ty {
            Ty::List(e) | Ty::ListView(e) | Ty::MutView(e) => ((**e).clone(), Dim::Int),
            Ty::Str | Ty::StrView => (Ty::Char, Dim::Int),
            Ty::Array { dims, elem } if dims.len() == 1 => ((**elem).clone(), dims[0]),
            _ => return self.err_expr(span),
        };
        let pair_ty = Ty::Tuple(vec![elem_ty.clone(), elem_ty.clone()]);
        let reusable = matches!(value.kind, ExprKind::Local(_) | ExprKind::Global(_));
        self.collected(pair_ty.clone(), span, |lw, out, push| {
            let seq = if reusable {
                value
            } else {
                let held = lw.temp(value.ty.clone());
                out.push(Stmt::Let(held, Some(value)));
                lw.local(held, span)
            };
            let len = lw.len_of(seq.clone(), span);
            let hi = lw.int_add(len, -1);
            let lo = lw.int_lit(0, span);
            let slot = lw.temp(Ty::Int);
            let lid = lw.fresh_loop();
            let at = |lw: &mut Self, offset: i64| {
                let i = lw.local(slot, span);
                let i = if offset == 0 { i } else { lw.int_add(i, offset) };
                let index = match dim {
                    Dim::Pos(d) => lw.cast(Cast::Trust, i, Ty::Pos(d)),
                    Dim::Bound(d) => lw.cast(Cast::Trust, i, Ty::Bound(d)),
                    Dim::Enum(e) => lw.cast(Cast::EnumOrd(e), i, Ty::Int),
                    Dim::Sym(_) | Dim::Int | Dim::IntFrom(_) => i,
                };
                lw.mk(
                    ExprKind::Index(Box::new(seq.clone()), vec![index]),
                    elem_ty.clone(),
                    span,
                )
            };
            let (a, b) = (at(lw, 0), at(lw, 1));
            let pair = lw.mk(ExprKind::Tuple(vec![a, b]), pair_ty, span);
            let body = Block {
                stmts: vec![push(lw, pair)],
            };
            out.push(Stmt::For(
                lid,
                slot,
                Iter::Range {
                    lo,
                    hi,
                    step: None,
                    rev: false,
                    closed: false,
                },
                body,
            ));
        })
    }

    /// `span(a, b)`: smallest `Range<D>` covering positions or bounds `a` and `b` of domain `D`.
    fn lower_span(&mut self, args: &[ast::Arg], span: Span) -> Expr {
        let Some(exprs) = self.positional_any("span", args, 2, span) else {
            return self.err_expr(span);
        };
        let a = self.expr(exprs[0], None);
        let b = self.expr(exprs[1], None);
        if a.ty == Ty::Error || b.ty == Ty::Error {
            return self.err_expr(span);
        }
        let dom_of = |ty: &Ty| match ty {
            Ty::Pos(d) => Some((*d, true)),
            Ty::Bound(d) => Some((*d, false)),
            _ => None,
        };
        let (Some((da, a_pos)), Some((db, b_pos))) = (dom_of(&a.ty), dom_of(&b.ty)) else {
            let message = format!(
                "`span` takes two positions or bounds of a domain, found `{}` and `{}`",
                self.show(&a.ty),
                self.show(&b.ty)
            );
            self.error(codes::TYPE_MISMATCH, span, message);
            return self.err_expr(span);
        };
        if da != db {
            let message = format!(
                "`span` takes two positions or bounds of one domain, found `{}` and `{}`",
                self.show(&a.ty),
                self.show(&b.ty)
            );
            self.error(codes::WRONG_DOMAIN, span, message);
            return self.err_expr(span);
        }
        let out_ty = Ty::Range(Box::new(Ty::Pos(da)));
        let ao = self.cast(Cast::Ord, a, Ty::Int);
        let bo = self.cast(Cast::Ord, b, Ty::Int);
        self.do_expr(Some(&out_ty), span, |lw, out| {
            let (aslot, bslot) = (lw.temp(Ty::Int), lw.temp(Ty::Int));
            out.push(Stmt::Let(aslot, Some(ao)));
            out.push(Stmt::Let(bslot, Some(bo)));
            let (av, bv) = (lw.local(aslot, span), lw.local(bslot, span));
            let lo = lw.mk(
                ExprKind::Builtin(Builtin::Min, vec![av.clone(), bv.clone()]),
                Ty::Int,
                span,
            );
            let a_end = if a_pos { lw.int_add(av, 1) } else { av };
            let b_end = if b_pos { lw.int_add(bv, 1) } else { bv };
            let hi = lw.mk(
                ExprKind::Builtin(Builtin::Max, vec![a_end, b_end]),
                Ty::Int,
                span,
            );
            let range = lw.mk(
                ExprKind::Range {
                    lo: Box::new(lo),
                    hi: Box::new(hi),
                },
                out_ty.clone(),
                span,
            );
            out.push(Stmt::Give(range));
        })
    }

    /// `counts(xs)`: how many times each value occurs; `c[k]` is `0` for
    /// a value that does not.
    fn lower_counts(&mut self, args: &[ast::Arg], span: Span) -> Expr {
        let Some(exprs) = self.positional_any("counts", args, 1, span) else {
            return self.err_expr(span);
        };
        let (iter, elem) = self.iter(exprs[0]);
        if elem == Ty::Error {
            return self.err_expr(span);
        }
        let hashable = matches!(
            elem,
            Ty::Int | Ty::Char | Ty::Bool | Ty::Str | Ty::Pos(_) | Ty::Bound(_)
        );
        if !hashable && !self.ordered(&elem) {
            let message = format!("a `{}` has no order to count by", self.show(&elem));
            self.error(codes::BAD_OPERANDS, span, message);
            return self.err_expr(span);
        }
        let sid = match self.counter_structs.get(&elem) {
            Some(&sid) => sid,
            None => {
                let map_ty = if hashable {
                    Ty::HashMap(Box::new(elem.clone()), Box::new(Ty::Int))
                } else {
                    Ty::Map(Box::new(elem.clone()), Box::new(Ty::Int))
                };
                let name = format!("Counter_{}", self.counter_structs.len());
                let sid = self.push_synth_struct(name, vec![("m".into(), map_ty)]);
                self.counter_structs.insert(elem.clone(), sid);
                self.counter_key.insert(sid, elem.clone());
                sid
            }
        };
        let map_ty = self.structs[sid].def.fields[0].1.clone();
        let struct_ty = Ty::Struct(sid);
        self.do_expr(Some(&struct_ty), span, |lw, out| {
            let m_slot = lw.temp(map_ty.clone());
            out.push(Stmt::Let(
                m_slot,
                Some(lw.mk(ExprKind::Tuple(Vec::new()), map_ty.clone(), span)),
            ));
            let item = lw.temp(elem);
            let lid = lw.fresh_loop();
            let place = Place {
                kind: PlaceKind::Index(
                    Box::new(Place {
                        kind: PlaceKind::Local(m_slot),
                        ty: map_ty.clone(),
                        span: lw.span(span),
                    }),
                    vec![lw.local(item, span)],
                ),
                ty: Ty::Int,
                span: lw.span(span),
            };
            let one = lw.int_lit(1, span);
            out.push(Stmt::For(
                lid,
                item,
                iter,
                Block {
                    stmts: vec![Stmt::Assign(place, AssignOp::Binary(BinOp::Add), one)],
                },
            ));
            let res = lw.mk(
                ExprKind::Tuple(vec![lw.local(m_slot, span)]),
                struct_ty.clone(),
                span,
            );
            out.push(Stmt::Give(res));
        })
    }

    /// `prefix(a)` dispatches on the shape of `a`: `List<T>` / `[0..n]T`,
    /// `[D]T`, or `[R, C]T`, constructing the matching `Prefix` shape from
    /// `std.prefix`.
    fn lower_prefix(&mut self, args: &[ast::Arg], span: Span) -> Expr {
        let Some(exprs) = self.positional_any("prefix", args, 1, span) else {
            return self.err_expr(span);
        };
        let src = self.expr(exprs[0], None);
        if src.ty == Ty::Error {
            return self.err_expr(span);
        }
        let elem_ty = match &src.ty {
            Ty::Array { elem, .. } | Ty::List(elem) | Ty::ListView(elem) | Ty::MutView(elem) => (**elem).clone(),
            _ => Ty::Error,
        };
        let adds = match &elem_ty {
            Ty::Int | Ty::Float | Ty::I128 | Ty::U64 => true,
            Ty::Struct(sid) => ["add", "sub"].iter().all(|op| self.struct_op_fn(*sid, op).is_some()),
            _ => false,
        };
        let doms: Option<Vec<DomainId>> = match &src.ty {
            Ty::Array { dims, .. } if adds && matches!(dims.len(), 1 | 2) => dims
                .iter()
                .map(|d| match d {
                    Dim::Pos(d) => Some(*d),
                    _ => None,
                })
                .collect(),
            _ => None,
        };
        // A list, or an array indexed by plain `Int`s from 0: `p.sum(l..r)`
        // takes `Int`s.
        let counted = match &src.ty {
            Ty::List(_) | Ty::ListView(_) | Ty::MutView(_) => true,
            Ty::Array { dims, .. } => matches!(dims[..], [Dim::Int]),
            _ => false,
        };
        if counted && adds {
            let Some(Sym::GenStruct(gid)) = self.get_sym("_PrefixList") else {
                self.unsupported(span, "`prefix` without `std.prefix.Prefix` in scope");
                return self.err_expr(span);
            };
            let list = Ty::List(Box::new(elem_ty.clone()));
            let src = self.coerce(src, &list, span);
            let sid = self.instantiate_struct(gid, vec![GenArg::Ty(elem_ty)]);
            let Some(&fn_id) = self.structs[sid].methods.get("new") else {
                self.unsupported(span, "`Prefix` has no `new` constructor");
                return self.err_expr(span);
            };
            let ret = self.ret_of(fn_id, span);
            return self.mk(ExprKind::Call(fn_id, vec![Arg::Value(src)]), ret, span);
        }
        if adds && matches!(&src.ty, Ty::Array { dims, .. } if matches!(dims[..], [Dim::IntFrom(_)])) {
            let Some(Sym::GenStruct(gid)) = self.get_sym("Prefix") else {
                self.unsupported(span, "`prefix` without `std.prefix.Prefix` in scope");
                return self.err_expr(span);
            };
            return self.call_gen_struct(gid, Vec::new(), args, None, span);
        }
        let Some(doms) = doms else {
            let message = format!(
                "`prefix` takes a `[D]T`, `[R, C]T`, or `List<T>` of numbers, found `{}`",
                self.show(&src.ty)
            );
            self.error(codes::TYPE_MISMATCH, span, message);
            return self.err_expr(span);
        };
        let struct_name = if doms.len() == 1 { "Prefix" } else { "_Prefix2" };
        let Some(Sym::GenStruct(gid)) = self.get_sym(struct_name) else {
            self.unsupported(span, "`prefix` without `std.prefix.Prefix` in scope");
            return self.err_expr(span);
        };
        let mut gargs: Vec<GenArg> = doms.into_iter().map(GenArg::Domain).collect();
        gargs.push(GenArg::Ty(elem_ty));
        let sid = self.instantiate_struct(gid, gargs);
        let Some(&fn_id) = self.structs[sid].methods.get("new") else {
            self.unsupported(span, "`Prefix` has no `new` constructor");
            return self.err_expr(span);
        };
        let ret = self.ret_of(fn_id, span);
        self.mk(ExprKind::Call(fn_id, vec![Arg::Value(src)]), ret, span)
    }
    /// `T.f(list)` where `list` is a plain list (or array over a plain
    /// range) of pairs or triples: the `f_of_pairs` / `f_of_triples` fn
    /// `T`'s module declares, if any.
    fn list_form_call(
        &mut self,
        gid: GenStructId,
        name: &ast::Name,
        type_args: &[ast::Type],
        args: &[ast::Arg],
        span: Span,
    ) -> Option<Expr> {
        let tpl_mod = self.gen_structs[gid].mod_id;
                // A static fn can take a plain list of pairs or triples when
                // its module declares `<name>_of_pairs` / `<name>_of_triples`:
                // `Graph.undirected(pairs)` needs no edge domain.
                let [only] = args else {
                    return None;
                };
                {
                    let dlen = self.diags.len();
                    let probed = self.expr(&only.value, None);
                    self.diags.truncate(dlen);
                    // So does an array over a plain range, `[0..m](u: V, v: V)`:
                    // it names no edge domain either. And one of plain
                    // tuples, `[E](V, V, W)`, whose cells have no `u` and `v`.
                    let arity = match &probed.ty {
                        Ty::List(elem) => elem.fields().map(|f| f.len()),
                        Ty::Array { dims, elem }
                            if matches!(dims.as_slice(), [Dim::Int])
                                || (dims.len() == 1 && matches!(**elem, Ty::Tuple(_))) =>
                        {
                            elem.fields().map(|f| f.len())
                        }
                        _ => None,
                    };
                    let suffix = match arity {
                        Some(2) => Some("pairs"),
                        Some(3) => Some("triples"),
                        _ => None,
                    };
                    let free = suffix
                        .and_then(|s| self.mod_syms[tpl_mod].get(&format!("{}_of_{s}", name.text)).copied());
                    if let Some(Sym::GenFn(gfid)) = free {
                        // `Graph<V>.undirected(pairs)` is the fn `Graph.undirected(pairs)`
                        // is: the receiver's type arguments fill its like-named generics.
                        let struct_generics = &self.gen_structs[gid].decl.generics;
                        let explicit: Vec<GenArg> = self.gen_fns[gfid]
                            .decl
                            .generics
                            .iter()
                            .map_while(|g| {
                                let at = struct_generics.iter().position(|s| s.text == g.text)?;
                                type_args.get(at)
                            })
                            .cloned()
                            .collect::<Vec<_>>()
                            .iter()
                            .map(|t| self.resolve_gen_arg(t))
                            .collect();
                        if !matches!(probed.ty, Ty::Array { .. }) {
                            return Some(self.call_gen_fn(gfid, explicit, None, args, span));
                        }
                        // A 1D array has the same backing storage as a list; `List(edges)`
                        // lowers to `Builtin::ArrayToList`, avoiding a copy loop.
                        let at = only.value.span;
                        let list_call = ast::ExprKind::Call {
                            callee: Box::new(ast::Expr {
                                kind: ast::ExprKind::Name("List".into()),
                                span: at,
                            }),
                            args: vec![ast::Arg {
                                name: None,
                                mutable: false,
                                value: only.value.clone(),
                            }],
                        };
                        let listed = ast::Arg {
                            name: None,
                            mutable: false,
                            value: ast::Expr { kind: list_call, span: at },
                        };
                        return Some(self.call_gen_fn(gfid, explicit, None, &[listed], span));
                    }
                }
        None
    }

}

/// Turns the `break` that ends an `all`/`any` early into one that leaves
/// the outermost loop of the comprehension.
fn retarget_break(stmts: &mut [Stmt], outer: LoopId, depth: usize) {
    for stmt in stmts {
        match stmt {
            Stmt::For(_, _, _, body) => retarget_break(&mut body.stmts, outer, depth + 1),
            Stmt::If(_, then, _) => retarget_break(&mut then.stmts, outer, depth),
            Stmt::Break(target @ None) if depth > 1 => *target = Some(outer),
            _ => {}
        }
    }
}

/// Whether `e` names somewhere a value lives: a variable, a field or an
/// element of one.
pub(super) fn is_place(e: &ast::Expr) -> bool {
    match &e.kind {
        ast::ExprKind::Name(_) | ast::ExprKind::SelfValue => true,
        ast::ExprKind::Paren(inner) => is_place(inner),
        ast::ExprKind::Field { base, .. } | ast::ExprKind::Index { base, .. } => is_place(base),
        _ => false,
    }
}

/// Makes every `?T` of `spec` read `absent` as `none`; false when it has no
/// `?T`.
pub(super) fn reads_none_as(spec: &mut Spec, absent: &Expr) -> bool {
    match spec {
        Spec::Plain(_) | Spec::Int { .. } | Spec::Call(_) => false,
        Spec::Fields(_, fields) => {
            let mut any = false;
            for field in fields {
                any |= reads_none_as(field, absent);
            }
            any
        }
        Spec::Array { elem, .. } => reads_none_as(elem, absent),
        Spec::Opt(_, slot) => {
            *slot = Some(Box::new(absent.clone()));
            true
        }
    }
}

/// What `for .. in g.items` goes through on a grid: `total` cells, and a
/// counter per axis that says which one the loop is at.
pub(super) struct GridItems {
    grid: Expr,
    dims: Vec<Dim>,
    elem: Ty,
    lens: Vec<Expr>,
    total: Expr,
    counters: Vec<Slot>,
}

impl GridItems {
    pub(super) fn range(&self) -> Iter {
        let lo = Expr {
            kind: ExprKind::Int(0),
            ty: Ty::Int,
            span: self.total.span,
        };
        Iter::Range {
            lo,
            hi: self.total.clone(),
            step: None,
            rev: false,
            closed: false,
        }
    }
}

pub(super) struct MapProj {
    map: Expr,
    pair_ty: Ty,
    elem_ty: Ty,
    idx: usize,
    rev: bool,
}

impl MapProj {
    pub(super) fn iter(&self) -> Iter {
        Iter::Seq(self.map.clone(), self.rev)
    }
}

/// What `xs.items` goes through: each of the `len` cells of `seq`, with
/// its index. An `Int` index starts at `first`.
pub(super) struct Items {
    seq: Expr,
    len: Expr,
    index: Ty,
    first: i64,
    elem: Ty,
}

impl Items {
    pub(super) fn range(&self) -> Iter {
        let lo = Expr {
            kind: ExprKind::Int(0),
            ty: Ty::Int,
            span: self.len.span,
        };
        Iter::Range {
            lo,
            hi: self.len.clone(),
            step: None,
            rev: false,
            closed: false,
        }
    }
}

impl<'a> Lowerer<'a> {
    pub(super) fn sorted_container_key(&self, sid: StructId) -> Option<Ty> {
        let info = &self.structs[sid];
        if !info.def.name.starts_with("SortedBag") && !info.def.name.starts_with("SortedSet") {
            return None;
        }
        let (_, gargs) = info.origin_gen.as_ref()?;
        match gargs.first()? {
            GenArg::Ty(k) => Some(k.clone()),
            GenArg::Domain(d) => Some(Ty::Pos(*d)),
            _ => None,
        }
    }

    pub(super) fn is_sorted_range_expr(&mut self, mut e: &ast::Expr) -> bool {
        loop {
            match &e.kind {
                ast::ExprKind::Paren(inner) => e = inner,
                ast::ExprKind::Name(n) => {
                    return self
                        .lookup(n)
                        .is_some_and(|slot| self.cx.sorted_views.contains_key(&slot));
                }
                ast::ExprKind::Field { base, name } if name.text == "rev" => e = base,
                ast::ExprKind::Call { callee, args } if args.len() == 1 => {
                    if let ast::ExprKind::Field { base, name } = &callee.kind {
                        if name.text == "range" {
                            return match self.probe_ty_silent(base) {
                                Some(Ty::Struct(sid)) => self.sorted_container_key(sid).is_some(),
                                Some(Ty::Set(_) | Ty::Map(_, _)) => true,
                                _ => false,
                            };
                        }
                    }
                    return false;
                }
                _ => return false,
            }
        }
    }

    fn sorted_domain_bounds(
        &mut self,
        d: DomainId,
        lo_ord: Expr,
        hi_ord: Expr,
        span: Span,
        out: &mut Vec<Stmt>,
    ) -> (Expr, Expr, Expr, Expr, Option<Expr>) {
        let lo_s = self.temp(Ty::Int);
        out.push(Stmt::Let(lo_s, Some(lo_ord)));
        let hi_s = self.temp(Ty::Int);
        out.push(Stmt::Let(hi_s, Some(hi_ord)));
        let dlen_s = self.temp(Ty::Int);
        let dlen = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
        out.push(Stmt::Let(dlen_s, Some(dlen)));

        let lo_l = self.local(lo_s, span);
        let hi_l = self.local(hi_s, span);
        let dlen_l = self.local(dlen_s, span);
        let zero = self.int_lit(0, span);

        let c1 = self.mk(
            ExprKind::Cmp(CmpOp::Ge, Box::new(lo_l.clone()), Box::new(hi_l.clone())),
            Ty::Bool,
            span,
        );
        let c2 = self.mk(
            ExprKind::Cmp(CmpOp::Ge, Box::new(lo_l.clone()), Box::new(dlen_l.clone())),
            Ty::Bool,
            span,
        );
        let c3 = self.mk(
            ExprKind::Cmp(CmpOp::Le, Box::new(hi_l.clone()), Box::new(zero.clone())),
            Ty::Bool,
            span,
        );
        let c12 = self.mk(ExprKind::Binary(BinOp::Or, Box::new(c1), Box::new(c2)), Ty::Bool, span);
        let empty = self.mk(ExprKind::Binary(BinOp::Or, Box::new(c12), Box::new(c3)), Ty::Bool, span);
        let empty_s = self.temp(Ty::Bool);
        out.push(Stmt::Let(empty_s, Some(empty)));
        let empty_l = self.local(empty_s, span);
        let not_empty = self.mk(ExprKind::Unary(UnOp::Not, Box::new(empty_l.clone())), Ty::Bool, span);

        let opt_k = Ty::opt(Ty::Pos(d));
        let lo_gt_zero = self.mk(
            ExprKind::Cmp(CmpOp::Gt, Box::new(lo_l.clone()), Box::new(zero)),
            Ty::Bool,
            span,
        );
        let has_lo = self.mk(
            ExprKind::Binary(BinOp::And, Box::new(not_empty.clone()), Box::new(lo_gt_zero)),
            Ty::Bool,
            span,
        );
        let lo_pos = self.cast(Cast::Trust, lo_l, Ty::Pos(d));
        let lo_some = self.mk(ExprKind::Some(Box::new(lo_pos)), opt_k.clone(), span);
        let lo_none = self.mk(ExprKind::None, opt_k.clone(), span);
        let lo_expr = self.mk(
            ExprKind::If(Box::new(has_lo), Box::new(lo_some), Box::new(lo_none)),
            opt_k.clone(),
            span,
        );

        let hi_lt_len = self.mk(
            ExprKind::Cmp(CmpOp::Lt, Box::new(hi_l.clone()), Box::new(dlen_l)),
            Ty::Bool,
            span,
        );
        let has_hi = self.mk(
            ExprKind::Binary(BinOp::And, Box::new(not_empty), Box::new(hi_lt_len)),
            Ty::Bool,
            span,
        );
        let hi_pos = self.cast(Cast::Trust, hi_l, Ty::Pos(d));
        let hi_some = self.mk(ExprKind::Some(Box::new(hi_pos)), opt_k.clone(), span);
        let hi_none = self.mk(ExprKind::None, opt_k.clone(), span);
        let hi_expr = self.mk(
            ExprKind::If(Box::new(has_hi), Box::new(hi_some), Box::new(hi_none)),
            opt_k,
            span,
        );

        let lo_inc = self.mk(ExprKind::Bool(true), Ty::Bool, span);
        let hi_inc = self.mk(ExprKind::Bool(false), Ty::Bool, span);
        (lo_expr, lo_inc, hi_expr, hi_inc, Some(empty_l))
    }

    pub(super) fn sorted_view_from_expr(
        &mut self,
        e: &ast::Expr,
        out: &mut Vec<Stmt>,
    ) -> Option<SortedView> {
        let mut cur = e;
        let mut flip = false;
        loop {
            match &cur.kind {
                ast::ExprKind::Paren(inner) => cur = inner,
                ast::ExprKind::Field { base, name } if name.text == "rev" => {
                    flip = !flip;
                    cur = base;
                }
                ast::ExprKind::Name(n) => {
                    let slot = self.lookup(n)?;
                    let mut v = self.cx.sorted_views.get(&slot)?.clone();
                    if flip {
                        v.rev = !v.rev;
                    }
                    return Some(v);
                }
                ast::ExprKind::Call { callee, args } if args.len() == 1 => {
                    let ast::ExprKind::Field { base, name } = &callee.kind else {
                        return None;
                    };
                    if name.text != "range" {
                        return None;
                    }
                    let (kind, key_ty, elem_ty) = match self.probe_ty_silent(base) {
                        Some(Ty::Struct(sid)) => {
                            let k = self.sorted_container_key(sid)?;
                            (SortedViewKind::Bag(sid), k.clone(), k)
                        }
                        Some(Ty::Set(k)) => (SortedViewKind::Set, (*k).clone(), *k),
                        Some(Ty::Map(k, v)) => {
                            let kt = (*k).clone();
                            let et = Ty::Tuple(vec![*k, *v]);
                            (SortedViewKind::Map, kt, et)
                        }
                        _ => return None,
                    };
                    let span = cur.span;
                    let raw_place = self.place(base);
                    let place = self.settle_indexes(raw_place, out);
                    let arg = &args[0].value;
                    let opt_k = Ty::opt(key_ty.clone());

                    let (lo_expr, lo_inc_expr, hi_expr, hi_inc_expr, empty_expr) = match &arg.kind {
                        ast::ExprKind::Range { lo, hi, inclusive, step: None } => {
                            let use_domain_ords = if let Ty::Pos(d) = key_ty {
                                let lo_is_bound = lo
                                    .as_deref()
                                    .and_then(|x| self.probe_ty_silent(x))
                                    .is_some_and(|t| t == Ty::Bound(d));
                                let hi_is_bound = hi
                                    .as_deref()
                                    .and_then(|x| self.probe_ty_silent(x))
                                    .is_some_and(|t| t == Ty::Bound(d));
                                lo_is_bound || hi_is_bound
                            } else {
                                false
                            };
                            if use_domain_ords {
                                let Ty::Pos(d) = key_ty else { unreachable!() };
                                let lo_ord = match lo.as_deref() {
                                    Some(l) => {
                                        let le = self.expr(l, None);
                                        self.cast(Cast::Ord, le, Ty::Int)
                                    }
                                    None => self.int_lit(0, span),
                                };
                                let hi_ord = match hi.as_deref() {
                                    Some(h) => {
                                        let he = self.expr(h, None);
                                        let is_pos = he.ty == Ty::Pos(d);
                                        let ho = self.cast(Cast::Ord, he, Ty::Int);
                                        if is_pos && *inclusive {
                                            self.int_add(ho, 1)
                                        } else {
                                            ho
                                        }
                                    }
                                    None => self.mk(ExprKind::DomainLen(d), Ty::Int, span),
                                };
                                self.sorted_domain_bounds(d, lo_ord, hi_ord, span, out)
                            } else {
                                let lo_e = match lo.as_deref() {
                                    Some(l) => {
                                        let lv = self.expect(l, &key_ty);
                                        self.mk(ExprKind::Some(Box::new(lv)), opt_k.clone(), span)
                                    }
                                    None => self.mk(ExprKind::None, opt_k.clone(), span),
                                };
                                let hi_e = match hi.as_deref() {
                                    Some(h) => {
                                        let hv = self.expect(h, &key_ty);
                                        self.mk(ExprKind::Some(Box::new(hv)), opt_k.clone(), span)
                                    }
                                    None => self.mk(ExprKind::None, opt_k.clone(), span),
                                };
                                (
                                    lo_e,
                                    self.mk(ExprKind::Bool(true), Ty::Bool, span),
                                    hi_e,
                                    self.mk(ExprKind::Bool(*inclusive), Ty::Bool, span),
                                    None,
                                )
                            }
                        }
                        _ => {
                            let want_r = Ty::Range(Box::new(key_ty.clone()));
                            let rv = self.expr(arg, Some(&want_r));
                            let r_slot = self.temp(rv.ty.clone());
                            out.push(Stmt::Let(r_slot, Some(rv)));
                            let r_l = self.local(r_slot, span);
                            let r_lo = self.mk(
                                ExprKind::Builtin(Builtin::RangeLo, vec![r_l.clone()]),
                                Ty::Int,
                                span,
                            );
                            let r_hi = self.mk(
                                ExprKind::Builtin(Builtin::RangeHi, vec![r_l]),
                                Ty::Int,
                                span,
                            );
                            if let Ty::Pos(d) = key_ty {
                                self.sorted_domain_bounds(d, r_lo, r_hi, span, out)
                            } else {
                                let r_lo = self.coerce(r_lo, &key_ty, span);
                                let r_hi = self.coerce(r_hi, &key_ty, span);
                                (
                                    self.mk(ExprKind::Some(Box::new(r_lo)), opt_k.clone(), span),
                                    self.mk(ExprKind::Bool(true), Ty::Bool, span),
                                    self.mk(ExprKind::Some(Box::new(r_hi)), opt_k.clone(), span),
                                    self.mk(ExprKind::Bool(false), Ty::Bool, span),
                                    None,
                                )
                            }
                        }
                    };

                    let lo_s = self.temp(opt_k.clone());
                    out.push(Stmt::Let(lo_s, Some(lo_expr)));
                    let lo_inc_s = self.temp(Ty::Bool);
                    out.push(Stmt::Let(lo_inc_s, Some(lo_inc_expr)));
                    let hi_s = self.temp(opt_k);
                    out.push(Stmt::Let(hi_s, Some(hi_expr)));
                    let hi_inc_s = self.temp(Ty::Bool);
                    out.push(Stmt::Let(hi_inc_s, Some(hi_inc_expr)));
                    let empty_s = empty_expr.map(|ee| {
                        let s = self.temp(Ty::Bool);
                        out.push(Stmt::Let(s, Some(ee)));
                        s
                    });
                    return Some(SortedView {
                        base: place,
                        kind,
                        key_ty,
                        elem_ty,
                        lo: lo_s,
                        lo_inc: lo_inc_s,
                        hi: hi_s,
                        hi_inc: hi_inc_s,
                        empty: empty_s,
                        rev: flip,
                    });
                }
                _ => return None,
            }
        }
    }

    pub(super) fn call_sorted_view(
        &mut self,
        view: &SortedView,
        sid: StructId,
        method: &str,
        ret_ty: Ty,
        empty_default: Expr,
        span: Span,
    ) -> Expr {
        let Some(fn_id) = self.struct_op_fn(sid, method) else {
            return empty_default;
        };
        let args = vec![
            Arg::Value(super::stmt::place_to_expr(&view.base)),
            Arg::Value(self.local(view.lo, span)),
            Arg::Value(self.local(view.lo_inc, span)),
            Arg::Value(self.local(view.hi, span)),
            Arg::Value(self.local(view.hi_inc, span)),
        ];
        let call = self.mk(ExprKind::Call(fn_id, args), ret_ty.clone(), span);
        if let Some(emp) = view.empty {
            let cond = self.local(emp, span);
            self.mk(
                ExprKind::If(Box::new(cond), Box::new(empty_default), Box::new(call)),
                ret_ty,
                span,
            )
        } else {
            call
        }
    }

    fn tree_view_key(&mut self, view: &SortedView, val: Expr, span: Span) -> Expr {
        match view.kind {
            SortedViewKind::Map => {
                self.mk(ExprKind::Field(Box::new(val), 0), view.key_ty.clone(), span)
            }
            _ => val,
        }
    }

    /// Filter an optional candidate `cand` against the upper bound (`hi`, `hi_inc`)
    /// when `check_hi`, or the lower bound (`lo`, `lo_inc`) when `!check_hi`.
    fn tree_view_clip(&mut self, view: &SortedView, cand: Expr, check_hi: bool, span: Span) -> Expr {
        let opt_e = Ty::opt(view.elem_ty.clone());
        let bound_slot = if check_hi { view.hi } else { view.lo };
        let inc_slot = if check_hi { view.hi_inc } else { view.lo_inc };
        self.do_expr(Some(&opt_e), span, |lw, out| {
            let cs = lw.temp(opt_e.clone());
            out.push(Stmt::Let(cs, Some(cand)));
            let c_ref = lw.local(cs, span);
            let c_some = lw.mk(ExprKind::IsSome(Box::new(c_ref.clone())), Ty::Bool, span);
            let b_ref = lw.local(bound_slot, span);
            let b_some = lw.mk(ExprKind::IsSome(Box::new(b_ref.clone())), Ty::Bool, span);
            let both = lw.mk(
                ExprKind::Binary(BinOp::And, Box::new(c_some), Box::new(b_some)),
                Ty::Bool,
                span,
            );
            let elem_val = lw.mk(ExprKind::Unwrap(Box::new(c_ref.clone()), false), view.elem_ty.clone(), span);
            let k_val = lw.tree_view_key(view, elem_val, span);
            let b_val = lw.mk(ExprKind::Unwrap(Box::new(b_ref), false), view.key_ty.clone(), span);
            let (strict_op, inc_op) = if check_hi {
                (CmpOp::Lt, CmpOp::Le)
            } else {
                (CmpOp::Gt, CmpOp::Ge)
            };
            let inc_cmp = lw.mk(ExprKind::Cmp(inc_op, Box::new(k_val.clone()), Box::new(b_val.clone())), Ty::Bool, span);
            let strict_cmp = lw.mk(ExprKind::Cmp(strict_op, Box::new(k_val), Box::new(b_val)), Ty::Bool, span);
            let in_bounds = lw.mk(
                ExprKind::If(Box::new(lw.local(inc_slot, span)), Box::new(inc_cmp), Box::new(strict_cmp)),
                Ty::Bool,
                span,
            );
            let keep = lw.mk(
                ExprKind::If(
                    Box::new(in_bounds),
                    Box::new(c_ref.clone()),
                    Box::new(lw.mk(ExprKind::None, opt_e.clone(), span)),
                ),
                opt_e.clone(),
                span,
            );
            let res = lw.mk(
                ExprKind::If(Box::new(both), Box::new(keep), Box::new(c_ref)),
                opt_e.clone(),
                span,
            );
            out.push(Stmt::Give(res));
        })
    }

    fn tree_view_end(&mut self, view: &SortedView, want_last: bool, span: Span) -> Expr {
        let opt_e = Ty::opt(view.elem_ty.clone());
        let none_e = self.mk(ExprKind::None, opt_e.clone(), span);
        let body = self.do_expr(Some(&opt_e), span, |lw, out| {
            let base_e = super::stmt::place_to_expr(&view.base);
            let (start_slot, start_inc, end_op, ge_op, gt_op, check_hi) = if want_last {
                (view.hi, view.hi_inc, Builtin::SetMax, Builtin::LastLe, Builtin::LastLt, false)
            } else {
                (view.lo, view.lo_inc, Builtin::SetMin, Builtin::FirstGe, Builtin::FirstGt, true)
            };
            let s_ref = lw.local(start_slot, span);
            let s_some = lw.mk(ExprKind::IsSome(Box::new(s_ref.clone())), Ty::Bool, span);
            let s_val = lw.mk(ExprKind::Unwrap(Box::new(s_ref), false), view.key_ty.clone(), span);
            let call_ge = lw.mk(ExprKind::Builtin(ge_op, vec![base_e.clone(), s_val.clone()]), opt_e.clone(), span);
            let call_gt = lw.mk(ExprKind::Builtin(gt_op, vec![base_e.clone(), s_val]), opt_e.clone(), span);
            let bounded = lw.mk(
                ExprKind::If(Box::new(lw.local(start_inc, span)), Box::new(call_ge), Box::new(call_gt)),
                opt_e.clone(),
                span,
            );
            let open = lw.mk(ExprKind::Builtin(end_op, vec![base_e]), opt_e.clone(), span);
            let cand = lw.mk(ExprKind::If(Box::new(s_some), Box::new(bounded), Box::new(open)), opt_e.clone(), span);
            let clipped = lw.tree_view_clip(view, cand, check_hi, span);
            out.push(Stmt::Give(clipped));
        });
        if let Some(emp) = view.empty {
            self.mk(
                ExprKind::If(Box::new(self.local(emp, span)), Box::new(none_e), Box::new(body)),
                opt_e,
                span,
            )
        } else {
            body
        }
    }

    fn tree_view_collect(&mut self, view: &SortedView, span: Span) -> Expr {
        let list_ty = Ty::List(Box::new(view.elem_ty.clone()));
        let opt_e = Ty::opt(view.elem_ty.clone());
        self.do_expr(Some(&list_ty), span, |lw, out| {
            let out_s = lw.temp(list_ty.clone());
            out.push(Stmt::Let(out_s, Some(lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span))));
            let cur_s = lw.temp(opt_e.clone());
            let first = lw.tree_view_end(view, view.rev, span);
            out.push(Stmt::Let(cur_s, Some(first)));
            let cur_ref = lw.local(cur_s, span);
            let cond = lw.mk(ExprKind::IsSome(Box::new(cur_ref.clone())), Ty::Bool, span);
            let v_s = lw.temp(view.elem_ty.clone());
            let v_init = lw.mk(ExprKind::Unwrap(Box::new(cur_ref), false), view.elem_ty.clone(), span);
            let v_ref = lw.local(v_s, span);
            let push_stmt = Stmt::Expr(lw.mk(
                ExprKind::Mutate(
                    Mutation::Push,
                    Box::new(Place {
                        kind: PlaceKind::Local(out_s),
                        ty: list_ty.clone(),
                        span: lw.span(span),
                    }),
                    vec![v_ref.clone()],
                ),
                Ty::Unit,
                span,
            ));
            let k_val = lw.tree_view_key(view, v_ref, span);
            let step_op = if view.rev { Builtin::LastLt } else { Builtin::FirstGt };
            let next_raw = lw.mk(
                ExprKind::Builtin(step_op, vec![super::stmt::place_to_expr(&view.base), k_val]),
                opt_e.clone(),
                span,
            );
            let next_clipped = lw.tree_view_clip(view, next_raw, !view.rev, span);
            let step_stmt = Stmt::Assign(
                Place {
                    kind: PlaceKind::Local(cur_s),
                    ty: opt_e.clone(),
                    span: lw.span(span),
                },
                AssignOp::Set,
                next_clipped,
            );
            let lid = lw.next_loop;
            lw.next_loop += 1;
            out.push(Stmt::While(
                lid,
                cond,
                Block {
                    stmts: vec![Stmt::Let(v_s, Some(v_init)), push_stmt, step_stmt],
                },
            ));
            out.push(Stmt::Give(lw.local(out_s, span)));
        })
    }

    fn sorted_view_field(&mut self, base: &ast::Expr, field: &str, span: Span) -> Expr {
        let mut res_ty = Ty::Error;
        let mut e = self.do_expr(None, span, |lw, out| {
            let Some(view) = lw.sorted_view_from_expr(base, out) else {
                out.push(Stmt::Give(lw.err_expr(span)));
                return;
            };
            let opt_e = Ty::opt(view.elem_ty.clone());
            let res = match view.kind {
                SortedViewKind::Bag(sid) => match field {
                    "first" => {
                        let m = if view.rev { "range_last" } else { "range_first" };
                        let d = lw.mk(ExprKind::None, opt_e.clone(), span);
                        lw.call_sorted_view(&view, sid, m, opt_e, d, span)
                    }
                    "last" => {
                        let m = if view.rev { "range_first" } else { "range_last" };
                        let d = lw.mk(ExprKind::None, opt_e.clone(), span);
                        lw.call_sorted_view(&view, sid, m, opt_e, d, span)
                    }
                    "len" => {
                        let d = lw.int_lit(0, span);
                        lw.call_sorted_view(&view, sid, "range_len", Ty::Int, d, span)
                    }
                    "is_empty" => {
                        let d = lw.int_lit(0, span);
                        let len = lw.call_sorted_view(&view, sid, "range_len", Ty::Int, d, span);
                        let zero = lw.int_lit(0, span);
                        lw.mk(
                            ExprKind::Cmp(CmpOp::Eq, Box::new(len), Box::new(zero)),
                            Ty::Bool,
                            span,
                        )
                    }
                    _ => lw.err_expr(span),
                },
                SortedViewKind::Set | SortedViewKind::Map => match field {
                    "first" => lw.tree_view_end(&view, view.rev, span),
                    "last" => lw.tree_view_end(&view, !view.rev, span),
                    "is_empty" => {
                        let first = lw.tree_view_end(&view, false, span);
                        let some = lw.mk(ExprKind::IsSome(Box::new(first)), Ty::Bool, span);
                        lw.mk(ExprKind::Unary(UnOp::Not, Box::new(some)), Ty::Bool, span)
                    }
                    _ => lw.err_expr(span),
                },
            };
            res_ty = res.ty.clone();
            out.push(Stmt::Give(res));
        });
        e.ty = res_ty;
        e
    }

    pub(super) fn sorted_view_iter(&mut self, e: &ast::Expr, span: Span) -> Option<(Iter, Ty)> {
        if !self.is_sorted_range_expr(e) {
            return None;
        }
        // Zero-allocation direct iterator walk for `Set.range(r)` / `Map.range(r)`
        // when `r` is a syntactic range literal (`a..b`, `a..=b`, `a..`, `..b`, `..=b`, `..`).
        let mut cur = e;
        let mut rev = false;
        while let ast::ExprKind::Field { base, name } = &cur.kind {
            if name.text == "rev" {
                rev = !rev;
                cur = base;
            } else {
                break;
            }
        }
        if let ast::ExprKind::Call { callee, args } = &cur.kind {
            if let ast::ExprKind::Field { base, name } = &callee.kind {
                if name.text == "range" && args.len() == 1 {
                    let base_ty = self.probe_ty_silent(base);
                    if let Some((key_ty, elem_ty)) = match base_ty {
                        Some(Ty::Set(k)) => Some(((*k).clone(), *k)),
                        Some(Ty::Map(k, v)) => {
                            let kt = *k;
                            let et = Ty::Tuple(vec![kt.clone(), *v]);
                            Some((kt, et))
                        }
                        _ => None,
                    } {
                        if let ast::ExprKind::Range { lo, hi, inclusive, .. } = &args[0].value.kind {
                            let container = self.expr(base, None);
                            let lo = lo.as_ref().map(|x| (self.expect(x, &key_ty), true));
                            let hi = hi.as_ref().map(|x| (self.expect(x, &key_ty), *inclusive));
                            return Some((
                                Iter::TreeRange {
                                    container,
                                    lo,
                                    hi,
                                    rev,
                                },
                                elem_ty,
                            ));
                        }
                    }
                }
            }
        }
        let mut elem_ty = Ty::Error;
        let mut seq_ty = Ty::Error;
        let mut seq = self.do_expr(None, span, |lw, out| {
            let Some(view) = lw.sorted_view_from_expr(e, out) else {
                out.push(Stmt::Give(lw.err_expr(span)));
                return;
            };
            elem_ty = view.elem_ty.clone();
            let list_ty = Ty::List(Box::new(view.elem_ty.clone()));
            let res = match view.kind {
                SortedViewKind::Bag(sid) => {
                    let m = if view.rev { "range_each_rev" } else { "range_each" };
                    let empty_list = lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span);
                    lw.call_sorted_view(&view, sid, m, list_ty.clone(), empty_list, span)
                }
                SortedViewKind::Set | SortedViewKind::Map => lw.tree_view_collect(&view, span),
            };
            seq_ty = list_ty;
            out.push(Stmt::Give(res));
        });
        seq.ty = seq_ty;
        Some((Iter::Seq(seq, false), elem_ty))
    }
}

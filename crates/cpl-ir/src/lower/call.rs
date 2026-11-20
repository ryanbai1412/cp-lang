//! Calls, methods and members: the user's fns and the ones the language
//! provides.

use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Reduce {
    Sum,
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
            let v = match param_tys.get(i) {
                Some(want) => self.expect(&arg.value, want),
                None => self.expr(&arg.value, None),
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
                    Some(Sym::GenFn(gfid)) => self.call_gen_fn(gfid, Vec::new(), None, args, span),
                    Some(Sym::Struct(id)) => self.construct_or_new(id, args, span),
                    Some(Sym::GenStruct(gid)) => self.call_gen_struct(gid, Vec::new(), args, hint, span),
                    Some(Sym::Variant(id, tag)) => self.variant(id, tag, args, span),
                    Some(Sym::Domain(id)) => {
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
        // `read::<Int>()`: a read that says its own type.
        if name == "read" && self.get_sym(name).is_none() {
            if let [ty] = types {
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
        match self.get_sym(name) {
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
            _ => {
                self.unsupported(span, "generic fns and types");
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
        if explicit.len() == gen_names.len() {
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
        let sid = self.instantiate_struct(gid, final_args);
        self.construct_or_new(sid, args, span)
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
        id: FnId,
        receiver: Option<Arg>,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        self.queue_fn(id);
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
        for (param, arg) in params.iter().zip(given) {
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
                    if place.ty != param.ty && place.ty != Ty::Error && param.ty != Ty::Error {
                        self.mismatch(arg.value.span, &param.ty, &place.ty);
                        ok = false;
                    }
                    lowered.push(Arg::Mut(place));
                }
                Some(arg) => {
                    if param.ty == Ty::Unit && self.detect_bare_op(&arg.value).is_some() {
                        lowered.push(Arg::Value(self.mk(ExprKind::Unit, Ty::Unit, arg.value.span)));
                    } else {
                        lowered.push(Arg::Value(self.expect(&arg.value, &param.ty)));
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
        let ret = self.ret_of(id, span);
        if !ok {
            return self.err_expr(span);
        }
        let call_expr = self.mk(ExprKind::Call(id, lowered), ret.clone(), span);
        if pre_stmts.is_empty() && post_stmts.is_empty() {
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
        }
    }

    /// `Edge(u: a, v: b, w: 5)` or `Edge(a, b, 5)`.
    fn construct(&mut self, id: StructId, args: &[ast::Arg], span: Span) -> Expr {
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
                (Some(arg), _) => values.push(self.expect(&arg.value, ty)),
                (None, Some(default)) => {
                    let saved = std::mem::replace(&mut self.offset, self.structs[id].offset);
                    values.push(self.expect(default, ty));
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

    fn variant(&mut self, id: EnumId, tag: usize, args: &[ast::Arg], span: Span) -> Expr {
        let (name, fields) = self.enums[id].variants[tag].clone();
        let Some(exprs) = self.positional_any(&name, args, fields.len(), span) else {
            return self.err_expr(span);
        };
        let values = exprs
            .into_iter()
            .zip(&fields)
            .map(|(e, ty)| self.expect(e, ty))
            .collect();
        self.mk(ExprKind::Variant(tag, values), Ty::Enum(id), span)
    }

    /// Fns every file can call without declaring them.
    fn prelude(&mut self, name: &str, args: &[ast::Arg], hint: Option<&Ty>, span: Span) -> Expr {
        match name {
            "print" => {
                let mut digits = 6;
                let mut values = Vec::new();
                let mut sep = None;
                let mut none_text = None;
                for arg in args {
                    match &arg.name {
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
                self.mk(ExprKind::Print(values), Ty::Unit, span)
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
                    .filter_map(|(value, at)| self.fmt_arg(value, 6, at))
                    .collect();
                let query = self.mk(ExprKind::Print(values), Ty::Unit, span);
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
                if !args
                    .iter()
                    .all(|a| a.name.as_ref().is_some_and(|n| n.text == "gen"))
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
                    Some(spec) => self.read(spec, span),
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
                let Some(exprs) = self.positional_any(name, args, 1, span) else {
                    return self.err_expr(span);
                };
                self.lower_first(exprs[0], span)
            }
            "sum" | "count" | "all" | "any" => {
                let kind = match name {
                    "sum" => Reduce::Sum,
                    "count" => Reduce::Count,
                    "all" => Reduce::All,
                    _ => Reduce::Any,
                };
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
            "flush" => match self.positional_any(name, args, 0, span) {
                Some(_) => self.mk(ExprKind::Builtin(Builtin::Flush, Vec::new()), Ty::Unit, span),
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
                match self.one_arg(name, args, &Ty::Int, span) {
                    Some(value) => self.mk(ExprKind::Builtin(op, vec![value]), Ty::Int, span),
                    None => self.err_expr(span),
                }
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
                    _ => Ty::Error,
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
            "sorted" => {
                if args.is_empty() {
                    self.error(codes::WRONG_ARG_COUNT, span, "`sorted` takes a sequence");
                    return self.err_expr(span);
                }
                let value = self.expr(&args[0].value, None);
                if args.len() == 1 {
                    self.sorted(value, span)
                } else {
                    self.sorted_by(value, &args[1..], span)
                }
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
                    Ty::Str => self.mk(
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
            ("Int", Ty::Bool) => {
                let one = self.int_lit(1, span);
                let zero = self.int_lit(0, span);
                return self.mk(
                    ExprKind::If(Box::new(value), Box::new(one), Box::new(zero)),
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
            | Ty::Str
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
            Ty::List(elem) => {
                if !self.ordered(elem) {
                    let message = format!("a `{}` has no order to sort by", self.show(elem));
                    self.error(codes::BAD_OPERANDS, span, message);
                    return self.err_expr(span);
                }
                let ty = value.ty.clone();
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
            };
            out.push(Stmt::For(lid, at, iter, Block { stmts: vec![Stmt::Expr(push)] }));
            out.push(Stmt::Give(lw.local(all, span)));
        })
    }

    /// `sorted(xs, by: ...)`
    fn sorted_by(&mut self, value: Expr, args: &[ast::Arg], span: Span) -> Expr {
        // `sorted(V, by: ...)`: the positions of a domain, in another order.
        let value = match value.kind {
            ExprKind::DomainVal(d) => self.positions(d, span),
            _ => value,
        };
        let list_ty = match value.ty.clone() {
            Ty::List(e) => Ty::List(e),
            Ty::Array { elem, .. } => Ty::List(elem),
            Ty::Set(e) | Ty::HashSet(e) => Ty::List(e),
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
                Ty::Array { .. } => lw.mk(
                    ExprKind::Builtin(Builtin::ArrayToList, vec![value]),
                    list_ty.clone(),
                    span,
                ),
                _ => lw.mk(
                    ExprKind::Builtin(Builtin::Sorted, vec![value]),
                    list_ty.clone(),
                    span,
                ),
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
        let elem = match &seq.ty {
            Ty::List(e) | Ty::Array { elem: e, .. } => (**e).clone(),
            Ty::Str => Ty::Char,
            Ty::Error => return self.err_expr(span),
            other => {
                self.error(
                    codes::TYPE_MISMATCH,
                    exprs[0].span,
                    format!("`runs` takes a list, array, or string, not `{}`", self.show(other)),
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
            out.push(Stmt::For(
                lid,
                item_slot,
                Iter::Seq(seq, false),
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
                        Dim::Int => None,
                    })
                    .collect();
                Spec::Array {
                    dims: dims?,
                    elem: Box::new(self.spec_of(elem)?),
                }
            }
            other => Spec::Plain(other.clone()),
        })
    }

    /// `read()` of a value shaped like `spec`.
    pub(super) fn read(&mut self, spec: Spec, span: Span) -> Expr {
        fn unreadable(spec: &Spec) -> Option<Ty> {
            match spec {
                Spec::Plain(ty) => match ty {
                    Ty::Int | Ty::I128 | Ty::U64 | Ty::Float | Ty::Char | Ty::Str | Ty::Pos(_) | Ty::Error => None,
                    other => Some(other.clone()),
                },
                Spec::Int { .. } => None,
                Spec::Fields(_, fields) => fields.iter().find_map(unreadable),
                Spec::Array { elem, .. } => unreadable(elem),
                Spec::Opt(_) => Some(spec.ty()),
            }
        }
        if let Some(ty) = unreadable(&spec) {
            let message = format!("`read()` has no input format for a `{}`", self.show(&ty));
            self.error(codes::UNSUPPORTED, span, message);
            return self.err_expr(span);
        }
        if let Spec::Array { dims, elem } = &spec {
            if let [DimSpec::Pos(d)] = dims[..] {
                if self.own_domains.contains(&d) {
                    return self.read_own(d, elem, spec.ty(), span);
                }
            }
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
            };
            let copy = Stmt::Assign(cell, AssignOp::Set, byte);
            out.push(Stmt::For(lid, at, iter, Block { stmts: vec![copy] }));
            out.push(Stmt::Give(lw.local(cells, span)));
        })
    }

    fn min_max(&mut self, is_max: bool, args: &[ast::Arg], span: Span) -> Expr {
        let name = if is_max { "max" } else { "min" };
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
                let elem = match &seq.ty {
                    Ty::Array { elem, .. } | Ty::List(elem) | Ty::Set(elem) | Ty::Range(elem) => (**elem).clone(),
                    Ty::Error => return self.err_expr(span),
                    other => {
                        let message = format!(
                            "this needs a sequence of values, not a `{}`",
                            self.show(other)
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

    /// Lowers `e`, returning `?T` when `e` is a 1-arg `min(...)` or `max(...)` reduction
    /// (for `min(...) ?? default` and `min(...)!`).
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
        }
        self.expr(e, None)
    }

    /// `sum(a[v] for v in V)`, `max(xs)`, `any(...)`: a loop that folds
    /// the values into one.
    fn reduce(&mut self, kind: Reduce, arg: &ast::Expr, span: Span) -> Expr {
        self.reduce_opt(kind, arg, false, span)
    }

    fn reduce_opt(&mut self, kind: Reduce, arg: &ast::Expr, optional: bool, span: Span) -> Expr {
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
                let elem = match &seq.ty {
                    Ty::Array { elem, .. } | Ty::List(elem) | Ty::Set(elem) => (**elem).clone(),
                    Ty::Error => return self.err_expr(span),
                    other => {
                        let message = format!(
                            "this needs a sequence of values, not a `{}`",
                            self.show(other)
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
                // A `Mod<P>` adds with its own `+`, from its own zero.
                if let Ty::Struct(sid) = ty {
                    if let (true, Some(add)) = (self.is_std_mod(sid), self.struct_op_fn(sid, "add")) {
                        let zero = self.int_lit(0, span);
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
                        self.close_loops(frames, Vec::new());
                        return self.err_expr(span);
                    }
                };
                let acc = self.temp(ty.clone());
                stmts.push(Stmt::Let(acc, Some(lit(zero, ty.clone()))));
                let add = Stmt::Assign(place(acc, &ty), AssignOp::Binary(BinOp::Add), value);
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
    pub(super) fn field(&mut self, whole: &ast::Expr, base: &ast::Expr, name: &ast::Name) -> Expr {
        let span = whole.span;
        // Handle `GenericBase<...>.member` (e.g., `Mod<7>.zero`, `Sum<Int>.id`, `Less<Int>.rev`).
        if let ast::ExprKind::Generic { base: gbase, args: gtypes } = &base.kind {
            if let ast::ExprKind::Name(gname) = &gbase.kind {
                if let Some(Sym::GenStruct(gid)) = self.get_sym(gname) {
                    let gargs = self.resolve_gen_args(gtypes);
                    let sid = self.instantiate_struct(gid, gargs);
                    if let Some(sc) = self.structs[sid].consts.get(&name.text).cloned() {
                        return self.eval_struct_const(sid, sc, span);
                    }
                    if let Some(&fn_id) = self.structs[sid].methods.get(&name.text) {
                        return self.method_call(fn_id, None, &[], span);
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
                    return self.domain_field(sub, name, span);
                }
            }
        }
        // Local type alias pointing to a domain (e.g. `type Group = Subset<Rabbit>` inside a block).
        if let ast::ExprKind::Name(n) = &base.kind {
            if self.lookup(n).is_none() {
                if let Some(&Ty::Pos(id)) = self.cx.local_type_aliases.get(n) {
                    return self.domain_field(id, name, span);
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
                    }
                    Ty::Pos(id) => return self.domain_field(id, name, span),
                    _ => {}
                }
            }
            Some(Sym::Domain(id)) => return self.domain_field(id, name, span),
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
            (Ty::Range(inner), "lo") => {
                let inner = (**inner).clone();
                let lo = builtin(self, Builtin::RangeLo, vec![value], Ty::Int);
                match inner {
                    Ty::Pos(d) => self.cast(Cast::Trust, lo, Ty::Bound(d)),
                    _ => lo,
                }
            }
            (Ty::Range(inner), "hi") => {
                let inner = (**inner).clone();
                let hi = builtin(self, Builtin::RangeHi, vec![value], Ty::Int);
                match inner {
                    Ty::Pos(d) => self.cast(Cast::Trust, hi, Ty::Bound(d)),
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
            (Ty::DomainVal, "len" | "size" | "K") => {
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
                | Ty::Str
                | Ty::Map(..)
                | Ty::HashMap(..)
                | Ty::Set(_)
                | Ty::HashSet(_)
                | Ty::Range(_),
                "len",
            ) => self.len_of(value, span),
            (
                Ty::Array { .. }
                | Ty::List(_)
                | Ty::Str
                | Ty::Map(..)
                | Ty::HashMap(..)
                | Ty::Set(_)
                | Ty::HashSet(_)
                | Ty::Range(_),
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
            (Ty::List(elem), "first") => {
                let opt = Ty::opt((**elem).clone());
                builtin(self, Builtin::First, vec![value], opt)
            }
            (Ty::List(elem), "last") => {
                let opt = Ty::opt((**elem).clone());
                builtin(self, Builtin::Last, vec![value], opt)
            }
            (Ty::List(_) | Ty::Str, "rev") => {
                builtin(self, Builtin::Reversed, vec![value], ty.clone())
            }
            (Ty::List(_) | Ty::Str, "adjacent") => self.lower_adjacent(value, span),
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
            (Ty::Pos(d), "K" | "len") => self.mk(ExprKind::DomainLen(*d), Ty::Int, span),
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
            (Ty::Set(elem), "first" | "last") => {
                let op = if name.text == "first" {
                    Builtin::SetMin
                } else {
                    Builtin::SetMax
                };
                let ty = Ty::Opt(elem.clone());
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
                self.err_expr(span)
            }
        }
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
        let sym = self.get_sym(name)?;
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

    /// Lowers `xs.first_ge(val)`, `xs.last_le(val)`, or `xs.find(val)` on a `List<T>` or 1D `[D]T`.
    fn seq_search_method(
        &mut self,
        method: &str,
        container: Expr,
        elem: Ty,
        dom: Option<DomainId>,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let Some(target) = self.one_arg(method, args, &elem, span) else {
            return self.err_expr(span);
        };
        let idx_ty = match dom {
            Some(d) => Ty::Pos(d),
            None => Ty::Int,
        };
        let out_ty = Ty::opt(idx_ty.clone());
        let cont_ty = container.ty.clone();
        let out_ty_hint = out_ty.clone();
        self.do_expr(Some(&out_ty_hint), span, |lw, out| {
            let c_slot = lw.temp(cont_ty);
            let t_slot = lw.temp(elem.clone());
            out.push(Stmt::Let(c_slot, Some(container)));
            out.push(Stmt::Let(t_slot, Some(target)));
            let c_ref = lw.local(c_slot, span);
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
                let go_left = if is_ge {
                    lw.mk(ExprKind::Cmp(CmpOp::Ge, Box::new(cur), Box::new(t_ref)), Ty::Bool, span)
                } else {
                    lw.mk(ExprKind::Cmp(CmpOp::Gt, Box::new(cur), Box::new(t_ref)), Ty::Bool, span)
                };
                let lid = lw.next_loop;
                lw.next_loop += 1;
                out.push(Stmt::While(
                    lid,
                    cond,
                    Block {
                        stmts: vec![
                            Stmt::Let(mid_slot, Some(mid)),
                            Stmt::If(
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
                            ),
                        ],
                    },
                ));
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
                .filter_map(|(value, at)| self.fmt_arg(value, 6, at))
                .collect();
            return self.mk(ExprKind::Print(values), Ty::Unit, span);
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
                let val = self.expr(exprs[1], val_hint);
                let list_ty = Ty::list(val.ty.clone());
                return self.mk(
                    ExprKind::Builtin(Builtin::ListFilled, vec![count, val]),
                    list_ty,
                    span,
                );
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
                if let Some(Sym::GenStruct(gid)) = self.get_sym(gname) {
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
                // A static fn can take a plain list of pairs or triples when
                // its module declares `<name>_of_pairs` / `<name>_of_triples`:
                // `Graph.undirected(pairs)` needs no edge domain.
                if let [only] = args {
                    let dlen = self.diags.len();
                    let probed = self.expr(&only.value, None);
                    self.diags.truncate(dlen);
                    let arity = match &probed.ty {
                        Ty::List(elem) => elem.fields().map(|f| f.len()),
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
                        return self.call_gen_fn(gfid, Vec::new(), None, args, span);
                    }
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
            && matches!(method, "len" | "is_empty" | "first" | "last")
            && !matches!(ty, Ty::Set(_) | Ty::HashSet(_) | Ty::Struct(_))
        {
            return self.field(whole, base, name);
        }
        let elem = match &ty {
            Ty::List(elem) | Ty::Set(elem) | Ty::HashSet(elem) | Ty::Range(elem) => (**elem).clone(),
            Ty::Array { elem, .. } => (**elem).clone(),
            Ty::Map(key, _) | Ty::HashMap(key, _) => (**key).clone(),
            Ty::Str => Ty::Char,
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
            (Ty::Pos(d), "offset") => {
                let d = *d;
                match self.one_arg("offset", args, &Ty::Int, span) {
                    Some(k) => self.mk(
                        ExprKind::Builtin(Builtin::PosOffset(d), vec![value, k]),
                        Ty::opt(Ty::Pos(d)),
                        span,
                    ),
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
            (Ty::List(elem), "get") => {
                let opt = Ty::opt((**elem).clone());
                match self.one_arg("get", args, &Ty::Int, span) {
                    Some(at) => self.mk(ExprKind::Builtin(Builtin::SeqGet, vec![value, at]), opt, span),
                    None => self.err_expr(span),
                }
            }
            (Ty::List(_) | Ty::Str, "push") => match self.one_arg(method, args, &elem, span) {
                Some(arg) => mutate(self, Mutation::Push, vec![arg], Ty::Unit),
                None => self.err_expr(span),
            },
            (Ty::List(_), "push_id") => match self.one_arg(method, args, &elem, span) {
                // The index the new element lands on.
                Some(arg) => self.do_expr(Some(&Ty::Int), span, |lw, out| {
                    let place = lw.place(base);
                    let place = lw.settle_indexes(place, out);
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
            (Ty::List(_), "pop") => match self.positional_any(method, args, 0, span) {
                Some(_) => {
                    if matches!(elem, Ty::Opt(_)) {
                        mutate(self, Mutation::PopVal, Vec::new(), elem)
                    } else {
                        mutate(self, Mutation::Pop, Vec::new(), Ty::opt(elem))
                    }
                }
                None => self.err_expr(span),
            },
            (Ty::List(_) | Ty::Array { .. }, "swap") => {
                match self.positional_any(method, args, 2, span) {
                    Some(exprs) => {
                        let idx_ty = match &ty {
                            Ty::Array { dims, .. } if dims.len() == 1 => match dims[0] {
                                Dim::Pos(d) => Ty::Pos(d),
                                Dim::Bound(d) => Ty::Bound(d),
                                Dim::Int => Ty::Int,
                            },
                            _ => Ty::Int,
                        };
                        let a = self.expect(exprs[0], &idx_ty);
                        let b = self.expect(exprs[1], &idx_ty);
                        let a = if matches!(a.ty, Ty::Pos(_)) { self.cast(Cast::Ord, a, Ty::Int) } else { a };
                        let b = if matches!(b.ty, Ty::Pos(_)) { self.cast(Cast::Ord, b, Ty::Int) } else { b };
                        mutate(self, Mutation::Swap, vec![a, b], Ty::Unit)
                    }
                    None => self.err_expr(span),
                }
            }
            (Ty::List(_), "resize") => match self.positional_any(method, args, 2, span) {
                Some(exprs) => {
                    let n = self.expect(exprs[0], &Ty::Int);
                    let v = self.expect(exprs[1], &elem);
                    mutate(self, Mutation::Resize, vec![n, v], Ty::Unit)
                }
                None => self.err_expr(span),
            },
            (Ty::List(_), "truncate") => match self.one_arg(method, args, &Ty::Int, span) {
                Some(n) => mutate(self, Mutation::Truncate, vec![n], Ty::Unit),
                None => self.err_expr(span),
            },
            (Ty::List(_), "extend") => match self.positional_any(method, args, 1, span) {
                Some(exprs) => {
                    let other = self.expect(exprs[0], &ty);
                    mutate(self, Mutation::Extend, vec![other], Ty::Unit)
                }
                None => self.err_expr(span),
            },
            (Ty::List(_) | Ty::Array { .. }, "fill") => match self.one_arg(method, args, &elem, span) {
                Some(v) => mutate(self, Mutation::Fill, vec![v], Ty::Unit),
                None => self.err_expr(span),
            },
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
            (Ty::List(_) | Ty::Set(_) | Ty::HashSet(_) | Ty::Map(..) | Ty::HashMap(..) | Ty::Str, "clear") => {
                match self.positional_any(method, args, 0, span) {
                    Some(_) => mutate(self, Mutation::Clear, Vec::new(), Ty::Unit),
                    None => self.err_expr(span),
                }
            }
            (Ty::List(_) | Ty::Array { .. }, "reverse") => match self.positional_any(method, args, 0, span) {
                Some(_) => mutate(self, Mutation::Reverse, Vec::new(), Ty::Unit),
                None => self.err_expr(span),
            },
            (Ty::List(_) | Ty::Array { .. }, "sort") => self.sort(base, &elem, args, span),
            (Ty::List(_) | Ty::Array { .. } | Ty::Set(_) | Ty::HashSet(_), "sorted") => {
                if args.is_empty() {
                    self.sorted(value, span)
                } else {
                    self.sorted_by(value, args, span)
                }
            }
            (Ty::List(_) | Ty::Str, "reversed") => match self.positional_any(method, args, 0, span) {
                Some(_) => self.mk(
                    ExprKind::Builtin(Builtin::Reversed, vec![value]),
                    ty.clone(),
                    span,
                ),
                None => self.err_expr(span),
            },
            (
                Ty::List(_)
                | Ty::Set(_)
                | Ty::HashSet(_)
                | Ty::Map(..)
                | Ty::HashMap(..)
                | Ty::Array { .. }
                | Ty::Str
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
                        },
                        Block {
                            stmts: vec![Stmt::Expr(push)],
                        },
                    ));
                    out.push(Stmt::Give(lw.local(acc_slot, span)));
                })
            }
            (Ty::List(e), "first_ge" | "last_le" | "find") => {
                let e = (**e).clone();
                self.seq_search_method(method, value, e, None, args, span)
            }
            (Ty::Array { dims, elem: e }, "first_ge" | "last_le" | "find") if dims.len() == 1 => {
                let dom = match dims[0] {
                    Dim::Pos(d) => Some(d),
                    _ => None,
                };
                let e = (**e).clone();
                self.seq_search_method(method, value, e, dom, args, span)
            }
            (Ty::Fn(_, _), "out") => self.call_fn_val(value, args, span),
            // `Neighbours` for a table of neighbour lists: `adj.out(u)` is
            // `adj[u]`, as it is `adj(u)` for a fn.
            (Ty::Array { dims, .. }, "out") if dims.len() == 1 && args.len() == 1 => {
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
                        },
                        Block {
                            stmts: vec![Stmt::Expr(push)],
                        },
                    ));
                    out.push(Stmt::Give(lw.local(acc, span)));
                })
            }
            (_, "rev") if args.is_empty() => value,
            _ => {
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
        let key = match args {
            [] => {
                if !self.ordered(elem) {
                    let message = format!("a `{}` has no order to sort by", self.show(elem));
                    self.error(codes::BAD_OPERANDS, span, message);
                    self.help("sort by a key: `xs.sort(by: x => x.w)`");
                    return self.err_expr(span);
                }
                SortBy::Natural
            }
            [ast::Arg {
                name: Some(by),
                value:
                    ast::Expr {
                        kind: ast::ExprKind::Lambda { params, body },
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
        self.mk(ExprKind::Sort(Box::new(place), key), Ty::Unit, span)
    }

    fn domain_field(&mut self, id: DomainId, name: &ast::Name, span: Span) -> Expr {
        let len = self.mk(ExprKind::DomainLen(id), Ty::Int, span);
        match name.text.as_str() {
            "len" | "size" | "K" => len,
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
            "all" => self.mk(
                ExprKind::Range {
                    lo: Box::new(self.int_lit(0, span)),
                    hi: Box::new(len),
                },
                Ty::Range(Box::new(Ty::Pos(id))),
                span,
            ),
            other => {
                self.unsupported(name.span, &format!("`.{other}` of a domain as a value"));
                self.err_expr(span)
            }
        }
    }

    /// A `[]` list local, the loop that pushes `value` for each `item` of
    /// `iter`, and the list.
    fn collected(
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
            Ty::List(e) => ((**e).clone(), Dim::Int),
            Ty::Str => (Ty::Char, Dim::Int),
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
                    Dim::Int => i,
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

    /// `prefix(a)`: dispatches to userland `std.prefix.{prefix1, prefix2}`
    /// (`stdlib/prefix.cp`) so prefix sums stay in userland rather than bespoke IR.
    fn lower_prefix(&mut self, args: &[ast::Arg], span: Span) -> Expr {
        let Some(exprs) = self.positional_any("prefix", args, 1, span) else {
            return self.err_expr(span);
        };
        let src = self.expr(exprs[0], None);
        if src.ty == Ty::Error {
            return self.err_expr(span);
        }
        let doms: Option<Vec<DomainId>> = match &src.ty {
            Ty::Array { dims, elem } if **elem == Ty::Int && matches!(dims.len(), 1 | 2) => dims
                .iter()
                .map(|d| match d {
                    Dim::Pos(d) => Some(*d),
                    _ => None,
                })
                .collect(),
            _ => None,
        };
        let Some(doms) = doms else {
            let message = format!(
                "`prefix` takes a `[D]Int` or a `[R, C]Int`, found `{}`",
                self.show(&src.ty)
            );
            self.error(codes::TYPE_MISMATCH, span, message);
            return self.err_expr(span);
        };
        let Some(mod_id) = self.std_prefix else {
            self.unsupported(span, "`prefix` without `stdlib/prefix.cp`");
            return self.err_expr(span);
        };
        let fn_name = if doms.len() == 1 { "prefix1" } else { "prefix2" };
        let Some(Sym::GenFn(gfid)) = self.mod_syms[mod_id].get(fn_name).copied() else {
            self.unsupported(span, "`prefix` without `stdlib/prefix.cp`");
            return self.err_expr(span);
        };
        let gargs: Vec<GenArg> = doms.into_iter().map(GenArg::Domain).collect();
        let fn_id = self.instantiate_fn(gfid, gargs);
        let ret = self.ret_of(fn_id, span);
        self.mk(ExprKind::Call(fn_id, vec![Arg::Value(src)]), ret, span)
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
fn is_place(e: &ast::Expr) -> bool {
    match &e.kind {
        ast::ExprKind::Name(_) | ast::ExprKind::SelfValue => true,
        ast::ExprKind::Paren(inner) => is_place(inner),
        ast::ExprKind::Field { base, .. } | ast::ExprKind::Index { base, .. } => is_place(base),
        _ => false,
    }
}

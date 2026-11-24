//! Names as values: locals, globals, consts and generic parameters, read
//! the way a test has shown them to hold.

use super::*;

impl<'a> Lowerer<'a> {
    /// Whether the step `e`, a `?I`, is known to stay inside `I`: the
    /// same question `cpl check` asks ([`cpl_syntax::order`]).
    pub(in crate::lower) fn step_stays(&mut self, e: &ast::Expr, domain: &Ty) -> bool {
        if !matches!(domain, Ty::Pos(_)) {
            return false;
        }
        let steps = self.steps.clone();
        let Some(steps) = steps.get(self.cur_mod) else {
            return false;
        };
        steps.bounds(e, self.offset).into_iter().any(|name| {
            // `i > D.first`: an end of the domain itself.
            if let Some(of) = cpl_syntax::order::end_domain(name) {
                return matches!(domain, Ty::Pos(d) if self.resolve_domain_name(of) == Some(*d));
            }
            let (diags, written) = (self.diags.len(), self.cx.as_written);
            let ty = self.name_as_written(name, None, e.span).ty;
            self.diags.truncate(diags);
            self.cx.as_written = written;
            ty == *domain
        })
    }

    /// A name as a value. One that a test has shown to hold a value is the
    /// value it holds (see [`narrow`]).
    pub(in crate::lower) fn name(&mut self, name: &str, hint: Option<&Ty>, span: Span) -> Expr {
        if name == "io" && self.lookup("io").is_none() && self.get_sym("io").is_none() {
            let gid = self.ensure_io_global();
            let ty = self.globals[gid].ty.clone();
            return self.mk(ExprKind::Global(gid), ty, span);
        }
        // `x!`, `x ?? 0`, `match x`: the optional itself is asked for.
        let written = std::mem::take(&mut self.cx.as_written);
        let value = self.name_as_written(name, hint, span);
        let shown = |cx: &FnCx| cx.present.iter().any(|n| n == name);
        let present = !written
            && (shown(&self.cx) || (self.cx.is_closure && self.outer_cxs.iter().any(shown)));
        match value.ty.clone() {
            Ty::Opt(inner) if present => {
                self.mk(ExprKind::Unwrap(Box::new(value), false), *inner, span)
            }
            _ => value,
        }
    }

    pub(in crate::lower) fn name_as_written(
        &mut self,
        name: &str,
        hint: Option<&Ty>,
        span: Span,
    ) -> Expr {
        if let Some(view) = self.map_view_named(name) {
            return self.lower_map_view_materialize(&view, span);
        }
        if let Some(slot) = self.lookup(name) {
            return self.local(slot, span);
        }
        if let Some((ref recv_place, owner)) = self.cx.inline_self.clone() {
            if let Some((index, ty)) = self.field_of(&Ty::Struct(owner), name) {
                let base = place_to_expr(recv_place);
                return self.mk(ExprKind::Field(Box::new(base), index), ty, span);
            }
        }
        if let Some((slot, owner)) = self.cx.receiver {
            if let Some((index, ty)) = self.field_of(&Ty::Struct(owner), name) {
                let base = self.local(slot, span);
                return self.mk(ExprKind::Field(Box::new(base), index), ty, span);
            }
        }
        if let Some(sid) = self.self_struct {
            if let Some(sc) = self.structs[sid].consts.get(name).cloned() {
                return self.eval_struct_const(sid, sc, span);
            }
        }
        if let Some(slot) = self.capture_from_outer(name) {
            return self.local(slot, span);
        }
        if let Some((slot, owner)) = self.capture_self_from_outer() {
            if let Some((index, ty)) = self.field_of(&Ty::Struct(owner), name) {
                let base = self.local(slot, span);
                return self.mk(ExprKind::Field(Box::new(base), index), ty, span);
            }
        }
        // Generic value parameter (e.g. `P` in `Mod<P>`) or generic domain parameter (`C` in `(C, comp)`).
        if let Some(ga) = self.gen_env.get(name).cloned() {
            match ga {
                GenArg::Int(v) => return self.mk(ExprKind::Int(v), Ty::Int, span),
                GenArg::RuntimeInt(gid) => return self.mk(ExprKind::Global(gid), Ty::Int, span),
                GenArg::Domain(d) => return self.mk(ExprKind::DomainVal(d), Ty::DomainVal, span),
                GenArg::Ty(_) | GenArg::Op(_) | GenArg::Range(_, _) => {}
            }
        }
        if let Some(&Ty::Pos(d)) = self.cx.local_type_aliases.get(name) {
            return self.mk(ExprKind::DomainVal(d), Ty::DomainVal, span);
        }
        match self.get_sym(name) {
            Some(Sym::Global(id)) => return self.global_read(id, span),
            Some(Sym::ModConst(mcid)) => return self.ensure_mod_const(mcid, span),
            Some(Sym::Variant(id, tag)) => {
                let (id, tag) = self.expected_variant(name, hint).unwrap_or((id, tag));
                if let Some(&d) = self.enum_to_sum_domain.get(&id) {
                    if self.sum_enums[&d].variants[tag].1.is_empty() {
                        return self.encode_sum_variant(d, tag, Vec::new(), span);
                    }
                    let message = format!("the variant `{name}` holds values: write `{name}(...)`");
                    self.error(codes::WRONG_ARG_COUNT, span, message);
                    return self.err_expr(span);
                }
                if self.enums[id].variants[tag].1.is_empty() {
                    return self.mk(ExprKind::Variant(tag, Vec::new()), Ty::Enum(id), span);
                }
                let message = format!("the variant `{name}` holds values: write `{name}(...)`");
                self.error(codes::WRONG_ARG_COUNT, span, message);
            }
            Some(Sym::Fn(id)) => return self.fn_as_closure(id, hint, span),
            Some(Sym::GenFn(gfid)) => return self.gen_fn_as_closure(gfid, name, hint, span),
            Some(Sym::Domain(d)) => return self.mk(ExprKind::DomainVal(d), Ty::DomainVal, span),
            Some(Sym::Struct(id)) if self.structs[id].def.fields.is_empty() => {
                return self.mk(ExprKind::Tuple(Vec::new()), Ty::Struct(id), span);
            }
            Some(Sym::Struct(id)) => {
                // Struct constructor passed as a function value (e.g., `sort(mut pts, by: ByAngle)`).
                let fields: Vec<Ty> = self.structs[id]
                    .def
                    .fields
                    .iter()
                    .map(|(_, t)| t.clone())
                    .collect();
                let mut locals = Vec::new();
                let mut params = Vec::new();
                let mut args_expr = Vec::new();
                for (i, fty) in fields.iter().enumerate() {
                    locals.push(Local {
                        name: format!("_p{i}"),
                        ty: fty.clone(),
                    });
                    params.push(i);
                    args_expr.push(Expr {
                        kind: ExprKind::Local(i),
                        ty: fty.clone(),
                        span: self.span(span),
                    });
                }
                let body = Block {
                    stmts: vec![Stmt::Return(Some(Expr {
                        kind: ExprKind::Tuple(args_expr),
                        ty: Ty::Struct(id),
                        span: self.span(span),
                    }))],
                };
                let cid = self.closures.len();
                self.closures.push(ClosureDef {
                    params,
                    muts: Vec::new(),
                    captures: Vec::new(),
                    locals,
                    ret: Ty::Struct(id),
                    body,
                });
                let fn_ty = Ty::Fn(fields, Box::new(Ty::Struct(id)));
                return self.mk(ExprKind::Closure(cid), fn_ty, span);
            }
            Some(Sym::Alias(aid)) => {
                let (m, ty_ast, offset) = self.aliases[aid];
                let resolved = self.in_alias(aid, |lw, ty| lw.resolve_ty(ty));
                if let Ty::Pos(d) = resolved {
                    return self.mk(ExprKind::DomainVal(d), Ty::DomainVal, span);
                }
                // R04.2.10: `type Cell = (Row, Col)` as a value is the tuple of its
                // factor domains, walked like `for p in (Row, Col)`.
                if let Ty::Tuple(items) = &resolved {
                    let doms: Option<Vec<Expr>> = items
                        .iter()
                        .map(|t| match t {
                            Ty::Pos(d) => {
                                Some(self.mk(ExprKind::DomainVal(*d), Ty::DomainVal, span))
                            }
                            _ => None,
                        })
                        .collect();
                    if let Some(doms) = doms.filter(|d| d.len() > 1) {
                        let ty = Ty::Tuple(vec![Ty::DomainVal; doms.len()]);
                        return self.mk(ExprKind::Tuple(doms), ty, span);
                    }
                }
                let message = format!("`{name}` is a type, and a value is needed here");
                self.error(codes::TYPE_MISMATCH, span, message);
                if m == 0 && matches!(ty_ast.kind, ast::TypeKind::Range { .. }) {
                    self.range_alias_as_domain(name, ty_ast.span.shift(offset));
                }
            }
            Some(Sym::GenStruct(_) | Sym::Enum(_) | Sym::GenEnum(_) | Sym::GenAlias(_)) => {
                let message = format!("`{name}` is a type, and a value is needed here");
                self.error(codes::TYPE_MISMATCH, span, message);
            }
            _ if name == "INF" => return self.mk(ExprKind::Int(INF), Ty::Int, span),
            _ if matches!(name, "min" | "max" | "some" | "Some") => {
                return self.prelude_fn_value(name, hint, span);
            }
            _ if name == "todo" => {
                let at = self.span(span);
                let stmts = vec![Stmt::Trap("`todo` reached".into(), at)];
                return self.mk(ExprKind::Do(Block { stmts }), Ty::Error, span);
            }
            _ => {
                if self.tail_names.contains(name) {
                    let gen = matches!(self.aside, Some(Aside::Gen(_)));
                    let message = if gen {
                        format!(
                            "`{name}` is read from the input, which a `@gen` fn is there to make"
                        )
                    } else {
                        format!("`{name}` is the solution's own work: a brute or a checker shares only the input")
                    };
                    self.error(codes::UNRESOLVED_NAME, span, message);
                    self.help(if gen {
                        "a `@gen` fn shares only `const`s and fns with the file; pick the value with `rand(lo, hi)` and print it"
                    } else {
                        "it sees every top-level line down to the last `read()`; compute the rest inside the fn"
                    });
                } else if self.judge {
                    let message = format!("an `@interactor` can't see the solution's `{name}`");
                    self.error(codes::UNRESOLVED_NAME, span, message);
                    self.help("the judge is a program of its own: it reads the hidden test with `read()` and shares only `const`s with the solution");
                } else if !self.reported_an_error() {
                    let message = format!("no value named `{name}` here");
                    self.error(codes::UNRESOLVED_NAME, span, message);
                    self.help("a top-level `let` is visible only after the line that sets it");
                }
            }
        }
        self.err_expr(span)
    }

    pub(in crate::lower) fn generic_expr(
        &mut self,
        base: &ast::Expr,
        args: &[ast::Type],
        span: Span,
    ) -> Expr {
        if let ast::ExprKind::Name(name) = &base.kind {
            if let Some(Sym::GenConst(gcid)) = self.get_sym(name) {
                let gargs = self.resolve_gen_args(args);
                return self.eval_gen_const(gcid, gargs);
            }
        }
        if let ast::ExprKind::Field { base: mbase, name } = &base.kind {
            if let Some(Sym::Mod(target_mod)) = self.sym(mbase) {
                let saved_mod = std::mem::replace(&mut self.cur_mod, target_mod);
                let sym = self.mod_syms[target_mod].get(&name.text).copied();
                self.cur_mod = saved_mod;
                if let Some(Sym::GenConst(gcid)) = sym {
                    let gargs = self.resolve_gen_args(args);
                    return self.eval_gen_const(gcid, gargs);
                }
            }
        }
        self.unsupported(span, "explicit type arguments here");
        self.err_expr(span)
    }
}

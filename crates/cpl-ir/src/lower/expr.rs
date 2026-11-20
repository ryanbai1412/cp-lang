//! Expressions.

use super::stmt::place_to_expr;
use super::*;

/// One `for` of a comprehension while its body is being lowered.
pub(super) struct LoopFrame {
    id: LoopId,
    slot: Slot,
    iter: Iter,
    /// Statements that take the loop variable's pattern apart.
    pre: Vec<Stmt>,
    cond: Option<Expr>,
}

impl<'a> Lowerer<'a> {
    /// Lowers `e` where a `want` is needed.
    pub(super) fn expect(&mut self, e: &ast::Expr, want: &Ty) -> Expr {
        let value = self.expr(e, Some(want));
        self.coerce(value, want, e.span)
    }

    /// Lowers `e`. `hint` is the type the context would like, which only
    /// the forms that can't tell on their own look at: `none`, `[]`,
    /// `read()`, and number literals.
    pub(super) fn expr(&mut self, e: &ast::Expr, hint: Option<&Ty>) -> Expr {
        // Where an optional is wanted, a plain value is typed as the inside.
        let inner_hint = match hint {
            Some(Ty::Opt(inner)) => Some(&**inner),
            other => other,
        };
        let span = e.span;
        match &e.kind {
            ast::ExprKind::Int(value) => match i64::try_from(*value) {
                Ok(value) if inner_hint == Some(&Ty::Float) => {
                    self.mk(ExprKind::Float(value as f64), Ty::Float, span)
                }
                Ok(value) => self.mk(ExprKind::Int(value), Ty::Int, span),
                Err(_) => {
                    self.error(
                        codes::BAD_NUMBER,
                        span,
                        "this number does not fit in an `Int` (64 bits)",
                    );
                    self.err_expr(span)
                }
            },
            ast::ExprKind::Float(text) => {
                let clean: String = text.chars().filter(|c| *c != '_').collect();
                match clean.parse::<f64>() {
                    Ok(value) => self.mk(ExprKind::Float(value), Ty::Float, span),
                    Err(_) => {
                        self.error(codes::BAD_NUMBER, span, "this is not a number");
                        self.err_expr(span)
                    }
                }
            }
            ast::ExprKind::Str(pieces) => self.string(pieces, span),
            ast::ExprKind::Char(c) => match self.ascii(&c.to_string(), span) {
                Some(bytes) => self.mk(ExprKind::Char(bytes[0]), Ty::Char, span),
                None => self.err_expr(span),
            },
            ast::ExprKind::Bool(value) => self.mk(ExprKind::Bool(*value), Ty::Bool, span),
            ast::ExprKind::None => match hint {
                Some(ty @ Ty::Opt(_)) => self.mk(ExprKind::None, ty.clone(), span),
                Some(Ty::Error) => self.err_expr(span),
                _ => {
                    self.error(
                        codes::TYPE_MISMATCH,
                        span,
                        "can't tell which optional this `none` is",
                    );
                    self.help("say the type where it is stored: `let best: ?Int = none`");
                    self.err_expr(span)
                }
            },
            ast::ExprKind::Name(name) => self.name(name, inner_hint, span),
            ast::ExprKind::SelfValue => {
                if let Some(slot) = self.lookup("self") {
                    return self.local(slot, span);
                }
                if let Some((ref recv_place, _)) = self.cx.inline_self {
                    return place_to_expr(recv_place);
                }
                if let Some((slot, _)) = self.cx.receiver {
                    return self.local(slot, span);
                }
                if let Some((slot, _)) = self.capture_self_from_outer() {
                    return self.local(slot, span);
                }
                self.error(codes::UNRESOLVED_NAME, span, "`self` outside a method");
                self.err_expr(span)
            }
            ast::ExprKind::Paren(inner) if self.is_filled(inner) => {
                let held = self.expr(inner, None);
                match held.ty.clone() {
                    Ty::Opt(ty) => self.mk(ExprKind::Unwrap(Box::new(held), false), *ty, span),
                    _ => held,
                }
            }
            ast::ExprKind::Paren(inner) => self.expr(inner, hint),
            ast::ExprKind::Unary { op, expr } => self.unary(*op, expr, inner_hint, span),
            ast::ExprKind::Binary { op, lhs, rhs, .. } => self.binary(*op, lhs, rhs, span),
            ast::ExprKind::Compare { first, rest } => self.compare(first, rest, span),
            ast::ExprKind::Range {
                lo,
                hi,
                inclusive,
                step: None,
            } => self.range_val(lo.as_deref(), hi.as_deref(), *inclusive, span),
            ast::ExprKind::Range { .. } => {
                self.unsupported(span, "a stepped range kept as a value");
                self.err_expr(span)
            }
            ast::ExprKind::Call { callee, args } => self.call(e, callee, args, hint),
            ast::ExprKind::Index { base, indices } => {
                let raw_base = self.expr(base, None);
                let base_expr = match (&raw_base.kind, &raw_base.ty) {
                    (ExprKind::Builtin(Builtin::First, args), Ty::Opt(inner)) => Expr {
                        kind: ExprKind::Builtin(Builtin::FirstVal, args.clone()),
                        ty: (**inner).clone(),
                        span: raw_base.span,
                    },
                    (ExprKind::Builtin(Builtin::Last, args), Ty::Opt(inner)) => Expr {
                        kind: ExprKind::Builtin(Builtin::LastVal, args.clone()),
                        ty: (**inner).clone(),
                        span: raw_base.span,
                    },
                    _ => raw_base,
                };
                if let Ty::Struct(sid) = base_expr.ty {
                    let ast_args: Vec<ast::Arg> = indices
                        .iter()
                        .map(|idx| ast::Arg {
                            name: None,
                            mutable: false,
                            value: idx.clone(),
                        })
                        .collect();
                    if let Some(&fn_id) = self.structs[sid].methods.get("get") {
                        return self.method_call(fn_id, Some((base, base_expr)), &ast_args, span);
                    }
                    if let Some(&gfn_id) = self.structs[sid].gen_methods.get("get") {
                        return self.call_gen_fn(
                            gfn_id,
                            Vec::new(),
                            Some(Arg::Value(base_expr)),
                            &ast_args,
                            span,
                        );
                    }
                }
                if indices.len() == 1 && matches!(indices[0].kind, ast::ExprKind::Range { .. }) {
                    return self.slice_expr(base_expr, &indices[0], span);
                }
                // Runtime `Range<T>` value passed as an index (e.g. `vals[r]` inside `Compressed.get<Range<D>>`).
                if indices.len() == 1
                    && matches!(
                        base_expr.ty,
                        Ty::Array { .. } | Ty::List(_) | Ty::Str
                    )
                {
                    if let Some(Ty::Range(_)) = self.probe_ty_silent(&indices[0]) {
                        let range_val = self.expr(&indices[0], None);
                        return self.slice_by_range_val(base_expr, range_val, span);
                    }
                }
                let (indices, ty) = self.index_args(&base_expr.ty, indices, span);
                self.mk(ExprKind::Index(Box::new(base_expr), indices), ty, span)
            }
            ast::ExprKind::Field { base, name } => self.field(e, base, name),
            ast::ExprKind::Generic { base, args } => self.generic_expr(base, args, span),
            ast::ExprKind::Tuple(items) => {
                let hints = inner_hint
                    .and_then(Ty::fields)
                    .filter(|f| f.len() == items.len());
                let values: Vec<Expr> = items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| match &hints {
                        Some(hints) => self.expect(item, &hints[i]),
                        None => self.expr(item, None),
                    })
                    .collect();
                let ty = match (inner_hint, &hints) {
                    (Some(ty @ (Ty::Tuple(_) | Ty::Record(_))), Some(_)) => ty.clone(),
                    _ => Ty::Tuple(values.iter().map(|v| v.ty.clone()).collect()),
                };
                self.mk(ExprKind::Tuple(values), ty, span)
            }
            ast::ExprKind::Record(fields) => {
                let hints = match inner_hint {
                    Some(Ty::Record(want))
                        if want.len() == fields.len()
                            && want.iter().zip(fields).all(|((w, _), (f, _))| *w == f.text) =>
                    {
                        Some(want.clone())
                    }
                    _ => None,
                };
                let values: Vec<Expr> = fields
                    .iter()
                    .enumerate()
                    .map(|(i, (_, value))| match &hints {
                        Some(hints) => self.expect(value, &hints[i].1),
                        None => self.expr(value, None),
                    })
                    .collect();
                let ty = Ty::Record(
                    fields
                        .iter()
                        .zip(&values)
                        .map(|((name, _), value)| (name.text.clone(), value.ty.clone()))
                        .collect(),
                );
                self.mk(ExprKind::Tuple(values), ty, span)
            }
            ast::ExprKind::List(items) => self.list(items, inner_hint, span),
            ast::ExprKind::Map(entries) => self.map_lit(entries, inner_hint, span),
            ast::ExprKind::Comprehension { body, clauses } => {
                self.comprehension(body, clauses, inner_hint, span)
            }
            ast::ExprKind::Table {
                domains,
                params,
                body,
            } => self.table(domains, params, body, inner_hint, span),
            ast::ExprKind::Lambda { params, body } => {
                self.lower_lambda(params, body, inner_hint, span)
            }
            ast::ExprKind::Block(block) => self.do_expr(hint, span, |lw, out| {
                out.extend(lw.block(block, Tail::Give).stmts);
            }),
            ast::ExprKind::If {
                branches,
                otherwise,
            } => {
                if let Some(value) = self.ternary(branches, otherwise.as_ref(), hint) {
                    return value;
                }
                self.do_expr(hint, span, |lw, out| lw.tail(e, Tail::Give, out))
            }
            ast::ExprKind::Match { .. } => {
                self.do_expr(hint, span, |lw, out| lw.tail(e, Tail::Give, out))
            }
            ast::ExprKind::With { .. } => {
                self.unsupported(span, "`with`");
                self.err_expr(span)
            }
            ast::ExprKind::Try(inner) => self.try_expr(inner, span),
            ast::ExprKind::Unwrap(inner) => {
                let value = self.expr_maybe_opt_reduce(inner);
                match value.ty.clone() {
                    Ty::Opt(inside) => {
                        self.mk(ExprKind::Unwrap(Box::new(value), true), *inside, span)
                    }
                    Ty::Error => value,
                    // Inside a generic function (e.g. `bsearch_first<I>` instantiated with `I = Int`),
                    // `(base + mid)!` is a no-op when `base + mid` is already non-optional `Int`.
                    _ if !self.gen_env.is_empty() => value,
                    other => {
                        let message = format!(
                            "`!` takes the value out of an optional, but this is a `{}`",
                            self.show(&other)
                        );
                        self.error(codes::TYPE_MISMATCH, span, message);
                        self.err_expr(span)
                    }
                }
            }
            ast::ExprKind::Missing | ast::ExprKind::Error => self.err_expr(span),
        }
    }

    /// A block with a value. The first value given decides the type when
    /// the context doesn't.
    pub(super) fn do_expr(
        &mut self,
        hint: Option<&Ty>,
        span: Span,
        body: impl FnOnce(&mut Self, &mut Vec<Stmt>),
    ) -> Expr {
        self.cx.dos.push(DoFrame {
            ty: hint.cloned(),
            loops: self.cx.loops.len(),
        });
        let mut out = Vec::new();
        body(self, &mut out);
        let frame = self.cx.dos.pop().expect("pushed above");
        self.mk(
            ExprKind::Do(Block { stmts: out }),
            frame.ty.unwrap_or(Ty::Error),
            span,
        )
    }

    /// An `if` whose branches are single expressions is a conditional
    /// expression: `if a < b: b - a else: a - b`.
    fn ternary(
        &mut self,
        branches: &[(ast::Cond, ast::Block)],
        otherwise: Option<&ast::Block>,
        hint: Option<&Ty>,
    ) -> Option<Expr> {
        fn single(block: &ast::Block) -> Option<&ast::Expr> {
            match block.stmts.as_slice() {
                [ast::Stmt {
                    kind: ast::StmtKind::Expr(e),
                    ..
                }] if !matches!(
                    e.kind,
                    ast::ExprKind::If { .. }
                        | ast::ExprKind::Match { .. }
                        | ast::ExprKind::Block(_)
                ) =>
                {
                    Some(e)
                }
                _ => None,
            }
        }
        let otherwise = single(otherwise?)?;
        let mut arms = Vec::new();
        for (cond, block) in branches {
            let ast::Cond::Expr(cond) = cond else {
                return None;
            };
            arms.push((cond, single(block)?));
        }
        let mut lowered = Vec::new();
        let mut ty = hint.cloned();
        let first_needs_hint = ty.is_none()
            && arms.first().is_some_and(|(_, v)| {
                matches!(v.kind, ast::ExprKind::None)
                    || matches!(&v.kind, ast::ExprKind::List(items) if items.is_empty())
            });
        let mut pre_otherwise = None;
        if first_needs_hint {
            let other_val = self.expr(otherwise, None);
            if other_val.ty != Ty::Error {
                if matches!(arms[0].1.kind, ast::ExprKind::None) {
                    ty = Some(match &other_val.ty {
                        Ty::Opt(_) => other_val.ty.clone(),
                        t => Ty::opt(t.clone()),
                    });
                } else {
                    ty = Some(other_val.ty.clone());
                }
            }
            pre_otherwise = Some(other_val);
        }
        for (cond, value) in arms {
            let cond = self.expect(cond, &Ty::Bool);
            let value = self.expr(value, ty.as_ref());
            if ty.is_none() {
                ty = Some(value.ty.clone());
            }
            let at = value.span;
            lowered.push((cond, value, at));
        }
        let mut result = match pre_otherwise {
            Some(v) => v,
            None => self.expr(otherwise, ty.as_ref()),
        };
        // The branches agree on one type: the hint, or the widest of them.
        let mut ty = ty.unwrap_or(Ty::Error);
        for candidate in lowered.iter().map(|(_, v, _)| &v.ty).chain([&result.ty]) {
            if *candidate == Ty::Float && ty == Ty::Int {
                ty = Ty::Float;
            }
            if let Ty::Opt(inner) = candidate {
                if **inner == ty {
                    ty = candidate.clone();
                }
            }
        }
        result = self.coerce(result, &ty, otherwise.span);
        for (cond, value, span) in lowered.into_iter().rev() {
            let value = match self.try_coerce(value, &ty) {
                Ok(value) => value,
                Err(value) => {
                    let message = format!(
                        "the branches of this `if` have different types: `{}` and `{}`",
                        self.show(&ty),
                        self.show(&value.ty)
                    );
                    self.diags
                        .push(Diagnostic::error(codes::TYPE_MISMATCH, span, message));
                    Expr {
                        kind: ExprKind::Error,
                        ty: Ty::Error,
                        span,
                    }
                }
            };
            let span = cond.span;
            result = Expr {
                kind: ExprKind::If(Box::new(cond), Box::new(value), Box::new(result)),
                ty: ty.clone(),
                span,
            };
        }
        Some(result)
    }

    pub(super) fn name(&mut self, name: &str, hint: Option<&Ty>, span: Span) -> Expr {
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
                GenArg::Ty(_) | GenArg::Op(_) => {}
            }
        }
        match self.get_sym(name) {
            Some(Sym::Global(id)) => {
                return self.mk(ExprKind::Global(id), self.globals[id].ty.clone(), span)
            }
            Some(Sym::ModConst(mcid)) => return self.ensure_mod_const(mcid, span),
            Some(Sym::Variant(id, tag)) => {
                if self.enums[id].variants[tag].1.is_empty() {
                    return self.mk(ExprKind::Variant(tag, Vec::new()), Ty::Enum(id), span);
                }
                let message = format!("the variant `{name}` holds values: write `{name}(...)`");
                self.error(codes::WRONG_ARG_COUNT, span, message);
            }
            Some(Sym::Fn(id)) => return self.fn_as_closure(id, span),
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
                    captures: Vec::new(),
                    locals,
                    ret: Ty::Struct(id),
                    body,
                });
                let fn_ty = Ty::Fn(fields, Box::new(Ty::Struct(id)));
                return self.mk(ExprKind::Closure(cid), fn_ty, span);
            }
            Some(Sym::GenStruct(_) | Sym::Enum(_) | Sym::GenEnum(_) | Sym::Alias(_) | Sym::GenAlias(_)) => {
                let message = format!("`{name}` is a type, and a value is needed here");
                self.error(codes::TYPE_MISMATCH, span, message);
            }
            _ if name == "INF" => return self.mk(ExprKind::Int(INF), Ty::Int, span),
            _ if matches!(name, "min" | "max") => {
                if let Some(Ty::Fn(ps, r)) = hint {
                    if ps.len() == 2 {
                        let a_ty = ps[0].clone();
                        let op = if name == "min" { Builtin::Min } else { Builtin::Max };
                        let mut locals = vec![
                            Local { name: "a".into(), ty: a_ty.clone() },
                            Local { name: "b".into(), ty: a_ty.clone() },
                        ];
                        let body = Block {
                            stmts: vec![Stmt::Return(Some(Expr {
                                kind: ExprKind::Builtin(
                                    op,
                                    vec![
                                        Expr { kind: ExprKind::Local(0), ty: a_ty.clone(), span: self.span(span) },
                                        Expr { kind: ExprKind::Local(1), ty: a_ty.clone(), span: self.span(span) },
                                    ],
                                ),
                                ty: (**r).clone(),
                                span: self.span(span),
                            }))],
                        };
                        let cid = self.closures.len();
                        self.closures.push(ClosureDef {
                            params: vec![0, 1],
                            captures: Vec::new(),
                            locals: std::mem::take(&mut locals),
                            ret: (**r).clone(),
                            body,
                        });
                        return self.mk(ExprKind::Closure(cid), Ty::Fn(ps.clone(), r.clone()), span);
                    }
                }
                self.error(codes::UNRESOLVED_NAME, span, format!("`cpl run` doesn't know the name `{name}` here"));
            }
            _ => {
                let message = format!("`cpl run` doesn't know the name `{name}` here");
                self.error(codes::UNRESOLVED_NAME, span, message);
                self.help("a top-level `let` is visible only after the line that sets it");
            }
        }
        self.err_expr(span)
    }

    fn generic_expr(&mut self, base: &ast::Expr, args: &[ast::Type], span: Span) -> Expr {
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

    fn range_val(
        &mut self,
        lo: Option<&ast::Expr>,
        hi: Option<&ast::Expr>,
        inclusive: bool,
        span: Span,
    ) -> Expr {
        let lo_e = lo.map(|e| self.expr(e, None));
        let hi_e = hi.map(|e| self.expr(e, None));
        match (lo_e, hi_e) {
            (Some(l), Some(r)) => match (&l.ty, &r.ty) {
                (Ty::Int, Ty::Int) => {
                    let hi = if inclusive { self.int_add(r, 1) } else { r };
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(l),
                            hi: Box::new(hi),
                        },
                        Ty::Range(Box::new(Ty::Int)),
                        span,
                    )
                }
                (Ty::Pos(d), Ty::Pos(d2)) if d == d2 => {
                    let d = *d;
                    let lo_ord = self.cast(Cast::Ord, l, Ty::Int);
                    let hi_ord = self.cast(Cast::Ord, r, Ty::Int);
                    let hi_ord = if inclusive { self.int_add(hi_ord, 1) } else { hi_ord };
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(lo_ord),
                            hi: Box::new(hi_ord),
                        },
                        Ty::Range(Box::new(Ty::Pos(d))),
                        span,
                    )
                }
                (Ty::Bound(d), Ty::Bound(d2)) if d == d2 => {
                    let d = *d;
                    let lo_ord = self.cast(Cast::Ord, l, Ty::Int);
                    let hi_ord = self.cast(Cast::Ord, r, Ty::Int);
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(lo_ord),
                            hi: Box::new(hi_ord),
                        },
                        Ty::Range(Box::new(Ty::Pos(d))),
                        span,
                    )
                }
                (Ty::Error, _) | (_, Ty::Error) => self.err_expr(span),
                _ => {
                    self.mismatch(span, &l.ty, &r.ty);
                    self.err_expr(span)
                }
            },
            (Some(l), None) => match l.ty {
                Ty::Pos(d) | Ty::Bound(d) => {
                    let lo_ord = self.cast(Cast::Ord, l, Ty::Int);
                    let hi_ord = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(lo_ord),
                            hi: Box::new(hi_ord),
                        },
                        Ty::Range(Box::new(Ty::Pos(d))),
                        span,
                    )
                }
                _ => {
                    self.unsupported(span, "an unbounded range value");
                    self.err_expr(span)
                }
            },
            (None, Some(r)) => match r.ty {
                Ty::Int => {
                    let lo = self.int_lit(0, span);
                    let hi = if inclusive { self.int_add(r, 1) } else { r };
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(lo),
                            hi: Box::new(hi),
                        },
                        Ty::Range(Box::new(Ty::Int)),
                        span,
                    )
                }
                Ty::Pos(d) => {
                    let lo_ord = self.int_lit(0, span);
                    let hi_ord = self.cast(Cast::Ord, r, Ty::Int);
                    let hi_ord = if inclusive { self.int_add(hi_ord, 1) } else { hi_ord };
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(lo_ord),
                            hi: Box::new(hi_ord),
                        },
                        Ty::Range(Box::new(Ty::Pos(d))),
                        span,
                    )
                }
                Ty::Bound(d) => {
                    let lo_ord = self.int_lit(0, span);
                    let hi_ord = self.cast(Cast::Ord, r, Ty::Int);
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(lo_ord),
                            hi: Box::new(hi_ord),
                        },
                        Ty::Range(Box::new(Ty::Pos(d))),
                        span,
                    )
                }
                _ => {
                    self.unsupported(span, "an unbounded range value");
                    self.err_expr(span)
                }
            },
            (None, None) => {
                self.unsupported(span, "a range with neither end");
                self.err_expr(span)
            }
        }
    }

    fn try_expr(&mut self, inner: &ast::Expr, span: Span) -> Expr {
        let opt = self.expr(inner, None);
        let Ty::Opt(inner_ty) = opt.ty.clone() else {
            if opt.ty != Ty::Error {
                self.error(
                    codes::TYPE_MISMATCH,
                    span,
                    format!("`?` needs an optional, found `{}`", self.show(&opt.ty)),
                );
            }
            return self.err_expr(span);
        };
        let ret_ty = self.cx.ret.clone().unwrap_or_else(|| Ty::opt((*inner_ty).clone()));
        self.do_expr(Some(&inner_ty), span, |lw, out| {
            let slot = lw.temp(opt.ty.clone());
            out.push(Stmt::Let(slot, Some(opt)));
            let held = lw.local(slot, span);
            let is_some = lw.mk(ExprKind::IsSome(Box::new(held.clone())), Ty::Bool, span);
            let is_none = lw.mk(ExprKind::Unary(UnOp::Not, Box::new(is_some)), Ty::Bool, span);
            let ret_none = lw.mk(ExprKind::None, ret_ty.clone(), span);
            out.push(Stmt::If(
                is_none,
                Block {
                    stmts: vec![Stmt::Return(Some(ret_none))],
                },
                Block::default(),
            ));
            let val = lw.mk(ExprKind::Unwrap(Box::new(held), false), (*inner_ty).clone(), span);
            out.push(Stmt::Give(val));
        })
    }

    fn map_lit(&mut self, entries: &[(ast::Expr, ast::Expr)], hint: Option<&Ty>, span: Span) -> Expr {
        let (kh, vh, hashed) = match hint {
            Some(Ty::Map(k, v)) => (Some((**k).clone()), Some((**v).clone()), false),
            Some(Ty::HashMap(k, v)) => (Some((**k).clone()), Some((**v).clone()), true),
            _ => (None, None, false),
        };
        let mut pairs = Vec::with_capacity(entries.len());
        let mut kt = kh;
        let mut vt = vh;
        for (k, v) in entries {
            let ke = match &kt {
                Some(t) => self.expect(k, t),
                None => {
                    let e = self.expr(k, None);
                    kt = Some(e.ty.clone());
                    e
                }
            };
            let ve = match &vt {
                Some(t) => self.expect(v, t),
                None => {
                    let e = self.expr(v, None);
                    vt = Some(e.ty.clone());
                    e
                }
            };
            pairs.push((ke, ve));
        }
        let k_ty = kt.unwrap_or(Ty::Int);
        let v_ty = vt.unwrap_or(Ty::Int);
        let map_ty = if hashed {
            Ty::HashMap(Box::new(k_ty), Box::new(v_ty.clone()))
        } else {
            Ty::Map(Box::new(k_ty), Box::new(v_ty.clone()))
        };
        self.do_expr(Some(&map_ty), span, |lw, out| {
            let slot = lw.temp(map_ty.clone());
            out.push(Stmt::Let(
                slot,
                Some(lw.mk(ExprKind::Tuple(Vec::new()), map_ty.clone(), span)),
            ));
            for (k, v) in pairs {
                let place = Place {
                    kind: PlaceKind::Index(
                        Box::new(Place {
                            kind: PlaceKind::Local(slot),
                            ty: map_ty.clone(),
                            span: lw.span(span),
                        }),
                        vec![k],
                    ),
                    ty: v_ty.clone(),
                    span: lw.span(span),
                };
                out.push(Stmt::Assign(place, AssignOp::Set, v));
            }
            let res = lw.local(slot, span);
            out.push(Stmt::Give(res));
        })
    }

    /// The bytes of literal text. `Char` is a byte and `Str` is bytes, so
    /// text outside ASCII has no value.
    fn ascii(&mut self, text: &str, span: Span) -> Option<Vec<u8>> {
        match text.chars().find(|c| !c.is_ascii()) {
            None => Some(text.as_bytes().to_vec()),
            Some(c) => {
                let message =
                    format!("`{c}` is not ASCII: a `Char` is one byte and a `Str` is a byte array");
                self.error(codes::NON_ASCII, span, message);
                self.help("only ASCII can be written in a literal; other bytes come from input");
                None
            }
        }
    }

    fn string(&mut self, pieces: &[ast::StrPiece], span: Span) -> Expr {
        let mut out: Vec<FmtPiece> = Vec::new();
        for piece in pieces {
            match piece {
                ast::StrPiece::Text(text) => {
                    let Some(bytes) = self.ascii(text, span) else {
                        return self.err_expr(span);
                    };
                    // `{{` and `}}` are how a brace is written.
                    let mut plain = Vec::with_capacity(bytes.len());
                    let mut i = 0;
                    while i < bytes.len() {
                        plain.push(bytes[i]);
                        let doubled =
                            matches!(bytes[i], b'{' | b'}') && bytes.get(i + 1) == Some(&bytes[i]);
                        i += if doubled { 2 } else { 1 };
                    }
                    match out.last_mut() {
                        Some(FmtPiece::Text(prev)) => prev.extend(plain),
                        _ => out.push(FmtPiece::Text(plain)),
                    }
                }
                ast::StrPiece::Expr(e) => {
                    let value = self.expr(e, None);
                    if let Some(arg) = self.fmt_arg(value, 6, e.span) {
                        out.push(FmtPiece::Value(arg));
                    }
                }
                ast::StrPiece::Fixed(e, digits) => {
                    let value = self.expr(e, None);
                    if !matches!(value.ty, Ty::Float | Ty::Error) {
                        let message = format!(
                            "`:.{digits}f` sets the digits of an `F64`, but this is a `{}`",
                            self.show(&value.ty)
                        );
                        self.error(codes::TYPE_MISMATCH, e.span, message);
                    }
                    if let Some(arg) = self.fmt_arg(value, *digits, e.span) {
                        out.push(FmtPiece::Value(arg));
                    }
                }
            }
        }
        match out.as_slice() {
            [] => self.mk(ExprKind::Str(Vec::new()), Ty::Str, span),
            [FmtPiece::Text(text)] => self.mk(ExprKind::Str(text.clone()), Ty::Str, span),
            _ => self.mk(ExprKind::Format(out), Ty::Str, span),
        }
    }

    /// A value to print. Everything prints except nothing.
    pub(super) fn fmt_arg(&mut self, value: Expr, digits: u32, span: Span) -> Option<FmtArg> {
        if value.ty == Ty::Unit {
            self.error(codes::TYPE_MISMATCH, span, "this has no value to print");
            return None;
        }
        Some(FmtArg { value, digits })
    }

    /// A `Str` or a `Char` as a `Str`, for `+`.
    pub(super) fn as_str(&mut self, value: Expr, span: Span) -> Expr {
        match value.ty {
            Ty::Str | Ty::Error => value,
            Ty::Char => {
                let at = value.span;
                Expr {
                    kind: ExprKind::Format(vec![FmtPiece::Value(FmtArg { value, digits: 6 })]),
                    ty: Ty::Str,
                    span: at,
                }
            }
            _ => {
                self.mismatch(span, &Ty::Str, &value.ty);
                self.err_expr(span)
            }
        }
    }

    fn unary(&mut self, op: UnOp, operand: &ast::Expr, hint: Option<&Ty>, span: Span) -> Expr {
        match op {
            UnOp::Neg => {
                let value = self.expr(operand, hint);
                match (&value.kind, &value.ty) {
                    (ExprKind::Int(v), _) => self.mk(ExprKind::Int(-*v), Ty::Int, span),
                    (ExprKind::Float(v), _) => self.mk(ExprKind::Float(-*v), Ty::Float, span),
                    (_, Ty::Int | Ty::Float) => {
                        let ty = value.ty.clone();
                        self.mk(ExprKind::Unary(op, Box::new(value)), ty, span)
                    }
                    (_, Ty::Struct(sid)) => {
                        let sid = *sid;
                        if let Some(fn_id) = self.struct_op_fn(sid, "neg") {
                            let ret = self.ret_of(fn_id, span);
                            return self.mk(
                                ExprKind::Call(fn_id, vec![Arg::Value(value)]),
                                ret,
                                span,
                            );
                        }
                        let message =
                            format!("`-` can't be applied to a `{}`", self.show(&value.ty));
                        self.error(codes::BAD_OPERANDS, span, message);
                        self.err_expr(span)
                    }
                    (_, Ty::Error) => value,
                    // `-x` on a `?Int` or `?F64` keeps `none`.
                    (_, Ty::Opt(inner)) if matches!(**inner, Ty::Int | Ty::Float) => {
                        let zero = if **inner == Ty::Int {
                            self.mk(ExprKind::Int(0), Ty::Int, span)
                        } else {
                            self.mk(ExprKind::Float(0.0), Ty::Float, span)
                        };
                        self.tropical(BinOp::Sub, zero, value, span)
                    }
                    _ => {
                        let message =
                            format!("`-` can't be applied to a `{}`", self.show(&value.ty));
                        self.error(codes::BAD_OPERANDS, span, message);
                        self.err_expr(span)
                    }
                }
            }
            UnOp::Not => {
                let value = self.expect(operand, &Ty::Bool);
                self.mk(ExprKind::Unary(op, Box::new(value)), Ty::Bool, span)
            }
            UnOp::BitNot => {
                let value = self.expect(operand, &Ty::Int);
                self.mk(ExprKind::Unary(op, Box::new(value)), Ty::Int, span)
            }
        }
    }

    pub(super) fn cast(&self, cast: Cast, value: Expr, ty: Ty) -> Expr {
        let span = value.span;
        Expr {
            kind: ExprKind::Cast(cast, Box::new(value)),
            ty,
            span,
        }
    }

    fn binary(&mut self, op: BinOp, lhs: &ast::Expr, rhs: &ast::Expr, span: Span) -> Expr {
        use BinOp::*;
        match op {
            And | Or => {
                let l = self.expect(lhs, &Ty::Bool);
                let r = self.expect(rhs, &Ty::Bool);
                return self.mk(
                    ExprKind::Binary(op, Box::new(l), Box::new(r)),
                    Ty::Bool,
                    span,
                );
            }
            Coalesce => {
                let l = self.expr_maybe_opt_reduce(lhs);
                let Ty::Opt(inner) = l.ty.clone() else {
                    if l.ty != Ty::Error {
                        let message = format!(
                            "`??` gives a default for an optional, but the left side is a `{}`",
                            self.show(&l.ty)
                        );
                        self.error(codes::BAD_OPERANDS, span, message);
                    }
                    self.expr(rhs, None);
                    return self.err_expr(span);
                };
                if let ast::ExprKind::Block(block) = &rhs.kind {
                    if let [way_out @ ast::Stmt {
                        kind:
                            ast::StmtKind::Return(_) | ast::StmtKind::Break(_) | ast::StmtKind::Continue(_),
                        ..
                    }] = block.stmts.as_slice()
                    {
                        if self.cx.way_out.take() != Some(span) {
                            self.error(
                                codes::UNSUPPORTED,
                                rhs.span,
                                "`?? return`, `?? break` and `?? continue` have to be the whole right side of a `let`, an assignment or a statement",
                            );
                            self.help("give the value a name first: `let u = o ?? return`");
                            return self.err_expr(span);
                        }
                        let slot = self.temp(l.ty.clone());
                        let held = self.local(slot, span);
                        let some = self.mk(ExprKind::IsSome(Box::new(held.clone())), Ty::Bool, span);
                        let missing = self.mk(ExprKind::Unary(UnOp::Not, Box::new(some)), Ty::Bool, span);
                        let mut leave = Vec::new();
                        // The way out is lowered as the statement it is,
                        // outside any value block.
                        let dos = std::mem::take(&mut self.cx.dos);
                        self.stmt(way_out, &mut leave);
                        self.cx.dos = dos;
                        self.cx.pending.push(Stmt::Let(slot, Some(l)));
                        self.cx.pending.push(Stmt::If(missing, Block { stmts: leave }, Block::default()));
                        return self.mk(ExprKind::Unwrap(Box::new(held), false), *inner, span);
                    }
                }
                let r = self.expr(rhs, Some(&l.ty));
                // `a ?? b ?? 0`: an optional default keeps the result optional.
                let ty = if r.ty == l.ty { l.ty.clone() } else { *inner };
                let r = self.coerce(r, &ty, rhs.span);
                return self.mk(ExprKind::Binary(op, Box::new(l), Box::new(r)), ty, span);
            }
            _ => {}
        }
        let unwrap_first_last = |e: Expr| -> Expr {
            match (&e.kind, &e.ty) {
                (ExprKind::Builtin(Builtin::First, args), Ty::Opt(inner)) => Expr {
                    kind: ExprKind::Builtin(Builtin::FirstVal, args.clone()),
                    ty: (**inner).clone(),
                    span: e.span,
                },
                (ExprKind::Builtin(Builtin::Last, args), Ty::Opt(inner)) => Expr {
                    kind: ExprKind::Builtin(Builtin::LastVal, args.clone()),
                    ty: (**inner).clone(),
                    span: e.span,
                },
                _ => e,
            }
        };
        let l = unwrap_first_last(self.expr(lhs, None));
        let r = unwrap_first_last(self.expr(rhs, Some(&l.ty)));
        // Dispatch operator traits (`impl Add/Sub/Mul/Div for Struct`).
        if let Ty::Struct(sid) = l.ty {
            let mname = match op {
                Add => Some("add"),
                Sub => Some("sub"),
                Mul => Some("mul"),
                Div => Some("div"),
                _ => None,
            };
            if let Some(fn_id) = mname.and_then(|m| self.struct_op_fn(sid, m)) {
                let r = self.coerce(r, &Ty::Struct(sid), rhs.span);
                let ret = self.ret_of(fn_id, span);
                return self.mk(
                    ExprKind::Call(fn_id, vec![Arg::Value(l), Arg::Value(r)]),
                    ret,
                    span,
                );
            }
        }
        let arith = matches!(op, Add | Sub | Mul | Div | Pow);
        let (l, r, ty) = match (&l.ty, &r.ty) {
            (Ty::Error, _) | (_, Ty::Error) => return self.err_expr(span),
            (a, b)
                if matches!(op, Add | Sub)
                    && (matches!(a, Ty::Opt(_)) || matches!(b, Ty::Opt(_)))
                    && tropical_side(a).is_some()
                    && tropical_side(b).is_some() =>
            {
                return self.tropical(op, l, r, span);
            }
            (Ty::Int, Ty::Int) => (l, r, Ty::Int),
            (Ty::Float, Ty::Float) if arith => (l, r, Ty::Float),
            (Ty::Float, Ty::Int) if arith => {
                let r = self.coerce(r, &Ty::Float, rhs.span);
                (l, r, Ty::Float)
            }
            (Ty::Int, Ty::Float) if arith => {
                let l = self.coerce(l, &Ty::Float, lhs.span);
                (l, r, Ty::Float)
            }
            (Ty::Bool, Ty::Bool) if matches!(op, BitAnd | BitOr | BitXor) => (l, r, Ty::Bool),
            (Ty::Str | Ty::Char, Ty::Str) | (Ty::Str, Ty::Char) if op == Add => {
                let l = self.as_str(l, lhs.span);
                let r = self.as_str(r, rhs.span);
                (l, r, Ty::Str)
            }
            (Ty::List(a), Ty::List(b)) if a == b && op == Add => {
                let list_ty = l.ty.clone();
                return self.do_expr(Some(&list_ty), span, |lw, out| {
                    let slot = lw.temp(list_ty.clone());
                    let cloned = lw.mk(ExprKind::Builtin(Builtin::Copy, vec![l]), list_ty.clone(), span);
                    out.push(Stmt::Let(slot, Some(cloned)));
                    let ext = lw.mk(
                        ExprKind::Mutate(
                            Mutation::Extend,
                            Box::new(Place {
                                kind: PlaceKind::Local(slot),
                                ty: list_ty.clone(),
                                span: lw.span(span),
                            }),
                            vec![r],
                        ),
                        Ty::Unit,
                        span,
                    );
                    out.push(Stmt::Expr(ext));
                    let res = lw.local(slot, span);
                    out.push(Stmt::Give(res));
                });
            }
            (Ty::Char, Ty::Char) if op == Sub => {
                let l = self.cast(Cast::CharToInt, l, Ty::Int);
                let r = self.cast(Cast::CharToInt, r, Ty::Int);
                (l, r, Ty::Int)
            }
            (Ty::Char, Ty::Int) if matches!(op, Add | Sub) => {
                let l = self.cast(Cast::CharToInt, l, Ty::Int);
                let sum = self.mk(
                    ExprKind::Binary(op, Box::new(l), Box::new(r)),
                    Ty::Int,
                    span,
                );
                return self.cast(Cast::IntToChar, sum, Ty::Char);
            }
            (Ty::Pos(a), Ty::Pos(b)) | (Ty::Bound(a), Ty::Bound(b)) if a == b && op == Sub => {
                let l = self.cast(Cast::Ord, l, Ty::Int);
                let r = self.cast(Cast::Ord, r, Ty::Int);
                (l, r, Ty::Int)
            }
            (Ty::Pos(d), Ty::Int) if matches!(op, Add | Sub) => {
                let d = *d;
                let step = if op == Sub {
                    self.mk(ExprKind::Unary(UnOp::Neg, Box::new(r)), Ty::Int, rhs.span)
                } else {
                    r
                };
                return self.mk(
                    ExprKind::Builtin(Builtin::PosOffset(d), vec![l, step]),
                    Ty::opt(Ty::Pos(d)),
                    span,
                );
            }
            _ => {
                let message = format!(
                    "`{}` can't be applied to `{}` and `{}`",
                    op.text(),
                    self.show(&l.ty),
                    self.show(&r.ty)
                );
                self.error(codes::BAD_OPERANDS, span, message);
                return self.err_expr(span);
            }
        };
        self.mk(ExprKind::Binary(op, Box::new(l), Box::new(r)), ty, span)
    }

    /// `a + b` and `a - b` where a side is a `?Int` or `?F64`: `none` on
    /// either side makes the whole `none`.
    fn tropical(&mut self, op: BinOp, l: Expr, r: Expr, span: Span) -> Expr {
        let float = tropical_side(&l.ty) == Some(Ty::Float) || tropical_side(&r.ty) == Some(Ty::Float);
        let num = if float { Ty::Float } else { Ty::Int };
        let ty = Ty::opt(num.clone());
        self.do_expr(Some(&ty), span, |lw, out| {
            let mut present: Option<Expr> = None;
            let mut sides = Vec::with_capacity(2);
            for side in [l, r] {
                let side_ty = side.ty.clone();
                let slot = lw.temp(side_ty.clone());
                out.push(Stmt::Let(slot, Some(side)));
                let held = lw.local(slot, span);
                let value = if let Ty::Opt(inner) = &side_ty {
                    let some = lw.mk(ExprKind::IsSome(Box::new(held.clone())), Ty::Bool, span);
                    present = Some(match present.take() {
                        Some(first) => lw.mk(
                            ExprKind::Binary(BinOp::And, Box::new(first), Box::new(some)),
                            Ty::Bool,
                            span,
                        ),
                        None => some,
                    });
                    lw.mk(ExprKind::Unwrap(Box::new(held), false), (**inner).clone(), span)
                } else {
                    held
                };
                sides.push(lw.coerce(value, &num, span));
            }
            let r = sides.pop().expect("two sides");
            let l = sides.pop().expect("two sides");
            let sum = lw.mk(ExprKind::Binary(op, Box::new(l), Box::new(r)), num.clone(), span);
            let some = lw.coerce(sum, &ty, span);
            let none = lw.mk(ExprKind::None, ty.clone(), span);
            let cond = present.expect("one side is optional");
            let value = lw.mk(
                ExprKind::If(Box::new(cond), Box::new(some), Box::new(none)),
                ty.clone(),
                span,
            );
            out.push(Stmt::Give(value));
        })
    }

    /// Whether `<` means something for the type.
    pub(super) fn ordered(&self, ty: &Ty) -> bool {
        match ty {
            Ty::Int | Ty::Float | Ty::Bool | Ty::Char | Ty::Str | Ty::Pos(_) | Ty::Bound(_) => true,
            Ty::Tuple(items) => items.iter().all(|t| self.ordered(t)),
            Ty::Record(fields) => fields.iter().all(|(_, t)| self.ordered(t)),
            Ty::List(elem) => self.ordered(elem),
            Ty::Struct(id) => {
                let fields: Vec<Ty> = self.structs[*id]
                    .def
                    .fields
                    .iter()
                    .map(|(_, t)| t.clone())
                    .collect();
                fields.iter().all(|t| self.ordered(t))
            }
            Ty::Error => true,
            _ => false,
        }
    }

    fn compare(&mut self, first: &ast::Expr, rest: &[(CmpOp, ast::Expr)], span: Span) -> Expr {
        if let [(op @ (CmpOp::In | CmpOp::NotIn), container)] = rest {
            let found = self.contains(first, container, span);
            return if *op == CmpOp::In {
                found
            } else {
                self.mk(ExprKind::Unary(UnOp::Not, Box::new(found)), Ty::Bool, span)
            };
        }
        // `none == x` needs the other side's type first.
        let mut operands: Vec<(Expr, Span)> = Vec::new();
        let asts: Vec<&ast::Expr> = std::iter::once(first)
            .chain(rest.iter().map(|(_, e)| e))
            .collect();
        for (i, e) in asts.iter().enumerate() {
            let value = if matches!(e.kind, ast::ExprKind::None) {
                match (operands.last(), asts.get(i + 1)) {
                    (Some((prev, _)), _) => {
                        let hint = Ty::opt(strip_opt(&prev.ty));
                        self.expr(e, Some(&hint))
                    }
                    (None, Some(next)) => {
                        let next = self.expr(next, None);
                        let hint = Ty::opt(strip_opt(&next.ty));
                        self.expr(e, Some(&hint))
                    }
                    _ => self.expr(e, None),
                }
            } else if matches!(&e.kind, ast::ExprKind::List(_)) {
                match (operands.last(), asts.get(i + 1)) {
                    (Some((prev, _)), _) if matches!(prev.ty, Ty::List(_)) => {
                        self.expr(e, Some(&prev.ty))
                    }
                    (Some((prev, _)), _)
                        if matches!(&prev.ty, Ty::Opt(inner) if matches!(**inner, Ty::List(_))) =>
                    {
                        let Ty::Opt(inner) = &prev.ty else {
                            unreachable!()
                        };
                        self.expr(e, Some(inner))
                    }
                    (None, Some(next)) if matches!(&e.kind, ast::ExprKind::List(items) if items.is_empty()) => {
                        let next = self.expr(next, None);
                        let hint = match &next.ty {
                            Ty::Opt(inner) if matches!(**inner, Ty::List(_)) => inner.as_ref(),
                            other => other,
                        };
                        self.expr(e, Some(hint))
                    }
                    _ => self.expr(e, None),
                }
            } else if matches!(
                &e.kind,
                ast::ExprKind::If { .. } | ast::ExprKind::Paren(_) | ast::ExprKind::Block(_)
            ) {
                match operands.last() {
                    Some((prev, _)) => self.expr(e, Some(&prev.ty)),
                    None => self.expr(e, None),
                }
            } else if operands.last().is_some_and(|(prev, _)| matches!(prev.ty, Ty::Opt(_))) {
                self.expr_maybe_opt_reduce(e)
            } else {
                self.expr(e, None)
            };
            operands.push((value, e.span));
        }
        if rest.len() == 1 {
            let (r, r_span) = operands.pop().expect("two operands");
            let (l, l_span) = operands.pop().expect("two operands");
            return self.cmp(rest[0].0, l, l_span, r, r_span, span);
        }
        // `lo <= x < hi`: each operand is evaluated once, left to right.
        let mut stmts = Vec::new();
        let held: Vec<(Expr, Span)> = operands
            .into_iter()
            .map(|(value, at)| {
                let atom = matches!(
                    value.kind,
                    ExprKind::Local(_)
                        | ExprKind::Global(_)
                        | ExprKind::Int(_)
                        | ExprKind::Char(_)
                        | ExprKind::Float(_)
                );
                if atom {
                    (value, at)
                } else {
                    (self.hold(value, &mut stmts), at)
                }
            })
            .collect();
        let mut all: Option<Expr> = None;
        for (i, (op, _)) in rest.iter().enumerate() {
            let (l, l_span) = held[i].clone();
            let (r, r_span) = held[i + 1].clone();
            let one = self.cmp(*op, l, l_span, r, r_span, span);
            all = Some(match all {
                None => one,
                Some(prev) => self.mk(
                    ExprKind::Binary(BinOp::And, Box::new(prev), Box::new(one)),
                    Ty::Bool,
                    span,
                ),
            });
        }
        let all = all.expect("a comparison has an operator");
        if stmts.is_empty() {
            return all;
        }
        stmts.push(Stmt::Give(all));
        self.mk(ExprKind::Do(Block { stmts }), Ty::Bool, span)
    }

    fn cmp(&mut self, op: CmpOp, l: Expr, l_span: Span, r: Expr, r_span: Span, span: Span) -> Expr {
        if l.ty == Ty::Error || r.ty == Ty::Error {
            return self.err_expr(span);
        }
        match op {
            CmpOp::In | CmpOp::NotIn => {
                self.unsupported(span, "`in` inside a chain of comparisons");
                return self.err_expr(span);
            }
            CmpOp::Order => {
                self.unsupported(span, "`<=>`");
                return self.err_expr(span);
            }
            _ => {}
        }
        // Bring both sides to one type.
        let (l, r) = if l.ty == r.ty {
            (l, r)
        } else {
            let want = match (&l.ty, &r.ty) {
                (Ty::Float, Ty::Int) | (Ty::Int, Ty::Float) => Ty::Float,
                (Ty::Opt(_), _) => l.ty.clone(),
                (_, Ty::Opt(_)) => r.ty.clone(),
                _ => l.ty.clone(),
            };
            let l = match self.try_coerce(l, &want) {
                Ok(l) => l,
                Err(l) => {
                    let message = format!(
                        "can't compare a `{}` with a `{}`",
                        self.show(&l.ty),
                        self.show(&r.ty)
                    );
                    let code = match (&l.ty, &r.ty) {
                        (Ty::Pos(_), Ty::Pos(_)) => codes::WRONG_DOMAIN,
                        _ => codes::TYPE_MISMATCH,
                    };
                    self.error(code, span, message);
                    return self.err_expr(span);
                }
            };
            let r = match self.try_coerce(r, &want) {
                Ok(r) => r,
                Err(r) => {
                    let message = format!(
                        "can't compare a `{}` with a `{}`",
                        self.show(&l.ty),
                        self.show(&r.ty)
                    );
                    let code = match (&l.ty, &r.ty) {
                        (Ty::Pos(_), Ty::Pos(_)) => codes::WRONG_DOMAIN,
                        _ => codes::TYPE_MISMATCH,
                    };
                    self.error(code, span, message);
                    return self.err_expr(span);
                }
            };
            (l, r)
        };
        let _ = (l_span, r_span);
        if !matches!(op, CmpOp::Eq | CmpOp::Ne) {
            if matches!(l.ty, Ty::Opt(_)) {
                self.error(
                    codes::OPTIONAL_COMPARE,
                    span,
                    "an optional has no order: `none` is neither small nor large",
                );
                self.help("give it a value first: `(x ?? INF) < y`");
                return self.err_expr(span);
            }
            if !self.ordered(&l.ty) {
                let message = format!("a `{}` has no order to compare by", self.show(&l.ty));
                self.error(codes::BAD_OPERANDS, span, message);
                return self.err_expr(span);
            }
        }
        self.mk(ExprKind::Cmp(op, Box::new(l), Box::new(r)), Ty::Bool, span)
    }

    /// `x in c`: a range, a domain, or anything that holds values.
    fn contains(&mut self, item: &ast::Expr, container: &ast::Expr, span: Span) -> Expr {
        let mut inner = container;
        while let ast::ExprKind::Paren(e) = &inner.kind {
            inner = e;
        }
        // `r in mask`, for a `Subset<Row>`.
        if let Some(Ty::Pos(subset)) = self.probe_ty_silent(inner) {
            if let Some(base) = self.subset_base(subset) {
                let mask = self.expr(inner, None);
                let item = self.expect(item, &Ty::Pos(base));
                return self.subset_has(mask, item, span);
            }
        }
        let between = |lw: &mut Self, value: Expr, lo: Expr, hi: Expr| {
            // `lo <= x and x < hi`, with `x` evaluated once.
            let mut stmts = Vec::new();
            let value = lw.hold(value, &mut stmts);
            let ge = lw.mk(
                ExprKind::Cmp(CmpOp::Le, Box::new(lo), Box::new(value.clone())),
                Ty::Bool,
                span,
            );
            let lt = lw.mk(
                ExprKind::Cmp(CmpOp::Lt, Box::new(value), Box::new(hi)),
                Ty::Bool,
                span,
            );
            let both = lw.mk(
                ExprKind::Binary(BinOp::And, Box::new(ge), Box::new(lt)),
                Ty::Bool,
                span,
            );
            if stmts.is_empty() {
                return both;
            }
            stmts.push(Stmt::Give(both));
            lw.mk(ExprKind::Do(Block { stmts }), Ty::Bool, span)
        };
        match &inner.kind {
            ast::ExprKind::Range {
                lo: Some(lo),
                hi: Some(hi),
                inclusive,
                step: None,
            } => {
                let value = self.expect(item, &Ty::Int);
                let lo = self.expect(lo, &Ty::Int);
                let mut hi = self.expect(hi, &Ty::Int);
                if *inclusive {
                    hi = self.int_add(hi, 1);
                }
                between(self, value, lo, hi)
            }
            ast::ExprKind::Name(name)
                if self.lookup(name).is_none()
                    && (matches!(self.gen_env.get(name), Some(GenArg::Domain(_)))
                        || matches!(self.get_sym(name), Some(Sym::Domain(_)))) =>
            {
                let id = match self.gen_env.get(name) {
                    Some(GenArg::Domain(id)) => *id,
                    _ => match self.get_sym(name) {
                        Some(Sym::Domain(id)) => id,
                        _ => unreachable!(),
                    },
                };
                let value = self.expect(item, &Ty::Int);
                let lo = self.mk(ExprKind::DomainLo(id), Ty::Int, span);
                let len = self.mk(ExprKind::DomainLen(id), Ty::Int, span);
                let hi = self.mk(
                    ExprKind::Binary(BinOp::Add, Box::new(lo.clone()), Box::new(len)),
                    Ty::Int,
                    span,
                );
                between(self, value, lo, hi)
            }
            _ => {
                let holder = self.expr(container, None);
                if let Ty::Struct(sid) = holder.ty {
                    let has_fn = self.structs[sid]
                        .methods
                        .get("has")
                        .or_else(|| self.structs[sid].methods.get("contains"))
                        .copied();
                    if let Some(fn_id) = has_fn {
                        let ast_arg = ast::Arg {
                            name: None,
                            mutable: false,
                            value: item.clone(),
                        };
                        return self.method_call(fn_id, Some((container, holder)), &[ast_arg], span);
                    }
                }
                let elem = match &holder.ty {
                    Ty::List(elem) | Ty::Set(elem) | Ty::HashSet(elem) | Ty::Range(elem) => {
                        (**elem).clone()
                    }
                    Ty::Array { elem, .. } => (**elem).clone(),
                    Ty::Map(key, _) | Ty::HashMap(key, _) => (**key).clone(),
                    Ty::Str => Ty::Char,
                    Ty::Error => return self.err_expr(span),
                    other => {
                        let message = format!("`in` can't look inside a `{}`", self.show(other));
                        self.error(codes::BAD_OPERANDS, span, message);
                        return self.err_expr(span);
                    }
                };
                let value = self.expect(item, &elem);
                self.mk(
                    ExprKind::Builtin(Builtin::Contains, vec![holder, value]),
                    Ty::Bool,
                    span,
                )
            }
        }
    }

    /// The indices of `base[...]` and the type of the element.
    pub(super) fn index_args(
        &mut self,
        base: &Ty,
        indices: &[ast::Expr],
        span: Span,
    ) -> (Vec<Expr>, Ty) {
        let one = |lw: &mut Self, want: &Ty| -> Vec<Expr> {
            if indices.len() != 1 {
                let message = format!("this takes 1 index, but {} were given", indices.len());
                lw.error(codes::WRONG_ARG_COUNT, span, message);
                return vec![lw.err_expr(span)];
            }
            vec![lw.expect(&indices[0], want)]
        };
        match base {
            Ty::Array { dims, elem } => {
                if dims.len() > 1 && indices.len() == 1 {
                    // Allow indexing a multi-D array with a single tuple/record coordinate `g[p]` where `p: (Row, Col)`.
                    let tuple_val = self.expr(&indices[0], None);
                    let field_tys: Option<Vec<Ty>> = match &tuple_val.ty {
                        Ty::Tuple(ts) if ts.len() == dims.len() => Some(ts.clone()),
                        Ty::Record(fs) if fs.len() == dims.len() => {
                            Some(fs.iter().map(|(_, t)| t.clone()).collect())
                        }
                        _ => None,
                    };
                    if let Some(ftys) = field_tys {
                        let mut out = Vec::new();
                        for (i, (dim, fty)) in dims.iter().zip(ftys).enumerate() {
                            let want = match dim {
                                Dim::Pos(d) => Ty::Pos(*d),
                                Dim::Bound(d) => Ty::Bound(*d),
                                Dim::Int => Ty::Int,
                            };
                            let field_e = self.mk(
                                ExprKind::Field(Box::new(tuple_val.clone()), i),
                                fty,
                                indices[0].span,
                            );
                            out.push(self.coerce(field_e, &want, indices[0].span));
                        }
                        return (out, (**elem).clone());
                    }
                }
                if dims.len() != indices.len() {
                    let message = format!(
                        "a `{}` takes {} indices, but {} were given",
                        self.show(base),
                        dims.len(),
                        indices.len()
                    );
                    self.error(codes::WRONG_ARG_COUNT, span, message);
                    if dims.len() > 1 && indices.len() == 1 {
                        self.help("a grid is indexed `g[r, c]`, never `g[r][c]`");
                    }
                    return (vec![self.err_expr(span)], (**elem).clone());
                }
                let mut out = Vec::new();
                for (dim, index) in dims.iter().zip(indices) {
                    let want = match dim {
                        Dim::Pos(d) => Ty::Pos(*d),
                        Dim::Bound(d) => Ty::Bound(*d),
                        Dim::Int => Ty::Int,
                    };
                    let value = self.expr(index, Some(&want));
                    if let (Dim::Pos(d), Ty::Struct(sid)) = (dim, &value.ty) {
                        if self.structs[*sid].index_domain == Some(*d) {
                            if let Some(to_idx) = self.structs[*sid].methods.get("to_idx").copied() {
                                let ord = self.mk(
                                    ExprKind::Call(to_idx, vec![Arg::Value(value)]),
                                    Ty::Int,
                                    index.span,
                                );
                                out.push(self.cast(Cast::Trust, ord, Ty::Pos(*d)));
                                continue;
                            }
                        }
                    }
                    if value.ty != want && value.ty != Ty::Error {
                        let message = format!(
                            "this is indexed by `{}`, but the index is a `{}`",
                            self.show(&want),
                            self.show(&value.ty)
                        );
                        self.error(codes::WRONG_DOMAIN, index.span, message);
                    }
                    out.push(value);
                }
                (out, (**elem).clone())
            }
            Ty::List(elem) => (one(self, &Ty::Int), (**elem).clone()),
            Ty::Str => (one(self, &Ty::Int), Ty::Char),
            Ty::Map(key, value) | Ty::HashMap(key, value) => (one(self, key), (**value).clone()),
            Ty::Error => {
                for index in indices {
                    self.expr(index, None);
                }
                (vec![self.err_expr(span)], Ty::Error)
            }
            other => {
                let message = format!("a `{}` can't be indexed", self.show(other));
                self.error(codes::NOT_INDEXABLE, span, message);
                (vec![self.err_expr(span)], Ty::Error)
            }
        }
    }

    /// Lowers a slice by a runtime `Range<T>` value (`base[r]`) into a `List<T>` (or `Str`).
    fn slice_by_range_val(&mut self, base: Expr, range_val: Expr, span: Span) -> Expr {
        let is_str = base.ty == Ty::Str;
        let (elem_ty, dim_dom) = match &base.ty {
            Ty::List(e) => ((**e).clone(), None),
            Ty::Str => (Ty::Char, None),
            Ty::Array { dims, elem } if dims.len() == 1 => {
                let d = match dims[0] {
                    Dim::Pos(id) => Some(id),
                    _ => None,
                };
                ((**elem).clone(), d)
            }
            _ => return self.err_expr(span),
        };
        let result_ty = if is_str {
            Ty::Str
        } else {
            Ty::list(elem_ty.clone())
        };
        let base_ty = base.ty.clone();
        let range_ty = range_val.ty.clone();
        let range_inner = match &range_ty {
            Ty::Range(inner) => (**inner).clone(),
            _ => Ty::Int,
        };
        self.do_expr(Some(&result_ty), span, |lw, out| {
            let b_slot = lw.temp(base_ty.clone());
            out.push(Stmt::Let(b_slot, Some(base)));
            let r_slot = lw.temp(range_ty.clone());
            out.push(Stmt::Let(r_slot, Some(range_val)));
            let acc_slot = lw.temp(result_ty.clone());
            let init = if is_str {
                lw.mk(ExprKind::Str(Vec::new()), Ty::Str, span)
            } else {
                lw.mk(ExprKind::Tuple(Vec::new()), result_ty.clone(), span)
            };
            out.push(Stmt::Let(acc_slot, Some(init)));
            let r_ref = lw.local(r_slot, span);
            let lo_bound = lw.mk(
                ExprKind::Builtin(Builtin::RangeLo, vec![r_ref.clone()]),
                range_inner.clone(),
                span,
            );
            let hi_bound = lw.mk(
                ExprKind::Builtin(Builtin::RangeHi, vec![r_ref]),
                range_inner.clone(),
                span,
            );
            let lo_ord = if matches!(range_inner, Ty::Pos(_) | Ty::Bound(_)) {
                lw.cast(Cast::Ord, lo_bound, Ty::Int)
            } else {
                lo_bound
            };
            let hi_ord = if matches!(range_inner, Ty::Pos(_) | Ty::Bound(_)) {
                lw.cast(Cast::Ord, hi_bound, Ty::Int)
            } else {
                hi_bound
            };
            let i_slot = lw.temp(Ty::Int);
            let i_ref = lw.local(i_slot, span);
            let idx_expr = match dim_dom {
                Some(d) => lw.mk(
                    ExprKind::Cast(Cast::Trust, Box::new(i_ref)),
                    Ty::Pos(d),
                    span,
                ),
                None => i_ref,
            };
            let elem = lw.mk(
                ExprKind::Index(Box::new(lw.local(b_slot, span)), vec![idx_expr]),
                elem_ty.clone(),
                span,
            );
            let push_stmt = if is_str {
                let item_str = lw.as_str(elem, span);
                let cur = lw.local(acc_slot, span);
                let joined = lw.mk(
                    ExprKind::Binary(BinOp::Add, Box::new(cur), Box::new(item_str)),
                    Ty::Str,
                    span,
                );
                Stmt::Assign(
                    Place {
                        kind: PlaceKind::Local(acc_slot),
                        ty: Ty::Str,
                        span: lw.span(span),
                    },
                    AssignOp::Set,
                    joined,
                )
            } else {
                Stmt::Expr(lw.mk(
                    ExprKind::Mutate(
                        Mutation::Push,
                        Box::new(Place {
                            kind: PlaceKind::Local(acc_slot),
                            ty: result_ty.clone(),
                            span: lw.span(span),
                        }),
                        vec![elem],
                    ),
                    Ty::Unit,
                    span,
                ))
            };
            let lid = lw.next_loop;
            lw.next_loop += 1;
            out.push(Stmt::For(
                lid,
                i_slot,
                Iter::Range {
                    lo: lo_ord,
                    hi: hi_ord,
                    step: None,
                    rev: false,
                },
                Block {
                    stmts: vec![push_stmt],
                },
            ));
            let res = lw.local(acc_slot, span);
            out.push(Stmt::Give(res));
        })
    }

    /// Lowers a slice expression `base[lo..hi]` / `base[lo..=hi]` / `base[..hi]` / `base[lo..]`
    /// on a `List<T>`, `Str`, or `[D]T` into a `List<T>` (or `Str` for `Str`).
    fn slice_expr(&mut self, base: Expr, range: &ast::Expr, span: Span) -> Expr {
        let ast::ExprKind::Range {
            lo,
            hi,
            inclusive,
            step,
        } = &range.kind
        else {
            return self.err_expr(span);
        };
        if step.is_some() {
            self.unsupported(span, "stepped slice");
            return self.err_expr(span);
        }
        let is_str = base.ty == Ty::Str;
        let (elem_ty, dim_dom) = match &base.ty {
            Ty::List(e) => ((**e).clone(), None),
            Ty::Str => (Ty::Char, None),
            Ty::Array { dims, elem } if dims.len() == 1 => {
                let d = match dims[0] {
                    Dim::Pos(d) => Some(d),
                    _ => None,
                };
                ((**elem).clone(), d)
            }
            Ty::Error => return self.err_expr(span),
            other => {
                self.error(
                    codes::BAD_OPERANDS,
                    span,
                    format!("can't slice a `{}`", self.show(other)),
                );
                return self.err_expr(span);
            }
        };
        let out_ty = if is_str { Ty::Str } else { Ty::list(elem_ty.clone()) };
        self.do_expr(Some(&out_ty), span, |lw, out| {
            let base_slot = lw.temp(base.ty.clone());
            out.push(Stmt::Let(base_slot, Some(base)));
            let base_ref = lw.local(base_slot, span);
            let len_expr = match dim_dom {
                Some(d) => lw.mk(ExprKind::DomainLen(d), Ty::Int, span),
                None => lw.mk(ExprKind::Builtin(Builtin::Len, vec![base_ref.clone()]), Ty::Int, span),
            };
            let to_idx = |lw: &mut Self, e: &ast::Expr, is_end: bool| -> Expr {
                let v = lw.expr(e, None);
                match v.ty {
                    Ty::Int => {
                        if is_end && *inclusive {
                            lw.int_add(v, 1)
                        } else {
                            v
                        }
                    }
                    Ty::Pos(_) => {
                        let ord = lw.cast(Cast::Ord, v, Ty::Int);
                        if is_end && *inclusive {
                            lw.int_add(ord, 1)
                        } else {
                            ord
                        }
                    }
                    Ty::Bound(_) => lw.cast(Cast::Ord, v, Ty::Int),
                    _ => lw.coerce(v, &Ty::Int, e.span),
                }
            };
            let lo_e = match lo {
                Some(l) => to_idx(lw, l, false),
                None => lw.int_lit(0, span),
            };
            let hi_e = match hi {
                Some(h) => to_idx(lw, h, true),
                None => len_expr,
            };
            let acc_slot = lw.temp(out_ty.clone());
            let empty_init = if is_str {
                lw.mk(ExprKind::Str(Vec::new()), Ty::Str, span)
            } else {
                lw.mk(ExprKind::Tuple(Vec::new()), out_ty.clone(), span)
            };
            out.push(Stmt::Let(acc_slot, Some(empty_init)));
            let idx_slot = lw.temp(Ty::Int);
            let idx_e = lw.local(idx_slot, span);
            let index_arg = match dim_dom {
                Some(d) => lw.mk(ExprKind::Cast(Cast::FromOrd(d), Box::new(idx_e)), Ty::Pos(d), span),
                None => idx_e,
            };
            let item = lw.mk(
                ExprKind::Index(Box::new(base_ref), vec![index_arg]),
                elem_ty.clone(),
                span,
            );
            let push_stmt = if is_str {
                let item_str = lw.as_str(item, span);
                let cur = lw.local(acc_slot, span);
                let joined = lw.mk(
                    ExprKind::Binary(BinOp::Add, Box::new(cur), Box::new(item_str)),
                    Ty::Str,
                    span,
                );
                Stmt::Assign(
                    Place {
                        kind: PlaceKind::Local(acc_slot),
                        ty: Ty::Str,
                        span: lw.span(span),
                    },
                    AssignOp::Set,
                    joined,
                )
            } else {
                Stmt::Expr(lw.mk(
                    ExprKind::Mutate(
                        Mutation::Push,
                        Box::new(Place {
                            kind: PlaceKind::Local(acc_slot),
                            ty: out_ty.clone(),
                            span: lw.span(span),
                        }),
                        vec![item],
                    ),
                    Ty::Unit,
                    span,
                ))
            };
            let lid = lw.next_loop;
            lw.next_loop += 1;
            out.push(Stmt::For(
                lid,
                idx_slot,
                Iter::Range {
                    lo: lo_e,
                    hi: hi_e,
                    step: None,
                    rev: false,
                },
                Block {
                    stmts: vec![push_stmt],
                },
            ));
            let res = lw.local(acc_slot, span);
            out.push(Stmt::Give(res));
        })
    }

    fn list(&mut self, items: &[ast::Expr], hint: Option<&Ty>, span: Span) -> Expr {
        match hint {
            Some(ty @ (Ty::List(elem) | Ty::Set(elem) | Ty::HashSet(elem))) => {
                let values = items.iter().map(|item| self.expect(item, elem)).collect();
                self.mk(ExprKind::Tuple(values), ty.clone(), span)
            }
            Some(ty @ (Ty::Map(..) | Ty::HashMap(..))) if items.is_empty() => {
                self.mk(ExprKind::Tuple(Vec::new()), ty.clone(), span)
            }
            _ => {
                let Some((first, rest)) = items.split_first() else {
                    self.error(
                        codes::TYPE_MISMATCH,
                        span,
                        "can't tell what this empty list holds",
                    );
                    self.help("say the type where it is stored: `var xs: List<Int> = []`");
                    return self.err_expr(span);
                };
                let has_none = items.iter().any(|it| matches!(it.kind, ast::ExprKind::None));
                if has_none {
                    let non_none = items.iter().find(|it| !matches!(it.kind, ast::ExprKind::None));
                    if let Some(sample) = non_none {
                        let sample_e = self.expr(sample, None);
                        let elem = match &sample_e.ty {
                            Ty::Opt(_) => sample_e.ty.clone(),
                            other => Ty::opt(other.clone()),
                        };
                        let values = items.iter().map(|item| self.expect(item, &elem)).collect();
                        return self.mk(ExprKind::Tuple(values), Ty::list(elem), span);
                    }
                }
                let first = self.expr(first, None);
                let elem = first.ty.clone();
                let mut values = vec![first];
                for item in rest {
                    values.push(self.expect(item, &elem));
                }
                self.mk(ExprKind::Tuple(values), Ty::list(elem), span)
            }
        }
    }

    /// What a `for` walks over, and the type of each value.
    pub(super) fn iter(&mut self, e: &ast::Expr) -> (Iter, Ty) {
        let span = e.span;
        let error = |lw: &mut Self| {
            let zero = lw.int_lit(0, span);
            (
                Iter::Range {
                    lo: zero.clone(),
                    hi: zero,
                    step: None,
                    rev: false,
                },
                Ty::Error,
            )
        };
        match &e.kind {
            ast::ExprKind::Paren(inner) => self.iter(inner),
            // `for mask in Subset<Row>:` walks every subset, and `for p in
            // ?Col:` walks `none` and then every `Col`.
            ast::ExprKind::Generic { base, args }
                if matches!(&base.kind, ast::ExprKind::Name(n) if n == "Subset" || n == "Opt")
                    && args.len() == 1 =>
            {
                let ast::ExprKind::Name(kind) = &base.kind else {
                    unreachable!()
                };
                let domain = match self.resolve_ty(&args[0]) {
                    Ty::Pos(domain) => domain,
                    Ty::Error => return error(self),
                    other => {
                        let message = format!(
                            "a `for` can walk the values of a domain's `?D` or `Subset<D>`, and `{}` is not a domain",
                            self.show(&other)
                        );
                        self.error(codes::TYPE_MISMATCH, span, message);
                        return error(self);
                    }
                };
                if kind == "Subset" {
                    let subset = self.subset_domain(domain);
                    let lo = self.int_lit(0, span);
                    let hi = self.mk(ExprKind::DomainLen(subset), Ty::Int, span);
                    return (
                        Iter::Range {
                            lo,
                            hi,
                            step: None,
                            rev: false,
                        },
                        Ty::Pos(subset),
                    );
                }
                let elem = Ty::opt(Ty::Pos(domain));
                let none = self.mk(ExprKind::None, elem.clone(), span);
                let some_ty = elem.clone();
                let all = self.listed(domain, elem.clone(), Some(none), span, |lw, ord| {
                    let pos = lw.cast(Cast::Trust, ord, Ty::Pos(domain));
                    (None, lw.mk(ExprKind::Some(Box::new(pos)), some_ty, span))
                });
                (Iter::Seq(all, false), elem)
            }
            ast::ExprKind::Range {
                lo: Some(lo),
                hi: Some(hi),
                inclusive,
                step,
            } => {
                let lo = self.expr(lo, None);
                let ty = lo.ty.clone();
                if !matches!(ty, Ty::Int | Ty::Pos(_) | Ty::Bound(_) | Ty::Error) {
                    let message = format!(
                        "a range runs over numbers or positions, not `{}`",
                        self.show(&ty)
                    );
                    self.error(codes::TYPE_MISMATCH, span, message);
                    return error(self);
                }
                let mut hi = self.expect(hi, &ty);
                let mut lo = lo;
                if ty != Ty::Int {
                    lo = self.cast(Cast::Ord, lo, Ty::Int);
                    hi = self.cast(Cast::Ord, hi, Ty::Int);
                }
                if *inclusive {
                    hi = self.int_add(hi, 1);
                }
                let step = step.as_ref().map(|s| self.expect(s, &Ty::Int));
                (
                    Iter::Range {
                        lo,
                        hi,
                        step,
                        rev: false,
                    },
                    ty,
                )
            }
            ast::ExprKind::Range {
                lo: Some(lo),
                hi: None,
                step,
                ..
            } => {
                let lo = self.expr(lo, None);
                match lo.ty {
                    Ty::Pos(d) | Ty::Bound(d) => {
                        let lo_ord = self.cast(Cast::Ord, lo, Ty::Int);
                        let hi = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
                        let step = step.as_ref().map(|s| self.expect(s, &Ty::Int));
                        (
                            Iter::Range {
                                lo: lo_ord,
                                hi,
                                step,
                                rev: false,
                            },
                            Ty::Pos(d),
                        )
                    }
                    _ => {
                        self.unsupported(span, "a range without an end in a `for`");
                        error(self)
                    }
                }
            }
            ast::ExprKind::Range {
                lo: None,
                hi: Some(hi),
                inclusive,
                step,
            } => {
                let hi = self.expr(hi, None);
                match hi.ty {
                    Ty::Pos(d) | Ty::Bound(d) => {
                        let lo_ord = self.int_lit(0, span);
                        let mut hi_ord = self.cast(Cast::Ord, hi, Ty::Int);
                        if *inclusive {
                            hi_ord = self.int_add(hi_ord, 1);
                        }
                        let step = step.as_ref().map(|s| self.expect(s, &Ty::Int));
                        (
                            Iter::Range {
                                lo: lo_ord,
                                hi: hi_ord,
                                step,
                                rev: false,
                            },
                            Ty::Pos(d),
                        )
                    }
                    _ => {
                        self.unsupported(span, "a range without a start in a `for`");
                        error(self)
                    }
                }
            }
            ast::ExprKind::Range { .. } => {
                self.unsupported(span, "a range without both ends in a `for`");
                error(self)
            }
            ast::ExprKind::Name(name)
                if self.lookup(name).is_none()
                    && (matches!(self.gen_env.get(name), Some(GenArg::Domain(_)))
                        || matches!(self.get_sym(name), Some(Sym::Domain(_)))) =>
            {
                let id = match self.gen_env.get(name) {
                    Some(GenArg::Domain(id)) => *id,
                    _ => match self.get_sym(name) {
                        Some(Sym::Domain(id)) => id,
                        _ => unreachable!(),
                    },
                };
                let lo = self.int_lit(0, span);
                let hi = self.mk(ExprKind::DomainLen(id), Ty::Int, span);
                (
                    Iter::Range {
                        lo,
                        hi,
                        step: None,
                        rev: false,
                    },
                    Ty::Pos(id),
                )
            }
            ast::ExprKind::Name(name)
                if self.lookup(name).is_none()
                    && matches!(self.get_sym(name), Some(Sym::Struct(sid)) if self.structs[sid].index_domain.is_some()) =>
            {
                let Some(Sym::Struct(sid)) = self.get_sym(name) else {
                    unreachable!()
                };
                let dom_id = self.structs[sid].index_domain.unwrap();
                let from_idx = self.structs[sid].methods.get("from_idx").copied();
                // Materialize `[S.from_idx(i) for i in 0..S.K]` as a sequence.
                if let Some(from_fn) = from_idx {
                    let list_ty = Ty::list(Ty::Struct(sid));
                    let seq = self.do_expr(Some(&list_ty), span, |lw, out| {
                        let acc = lw.temp(list_ty.clone());
                        out.push(Stmt::Let(
                            acc,
                            Some(lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span)),
                        ));
                        let idx_slot = lw.temp(Ty::Int);
                        let lid = lw.open_loop(None);
                        let call = lw.mk(
                            ExprKind::Call(from_fn, vec![Arg::Value(lw.local(idx_slot, span))]),
                            Ty::Struct(sid),
                            span,
                        );
                        let push = lw.mk(
                            ExprKind::Mutate(
                                Mutation::Push,
                                Box::new(Place {
                                    kind: PlaceKind::Local(acc),
                                    ty: list_ty.clone(),
                                    span: lw.span(span),
                                }),
                                vec![call],
                            ),
                            Ty::Unit,
                            span,
                        );
                        lw.cx.loops.pop();
                        out.push(Stmt::For(
                            lid,
                            idx_slot,
                            Iter::Range {
                                lo: lw.int_lit(0, span),
                                hi: lw.mk(ExprKind::DomainLen(dom_id), Ty::Int, span),
                                step: None,
                                rev: false,
                            },
                            Block {
                                stmts: vec![Stmt::Expr(push)],
                            },
                        ));
                        let res = lw.local(acc, span);
                        out.push(Stmt::Give(res));
                    });
                    return (Iter::Seq(seq, false), Ty::Struct(sid));
                }
                error(self)
            }
            ast::ExprKind::Field { base, name } if name.text == "rev" => {
                let (iter, ty) = self.iter(base);
                let iter = match iter {
                    Iter::Range { lo, hi, step, rev } => Iter::Range {
                        lo,
                        hi,
                        step,
                        rev: !rev,
                    },
                    Iter::RangeVal(value, rev) => Iter::RangeVal(value, !rev),
                    Iter::Seq(value, rev) => Iter::Seq(value, !rev),
                };
                (iter, ty)
            }
            ast::ExprKind::Field { base, name } if name.text == "bounds" => {
                let domain = match &base.kind {
                    ast::ExprKind::Name(n) if self.lookup(n).is_none() => {
                        if let Some(GenArg::Domain(id)) = self.gen_env.get(n) {
                            Some(*id)
                        } else if let Some(Sym::Domain(id)) = self.get_sym(n) {
                            Some(id)
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
                let Some(id) = domain else {
                    self.error(
                        codes::UNKNOWN_FIELD,
                        name.span,
                        "`.bounds` belongs to a domain: `V.bounds`",
                    );
                    return error(self);
                };
                let lo = self.int_lit(0, span);
                let len = self.mk(ExprKind::DomainLen(id), Ty::Int, span);
                let hi = self.int_add(len, 1);
                (
                    Iter::Range {
                        lo,
                        hi,
                        step: None,
                        rev: false,
                    },
                    Ty::Bound(id),
                )
            }
            _ => {
                if let ast::ExprKind::Field { base, name } = &e.kind {
                    if let ast::ExprKind::Generic {
                        base: gbase,
                        args: gargs,
                    } = &base.kind
                    {
                        if let Some(Sym::GenStruct(gid)) = self.sym(gbase) {
                            let resolved = self.resolve_gen_args(gargs);
                            let sid = self.instantiate_struct(gid, resolved);
                            if let Some(&dom_id) = self.structs[sid].owned_domains.get(&name.text) {
                                let lo = self.int_lit(0, span);
                                let hi = self.mk(ExprKind::DomainLen(dom_id), Ty::Int, span);
                                return (
                                    Iter::Range {
                                        lo,
                                        hi,
                                        step: None,
                                        rev: false,
                                    },
                                    Ty::Pos(dom_id),
                                );
                            }
                        }
                    }
                }
                let value = self.expr(e, None);
                if let ExprKind::DomainVal(dom_id) = value.kind {
                    let lo = self.int_lit(0, span);
                    let hi = self.mk(ExprKind::DomainLen(dom_id), Ty::Int, span);
                    return (
                        Iter::Range {
                            lo,
                            hi,
                            step: None,
                            rev: false,
                        },
                        Ty::Pos(dom_id),
                    );
                }
                if let Ty::Range(inner) = value.ty.clone() {
                    return (Iter::RangeVal(value, false), *inner);
                }
                // `for v in mask`: the members of a `Subset<D>`, in order.
                if let Ty::Pos(subset) = value.ty {
                    if let Some(base) = self.subset_base(subset) {
                        let mut stmts = Vec::new();
                        let mask = self.hold(value, &mut stmts);
                        let members = self.listed(base, Ty::Pos(base), None, span, |lw, ord| {
                            let pos = lw.cast(Cast::Trust, ord, Ty::Pos(base));
                            (Some(lw.subset_has(mask, pos.clone(), span)), pos)
                        });
                        stmts.push(Stmt::Give(members.clone()));
                        let all = self.mk(ExprKind::Do(Block { stmts }), members.ty, span);
                        return (Iter::Seq(all, false), Ty::Pos(base));
                    }
                }
                let elem = match &value.ty {
                    Ty::Array { elem, .. } | Ty::List(elem) | Ty::Set(elem) | Ty::HashSet(elem) => {
                        (**elem).clone()
                    }
                    Ty::Str => Ty::Char,
                    Ty::Map(key, val) | Ty::HashMap(key, val) => {
                        Ty::Tuple(vec![(**key).clone(), (**val).clone()])
                    }
                    Ty::Error => return error(self),
                    Ty::Int => {
                        self.error(
                            codes::TYPE_MISMATCH,
                            span,
                            "a `for` can't walk over an `Int`",
                        );
                        self.help("count with a range: `for i in 0..n`");
                        return error(self);
                    }
                    other => {
                        let message = format!("a `for` can't walk over a `{}`", self.show(other));
                        self.error(codes::TYPE_MISMATCH, span, message);
                        return error(self);
                    }
                };
                (Iter::Seq(value, false), elem)
            }
        }
    }

    /// Opens the loops of a comprehension. Its body is lowered next, with
    /// the loop variables in scope, and [`Self::close_loops`] wraps it.
    pub(super) fn open_loops(&mut self, clauses: &[ast::Clause]) -> Vec<LoopFrame> {
        let mut frames = Vec::new();
        for clause in clauses {
            let (iter, elem) = self.iter(&clause.iter);
            let id = self.next_loop;
            self.next_loop += 1;
            self.cx.scopes.push(HashMap::new());
            let mut pre = Vec::new();
            let slot = self.loop_var(&clause.pat, elem, &mut pre);
            let cond = match clause.conds.as_slice() {
                [] => None,
                [ast::Cond::Expr(c)] => Some(self.expect(c, &Ty::Bool)),
                [first, ..] => {
                    let span = match first {
                        ast::Cond::Expr(e) | ast::Cond::Const(e) => e.span,
                        ast::Cond::Let { value, .. } => value.span,
                    };
                    self.unsupported(
                        span,
                        "more than one `if`, or an `if let`, in a comprehension",
                    );
                    None
                }
            };
            frames.push(LoopFrame {
                id,
                slot,
                iter,
                pre,
                cond,
            });
        }
        frames
    }

    /// One loop over the values of `seq`, shaped like a comprehension's.
    pub(super) fn open_seq(&mut self, seq: Expr, elem: Ty) -> (Vec<LoopFrame>, Expr) {
        let id = self.next_loop;
        self.next_loop += 1;
        self.cx.scopes.push(HashMap::new());
        let span = seq.span;
        let slot = self.temp(elem.clone());
        let value = Expr {
            kind: ExprKind::Local(slot),
            ty: elem,
            span,
        };
        let frame = LoopFrame {
            id,
            slot,
            iter: Iter::Seq(seq, false),
            pre: Vec::new(),
            cond: None,
        };
        (vec![frame], value)
    }

    /// Wraps `inner` in the loops; gives the statements and the id of the
    /// outermost loop, for an early exit.
    pub(super) fn close_loops(
        &mut self,
        frames: Vec<LoopFrame>,
        inner: Vec<Stmt>,
    ) -> (Vec<Stmt>, LoopId) {
        let outer = frames.first().map_or(0, |f| f.id);
        let mut stmts = inner;
        for frame in frames.into_iter().rev() {
            self.cx.scopes.pop();
            let mut body = frame.pre;
            match frame.cond {
                Some(cond) => body.push(Stmt::If(cond, Block { stmts }, Block::default())),
                None => body.extend(stmts),
            }
            stmts = vec![Stmt::For(
                frame.id,
                frame.slot,
                frame.iter,
                Block { stmts: body },
            )];
        }
        (stmts, outer)
    }

    /// `[body for x in xs if cond]`: a list, or a set where one is wanted.
    fn comprehension(
        &mut self,
        body: &ast::Expr,
        clauses: &[ast::Clause],
        hint: Option<&Ty>,
        span: Span,
    ) -> Expr {
        let frames = self.open_loops(clauses);
        let (elem_hint, is_set) = match hint {
            Some(Ty::List(elem)) => (Some((**elem).clone()), false),
            Some(Ty::Set(elem)) => (Some((**elem).clone()), true),
            _ => (None, false),
        };
        let value = match &elem_hint {
            Some(elem) => self.expect(body, elem),
            None => self.expr(body, None),
        };
        let ty = if is_set {
            Ty::Set(Box::new(value.ty.clone()))
        } else {
            Ty::list(value.ty.clone())
        };
        let acc = self.temp(ty.clone());
        let at = self.span(span);
        let place = Place {
            kind: PlaceKind::Local(acc),
            ty: ty.clone(),
            span: at,
        };
        let op = if is_set {
            Mutation::Insert
        } else {
            Mutation::Push
        };
        let result_ty = if is_set { Ty::Bool } else { Ty::Unit };
        let add = Expr {
            kind: ExprKind::Mutate(op, Box::new(place), vec![value]),
            ty: result_ty,
            span: at,
        };
        let (loops, _) = self.close_loops(frames, vec![Stmt::Expr(add)]);
        let mut stmts = vec![Stmt::Let(
            acc,
            Some(self.mk(ExprKind::Tuple(Vec::new()), ty.clone(), span)),
        )];
        stmts.extend(loops);
        stmts.push(Stmt::Give(self.local(acc, span)));
        self.mk(ExprKind::Do(Block { stmts }), ty, span)
    }

    /// `[deg[u] * 2 for u in V]` where a `[V]T` is declared builds the
    /// array, like the table `[V] u => deg[u] * 2`.
    pub(super) fn comprehension_table(
        &mut self,
        init: &ast::Expr,
        dims: &[DimSpec],
        elem: &Ty,
    ) -> Option<Expr> {
        let ast::ExprKind::Comprehension { body, clauses } = &init.kind else {
            return None;
        };
        let ([clause], [DimSpec::Pos(dim)]) = (clauses.as_slice(), dims) else {
            return None;
        };
        let ast::ExprKind::Name(domain) = &clause.iter.kind else {
            return None;
        };
        if !clause.conds.is_empty() || self.lookup(domain).is_some() {
            return None;
        }
        let dom_matches = match self.gen_env.get(domain) {
            Some(GenArg::Domain(id)) => id == dim,
            _ => matches!(self.get_sym(domain), Some(Sym::Domain(id)) if id == *dim),
        };
        if !dom_matches {
            return None;
        }
        let pats = std::slice::from_ref(&clause.pat);
        Some(self.build_table(dims.to_vec(), pats, body, Some(elem), init.span))
    }

    fn table(
        &mut self,
        domains: &[ast::Type],
        params: &[ast::Pattern],
        body: &ast::Expr,
        hint: Option<&Ty>,
        span: Span,
    ) -> Expr {
        let dims: Option<Vec<DimSpec>> = domains.iter().map(|d| self.dim(d, true)).collect();
        let Some(dims) = dims else {
            return self.err_expr(span);
        };
        let elem = match hint {
            Some(Ty::Array { elem, .. }) => Some((**elem).clone()),
            _ => None,
        };
        self.build_table(dims, params, body, elem.as_ref(), span)
    }

    fn build_table(
        &mut self,
        dims: Vec<DimSpec>,
        params: &[ast::Pattern],
        body: &ast::Expr,
        elem: Option<&Ty>,
        span: Span,
    ) -> Expr {
        if params.len() != dims.len() {
            let message = format!(
                "this table runs over {} domains but names {} positions",
                dims.len(),
                params.len()
            );
            self.error(codes::WRONG_ARG_COUNT, span, message);
            return self.err_expr(span);
        }
        let at = self.span(span);
        let mut stmts = Vec::new();
        // Ensure any `impl Index` struct dimensions have their backing domain initialized (`let K = ...`).
        for dim in &dims {
            if let DimSpec::Pos(dom_id) = dim {
                if let Some((sid, sc)) = self.structs.iter().enumerate().find_map(|(sid, s)| {
                    if s.index_domain == Some(*dom_id) {
                        s.consts.get("K").cloned().map(|sc| (sid, sc))
                    } else {
                        None
                    }
                }) {
                    let k_val = self.eval_struct_const(sid, sc, span);
                    let lo = self.int_lit(0, span);
                    stmts.push(Stmt::DefDomain {
                        id: *dom_id,
                        lo,
                        len: k_val,
                    });
                }
            }
        }
        // A length is used to allocate and again to loop: hold it.
        let dims: Vec<DimSpec> = dims
            .into_iter()
            .map(|dim| match dim {
                DimSpec::Int(len) => DimSpec::Int(Box::new(self.hold(*len, &mut stmts))),
                other => other,
            })
            .collect();
        self.cx.scopes.push(HashMap::new());
        let mut loops = Vec::new();
        let mut indices = Vec::new();
        for (dim, param) in dims.iter().zip(params) {
            let (ty, hi) = match dim {
                DimSpec::Pos(d) => (Ty::Pos(*d), self.mk(ExprKind::DomainLen(*d), Ty::Int, span)),
                DimSpec::Bound(d) => {
                    let len = self.mk(ExprKind::DomainLen(*d), Ty::Int, span);
                    (Ty::Bound(*d), self.int_add(len, 1))
                }
                DimSpec::Int(len) => (Ty::Int, (**len).clone()),
            };
            let mut pre = Vec::new();
            let slot = self.loop_var(param, ty.clone(), &mut pre);
            let id = self.next_loop;
            self.next_loop += 1;
            let iter = Iter::Range {
                lo: self.int_lit(0, span),
                hi,
                step: None,
                rev: false,
            };
            indices.push(Expr {
                kind: ExprKind::Local(slot),
                ty,
                span: at,
            });
            loops.push((id, slot, iter, pre));
        }
        let value = match elem {
            Some(elem) => self.expect(body, elem),
            None => self.expr(body, None),
        };
        self.cx.scopes.pop();
        let ty = Ty::Array {
            dims: dims.iter().map(DimSpec::dim).collect(),
            elem: Box::new(value.ty.clone()),
        };
        let acc = self.temp(ty.clone());
        let cell = Place {
            kind: PlaceKind::Index(
                Box::new(Place {
                    kind: PlaceKind::Local(acc),
                    ty: ty.clone(),
                    span: at,
                }),
                indices,
            ),
            ty: value.ty.clone(),
            span: at,
        };
        let mut inner = vec![Stmt::Assign(cell, AssignOp::Set, value)];
        for (id, slot, iter, mut pre) in loops.into_iter().rev() {
            pre.extend(inner);
            inner = vec![Stmt::For(id, slot, iter, Block { stmts: pre })];
        }
        stmts.push(Stmt::Let(
            acc,
            Some(self.mk(ExprKind::Fill(dims, None), ty.clone(), span)),
        ));
        stmts.extend(inner);
        stmts.push(Stmt::Give(self.local(acc, span)));
        self.mk(ExprKind::Do(Block { stmts }), ty, span)
    }
}

fn strip_opt(ty: &Ty) -> Ty {
    match ty {
        Ty::Opt(inner) => (**inner).clone(),
        other => other.clone(),
    }
}

/// The number type of one side of a `none`-absorbing `+` or `-`: the type
/// itself, or what its optional holds.
fn tropical_side(ty: &Ty) -> Option<Ty> {
    let inner = match ty {
        Ty::Opt(inner) => &**inner,
        other => other,
    };
    matches!(inner, Ty::Int | Ty::Float).then(|| inner.clone())
}

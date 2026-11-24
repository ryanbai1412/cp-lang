//! `gen(lo..=hi)` and `gen<T>(...)`: a random value of a type the input
//! could hold (R06.4.3, lib/random-stress.md). Ranges draw with `rand`,
//! arrays draw each element, `Str of` calls the std fn beside it, and a
//! named type's `impl Random` draws the rest.

use super::*;

impl<'a> Lowerer<'a> {
    /// `gen()` with no type: a value of the type its value goes to, as a
    /// generic fn's return-only type parameter binds (`let x: 1..=9 = gen()`).
    pub(in crate::lower) fn gen_wanted(
        &mut self,
        args: &[ast::Arg],
        hint: Option<&Ty>,
        span: Span,
    ) -> Expr {
        let written = self.want_written.take().filter(|w| w.0 == span);
        if !args.is_empty() {
            return self.gen_call(None, args, span);
        }
        if let Some((_, ty, m, off, env)) = written {
            let saved = (
                std::mem::replace(&mut self.cur_mod, m),
                std::mem::replace(&mut self.offset, off),
                std::mem::replace(&mut self.gen_env, env),
            );
            let value = self.gen_call(Some(&ty), args, span);
            (self.cur_mod, self.offset, self.gen_env) = saved;
            return value;
        }
        match hint.filter(|t| !matches!(t, Ty::Unit | Ty::Error)) {
            Some(want) => self.gen_param(GenArg::Ty(want.clone()), args, span),
            None => self.gen_call(None, args, span),
        }
    }

    /// Notes `ty` as the type written where `value` goes, for a call there
    /// that takes its type from it.
    pub(in crate::lower) fn want_written(&mut self, at: Span, ty: &ast::Type) {
        self.want_written = Some((
            at,
            ty.clone(),
            self.cur_mod,
            self.offset,
            self.gen_env.clone(),
        ));
    }

    pub(in crate::lower) fn gen_call(
        &mut self,
        ty: Option<&ast::Type>,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let Some(ty) = ty else {
            return match args {
                [ast::Arg {
                    name: None,
                    value:
                        ast::Expr {
                            kind:
                                ast::ExprKind::Range {
                                    lo: Some(lo),
                                    hi: Some(hi),
                                    inclusive,
                                    step: None,
                                },
                            ..
                        },
                    ..
                }] => self.gen_range(lo, hi, *inclusive, span),
                _ => self.gen_error(cpl_diag::msgs::ir::GEN_TAKES.at(&[]), span),
            };
        };
        let named = |name: &str| {
            args.iter()
                .find(|a| a.name.as_ref().is_some_and(|n| n.text == name))
        };
        match &ty.kind {
            ast::TypeKind::Range {
                lo: Some(lo),
                hi: Some(hi),
                inclusive,
            } => self.gen_range(lo, hi, *inclusive, span),
            ast::TypeKind::Array { index, elem } => {
                let table = ast::Expr {
                    kind: ast::ExprKind::Table {
                        domains: index.clone(),
                        params: vec![
                            ast::Pattern {
                                kind: ast::PatternKind::Wildcard,
                                span: ty.span
                            };
                            index.len()
                        ],
                        body: Box::new(gen_of(elem, Vec::new())),
                    },
                    span,
                };
                self.expr(&table, None)
            }
            ast::TypeKind::ConstrainedStr { alphabet, len } => {
                let by_arg = match named("len").map(|a| &a.value.kind) {
                    Some(ast::ExprKind::Range {
                        lo: Some(lo),
                        hi: Some(hi),
                        inclusive,
                        ..
                    }) => Some((lo, hi, inclusive)),
                    _ => None,
                };
                let by_type = match len.as_deref().map(|t| &t.kind) {
                    Some(ast::TypeKind::Range {
                        lo: Some(lo),
                        hi: Some(hi),
                        inclusive,
                    }) => Some((lo, hi, inclusive)),
                    _ => None,
                };
                let Some((lo, hi, inclusive)) = by_arg.or(by_type) else {
                    return self.gen_error(cpl_diag::msgs::ir::GEN_STR_LEN.at(&[]), span);
                };
                let alphabet = alphabet
                    .clone()
                    .unwrap_or_else(|| "abcdefghijklmnopqrstuvwxyz".into());
                let hi = if *inclusive {
                    (**hi).clone()
                } else {
                    let one = ast::Expr {
                        kind: ast::ExprKind::Int(1),
                        span: hi.span,
                    };
                    ast::Expr {
                        kind: ast::ExprKind::Binary {
                            op: ast::BinOp::Sub,
                            op_span: hi.span,
                            lhs: hi.clone(),
                            rhs: Box::new(one),
                        },
                        span: hi.span,
                    }
                };
                let text = ast::Expr {
                    kind: ast::ExprKind::Str(vec![ast::StrPiece::Text(alphabet)]),
                    span: ty.span,
                };
                let call = [text, (**lo).clone(), hi].map(|value| ast::Arg {
                    name: None,
                    mutable: false,
                    value,
                });
                self.std_call("strings", "_gen_str", &[], &call, span)
            }
            ast::TypeKind::Named { args: tys, .. }
                if tys.is_empty()
                    && let Ty::Pos(d) = self.resolve_ty(ty) =>
            {
                self.gen_pos(d, span)
            }
            ast::TypeKind::Named { name, args: tys } => {
                match (name.text.as_str(), tys.as_slice()) {
                    (_, [])
                        if let Some(arg) =
                            (self.gen_env.get(&mono::range_ends_marker(&name.text)))
                                .or_else(|| self.gen_env.get(&name.text))
                                .cloned() =>
                    {
                        self.gen_param(arg, args, span)
                    }
                    _ => match ty.gen_call(args, span) {
                        Some(call) => self.expr(&call, None),
                        None => self.gen_unsupported(span),
                    },
                }
            }
            ast::TypeKind::Record(fields) => {
                let fields = fields
                    .iter()
                    .map(|(n, t)| (n.clone(), gen_of(t, Vec::new())))
                    .collect();
                let record = ast::Expr {
                    kind: ast::ExprKind::Record(fields),
                    span,
                };
                self.expr(&record, None)
            }
            ast::TypeKind::Tuple(items) if !items.is_empty() => {
                let tuple = ast::Expr {
                    kind: ast::ExprKind::Tuple(
                        items.iter().map(|t| gen_of(t, Vec::new())).collect(),
                    ),
                    span,
                };
                self.expr(&tuple, None)
            }
            _ => self.gen_unsupported(span),
        }
    }

    fn gen_range(&mut self, lo: &ast::Expr, hi: &ast::Expr, inclusive: bool, span: Span) -> Expr {
        let lo = self.expr_as(lo, &Ty::Int);
        let mut hi = self.expr_as(hi, &Ty::Int);
        if !inclusive {
            hi = self.int_add(hi, -1);
        }
        self.mk_builtin(Builtin::Rand, [lo, hi], Ty::Int, span)
    }

    /// A call of the fn `name` of the loaded module `std.<module>`.
    pub(in crate::lower) fn std_call(
        &mut self,
        module: &str,
        name: &str,
        tys: &[ast::Type],
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let file = format!("std/{module}.cp");
        let sym = self
            .path_to_mod
            .iter()
            .find(|(path, _)| path.replace('\\', "/").ends_with(&file))
            .and_then(|(_, &m)| self.mod_syms[m].get(name).copied());
        match sym {
            Some(Sym::GenFn(id)) => {
                let explicit = self.resolve_gen_args(tys);
                self.call_gen_fn(id, explicit, None, args, span)
            }
            Some(Sym::Fn(id)) => self.user_call(id, None, args, span),
            _ => self.gen_error(cpl_diag::msgs::ir::GEN_NEEDS_MODULE.at(&[&module]), span),
        }
    }

    /// `gen<A>()` for a type parameter `A`: what the argument it stands for draws.
    fn gen_param(&mut self, arg: GenArg, args: &[ast::Arg], span: Span) -> Expr {
        let (ty, items) = match arg {
            GenArg::Range(Some(lo), Some(hi)) => {
                let bounds = [self.int_lit(lo, span), self.int_lit(hi, span)];
                return self.mk_builtin(Builtin::Rand, bounds, Ty::Int, span);
            }
            GenArg::Ty(Ty::Pos(d)) | GenArg::Domain(d) => return self.gen_pos(d, span),
            GenArg::Ty(Ty::Struct(sid)) => {
                return match self.structs[sid].methods.get("gen") {
                    Some(&fn_id) => self.user_call(fn_id, None, args, span),
                    None => self.gen_unsupported(span),
                };
            }
            GenArg::Ty(ty)
                if let Some(fn_id) = (self.prim_methods.iter())
                    .find(|(t, m, _)| *t == ty && m == "gen")
                    .map(|p| p.2) =>
            {
                return self.user_call(fn_id, None, args, span);
            }
            GenArg::Drawn(ty, items) => (ty, items),
            GenArg::Ty(ty) if let Some(fields) = ty.fields() => {
                (ty, fields.into_iter().map(GenArg::Ty).collect())
            }
            _ => return self.gen_unsupported(span),
        };
        let mut parts = Vec::new();
        for a in items {
            parts.push(self.gen_param(a, &[], span));
        }
        self.mk(ExprKind::Tuple(parts), ty, span)
    }

    /// A position drawn uniformly: `V.nth(rand(0, V.count - 1))`.
    fn gen_pos(&mut self, d: DomainId, span: Span) -> Expr {
        let n = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
        let last = self.int_add(n, -1);
        let ord = self.mk_builtin(Builtin::Rand, [self.int_lit(0, span), last], Ty::Int, span);
        self.cast(Cast::Trust, ord, Ty::Pos(d))
    }

    fn gen_unsupported(&mut self, span: Span) -> Expr {
        self.gen_error(cpl_diag::msgs::ir::GEN_MAKES.at(&[]), span)
    }

    fn gen_error(&mut self, said: cpl_diag::msgs::Said, span: Span) -> Expr {
        self.report_said(span, said);
        self.err_expr(span)
    }
}

/// `domain V = 0..5`: a domain no input sizes, which a `@gen` fn or a
/// brute shares with the file as it shares a `const`.
pub(in crate::lower) fn fixed_domain(stmt: &ast::Stmt) -> bool {
    fn literal(e: &ast::Expr) -> bool {
        match &e.kind {
            ast::ExprKind::Int(_) => true,
            ast::ExprKind::Paren(inner) | ast::ExprKind::Unary { expr: inner, .. } => {
                literal(inner)
            }
            ast::ExprKind::Binary { lhs, rhs, .. } => literal(lhs) && literal(rhs),
            _ => false,
        }
    }
    matches!(&stmt.kind, ast::StmtKind::Domain { def: ast::Expr { kind: ast::ExprKind::Range { lo: Some(lo), hi: Some(hi), step: None, .. }, .. }, .. } if literal(lo) && literal(hi))
}

/// `gen<T>(args)`.
fn gen_of(ty: &ast::Type, args: Vec<ast::Arg>) -> ast::Expr {
    let span = ty.span;
    let name = ast::Expr {
        kind: ast::ExprKind::Name("gen".into()),
        span,
    };
    let callee = ast::Expr {
        kind: ast::ExprKind::Generic {
            base: Box::new(name),
            args: vec![ty.clone()],
        },
        span,
    };
    ast::Expr {
        kind: ast::ExprKind::Call {
            callee: Box::new(callee),
            args,
        },
        span,
    }
}

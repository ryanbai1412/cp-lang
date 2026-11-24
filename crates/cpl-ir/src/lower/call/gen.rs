//! `gen(lo..=hi)` and `gen<T>(...)`: a random value of a type the input
//! could hold (R06.4.3, lib/random-stress.md). Ranges draw with `rand`,
//! arrays draw each element, `Str of` and `Graph` call the std fns beside
//! each type, and a named type's `impl Random` draws the rest.

use super::*;

impl<'a> Lowerer<'a> {
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
                // A position drawn uniformly: `V.nth(rand(0, V.count - 1))`.
                let n = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
                let last = self.int_add(n, -1);
                let ord =
                    self.mk_builtin(Builtin::Rand, [self.int_lit(0, span), last], Ty::Int, span);
                self.cast(Cast::Trust, ord, Ty::Pos(d))
            }
            ast::TypeKind::Named { name, args: tys } => {
                match (name.text.as_str(), tys.as_slice()) {
                    ("Graph", [_, arc]) => {
                        let flag = |name: &str| ast::Arg {
                            name: Some(ast::Name {
                                text: name.into(),
                                span,
                            }),
                            mutable: false,
                            value: named(name).map_or(
                                ast::Expr {
                                    kind: ast::ExprKind::Bool(false),
                                    span,
                                },
                                |a| a.value.clone(),
                            ),
                        };
                        let (make, mirror) = arc_lambdas(arc, span);
                        let call = [
                            ast::Arg {
                                name: None,
                                mutable: false,
                                value: make,
                            },
                            ast::Arg {
                                name: None,
                                mutable: false,
                                value: mirror,
                            },
                            flag("simple"),
                            flag("connected"),
                            flag("directed"),
                        ];
                        self.std_call("graph", "gen_graph", tys, &call, span)
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
    fn std_call(
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

/// For an arc type `A`: `to => A` drawing every field but `to`, and
/// `(a, to) => A`, the same arc pointing back.
fn arc_lambdas(arc: &ast::Type, span: Span) -> (ast::Expr, ast::Expr) {
    let name = |text: &str| ast::Expr {
        kind: ast::ExprKind::Name(text.into()),
        span,
    };
    let lambda = |params: &[&str], body: ast::Expr| ast::Expr {
        kind: ast::ExprKind::Lambda {
            params: params
                .iter()
                .map(|p| ast::Pattern {
                    kind: ast::PatternKind::Name(p.to_string()),
                    span,
                })
                .collect(),
            param_tys: vec![None; params.len()],
            param_mut: Vec::new(),
            ret_ty: None,
            body: Box::new(body),
        },
        span,
    };
    let ast::TypeKind::Record(fields) = &arc.kind else {
        return (
            lambda(&["gen_to"], name("gen_to")),
            lambda(&["gen_arc", "gen_to"], name("gen_to")),
        );
    };
    let record = |value: &dyn Fn(&ast::Name, &ast::Type) -> ast::Expr| ast::Expr {
        kind: ast::ExprKind::Record(
            fields
                .iter()
                .map(|(field, ty)| {
                    let value = if field.text == "to" {
                        name("gen_to")
                    } else {
                        value(field, ty)
                    };
                    (field.clone(), value)
                })
                .collect(),
        ),
        span,
    };
    let drawn = record(&|_, ty| gen_of(ty, Vec::new()));
    let back = record(&|field, _| ast::Expr {
        kind: ast::ExprKind::Field {
            base: Box::new(name("gen_arc")),
            name: field.clone(),
        },
        span,
    });
    (
        lambda(&["gen_to"], drawn),
        lambda(&["gen_arc", "gen_to"], back),
    )
}

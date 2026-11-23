//! W8 `IndexSet`: on a struct whose only place protocol is `set`, `x[k] = v` is `x.set(k, v)`
//! and `x[k] op= v` is `x.set(k, x[k] op v)`, with `k` evaluated once (held in a
//! temporary when it is not a plain name).
//! Also `d[p, q]` on `[P, P]T`: product positions spelled out as their factor indices.

use super::call::is_place;
use super::*;

impl<'a> Lowerer<'a> {
    pub(super) fn index_set(
        &mut self,
        target: &ast::Expr,
        op: ast::AssignOp,
        value: &ast::Expr,
        out: &mut Vec<Stmt>,
    ) -> bool {
        let ast::ExprKind::Index { base, indices } = &target.kind else {
            return false;
        };
        let [key] = indices.as_slice() else {
            return false;
        };
        let Some(Ty::Struct(id)) = self.probe_ty_silent(base) else {
            return false;
        };
        let s = &self.structs[id];
        let has = |m: &str| s.methods.contains_key(m) || s.gen_methods.contains_key(m);
        if !has("set") || has("at_mut") {
            return false;
        }
        let span = target.span;
        let named = |f: &str, args: Vec<ast::Expr>| ast::Expr {
            kind: ast::ExprKind::Call {
                callee: Box::new(ast::Expr {
                    kind: ast::ExprKind::Name(f.into()),
                    span,
                }),
                args: args.into_iter().map(arg).collect(),
            },
            span,
        };
        // R01.6.4: `x[k] op= v` evaluates `k` once, so a key that is not a
        // plain name is held in a temporary the rewrite reads twice.
        let held = !matches!(op, ast::AssignOp::Set) && !pure(key);
        let (key, target) = if held {
            let value = self.expr(key, None);
            let slot = self.temp(value.ty.clone());
            out.push(Stmt::Let(slot, Some(value)));
            let mut scope = HashMap::new();
            scope.insert(HELD_KEY.to_string(), slot);
            self.cx.scopes.push(scope);
            let key = ast::Expr {
                kind: ast::ExprKind::Name(HELD_KEY.into()),
                span: key.span,
            };
            let target = ast::Expr {
                kind: ast::ExprKind::Index {
                    base: base.clone(),
                    indices: vec![key.clone()],
                },
                span,
            };
            (key, target)
        } else {
            (key.clone(), target.clone())
        };
        let target = &target;
        let stored = match op {
            ast::AssignOp::Set => value.clone(),
            ast::AssignOp::Binary(bin) => ast::Expr {
                kind: ast::ExprKind::Binary {
                    op: bin,
                    op_span: span,
                    lhs: Box::new(target.clone()),
                    rhs: Box::new(value.clone()),
                },
                span,
            },
            ast::AssignOp::Max => named("max", vec![target.clone(), value.clone()]),
            ast::AssignOp::Min => named("min", vec![target.clone(), value.clone()]),
        };
        let call = ast::Expr {
            kind: ast::ExprKind::Call {
                callee: Box::new(ast::Expr {
                    kind: ast::ExprKind::Field {
                        base: base.clone(),
                        name: ast::Name {
                            text: "set".into(),
                            span,
                        },
                    },
                    span,
                }),
                args: vec![arg(key), arg(stored)],
            },
            span,
        };
        let e = self.expr(&call, Some(&Ty::Unit));
        if held {
            self.cx.scopes.pop();
        }
        out.push(Stmt::Expr(e));
        true
    }
}

/// The name a held `set` key goes by; no source name can spell it.
const HELD_KEY: &str = "key held for set";

fn arg(value: ast::Expr) -> ast::Arg {
    ast::Arg {
        name: None,
        mutable: false,
        value,
    }
}

fn pure(e: &ast::Expr) -> bool {
    match &e.kind {
        ast::ExprKind::Name(_) | ast::ExprKind::Int(_) => true,
        ast::ExprKind::Field { base, .. } => pure(base),
        _ => false,
    }
}

impl<'a> Lowerer<'a> {
    /// `e` as a place; a value with no home (`SortedBag.of(xs).range(r)`) is held in a temporary
    /// that lives as long as the statements in `out`.
    pub(super) fn held_place(&mut self, e: &ast::Expr, out: &mut Vec<Stmt>) -> Place {
        if is_place(e) {
            return self.place(e);
        }
        let value = self.expr(e, None);
        let held = self.temp(value.ty.clone());
        let ty = value.ty.clone();
        out.push(Stmt::Let(held, Some(value)));
        Place {
            kind: PlaceKind::Local(held),
            ty,
            span: e.span,
        }
    }

    /// `g[p]` with `p: (Row, Col)`, or `d[p, q]` on `[P, P]T` with `type P = (A, B)`:
    /// each tuple, record or product position stands for its factor indices.
    pub(super) fn spread_indices(
        &mut self,
        dims: &[Dim],
        indices: &[ast::Expr],
    ) -> Option<Vec<Expr>> {
        if indices.len() >= dims.len() {
            return None;
        }
        let mut out = Vec::with_capacity(dims.len());
        for index in indices {
            let value = self.expr(index, None);
            if let Ty::Pos(p) = value.ty {
                if let Some(grid) = self.product_grid(p) {
                    if dims.get(out.len()..out.len() + grid.len()) != Some(grid.as_slice()) {
                        return None;
                    }
                    for i in 0..grid.len() {
                        out.push(self.decode_product_field(p, value.clone(), i, index.span));
                    }
                    continue;
                }
            }
            let parts: Vec<Ty> = match &value.ty {
                Ty::Tuple(ts) => ts.clone(),
                Ty::Record(fs) => fs.iter().map(|(_, t)| t.clone()).collect(),
                _ => return None,
            };
            for (i, part) in parts.into_iter().enumerate() {
                let dim = *dims.get(out.len())?;
                let want = match dim {
                    Dim::Pos(d) => Ty::Pos(d),
                    Dim::Bound(d) => Ty::Bound(d),
                    Dim::Enum(e) => Ty::Enum(e),
                    Dim::Sym(_) | Dim::Int | Dim::IntFrom(_) => Ty::Int,
                };
                let field = self.mk(
                    ExprKind::Field(Box::new(value.clone()), i),
                    part,
                    index.span,
                );
                let at = self.coerce(field, &want, index.span);
                out.push(match dim {
                    Dim::Enum(e) => {
                        let ord = self.cast(Cast::EnumOrd(e), at, Ty::Int);
                        self.cast(Cast::Raw, ord, Ty::Int)
                    }
                    _ => at,
                });
            }
        }
        (out.len() == dims.len()).then_some(out)
    }
}

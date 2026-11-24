//! `with io(base: b)` on output: counted positions shift wherever a
//! printed value holds them (R10.2.2).

use super::*;

impl<'a> Lowerer<'a> {
    /// `with io(base: b)`: a position of a counted domain prints as its
    /// ordinal plus `b`, inside tuples, records, options and sequences too
    /// (R10.2.2).
    pub(super) fn in_base(&mut self, value: Expr, span: Span) -> Expr {
        let base = self
            .io_base
            .filter(|&b| b != 0)
            .or(self.base_one().then_some(1));
        match base {
            Some(b) => self.shifted(value, b, span),
            None => value,
        }
    }

    /// The type `ty` prints as once its counted positions are shifted, or
    /// `None` when it holds none.
    fn shifted_ty(&self, ty: &Ty) -> Option<Ty> {
        let parts = |lw: &Self, ts: &[Ty]| {
            let shifted: Vec<_> = ts.iter().map(|t| lw.shifted_ty(t)).collect();
            shifted.iter().any(Option::is_some).then(|| {
                shifted
                    .into_iter()
                    .zip(ts)
                    .map(|(s, t)| s.unwrap_or_else(|| t.clone()))
                    .collect::<Vec<_>>()
            })
        };
        match ty {
            Ty::Pos(d) if self.counted_domains.contains(d) => Some(Ty::Int),
            Ty::Tuple(ts) => parts(self, ts).map(Ty::Tuple),
            Ty::Record(fs) => {
                let ts: Vec<Ty> = fs.iter().map(|(_, t)| t.clone()).collect();
                let shifted = parts(self, &ts)?;
                Some(Ty::Record(
                    fs.iter().map(|(n, _)| n.clone()).zip(shifted).collect(),
                ))
            }
            Ty::Opt(t) => self.shifted_ty(t).map(|t| Ty::Opt(Box::new(t))),
            Ty::List(t) | Ty::ListView(t, _) | Ty::Keyed(t, _) | Ty::MutView(t, _) => {
                self.shifted_ty(t).map(Ty::list)
            }
            Ty::Set(t) | Ty::HashSet(t) => self.shifted_ty(t).map(Ty::list),
            // A grid prints as its rows do: a list per axis.
            Ty::Array { dims, elem } => {
                let cell = self.shifted_ty(elem)?;
                Some(dims.iter().fold(cell, |t, _| Ty::list(t)))
            }
            Ty::Struct(id) if self.structs[*id].def.show_fn.is_none() => {
                let ts: Vec<Ty> = self.shown_fields(*id).into_iter().map(|(_, t)| t).collect();
                parts(self, &ts).map(Ty::Tuple)
            }
            _ => None,
        }
    }

    fn shifted(&mut self, value: Expr, b: i64, span: Span) -> Expr {
        let Some(to) = self.shifted_ty(&value.ty) else {
            return value;
        };
        match value.ty.clone() {
            Ty::Pos(_) => {
                let ord = self.cast(Cast::Ord, value, Ty::Int);
                self.int_add(ord, b)
            }
            from @ (Ty::Tuple(_) | Ty::Record(_) | Ty::Struct(_)) => {
                let fields: Vec<(usize, Ty)> = match from {
                    Ty::Tuple(ts) => ts.into_iter().enumerate().collect(),
                    Ty::Record(fs) => fs.into_iter().map(|(_, t)| t).enumerate().collect(),
                    Ty::Struct(id) => self.shown_fields(id),
                    _ => unreachable!(),
                };
                self.do_expr(Some(&to.clone()), span, |lw, out| {
                    let held = lw.hold(value, out);
                    let parts = fields
                        .into_iter()
                        .map(|(i, t)| {
                            let field = lw.mk(ExprKind::Field(Box::new(held.clone()), i), t, span);
                            lw.shifted(field, b, span)
                        })
                        .collect();
                    out.push(Stmt::Give(lw.mk(ExprKind::Tuple(parts), to, span)));
                })
            }
            Ty::Opt(inner) => self.do_expr(Some(&to.clone()), span, |lw, out| {
                let held = lw.hold(value, out);
                let some = lw.mk(ExprKind::IsSome(Box::new(held.clone())), Ty::Bool, span);
                let inside = lw.mk(ExprKind::Unwrap(Box::new(held), false), *inner, span);
                let inside = lw.shifted(inside, b, span);
                let some_val = lw.mk(ExprKind::Some(Box::new(inside)), to.clone(), span);
                let none = lw.mk(ExprKind::None, to.clone(), span);
                out.push(Stmt::Give(lw.mk(
                    ExprKind::If(Box::new(some), Box::new(some_val), Box::new(none)),
                    to,
                    span,
                )));
            }),
            Ty::Array { dims, elem } if dims.len() > 1 => {
                let cell = self.shifted_ty(&elem).expect("a cell that shifts");
                self.do_expr(Some(&to.clone()), span, |lw, out| {
                    let held = lw.hold(value, out);
                    let lens: Vec<Expr> = (0..dims.len())
                        .map(|axis| {
                            let at = lw.int_lit(axis as i64, span);
                            let args = vec![held.clone(), at];
                            let len =
                                lw.mk(ExprKind::Builtin(Builtin::AxisLen, args), Ty::Int, span);
                            let slot = lw.temp(Ty::Int);
                            out.push(Stmt::Let(slot, Some(len)));
                            lw.local(slot, span)
                        })
                        .collect();
                    let flat = lw.shifted_cells(held, *elem, cell.clone(), b, span);
                    let flat = lw.hold(flat, out);
                    let zero = lw.int_lit(0, span);
                    let rows = lw.nest_rows(&flat, &lens, &cell, zero, span);
                    out.push(Stmt::Give(rows));
                })
            }
            from => {
                let (Ty::List(elem)
                | Ty::ListView(elem, _)
                | Ty::Keyed(elem, _)
                | Ty::MutView(elem, _)
                | Ty::Set(elem)
                | Ty::HashSet(elem)
                | Ty::Array { elem, .. }) = from
                else {
                    return value;
                };
                let Ty::List(to_elem) = to else { return value };
                self.shifted_cells(value, *elem, *to_elem, b, span)
            }
        }
    }

    /// The fields a struct without `impl Show` prints, by position.
    fn shown_fields(&self, id: StructId) -> Vec<(usize, Ty)> {
        let def = &self.structs[id].def;
        (def.fields.iter().enumerate())
            .filter(|(i, _)| !def.cached_fields.contains(i))
            .map(|(i, (_, t))| (i, t.clone()))
            .collect()
    }

    /// The list of `seq`'s items in order, each shifted (`in_base`).
    fn shifted_cells(&mut self, seq: Expr, elem: Ty, to: Ty, b: i64, span: Span) -> Expr {
        self.collected(to, span, |lw, out, push| {
            let item = lw.temp(elem);
            let lid = lw.fresh_loop();
            let shown = lw.local(item, span);
            let shown = lw.shifted(shown, b, span);
            let body = Block {
                stmts: vec![push(lw, shown)],
            };
            out.push(Stmt::For(lid, item, Iter::Seq(seq, false), body));
        })
    }

    /// The row-major `flat` cells of a grid with axes `lens`, as a list per
    /// axis from the one at `idx` on: a grid prints as its rows do.
    fn nest_rows(&mut self, flat: &Expr, lens: &[Expr], cell: &Ty, idx: Expr, span: Span) -> Expr {
        let Some((len, rest)) = lens.split_first() else {
            return self.mk(
                ExprKind::Index(Box::new(flat.clone()), vec![idx]),
                cell.clone(),
                span,
            );
        };
        let inner = rest.iter().fold(cell.clone(), |t, _| Ty::list(t));
        self.collected(inner, span, |lw, out, push| {
            let k = lw.temp(Ty::Int);
            let lid = lw.fresh_loop();
            let scaled = lw.mk(
                ExprKind::Binary(BinOp::Mul, Box::new(idx), Box::new(len.clone())),
                Ty::Int,
                span,
            );
            let at = lw.local(k, span);
            let next = lw.mk(
                ExprKind::Binary(BinOp::Add, Box::new(scaled), Box::new(at)),
                Ty::Int,
                span,
            );
            let row = lw.nest_rows(flat, rest, cell, next, span);
            let body = Block {
                stmts: vec![push(lw, row)],
            };
            let iter = Iter::Range {
                lo: lw.int_lit(0, span),
                hi: len.clone(),
                step: None,
                rev: false,
                closed: false,
            };
            out.push(Stmt::For(lid, k, iter, body));
        })
    }
}

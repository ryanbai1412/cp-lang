//! `xs!` on `[D]?T` or `List<?T>` (R02.4.8): every cell checked in one pass,
//! giving `[D]T` or `List<T>`.

use super::super::*;

impl<'a> Lowerer<'a> {
    pub(in crate::lower) fn unwraps_cells(ty: &Ty) -> bool {
        match ty {
            Ty::Array { dims, elem } => {
                matches!(dims[..], [Dim::Pos(_) | Dim::Int | Dim::IntFrom(_)])
                    && matches!(**elem, Ty::Opt(_))
            }
            Ty::List(elem) => matches!(**elem, Ty::Opt(_)),
            _ => false,
        }
    }

    pub(in crate::lower) fn unwrap_cells(&mut self, value: Expr, span: Span) -> Expr {
        let (dim, cell, list) = match &value.ty {
            Ty::Array { dims, elem } => (dims[0], (**elem).clone(), false),
            Ty::List(elem) => (Dim::Int, (**elem).clone(), true),
            _ => unreachable!("checked by unwraps_cells"),
        };
        let Ty::Opt(inside) = cell.clone() else {
            unreachable!("checked by unwraps_cells")
        };
        let inside = *inside;
        let at = self.span(span);
        let mut stmts = Vec::new();
        let value = if list {
            let ty = Ty::Array {
                dims: vec![Dim::Int],
                elem: Box::new(cell.clone()),
            };
            self.mk(
                ExprKind::Builtin(Builtin::ListToArray, vec![value]),
                ty,
                span,
            )
        } else {
            value
        };
        let src = self.hold(value, &mut stmts);
        let (spec, index_ty, lo, hi) = match dim {
            Dim::Pos(d) => {
                let hi = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
                (DimSpec::Pos(d), Ty::Pos(d), self.int_lit(0, span), hi)
            }
            _ => {
                let first = if let Dim::IntFrom(first) = dim {
                    first
                } else {
                    0
                };
                let len = self.mk(
                    ExprKind::Builtin(Builtin::Len, vec![src.clone()]),
                    Ty::Int,
                    span,
                );
                let len = self.hold(len, &mut stmts);
                let spec = match first {
                    0 => DimSpec::Int(Box::new(len.clone())),
                    _ => DimSpec::IntFrom(first, Box::new(len.clone())),
                };
                // The cells are stored from 0 whatever the first key: index them so.
                (spec, Ty::Int, self.int_lit(0, span), len)
            }
        };
        let ty = Ty::Array {
            dims: vec![dim],
            elem: Box::new(inside.clone()),
        };
        let acc = self.temp(ty.clone());
        let slot = self.temp(index_ty);
        let id = self.next_loop;
        self.next_loop += 1;
        let i = self.local(slot, span);
        let cell = self.mk(ExprKind::Index(Box::new(src), vec![i.clone()]), cell, span);
        let got = self.mk(ExprKind::Unwrap(Box::new(cell), true), inside.clone(), span);
        let place = Place {
            kind: PlaceKind::Index(
                Box::new(Place {
                    kind: PlaceKind::Local(acc),
                    ty: ty.clone(),
                    span: at,
                }),
                vec![i],
            ),
            ty: inside.clone(),
            span: at,
        };
        let iter = Iter::Range {
            lo,
            hi,
            step: None,
            rev: false,
            closed: false,
        };
        stmts.push(Stmt::Let(
            acc,
            Some(self.mk(ExprKind::Fill(vec![spec], None), ty.clone(), span)),
        ));
        stmts.push(Stmt::For(
            id,
            slot,
            iter,
            Block {
                stmts: vec![Stmt::Assign(place, AssignOp::Set, got)],
            },
        ));
        stmts.push(Stmt::Give(self.local(acc, span)));
        let cells = self.mk(ExprKind::Do(Block { stmts }), ty, span);
        if !list {
            return cells;
        }
        let ty = Ty::List(Box::new(inside));
        self.mk(
            ExprKind::Builtin(Builtin::ArrayToList, vec![cells]),
            ty,
            span,
        )
    }
}

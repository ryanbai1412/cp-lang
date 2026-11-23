//! Input formats for `read()` (R10.2): which types have one, and
//! `read<List<T>>()`, a count then that many `T`s, so `[D]List<T>` reads
//! `D`-many count-prefixed rows (R10.2.3).

use super::*;

impl<'a> Lowerer<'a> {
    /// A count, then that many `each`, as a `List`.
    pub(super) fn counted_list(&mut self, each: Spec) -> Spec {
        let span = Span::default();
        let list = Ty::list(each.ty());
        let call = self.do_expr(Some(&list), span, |lw, out| {
            let place = |lw: &Self, slot, ty: &Ty| Place {
                kind: PlaceKind::Local(slot),
                ty: ty.clone(),
                span: lw.span(span),
            };
            let left = lw.temp(Ty::Int);
            let count = lw.mk(
                ExprKind::Read(Box::new(Spec::Plain(Ty::Int))),
                Ty::Int,
                span,
            );
            out.push(Stmt::Let(left, Some(count)));
            let acc = lw.temp(list.clone());
            let empty = lw.mk(ExprKind::Tuple(Vec::new()), list.clone(), span);
            out.push(Stmt::Let(acc, Some(empty)));
            let lid = lw.open_loop(None);
            let item = lw.read(each, span);
            let push =
                ExprKind::Mutate(Mutation::Push, Box::new(place(lw, acc, &list)), vec![item]);
            let one = lw.int_lit(1, span);
            let body = vec![
                Stmt::Expr(lw.mk(push, Ty::Unit, span)),
                Stmt::Assign(place(lw, left, &Ty::Int), AssignOp::Binary(BinOp::Sub), one),
            ];
            lw.cx.loops.pop();
            let (rest, zero) = (lw.local(left, span), lw.int_lit(0, span));
            let more = lw.mk(
                ExprKind::Cmp(CmpOp::Gt, Box::new(rest), Box::new(zero)),
                Ty::Bool,
                span,
            );
            out.push(Stmt::While(lid, more, Block { stmts: body }));
            out.push(Stmt::Give(lw.local(acc, span)));
        });
        Spec::Call(Box::new(call))
    }
}

/// The type inside `spec` that has no input format, if any.
pub(super) fn unreadable(spec: &Spec) -> Option<Ty> {
    match spec {
        Spec::Plain(ty) => match ty {
            Ty::Int
            | Ty::I128
            | Ty::U64
            | Ty::Float
            | Ty::Char
            | Ty::Str
            | Ty::Bool
            | Ty::Pos(_)
            | Ty::Error => None,
            Ty::Range(inner) if matches!(**inner, Ty::Int | Ty::Pos(_)) => None,
            other => Some(other.clone()),
        },
        Spec::Int { .. } | Spec::Call(_) | Spec::Line => None,
        Spec::Parse(_, inner) => unreadable(inner),
        Spec::Fields(_, fields) => fields.iter().find_map(unreadable),
        Spec::Array { elem, .. } | Spec::List(elem) => unreadable(elem),
        Spec::Opt(inner, absent) => match (&**inner, absent) {
            (Spec::Plain(Ty::Int | Ty::Pos(_)) | Spec::Int { .. }, Some(_)) => None,
            _ => Some(spec.ty()),
        },
    }
}

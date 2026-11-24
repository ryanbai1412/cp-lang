//! The members every finite space of positions has by ordinal, one table for
//! a domain, a fieldless enum and `Char` (R04.2): `try_at`, `nth`, `wrap`.

use super::*;

/// A space whose positions are numbered `0..count`.
#[derive(Clone, Copy)]
pub(super) enum OrdSpace {
    Domain(DomainId),
    Enum(EnumId),
    Char,
}

impl Lowerer<'_> {
    fn space_ty(&self, s: OrdSpace) -> Ty {
        match s {
            OrdSpace::Domain(d) => Ty::Pos(d),
            OrdSpace::Enum(e) => Ty::Enum(e),
            OrdSpace::Char => Ty::Char,
        }
    }

    fn space_count(&mut self, s: OrdSpace, span: Span) -> Expr {
        match s {
            OrdSpace::Domain(d) => self.mk(ExprKind::DomainLen(d), Ty::Int, span),
            OrdSpace::Enum(e) => self.int_lit(self.enums[e].variants.len() as i64, span),
            OrdSpace::Char => self.int_lit(128, span),
        }
    }

    fn space_at_ord(&self, s: OrdSpace, ord: Expr) -> Expr {
        let cast = match s {
            OrdSpace::Domain(d) => Cast::FromOrd(d),
            OrdSpace::Enum(e) => Cast::OrdToEnum(e),
            OrdSpace::Char => Cast::IntToChar,
        };
        self.cast(cast, ord, self.space_ty(s))
    }

    /// `S.try_at(i)`, `S.nth(i)` or `S.wrap(i)`; `None` for any other member.
    pub(super) fn space_member(
        &mut self,
        s: OrdSpace,
        name: &str,
        args: &[ast::Arg],
        span: Span,
    ) -> Option<Expr> {
        if !matches!(name, "try_at" | "nth" | "wrap") {
            return None;
        }
        let Some(i) = self.one_arg(name, args, &Ty::Int, span) else {
            return Some(self.err_expr(span));
        };
        Some(match (name, s) {
            ("nth", _) => self.space_at_ord(s, i),
            ("wrap", OrdSpace::Domain(d)) => self.domain_wrap(d, i, span),
            ("wrap", _) => self.space_wrap(s, i, span),
            (_, OrdSpace::Domain(d)) => self.mk(
                ExprKind::Builtin(Builtin::PosTryOrd(d), vec![i]),
                Ty::opt(Ty::Pos(d)),
                span,
            ),
            _ => self.space_try_at(s, i, span),
        })
    }

    /// `S.wrap(i)`: the position `i` lands on, counted around the space.
    fn space_wrap(&mut self, s: OrdSpace, i: Expr, span: Span) -> Expr {
        let rem = |lw: &mut Self, e: Expr| {
            let n = lw.space_count(s, span);
            lw.mk(
                ExprKind::Binary(BinOp::Rem, Box::new(e), Box::new(n)),
                Ty::Int,
                span,
            )
        };
        let once = rem(self, i);
        let n = self.space_count(s, span);
        let up = self.mk(
            ExprKind::Binary(BinOp::Add, Box::new(once), Box::new(n)),
            Ty::Int,
            span,
        );
        let ord = rem(self, up);
        self.space_at_ord(s, ord)
    }

    /// `S.try_at(i)`: the `i`th position, `None` outside `0..S.count`.
    fn space_try_at(&mut self, s: OrdSpace, value: Expr, span: Span) -> Expr {
        let mut stmts = Vec::new();
        let i = self.hold(value, &mut stmts);
        let zero = self.int_lit(0, span);
        let n = self.space_count(s, span);
        let ge0 = self.mk(
            ExprKind::Cmp(CmpOp::Ge, Box::new(i.clone()), Box::new(zero)),
            Ty::Bool,
            span,
        );
        let lt_n = self.mk(
            ExprKind::Cmp(CmpOp::Lt, Box::new(i.clone()), Box::new(n)),
            Ty::Bool,
            span,
        );
        let ok = self.mk(
            ExprKind::Binary(BinOp::And, Box::new(ge0), Box::new(lt_n)),
            Ty::Bool,
            span,
        );
        let ty = Ty::opt(self.space_ty(s));
        let at = self.space_at_ord(s, i);
        let some = self.mk(ExprKind::Some(Box::new(at)), ty.clone(), span);
        let none = self.mk(ExprKind::None, ty.clone(), span);
        let pick = self.mk(
            ExprKind::If(Box::new(ok), Box::new(some), Box::new(none)),
            ty.clone(),
            span,
        );
        stmts.push(Stmt::Give(pick));
        self.mk(ExprKind::Do(Block { stmts }), ty, span)
    }

    /// `V.wrap(x)`: the position labelled `x` modulo `V.count` (R04.2.13).
    pub(super) fn domain_wrap(&mut self, id: DomainId, label: Expr, span: Span) -> Expr {
        let lo = self.domain_first_label(id, span);
        let off = self.mk(
            ExprKind::Binary(BinOp::Sub, Box::new(label), Box::new(lo)),
            Ty::Int,
            span,
        );
        let rem = |lw: &mut Self, e: Expr| {
            let len = lw.mk(ExprKind::DomainLen(id), Ty::Int, span);
            lw.mk(
                ExprKind::Binary(BinOp::Rem, Box::new(e), Box::new(len)),
                Ty::Int,
                span,
            )
        };
        let once = rem(self, off);
        let len = self.mk(ExprKind::DomainLen(id), Ty::Int, span);
        let shifted = self.mk(
            ExprKind::Binary(BinOp::Add, Box::new(once), Box::new(len)),
            Ty::Int,
            span,
        );
        let ord = rem(self, shifted);
        self.mk(
            ExprKind::Cast(Cast::FromOrd(id), Box::new(ord)),
            Ty::Pos(id),
            span,
        )
    }
}

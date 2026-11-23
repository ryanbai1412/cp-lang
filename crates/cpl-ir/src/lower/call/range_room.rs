//! How far a cut of a range can reach, without overflow.

use super::*;

impl<'a> Lowerer<'a> {
    /// How many values `lo..hi` holds, as far as a cut of it needs: past
    /// `Int.last` the span wraps, and then every cut fits. (R04, open-ended
    /// ranges: `-3..` builds, and `.take(3)` of it is `-3..0`.)
    pub(in crate::lower) fn range_room(&mut self, lo: Slot, hi: Slot, span: Span) -> Expr {
        let (l, h) = (self.local(lo, span), self.local(hi, span));
        let span_w = self.mk(
            ExprKind::Builtin(Builtin::WrapSub, vec![h, l]),
            Ty::Int,
            span,
        );
        let zero = self.int_lit(0, span);
        let wrapped = self.mk(
            ExprKind::Cmp(CmpOp::Lt, Box::new(span_w.clone()), Box::new(zero)),
            Ty::Bool,
            span,
        );
        let most = self.int_lit(i64::MAX, span);
        let room = self.mk(
            ExprKind::If(Box::new(wrapped), Box::new(most), Box::new(span_w)),
            Ty::Int,
            span,
        );
        let (l, h) = (self.local(lo, span), self.local(hi, span));
        let empty = self.mk(
            ExprKind::Cmp(CmpOp::Ge, Box::new(l), Box::new(h)),
            Ty::Bool,
            span,
        );
        let zero = self.int_lit(0, span);
        self.mk(
            ExprKind::If(Box::new(empty), Box::new(zero), Box::new(room)),
            Ty::Int,
            span,
        )
    }
}

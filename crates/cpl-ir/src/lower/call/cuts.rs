//! `s.cut_after(i)`, `s.cut_before(i)`: the cut beside a key in walk order
//! (R07.4.14), so a generic body never hard-codes `.l`/`.r` for a `.rev`.

use super::*;

impl<'a> Lowerer<'a> {
    /// A sequence whose keys have cuts between them.
    pub(in crate::lower) fn has_cuts(ty: &Ty) -> bool {
        matches!(
            ty,
            Ty::List(_) | Ty::Str | Ty::StrView | Ty::ListView(..) | Ty::MutView(..)
        ) || matches!(ty, Ty::Array { dims, .. } if dims.len() == 1)
    }

    pub(in crate::lower) fn cut_beside(
        &mut self,
        value: Expr,
        ty: Ty,
        method: &str,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let Some(key) = self.one_arg(method, args, &ty.key_ty(), span) else {
            return self.err_expr(span);
        };
        let key = match key.ty {
            Ty::Pos(_) => self.cast(Cast::Ord, key, Ty::Int),
            _ => key,
        };
        let seq = self.hold_view(value, span);
        let op = if method == "cut_after" {
            Builtin::CutAfter
        } else {
            Builtin::CutBefore
        };
        let cut = self.mk(ExprKind::Builtin(op, vec![seq, key]), Ty::Int, span);
        match ty.seq_key() {
            Some(Dim::Pos(d)) => self.cast(Cast::Trust, cut, Ty::Bound(d)),
            _ => cut,
        }
    }
}

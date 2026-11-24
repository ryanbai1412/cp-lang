//! Lowering an expression only to learn its type. The checker already knows
//! it (`cpl_check::Facts::type_of`), so each caller is debt: the
//! `ir_reinfer` ratchet counts them down to none, and this file goes with
//! the last.

use super::*;

impl<'a> Lowerer<'a> {
    /// Lowers `e` to see its type, keeping no diagnostics or context.
    pub(in crate::lower) fn probe_expr(&mut self, e: &ast::Expr, hint: Option<&Ty>) -> Expr {
        // `B<Int, Int>` probed as `B<Int, Int>.new(..)`'s receiver is a type:
        // its constructor closure would instantiate `B` inside a dropped lambda.
        if let ast::ExprKind::Generic { base, .. } = &e.kind {
            if let ast::ExprKind::Name(name) = &base.kind {
                if matches!(self.get_sym(name), Some(Sym::GenStruct(_))) {
                    return self.err_expr(e.span);
                }
            }
        }
        let saved_diags = self.diags.len();
        let saved_cx = self.cx.clone();
        let saved_outer = self.outer_cxs.clone();
        let saved_closures = self.closures.len();
        let val = self.expr(e, hint);
        self.diags.truncate(saved_diags);
        self.cx = saved_cx;
        self.outer_cxs = saved_outer;
        self.drop_closures_from(saved_closures);
        val
    }

    /// Forgets the closures a probe made: nothing that runs makes them, and
    /// a lambda probed at the wrong type lowers to `Error`.
    pub(in crate::lower) fn drop_closures_from(&mut self, len: usize) {
        self.closures.truncate(len);
        self.narrow_elems
            .retain(|loc, _| !matches!(loc, ElemLoc::Local(ScopeId::Closure(c), _) if *c >= len));
    }
}

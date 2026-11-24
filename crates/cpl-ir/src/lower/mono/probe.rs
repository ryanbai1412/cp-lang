//! Lowering an expression only to learn its type. The checker already knows
//! it (`cpl_check::Facts::type_of`), so each caller is debt: the
//! `ir_reinfer` ratchet counts them down to none, and this file goes with
//! the last.

use super::*;

impl<'a> Lowerer<'a> {
    /// Lowers `e` to see its type, keeping no diagnostics or context.
    pub(in crate::lower) fn probe_expr(&mut self, e: &ast::Expr, hint: Option<&Ty>) -> Expr {
        let saved_diags = self.diags.len();
        let saved_cx = self.cx.clone();
        let saved_outer = self.outer_cxs.clone();
        let val = self.expr(e, hint);
        self.diags.truncate(saved_diags);
        self.cx = saved_cx;
        self.outer_cxs = saved_outer;
        val
    }
}

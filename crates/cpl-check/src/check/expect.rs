//! `expect_fits` takes its message as `&dyn Fn`, so one copy of it (and of
//! the check behind it) serves every caller in cpl.wasm.

use super::*;
use cpl_syntax::ast::Expr;

impl Checker<'_> {
    pub(crate) fn expect_fits(
        &mut self,
        got: &Ty,
        want: &Ty,
        at: &Expr,
        message: &dyn Fn(&Self, &Ty, &Ty) -> cpl_diag::msgs::Said,
    ) -> bool {
        self.expect_fits_by(got, want, at, &mut |c, got, want| {
            Some(message(c, got, want))
        })
    }
}

//! `expect_fits`, generic over its message, is a thin shell: each of its
//! callers' closures would otherwise carry a whole copy of the check into
//! cpl.wasm.

use super::*;
use cpl_syntax::ast::Expr;

impl Checker<'_> {
    pub(crate) fn expect_fits(
        &mut self,
        got: &Ty,
        want: &Ty,
        at: &Expr,
        message: impl FnOnce(&Self, &Ty, &Ty) -> String,
    ) -> bool {
        let mut message = Some(message);
        self.expect_fits_by(got, want, at, &mut |c, got, want| {
            message.take().map_or_else(String::new, |m| m(c, got, want))
        })
    }
}

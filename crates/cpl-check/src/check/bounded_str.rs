//! `Str(len: lo..=hi)`: a string type whose length is a range (R02.3.2).

use super::*;
use cpl_syntax::ast;

impl Checker<'_> {
    /// The type of `Str(len: …) of "…"`: a `Str`, once the bounds of `len`,
    /// which may be runtime values, are checked.
    pub(super) fn bounded_str(&mut self, len: Option<&ast::Type>) -> Ty {
        if let Some(len) = len {
            self.resolve_type(len);
        }
        Ty::Str
    }
}

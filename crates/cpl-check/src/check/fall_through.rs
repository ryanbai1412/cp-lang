//! E0409: a fn that gives a value gives one, or leaves, on every path
//! through its body (the analysis is `cpl_syntax::fall_through`'s).

use super::*;
use cpl_diag::msgs;
use cpl_syntax::ast::FnDecl;

impl Checker<'_> {
    /// What is checked once a fn's body is: E0409 when it gives a value
    /// (not a generator's), and L0610 for a `plain` fn.
    pub(crate) fn after_fn_body(&mut self, f: &FnDecl, ret: &Ty, generator: bool, plain: bool) {
        let gives =
            f.ret.is_some() && !generator && !matches!(ret, Ty::Unit | Ty::Error | Ty::Unknown);
        if let Some((at, how)) = f.body.falls_through().filter(|_| gives) {
            let name = &f.name.text;
            if let Some(diag) = self.report(
                at,
                msgs::check::FN_FALLS_THROUGH_AND_THEN_ENDS_WITHOUT,
                &[&how, &name],
            ) {
                diag.help = Some(cpl_syntax::fall_through::help(
                    name,
                    &ret.with_article(),
                    how,
                ));
            }
        }
        if plain {
            self.unchanged_mut_params(f);
        }
    }
}

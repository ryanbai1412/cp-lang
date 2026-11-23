//! `mut` at the call for library fns and core builtins (R01.10.1).

use super::*;
use cpl_syntax::ast::{Arg, Expr, ExprKind};

impl Checker<'_> {
    /// `sort(xs)` for `sort(mut xs)`: a std fn or core builtin that changes
    /// an argument has it marked `mut` at the call, as a user fn does.
    pub(crate) fn free_call_mut_args(&mut self, callee: &Expr, args: &[Arg]) {
        let ExprKind::Name(name) = &callee.kind else {
            return;
        };
        let user_fn = self
            .global(name)
            .is_some_and(|d| matches!(&*d, Def::Fn { .. }));
        if self.local(name).is_some() || user_fn {
            return;
        }
        let first = args.iter().find(|a| a.name.is_none());
        let changes: Vec<bool> = if name == "swap" {
            vec![true, true]
        } else if let Some(first) = first.filter(|_| mutator(name)) {
            match self.quietly(|c| c.catalog_row(&first.value, name)) {
                Some((row, _)) if row.mutates => vec![true],
                _ => return,
            }
        } else {
            return;
        };
        let positional = args.iter().filter(|a| a.name.is_none());
        for (arg, _) in positional.zip(changes).filter(|(_, m)| *m) {
            if arg.mutable || !super::call::is_place(&arg.value) {
                continue;
            }
            let quoted = self.quote(arg.value.span);
            let message = format!(
                "`{name}` changes `{quoted}`, so the argument must be passed as `mut {quoted}`"
            );
            let at = arg.value.span;
            if let Some(diag) = self.error(codes::MISSING_MUT_AT_CALL, at, message) {
                diag.fix("add `mut`", Span::point(at.start), "mut ");
            }
        }
    }
}

/// Whether a built-in type has a method `name` that changes its receiver.
fn mutator(name: &str) -> bool {
    crate::builtins::BUILTIN_MEMBERS
        .iter()
        .any(|m| m.mutates && m.name == name)
}

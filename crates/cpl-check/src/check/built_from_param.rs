//! E0603: a struct or tuple built from a lent parameter is lent (R05.4.4),
//! so returning one needs `lends` or a copy (R05.5.1, R05.5.2).

use super::lends::unparen;
use super::*;
use cpl_syntax::ast::{Expr, ExprKind};

impl Checker<'_> {
    pub(crate) fn escape_built_from_param(&mut self, e: &Expr, what: &str) {
        let Some(part) = self.lent_param_part(e) else {
            return;
        };
        let ty = self.quietly(|c| c.infer(e)).to_string();
        let name = self.text(part.span);
        let whole = self.text(e.span);
        let at = (part.span.start - e.span.start) as usize;
        let copied = format!(
            "{}{name}.clone(){}",
            &whole[..at],
            &whole[at + name.len()..]
        );
        let message = format!(
            "{what} would return a `{ty}` holding lent parameter `{name}` without a `lends` clause"
        );
        if let Some(diag) = self.error(codes::LENT_ESCAPES, e.span, message) {
            diag.help = Some(format!(
                "build it from a copy, `{copied}`, or declare `lends {ty}`"
            ));
        }
    }

    /// The lent parameter a struct or tuple value is built from, if any.
    fn lent_param_part<'e>(&mut self, e: &'e Expr) -> Option<&'e Expr> {
        let e = unparen(e);
        let parts: Vec<&Expr> = match &e.kind {
            ExprKind::Tuple(items) => items.iter().collect(),
            ExprKind::Call { callee, args } if self.constructs_struct(callee) => {
                args.iter().map(|a| &a.value).collect()
            }
            ExprKind::Name(name) => {
                let local = self.local(name).cloned()?;
                let lent = local.keyword == "param" && self.is_heap_type(&local.ty);
                return lent.then_some(e);
            }
            _ => return None,
        };
        parts.into_iter().find_map(|p| self.lent_param_part(p))
    }

    /// Whether calling `callee` constructs a struct (`Box(..)`, `Box<T>(..)`).
    pub(crate) fn constructs_struct(&mut self, callee: &Expr) -> bool {
        let base = match &unparen(callee).kind {
            ExprKind::Generic { base, .. } => unparen(base),
            _ => unparen(callee),
        };
        let ExprKind::Name(n) = &base.kind else {
            return false;
        };
        matches!(self.type_def(n).as_deref(), Some(Def::Struct { .. }))
    }
}

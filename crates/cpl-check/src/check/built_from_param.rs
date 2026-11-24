//! E0603: a struct or tuple built from a lent parameter is lent (R05.4.4),
//! so returning one needs `lends` or a copy (R05.5.1, R05.5.2), and so is
//! a call whose result holds one (R05.5.3).

use super::lends::unparen;
use super::*;
use cpl_diag::msgs;
use cpl_syntax::ast::{Expr, ExprKind};

impl Checker<'_> {
    pub(crate) fn escape_built_from_param(&mut self, e: &Expr, what: &str) {
        let Some(part) = self.lent_param_part(e) else {
            return;
        };
        let ty = self.infer_quietly(e).to_string();
        let name = self.text(part.span);
        let whole = self.text(e.span);
        let at = (part.span.start - e.span.start) as usize;
        let copied = format!(
            "{}{name}.clone(){}",
            &whole[..at],
            &whole[at + name.len()..]
        );
        if let Some(diag) = self.report(
            e.span,
            msgs::check::LENT_ESCAPES_WOULD_RETURN_A_HOLDING,
            &[&what, &ty, &name],
        ) {
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
            // `filter(xs, p)`: the arguments the call's result holds (R05.5.3).
            ExprKind::Call { callee, args } if !self.in_generator() => {
                let mut held = Vec::new();
                self.held_args(e, callee, args, &mut |_, arg, _| held.push(arg));
                held
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

    /// A generator holds its arguments while it runs, so it may yield a
    /// view of one (R07.2.10).
    fn in_generator(&self) -> bool {
        self.fns
            .last()
            .is_some_and(|f| matches!(&f.1, Ty::Named { name, .. } if name == "Gen"))
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

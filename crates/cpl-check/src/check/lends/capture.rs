//! What a closure or a `lends` getter holds while its value lives (R05.10.1).

use super::*;

impl Checker<'_> {
    /// What a closure holds while it lives: each name it captures by reference, and for a
    /// captured view the place the view was lent from.
    /// A capture sees later writes, so only a captured view's source is held as a lend.
    pub(crate) fn captured_places(&mut self, lambda: &Expr) -> Vec<(String, Vec<Step>, bool)> {
        let caps = self.lambda_ref_captures(lambda);
        let mut out = Vec::new();
        for cap in caps {
            let lent = match self.local(&cap) {
                Some(l) => self.lends.views.get(&l.span).cloned(),
                None => self.lends.global_views.get(&cap).cloned(),
            };
            if let Some(lent) = lent {
                out.push((lent.root, lent.steps, false));
            }
            out.push((cap, Vec::new(), true));
        }
        out
    }

    /// Whether `value` bound to `name` is a closure; one that captures only
    /// copied values is noted as a plain value (decision 471).
    pub(crate) fn note_closure(&mut self, name: &str, value: &Expr) -> bool {
        if !matches!(unparen(value).kind, ExprKind::Lambda { .. }) {
            return false;
        }
        let caps = self.lambda_ref_captures(value);
        let plain = caps.iter().all(|cap| {
            self.local(cap)
                .is_some_and(|l| !l.mutable && self.lends.plain_closures.contains(&l.span))
        });
        if let Some(decl) = self.local(name).filter(|_| plain).map(|l| l.span) {
            self.lends.plain_closures.insert(decl);
        }
        true
    }

    /// Whether a closure holds the captured local `local` by reference: a
    /// `var`, or a heap or closure value that is not a plain one.
    pub(crate) fn captured_by_ref(&mut self, local: &Local) -> bool {
        local.mutable
            || (self.is_lent_param_ty(&local.ty)
                && !self.lends.plain_closures.contains(&local.span))
    }

    /// Whether the getter `name` is a `lends` fn (`g.rows`).
    pub(crate) fn lends_getter(&mut self, name: &ast::Name) -> bool {
        matches!(
            self.chosen_method(name).as_deref(),
            Some(Def::Fn { lends: true, .. })
        )
    }

    /// A `lends` getter holds its whole receiver `base`.
    pub(crate) fn receiver_hold(&mut self, base: &Expr) -> Vec<(&'static str, Place, bool, bool)> {
        let Some(mut place) = self.place(base) else {
            return Vec::new();
        };
        place.steps.clear();
        vec![("lent view", place, true, false)]
    }

    /// Returning a lent parameter without `lends`, or returning a closure that
    /// captures a local variable, is E0603 (R05.7.5 / R05.7.8 / R05.10.2).
    pub(crate) fn no_escape_return(&mut self, e: &Expr, what: &str) {
        if self.fn_lends.last().copied().unwrap_or(false) {
            return;
        }
        let e = unparen(e);
        match &e.kind {
            ExprKind::Name(name) => {
                if let Some(local) = self.local(name).cloned() {
                    if local.keyword == "param" && self.is_lent_param_ty(&local.ty) {
                        let message = format!(
                            "{what} would return lent parameter `{name}` without a `lends` clause"
                        );
                        self.error(codes::LENT_ESCAPES, e.span, message);
                    }
                }
            }
            ExprKind::Lambda { params, body, .. } => {
                let mut param_names: Vec<String> = Vec::new();
                for p in params {
                    if let ast::PatternKind::Name(n) = &p.kind {
                        param_names.push(n.clone());
                    }
                }
                let fn_depth = self.fns.last().map(|f| f.3).unwrap_or(0);
                let escaped = names_in(body).into_iter().find(|n| {
                    !param_names.iter().any(|p| p == n)
                        && self.var_depth(n) >= fn_depth
                        && self
                            .local(n)
                            .cloned()
                            .is_some_and(|l| self.captured_by_ref(&l))
                });
                if let Some(cap) = escaped {
                    let message = format!(
                        "{what} returns a closure that captures local `{cap}`, which does not outlive this fn"
                    );
                    self.error(codes::LENT_ESCAPES, e.span, message);
                }
            }
            _ => self.escape_built_from_param(e, what),
        }
    }
}

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
}

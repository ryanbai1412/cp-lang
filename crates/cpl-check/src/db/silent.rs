//! The types the checker lost without saying so, counted by
//! crates/cpl-conform/tests/silent_unknowns.rs until there are none.

use super::*;

impl Analysis {
    /// Every expression the checker typed `Unknown` with no diagnostic over
    /// it: a type it lost without saying so, which every tool then guesses at.
    /// A callee (`f` of `f(x)`, `xs.map` of `xs.map(g)`) is not one: its fact
    /// is the callee, not a type.
    pub fn silent_unknowns(&self) -> Vec<Span> {
        let callees: std::collections::HashSet<Span> = (self.items.iter())
            .flat_map(|item| {
                (item.check.exprs.calls.iter())
                    .flat_map(|c| [c.name_span, Span::new(c.call.start, c.name_span.end)])
                    .map(|s| s.shift(item.offset))
            })
            .collect();
        let wide = |s: Span| (s.start, s.end.max(s.start + 1));
        let reported = |s: Span| {
            let (a, b) = wide(s);
            self.diagnostics.iter().any(|d| {
                let (c, e) = wide(d.span);
                a < e && c < b
            })
        };
        let file = Span::new(0, self.text.len() as u32);
        (self.facts().types_in(file))
            .filter(|(s, ty)| matches!(ty, Ty::Unknown) && !callees.contains(s) && !reported(*s))
            .map(|(s, _)| s)
            .collect()
    }
}

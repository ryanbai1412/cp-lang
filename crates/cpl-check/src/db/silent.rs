//! The types the checker lost without saying so, counted per file and per
//! cause by crates/cpl-conform/tests/silent_unknowns.rs until there are none.

use super::*;

impl Analysis {
    /// Every expression the checker typed `Unknown` with no diagnostic over
    /// it: a type it lost without saying so, which every tool then guesses at.
    /// A callee (`f` of `f(x)`, `xs.map` of `xs.map(g)`, `read<Int>` of
    /// `read<Int>()`) is not one: its fact is the callee, not a type.
    pub fn silent_unknowns(&self) -> Vec<Span> {
        let calls: Vec<(Span, Span)> = (self.items.iter())
            .flat_map(|item| {
                (item.check.exprs.calls.iter())
                    .map(|c| (c.call.shift(item.offset), c.name_span.shift(item.offset)))
            })
            .collect();
        let callees: std::collections::HashSet<Span> = (calls.iter())
            .flat_map(|&(call, name)| [name, Span::new(call.start, name.end)])
            .collect();
        let bytes = self.text.as_bytes();
        let generic_callee = |s: Span| {
            bytes.get(s.end as usize) == Some(&b'(')
                && calls.iter().any(|&(call, name)| {
                    s.start == call.start && s.end > name.end && s.end < call.end
                })
        };
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
            .filter(|(s, ty)| {
                matches!(ty, Ty::Unknown)
                    && !callees.contains(s)
                    && !generic_callee(*s)
                    && !reported(*s)
            })
            .map(|(s, _)| s)
            .collect()
    }

    /// [`Analysis::silent_unknowns`], each with its cause: the kind of
    /// expression whose type was lost, `binding` for a name a pattern binds.
    /// The gate holds each cause to its own ceiling, so a new cause fails
    /// instead of hiding under another's fixes.
    pub fn silent_unknown_causes(&self) -> Vec<(Span, &'static str)> {
        struct Kinds(std::collections::HashMap<Span, &'static str>, u32);
        impl cpl_syntax::visit::Visitor for Kinds {
            fn expr(&mut self, expr: &cpl_syntax::ast::Expr) {
                let span = expr.span.shift(self.1);
                self.0.entry(span).or_insert_with(|| cause(&expr.kind));
            }
        }
        let mut kinds = Kinds(Default::default(), 0);
        for item in &self.items {
            kinds.1 = item.offset;
            for stmt in &item.syntax.parsed.stmts {
                cpl_syntax::visit::walk_stmt(stmt, &mut kinds);
            }
        }
        (self.silent_unknowns().into_iter())
            .map(|s| (s, kinds.0.get(&s).copied().unwrap_or("binding")))
            .collect()
    }
}

fn cause(kind: &cpl_syntax::ast::ExprKind) -> &'static str {
    use cpl_syntax::ast::ExprKind as K;
    match kind {
        K::Name(_) | K::SelfValue => "name",
        K::Call { callee, .. } => match &callee.kind {
            K::Field { .. } => "method-call",
            K::Generic { base, .. } if matches!(base.kind, K::Field { .. }) => "method-call",
            _ => "call",
        },
        K::Field { .. } => "member",
        K::Index { .. } => "index",
        K::List(_) | K::Comprehension { .. } => "list",
        K::Unwrap(_) | K::Try(_) => "unwrap",
        K::Lambda { .. } => "lambda",
        K::Generic { .. } => "generic",
        K::Unary { .. } | K::Binary { .. } | K::Compare { .. } => "operator",
        K::If { .. } | K::Match { .. } | K::Block(_) => "branch",
        _ => "other",
    }
}

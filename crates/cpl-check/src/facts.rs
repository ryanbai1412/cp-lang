//! The fact table (design/overview.md, M3): what checking one file found
//! about each place, in file positions, for the tools that read it instead
//! of re-deriving it from syntax. Built once per revision on first use, so
//! `cpl lint`, the editor and `cpl fix` read the facts the checker reported from.

use crate::{Analysis, Revision, Ty, TypedIndex};
use cpl_diag::Span;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct Facts {
    revision: Revision,
    /// The type of each name, member, call, index, unwrap and parenthesised
    /// expression, by file span.
    types: Vec<(Span, Ty)>,
    /// The type a member belongs to, by the member name's file span:
    /// `Bitset` for the `first` of `b.first()`.
    owners: HashMap<(u32, u32), String>,
    /// The indexes the checker proves by their type, `dist[v]` with `v: V`.
    typed_indexes: Vec<TypedIndex>,
    /// The type of every expression, one per file span, by start then end.
    exprs: Vec<(Span, Ty)>,
}

impl Facts {
    pub fn from_analysis(analysis: &Analysis) -> Facts {
        let mut facts = Facts {
            revision: analysis.revision,
            ..Facts::default()
        };
        let mut exprs: HashMap<(u32, u32), Ty> = HashMap::new();
        for item in &analysis.items {
            let shift = |s: Span| s.shift(item.offset);
            for (span, ty) in &item.check.expr_types {
                let span = shift(*span);
                let slot = exprs.entry((span.start, span.end)).or_insert(Ty::Unknown);
                // The last answer wins, but a vague re-check keeps a known one.
                if !ty.is_vague() || slot.is_vague() {
                    *slot = ty.clone();
                }
            }
            for entry in &item.check.hovers {
                facts.types.push((shift(entry.span), entry.ty.clone()));
                if let ("field", Some(owner)) = (entry.keyword, &entry.owner) {
                    let span = shift(entry.span);
                    facts.owners.insert((span.start, span.end), owner.clone());
                }
            }
            for (span, ty) in &item.check.shapes {
                facts.types.push((shift(*span), ty.clone()));
            }
            for t in &item.check.typed_indexes {
                facts.typed_indexes.push(TypedIndex {
                    access: shift(t.access),
                    seq: shift(t.seq),
                    index: shift(t.index),
                    proof: t.proof.clone(),
                });
            }
        }
        // By end, then start: `place_type` looks a place up by its end.
        facts.types.sort_by_key(|(s, _)| (s.end, s.start));
        facts.exprs = (exprs.into_iter())
            .map(|((start, end), ty)| (Span::new(start, end), ty))
            .collect();
        facts.exprs.sort_by_key(|(s, _)| (s.start, s.end));
        facts
    }

    /// The type the checker gave the expression at exactly `expr`.
    pub fn type_of(&self, expr: Span) -> Option<&Ty> {
        (self.exprs)
            .binary_search_by_key(&(expr.start, expr.end), |(s, _)| (s.start, s.end))
            .ok()
            .map(|at| &self.exprs[at].1)
    }

    /// Every typed expression inside `span`, in source order: by start, then end.
    pub fn types_in(&self, span: Span) -> impl Iterator<Item = (Span, &Ty)> {
        let from = (self.exprs).partition_point(|(s, _)| s.start < span.start);
        self.exprs[from..]
            .iter()
            .take_while(move |(s, _)| s.start <= span.end)
            .filter(move |(s, _)| s.end <= span.end)
            .map(|(s, ty)| (*s, ty))
    }

    /// The session revision these facts were found at.
    pub fn revision(&self) -> Revision {
        self.revision
    }

    /// The type of the place `span` names (`x`, `g[i][j]`, `self.best`): the
    /// widest typed span inside it that ends where it ends.
    pub fn place_type(&self, span: Span) -> Option<&Ty> {
        let at = self
            .types
            .partition_point(|(s, _)| (s.end, s.start) < (span.end, span.start));
        self.types
            .get(at)
            .filter(|(s, _)| s.end == span.end)
            .map(|(_, ty)| ty)
    }

    /// The type the member named at `span` belongs to.
    pub fn owner(&self, span: Span) -> Option<&str> {
        self.owners.get(&(span.start, span.end)).map(String::as_str)
    }

    /// The checker's proof that the index access at `access` is in bounds.
    pub fn typed_index(&self, access: Span) -> Option<&TypedIndex> {
        self.typed_indexes.iter().find(|t| t.access == access)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Database, Ty};
    use cpl_diag::Span;

    #[test]
    fn a_revision_builds_its_facts_once_in_file_positions() {
        let src = "let n = 3\nfn f(x: Int) -> Int:\n    x + n\nprint(f(n))\n";
        let mut db = Database::new();
        db.set_file("a.cp", src);
        let analysis = db.analyze("a.cp").unwrap();
        assert!(std::ptr::eq(analysis.facts(), analysis.facts()));
        let x = src.find("x + n").unwrap() as u32;
        assert_eq!(
            analysis.facts().place_type(Span::new(x, x + 1)),
            Some(&Ty::Int)
        );
    }

    /// Every expression but a callee, whose answer is `Facts::callee`.
    struct Exprs(Vec<Span>, Vec<Span>);
    impl cpl_syntax::visit::Visitor for Exprs {
        fn expr(&mut self, e: &cpl_syntax::ast::Expr) {
            if let cpl_syntax::ast::ExprKind::Call { callee, .. } = &e.kind {
                self.1.push(callee.span);
            }
            if !self.1.contains(&e.span) {
                self.0.push(e.span);
            }
        }
    }

    #[test]
    fn every_expression_has_its_type() {
        let src = "struct P:\n    x: Int\n    y: Int\n\nfn f(x: Int) -> Int = x * 2\n\nlet n = 3\nlet xs = [1, 2, n]\nlet p = P(x: 1, y: (n + 1))\nvar t = 0\nfor i in 0..n:\n    t += xs[i] + f(i)\nlet m = xs.map(v => v + p.x).sum()\nlet o: ?Int = None\nlet k = if t > 2: -t else: o ?? 0\nprint(m, k, \"s\", 'c', 1.5, yes(true), (n, t).0)\n";
        let mut db = Database::new();
        db.set_file("a.cp", src);
        let analysis = db.analyze("a.cp").unwrap();
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let facts = analysis.facts();
        let mut exprs = Exprs(Vec::new(), Vec::new());
        for (offset, item) in cpl_syntax::parse_file(src).items {
            let mut item_exprs = Exprs(Vec::new(), Vec::new());
            for stmt in &item.stmts {
                cpl_syntax::visit::walk_stmt(stmt, &mut item_exprs);
            }
            exprs
                .0
                .extend(item_exprs.0.into_iter().map(|s| s.shift(offset)));
        }
        let untyped: Vec<_> = (exprs.0.iter())
            .filter(|s| {
                facts
                    .type_of(**s)
                    .is_none_or(|t| *t == Ty::Unknown || *t == Ty::Error)
            })
            .map(|s| (&src[s.start as usize..s.end as usize], facts.type_of(*s)))
            .collect();
        assert!(untyped.is_empty(), "no type for {untyped:?}");
        let at = |text: &str| {
            let start = src.find(text).unwrap() as u32;
            facts.type_of(Span::new(start, start + text.len() as u32))
        };
        assert_eq!(at("xs[i]"), Some(&Ty::Int));
        assert_eq!(at("v + p.x"), Some(&Ty::Int));
        assert!(matches!(at("v => v + p.x"), Some(Ty::Fn { ret, .. }) if **ret == Ty::Int));
        assert_eq!(at("None"), Some(&Ty::Optional(Box::new(Ty::Unknown))));
        let whole = Span::new(0, src.len() as u32);
        let order: Vec<Span> = facts.types_in(whole).map(|(s, _)| s).collect();
        assert!(order
            .windows(2)
            .all(|w| (w[0].start, w[0].end) < (w[1].start, w[1].end)));
        assert_eq!(order.len(), facts.types_in(whole).count());
    }
}

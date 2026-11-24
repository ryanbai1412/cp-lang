//! The fact table (design/overview.md, M3): what checking one file found
//! about each place, in file positions, for the tools that read it instead
//! of re-deriving it from syntax. Built once per revision on first use, so
//! `cpl lint`, the editor and `cpl fix` read the facts the checker reported from.

use crate::db::ItemCheck;
use crate::{Analysis, Revision, Ty, TypedIndex};
use by_span::BySpan;
use cpl_diag::Span;

mod by_span;

/// What a call calls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalleeKind {
    /// A function by name, local, imported or `alias.f` of a module.
    Fn,
    /// `x.m(..)`: a method of `x`'s type, std's or the program's.
    Method,
    /// `x.m` read as a getter.
    Getter,
    /// A type's name, called to build or convert a value.
    Ctor,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Callee {
    /// The module the callee is declared in: `prelude`, `std.graph`, the
    /// file's own (empty), or empty for a built-in with no declaration.
    pub module: String,
    /// The type a method or getter belongs to, when the checker knows it.
    pub owner: Option<String>,
    pub name: String,
    pub kind: CalleeKind,
}

/// One call the checker typed, relative to its item.
#[derive(Clone, Debug)]
pub struct CallFact {
    pub call: Span,
    /// The callee's name: `f` of `f(x)`, `m` of `x.m(y)`.
    pub name_span: Span,
    pub name: String,
    pub kind: CalleeKind,
}

/// What checking one item found about its expressions, relative to it.
#[derive(Clone, Debug, Default)]
pub struct ExprFacts {
    /// Each type `infer` gave, in the order it did.
    pub types: Vec<(Span, Ty)>,
    pub calls: Vec<CallFact>,
    /// The type arguments each generic call bound, by name.
    pub instantiations: Vec<(Span, Vec<(String, Ty)>)>,
    /// The `m` of each `alias.m` naming a module's member.
    pub module_members: Vec<Span>,
}

impl ExprFacts {
    pub(crate) fn mark(&self) -> [usize; 4] {
        let (t, c) = (self.types.len(), self.calls.len());
        [t, c, self.instantiations.len(), self.module_members.len()]
    }

    /// Forgets what was recorded since `mark`, as a quiet probe does.
    pub(crate) fn truncate(&mut self, [t, c, i, m]: [usize; 4]) {
        self.types.truncate(t);
        self.calls.truncate(c);
        self.instantiations.truncate(i);
        self.module_members.truncate(m);
    }
}

#[derive(Clone, Debug, Default)]
pub struct Facts {
    revision: Revision,
    /// The type of each name, member, call, index, unwrap and parenthesised
    /// expression, by file span.
    types: Vec<(Span, Ty)>,
    /// The type a member belongs to, by the member name's file span:
    /// `Bitset` for the `first` of `b.first()`.
    owners: BySpan<String>,
    /// The indexes the checker proves by their type, `dist[v]` with `v: V`.
    typed_indexes: Vec<TypedIndex>,
    /// The type of every expression, one per file span, by start then end.
    exprs: BySpan<Ty>,
    /// The callee of each call, by the call's file span.
    callees: BySpan<Callee>,
    instantiations: BySpan<Vec<(String, Ty)>>,
}

impl Facts {
    pub fn from_analysis(analysis: &Analysis) -> Facts {
        let items: Vec<(u32, &ItemCheck)> = (analysis.items.iter())
            .map(|item| (item.offset, &*item.check))
            .collect();
        Facts::from_checks(analysis.revision, &items)
    }

    /// The facts of the checked items `items`, each at its file offset.
    pub(crate) fn from_checks(revision: Revision, items: &[(u32, &ItemCheck)]) -> Facts {
        let mut facts = Facts {
            revision,
            ..Facts::default()
        };
        let (mut exprs, mut owners) = (Vec::new(), Vec::new());
        for &(offset, check) in items {
            let shift = |s: Span| s.shift(offset);
            for (span, ty) in &check.exprs.types {
                exprs.push((shift(*span), ty.clone()));
            }
            for entry in &check.hovers {
                facts.types.push((shift(entry.span), entry.ty.clone()));
                if let ("field", Some(owner)) = (entry.keyword, &entry.owner) {
                    owners.push((shift(entry.span), owner.clone()));
                }
            }
            for (span, ty) in &check.shapes {
                facts.types.push((shift(*span), ty.clone()));
            }
            for t in &check.typed_indexes {
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
        let (mut callees, mut instantiations) = (Vec::new(), Vec::new());
        for &(offset, check) in items {
            add_calls(offset, check, &mut callees, &mut instantiations);
        }
        // The last answer wins, but a vague re-check keeps a known one.
        facts.exprs = BySpan::merged(exprs, |kept, earlier| {
            if kept.is_vague() && !earlier.is_vague() {
                std::mem::swap(kept, earlier);
            }
        });
        facts.owners = BySpan::new(owners);
        facts.callees = BySpan::new(callees);
        facts.instantiations = BySpan::new(instantiations);
        facts
    }

    /// What the call at exactly `call` calls.
    pub fn callee(&self, call: Span) -> Option<&Callee> {
        self.callees.get(call)
    }

    /// The type each generic parameter took at the call `call`, by name.
    pub fn instantiation(&self, call: Span) -> Option<&[(String, Ty)]> {
        self.instantiations.get(call).map(Vec::as_slice)
    }

    /// The type the checker gave the expression at exactly `expr`.
    pub fn type_of(&self, expr: Span) -> Option<&Ty> {
        self.exprs.get(expr)
    }

    /// Every typed expression inside `span`, in source order: by start, then end.
    pub fn types_in(&self, span: Span) -> impl Iterator<Item = (Span, &Ty)> {
        self.exprs.within(span)
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
        self.owners.get(span).map(String::as_str)
    }

    /// The checker's proof that the index access at `access` is in bounds.
    pub fn typed_index(&self, access: Span) -> Option<&TypedIndex> {
        self.typed_indexes.iter().find(|t| t.access == access)
    }
}

fn add_calls(
    offset: u32,
    check: &ItemCheck,
    callees: &mut Vec<(Span, Callee)>,
    instantiations: &mut Vec<(Span, Vec<(String, Ty)>)>,
) {
    let shift = |s: Span| s.shift(offset);
    let resolved = BySpan::new((check.resolved.iter()).map(|(s, r)| (*s, r)).collect());
    let owners = BySpan::new(
        (check.hovers.iter())
            .filter_map(|h| Some((h.span, h.owner.as_deref()?)))
            .collect(),
    );
    for call in &check.exprs.calls {
        let found = resolved.get(call.name_span);
        let owner = match call.kind {
            CalleeKind::Method | CalleeKind::Getter => (found.and_then(|r| r.owner.clone()))
                .or_else(|| owners.get(call.name_span).map(|o| o.to_string())),
            _ => None,
        };
        let callee = Callee {
            module: found.map(|r| r.module.clone()).unwrap_or_default(),
            owner,
            name: call.name.clone(),
            kind: call.kind,
        };
        callees.push((shift(call.call), callee));
    }
    for (call, bound) in &check.exprs.instantiations {
        instantiations.push((shift(*call), bound.clone()));
    }
}

#[cfg(test)]
mod tests {
    use crate::{CalleeKind, Database, Ty};
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

    #[test]
    fn every_call_has_its_callee_and_bindings() {
        let src = "struct P:\n    x: Int\n\nimpl P:\n    fn twice(self) -> Int = self.x * 2\n\nfn id<T>(v: T) -> T = v\nfn f(x: Int) -> Int = x + 1\n\nlet p = P(x: 3)\nlet a = f(p.twice())\nlet b = id(\"s\")\nlet xs = [3, 1, 2]\nlet c = xs.sorted()\nprint(a, b, c)\n";
        let mut db = Database::new();
        db.set_file("a.cp", src);
        let analysis = db.analyze("a.cp").unwrap();
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let facts = analysis.facts();
        let at = |text: &str| {
            let start = src.find(text).unwrap() as u32;
            Span::new(start, start + text.len() as u32)
        };
        let callee = |text: &str| facts.callee(at(text)).cloned();
        let kind = |text: &str| callee(text).map(|c| (c.name, c.kind));
        assert_eq!(kind("f(p.twice())"), Some(("f".into(), CalleeKind::Fn)));
        assert_eq!(
            kind("p.twice()"),
            Some(("twice".into(), CalleeKind::Method))
        );
        assert_eq!(kind("P(x: 3)"), Some(("P".into(), CalleeKind::Ctor)));
        assert_eq!(
            kind("xs.sorted()"),
            Some(("sorted".into(), CalleeKind::Method))
        );
        assert_eq!(
            kind("print(a, b, c)"),
            Some(("print".into(), CalleeKind::Fn))
        );
        assert_eq!(callee("print(a, b, c)").unwrap().module, "prelude");
        assert_eq!(callee("p.twice()").unwrap().owner.as_deref(), Some("P"));
        assert_eq!(
            facts.instantiation(at("id(\"s\")")),
            Some(&[("T".to_string(), Ty::Str)][..])
        );
        assert_eq!(facts.instantiation(at("f(p.twice())")), None);
    }
}

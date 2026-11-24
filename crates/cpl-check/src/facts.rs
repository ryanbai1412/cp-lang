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
    /// A method or getter no declaration gives: the compiler's own, such
    /// as `List.push` or `F64.sqrt`. A program's methods are declared.
    pub builtin: bool,
}

/// One call the checker typed, relative to its item.
#[derive(Clone, Debug)]
pub struct CallFact {
    pub call: Span,
    /// The callee's name: `f` of `f(x)`, `m` of `x.m(y)`.
    pub name_span: Span,
    pub name: String,
    pub kind: CalleeKind,
    /// A member the compiler gives a built-in type, `xs.push` or `x.sqrt`.
    pub builtin: bool,
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
    /// The `m` of each `x.m(..)` that calls a fn of the name because the
    /// built-in member `m` takes fewer arguments: `a.contains(i, x)`.
    pub past_rows: Vec<Span>,
    /// Each member access the checker could not resolve, with its path as
    /// written: `t.postorder` on an unknown `t`, `V.sorted` on a domain.
    pub unresolved: Vec<(Span, Vec<String>)>,
    /// The type each written type resolved to: `Map<V, Int>` in `m: Map<V, Int>`.
    pub written: Vec<(Span, Ty)>,
    /// The ends of each range type written with both: `x + 1..=n`.
    pub ranges: Vec<(Span, RangeEnds)>,
}

/// The end expressions of a written range type, by span.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RangeEnds {
    pub lo: Span,
    pub hi: Span,
    pub inclusive: bool,
}

impl ExprFacts {
    /// The last type `infer` gave the expression at `span`.
    #[inline(never)]
    pub(crate) fn last_type(&self, span: Span) -> Option<&Ty> {
        self.types
            .iter()
            .rev()
            .find(|(at, _)| *at == span)
            .map(|(_, ty)| ty)
    }

    pub(crate) fn mark(&self) -> [usize; 6] {
        let (t, c) = (self.types.len(), self.calls.len());
        let rows = self.past_rows.len();
        [
            t,
            c,
            self.instantiations.len(),
            self.module_members.len(),
            rows,
            self.unresolved.len(),
        ]
    }

    /// Forgets what was recorded since `mark`, as a quiet probe does.
    pub(crate) fn truncate(&mut self, [t, c, i, m, r, u]: [usize; 6]) {
        self.types.truncate(t);
        self.calls.truncate(c);
        self.instantiations.truncate(i);
        self.module_members.truncate(m);
        self.past_rows.truncate(r);
        self.unresolved.truncate(u);
    }
}

#[derive(Clone, Debug, Default)]
pub struct Facts {
    revision: Revision,
    /// The type a member belongs to, by the member name's file span:
    /// `Bitset` for the `first` of `b.first()`.
    owners: BySpan<String>,
    /// The indexes the checker proves by their type, `dist[v]` with `v: V`.
    typed_indexes: Vec<TypedIndex>,
    /// The type of every expression, and of each declared name, one per
    /// file span, by start then end.
    exprs: BySpan<Ty>,
    /// The callee of each call, by the call's file span.
    callees: BySpan<Callee>,
    instantiations: BySpan<Vec<(String, Ty)>>,
    /// The written path of each member access the checker could not resolve.
    unresolved: BySpan<Vec<String>>,
    /// The type each written type resolved to, by file span.
    written: BySpan<Ty>,
    /// The ends of each range type written with both, by file span.
    ranges: BySpan<RangeEnds>,
}

impl Facts {
    pub fn from_analysis(analysis: &Analysis) -> Facts {
        let items: Vec<(u32, &ItemCheck)> = (analysis.items.iter())
            .map(|item| (item.offset, &*item.check))
            .collect();
        Facts::from_checks(analysis, &items)
    }

    /// The facts of the checked items `items`, each at its file offset.
    pub(crate) fn from_checks(analysis: &Analysis, items: &[(u32, &ItemCheck)]) -> Facts {
        let mut facts = Facts {
            revision: analysis.revision,
            ..Facts::default()
        };
        let (mut exprs, mut owners, mut unresolved) = (Vec::new(), Vec::new(), Vec::new());
        let (mut written, mut named, mut ranges) = (Vec::new(), Vec::new(), Vec::new());
        for &(offset, check) in items {
            let shift = |s: Span| s.shift(offset);
            for (span, path) in &check.exprs.unresolved {
                unresolved.push((shift(*span), path.clone()));
            }
            for (span, ty) in &check.exprs.types {
                exprs.push((shift(*span), ty.clone()));
            }
            for (span, ty) in &check.exprs.written {
                written.push((shift(*span), ty.clone()));
            }
            for entry in check.hovers.iter().filter(|e| !e.ty.is_vague()) {
                named.push((shift(entry.span), entry.ty.clone()));
            }
            for &(span, RangeEnds { lo, hi, inclusive }) in &check.exprs.ranges {
                let (lo, hi) = (shift(lo), shift(hi));
                ranges.push((shift(span), RangeEnds { lo, hi, inclusive }));
            }
            for entry in &check.hovers {
                if let ("field", Some(owner)) = (entry.keyword, &entry.owner) {
                    owners.push((shift(entry.span), owner.clone()));
                }
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
        let (mut callees, mut instantiations) = (Vec::new(), Vec::new());
        for &(offset, check) in items {
            add_calls(offset, check, analysis, &mut callees, &mut instantiations);
            // A call no declaration answers and the compiler doesn't give.
            for call in &check.exprs.calls {
                let resolved = check.resolved.iter().any(|(s, _)| *s == call.name_span);
                if !resolved && !call.builtin && call.kind != CalleeKind::Ctor {
                    unresolved.push((call.call.shift(offset), vec![call.name.clone()]));
                }
            }
        }
        // The last answer wins, but a vague re-check keeps a known one. A
        // name's declared type answers where no expression is: `x` of `let x`.
        named.append(&mut exprs);
        facts.exprs = BySpan::merged(named, |kept, earlier| {
            if kept.is_vague() && !earlier.is_vague() {
                std::mem::swap(kept, earlier);
            }
        });
        facts.owners = BySpan::new(owners);
        facts.unresolved = BySpan::new(unresolved);
        facts.written = BySpan::new(written);
        facts.ranges = BySpan::new(ranges);
        facts.callees = BySpan::new(callees);
        facts.instantiations = BySpan::new(instantiations);
        facts
    }

    /// What the call at exactly `call` calls.
    pub fn callee(&self, call: Span) -> Option<&Callee> {
        self.callees.get(call)
    }

    /// The path as written of the member access at exactly `at` when the
    /// checker could not resolve it: `["t", "postorder"]`. A lint about a
    /// misspelled member reads this, never the source text.
    pub fn unresolved(&self, at: Span) -> Option<&[String]> {
        self.unresolved.get(at).map(Vec::as_slice)
    }

    /// The type each generic parameter took at the call `call`, by name.
    pub fn instantiation(&self, call: Span) -> Option<&[(String, Ty)]> {
        self.instantiations.get(call).map(Vec::as_slice)
    }

    /// The type the checker gave the expression at exactly `expr`.
    pub fn type_of(&self, expr: Span) -> Option<&Ty> {
        self.exprs.get(expr)
    }

    /// The type the type written at exactly `ty` resolved to.
    pub fn written_type(&self, ty: Span) -> Option<&Ty> {
        self.written.get(ty)
    }

    /// The ends of the range type written at exactly `range`.
    pub fn range_ends(&self, range: Span) -> Option<RangeEnds> {
        self.ranges.get(range).copied()
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
        let mut ends = (self.exprs.within(span)).filter(|(s, _)| s.end == span.end);
        ends.find(|(_, ty)| !ty.is_vague()).map(|(_, ty)| ty)
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

/// The module `owner` comes from: imported by name, or declared in a module
/// this file reaches (`Heap` behind std's `MinHeap`). `None` for its own.
fn owner_module(analysis: &Analysis, owner: &str) -> Option<String> {
    let modules = analysis.modules();
    match analysis.imports().get(owner) {
        Some(import) => Some(modules[import.module].name.clone()),
        None if analysis.decl_span(owner).is_some() => None,
        None => (modules.iter())
            .find(|m| m.analysis.decl_span(owner).is_some())
            .map(|m| m.name.clone()),
    }
}

fn add_calls(
    offset: u32,
    check: &ItemCheck,
    analysis: &Analysis,
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
            // A member of an imported type, `h.push(x)` on std's `MinHeap`,
            // comes from the module the type is imported from.
            module: (found.map(|r| r.module.clone()))
                .or_else(|| owner_module(analysis, owner.as_deref()?).filter(|_| !call.builtin))
                .unwrap_or_default(),
            owner,
            name: call.name.clone(),
            kind: call.kind,
            builtin: call.builtin && found.is_none(),
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

    #[test]
    fn a_range_folds_its_constant_ends() {
        let src = "const K = 2000\nconst N = 5\nlet a: 1..=K * K = read()\nlet b: 0..2 * N + 1 = read()\nlet c: 1..=10 ** 9 + 7 = read()\nprint(a, b, c)\n";
        let mut db = Database::new();
        db.set_file("a.cp", src);
        let analysis = db.analyze("a.cp").unwrap();
        let ends = |name: &str| {
            let start = src.find(&format!("let {name}")).unwrap() as u32 + 4;
            match analysis.facts().place_type(Span::new(start, start + 1)) {
                Some(Ty::Range { lo, hi, .. }) => (*lo, *hi),
                other => panic!("{name}: {other:?}"),
            }
        };
        assert_eq!(ends("a"), (Some(1), Some(4_000_000)));
        assert_eq!(ends("b"), (Some(0), Some(10)));
        assert_eq!(ends("c"), (Some(1), Some(1_000_000_007)));
    }

    #[test]
    fn a_written_type_keeps_the_type_it_resolved_to() {
        let src = "domain V = 1..=5\nfn f<K>(m: Map<V, Int>, k: Set<K>) -> Int = m.len\n";
        let mut db = Database::new();
        db.set_file("a.cp", src);
        let analysis = db.analyze("a.cp").unwrap();
        let at = |text: &str| {
            let start = src.find(text).unwrap() as u32;
            Span::new(start, start + text.len() as u32)
        };
        let facts = analysis.facts();
        let map = Ty::Named {
            name: "Map".into(),
            args: vec![Ty::Pos("V".into()), Ty::Int],
        };
        assert_eq!(facts.written_type(at("Map<V, Int>")), Some(&map));
        assert_eq!(facts.written_type(at("V, Int")), None);
        assert!(!matches!(
            facts.written_type(at("Set<K>")),
            Some(Ty::Named { args, .. }) if matches!(args[..], [Ty::Pos(_)])
        ));
    }

    #[test]
    fn a_written_range_keeps_the_spans_of_its_ends() {
        let src = "let n: 1..=9 = read()\nlet x: 0..n = read()\nlet y: x + 1..=n = read()\nlet a: [V, 0..n]Int = read()\ndomain V = 1..=n\nlet z: 0.. = read()\nprint(x, y, a, z)\n";
        let mut db = Database::new();
        db.set_file("a.cp", src);
        let analysis = db.analyze("a.cp").unwrap();
        let at = |text: &str, nth: usize| {
            let start = src.match_indices(text).nth(nth).unwrap().0 as u32;
            Span::new(start, start + text.len() as u32)
        };
        let facts = analysis.facts();
        let ends = |lo, hi, inclusive| Some(super::RangeEnds { lo, hi, inclusive });
        let x_plus_one = at("x + 1", 0);
        assert_eq!(
            facts.range_ends(at("x + 1..=n", 0)),
            ends(x_plus_one, at("n", 2), true)
        );
        assert_eq!(
            facts.range_ends(at("0..n", 0)),
            ends(at("0", 0), at("n", 1), false)
        );
        let index = at("0..n", 1);
        let index_ends = facts.range_ends(index).unwrap();
        assert_eq!(
            (index_ends.lo.start, index_ends.hi.end),
            (index.start, index.end)
        );
        assert_eq!(facts.range_ends(at("1..=n", 0)), None);
        assert_eq!(facts.range_ends(at("0..", 1)), None);
    }

    #[test]
    fn a_member_it_cannot_resolve_keeps_its_written_path() {
        let src = "let xs = [3, 1]\nprint(t.postorder, xs.first(x => x > 1), xs.len)\n";
        let mut db = Database::new();
        db.set_file("a.cp", src);
        let analysis = db.analyze("a.cp").unwrap();
        let at = |text: &str| {
            let start = src.find(text).unwrap() as u32;
            Span::new(start, start + text.len() as u32)
        };
        let facts = analysis.facts();
        assert_eq!(
            facts.unresolved(at("t.postorder")),
            Some(&["t".to_string(), "postorder".to_string()][..])
        );
        assert_eq!(facts.unresolved(at("xs.len")), None);
        assert_eq!(
            facts.unresolved(at("xs.first(x => x > 1)")),
            Some(&["first".to_string()][..])
        );
        let own = "fn f(x: Int) -> Int = x\nprint(f(1), [1].sum())\n";
        db.set_file("b.cp", own);
        let analysis = db.analyze("b.cp").unwrap();
        for call in ["f(1)", "[1].sum()", "print(f(1), [1].sum())"] {
            let start = own.find(call).unwrap() as u32;
            let span = Span::new(start, start + call.len() as u32);
            assert_eq!(analysis.facts().unresolved(span), None, "{call}");
        }
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
    fn a_builtin_member_is_told_from_a_programs_own() {
        let src = "struct Bag:\n    n: Int\n\nimpl Bag:\n    fn push(self, x: Int) -> Int = self.n + x\n\nvar xs = [3, 1, 2]\nxs.push(4)\nlet b = Bag(n: 1)\nlet c = b.push(2)\nlet n = xs.len\nlet ys = List.filled(2, 0)\nprint(c, n, ys)\n";
        let mut db = Database::new();
        db.set_file("a.cp", src);
        let analysis = db.analyze("a.cp").unwrap();
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let facts = analysis.facts();
        let callee = |text: &str| {
            let start = src.find(text).unwrap() as u32;
            let c = facts
                .callee(Span::new(start, start + text.len() as u32))
                .unwrap();
            (c.name.clone(), c.kind, c.builtin)
        };
        assert_eq!(
            callee("xs.push(4)"),
            ("push".into(), CalleeKind::Method, true)
        );
        assert_eq!(
            callee("b.push(2)"),
            ("push".into(), CalleeKind::Method, false)
        );
        assert_eq!(callee("xs.len"), ("len".into(), CalleeKind::Getter, true));
        assert_eq!(
            callee("List.filled(2, 0)"),
            ("filled".into(), CalleeKind::Method, true)
        );
    }

    #[test]
    fn a_member_of_an_imported_type_names_its_module() {
        let src = "var s = SortedBag<Int>()\ns.add(3)\nlet k = s.len\nvar h = MinHeap<Int>()\nh.push(-3)\nprint(k, h.pop()!)\n";
        let mut db = Database::new();
        db.set_file("a.cp", src);
        let analysis = db.analyze("a.cp").unwrap();
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let facts = analysis.facts();
        let callee = |text: &str| {
            let start = src.find(text).unwrap() as u32;
            let c = facts
                .callee(Span::new(start, start + text.len() as u32))
                .unwrap();
            (c.module.clone(), c.kind)
        };
        assert_eq!(callee("s.add(3)"), ("std.bag".into(), CalleeKind::Method));
        assert_eq!(callee("s.len"), ("std.bag".into(), CalleeKind::Getter));
        // `MinHeap` is std's name for its `Heap`.
        assert_eq!(
            callee("h.push(-3)"),
            ("std.heap".into(), CalleeKind::Method)
        );
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

//! 2-SAT on top of the standard library's strongly connected components.
use std.graph.{scc}

/// A variable or its negation.
pub struct Lit<X>(x: X, value: Bool):
    /// The opposite literal.
    pub @get fn flip(self) -> Lit<X> = Lit(x, not value)

/// A conjunction of two-literal clauses over the variables `X`.
/// `solve` is O(variables + clauses).
pub struct TwoSat<X>:
    /// Implications between literal nodes: node `2 * i` is "variable i is false", `2 * i + 1` "is true".
    out: List<List<Int>>

    pub fn new() -> TwoSat<X>:
        TwoSat([[] for _ in 0..2 * X.K])

    /// At least one of `a`, `b` holds.
    pub fn either(mut self, a: Lit<X>, b: Lit<X>):
        out[node(a.flip)].push(node(b))
        out[node(b.flip)].push(node(a))

    /// `a` holds.
    pub fn must(mut self, a: Lit<X>):
        self.either(a, a)

    /// If `a` holds then `b` holds.
    pub fn implies(mut self, a: Lit<X>, b: Lit<X>):
        self.either(a.flip, b)

    /// Exactly one of `a`, `b` holds.
    pub fn differ(mut self, a: Lit<X>, b: Lit<X>):
        self.either(a, b)
        self.either(a.flip, b.flip)

    /// An assignment satisfying every clause, or `none` if there is none.
    pub fn solve(self) -> ?[X]Bool:
        domain N = 0..2 * X.K
        let (C, comp) = scc::<N>(u => [(to: N(v)) for v in out[u.to_idx]])
        // components are in topological order: a literal is true when it comes after its negation
        for x in X:
            if comp[N(2 * x.to_idx)] == comp[N(2 * x.to_idx + 1)]: return none
        [X] x => comp[N(2 * x.to_idx + 1)] > comp[N(2 * x.to_idx)]

fn node<X>(l: Lit<X>) -> Int = 2 * l.x.to_idx + (if l.value: 1 else: 0)

@test
fn solutions_satisfy_every_clause_and_unsat_is_detected():
    domain X = 0..5
    for _ in 0..60:
        let clauses = [(Lit(X.nth(rand(0, 4)), rand(0, 1) == 1), Lit(X.nth(rand(0, 4)), rand(0, 1) == 1)) for _ in 0..rand(1, 12)]
        var s = TwoSat<X>()
        for (a, b) in clauses: s.either(a, b)
        // brute force over all 32 assignments
        var any = false
        for mask in 0..32:
            let val = [X] x => ((mask >> x.to_idx) & 1) == 1
            var ok = true
            for (a, b) in clauses:
                if val[a.x] != a.value and val[b.x] != b.value: ok = false
            if ok: any = true
        let got = s.solve()
        assert (got != none) == any
        if let val = got:
            for (a, b) in clauses: assert val[a.x] == a.value or val[b.x] == b.value

@test
fn the_clause_helpers():
    domain X = 1..=3
    var s = TwoSat<X>()
    let (a, b, c) = (Lit(X(1), true), Lit(X(2), true), Lit(X(3), true))
    s.must(a)
    s.implies(a, b.flip)
    s.differ(b, c)
    let val = s.solve()!
    assert [val[x] for x in X] == [true, false, true]
    s.must(c.flip)
    assert s.solve() == none

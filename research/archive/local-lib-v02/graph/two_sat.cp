//! 2-SAT solver over a typed variable domain `X`, backed by `std.graph.scc`.

/// A variable or its negation: `Lit(x, true)` is "x holds".
use std.graph.{scc}
pub struct Lit<X>(x: X, value: Bool):
    /// The opposite literal.
    pub @get fn flip(self) -> Lit<X> = Lit(x, not value)

/// A conjunction of two-literal clauses over the variables `X`.
/// `solve` is O(variables + clauses).
pub struct TwoSat<X>:
    /// Implications between literal nodes: node `2 * i` is "variable i is false", `2 * i + 1` "is true".
    imp: List<List<Int>>

    pub fn new() -> TwoSat<X>:
        TwoSat([[] for _ in 0..2 * X.K])

    /// At least one of `a`, `b` holds.
    pub fn add_or(mut self, a: Lit<X>, b: Lit<X>):
        imp[lit_node(a.flip)].push(lit_node(b))
        imp[lit_node(b.flip)].push(lit_node(a))

    /// `a` holds.
    pub fn must(mut self, a: Lit<X>):
        self.add_or(a, a)

    /// If `a` holds then `b` holds.
    pub fn implies(mut self, a: Lit<X>, b: Lit<X>):
        self.add_or(a.flip, b)

    /// An assignment satisfying every clause, or `none` if there is none.
    pub fn solve(self) -> ?[X]Bool:
        domain N = 0..2 * X.K
        let (C, comp) = scc::<N>(u => [N(v) for v in imp[u.to_idx]])
        // components are in topological order: a literal is true when it comes after its negation
        for x in X:
            if comp[N(2 * x.to_idx)] == comp[N(2 * x.to_idx + 1)]: return none
        [X] x => comp[N(2 * x.to_idx + 1)] > comp[N(2 * x.to_idx)]

fn lit_node<X>(l: Lit<X>) -> Int = 2 * l.x.to_idx + Int(l.value)

@test
fn two_sat_solutions_satisfy_every_clause_and_unsat_is_detected():
    domain X = 0..5
    for _ in 0..60:
        var sat = TwoSat<X>()
        var clauses: List<(Lit<X>, Lit<X>)> = []
        for _ in 0..rand(1, 12):
            let a = Lit(X.nth(rand(0, 4)), rand(0, 1) == 1)
            let b = Lit(X.nth(rand(0, 4)), rand(0, 1) == 1)
            sat.add_or(a, b)
            clauses.push((a, b))
        var possible = false
        for mask in 0..32:
            if all(
                (((mask >> c.0.x.to_idx) & 1) == 1) == c.0.value or (
                    ((mask >> c.1.x.to_idx) & 1) == 1
                ) == c.1.value for c in clauses
            ): possible = true
        let got = sat.solve()
        assert (got != none) == possible
        if got != none:
            for c in clauses: assert got![c.0.x] == c.0.value or got![c.1.x] == c.1.value
    var s = TwoSat<X>()
    s.must(Lit(X(0), true))
    s.implies(Lit(X(0), true), Lit(X(3), false))
    let fixed = s.solve()!
    assert fixed[X(0)] and not fixed[X(3)]

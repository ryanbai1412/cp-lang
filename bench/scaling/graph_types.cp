// n: 65536
// fixed: k
use std.graph.{Lit, TwoSat, Functional}
// TwoSat and Functional on n variables and vertices.
let n: Int = read()
domain V = 0..n
var s = 0
// Two clauses per variable, then a solve; X is V.
for _ in 0..64:
    let fresh = TwoSat<V>() // timed: graph.TwoSat.new
    s += Int(fresh.solve() != None)
for _ in 0..4:
    var ts = TwoSat<V>()
    for i in 0..n:
        let (x, y) = (V(i), V((i + 1) % n))
        ts.add_or(Lit(x, true), Lit(y, true)) // timed: graph.TwoSat.add_or
        ts.implies(Lit(x, false), Lit(y, true)) // timed: graph.TwoSat.implies
        if i % 4 == 0: ts.add_xor(Lit(x, true), Lit(V((i + 2) % n), true)) // timed: graph.TwoSat.add_xor
        if i % 4 == 1: ts.add_eq(Lit(x, true), Lit(V((i + 2) % n), false)) // timed: graph.TwoSat.add_eq
        if i % 64 == 3: ts.must(Lit(x, true)) // timed: graph.TwoSat.must
        if i % 4 == 2: ts.at_most_one([Lit(x, false), Lit(y, false), Lit(V((i + 2) % n), false), Lit(V((i + 3) % n), false)]) // timed: graph.TwoSat.at_most_one
    s += Int(ts.solve() != None) // timed: graph.TwoSat.solve
// A functional graph i -> i / 2: a forest of depth log n hanging off the self-loop at 0.
let nx = [V] v => V(v.ord / 2)
for _ in 0..64:
    let f = Functional(nx) // timed: graph.Functional.new
    s += f.cycles().len // timed: graph.Functional.cycles
let f = Functional(nx)
for j in 0..2 * n:
    let i = j % n
    let u = V(i)
    s += Int(f.on_cycle(u)) // timed: graph.Functional.on_cycle
    s += f.root_of(u).ord // timed: graph.Functional.root_of
    s += f.depth(u) // timed: graph.Functional.depth
    s += f.cycle_len(u) // timed: graph.Functional.cycle_len
    s += f.after(u, i).ord // timed: graph.Functional.after
    s += (f.jump_while(u, x => x.ord > i / 8) ?? u).ord // timed: graph.Functional.jump_while
    s += f.dist(u, V(i / 4)) ?? 0 // timed: graph.Functional.dist
print(s)

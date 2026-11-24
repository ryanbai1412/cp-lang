// expect: ok, output "1"
// rule: 04b.1.5
// A table over `a.keys` names one fresh domain however often its type is read, so a call
// that reads it while checking its argument agrees with the table it builds.
fn f(n: 1..=4, a: List<1..=n>, b: List<1..=n>) -> Int:
    domain V = 1..=n
    let g = Graph<V>.undirected([a.keys] e => (u: V(a[e]), v: V(b[e])))
    g.deg(V.first!)
print(f(3, [1, 2], [2, 3]))

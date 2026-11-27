// expect: R0201
// rule: R03.1.4
// Two domains' `0..=D.count` ends are each that domain's own `Domain.count`, not a spelling:
// a field and a local named `count` beside them change nothing.
struct Pair<A: Domain, B: Domain>:
    a: [A]0..=A.count
    b: [B]0..=B.count
    count: Int
fn make<A: Domain, B: Domain>(x: A, y: B) -> Pair<A, B>:
    let count = 100
    var a: [A]0..=A.count = 0
    var b: [B]0..=B.count = 0
    Pair(a: a, b: b, count: count)
domain V = 0..2
domain W = 0..5
var p = make(V(0), W(0))
p.b[W(1)] = 5
p.a[V(1)] = 2
print(p.a[V(1)], p.b[W(1)], p.count)
p.a[V(0)] = 3  // <- here

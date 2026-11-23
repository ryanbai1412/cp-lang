// expect: ok, output "6 6 7"
// rule: 04.2.11
// `[P]T` over `type P = (A, B)` is `[A, B]T`, so it binds `V = P` in a `[V]T` parameter, and `[P, P]T` binds `[V, V]T` with each `V` the whole cell, not `A` and `B` one by one.
domain A = 0..2
domain B = 0..3
type P = (A, B)
fn total<V>(t: [V]Int) -> Int:
    var s = 0
    for v in V: s += t[v]
    return s
fn diag<V>(t: [V, V]Int) -> Int:
    var s = 0
    for v in V: s += t[v, v]
    return s
var d: [P, P]?Int = [P, P] (u, v) => if u == v: Some(0) else: None
d[(A(0), B(0)), (A(1), B(1))] = Some(3)
d[(A(1), B(1)), (A(0), B(2))] = Some(4)
let closed = floyd_warshall(mut d)
print(total([P] p => 1), diag([P, P] (u, v) => if u == v: 1 else: 0), if closed: d[(A(0), B(0)), (A(0), B(2))]! else: -1)

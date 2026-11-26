// expect: ok, output "1 2 2\n2 0\n3"
// rule: R04.2.18, R04.4.25, R02.5.2
// An integer literal where a position of `V` is expected is that position: in a `let`, a call, a
// list, and a record's fields inside an array literal.
domain V = 0..3
domain E = 0..2
let x: V = 1
let xs: List<V> = [0, 2]
fn f(v: V) -> Int = v.ord
print(x, xs[1], f(2))
let es: [E](u: V, v: V) = [(u: 0, v: 2), (u: 2, v: 1)]
print(es[E(0)].v, es[E(1)].u.ord - 2)
print(es.len + 1)

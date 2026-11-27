// expect: ok, output "4 6\n3\n2"
// rule: R04.2.29
// `domain V = e` with an `Int` count `e` has positions `0..e`, whether `e` is a value or a
// literal, at the top level or in a body.
let n = 4
domain V = n
var a: [V]Int = 0
for v in V: a[v] = Int(v) * 2
print(V.count, a[V(3)])
domain W = 3
print(W.count)
fn half(m: 0..=10) -> Int:
    domain H = m / 2
    H.count
print(half(5))

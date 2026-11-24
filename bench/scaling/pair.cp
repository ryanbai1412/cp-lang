use std.pair.{symmetric}
let n: Int = read()
let m = n / 64
domain V = 0..m
var s = 0
for i in 0..8 * n:
    let p = Pair<V>(V(i % (m - 1)), V(m - 1)) // timed: pair.Pair.new
    if let q = Pair<V>.at(V(i % m), V(i * 3 % m)): s += Int(q.lo) // timed: pair.Pair.at
    s += Int(p.other(p.lo)) // timed: pair.Pair.other
var d: [Pair<V>]Int = 1
for t in 0..64:
    let g = symmetric<V, Int>(d, diag: t) // timed: pair.symmetric
    s += g[V(t % m), V(0)]
print(s)

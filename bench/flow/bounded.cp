// Max flow with lower bounds: 5000 vertices, 200000 random arcs with lo <= hi / 4,
// plus s -> v and v -> t for every v so a feasible flow exists.
use std.flow.{max_bounded_flow}

domain V = 0..5000
var seed = 1
fn next(n: Int) -> Int:
    seed = seed * 48271 % 2147483647
    seed % n

let (s, t) = (V.first, V.last)
var arcs = List<(u: V, v: V, lo: Int, hi: Int)>()
for _ in 0..200000:
    let u = next(V.K)
    let v = next(V.K)
    let hi = next(1000) + 1
    let lo = next(hi / 4 + 1)
    arcs.push((u: V(u), v: V(v), lo: lo, hi: hi))
for v in V:
    arcs.push((u: s, v: v, lo: 0, hi: 1000000))
    arcs.push((u: v, v: t, lo: 0, hi: 1000000))
let r = max_bounded_flow(arcs, s, t)!
var sum = 0
for e in 0..arcs.len: sum += r.flow[e]
print(r.value, sum)

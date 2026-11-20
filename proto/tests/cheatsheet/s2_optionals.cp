domain V = 0..3
var dist: [V]?Int = none
dist[V(1)] = 3
let d = dist[V(0)] ?? -1
let a: ?Int = none
let b: ?Int = 2
let best = min(a, b)
dist[V(1)] min= 2
dist[V(2)] min= 7
var ans = 9
ans min= dist[V(0)]
ans min= dist[V(2)]
let x = dist[V(2)]!
let both: ?Int = min(a, a)
var hi: ?Int = none
hi max= a
hi max= 5
hi max= b
fn maybe(v: V) -> ?Int:
    let y = dist[v]?
    return y + 1
let below = if let p = dist[V(1)]: p + 1 else: 0
print(d, dist[V(1)] ?? 3, best, dist[V(1)], ans, x, both, hi, maybe(V(1)) ?? 0, below - 2, none: "none")

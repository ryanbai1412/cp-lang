let (n, m, q): (1..=400, 0..=1e5, 1..=1e5) = read()
domain V = 1..=n
let es: [0..m](u: V, v: V, w: 1..=1e9) = read()
var d: [V, V]?Int = None
for e in es: d[e.u, e.v] min= e.w
assert floyd_warshall(mut d)
for _ in 0..q:
    let (u, v): (V, V) = read()
    print(d[u, v] ?? -1)

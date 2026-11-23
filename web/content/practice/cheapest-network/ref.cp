let (n, m): (1..=2e5, 0..=2e5) = read()
domain V = 1..=n
let es: [0..m](u: V, v: V, w: 1..=1e9) = read()
var d = Dsu<V>.new()
var total = 0
var used = 0
for e in es.sorted(by: e => e.w):
    if d.unite(e.u, e.v):
        total += e.w
        used += 1
print(if used == n - 1: total else: -1)

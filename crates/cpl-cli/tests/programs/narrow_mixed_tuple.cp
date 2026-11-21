// A tuple of positions and ranged Ints is narrowed field by field: the
// positions stay `int`, the ranged fields drop to 32 bits, and every read
// widens again before arithmetic.
let (n, m): (1..=10, 0..=10) = read()
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V, w: 0..=1_000_000_000) = read()
var total = 0
for e in E:
    total += edges[e].w * edges[e].w
var heavy: List<(u: V, v: V, w: 0..=1_000_000_000)> = []
for (u, v, w) in edges:
    if w > 5: heavy.push((u: u, v: v, w: w))
var deg: [V]Int = 0
for (u, v, _) in heavy:
    deg[u] += 1
    deg[v] += 1
var copy = edges
copy[E(0)] = (u: copy[E(0)].v, v: copy[E(0)].u, w: 7)
print(total, heavy.len, [deg[v] for v in V], copy[E(0)], edges[E(0)])

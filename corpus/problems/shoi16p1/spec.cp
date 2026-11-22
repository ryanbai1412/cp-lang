// shoi16p1: cheapest guards so every marked vertex is within k of one. need(u, d): the cheapest
// way to settle u's subtree where d >= 0 means a guard inside reaches d levels above u, and d < 0
// means the top -d levels are already covered from outside. Raising d only gets harder.
let (n, k) = read<(1..=500_000, 0..=20)>()
domain V = 1..=n
let cost = read<[V]Int>()
var marked: [V]Bool = false
let m = read<0..=n>()
for v in read<[m]V>():
    marked[v] = true
let t = Rooted(Graph.undirected(read<[n - 1](u: V, v: V)>()), root: V.first!)
@memo(array)
fn need(u: V, d: -k..=k) -> Int:
    let kids = t.children(u)
    var best = if d < k: need(u, d + 1) else: inf
    if d == k:
        best min= cost[u] + kids.map(v => need(v, -k)).sum()
    elif d >= 0:
        best min= kids.map(v => need(v, -d)).sum() + (kids.map(v => need(v, d + 1) - need(v, -d)).min() ?? inf)
    else:
        best min= kids.map(v => need(v, d + 1)).sum()
    if d == 0 and not marked[u]:
        best min= kids.map(v => need(v, 0)).sum()
    best
print(need(t.root, 0))

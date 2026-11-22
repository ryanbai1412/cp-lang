let (n, m) = read<(1..=2e5, 1..=2e5)>()
domain V = 1..=n
let roads = read<[m](u: V, v: V, r: 0..=1e9, p: 0..=1e9)>()
let doomed = V.mask(Graph.directed(roads).transpose().peel())
let g = Graph.directed(roads).only(v => not doomed[v])
let into = g.transpose()
var left = [V] u => g.deg(u)
var need = Settle<roads.keys, Int>(best: max)
for u in V:
    for a in g.out(u):
        need.offer(a.e, a.r)
var ans: [V]?Int = None
while let (road, x) = need.pop():
    let u = roads[road].u
    left[u] -= 1
    if left[u] == 0:
        ans[u] = x
        for a in into.out(u):
            need.offer(a.e, max(a.r, x - a.p))
print(ans, none: -1)

// cco21p4 Travelling Merchant: for each city, the least money that lets you trade forever.
// A city whose roads only lead to dead ends never qualifies; peel those off the reversed graph.
// Then settle answers from the largest requirement down: popping a road's need from a max-heap
// retires it, and the city whose last live road retires has its answer, which raises the need
// of every road into it to max(r, answer - p).
let (n, m) = read<(1..=2e5, 1..=2e5)>()
domain V = 1..=n
domain Road = [m]
let roads = read<[Road](u: V, v: V, r: 0..=1e9, p: 0..=1e9)>()
let doomed = V.mask(Graph.directed(roads).transpose().peel())
let g = Graph.directed(roads).only(v => not doomed[v])
let into = g.transpose()
var left = [V] u => g.deg(u)
var need = Settle<Road, Int>(best: max)
for u in V:
    for a in g.out(u):
        need.offer(a.e, a.r)
var ans: [V]?Int = none
while let (road, x) = need.pop():
    let u = roads[road].u
    left[u] -= 1
    if left[u] == 0:
        ans[u] = x
        for a in into.out(u):
            need.offer(a.e, max(a.r, x - a.p))
print(ans, none: -1)

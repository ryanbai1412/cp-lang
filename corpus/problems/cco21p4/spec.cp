// cco21p4 Travelling Merchant: for each city, the least money that lets you trade forever.
// A city whose roads only lead to dead ends never qualifies; peel those off the reversed graph.
// Then settle answers from the largest requirement down: popping a road's need from a max-heap
// retires it, and the city whose last live road retires has its answer, which raises the need
// of every road into it to max(r, answer - p).
let (n, m) = read<(1..=200_000, 1..=200_000)>()
domain V = 1..=n
let roads = read<[m](u: V, v: V, r: 0..=1_000_000_000, p: 0..=1_000_000_000)>()
let g = Graph.directed(roads)
let into = g.transpose()
var alive: [V]Bool = true
for v in peel(into):
    alive[v] = false
var left: [V]Int = [V] u => g.out(u).count(a => alive[a.to])
var retired: [roads.keys]Bool = false
var heap = MaxHeap(by: x => x.need)
for u in V if alive[u]:
    for a in g.out(u) if alive[a.to]:
        heap.push((need: a.r, road: a.e, from: u))
var ans: [V]?Int = none
while let (need, road, u) = heap.pop():
    if retired[road]:
        continue
    retired[road] = true
    left[u] -= 1
    if left[u] == 0:
        ans[u] = need
        for a in into.out(u) if alive[a.to]:
            heap.push((need: max(a.r, need - a.p), road: a.e, from: a.to))
print(ans.map(a => a ?? -1))

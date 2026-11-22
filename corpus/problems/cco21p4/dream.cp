// cco21p4 Travelling Merchant: for each city, the least money that lets you trade forever.
// A city that can only lead to dead ends never qualifies; peel those off the reversed graph.
// Then settle answers from the largest requirement down: popping an edge's requirement from a
// max-heap retires it, and the city whose last out-edge retires has its answer.
let (n, m) = read<(1..=200_000, 1..=200_000)>()
domain V = 1..=n
let all = read<[m](u: V, v: V, r: 0..=1_000_000_000, p: 0..=1_000_000_000)>()
let rev = Graph.directed(all.map(e => (u: e.v, v: e.u)))
var doomed: [V]Bool = false
for v in peel(rev):
    doomed[v] = true
let edges = all.filter(e => not doomed[e.u] and not doomed[e.v]).collect()
var left: [V]Int = 0
for e in edges:
    left[e.u] += 1
let into = buckets(edges.map(e => e.v))
var ans: [V]?Int = none
var retired: [edges.keys]Bool = false
var heap = MaxHeap(by: x => x.need)
for (i, e) in edges.items:
    heap.push((need: e.r, i: i))
while let (need, i) = heap.pop():
    if retired[i]:
        continue
    retired[i] = true
    let u = edges[i].u
    left[u] -= 1
    if left[u] == 0:
        ans[u] = need
        for j in into[u]:
            heap.push((need: max(edges[j].r, need - edges[j].p), i: j))
print(ans.map(a => a ?? -1))

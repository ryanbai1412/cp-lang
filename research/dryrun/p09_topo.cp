// CSES 1679 Course Schedule (variant): the lexicographically smallest order, or IMPOSSIBLE.
let (n, m): (1..=1e5, 1..=2e5) = read()
domain V = 1..=n
let reqs: [0..m](u: V, v: V) = read()
let g = Graph.directed(reqs)
var indeg: [V]Int = 0
for r in reqs: indeg[r.v] += 1
var ready = MinHeap<V>()
for v in V if indeg[v] == 0: ready.push(v)
var order: List<V> = []
while let u = ready.pop():
    order.push(u)
    for v in g[u]:
        indeg[v] -= 1
        if indeg[v] == 0: ready.push(v)
if order.len < n:
    print("IMPOSSIBLE")
else:
    print(order)

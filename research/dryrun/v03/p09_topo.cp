// CSES 1679 Course Schedule (variant): the lexicographically smallest order, or IMPOSSIBLE.
let (n, m): (1..=1e5, 1..=2e5) = read()
domain V = 1..=n
domain E = 0..m
let reqs = read<[E](u: V, v: V)>()
let g = Graph.directed(reqs)
var indeg: [V]Int = 0
for r in reqs: indeg[r.v] += 1
var ready = Heap<V>.new()
for v in V if indeg[v] == 0: ready.push(v)
var order: List<V> = []
while let u = ready.pop():
    order.push(u)
    for a in g.out(u):
        indeg[a.to] -= 1
        if indeg[a.to] == 0: ready.push(a.to)
if order.len < n: print("IMPOSSIBLE") else: print(order)

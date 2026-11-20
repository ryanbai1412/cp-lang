// Universal Cup 5 Stage 6 (Hyakumanben) G: vertices on or below a cycle cost
// nothing; every other vertex pays the fewest reversed edges needed to be
// reached from one of them. Print the total, or -1 if some vertex is cut off.
@!multitest
let (n, m): (1..=5e5, 0..=5e5) = read()
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V) = read()
let g = Graph.directed(edges)

// Peel vertices of in-degree zero; whatever keeps an incoming edge is free.
var indeg: [V]Int = 0
for e in edges: indeg[e.v] += 1
var peeled = [v for v in V if indeg[v] == 0]
var head = 0
while head < peeled.len:
    for a in g.adj(peeled[head]):
        indeg[a.to] -= 1
        if indeg[a.to] == 0: peeled.push(a.to)
    head += 1

// Walking an edge forwards is free and backwards costs one.
let both = Graph.undirected(edges)
let dist = dijkstra(both, [(v, 0) for v in V if indeg[v] > 0], a => if edges[a.e].v == a.to: 0 else: 1)
print(if any(d == none for d in dist): -1 else: sum(d! for d in dist))

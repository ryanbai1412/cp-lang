// expect: ok
// tool: fix
// rewritten: "domain V = 0..4\nlet es = [(u: V(0), v: V(1), w: 3), (u: V(1), v: V(2), w: 4), (u: V(0), v: V(3), w: 5)]\nlet adj = Bitset.undirected(es.filter(e => e.w % 2 == 1))\nprint(adj[V(0)].len, adj[V(3)].len)"
// `cpl fix` rewrites the removed `adjacency(edges, into: Bitset)` to `Bitset.undirected(edges)`,
// the same neighbour sets: each edge in both directions.
domain V = 0..4
let es = [(u: V(0), v: V(1), w: 3), (u: V(1), v: V(2), w: 4), (u: V(0), v: V(3), w: 5)]
let adj = adjacency(es.filter(e => e.w % 2 == 1), into: Bitset)
print(adj[V(0)].len, adj[V(3)].len)

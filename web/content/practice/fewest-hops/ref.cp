let (n, m): (1..=2e5, 0..=2e5) = read()
domain V = 1..=n
let es: [0..m](u: V, v: V) = read()
let p = bfs(Graph.undirected(es), [V(1)])
print(p[V(n)] ?? -1)

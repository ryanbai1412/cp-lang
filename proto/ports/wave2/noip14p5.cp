// NOIP 2014 Finding the Way: the shortest path from s to t that only visits
// vertices all of whose out-neighbours can reach t.

let (n, m): (1..=1e4, 0..=2e5) = read()
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V) = read()
let (s, t): (V, V) = read()
let roads = Graph.directed(edges)

let reaches = bfs(roads.transpose(), from: [t])
let safe = [V] u => all(reaches[v] != none for (v, _) in roads.out(u))
let dist = bfs(u => [v for (v, _) in roads.out(u) if safe[v]], from: if safe[s]: [s] else: [])
print(dist[t] ?? -1)

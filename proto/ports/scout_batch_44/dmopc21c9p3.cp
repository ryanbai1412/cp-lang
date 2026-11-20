// DMOPC '21 Contest 9 P3 — two walkers with step budgets; either one finishes
// alone, or they meet and swap targets (costing each one step).
// original: corpus/submissions/4564027.cpp
let (n, m): (Int, Int) = read()
domain V = 1..=n
domain E = 0..m
let g = Graph.undirected(read<[E](V, V)>())

// The graph is connected, so every distance is known.
let dist: [V][V]Int = [V] s => [V] v => bfs(g, from: [s])[v]!

let q: Int = read()
for _ in 0..q:
    let (a, b, c): (V, V, Int) = read()
    let (d, e, f): (V, V, Int) = read()
    let alone = dist[a][b] <= c or dist[d][e] <= f
    let swapped = min(dist[a][d] + dist[b][e], dist[a][e] + dist[b][d]) <= c + f - 2
    print(if alone or swapped: "YES" else: "NO")

// CF 1324F revised: vertices carry weights in -1e9..=1e9 instead of colours;
// maximise the weight of a connected subtree containing each vertex.
// Rerooted tree DP. A vertex keeps a
// neighbour's side only when it helps: best[v] = own[v] + sum max(0, side).
let n: 2..=2e5 = read()
domain V = 1..=n
let own = read<[V]-1e9..=1e9>()
let g = Graph.undirected(read<[n - 1](u: V, v: V)>())

let t = Rooted(g, root: V.first!)
let best = t.reroot(+, lift: (x, _, _) => max(x, 0), finish: (s, v) => s + own[v])
print(best)

domain V = 1..=3
let g = Graph.undirected(read<[2](u: V, v: V)>())
let t = Rooted(g, root: V.first!)
let f = t.reroot(max, lift: (x, _, _) => x.map(d => d + 1), finish: (s, v) => Some(s ?? 0))
print(f[V.first!]!)

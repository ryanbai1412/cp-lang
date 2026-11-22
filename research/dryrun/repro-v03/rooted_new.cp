// `Rooted(graph, root:)` as in std.tree's examples. `cpl check` accepts; `cpl run` rejects it.
domain V = 1..=3
domain E = 0..2
let edges = [E] e => (u: V(1), v: V(Int(e) + 2))
let t = Rooted(Graph.undirected(edges), root: V(1))
print(t.down(0, (d, v) => d + 1)[V(3)])

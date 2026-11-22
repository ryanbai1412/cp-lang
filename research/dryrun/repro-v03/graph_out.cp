// `Next.out(u)` on a `Graph.directed`. `cpl check` accepts; `cpl run` rejects the argument.
domain V = 1..=3
domain E = 0..2
let edges = [E] e => (u: V(1), v: V(Int(e) + 2))
let g = Graph.directed(edges)
for a in g.out(V(1)): print(a.to)

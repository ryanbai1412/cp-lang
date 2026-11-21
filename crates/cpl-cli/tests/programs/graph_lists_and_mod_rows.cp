// A graph built from a plain list of `(V, V)` keeps its vertex domain, and a
// list or array of `Mod`s prints on one line, as numbers do.
domain V = 0..4
let g = Graph.directed([(V(0), V(1)), (V(1), V(2)), (V(0), V(3))])
print(topo(g)!)
print(peel(g))
let u = Graph.undirected([(V(0), V(1)), (V(2), V(3))])
let dist = bfs(u, [V(0)])
print(dist.dist)
type Z = Mod<998244353>
let zs: List<Z> = [1, 2, 3]
print(zs)
print([Z(1), Z(2)], 5)
struct P(x: Int, y: Int)
print([P(1, 2), P(3, 4)])
print([(1, 2), (3, 4)])
domain I = 0..3
let a: [I]Mod<7> = [I] i => i.to_idx
let rows: [I, I]Mod<7> = [I, I] (i, j) => i.to_idx + j.to_idx
print(a)
print(rows)

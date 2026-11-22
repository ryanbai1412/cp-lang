// The vertex domain of a graph given as a lambda is what the lambda indexes with its parameter.
domain V = 0..5
let adj: [V]List<V> = [V] v => if v.ord + 1 < 5: [V(v.ord + 1)] else: []
print(peel(v => adj[v]))
print(topo(v => adj[v])!)
print(scc(v => adj[v]).comp)
type Z = Mod<7>
let x = Z(3)
print(x.inv(), x.inv, x.inv() * x)

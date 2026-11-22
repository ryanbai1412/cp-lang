// `Graph<V>.undirected(pairs)` is the fn `Graph.undirected(pairs)` is: type
// arguments on the receiver fill the fn's generics, never pick another fn.
domain V = 0..4
let pairs = [(V(0), V(1)), (V(1), V(2)), (V(0), V(3))]
let a = Graph.undirected(pairs)
let b = Graph<V>.undirected(pairs)
for v in V: print(a.deg(v), b.deg(v))
print(topo(Graph<V>.directed(pairs))!)
let w = Graph<V>.undirected([(V(0), V(1), 5), (V(1), V(2), 7)])
for (t, c) in w.out(V(1)): print(t, c)

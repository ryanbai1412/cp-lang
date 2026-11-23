// expect: ok, output "YES YES NO NO"
domain V = 0..4
var g = FlowGraph<V>()
g.add(V(0), V(1), Some(4))
g.add(V(1), V(2), Some(1))
g.add(V(2), V(3), Some(9))
g.max_flow(V(0), V(3))
let c = g.min_cut(V(0))
print(yes(c[V(0)]), yes(c[V(1)]), yes(c[V(2)]), yes(c[V(3)]))

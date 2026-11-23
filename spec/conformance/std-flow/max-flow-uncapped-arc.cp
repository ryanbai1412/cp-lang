// expect: ok, output "5"
domain V = 0..3
var g = FlowGraph<V>()
g.add(V(0), V(1), None)
g.add(V(1), V(2), Some(5))
print(g.max_flow(V(0), V(2)))

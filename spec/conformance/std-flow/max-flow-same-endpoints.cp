// expect: R0401 (checked)
domain V = 0..2
var g = FlowGraph<V>()
print(g.max_flow(V(0), V(0))) // <- here

// expect: ok, output "3 6"
use std.flow.{FlowGraph}
domain V = 0..2
var g = FlowGraph<V>()
let a = g.add_both(V(0), V(1), 3)
print(g.max_flow(V(1), V(0)), g.residual(a))

// expect: ok, output "0 0"
use std.flow.{FlowGraph}
domain V = 0..3
var g = FlowGraph<V>()
let a = g.add(V(0), V(1), Some(7))
print(g.max_flow(V(0), V(2)), g.flow(a))

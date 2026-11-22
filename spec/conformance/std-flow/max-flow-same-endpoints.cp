// expect: R0401 at 4 (checked)
use std.flow.{FlowGraph}
domain V = 0..2
var g = FlowGraph<V>()
print(g.max_flow(V(0), V(0)))

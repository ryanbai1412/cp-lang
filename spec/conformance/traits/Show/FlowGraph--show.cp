// expect: ok, output "4\n0->1 2/3 0->2 2/2 1->3 2/2 2->3 2/5"
use std.flow.{FlowGraph}
var g = FlowGraph<0..4>()
g.add(0, 1, Some(3))
g.add(0, 2, Some(2))
g.add(1, 3, Some(2))
g.add(2, 3, Some(5))
print(g.max_flow(0, 3))
print(g)

// expect: ok, output "5 0 3"
use std.flow.{FlowGraph}
domain V = 0..4
var g = FlowGraph<V>()
let a = g.add(V(0), V(1), Some(3))
g.add(V(0), V(2), Some(2))
g.add(V(1), V(2), Some(1))
g.add(V(1), V(3), Some(2))
g.add(V(2), V(3), Some(3))
print(g.push_relabel(V(0), V(3)), Int(g.source(a)), Int(g.target(a)) + 2)

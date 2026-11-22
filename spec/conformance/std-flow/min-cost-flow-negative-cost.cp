// expect: ok, output "2 -4 -2"
use std.flow.{CostFlow}
domain V = 0..3
var g = CostFlow<V>()
let a = g.add(V(0), V(1), Some(2), -3)
g.add(V(1), V(2), Some(2), 1)
let r = g.min_cost_flow(V(0), V(2))
print(r.flow, r.cost, g.cost(a) + 1)

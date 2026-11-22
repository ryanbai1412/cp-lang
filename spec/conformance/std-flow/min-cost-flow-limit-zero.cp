// expect: ok, output "0 0"
use std.flow.{CostFlow}
domain V = 0..2
var g = CostFlow<V>()
g.add(V(0), V(1), Some(4), 3)
let r = g.min_cost_flow(V(0), V(1), limit: Some(0))
print(r.flow, r.cost)

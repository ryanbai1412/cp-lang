// expect: ok, output "0->1 1/2 @4 1->2 1/1 @1"
use std.flow.{CostFlow}
var g = CostFlow<0..3>()
g.add(0, 1, Some(2), 4)
g.add(1, 2, Some(1), 1)
g.min_cost_flow(0, 2)
print(g)

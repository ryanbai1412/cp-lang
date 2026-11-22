// expect: ok, output "0"
use std.flow.{CostFlow}
domain V = 0..2
var g = CostFlow<V>()
g.add(V(0), V(1), Some(1), 2)
print(g.cancel_cycles())

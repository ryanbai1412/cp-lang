// expect: ok, output "-1 1 1"
// rule: 08.1.1
use std.flow.{CostFlow}
domain V = 0..2
var g = CostFlow<V>()
let a = g.add(V(0), V(1), Some(1), -2)
let b = g.add(V(1), V(0), Some(1), 1)
print(g.cancel_cycles(), g.flow(a), g.flow(b))

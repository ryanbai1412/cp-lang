// expect: ok, output "YES YES"
use std.graph.{closure}
domain V = 0..2
let d: [V, V]?Int = None
let c = d.closure()!
print(yes(c[0, 0] == Some(0)), yes(c[0, 1] == None))

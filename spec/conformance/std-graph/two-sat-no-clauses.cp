// expect: ok, output "YES"
use std.graph.{TwoSat}
domain X = 0..3
let s = TwoSat<X>()
print(yes(s.solve() != None))

// expect: ok, output "YES YES"
use std.graph.{johnson}
domain V = 0..2
domain E = 0..0
let es: [E](u: V, v: V, w: Int) = []
let d = johnson(Graph.directed(es))!
print(yes(d[0, 0] == Some(0)), yes(d[0, 1] == None))

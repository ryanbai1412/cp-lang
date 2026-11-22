// expect: ok, output "YES"
use std.graph.{dag_fold}
domain V = 0..1
let g: [V]List<V> = [[0]]
print(yes(dag_fold(g, init: v => 0, step: (x, a) => x + 1, op: max) == None))

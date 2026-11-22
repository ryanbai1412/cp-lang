// expect: ok, output "YES"
use std.graph.{dag_fold}
domain V = 0..1
let g: [V]List<V> = [[]]
print(yes(dag_fold(g, init: v => 0, step: (x, a) => x + 1, op: max) == Some([0])))

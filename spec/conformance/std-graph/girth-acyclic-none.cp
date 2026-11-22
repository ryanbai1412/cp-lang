// expect: ok, output "-1 2"
use std.graph.{girth}
domain V = 0..3
let dag: [V]List<V> = [[V(1)], [V(2)], []]
let two: [V]List<V> = [[V(1)], [V(0)], []]
print(girth(dag, directed: true) ?? -1, girth(two, directed: true) ?? -1)

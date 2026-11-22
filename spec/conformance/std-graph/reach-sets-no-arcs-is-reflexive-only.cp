// expect: ok, output "YES NO"
use std.graph.{reach_sets}
domain V = 0..2
let g: [V]List<V> = [[], []]
let r = reach_sets(g)
print(yes(r[0].contains(0)), yes(r[0].contains(1)))

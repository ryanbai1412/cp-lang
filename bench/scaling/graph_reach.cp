// n: 1024
use std.graph.{reach_sets}
// Reachability bitsets of a DAG whose every vertex reaches all later ones (arcs i -> i+1 and
// i -> i+3), so every set is dense.
let n: Int = read()
domain V = 0..n
let dag = [V] v => [V(u) for u in [v.ord + 1, v.ord + 3] if u < n]
var s = 0
for _ in 0..32:
    s += reach_sets(dag)[V(0)].count() // timed: graph.reach_sets
print(s)

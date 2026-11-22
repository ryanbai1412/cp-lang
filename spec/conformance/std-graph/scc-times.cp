// expect: ok, output "2 2 2 3 -1"
// rule: std.graph.scc_times

use std.graph.{scc_times}

domain V = 0..3
domain E = 0..4
let es: [E](u: V, v: V) = [(u: V(0), v: V(1)), (u: V(1), v: V(2)), (u: V(2), v: V(0)), (u: V(2), v: V(1))]
let t = scc_times(es)

domain V2 = 0..2
domain E2 = 0..1
let es2: [E2](u: V2, v: V2) = [(u: V2(0), v: V2(1))]
let t2 = scc_times(es2)

print(t[E(0)] ?? -1, t[E(1)] ?? -1, t[E(2)] ?? -1, t[E(3)] ?? -1, t2[E2(0)] ?? -1)

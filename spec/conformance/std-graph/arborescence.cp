// expect: ok, output "2 2 1 6 2 3 0"
// rule: std.graph.arborescence

use std.graph.{arborescence}

domain V = 0..3
domain E = 0..4
let es: [E](u: V, v: V, w: Int) = [(u: V(0), v: V(1), w: 5), (u: V(0), v: V(2), w: 1), (u: V(2), v: V(1), w: 1), (u: V(1), v: V(2), w: 1)]
let r1 = arborescence(es, root: V(0))!

let es_cyc: [E](u: V, v: V, w: Int) = [(u: V(0), v: V(1), w: 10), (u: V(1), v: V(2), w: 1), (u: V(2), v: V(1), w: 1), (u: V(0), v: V(2), w: 5)]
let r2 = arborescence(es_cyc, root: V(0))!

domain V2 = 0..2
domain E2 = 0..1
let es_no: [E2](u: V2, v: V2, w: Int) = [(u: V2(1), v: V2(0), w: 1)]
let r3 = arborescence(es_no, root: V2(0))
let has3 = if let _ = r3: 1 else: 0

print(r1.total, r1.parent[V(1)]!, r1.parent[V(2)]!, r2.total, r2.parent[V(1)]!, r2.parent[V(2)]!, has3)

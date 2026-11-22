// expect: ok, output "5 1 0 1 3 2"
// rule: std.graph.boruvka

use std.graph.{boruvka}

domain V = 0..2
fn pick1(comp: [V]Int) -> List<?(u: V, v: V, w: Int)>:
    [Some((u: V(0), v: V(1), w: 5)), Some((u: V(0), v: V(1), w: 5))]
let r1 = boruvka(pick1)

domain V2 = 0..3
fn pick2(comp: [V2]Int) -> List<?(u: V2, v: V2, w: Int)>:
    if comp[V2(0)] != comp[V2(1)]:
        [Some((u: V2(0), v: V2(1), w: 1)), Some((u: V2(0), v: V2(1), w: 1)), Some((u: V2(1), v: V2(2), w: 2))]
    else:
        [Some((u: V2(1), v: V2(2), w: 2)), Some((u: V2(1), v: V2(2), w: 2))]
let r2 = boruvka(pick2)

print(r1.total, r1.edges.len, r1.edges[0].u, r1.edges[0].v, r2.total, r2.edges.len)

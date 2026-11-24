// n: 128
use std.graph.{Graph, bellman_ford, closure, floyd_warshall, girth, johnson, min_mean_cycle, negative_cycle, prim}
// All-pairs and n m algorithms on a sparse graph (arcs i -> i+1 and i -> i+3 mod n) and its
// weight matrix.
let n: Int = read()
domain V = 0..n
domain E = 0..2 * n
let es: [E](u: V, v: V) = [E] e => (u: V(e.ord % n), v: V((e.ord % n + 1 + 2 * (e.ord / n)) % n))
let wes: [E](u: V, v: V, w: Int) = [E] e => (u: es[e].u, v: es[e].v, w: e.ord % 10 + 1)
let cyc = [V] v => [V((v.ord + 1) % n), V((v.ord + 3) % n)]
let gd = Graph.directed(es)
let gw = Graph.directed(wes)
// The worst case for the n m relaxations: a chain n-1 -> ... -> 0 of weight -1 that a scan
// in vertex order advances by one arc per round, closed by 0 -> n-1 and padded with heavy arcs
// that never improve a distance.
let slow: [E](u: V, v: V, w: Int) = [E] e => if e.ord < n - 1: (u: V(e.ord + 1), v: V(e.ord), w: -1) elif e.ord == n - 1: (u: V(0), v: V(n - 1), w: 10 * n) else: (u: V(e.ord - n), v: V((e.ord * 7919) % n), w: 10 * n)
let gs = Graph.directed(slow)
var s = 0
for _ in 0..512:
    s += gw.matrix()[V(0), V(1)] ?? 0 // timed: graph.Graph.matrix
for _ in 0..128:
    let d = gw.matrix()
    s += (closure(d)!)[V(0), V(n - 1)] ?? 0 // timed: graph.closure
    var fw = d.clone()
    s += Int(floyd_warshall(mut fw)) // timed: graph.floyd_warshall
    s += (prim(d)!).total // timed: graph.prim
    s += gd.hops()[V(0), V(n - 1)] // timed: graph.Graph.hops
for _ in 0..64:
    s += (bellman_ford(gs, from: [(V(n - 1), 0)])!).get(V(n - 1)) ?? 0 // timed: graph.bellman_ford
    s += girth(cyc, directed: true) ?? 0 // timed: graph.girth
    s += (johnson(gw)!)[V(0), V(n - 1)] ?? 0 // timed: graph.johnson
    s += (min_mean_cycle(gs)!).len // timed: graph.min_mean_cycle
    s += (negative_cycle(gs) ?? []).len // timed: graph.negative_cycle
print(s)

// n: 65536
// fixed: k
use std.graph.{IncrementalBridges}
// IncrementalBridges on n vertices.
let n: Int = read()
domain V = 0..n
var s = 0
for _ in 0..16:
    var nb = IncrementalBridges<V>() // timed: graph.IncrementalBridges.new
    nb.add(V(0), V(1))
// Bridges over a path, then edges that close cycles of every length.
for _ in 0..4:
    var b = IncrementalBridges<V>()
    for i in 1..n:
        b.add(V(i - 1), V(i)) // timed: graph.IncrementalBridges.add
    for i in 0..n / 8:
        b.add(V((i * 7919) % n), V((i * 104729 + 5) % n)) // timed: graph.IncrementalBridges.add
    for j in 0..2 * n:
        let i = j % n
        s += Int(b.same(V(i), V((i * 31) % n))) // timed: graph.IncrementalBridges.same
        s += Int(b.two_edge_same(V(i), V((i * 31) % n))) // timed: graph.IncrementalBridges.two_edge_same
print(s)

// expect: ok, output "0 1 2 0 1 0 1"
// rule: std.graph.IncrementalBridges

use std.graph.{IncrementalBridges}

domain V = 0..3
var b = IncrementalBridges<V>()
let s0 = if b.same(V(0), V(1)): 1 else: 0
b.add(V(0), V(1))
b.add(V(1), V(2))
let s1 = if b.same(V(0), V(2)): 1 else: 0
let br1 = b.bridges
let te1 = if b.two_edge_same(V(0), V(2)): 1 else: 0
b.add(V(2), V(0))
let te2 = if b.two_edge_same(V(0), V(2)): 1 else: 0
let br2 = b.bridges

domain V2 = 0..2
var b2 = IncrementalBridges<V2>()
b2.add(V2(0), V2(1))
b2.add(V2(0), V2(1))
let te3 = if b2.two_edge_same(V2(0), V2(1)): 1 else: 0

print(s0, s1, br1, te1, te2, br2, te3)

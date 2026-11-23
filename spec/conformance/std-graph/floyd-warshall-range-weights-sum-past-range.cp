// expect: ok, output "2000000000"
// A path length is an `Int`: sums of range-typed weights may pass the range (decision 313; rebuild round 6 H-FW-WEIGHT-WIDTH).
domain V = 0..3
domain E = 0..2
let es: [E](u: V, v: V, w: 1..=1000000000) = [(u: 0, v: 1, w: 1000000000), (u: 1, v: 2, w: 1000000000)]
var d = Graph.directed(es).matrix()
let ok = floyd_warshall(mut d)
assert ok
print(d[0, 2]!)

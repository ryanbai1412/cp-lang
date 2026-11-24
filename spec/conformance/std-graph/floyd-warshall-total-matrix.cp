// expect: ok, output "YES 2 0 NO 4"
// A matrix with every entry `Some` takes the plain-weight path; its answers,
// negative cycles and other semirings match the optional path's.
domain V = 0..3
var d: [V, V]?Int = [V, V] (u, v) => Some(5)
d[V(0), V(1)] = Some(1)
d[V(1), V(2)] = Some(1)
let ok = floyd_warshall(mut d)
var n: [V, V]?Int = [V, V] (u, v) => Some(-1)
var w: [V, V]?Int = [V, V] (u, v) => Some(1)
w[V(0), V(1)] = Some(4)
w[V(1), V(2)] = Some(4)
let wide = w.closure(over: Semiring(max, min))!
print(yes(ok), d[V(0), V(2)]!, d[V(1), V(1)]!, yes(floyd_warshall(mut n)), wide[V(0), V(2)]!)

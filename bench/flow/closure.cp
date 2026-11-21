// Maximum closure: 50000 vertices with profits in [-10^6, 10^6], 300000 random needs.
use std.flow.{max_closure}

domain V = 0..50000
var seed = 1
fn next(n: Int) -> Int:
    seed = seed * 48271 % 2147483647
    seed % n

var profit: [V]Int = 0
for v in V: profit[v] = next(2000001) - 1000000
var needs = List<(u: V, v: V)>()
for _ in 0..300000:
    let u = next(V.K)
    let v = next(V.K)
    needs.push((u: V(u), v: V(v)))
let (value, take) = max_closure(profit, needs)
print(value, [v for v in V if take[v]].len)

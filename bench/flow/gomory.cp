// Gomory-Hu (Gusfield): 500 vertices, 5000 random undirected edges, weights up to 100.
use std.flow.{gomory_hu}

domain V = 0..500
var seed = 1
fn next(n: Int) -> Int:
    seed = seed * 48271 % 2147483647
    seed % n

var edges = List<(u: V, v: V, w: Int)>()
for _ in 0..5000:
    let u = next(V.K)
    let v = next(V.K)
    edges.push((u: V(u), v: V(v), w: next(100) + 1))
var sum = 0
for e in gomory_hu(edges): sum += e.w
print(sum)

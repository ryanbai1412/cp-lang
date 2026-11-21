// Weighted blossom: 800 vertices, 160000 random undirected edges, weights below 10^6.
use std.flow.{max_weight_general_matching}

domain V = 0..800
domain E = 0..160000
var seed = 1
fn next(n: Int) -> Int:
    seed = seed * 48271 % 2147483647
    seed % n

let edges = [E] _ => (u: V(next(V.K)), v: V(next(V.K)), w: next(1000000))
print(max_weight_general_matching(edges).total)

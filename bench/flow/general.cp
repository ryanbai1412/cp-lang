// Edmonds blossom: 10000 vertices, 15000 random undirected edges.
use std.flow.{general_matching}

domain V = 0..10000
var seed = 1
fn next(n: Int) -> Int:
    seed = seed * 48271 % 2147483647
    seed % n

var adj: [V]List<V> = []
for _ in 0..15000:
    let u = next(V.K)
    let v = next(V.K)
    adj[V(u)].push(V(v))
    adj[V(v)].push(V(u))
print(general_matching(adj).size)

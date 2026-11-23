// Hopcroft-Karp: 100000 + 100000 vertices, 400000 random edges.
use std.flow.{max_matching}

domain L = 0..100000
domain R = 0..100000
var seed = 1
fn next(n: Int) -> Int:
    seed = seed * 48271 % 2147483647
    seed % n

var adj: [L]List<R> = []
for _ in 0..400000:
    let u = next(L.count)
    adj[L(u)].push(R(next(R.count)))
print(max_matching(adj).size)

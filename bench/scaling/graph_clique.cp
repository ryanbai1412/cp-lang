// n: 64
// let: n = 3 \log n
use std.graph.{max_clique}
use std.bitset.{Bitset}
// Moon-Moser graphs on 3 log2(n) vertices, complete multipartite with parts of 3: they have
// 3^(n/3) maximal cliques, the worst case for Bron-Kerbosch.
let n: Int = read()
var lg = 0
while 1 << (lg + 1) <= n: lg += 1
domain V = 0..3 * lg
let adj = [V] v => Bitset.of([u for u in V if u.ord / 3 != v.ord / 3])
var s = 0
for _ in 0..24:
    s += max_clique(adj).len // timed: graph.max_clique
print(s)

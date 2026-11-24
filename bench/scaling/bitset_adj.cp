// n: 2048
use std.bitset.{Bitset}
let n: Int = read()
domain D = 0..n
let edges = [(u: D(i), v: D(i * 7 % n)) for i in 0..n]
var acc = 0
for _ in 0..128:
    let adj = Bitset<D>.undirected(edges) // timed: bitset.Bitset.undirected
    acc += adj[D(n - 1)].len
print(acc)

// A permutation and a tree read by their stdlib types.
use std.graph.{Tree}
use std.seq.{Perm}

let n: 1..=10 = read()
domain V = 1..=n
let p: Perm<V> = read()
let t: Tree<V> = read()
print(t.len, p[V.first!].label, p[V.last!].label)
for (u, v) in t: print(u.label, v.label)

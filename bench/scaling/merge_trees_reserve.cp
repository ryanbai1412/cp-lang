// n: 512
// reserve moves the nodes already in the pool, one per `set` into an empty tree. At n = 512 the
// copy and the reserved block stay under 128 KB at 4n too, so malloc serves both from the heap.
use std.segtree.{MergeTree, MergeTrees}

let n: Int = read()
domain V = 0..n
var s = 0
for _ in 0..8000:
    var p = MergeTrees<V, Int>()
    for i in 0..n:
        var t = MergeTree<V>()
        p.set(mut t, V(i * 7919 % n), i)
        s += p.total(t)
    p.reserve(sets: n / 8)  // timed: segtree.MergeTrees.reserve
print(s)

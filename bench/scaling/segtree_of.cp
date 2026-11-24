use std.segtree.{SegTree}
let n: Int = read()
domain V = 0..n
let reps = 1 << 17
var s = 0

let f: V -> Int = v => v.ord % 7 // within: segtree.SegTree.of
for _ in 0..200:
    s += SegTree<V, Int>.of(max, f).all // timed: segtree.SegTree.of
var u = SegTree<V, Int>.of(max, f)
for i in 0..reps:
    u.refresh(V(i * 29 % n)) // timed: segtree.SegTree.refresh
print(s % 1000000007)

use std.segtree.{SegTree}
let n: Int = read()
domain V = 0..n
let reps = 1 << 17
var s = 0
let a: [V]Int = [V] v => (v.ord * 7919) % n + 1

var w = SegTree(+, a, act: add)
for i in 0..reps:
    let l = i * 7919 % n
    w.apply(V(l)..V(l + (n - l) / 2), 1) // timed: segtree.SegTree.apply
    w.apply_all(2) // timed: segtree.SegTree.apply_all
print(s % 1000000007)

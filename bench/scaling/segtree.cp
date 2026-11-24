use std.segtree.{SegTree}
let n: Int = read()
domain V = 0..n
let reps = 1 << 17
var s = 0
let a: [V]Int = [V] v => (v.ord * 7919) % n + 1

for _ in 0..200:
    s += SegTree(+, a).all // timed: segtree.SegTree.new
var t = SegTree(+, a)
for i in 0..reps:
    let l = i * 7919 % n
    let r = l + (n - l) / 2
    s += t.fold(V(l)..V(r)) // timed: segtree.SegTree.fold
    s += t.try_fold(V(l)..V(r)) ?? 0 // timed: segtree.SegTree.try_fold
    s += t.all // timed: segtree.SegTree.all
    s += t.monoid.id // timed: segtree.SegTree.monoid
for _ in 0..400:
    t.clear(fill: 1) // timed: segtree.SegTree.clear
print(s % 1000000007)

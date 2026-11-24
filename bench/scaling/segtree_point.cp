use std.segtree.{SegTree}
let n: Int = read()
domain V = 0..n
let reps = 1 << 17
var s = 0
let a: [V]Int = [V] v => (v.ord * 7919) % n + 1

var t = SegTree(+, a)
let tenfold: Int -> Int = x => x % 1000 + 1 // within: segtree.SegTree.update
for i in 0..reps:
    s += t.get(V(i * 13 % n)) // timed: segtree.SegTree.get
    t.set(V(i * 17 % n), i % 9) // timed: segtree.SegTree.set
    t.update(V(i * 19 % n), tenfold) // timed: segtree.SegTree.update
    t.at(V(i * 23 % n)) += 1 // timed: segtree.SegTree.at
print(s % 1000000007)

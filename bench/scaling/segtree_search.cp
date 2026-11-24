use std.segtree.{SegTree}
let n: Int = read()
domain V = 0..n
let reps = 1 << 17
var s = 0
let a: [V]Int = [V] v => (v.ord * 7919) % n + 1

var t = SegTree(+, a)
let small_r: Int -> Bool = x => x <= n * 4 // within: segtree.SegTree.right_while
let small_l: Int -> Bool = x => x <= n * 4 // within: segtree.SegTree.left_while
for i in 0..reps:
    let l = i * 7919 % n
    let r = l + (n - l) / 2
    s += t.right_while(V.cuts.nth(l), small_r).ord // timed: segtree.SegTree.right_while
    s += t.left_while(V.cuts.nth(r), small_l).ord // timed: segtree.SegTree.left_while
print(s % 1000000007)

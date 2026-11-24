use std.segtree.{MergeSeg, MergeSegs}
let n: Int = read()
domain V = 0..n
var s = 0

let even: V -> Bool = x => x.ord % 2 == 0 // within: segtree.MergeSegs.first_alive
for _ in 0..8:
    var p = MergeSegs<V>()
    var sets: List<MergeSeg<V>> = []
    for i in 0..n:
        sets.push(p.singleton(V(i * 7919 % n))) // timed: segtree.MergeSegs.singleton
    var m = sets[0]
    for i in 1..n:
        m = p.merge(m, sets[i]) // timed: segtree.MergeSegs.merge
    for i in 0..n:
        s += (p.first_ge(m, V(i * 13 % n)) ?? V(0)).ord // timed: segtree.MergeSegs.first_ge
        if p.any_in(m, V(i / 2)..V(i / 2 + 3)): s += 1 // timed: segtree.MergeSegs.any_in
    for i in 0..n:
        s += (p.first_alive(mut m, even) ?? V(0)).ord // timed: segtree.MergeSegs.first_alive
print(s % 1000000007)

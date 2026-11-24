use std.segtree.{SegTree, SegTree2D, MergeSortTree, WaveletMatrix, DynamicOrderTree, MergeSeg, MergeSegs, MergeTree, MergeTrees}
let n: Int = read()
domain V = 0..n
let reps = 1 << 17
var s = 0
let a: [V]Int = [V] v => (v.ord * 7919) % n + 1

for _ in 0..200:
    s += SegTree(+, a).all // timed: segtree.SegTree.new
let f: V -> Int = v => v.ord % 7 // within: segtree.SegTree.of
for _ in 0..200:
    s += SegTree<V, Int>.of(max, f).all // timed: segtree.SegTree.of
var t = SegTree(+, a)
let small_r: Int -> Bool = x => x <= n * 4 // within: segtree.SegTree.right_while
let small_l: Int -> Bool = x => x <= n * 4 // within: segtree.SegTree.left_while
let tenfold: Int -> Int = x => x % 1000 + 1 // within: segtree.SegTree.update
for i in 0..reps:
    let l = i * 7919 % n
    let r = l + (n - l) / 2
    s += t.fold(V(l)..V(r)) // timed: segtree.SegTree.fold
    s += t.try_fold(V(l)..V(r)) ?? 0 // timed: segtree.SegTree.try_fold
    s += t.get(V(i * 13 % n)) // timed: segtree.SegTree.get
    t.set(V(i * 17 % n), i % 9) // timed: segtree.SegTree.set
    t.update(V(i * 19 % n), tenfold) // timed: segtree.SegTree.update
    t.at(V(i * 23 % n)) += 1 // timed: segtree.SegTree.at
    s += t.all // timed: segtree.SegTree.all
    s += t.monoid.id // timed: segtree.SegTree.monoid
    s += t.right_while(V.cuts.nth(l), small_r).ord // timed: segtree.SegTree.right_while
    s += t.left_while(V.cuts.nth(r), small_l).ord // timed: segtree.SegTree.left_while
    s += t.walk(V(l)..V(r)).collect().len // timed: segtree.SegTree.walk
for _ in 0..400:
    t.clear(fill: 1) // timed: segtree.SegTree.clear
var u = SegTree<V, Int>.of(max, f)
for i in 0..reps:
    u.refresh(V(i * 29 % n)) // timed: segtree.SegTree.refresh
var w = SegTree(+, a, act: add)
for i in 0..reps:
    let l = i * 7919 % n
    w.apply(V(l)..V(l + (n - l) / 2), 1) // timed: segtree.SegTree.apply
    w.apply_all(2) // timed: segtree.SegTree.apply_all

let r2 = isqrt(n)
domain R = 0..r2
domain C = 0..r2
for _ in 0..100:
    s += SegTree2D<R, C, Int>(+, fill: 1).all // timed: segtree.SegTree2D.new [let: R = \sqrt n; C = \sqrt n]
var g = SegTree2D<R, C, Int>(+, fill: 1)
let inc: Int -> Int = x => x + 1 // within: segtree.SegTree2D.update
for i in 0..reps:
    let (x, y) = (i * 7919 % r2, i * 13 % r2)
    g.set(R(x), C(y), i % 5) // timed: segtree.SegTree2D.set [let: R = \sqrt n; C = \sqrt n]
    g.update(R(y), C(x), inc) // timed: segtree.SegTree2D.update [let: R = \sqrt n; C = \sqrt n]
    s += g.get(R(x), C(x)) // timed: segtree.SegTree2D.get [let: R = \sqrt n; C = \sqrt n]
    s += g.fold(R(x / 2)..R(x / 2 + r2 / 2), C(y / 2)..C(y / 2 + r2 / 2)) // timed: segtree.SegTree2D.fold [let: R = \sqrt n; C = \sqrt n]
    s += g.all // timed: segtree.SegTree2D.all [let: R = \sqrt n; C = \sqrt n]

for _ in 0..10:
    s += MergeSortTree(a).count_lt(V(0)..V(n - 1), n / 2) // timed: segtree.MergeSortTree.new
let mt = MergeSortTree(a)
for i in 0..reps / 4:
    let l = i * 7919 % n
    let rg = V(l / 2)..V(l / 2 + n / 2)
    s += mt.count_lt(rg, l) // timed: segtree.MergeSortTree.count_lt
    s += mt.count_le(rg, l) // timed: segtree.MergeSortTree.count_le
    s += mt.count_gt(rg, l) // timed: segtree.MergeSortTree.count_gt
    s += mt.count_ge(rg, l) // timed: segtree.MergeSortTree.count_ge
    s += mt.count_in(rg, l / 2, l) // timed: segtree.MergeSortTree.count_in
    s += mt.first_ge(rg, l) ?? 0 // timed: segtree.MergeSortTree.first_ge
    s += mt.last_le(rg, l) ?? 0 // timed: segtree.MergeSortTree.last_le

for _ in 0..20:
    s += WaveletMatrix(a).rank(V(0)..V(n - 1), 3) // timed: segtree.WaveletMatrix.new
let wm = WaveletMatrix(a)
for i in 0..reps:
    let l = i * 7919 % n
    let rg = V(l / 2)..V(l / 2 + n / 2)
    s += wm.kth(rg, l % (n / 2)) ?? 0 // timed: segtree.WaveletMatrix.kth
    s += wm.rank(rg, l) // timed: segtree.WaveletMatrix.rank
    s += wm.count_lt(rg, l) // timed: segtree.WaveletMatrix.count_lt
    s += wm.count_in(rg, l / 2, l) // timed: segtree.WaveletMatrix.count_in
    s += wm.prev_val(rg, l) ?? 0 // timed: segtree.WaveletMatrix.prev_val
    s += wm.next_val(rg, l) ?? 0 // timed: segtree.WaveletMatrix.next_val
    s += wm.quantile_sum(rg, l % (n / 2)) // timed: segtree.WaveletMatrix.quantile_sum

for _ in 0..3:
    s += DynamicOrderTree(a).get(V(0)) // timed: segtree.DynamicOrderTree.new
var dt = DynamicOrderTree(a)
for i in 0..reps / 8:
    let l = i * 7919 % n
    let rg = V(l / 2)..V(l / 2 + n / 2)
    dt.set(V(i * 13 % n), l) // timed: segtree.DynamicOrderTree.set
    s += dt.count_lt(rg, l) // timed: segtree.DynamicOrderTree.count_lt
    s += dt.count_in(rg, l / 2, l) // timed: segtree.DynamicOrderTree.count_in
    s += dt.kth(rg, l % (n / 2)) ?? 0 // timed: segtree.DynamicOrderTree.kth

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

for i in 0..reps:
    s += dt.get(V(i * 17 % n)) // timed: segtree.DynamicOrderTree.get
for _ in 0..1 << 17:
    var e = MergeTrees<V, Int>() // timed: segtree.MergeTrees.new
    s += 1

let leaf: (Int, Int, Int, Int) -> Int = (x, y, px, py) => x + y // within: segtree.MergeTrees.merge
for _ in 0..8:
    var p = MergeTrees<V, Int>()
    var trees: List<MergeTree<V>> = []
    for i in 0..n:
        var tr = MergeTree<V>()
        p.set(mut tr, V(i * 7919 % n), i) // timed: segtree.MergeTrees.set
        trees.push(tr)
    var m = trees[0]
    for i in 1..n:
        m = p.merge(m, trees[i], leaf) // timed: segtree.MergeTrees.merge
    for i in 0..n:
        s += p.sum(m, V(i / 2)..V(i / 2 + n / 2)) // timed: segtree.MergeTrees.sum
        s += p.get(m, V(i * 13 % n)) // timed: segtree.MergeTrees.get
        s += p.total(m) // timed: segtree.MergeTrees.total
        p.scale(m, 1) // timed: segtree.MergeTrees.scale
print(s % 1000000007)

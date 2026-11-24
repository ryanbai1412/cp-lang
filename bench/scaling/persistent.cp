use std.persistent.{PArena, PersistentSegTree, PersistentLazySegTree, PersistentArray, PMultiset}
let n: Int = read()
domain V = 0..n
let reps = 1 << 17
var s = 0
let a: [V]Int = [V] v => (v.ord * 7919) % n + 1

for _ in 0..reps:
    let ar = PArena<Int>(cap: n) // timed: persistent.PArena.new
    s += ar.len
var ar = PArena<Int>()
for i in 0..reps:
    let id = ar.alloc(i) // timed: persistent.PArena.alloc
    s += ar.get(id) // timed: persistent.PArena.get
    s += ar.len // timed: persistent.PArena.len
    if ar.is_empty: s += 1 // timed: persistent.PArena.is_empty
// Storage grows by moving the nodes already there.
for _ in 0..700:
    var r = PArena<Int>(cap: n)
    for i in 0..n: r.alloc(i)
    r.reserve(2 * n) // timed: persistent.PArena.reserve
    s += r.len

for _ in 0..100:
    let t = PersistentSegTree(+, a: a) // timed: persistent.PersistentSegTree.new
    s += t.all(t.base)
    s += t.all(t.base)
var t = PersistentSegTree(+, a: a)
var v = t.base
let small_r: Int -> Bool = x => x <= n * 4 // within: persistent.PersistentSegTree.right_while
let small_l: Int -> Bool = x => x <= n * 4 // within: persistent.PersistentSegTree.left_while
let bump: Int -> Int = x => x % 1000 + 1 // within: persistent.PersistentSegTree.update
for i in 0..reps:
    let l = i * 7919 % n
    let r = l + (n - l) / 2
    let b = t.base // timed: persistent.PersistentSegTree.base
    v = t.set(v, V(i * 17 % n), i % 9) // timed: persistent.PersistentSegTree.set
    v = t.update(v, V(i * 19 % n), bump) // timed: persistent.PersistentSegTree.update
    s += t.get(v, V(i * 13 % n)) // timed: persistent.PersistentSegTree.get
    s += t.fold(v, V(l)..V(r)) // timed: persistent.PersistentSegTree.fold
    s += t.all(v) // timed: persistent.PersistentSegTree.all
    s += t.right_while(b, V.cuts.nth(l), small_r).ord // timed: persistent.PersistentSegTree.right_while
    s += t.left_while(b, V.cuts.nth(r), small_l).ord // timed: persistent.PersistentSegTree.left_while
var z = PersistentSegTree<V, Int>(+, fill: 0)
var zv = [z.base]
for i in 0..n: zv.push(z.set(zv[i], V(i * 7919 % n), 1))
for i in 0..reps:
    s += (z.diff_kth(zv[n], zv[i % n], i % (n - i % n)) ?? V(0)).ord // timed: persistent.PersistentSegTree.diff_kth


// one O(1) tag callback shared by every op below, so its line is charged to none of them
let act = Action<Int, Int>(apply: (d, x, span) => x + d * span)
for _ in 0..100:
    let p = PersistentLazySegTree(act, a: a) // timed: persistent.PersistentLazySegTree.new
    s += p.all(p.base)
var p = PersistentLazySegTree(act, a: a)
var pv = p.base
for i in 0..reps:
    let l = i * 7919 % n
    let r = l + (n - l) / 2
    let b = p.base // timed: persistent.PersistentLazySegTree.base
    pv = p.apply(pv, V(l)..V(r), 1) // timed: persistent.PersistentLazySegTree.apply
    pv = p.set(pv, V(i * 17 % n), i % 9) // timed: persistent.PersistentLazySegTree.set
    s += p.fold(pv, V(l)..V(r)) // timed: persistent.PersistentLazySegTree.fold
    s += p.get(b, V(i * 13 % n)) // timed: persistent.PersistentLazySegTree.get
    s += p.all(pv) // timed: persistent.PersistentLazySegTree.all

for _ in 0..100:
    let q = PersistentArray(a: a) // timed: persistent.PersistentArray.new
    s += q.get(q.base, V(0))
var q = PersistentArray(a: a)
var qv = q.base
for i in 0..reps:
    let b = q.base // timed: persistent.PersistentArray.base
    qv = q.set(qv, V(i * 17 % n), i % 9) // timed: persistent.PersistentArray.set
    s += q.get(b, V(i * 13 % n)) // timed: persistent.PersistentArray.get

var mar = PArena<PMultiset<V>>()
var ms = PMultiset<V>.empty()
for i in 0..reps:
    let e = PMultiset<V>.empty() // timed: persistent.PMultiset.empty
    s += e.len
    ms = ms.with(mut mar, V(i * 7919 % n)) // timed: persistent.PMultiset.with
    ms = ms.without(mut mar, V(i * 13 % n)) // timed: persistent.PMultiset.without
    s += ms.count(mar, V(i * 17 % n)) // timed: persistent.PMultiset.count
    s += ms.len // timed: persistent.PMultiset.len
    if ms.is_empty: s += 1 // timed: persistent.PMultiset.is_empty
    s += ms.cmp(mar, e) // timed: persistent.PMultiset.cmp
print(s)

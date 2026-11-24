// n: 2048
use std.rand.{Rng}
use std.tree.{EulerTourTree, LazyEulerTourTree}
let n: Int = read()
domain V = 0..n
var rng = Rng(9)
var par: [V]Int = [V] v => if v.ord == 0: -1 else: rng.int(0, v.ord - 1)
let vals: [V]Int = [V] v => v.ord % 10
let act = Action<Int, Int>(apply: (d, x, k) => x + d * k)
domain W = 0..32 * n
let wvals: [W]Int = [W] w => w.ord % 10
var s = 0
for _ in 0..20:
    var e0 = EulerTourTree(+, a: Some(wvals)) // timed: tree.EulerTourTree.new
    s += e0.get(W(0))
    var z0 = LazyEulerTourTree(act, a: Some(wvals)) // timed: tree.LazyEulerTourTree.new
    s += z0.get(W(0))
var et = EulerTourTree(+, a: Some(vals))
var lz = LazyEulerTourTree(act, a: Some(vals))
for v in V:
    if v.ord > 0:
        et.link(v, V(par[v]))
        lz.link(v, V(par[v]))
for i in 0..32 * n:
    let a = V((i * 7919) % n)
    let b = V((i * 104729 + 7) % n)
    let c = V(1 + (i * 31) % (n - 1))
    let p = V(rng.int(0, c.ord - 1))
    let q = V(par[c])
    et.cut(c, q) // timed: tree.EulerTourTree.cut
    et.link(c, p) // timed: tree.EulerTourTree.link
    lz.cut(c, q) // timed: tree.LazyEulerTourTree.cut
    lz.apply_tree(c, 1) // timed: tree.LazyEulerTourTree.apply_tree
    s += lz.fold_tree(c) // timed: tree.LazyEulerTourTree.fold_tree
    lz.link(c, p) // timed: tree.LazyEulerTourTree.link
    par[c] = p.ord
    if et.connected(a, b): s += 1 // timed: tree.EulerTourTree.connected
    s += et.len(a) // timed: tree.EulerTourTree.len
    et.set(a, i % 10) // timed: tree.EulerTourTree.set
    s += et.fold_sub(c, p) // timed: tree.EulerTourTree.fold_sub
    if lz.connected(a, b): s += 1 // timed: tree.LazyEulerTourTree.connected
    s += lz.len(a) // timed: tree.LazyEulerTourTree.len
    s += lz.get(a) // timed: tree.LazyEulerTourTree.get
    lz.set(a, i % 10) // timed: tree.LazyEulerTourTree.set
    s += lz.fold_sub(c, p) // timed: tree.LazyEulerTourTree.fold_sub
    lz.apply_sub(c, p, 1) // timed: tree.LazyEulerTourTree.apply_sub
    s %= 1000000007
for i in 0..256 * n:
    s += et.get(V((i * 7919) % n)) // timed: tree.EulerTourTree.get
for i in 0..128 * n:
    s += et.fold_tree(V((i * 7919) % n)) // timed: tree.EulerTourTree.fold_tree
print(s)

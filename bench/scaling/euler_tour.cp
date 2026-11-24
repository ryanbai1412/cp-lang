// n: 2048
use std.rand.{Rng}
use std.tree.{EulerTourTree}
let n: Int = read()
domain V = 0..n
var rng = Rng(9)
var par: [V]Int = [V] v => if v.ord == 0: -1 else: rng.int(0, v.ord - 1)
let vals: [V]Int = [V] v => v.ord % 10
domain W = 0..32 * n
let wvals: [W]Int = [W] w => w.ord % 10
var s = 0
for _ in 0..20:
    var e0 = EulerTourTree(+, a: Some(wvals)) // timed: tree.EulerTourTree.new
    s += e0.get(W(0))
var et = EulerTourTree(+, a: Some(vals))
for v in V:
    if v.ord > 0:
        et.link(v, V(par[v]))
for i in 0..32 * n:
    let a = V((i * 7919) % n)
    let b = V((i * 104729 + 7) % n)
    let c = V(1 + (i * 31) % (n - 1))
    let p = V(rng.int(0, c.ord - 1))
    let q = V(par[c])
    et.cut(c, q) // timed: tree.EulerTourTree.cut
    et.link(c, p) // timed: tree.EulerTourTree.link
    if et.connected(a, b): s += 1 // timed: tree.EulerTourTree.connected
    s += et.len(a) // timed: tree.EulerTourTree.len
    et.set(a, i % 10) // timed: tree.EulerTourTree.set
    et.set(b, i % 7) // timed: tree.EulerTourTree.set
    s += et.fold_sub(c, p) // timed: tree.EulerTourTree.fold_sub
    par[c] = p.ord
    s %= 1000000007
for i in 0..256 * n:
    s += et.get(V((i * 7919) % n)) // timed: tree.EulerTourTree.get
for i in 0..128 * n:
    s += et.fold_tree(V((i * 7919) % n)) // timed: tree.EulerTourTree.fold_tree
print(s)

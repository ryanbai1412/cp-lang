// n: 2048
use std.rand.{Rng}
use std.segtree.{SegTree}
use std.algebra.{add}
let n: Int = read()
domain V = 0..n
var rng = Rng(7)
let rpar: [V]?V = [V] v => if v.ord == 0: None else: Some(V(rng.int(0, v.ord - 1)))
let t = Rooted.from_parents(rpar)
let seg = SegTree(+, t.reorder([V] v => v.ord))
var lazy = SegTree(+, t.reorder([V] v => 0), act: add)
var s = 0
for i in 0..64 * n:
    let a = V(1 + (i * 7919) % (n - 1))
    let b = V((i * 104729 + 7) % n)
    for ch in t.chains(a, b): s += ch.range.count() // timed: tree.Rooted.chains
    s += t.fold_path(a, b, seg) // timed: tree.Rooted.fold_path
    t.apply_path(a, b, mut lazy, 1) // timed: tree.Rooted.apply_path
    s += t.w(a) // timed: tree.Rooted.w
    s += t.top(a).ord // timed: tree.Rooted.top
    s += t.chain(a).count() // timed: tree.Rooted.chain
    for r in t.sub_at(a, b): s += r.count() // timed: tree.Rooted.sub_at
    s %= 1000000007
print(s)

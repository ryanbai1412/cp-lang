// n: 2048
// let: size = n
use std.rand.{Rng}
let n: Int = read()
domain V = 0..n
domain E = 0..n - 1
var rng = Rng(11)
let rpar: [V]?V = [V] v => if v.ord == 0: None else: Some(V(rng.int(0, v.ord - 1)))
let es: [E](u: V, v: V) = [E] e => (u: V(e.ord + 1), v: rpar[V(e.ord + 1)]!)
let wes: [E](u: V, v: V, w: Int) = [E] e => (u: es[e].u, v: es[e].v, w: rng.int(0, 1000000))
var s = 0
let leaf_f: V -> Int = v => 1 // within: tree.KruskalTree.up
let (domain N, kt) = kruskal_tree(wes, w: e => wes[e].w)
for _ in 0..40:
    let (domain M, k2) = kruskal_tree(wes, w: e => wes[e].w) // timed: tree.kruskal_tree
    s += k2.leaf[V(0)].ord
    s += kt.up(+, leaf: leaf_f)[kt.leaf[V(0)]] // timed: tree.KruskalTree.up
    s += kt.tree.size(kt.leaf[V(0)]) // timed: tree.KruskalTree.tree
let top_f: E -> Bool = e => wes[e].w < 500000 // within: tree.KruskalTree.top
for i in 0..64 * n:
    let a = V((i * 7919) % n)
    let b = V((i * 104729 + 7) % n)
    s += kt.reach(a, 500000).ord // timed: tree.KruskalTree.reach
    s += kt.top(a, top_f).ord // timed: tree.KruskalTree.top
    s += kt.bottleneck(a, b) ?? 0 // timed: tree.KruskalTree.bottleneck
    s %= 1000000007
print(s)

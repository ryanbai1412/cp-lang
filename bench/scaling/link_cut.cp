// n: 2048
use std.rand.{Rng}
use std.tree.{LinkCut}
let n: Int = read()
domain V = 0..n
var rng = Rng(3)
var par: [V]Int = [V] v => if v.ord == 0: -1 else: rng.int(0, v.ord - 1)
let vals: [V]Int = [V] v => v.ord % 10
var s = 0
var lc = LinkCut(+, a: Some(vals))
for v in V:
    if v.ord > 0:
        lc.link(v, V(par[v]))
for i in 0..32 * n:
    let a = V((i * 7919) % n)
    let b = V((i * 104729 + 7) % n)
    let c = V(1 + (i * 31) % (n - 1))
    let p = V(rng.int(0, c.ord - 1))
    lc.cut(c, V(par[c])) // timed: tree.LinkCut.cut
    lc.link(c, p) // timed: tree.LinkCut.link
    s += lc.access(a).ord // timed: tree.LinkCut.access
    lc.make_root(b) // timed: tree.LinkCut.make_root
    s += lc.root_of(a).ord // timed: tree.LinkCut.root_of
    if lc.connected(a, b): s += 1 // timed: tree.LinkCut.connected
    s += (lc.lca(a, b, V(0)) ?? a).ord // timed: tree.LinkCut.lca
    lc.set(a, i % 10) // timed: tree.LinkCut.set
    s += lc.fold_path(a, b) // timed: tree.LinkCut.fold_path
    s += lc.dist(a, b) ?? 0 // timed: tree.LinkCut.dist
    s += (lc.jump(a, b, 2) ?? a).ord // timed: tree.LinkCut.jump
    par[c] = p.ord
    s %= 1000000007
for i in 0..256 * n:
    s += lc.get(V((i * 7919) % n)) // timed: tree.LinkCut.get
print(s)

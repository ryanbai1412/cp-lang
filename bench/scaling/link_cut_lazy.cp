// n: 2048
use std.rand.{Rng}
use std.tree.{LazyLinkCut}
let n: Int = read()
domain V = 0..n
var rng = Rng(3)
var par: [V]Int = [V] v => if v.ord == 0: -1 else: rng.int(0, v.ord - 1)
let vals: [V]Int = [V] v => v.ord % 10
let act = Action<Int, Int>(apply: (d, x, k) => x + d * k)
var s = 0
var lz = LazyLinkCut(act, a: Some(vals))
for v in V:
    if v.ord > 0:
        lz.link(v, V(par[v]))
for i in 0..32 * n:
    let a = V((i * 7919) % n)
    let b = V((i * 104729 + 7) % n)
    let c = V(1 + (i * 31) % (n - 1))
    let p = V(rng.int(0, c.ord - 1))
    lz.cut(c, V(par[c])) // timed: tree.LazyLinkCut.cut
    lz.link(c, p) // timed: tree.LazyLinkCut.link
    s += lz.access(a).ord // timed: tree.LazyLinkCut.access
    lz.make_root(b) // timed: tree.LazyLinkCut.make_root
    s += lz.root_of(a).ord // timed: tree.LazyLinkCut.root_of
    if lz.connected(a, b): s += 1 // timed: tree.LazyLinkCut.connected
    s += (lz.lca(a, b, V(0)) ?? a).ord // timed: tree.LazyLinkCut.lca
    par[c] = p.ord
    s %= 1000000007
print(s)

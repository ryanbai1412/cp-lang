// n: 2048
use std.rand.{Rng}
use std.tree.{LinkCut, LazyLinkCut, PreferredPaths}
let n: Int = read()
domain V = 0..n
var rng = Rng(3)
var par: [V]Int = [V] v => if v.ord == 0: -1 else: rng.int(0, v.ord - 1)
let vals: [V]Int = [V] v => v.ord % 10
let act = Action<Int, Int>(apply: (d, x, k) => x + d * k)
domain W = 0..32 * n
let wvals: [W]Int = [W] w => w.ord % 10
var wpar: [W]?W = [W] w => if w.ord == 0: None else: Some(W(rng.int(0, w.ord - 1)))
let wt = Rooted.from_parents(wpar)
var s = 0
for _ in 0..40:
    s += LinkCut(+, a: Some(wvals)).get(W(0)) // timed: tree.LinkCut.new
    var z = LazyLinkCut(act, a: Some(wvals)) // timed: tree.LazyLinkCut.new
    s += z.get(W(0))
    var pp0 = PreferredPaths(wt) // timed: tree.PreferredPaths.new
    s += pp0.hd[W(0)].ord
var lc = LinkCut(+, a: Some(vals))
var lz = LazyLinkCut(act, a: Some(vals))
for v in V:
    if v.ord > 0:
        lc.link(v, V(par[v]))
        lz.link(v, V(par[v]))
for i in 0..32 * n:
    let a = V((i * 7919) % n)
    let b = V((i * 104729 + 7) % n)
    let c = V(1 + (i * 31) % (n - 1))
    let p = V(rng.int(0, c.ord - 1))
    lc.cut(c, V(par[c])) // timed: tree.LinkCut.cut
    lc.link(c, p) // timed: tree.LinkCut.link
    lz.cut(c, V(par[c])) // timed: tree.LazyLinkCut.cut
    lz.link(c, p) // timed: tree.LazyLinkCut.link
    par[c] = p.ord
    s += lc.access(a).ord // timed: tree.LinkCut.access
    lc.make_root(b) // timed: tree.LinkCut.make_root
    s += lc.root_of(a).ord // timed: tree.LinkCut.root_of
    if lc.connected(a, b): s += 1 // timed: tree.LinkCut.connected
    s += (lc.lca(a, b, V(0)) ?? a).ord // timed: tree.LinkCut.lca
    lc.set(a, i % 10) // timed: tree.LinkCut.set
    s += lc.fold_path(a, b) // timed: tree.LinkCut.fold_path
    s += lc.dist(a, b) ?? 0 // timed: tree.LinkCut.dist
    s += (lc.jump(a, b, 2) ?? a).ord // timed: tree.LinkCut.jump
    s += lz.access(a).ord // timed: tree.LazyLinkCut.access
    lz.make_root(b) // timed: tree.LazyLinkCut.make_root
    s += lz.root_of(a).ord // timed: tree.LazyLinkCut.root_of
    if lz.connected(a, b): s += 1 // timed: tree.LazyLinkCut.connected
    s += (lz.lca(a, b, V(0)) ?? a).ord // timed: tree.LazyLinkCut.lca
    s += lz.get(a) // timed: tree.LazyLinkCut.get
    lz.set(a, i % 10) // timed: tree.LazyLinkCut.set
    lz.apply_path(a, b, 1) // timed: tree.LazyLinkCut.apply_path
    s += lz.fold_path(a, b) // timed: tree.LazyLinkCut.fold_path
    s += lz.dist(a, b) ?? 0 // timed: tree.LazyLinkCut.dist
    s += (lz.jump(a, b, 2) ?? a).ord // timed: tree.LazyLinkCut.jump
    s %= 1000000007
var radj: [V]List<V> = []
for v in V:
    if v.ord > 0:
        radj[v].push(V(par[v]))
        radj[V(par[v])].push(v)
let t = Rooted(radj, root: V(0))
var pp = PreferredPaths(t)
for i in 0..32 * n:
    for x in pp.access(V((i * 7919) % n)): s += 1 // timed: tree.PreferredPaths.access
for i in 0..256 * n:
    s += lc.get(V((i * 7919) % n)) // timed: tree.LinkCut.get
print(s)

// expect: ok, output "ok"
// input: "1 300\n"
// model: KruskalTree
// model: kruskal_tree
// Kruskal reconstruction trees of seeded random weighted edge lists (one per op) against a Dsu
// grown edge by edge: reach, top, up and bottleneck for a random vertex and limit, and the
// tree's shape. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
let seed0 = read<Int>()
let n = read<Int>()
domain V = 0..8
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    let es: [0..9](u: V, v: V, w: Int) = [0..9] _ => (u: V(rng.int(0, 7)), v: V(rng.int(0, 7)), w: rng.int(1, 6))
    let val: [V]Int = [V] _ => rng.int(-5, 9)
    let (domain N, kt) = kruskal_tree(es, w: e => es[e].w)
    let tot = kt.up(+, leaf: v => val[v])
    let cnt = kt.up(+, leaf: v => 1)
    let low = kt.up(min, leaf: v => val[v])
    let (a, b) = (V(rng.int(0, 7)), V(rng.int(0, 7)))
    let lim = rng.int(0, 7)
    var d = Dsu<V>()
    for e in es.keys:
        if es[e].w <= lim: d.unite(es[e].u, es[e].v)
    let same = [u for u in V if d.same(u, a)]
    var bn: ?Int = None
    for l in (1..=6).rev:
        var g = Dsu<V>()
        for e in es.keys:
            if es[e].w <= l: g.unite(es[e].u, es[e].v)
        if g.same(a, b) and a != b: bn = Some(l)
    let x = kt.reach(a, lim)
    let tr = kt.tree
    if tot[x] != sum(val[u] for u in same) or cnt[x] != same.len or low[x] != [val[u] for u in same].min(): bad = "op {k}: reach {a} {lim}"
    elif kt.top(a, e => es[e].w <= lim) != x: bad = "op {k}: top {a} {lim}"
    elif kt.bottleneck(a, b) != bn: bad = "op {k}: bottleneck {a} {b}"
    elif kt.w[kt.leaf[a]] != None or tr.size(x) != 2 * same.len - 1: bad = "op {k}: tree shape"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)

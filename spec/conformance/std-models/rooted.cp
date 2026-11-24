// expect: ok, output "ok"
// input: "1 300\n"
// model: Rooted
// model: LcaTable
// model: Layers
// model: Events
// model: LongPaths
// Rooted trees built from seeded random parent arrays (rebuilt every 25 ops) against climbing
// the parent array: parent, depth, size, root_of, children, subtree, is_anc, lca, dist, path,
// ancestor, ancestors, jump, lower, on_path, meet, diameter and centroid, with LcaTable, Layers,
// Events and LongPaths checked on the same tree. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
let seed0 = read<Int>()
let n = read<Int>()
domain V = 0..11
var rng = Rng(seed0)
fn climb(par: [V]?V, v: V) -> List<V>:
    var out: List<V> = [v]
    while let p = par[out[out.len - 1]]: out.push(p)
    out
fn bpath(par: [V]?V, a: V, b: V) -> List<V>:
    let (ca, cb) = (climb(par, a), climb(par, b))
    var i = 0
    while not cb.contains(ca[i]): i += 1
    var j = 0
    while cb[j] != ca[i]: j += 1
    ca[..=i] + cb[..j].rev.collect()
fn sorted(xs: List<V>) -> List<V>:
    var ys = xs.clone()
    ys.sort()
    ys
var par: [V]?V = None
var bad = ""
for k in 0..n:
    if k % 25 == 0:
        var perm = [V(i) for i in 0..11]
        rng.shuffle(mut perm)
        par = [V] _ => None
        for i in 1..11: par[perm[i]] = Some(perm[rng.int(0, i - 1)])
    let t = Rooted.from_parents(par)
    let lt = LcaTable(t)
    let ly = t.layers()
    let ev = t.events()
    let lp = t.long_paths()
    let (a, b, c) = (V(rng.int(0, 10)), V(rng.int(0, 10)), V(rng.int(0, 10)))
    let ch = climb(par, a)
    let pab = bpath(par, a, b)
    let l = pab.find(v => climb(par, a).contains(v) and climb(par, b).contains(v))!
    let sub = [v for v in V if climb(par, v).contains(a)]
    let kk = rng.int(0, 12)
    let dist = [V] v => climb(par, v).len - 1
    let height = [v for v in sub].map(v => dist[v] - dist[a]).max()!
    let diam = [bpath(par, x, y).len - 1 for x in V for y in V].max()!
    let biggest = [V] v => max(11 - [u for u in V if climb(par, u).contains(v)].len, ([[u for u in V if climb(par, u).contains(w)].len for w in V if par[w] == Some(v)].max() ?? 0))
    var at: [V]Int = 0
    for i in 0..t.order.len: at[t.order[i]] = i
    let m3 = [x for x in pab if bpath(par, b, c).contains(x) and bpath(par, a, c).contains(x)]
    if t.parent(a) != par[a] or t.depth(a) != ch.len - 1 or t.size(a) != sub.len or t.root_of(a) != ch[ch.len - 1]: bad = "op {k}: parent/depth/size/root_of {a}"
    elif sorted(t.children(a).collect()) != [v for v in V if par[v] == Some(a)] or sorted(t.subtree(a).collect()) != sub: bad = "op {k}: children/subtree {a}"
    elif t.is_anc(a, b) != climb(par, b).contains(a) or t.lca(a, b) != l or lt.lca(a, b) != l: bad = "op {k}: is_anc/lca {a} {b}"
    elif t.dist(a, b) != pab.len - 1 or lt.dist(a, b) != pab.len - 1 or t.path(a, b) != pab: bad = "op {k}: dist/path {a} {b}"
    elif t.ancestor(a, kk) != (if kk < ch.len: Some(ch[kk]) else: None) or t.ancestors(a).collect() != ch: bad = "op {k}: ancestor {a} {kk}"
    elif t.jump(a, b, kk) != (if kk < pab.len: Some(pab[kk]) else: None): bad = "op {k}: jump {a} {b} {kk}"
    elif pab.len == 2 and t.lower(a, b) != (if par[a] == Some(b): a else: b): bad = "op {k}: lower {a} {b}"
    elif t.on_path(c, a, b) != pab.contains(c) or m3 != [t.meet(a, b, c)]: bad = "op {k}: on_path/meet {a} {b} {c}"
    elif t.diameter().len != diam or bpath(par, t.diameter().u, t.diameter().v).len - 1 != diam: bad = "op {k}: diameter"
    elif biggest[t.centroid()] * 2 > 11 or sorted(t.centroids()) != [v for v in V if biggest[v] * 2 <= 11]: bad = "op {k}: centroid"
    elif t.sub(a).count() != sub.len or not (t.pos(a) in t.sub(a)) or sub.any(v => not (t.pos(v) in t.sub(a))): bad = "op {k}: sub {a}"
    elif t.order.len != 11 or V.any(v => if let p = par[v]: at[p] > at[v] else: false): bad = "op {k}: order"
    elif sorted(ly.level(kk).map(x => ly.at[x]).collect()) != [v for v in V if dist[v] == kk] or ly.at[ly.pos(a)] != a: bad = "op {k}: layers {kk}"
    elif sorted(ly.below(a, kk).map(x => ly.at[x]).collect()) != [v for v in sub if dist[v] == kk]: bad = "op {k}: layers below {a} {kk}"
    elif ev.at[ev.enter(a)] != Visit.Enter(a, par[a]) or ev.at[ev.exit(a)] != Visit.Exit(a, par[a]) or ev.sub(a).count() != 2 * sub.len: bad = "op {k}: events {a}"
    elif lp.slots(a).count() != height + 1 or (if let d = lp.deep[a]: lp.slot(a, 1) != lp.slot(d, 0) else: height != 0): bad = "op {k}: long_paths {a}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)

// expect: ok, output "ok"
// input: "1 300\n"
// model: LinkCut
// model: LazyLinkCut
// A LinkCut over + and a LazyLinkCut with range-add tags against an adjacency list and plain
// value arrays: seeded random link, cut, set, apply_path, make_root and access, with
// connected, root_of, dist, jump, fold_path, get and lca checked after each op. Reads `seed n`.
use std.rand.{Rng}
use std.tree.{LinkCut, LazyLinkCut}
let seed0 = read<Int>()
let n = read<Int>()
domain V = 0..9
var rng = Rng(seed0)
fn bpath(adj: [V]List<V>, a: V, b: V) -> ?List<V>:
    var from: [V]?V = None
    var seen: [V]Bool = false
    seen[a] = true
    var q = [a]
    var i = 0
    while i < q.len:
        let x = q[i]
        i += 1
        for y in adj[x]:
            if not seen[y]:
                seen[y] = true
                from[y] = Some(x)
                q.push(y)
    if not seen[b]: return None
    var out = [b]
    var x = b
    while let p = from[x]:
        out.push(p)
        x = p
    Some(out.rev.collect())
var w1: [V]Int = [V] _ => rng.int(-5, 5)
var w2 = w1.clone()
var lc = LinkCut(+, a: Some(w1))
var lz = LazyLinkCut<V, Int, Int>.new(add, Some(w2))
var adj: [V]List<V> = []
var bad = ""
for k in 0..n:
    let op = rng.int(0, 6)
    let (u, v) = (V(rng.int(0, 8)), V(rng.int(0, 8)))
    let x = rng.int(-5, 5)
    let p0 = bpath(adj, u, v)
    if op <= 1 and p0 == None:
        lc.link(u, v)
        lz.link(u, v)
        adj[u].push(v)
        adj[v].push(u)
    elif op == 2 and adj[u].len > 0:
        let y = adj[u][rng.int(0, adj[u].len - 1)]
        if rng.int(0, 1) == 0: lc.cut(u, y) else: lc.cut(y, u)
        lz.cut(u, y)
        adj[u] = [z for z in adj[u] if z != y]
        adj[y] = [z for z in adj[y] if z != u]
    elif op == 3:
        lc.set(u, x)
        lz.set(u, x)
        w1[u] = x
        w2[u] = x
    elif op == 4:
        if let q = p0:
            lz.apply_path(u, v, x)
            for z in q: w2[z] += x
    elif op == 5:
        lc.make_root(u)
        lz.make_root(u)
        if lc.root_of(u) != u or lz.root_of(u) != u: bad = "op {k}: make_root {u}"
    else:
        let _ = lc.access(u)
        let _ = lz.access(u)
    let p = bpath(adj, u, v)
    let kk = rng.int(0, 9)
    let r = V(rng.int(0, 8))
    var want_lca: ?V = None
    if let q = p:
        if bpath(adj, u, r) != None:
            var best = q[0]
            for z in q:
                if bpath(adj, z, r)!.len < bpath(adj, best, r)!.len: best = z
            want_lca = Some(best)
    if bad != "": break
    if lc.connected(u, v) != (p != None) or lz.connected(u, v) != (p != None): bad = "op {k}: connected {u} {v}"
    elif lc.dist(u, v) != p.map(q => q.len - 1) or lz.dist(u, v) != p.map(q => q.len - 1): bad = "op {k}: dist {u} {v}"
    elif lc.jump(u, v, kk) != (if let q = p: (if kk < q.len: Some(q[kk]) else: None) else: None) or lz.jump(u, v, kk) != lc.jump(u, v, kk): bad = "op {k}: jump {u} {v} {kk}"
    elif p != None and (lc.fold_path(u, v) != sum(w1[z] for z in p!) or lz.fold_path(u, v) != sum(w2[z] for z in p!)): bad = "op {k}: fold_path {u} {v}"
    elif lc.get(u) != w1[u] or lz.get(u) != w2[u]: bad = "op {k}: get {u}"
    elif lc.lca(u, v, r) != want_lca or lz.lca(u, v, r) != want_lca: bad = "op {k}: lca {u} {v} at {r}"
    elif want_lca != None and (lc.root_of(u) != r or lz.root_of(v) != r): bad = "op {k}: root_of after lca"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)

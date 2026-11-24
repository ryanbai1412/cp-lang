// expect: ok, output "ok"
// input: "1 300\n"
// model: Walk
// model: euler_walk
// model: bridges
// model: two_edge_components
// model: cut_vertices
// model: blocks
// model: block_cut_tree
// model: spanning_forest
// model: scc_times
// model: IncrementalBridges
// model: kruskal
// model: prim
// model: boruvka
// model: arborescence
// Cuts, blocks, Euler walks and spanning structures against flood-fill labels over edge
// subsets: each op draws a random multigraph on 6 vertices with 8 edges (sometimes one long
// walk, so Euler walks exist), then checks bridges, two-edge and biconnected blocks, cut
// vertices, the spanning forest, offline SCC times, incremental bridges, and minimum spanning
// forests and arborescences against subset enumeration. Reads `seed n`.
use std.graph.{Graph, IncrementalBridges, arborescence, block_cut_tree, blocks, boruvka, bridges, cut_vertices, euler_walk, kruskal, prim, scc_times, spanning_forest, two_edge_components}
use std.rand.{Rng}
let seed0 = read<Int>()
let n = read<Int>()
domain V = 0..6
domain E = 0..8
// each vertex's least connected vertex over the kept edges, ignoring vertex `skip`
fn label(eu: List<Int>, ev: List<Int>, keep: List<Bool>, skip: Int) -> List<Int>:
    var lab = [i for i in 0..6]
    var ch = true
    while ch:
        ch = false
        for i in 0..eu.len:
            if keep[i] and eu[i] != skip and ev[i] != skip:
                let m = min(lab[eu[i]], lab[ev[i]])
                if lab[eu[i]] != m or lab[ev[i]] != m:
                    lab[eu[i]] = m
                    lab[ev[i]] = m
                    ch = true
    lab
fn ncomp(lab: List<Int>, skip: Int) -> Int = (0..6).count(x => x != skip and lab[x] == x)
// least total weight over edge subsets forming a spanning forest (sign -1: greatest)
fn best_forest(eu: List<Int>, ev: List<Int>, ew: List<Int>, sign: Int) -> Int:
    let all = List.filled(eu.len, true)
    let want = 6 - ncomp(label(eu, ev, all, -1), -1)
    var best = 1_000_000_000
    for mask in 0..(1 << eu.len):
        let keep = [(mask >> i) & 1 == 1 for i in 0..eu.len]
        if keep.count(x => x) == want and ncomp(label(eu, ev, keep, -1), -1) == 6 - want:
            var s = 0
            for i in 0..eu.len:
                if keep[i]: s += ew[i] * sign
            if s < best: best = s
    best * sign
// per component label, its lightest edge leaving it (earliest on ties)
fn cheapest(eu: List<Int>, ev: List<Int>, ew: List<Int>, comp: List<Int>) -> List<?(u: V, v: V, w: Int)>:
    let c = comp.max()! + 1
    var out: List<?(u: V, v: V, w: Int)> = List.filled(c, None)
    for i in 0..eu.len:
        for x in [comp[eu[i]], comp[ev[i]]]:
            if comp[eu[i]] != comp[ev[i]]:
                if out[x] == None or ew[i] < out[x]!.w: out[x] = Some((u: V(eu[i]), v: V(ev[i]), w: ew[i]))
    out
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    var eu = List<Int>()
    var ev = List<Int>()
    if rng.int(0, 2) == 0:
        var at = rng.int(0, 5)
        let start = at
        for i in 0..8:
            let to = if i == 7 and rng.int(0, 1) == 0: start else: rng.int(0, 5)
            eu.push(at)
            ev.push(to)
            at = to
        for i in 0..8:
            let j = rng.int(i, 7)
            eu.swap(i, j)
            ev.swap(i, j)
            if rng.int(0, 1) == 0:
                let t = eu[i]
                eu[i] = ev[i]
                ev[i] = t
    else:
        for _ in 0..8:
            eu.push(rng.int(0, 5))
            ev.push(rng.int(0, 5))
    let ew = [rng.int(1, 6) for _ in 0..8]
    let es: [E](u: V, v: V) = [E] e => (u: V(eu[e.ord]), v: V(ev[e.ord]))
    let wes: [E](u: V, v: V, w: Int) = [E] e => (u: V(eu[e.ord]), v: V(ev[e.ord]), w: ew[e.ord])
    let g = Graph.undirected(es)
    let all = List.filled(8, true)
    let lab = label(eu, ev, all, -1)
    // bridges and two-edge components
    let br = bridges(g)
    var nobr = List<Bool>()
    for e in 0..8:
        let without = [i != e for i in 0..8]
        let is = eu[e] != ev[e] and label(eu, ev, without, -1)[eu[e]] != label(eu, ev, without, -1)[ev[e]]
        if br[E(e)] != is: bad = "op {k}: bridges {e}"
        nobr.push(not is)
    let two = label(eu, ev, nobr, -1)
    let (domain C, tc) = two_edge_components(g)
    for a in V:
        for b in V:
            if (tc[a] == tc[b]) != (two[a.ord] == two[b.ord]): bad = "op {k}: two_edge_components {a} {b}"
    // cut vertices and blocks: no vertex separates two edges of a block
    let cv = cut_vertices(g)
    for x in 0..6:
        if cv[V(x)] != (ncomp(label(eu, ev, all, x), x) > ncomp(lab, -1)): bad = "op {k}: cut_vertices {x}"
    let (domain B, blk) = blocks(g)
    let cut_lab = [label(eu, ev, all, x) for x in 0..6]
    for e in 0..8:
        for f in 0..8:
            var same = e == f
            if not same and eu[e] != ev[e] and eu[f] != ev[f] and lab[eu[e]] == lab[eu[f]]:
                same = true
                for x in 0..6:
                    let l2 = cut_lab[x]
                    let ends = [y for y in [eu[e], ev[e], eu[f], ev[f]] if y != x]
                    if ends.any(y => l2[y] != l2[ends[0]]): same = false
            if (blk[E(e)] == blk[E(f)]) != same: bad = "op {k}: blocks {e} {f}"
    var classes = 0
    for e in 0..8:
        if (0..e).all(f => blk[E(f)] != blk[E(e)]): classes += 1
    if B.count != classes: bad = "op {k}: blocks count"
    // block-cut tree of the simple graph underneath
    var sadj = [List<V>() for _ in 0..6]
    var su = List<Int>()
    var sv = List<Int>()
    for i in 0..8:
        let (a, b) = (min(eu[i], ev[i]), max(eu[i], ev[i]))
        if a != b and not sadj[a].contains(V(b)):
            sadj[a].push(V(b))
            sadj[b].push(V(a))
            su.push(a)
            sv.push(b)
    let sg = [V] v => sadj[v.ord].sorted()
    let (domain T, links) = block_cut_tree(sg)
    let sall = List.filled(su.len, true)
    let slab = label(su, sv, sall, -1)
    var want_links = List<(Int, Int)>()
    var nblocks = 0
    // group simple edges into blocks by the same separation test
    var bid = List.filled(su.len, -1)
    let scut = [label(su, sv, sall, x) for x in 0..6]
    for e in 0..su.len:
        if bid[e] < 0:
            for f in e..su.len:
                var same = slab[su[e]] == slab[su[f]]
                for x in 0..6:
                    let l2 = scut[x]
                    let ends = [y for y in [su[e], sv[e], su[f], sv[f]] if y != x]
                    if ends.any(y => l2[y] != l2[ends[0]]): same = false
                if same and bid[f] < 0: bid[f] = nblocks
            nblocks += 1
    if T.count != nblocks: bad = "op {k}: block_cut_tree count {T.count} {nblocks}"
    else:
        var cuts_of = [List<Int>() for _ in 0..T.count]
        for l in links: cuts_of[l.b.ord].push(l.v.ord)
        var got_sets = [c.sorted() for c in cuts_of]
        var want_sets = List<List<Int>>()
        for b in 0..nblocks:
            var vs = List<Int>()
            for x in 0..6:
                if (0..su.len).any(e => bid[e] == b and (su[e] == x or sv[e] == x)) and ncomp(scut[x], x) > ncomp(slab, -1): vs.push(x)
            want_sets.push(vs)
        got_sets.sort()
        want_sets.sort()
        if got_sets != want_sets: bad = "op {k}: block_cut_tree links"
    // spanning forest: vertices in order, arcs in edge order, each arc joining two trees taken
    var take = List.filled(8, false)
    for u in 0..6:
        for i in 0..8:
            for (x, y) in [(eu[i], ev[i]), (ev[i], eu[i])]:
                if x == u and not take[i]:
                    let l = label(eu, ev, take, -1)
                    if l[x] != l[y]: take[i] = true
    if spanning_forest(g) != [E] e => take[e.ord]: bad = "op {k}: spanning_forest"
    // Euler walks
    for directed in [false, true]:
        var surplus = List.filled(6, 0)
        for i in 0..8:
            surplus[eu[i]] += 1
            surplus[ev[i]] += if directed: -1 else: 1
        let used = [x for x in 0..6 if (0..8).any(i => eu[i] == x or ev[i] == x)]
        let linked = used.all(x => lab[x] == lab[used[0]])
        let odd = if directed: [x for x in 0..6 if surplus[x] != 0] else: [x for x in 0..6 if surplus[x] % 2 != 0]
        let exists = linked and (odd.len == 0 or (odd.len == 2 and (not directed or odd.any(x => surplus[x] == 1))))
        let start = if not exists: -1 elif odd.len == 0: used[0] elif directed: odd.filter(x => surplus[x] == 1).first! else: odd[0]
        if let w = euler_walk(es, directed:):
            var cur = w.start.ord
            var hit = List.filled(8, false)
            if not exists or cur != start or w.arcs.len != 8: bad = "op {k}: euler_walk {yes(directed)} start"
            for a in w.arcs:
                let e = a.e.ord
                let ok = (eu[e] == cur and ev[e] == a.to.ord) or (not directed and ev[e] == cur and eu[e] == a.to.ord)
                if hit[e] or not ok: bad = "op {k}: euler_walk {yes(directed)} arc {e}"
                hit[e] = true
                cur = a.to.ord
            if w.vertices.collect() != [w.start] + [a.to for a in w.arcs]: bad = "op {k}: Walk.vertices"
        elif exists: bad = "op {k}: euler_walk {yes(directed)} missed"
    // offline SCC times: the first prefix whose reachability joins each arc's ends both ways
    var times = List<?Int>()
    for e in 0..8:
        var t: ?Int = None
        for i in e..8:
            if t == None:
                var r = [[a == b for b in 0..6] for a in 0..6]
                for j in 0..i + 1: r[eu[j]][ev[j]] = true
                for w in 0..6:
                    for a in 0..6:
                        for b in 0..6:
                            if r[a][w] and r[w][b]: r[a][b] = true
                if r[eu[e]][ev[e]] and r[ev[e]][eu[e]]: t = Some(i)
        times.push(t)
    if scc_times(es) != [E] e => times[e.ord]: bad = "op {k}: scc_times"
    // incremental bridges, after each edge
    var ib = IncrementalBridges<V>()
    for i in 0..8:
        ib.add(V(eu[i]), V(ev[i]))
        let keep = [j <= i for j in 0..8]
        let cl = label(eu, ev, keep, -1)
        var nb = keep.clone()
        for e in 0..i + 1:
            let without = [j <= i and j != e for j in 0..8]
            let l2 = label(eu, ev, without, -1)
            if eu[e] != ev[e] and l2[eu[e]] != l2[ev[e]]: nb[e] = false
        let tl = label(eu, ev, nb, -1)
        let (a, b) = (rng.int(0, 5), rng.int(0, 5))
        if ib.same(V(a), V(b)) != (cl[a] == cl[b]) or ib.two_edge_same(V(a), V(b)) != (tl[a] == tl[b]): bad = "op {k}: IncrementalBridges after {i}: {a} {b}"
    // minimum spanning forests
    let kr = kruskal(wes)
    let krmax = kruskal(wes, by: e => desc(e.w))
    let want_min = best_forest(eu, ev, ew, 1)
    for (res, want) in [(kr, want_min), (krmax, best_forest(eu, ev, ew, -1))]:
        let keep = [res.take[E(i)] for i in 0..8]
        var s = 0
        for i in 0..8:
            if keep[i]: s += ew[i]
        if res.total != want or s != want or ncomp(label(eu, ev, keep, -1), -1) != ncomp(lab, -1) or keep.count(x => x) != 6 - ncomp(lab, -1): bad = "op {k}: kruskal"
    var wm: [V, V]?Int = None
    for i in 0..8:
        if eu[i] != ev[i]:
            let (a, b) = (V(eu[i]), V(ev[i]))
            let w = if let c = wm[a, b]: min(c, ew[i]) else: ew[i]
            wm[a, b] = Some(w)
            wm[b, a] = Some(w)
    if let p = prim(wm):
        var s = 0
        for v in V:
            if let u = p.parent[v]: s += wm[u, v]!
        if ncomp(lab, -1) != 1 or p.total != want_min or s != want_min: bad = "op {k}: prim"
    elif ncomp(lab, -1) == 1: bad = "op {k}: prim missed"
    let bv = boruvka<V, Int, Int>(comp => cheapest(eu, ev, ew, [comp[v] for v in V]))
    if bv.total != want_min or bv.edges.len != 6 - ncomp(lab, -1): bad = "op {k}: boruvka"
    // arborescence: one entering edge per non-root vertex, every choice enumerated
    let root = rng.int(0, 5)
    let darb: [E](u: V, v: V, w: Int) = [E] e => (u: V(eu[e.ord]), v: V(ev[e.ord]), w: ew[e.ord] - 3)
    var into = [[i for i in 0..8 if ev[i] == x and eu[i] != x] for x in 0..6]
    var best: ?Int = None
    if (0..6).all(x => x == root or into[x].len > 0):
        var choice = List.filled(6, 0)
        var done = false
        while not done:
            var ok = true
            var s = 0
            for x in 0..6:
                if x != root:
                    s += ew[into[x][choice[x]]] - 3
                    var at = x
                    var steps = 0
                    while at != root and steps <= 6:
                        at = eu[into[at][choice[at]]]
                        steps += 1
                    if at != root: ok = false
            if ok and (best == None or s < best!): best = Some(s)
            var x = 0
            while x < 6 and (x == root or choice[x] + 1 == into[x].len):
                if x != root: choice[x] = 0
                x += 1
            if x == 6: done = true
            else: choice[x] += 1
    let arb = arborescence(darb, root: V(root))
    if let a = arb:
        var s = 0
        for v in V:
            if let e = a.parent[v]:
                s += darb[e].w
                if darb[e].v != v: bad = "op {k}: arborescence edge"
        if best != Some(a.total) or s != a.total: bad = "op {k}: arborescence total"
    elif best != None: bad = "op {k}: arborescence missed"
    if bad != "": break
print(if bad == "": "ok" else: bad)

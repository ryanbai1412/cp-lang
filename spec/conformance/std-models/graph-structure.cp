// expect: ok, output "ok"
// input: "1 300\n"
// model: Graph
// model: Only
// model: Complement
// model: complement
// model: only
// model: adjacency
// model: directed_of_pairs
// model: undirected_of_pairs
// model: deg
// model: next
// model: transpose
// model: arc_len
// model: peel
// model: topo
// model: smallest_topo
// model: scc
// model: condense
// model: components
// model: two_color
// model: odd_cycle
// model: find_cycle
// model: girth
// model: dfs
// model: reachable
// model: dag_fold
// model: count_paths
// model: reach_sets
// model: dominators
// model: triangles
// model: max_clique
// model: chromatic_number
// Graph building, views and the reachability family against plain lists and a reachability
// matrix: each op draws a random digraph on 6 vertices (and a random simple undirected graph),
// builds it every way, and checks orders, components, colourings, cycles, DFS events, DAG
// folds, dominators, triangles, cliques and colour counts. Reads `seed n`.
use std.graph.{Edge, Graph, Visit, adjacency, arc_len, chromatic_number, complement, components, condense, count_paths, dag_fold, dfs, directed_of_pairs, dominators, find_cycle, girth, max_clique, odd_cycle, peel, reach_sets, reachable, scc, smallest_topo, topo, triangles, two_color, undirected_of_pairs}
use std.rand.{Rng}
let seed0 = read<Int>()
let n = read<Int>()
domain V = 0..6
domain E = 0..8
fn bdfs(adj: List<List<Int>>, u: Int, via: Int, mut seen: List<Bool>, mut ev: List<(Int, Int, Int)>):
    seen[u] = true
    ev.push((0, u, via))
    for v in adj[u]:
        if seen[v]: ev.push((2, u, v))
        else: bdfs(adj, v, v, mut seen, mut ev)
    ev.push((1, u, via))
// reach[u][v]: a path of one or more arcs from u to v, with `skip` removed
fn reach_of(adj: List<List<Int>>, skip: Int) -> List<List<Bool>>:
    var r = [[false for _ in 0..6] for _ in 0..6]
    for u in 0..6:
        for v in adj[u]:
            if u != skip and v != skip: r[u][v] = true
    for w in 0..6:
        for u in 0..6:
            for v in 0..6:
                if r[u][w] and r[w][v]: r[u][v] = true
    r
fn kahn(adj: List<List<Int>>, least: Bool) -> List<Int>:
    var indeg = List.filled(6, 0)
    for u in 0..6:
        for v in adj[u]: indeg[v] += 1
    var q = [u for u in 0..6 if indeg[u] == 0]
    var out = List<Int>()
    var done = List.filled(6, false)
    while q.len > 0:
        var at = 0
        if least:
            for i in 0..q.len:
                if q[i] < q[at]: at = i
        let u = q[at]
        q.remove(at)
        out.push(u)
        for v in adj[u]:
            indeg[v] -= 1
            if indeg[v] == 0: q.push(v)
    out
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    let m = rng.int(0, 8)
    let dag = rng.int(0, 1) == 1
    var pairs = List<(Int, Int)>()
    for _ in 0..m:
        let (a, b) = (rng.int(0, 5), rng.int(0, 5))
        pairs.push(if dag and a > b: (b, a) elif dag and a == b: (a, (a + 1) % 6) else: (a, b))
    if dag: pairs = [p for p in pairs if p.0 < p.1]
    var adj = [List<Int>() for _ in 0..6]
    for (a, b) in pairs: adj[a].push(b)
    let vp = [(V(a), V(b)) for (a, b) in pairs]
    let g = directed_of_pairs(vp)
    let gu = undirected_of_pairs(vp)
    let ad = adjacency([(u: V(a), v: V(b)) for (a, b) in pairs])
    let gt = g.transpose()
    let gg = Graph.of([(from: V(a), arc: V(b)) for (a, b) in pairs])
    let ban = rng.int(0, 6)
    let only = g.only(keep: v => v.ord != ban)
    for u in V:
        let want = [V(v) for v in adj[u.ord]]
        var und = List<V>()
        for (a, b) in pairs:
            if a == u.ord: und.push(V(b))
            if b == u.ord: und.push(V(a))
        und.sort()
        var gus = List(gu[u])
        gus.sort()
        let tw = [V(a) for (a, b) in pairs if b == u.ord]
        var gts = List(gt[u])
        gts.sort()
        var tws = tw.clone()
        tws.sort()
        if g[u] != want or ad[u] != want or g.next(u).collect() != want or g.deg(u) != want.len: bad = "op {k}: directed {u}"
        elif gus != und: bad = "op {k}: undirected {u}"
        elif gt[u] != tws: bad = "op {k}: transpose {u}"
        elif gg.next(u).collect() != want or gg.deg(u) != want.len or gg[u].len != want.len or gg.lists()[u] != want: bad = "op {k}: Graph.of {u}"
        elif gg.transpose().next(u).collect().len != tw.len: bad = "op {k}: Graph.transpose {u}"
        elif V.any(v => gg.contains(u, v) != want.contains(v)): bad = "op {k}: contains {u}"
        let ow = if u.ord == ban: List<V>() else: [v for v in want if v.ord != ban]
        if only.out(u).collect() != ow or only.deg(u) != ow.len or g.only(from: v => v.ord != ban).deg(u) != (if u.ord == ban: 0 else: want.len): bad = "op {k}: only {u}"
        let cw = [v for v in V if v != u and not want.contains(v)]
        if complement(g).out(u).collect() != cw: bad = "op {k}: complement {u}"
    // edge-built graphs: arcs keep their edge and fields
    let es: [E](u: V, v: V) = [E] e => (u: V(rng.int(0, 5)), v: V(rng.int(0, 5)))
    let ws = [rng.int(0, 9) for _ in 0..8]
    let gd = Graph.directed(es)
    let gw = Graph.directed(es, keep: a => (to: a.to, w: ws[a.e.ord]))
    let gn = Graph.undirected(es)
    if gd.edges().collect() != [Edge(e, es[e].u, es[e].v) for e in E]: bad = "op {k}: edges"
    for u in V:
        let outs = [(to: es[e].v, e:) for e in E if es[e].u == u]
        if gd.out(u).collect() != outs or gd.at(u).to_list() != outs: bad = "op {k}: Graph.directed out {u}"
        if gw.out(u).collect().map(a => arc_len(a)) != [ws[e.ord] for e in E if es[e].u == u]: bad = "op {k}: keep {u}"
        if arc_len(u) != 1: bad = "op {k}: arc_len"
        let deg = E.count(e => es[e].u == u) + E.count(e => es[e].v == u)
        if gn.deg(u) != deg or gn.out(u).count() != deg: bad = "op {k}: Graph.undirected deg {u}"
    // hops: BFS distances on the undirected edge graph, when connected
    var hd: List<List<Int>> = []
    for a in V:
        var d = List.filled(6, -1)
        d[a.ord] = 0
        for _ in 0..6:
            for e in E:
                let (x, y) = (es[e].u.ord, es[e].v.ord)
                if d[x] >= 0 and (d[y] < 0 or d[x] + 1 < d[y]): d[y] = d[x] + 1
                if d[y] >= 0 and (d[x] < 0 or d[y] + 1 < d[x]): d[x] = d[y] + 1
        hd.push(d)
    if hd.all(d => d.all(x => x >= 0)):
        let h = gn.hops()
        for a in V:
            for b in V:
                if h[a, b] != hd[a.ord][b.ord]: bad = "op {k}: hops {a} {b}"
    // orders
    let r = reach_of(adj, 6)
    let cyclic = (0..6).any(u => r[u][u])
    let fifo = kahn(adj, false)
    if peel(g) != [V(x) for x in fifo] or gg.peel() != [V(x) for x in fifo]: bad = "op {k}: peel"
    if topo(g) != (if cyclic: None else: Some([V(x) for x in fifo])): bad = "op {k}: topo"
    if smallest_topo(g) != (if cyclic: None else: Some([V(x) for x in kahn(adj, true)])): bad = "op {k}: smallest_topo"
    if let cyc = find_cycle(g):
        for i in 0..cyc.len:
            if not adj[cyc[i].ord].contains(cyc[(i + 1) % cyc.len].ord) or cyc.count(x => x == cyc[i]) != 1: bad = "op {k}: find_cycle {i}"
    elif cyclic: bad = "op {k}: find_cycle missed"
    // strong components and condensation
    let (domain C, comp) = scc(g)
    let h2 = condense(g, comp)
    for u in V:
        for v in V:
            let same = u == v or (r[u.ord][v.ord] and r[v.ord][u.ord])
            if (comp[u] == comp[v]) != same: bad = "op {k}: scc {u} {v}"
        for x in adj[u.ord]:
            if comp[u] > comp[V(x)]: bad = "op {k}: scc order {u}"
    for c in C:
        let want = [(to: comp[V(b)]) for u in V if comp[u] == c, b in adj[u.ord] if comp[V(b)] != c]
        if h2[c].len != want.len: bad = "op {k}: condense"
    let rs = reach_sets(g)
    for u in V:
        for v in V:
            if rs[u].contains(v) != (u == v or r[u.ord][v.ord]): bad = "op {k}: reach_sets {u} {v}"
    // the DFS walk and reachability
    let s = rng.int(0, 5)
    var seen = List.filled(6, false)
    var ev = List<(Int, Int, Int)>()
    bdfs(adj, s, -1, mut seen, mut ev)
    var got = List<(Int, Int, Int)>()
    for e in dfs(g, from: [V(s)]):
        match e:
            Visit.Enter(v, via) => got.push((0, v.ord, if let a = via: a.ord else: -1))
            Visit.Exit(v, via) => got.push((1, v.ord, if let a = via: a.ord else: -1))
            Visit.Again(from, via) => got.push((2, from.ord, via.ord))
    if got != ev: bad = "op {k}: dfs events"
    let pre = [V(x.1) for x in ev if x.0 == 0]
    if reachable(g, from: [V(s)]).collect() != pre: bad = "op {k}: reachable"
    var cr = reachable(complement(g), from: [V(s)]).collect()
    cr.sort()
    var cadj = [[v for v in 0..6 if v != u and not adj[u].contains(v)] for u in 0..6]
    let crr = reach_of(cadj, 6)
    if cr != [v for v in V if v.ord == s or crr[s][v.ord]]: bad = "op {k}: reachable complement"
    // DAG folds: longest paths and path counts by layers of exactly L arcs
    if not cyclic:
        var lp = List.filled(6, 0)
        var cnt = List.filled(6, 0)
        var layer = List.filled(6, 0)
        layer[s] = 1
        for len in 0..7:
            var nx = List.filled(6, 0)
            for u in 0..6:
                cnt[u] += layer[u]
                for v in adj[u]:
                    nx[v] += layer[u]
            layer = nx
        for _ in 0..6:
            for (a, b) in pairs:
                lp[b] = max(lp[b], lp[a] + 1)
        if dag_fold(g, init: v => 0, step: (x, a) => x + 1, op: max) != Some([V] v => lp[v.ord]): bad = "op {k}: dag_fold"
        if count_paths<Int>(g, from: [V(s)]) != Some([V] v => cnt[v.ord]): bad = "op {k}: count_paths"
    elif dag_fold(g, init: v => 0, step: (x, a) => x + 1, op: max) != None or count_paths<Int>(g, from: [V(s)]) != None: bad = "op {k}: dag fold on a cycle"
    // dominators: d dominates v when cutting d makes v unreachable from s
    let rr = [[if x == s: true else: (reach_of(adj, d)[s][x]) for x in 0..6] for d in 0..7]
    let reach_s = [x == s or r[s][x] for x in 0..6]
    var doms = [List<Int>() for _ in 0..6]
    for v in 0..6:
        if reach_s[v]:
            for d in 0..6:
                if d == v or d == s or not rr[d][v]: doms[v].push(d)
    let dm = dominators(g, root: V(s))
    for v in 0..6:
        var want: ?V = None
        if reach_s[v] and v != s:
            for d in doms[v]:
                if d != v and doms[d].len == doms[v].len - 1: want = Some(V(d))
        if dm[V(v)] != want: bad = "op {k}: dominators {v}"
    // a simple undirected graph: colouring, odd cycles, girth, triangles, cliques, colours
    var und = [List<Int>() for _ in 0..6]
    var am = [[false for _ in 0..6] for _ in 0..6]
    for a in 0..6:
        for b in a + 1..6:
            if rng.int(0, 2) == 0:
                am[a][b] = true
                am[b][a] = true
    for a in 0..6:
        for b in 0..6:
            if am[a][b]: und[a].push(b)
    let ug = [V] v => [V(x) for x in und[v.ord]]
    // parity reachability decides bipartite
    var odd = false
    for a in 0..6:
        var par = [[false, false] for _ in 0..6]
        par[a][0] = true
        for _ in 0..13:
            for x in 0..6:
                for y in und[x]:
                    for p in 0..2:
                        if par[x][p]: par[y][1 - p] = true
        if par[a][1]: odd = true
    if let col = two_color(ug):
        if odd or (0..6).any(x => und[x].any(y => col[V(x)] == col[V(y)])): bad = "op {k}: two_color"
    elif not odd: bad = "op {k}: two_color missed"
    if let oc = odd_cycle(ug):
        if oc.len % 2 == 0 or (0..oc.len).any(i => not am[oc[i].ord][oc[(i + 1) % oc.len].ord]): bad = "op {k}: odd_cycle"
    elif odd: bad = "op {k}: odd_cycle missed"
    // components numbered by least vertex
    let (domain K, kc) = components(ug)
    var first = List.filled(6, -1)
    var ranks = 0
    let ur = reach_of(und, 6)
    for x in 0..6:
        var lo = x
        for y in 0..6:
            if ur[x][y] and y < lo: lo = y
        if lo == x:
            first[x] = ranks
            ranks += 1
        if Int(kc[V(x)]) != first[lo]: bad = "op {k}: components {x}"
    if K.count != ranks: bad = "op {k}: components count"
    // girth: shortest cycle through an edge, by removing it
    var gir = 100
    for a in 0..6:
        for b in und[a]:
            var d = List.filled(6, 100)
            d[a] = 0
            for _ in 0..6:
                for x in 0..6:
                    for y in und[x]:
                        if not ((x == a and y == b) or (x == b and y == a)) and d[x] + 1 < d[y]: d[y] = d[x] + 1
            if d[b] + 1 < gir: gir = d[b] + 1
    if girth(ug, directed: false) != (if gir >= 100: None else: Some(gir)): bad = "op {k}: girth undirected"
    var dg = 100
    for a in 0..6:
        for (x, y) in pairs:
            if y == a and (x == a or r[a][x]):
                var d = List.filled(6, 100)
                d[a] = 0
                for _ in 0..6:
                    for (p, q) in pairs:
                        if d[p] + 1 < d[q]: d[q] = d[p] + 1
                if d[x] + 1 < dg: dg = d[x] + 1
    if girth(g, directed: true) != (if dg >= 100: None else: Some(dg)): bad = "op {k}: girth directed"
    var tri = 0
    for a in 0..6:
        for b in a + 1..6:
            for c in b + 1..6:
                if am[a][b] and am[b][c] and am[a][c]: tri += 1
    let ts = triangles(ug).collect()
    if ts.len != tri or ts.any(t => not (am[t.0.ord][t.1.ord] and am[t.1.ord][t.2.ord] and am[t.0.ord][t.2.ord])): bad = "op {k}: triangles"
    let bs = [V] v => Bitset.of([V(y) for y in und[v.ord]])
    var best = 0
    var indep = List.filled(64, false)
    for mask in 0..64:
        let mem = [x for x in 0..6 if (mask >> x) & 1 == 1]
        if mem.all(x => mem.all(y => x == y or am[x][y])) and mem.len > best: best = mem.len
        indep[mask] = mem.all(x => mem.all(y => not am[x][y]))
    let cl = max_clique(bs)
    if cl.len != best or cl.any(x => cl.any(y => x != y and not am[x.ord][y.ord])) or cl != cl.sorted(): bad = "op {k}: max_clique"
    var f = List.filled(64, 100)
    f[0] = 0
    for mask in 1..64:
        var sub = mask
        while sub > 0:
            if indep[sub] and f[mask ^ sub] + 1 < f[mask]: f[mask] = f[mask ^ sub] + 1
            sub = (sub - 1) & mask
    if chromatic_number(bs) != f[63]: bad = "op {k}: chromatic_number"
    if bad != "": break
print(if bad == "": "ok" else: bad)

// expect: ok, output "ok"
// input: "1 300\n"
// model: Paths
// model: Reweighted
// model: bfs
// model: bfs01
// model: dijkstra
// model: dial
// model: bellman_ford
// model: floyd_warshall
// model: closure
// model: johnson
// model: negative_cycle
// model: min_mean_cycle
// model: k_shortest
// model: fixed_cost_paths
// The shortest-path family against edge-list relaxation: each op draws a random digraph on 6
// vertices with 9 weighted edges (0/1, non-negative, or with negative arcs), runs every search
// from random sources and checks distances, witness paths and arcs, iter order, tight arcs,
// negative cycles, the least mean cycle and the k shortest walks. Reads `seed n`.
use std.graph.{Graph, bellman_ford, bfs, bfs01, dial, dijkstra, fixed_cost_paths, floyd_warshall, johnson, k_shortest, min_mean_cycle, negative_cycle}
use std.rand.{Rng}
let seed0 = read<Int>()
let n = read<Int>()
domain V = 0..6
domain E = 0..9
domain S = 0..20
fn inf() -> Int = 1_000_000_000
fn opt(x: Int) -> ?Int = if x >= inf(): None else: Some(x)
fn relax(eu: List<Int>, ev: List<Int>, ew: List<Int>, src: List<(Int, Int)>, bott: Bool, ban: Int) -> ?List<Int>:
    var d = List.filled(6, inf())
    for (s, d0) in src:
        if s != ban and d0 < d[s]: d[s] = d0
    for round in 0..8:
        var ch = false
        for i in 0..eu.len:
            if d[eu[i]] < inf() and ev[i] != ban:
                let nd = if bott: max(d[eu[i]], ew[i]) else: d[eu[i]] + ew[i]
                if nd < d[ev[i]]:
                    d[ev[i]] = nd
                    ch = true
        if not ch: return Some(d)
    None
// a witness walk: vertices `path`, arc ids `ids`, from a source at its start to `t`, costing `want`
fn walk_ok(path: List<V>, ids: List<Int>, eu: List<Int>, ev: List<Int>, ew: List<Int>, src: List<(Int, Int)>, t: Int, want: Int, bott: Bool) -> Bool:
    if path.len != ids.len + 1 or path[path.len - 1].ord != t: return false
    var cost = inf()
    for (s, d0) in src:
        if s == path[0].ord and d0 < cost: cost = d0
    if cost >= inf(): return false
    for i in 0..ids.len:
        let e = ids[i]
        if eu[e] != path[i].ord or ev[e] != path[i + 1].ord: return false
        cost = if bott: max(cost, ew[e]) else: cost + ew[e]
    cost == want
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    let kind = rng.int(0, 3)
    let eu = [rng.int(0, 5) for _ in 0..9]
    let ev = [rng.int(0, 5) for _ in 0..9]
    let ew = [(if kind == 0: rng.int(0, 1) elif kind == 1: rng.int(0, 9) elif kind == 2: rng.int(1, 9) else: rng.int(-3, 9)) for _ in 0..9]
    let es: [E](u: V, v: V, w: Int) = [E] e => (u: V(eu[e.ord]), v: V(ev[e.ord]), w: ew[e.ord])
    let plain: [E](u: V, v: V) = [E] e => (u: V(eu[e.ord]), v: V(ev[e.ord]))
    let g = Graph.directed(es)
    let rw = Graph.directed(plain).weighted(a => ew[a.e.ord])
    let s = rng.int(0, 5)
    let s2 = rng.int(0, 5)
    let d0 = rng.int(0, 3)
    let ones = [1 for _ in 0..9]
    let ban = rng.int(0, 6)
    let t = rng.int(0, 5)
    // bfs: unit lengths, a banned vertex through `enter`, and `until`
    let unit = relax(eu, ev, ones, [(s, 0), (s2, 0)], false, ban)!
    let p = bfs(g, from: [V(s), V(s2)], enter: (v, d) => v.ord != ban)
    var last = 0
    var seen = 0
    for (v, d) in p.iter():
        if d < last or Some(d) != opt(unit[v.ord]): bad = "op {k}: bfs iter {v} {d}"
        last = d
        seen += 1
    if seen != unit.count(x => x < inf()): bad = "op {k}: bfs iter count {seen}"
    let tight = p.tight(g)
    for v in V:
        if p[v] != opt(unit[v.ord]) or p.reached(v) != (unit[v.ord] < inf()): bad = "op {k}: bfs {v}"
        let want_tight = (0..9).count(i => eu[i] == v.ord and unit[v.ord] < inf() and unit[ev[i]] == unit[v.ord] + 1)
        if tight[v].len != want_tight: bad = "op {k}: tight {v}"
        if let path = p.path(v):
            let ids = [a.e.ord for a in p.arcs(v)!]
            if not walk_ok(path, ids, eu, ev, ones, [(s, 0), (s2, 0)], v.ord, unit[v.ord], false): bad = "op {k}: bfs path {v}"
        elif p.arcs(v) != None: bad = "op {k}: bfs arcs {v}"
    let pu = bfs(g, from: [V(s)], until: V(t))
    let full = relax(eu, ev, ones, [(s, 0)], false, 6)!
    if pu[V(t)] != opt(full[t]): bad = "op {k}: bfs until {t}"
    // the all-pairs answers, and whether any negative cycle exists
    var all: List<List<Int>> = []
    var neg = false
    for a in 0..6:
        if let row = relax(eu, ev, ew, [(a, 0)], false, 6): all.push(row)
        else:
            neg = true
            all.push(List.filled(6, 0))
    var m = g.matrix()
    let c = m.closure()
    if floyd_warshall(mut m) == neg or (c == None) != neg: bad = "op {k}: floyd_warshall says {yes(not neg)}"
    let j = johnson(g)
    if (j == None) != neg: bad = "op {k}: johnson"
    if not neg:
        for a in V:
            for b in V:
                let want = opt(all[a.ord][b.ord])
                if m[a, b] != want or c![a, b] != want or j![a, b] != want: bad = "op {k}: all pairs {a} {b}"
    if let cyc = negative_cycle(g):
        var sum = 0
        for i in 0..cyc.len:
            sum += cyc[i].w
            if cyc[i].to.ord != eu[cyc[(i + 1) % cyc.len].e.ord]: bad = "op {k}: negative cycle breaks at {i}"
        if sum >= 0 or not neg: bad = "op {k}: negative cycle sum {sum}"
    elif neg: bad = "op {k}: negative cycle missed"
    // single-source searches by weight regime
    let src = [(s, d0), (s2, rng.int(0, 3))]
    let from = [(V(x), y) for (x, y) in src]
    let bf = bellman_ford(g, from: from)
    let want = relax(eu, ev, ew, src, false, 6)
    if (bf == None) != (want == None): bad = "op {k}: bellman_ford reach of a negative cycle"
    var runs = [bf]
    if kind <= 2:
        runs.push(Some(dijkstra(g, from: from)))
        runs.push(Some(dijkstra(rw, from: from)))
    if kind == 0: runs.push(Some(bfs01(g, from: [V(s), V(s2)])))
    if kind <= 2 and d0 == 0 and src[1].1 == 0: runs.push(Some(dial(g, from: [V(s), V(s2)], max_w: 9)))
    for r in 0..runs.len:
        if let q = runs[r]:
            let zero = r >= 3
            let w = if zero: relax(eu, ev, ew, [(s, 0), (s2, 0)], false, 6)! else: want!
            let base = if zero: [(s, 0), (s2, 0)] else: src
            for v in V:
                if q[v] != opt(w[v.ord]): bad = "op {k}: search {r} dist {v}"
                elif let path = q.path(v):
                    let ids = [a.e.ord for a in q.arcs(v)!]
                    if not walk_ok(path, ids, eu, ev, ew, base, v.ord, w[v.ord], false): bad = "op {k}: search {r} path {v}"
    if kind <= 2:
        let bw = relax(eu, ev, ew, src, true, 6)!
        let q = dijkstra(g, from: from, step: max)
        for v in V:
            if q[v] != opt(bw[v.ord]): bad = "op {k}: bottleneck {v}"
            elif let path = q.path(v):
                let ids = [a.e.ord for a in q.arcs(v)!]
                if not walk_ok(path, ids, eu, ev, ew, src, v.ord, bw[v.ord], true): bad = "op {k}: bottleneck path {v}"
    // least mean cycle: closed walks of each length 1..6 through each vertex
    var best: ?(sum: Int, len: Int) = None
    for a in 0..6:
        var d = List.filled(6, inf())
        d[a] = 0
        for len in 1..7:
            var nd = List.filled(6, inf())
            for i in 0..9:
                if d[eu[i]] < inf() and d[eu[i]] + ew[i] < nd[ev[i]]: nd[ev[i]] = d[eu[i]] + ew[i]
            d = nd
            if d[a] < inf():
                if let b = best:
                    if d[a] * b.len < b.sum * len or (d[a] * b.len == b.sum * len and len < b.len): best = (sum: d[a], len:)
                else: best = (sum: d[a], len:)
    if min_mean_cycle(g) != best: bad = "op {k}: min_mean_cycle"
    // k shortest walks with positive weights: walk counts by total weight
    if kind == 2:
        let kk = rng.int(1, 4)
        var cnt: List<List<Int>> = [List.filled(6, 0) for _ in 0..400]
        cnt[0][s] = 1
        var ws = List<Int>()
        for tot in 0..400:
            for i in 0..9:
                if tot >= ew[i]: cnt[tot][ev[i]] = min(kk, cnt[tot][ev[i]] + cnt[tot - ew[i]][eu[i]])
            for _ in 0..cnt[tot][t]:
                if ws.len < kk: ws.push(tot)
        if k_shortest(g, from: [V(s)], to: V(t), k: kk) != ws: bad = "op {k}: k_shortest {kk}"
    // a state space with fixed move costs
    let moves = [rng.int(-4, 6) for _ in 0..3]
    let price = [rng.int(0, 5) for _ in 0..3]
    let start = rng.int(0, 19)
    var sd = List.filled(20, inf())
    sd[start] = 0
    for _ in 0..21:
        for x in 0..20:
            for i in 0..3:
                let y = x + moves[i]
                if sd[x] < inf() and y >= 0 and y < 20 and sd[x] + price[i] < sd[y]: sd[y] = sd[x] + price[i]
    let fp = fixed_cost_paths([S(start)], [0, 1, 2], cost: i => price[i], step: (i, x) => if Int(x) + moves[i] >= 0 and Int(x) + moves[i] < 20: Some(S(Int(x) + moves[i])) else: None)
    for x in S:
        if fp[x] != opt(sd[x.ord]): bad = "op {k}: fixed_cost_paths {x}"
        elif let path = fp.path(x):
            let via = fp.arcs(x)!
            var cost = 0
            for i in 0..via.len:
                if path[i].ord + moves[via[i]] != path[i + 1].ord: bad = "op {k}: fixed_cost_paths step {x}"
                cost += price[via[i]]
            if cost != sd[x.ord] or path[0].ord != start: bad = "op {k}: fixed_cost_paths path {x}"
    if bad != "": break
print(if bad == "": "ok" else: bad)

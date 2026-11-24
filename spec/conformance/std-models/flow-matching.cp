// expect: ok, output "ok"
// input: "1 300\n"
// model: Matching
// model: max_matching
// model: min_vertex_cover
// model: max_weight_matching
// model: min_weight_matching
// model: general_matching
// model: max_weight_general_matching
// model: best_matching
// model: stable_matching
// model: edge_coloring
// model: matroid_intersection
// model: GraphicMatroid
// model: ColorMatroid
// Matchings against subset enumeration on 3 or 4 vertices a side: maximum and weighted
// bipartite matchings by a DP over used right vertices, general matchings by a DP over vertex
// masks, stable matchings against every stable assignment, edge colourings by checking every
// vertex, matroid intersection against every edge subset. Reads `seed n`.
use std.flow.{ColorMatroid, GraphicMatroid, Matching, best_matching, edge_coloring, general_matching, matroid_intersection, max_matching, max_weight_general_matching, max_weight_matching, min_vertex_cover, min_weight_matching, stable_matching}
use std.rand.{Rng}
let seed0 = read<Int>()
let n = read<Int>()
domain L = 0..4
domain R = 0..4
domain P = 0..3
domain V = 0..6
domain E = 0..6
domain C = 0..3
let neg = -1_000_000
// best total over matchings of left vertices into distinct right ones; w[l][r] == neg forbids
// the pair, `all` forces every left vertex to match
fn assign(w: List<List<Int>>, nr: Int, all: Bool) -> Int:
    var dp = List.filled(1 << nr, neg)
    dp[0] = 0
    for l in 0..w.len:
        var next = if all: List.filled(1 << nr, neg) else: dp.clone()
        for mask in 0..(1 << nr):
            if dp[mask] > neg:
                for r in 0..nr:
                    if (mask >> r) & 1 == 0 and w[l][r] > neg:
                        next[mask | (1 << r)] = max(next[mask | (1 << r)], dp[mask] + w[l][r])
        dp = next
    dp.max()!
// best total over matchings of a 6-vertex graph, weights w[u][v] (neg: no edge)
fn gmatch(w: List<List<Int>>) -> Int:
    var dp = List.filled(64, 0)
    for mask in 1..64:
        var low = 0
        while (mask >> low) & 1 == 0: low += 1
        let rest = mask ^ (1 << low)
        dp[mask] = dp[rest]
        for v in 0..6:
            if (rest >> v) & 1 == 1 and w[low][v] > neg: dp[mask] = max(dp[mask], dp[rest ^ (1 << v)] + w[low][v])
    dp[63]
fn lab(eu: List<Int>, ev: List<Int>, keep: List<Bool>) -> List<Int>:
    var l = [i for i in 0..4]
    var ch = true
    while ch:
        ch = false
        for i in 0..eu.len:
            if keep[i] and l[eu[i]] != l[ev[i]]:
                let m = min(l[eu[i]], l[ev[i]])
                l[eu[i]] = m
                l[ev[i]] = m
                ch = true
    l
// independent in the graphic matroid: every kept edge joins two different trees when added
fn forest(eu: List<Int>, ev: List<Int>, keep: List<Bool>) -> Bool:
    let comps = (0..4).count(x => lab(eu, ev, keep)[x] == x)
    eu[0] >= 0 and keep.count(x => x) == 4 - comps and (0..eu.len).all(i => not keep[i] or eu[i] != ev[i])
fn distinct(xs: List<Int>) -> Bool = (0..xs.len).all(i => (0..i).all(j => xs[i] != xs[j]))
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    let adj: [L]List<R> = [L] _ => [R(rng.int(0, 3)) for _ in 0..rng.int(0, 3)]
    let ok = [[if adj[L(l)].contains(R(r)): 1 else: neg for r in 0..4] for l in 0..4]
    let size = assign(ok, 4, false)
    let mm = max_matching(adj)
    if mm.size != size or (0..4).count(l => mm.mate[L(l)] != None) != size: bad = "op {k}: max_matching size"
    for l in L:
        if let r = mm.mate[l]:
            if not adj[l].contains(r) or mm.back[r] != Some(l): bad = "op {k}: max_matching mate {l}"
    for r in R:
        if let l = mm.back[r]:
            if mm.mate[l] != Some(r): bad = "op {k}: max_matching back {r}"
    let cover = min_vertex_cover(adj)
    if (0..4).count(l => cover.left[L(l)]) + (0..4).count(r => cover.right[R(r)]) != size or L.any(l => adj[l].any(r => not cover.left[l] and not cover.right[r])): bad = "op {k}: min_vertex_cover"
    // online matching: random adds, augments and removals against the DP size
    var om = Matching<L, R>()
    var have = [[neg for _ in 0..4] for _ in 0..4]
    for step in 0..8:
        let op = rng.int(0, 4)
        let l = rng.int(0, 3)
        if op <= 1:
            let r = rng.int(0, 3)
            om.add(L(l), R(r))
            have[l][r] = 1
        elif op == 2:
            let before = om.len
            let free = om.mate(L(l)) == None
            // matchable when some matching covers the matched left vertices and l
            var req = [[if om.mate(L(x)) != None or x == l: have[x][r] else: 0 for r in 0..4] for x in 0..4]
            let need = (0..4).count(x => om.mate(L(x)) != None or x == l)
            var ww = [[if have[x][r] > neg: (if om.mate(L(x)) != None or x == l: 10 else: 0) else: neg for r in 0..4] for x in 0..4]
            let can = free and assign(ww, 4, false) >= need * 10
            if om.augment(L(l)) != can or om.len != before + (if can: 1 else: 0): bad = "op {k}: augment {step}"
        elif op == 3:
            om.augment_all()
            if om.len != assign(have, 4, false): bad = "op {k}: augment_all {step}"
        else:
            let maxed = om.len == assign(have, 4, false)
            let got = om.remove(L(l))
            if got != om.len or (maxed and got != assign(have, 4, false)): bad = "op {k}: remove {step}"
        var pairs = List<(L, R)>()
        for x in L:
            if let r = om.mate(x):
                pairs.push((x, r))
                if have[x.ord][r.ord] == neg or om.back(r) != Some(x): bad = "op {k}: Matching pair {step}"
        if om.items.collect() != pairs or om.len != pairs.len or om.is_empty != (pairs.len == 0): bad = "op {k}: Matching items {step}"
        let cv = om.cover()
        if om.len == assign(have, 4, false):
            if (0..4).count(x => cv.left[L(x)]) + (0..4).count(r => cv.right[R(r)]) != om.len or (0..4).any(x => (0..4).any(r => have[x][r] > neg and not cv.left[L(x)] and not cv.right[R(r)])): bad = "op {k}: Matching cover {step}"
    // assignments: every left vertex of 3 into 4 right vertices
    let w = [[rng.int(-5, 9) for _ in 0..4] for _ in 0..3]
    let wneg = [[-x for x in row] for row in w]
    let mx = max_weight_matching<P, R>(weight: (l, r) => w[l.ord][r.ord])
    let mn = min_weight_matching<P, R>(weight: (l, r) => w[l.ord][r.ord])
    for (res, want, name) in [(mx, assign(w, 4, true), 0), (mn, -assign(wneg, 4, true), 1)]:
        let rs = [res.mate[P(l)].ord for l in 0..3]
        if res.total != want or (0..3).sum(l => w[l][rs[l]]) != want or not distinct(rs): bad = "op {k}: weight_matching {name}"
    // general matchings on 6 vertices
    var gw = [[neg for _ in 0..6] for _ in 0..6]
    var gadj = [List<V>() for _ in 0..6]
    var ges = List<(u: V, v: V, w: Int)>()
    for _ in 0..6:
        let (u, v) = (rng.int(0, 5), rng.int(0, 5))
        let x = rng.int(-2, 6)
        ges.push((u: V(u), v: V(v), w: x))
        if u != v:
            if gw[u][v] == neg:
                gadj[u].push(V(v))
                gadj[v].push(V(u))
            gw[u][v] = max(gw[u][v], x)
            gw[v][u] = gw[u][v]
    let gm = general_matching([V] v => gadj[v.ord])
    let unit = [[if x > neg: 1 else: neg for x in row] for row in gw]
    if gm.size != gmatch(unit) or (0..6).count(v => gm.mate[V(v)] != None) != 2 * gm.size or V.any(v => if let u = gm.mate[v]: gm.mate[u] != Some(v) or gw[v.ord][u.ord] == neg else: false): bad = "op {k}: general_matching"
    let gwm = max_weight_general_matching([E] e => ges[e.ord])
    let pos = [[if x > 0: x else: neg for x in row] for row in gw]
    var gsum = 0
    for v in V:
        if let u = gwm.mate[v]:
            if gwm.mate[u] != Some(v) or gw[v.ord][u.ord] == neg: bad = "op {k}: max_weight_general_matching mate"
            elif v < u: gsum += gw[v.ord][u.ord]
    if gwm.total != gmatch(pos) or gsum != gwm.total: bad = "op {k}: max_weight_general_matching total"
    // best matching over 6 listed edges, optionally of a forced size
    let bl = [rng.int(0, 2) for _ in 0..6]
    let br = [rng.int(0, 2) for _ in 0..6]
    let bw = [rng.int(-3, 6) for _ in 0..6]
    let bes: [E](l: P, r: P, w: Int) = [E] e => (l: P(bl[e.ord]), r: P(br[e.ord]), w: bw[e.ord])
    let want_size: ?Int = if rng.int(0, 1) == 0: Some(rng.int(0, 3)) else: None
    let lowest = rng.int(0, 1) == 1
    var bbest: ?(Int, Int) = None
    for mask in 0..64:
        let used = [i for i in 0..6 if (mask >> i) & 1 == 1]
        if distinct([bl[i] for i in used]) and distinct([br[i] for i in used]) and (want_size == None or want_size == Some(used.len)):
            let t = used.sum(i => bw[i]) * (if lowest: -1 else: 1)
            if bbest == None or t > bbest!.0 or (t == bbest!.0 and used.len < bbest!.1): bbest = Some((t, used.len))
    if let bm = best_matching(bes, size: want_size, min: lowest):
        let used = [i for i in 0..6 if bm.used[E(i)]]
        let t = used.sum(i => bw[i])
        let pairs = used.len
        let sign = if lowest: -1 else: 1
        if bbest == None or bm.total != t or t * sign != bbest!.0 or (want_size == None and pairs != bbest!.1) or (want_size != None and Some(pairs) != want_size) or (0..3).count(l => bm.mate[P(l)] != None) != pairs or not used.all(i => bm.mate[P(bl[i])] == Some(P(br[i]))): bad = "op {k}: best_matching"
    elif bbest != None: bad = "op {k}: best_matching missed"
    // stable matching: the left-best of all stable assignments
    let pl: [P]List<P> = [P] _ => [P(x) for x in [0, 1, 2] if rng.int(0, 3) > 0].sorted(by: _ => rng.int(0, 99))
    let pr: [P]List<P> = [P] _ => [P(x) for x in [0, 1, 2] if rng.int(0, 3) > 0].sorted(by: _ => rng.int(0, 99))
    fn rank(pref: List<P>, x: P) -> Int:
        for i in 0..pref.len:
            if pref[i] == x: return i
        99
    var stables = List<List<Int>>()
    for code in 0..64:
        let a = [(code >> (2 * l)) & 3 for l in 0..3]
        var fine = a.all(x => x <= 3)
        for l in 0..3:
            if a[l] < 3 and (rank(pl[P(l)], P(a[l])) == 99 or rank(pr[P(a[l])], P(l)) == 99): fine = false
            for m in 0..3:
                if m != l and a[l] < 3 and a[l] == a[m]: fine = false
        if fine:
            for l in 0..3:
                for r in 0..3:
                    let lr = rank(pl[P(l)], P(r))
                    let rl = rank(pr[P(r)], P(l))
                    let lcur = if a[l] < 3: rank(pl[P(l)], P(a[l])) else: 98
                    let holder = [m for m in 0..3 if a[m] == r]
                    let rcur = if holder.len > 0: rank(pr[P(r)], P(holder[0])) else: 98
                    if lr < 99 and rl < 99 and lr < lcur and rl < rcur: fine = false
        if fine: stables.push(a)
    let sm = stable_matching(pl, pr)
    let got = [if let r = sm[P(l)]: r.ord else: 3 for l in 0..3]
    let pick_rank = (l: Int, r: Int) => if r < 3: rank(pl[P(l)], P(r)) else: 98
    if not stables.contains(got) or stables.any(s => (0..3).any(l => pick_rank(l, s[l]) < pick_rank(l, got[l]))): bad = "op {k}: stable_matching"
    // edge colouring with max-degree colours
    let el = [rng.int(0, 2) for _ in 0..6]
    let er = [rng.int(0, 2) for _ in 0..6]
    let ec = edge_coloring([E] e => (l: P(el[e.ord]), r: P(er[e.ord])))
    let deg = (0..3).map(x => max(el.count(y => y == x), er.count(y => y == x))).max()!
    if ec.colors != deg or (0..6).any(i => ec.color[E(i)] < 0 or ec.color[E(i)] >= deg or (0..i).any(j => ec.color[E(i)] == ec.color[E(j)] and (el[i] == el[j] or er[i] == er[j]))): bad = "op {k}: edge_coloring"
    // matroid intersection: forests of a 4-vertex graph with at most limit[c] edges per colour
    let mu = [rng.int(0, 3) for _ in 0..6]
    let mv = [rng.int(0, 3) for _ in 0..6]
    let col = [rng.int(0, 2) for _ in 0..6]
    let lim = [rng.int(0, 2) for _ in 0..3]
    let mwt = [rng.int(-3, 5) for _ in 0..6]
    let mes: [E](u: P, v: P) = [E] e => (u: P(min(mu[e.ord], 2)), v: P(min(mv[e.ord], 2)))
    let fu = [min(x, 2) for x in mu]
    let fv = [min(x, 2) for x in mv]
    var msize = 0
    var mweight = neg
    fn indep(fu: List<Int>, fv: List<Int>, col: List<Int>, lim: List<Int>, keep: List<Bool>) -> Bool = forest(fu, fv, keep) and (0..3).all(c => (0..6).count(i => keep[i] and col[i] == c) <= lim[c])
    for mask in 0..64:
        let keep = [(mask >> i) & 1 == 1 for i in 0..6]
        if indep(fu, fv, col, lim, keep):
            let sz = keep.count(x => x)
            let wt = (0..6).sum(i => if keep[i]: mwt[i] else: 0)
            if sz > msize or (sz == msize and wt > mweight):
                msize = sz
                mweight = wt
            elif sz == msize: mweight = max(mweight, wt)
    let colors: [E]C = [E] e => C(col[e.ord])
    let limits: [C]Int = [C] c => lim[c.ord]
    let plain = matroid_intersection<E>(GraphicMatroid(mes), ColorMatroid(colors, limits))
    let weighted = matroid_intersection<E>(GraphicMatroid(mes), ColorMatroid(colors, limits), weight: e => mwt[e.ord])
    for (res, check_w) in [(plain, false), (weighted, true)]:
        let keep = [res[E(i)] for i in 0..6]
        let wt = (0..6).sum(i => if keep[i]: mwt[i] else: 0)
        if not indep(fu, fv, col, lim, keep) or keep.count(x => x) != msize or (check_w and wt != mweight): bad = "op {k}: matroid_intersection {yes(check_w)}"
    if bad != "": break
print(if bad == "": "ok" else: bad)

// expect: ok, output "ok"
// input: "1 300\n"
// model: Arc
// model: FlowGraph
// model: CostFlow
// model: max_closure
// model: circulation
// model: max_bounded_flow
// model: min_bounded_flow
// model: min_cost_circulation
// model: gomory_hu
// model: global_min_cut
// Flow networks against enumeration: maximum flows against the least cut over every vertex
// subset, min-cost flows, circulations and bounded flows against every integer flow vector of a
// few small arcs, closures against every vertex subset. Reads `seed n`.
use std.flow.{Arc, CostFlow, FlowGraph, circulation, global_min_cut, gomory_hu, max_bounded_flow, max_closure, min_bounded_flow, min_cost_circulation}
use std.rand.{Rng}
let seed0 = read<Int>()
let n = read<Int>()
domain V = 0..5
domain W = 0..4
domain U = 0..3
domain B = 0..4
domain G = 0..5
let big = 1 << 60
// the least capacity over cuts with `s` inside and `t` outside (directed arcs)
fn mincut(du: List<Int>, dv: List<Int>, dc: List<Int>, nv: Int, s: Int, t: Int) -> Int:
    var best = big
    for mask in 0..(1 << nv):
        if (mask >> s) & 1 == 1 and (mask >> t) & 1 == 0:
            var c = 0
            for i in 0..du.len:
                if (mask >> du[i]) & 1 == 1 and (mask >> dv[i]) & 1 == 0: c = min(big, c + dc[i])
            if c < best: best = c
    best
// every integer vector with lo[i] <= f[i] <= hi[i]
fn vectors(lo: List<Int>, hi: List<Int>) -> List<List<Int>>:
    var out = [List<Int>()]
    for i in 0..lo.len:
        var next = List<List<Int>>()
        for f in out:
            for x in lo[i]..hi[i] + 1: next.push(f + [x])
        out = next
    out
// net flow out of each vertex
fn net(eu: List<Int>, ev: List<Int>, f: List<Int>, nv: Int) -> List<Int>:
    var out = List.filled(nv, 0)
    for i in 0..eu.len:
        out[eu[i]] += f[i]
        out[ev[i]] -= f[i]
    out
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    // FlowGraph: max flow 0 -> 4 against the least cut
    let m = rng.int(1, 7)
    var eu = List<Int>()
    var ev = List<Int>()
    var ec = List<Int>()
    var both = List<Bool>()
    var g = FlowGraph<V>()
    var arcs = List<Arc>()
    for _ in 0..m:
        let (u, v) = (rng.int(0, 4), rng.int(0, 4))
        let r = if u == 4 or v == 4: rng.int(1, 9) else: rng.int(0, 9)
        let c = if r == 0: big else: rng.int(0, 4)
        eu.push(u)
        ev.push(v)
        ec.push(c)
        both.push(r == 1)
        arcs.push(if r == 1: g.add_both(V(u), V(v), c) elif r == 0: g.add(V(u), V(v), None) else: g.add(V(u), V(v), Some(c)))
    fn directed(eu: List<Int>, ev: List<Int>, ec: List<Int>, both: List<Bool>) -> (List<Int>, List<Int>, List<Int>):
        var (du, dv, dc) = (eu.clone(), ev.clone(), ec.clone())
        for i in 0..eu.len:
            if both[i]:
                du.push(ev[i])
                dv.push(eu[i])
                dc.push(ec[i])
        (du, dv, dc)
    let (du, dv, dc) = directed(eu, ev, ec, both)
    let mc = mincut(du, dv, dc, 5, 0, 4)
    let limit: ?Int = if rng.int(0, 2) == 0: Some(rng.int(0, 6)) else: None
    let value = g.max_flow(V(0), V(4), limit:)
    let want = if let l = limit: min(l, mc) else: mc
    if value != want: bad = "op {k}: max_flow {value} want {want}"
    var fl = List<Int>()
    for i in 0..m:
        let a = arcs[i]
        let f = g.flow(a)
        fl.push(f)
        let lo = if both[i]: -ec[i] else: 0
        if g.cap(a) != ec[i] or g.cap(a.rev) != (if both[i]: ec[i] else: 0) or a.rev.rev != a: bad = "op {k}: cap {i}"
        if g.flow(a.rev) != -f or f < lo or f > ec[i] or g.residual(a) != g.cap(a) - f or g.residual(a.rev) != g.cap(a.rev) + f: bad = "op {k}: flow {i}"
        if g.source(a) != V(eu[i]) or g.target(a) != V(ev[i]) or g.source(a.rev) != V(ev[i]): bad = "op {k}: ends {i}"
    let nt = net(eu, ev, fl, 5)
    if nt[0] != value or nt[1] != 0 or nt[2] != 0 or nt[3] != 0: bad = "op {k}: conservation"
    if g.arcs.collect() != arcs: bad = "op {k}: arcs"
    for u in 0..5:
        if g.out(V(u)).collect() != [arcs[i] for i in 0..m if eu[i] == u]: bad = "op {k}: out {u}"
    var claimed = 0
    for p in g.paths(V(0), V(4)):
        claimed += p.amount
        var at = V(0)
        for a in p.arcs:
            if g.source(a) != at or g.flow(a) < p.amount: bad = "op {k}: paths step"
            at = g.target(a)
        if at != V(4) or p.amount <= 0: bad = "op {k}: paths end"
    if claimed != value: bad = "op {k}: paths sum {claimed} {value}"
    if limit == None:
        let side = g.min_cut(V(0))
        var cut = 0
        for i in 0..du.len:
            if side[V(du[i])] and not side[V(dv[i])]: cut += dc[i]
        if not side[V(0)] or side[V(4)] or cut != value: bad = "op {k}: min_cut"
        var sum = 0
        for a in g.cut_arcs(V(0)):
            sum += g.cap(a)
            if not side[g.source(a)] or side[g.target(a)] or g.residual(a) != 0: bad = "op {k}: cut_arcs"
        if sum != value: bad = "op {k}: cut_arcs sum"
    let pick = rng.int(0, m - 1)
    if not both[pick] and ec[pick] < big:
        ec[pick] += rng.int(0, 3)
        g.set_cap(arcs[pick], Some(ec[pick]))
    let (du2, dv2, dc2) = directed(eu, ev, ec, both)
    let mc2 = mincut(du2, dv2, dc2, 5, 0, 4)
    if g.max_flow(V(0), V(4)) != mc2 - value: bad = "op {k}: set_cap then max_flow"
    g.reset()
    if arcs.any(a => g.flow(a) != 0) or g.max_flow(V(0), V(4)) != mc2: bad = "op {k}: reset"
    var h = FlowGraph<V>()
    for i in 0..m:
        if both[i]: h.add_both(V(eu[i]), V(ev[i]), ec[i])
        else: h.add(V(eu[i]), V(ev[i]), Some(ec[i]))
    if h.push_relabel(V(0), V(4)) != mc2: bad = "op {k}: push_relabel"
    // CostFlow 0 -> 3 against every flow vector
    let cu = [rng.int(0, 3) for _ in 0..5]
    let cv = [rng.int(0, 3) for _ in 0..5]
    var cc = [rng.int(0, 2) for _ in 0..5]
    let cw = [rng.int(0, 4) for _ in 0..5]
    var cf = CostFlow<W>()
    let carcs = [cf.add(W(cu[i]), W(cv[i]), Some(cc[i]), cw[i]) for i in 0..5]
    let sp = rng.int(0, 4)
    cc[sp] = rng.int(0, 2)
    cf.set_cap(carcs[sp], Some(cc[sp]))
    var best = List.filled(12, big)
    for f in vectors(List.filled(5, 0), cc):
        let nt2 = net(cu, cv, f, 4)
        if nt2[1] == 0 and nt2[2] == 0 and nt2[0] >= 0:
            let c = (0..5).sum(i => f[i] * cw[i])
            if c < best[nt2[0]]: best[nt2[0]] = c
    var top = 0
    while top + 1 < 12 and best[top + 1] < big: top += 1
    let climit: ?Int = if rng.int(0, 2) == 0: Some(rng.int(0, 4)) else: None
    let fv = if let l = climit: min(l, top) else: top
    var slope_g = CostFlow<W>()
    for i in 0..5: slope_g.add(W(cu[i]), W(cv[i]), Some(cc[i]), cw[i])
    let got = cf.min_cost_flow(W(0), W(3), limit: climit)
    if got != (flow: fv, cost: best[fv]): bad = "op {k}: min_cost_flow {got.flow} {got.cost}"
    var cfl = [cf.flow(a) for a in carcs]
    if (0..5).sum(i => cfl[i] * cf.cost(carcs[i])) != got.cost or net(cu, cv, cfl, 4)[1] != 0: bad = "op {k}: cost flow arcs"
    for i in 0..5:
        let a = carcs[i]
        if cf.cap(a) != cc[i] or cf.cost(a.rev) != -cw[i] or cf.source(a) != W(cu[i]) or cf.target(a) != W(cv[i]): bad = "op {k}: cost arc {i}"
    if cf.arcs.collect() != carcs: bad = "op {k}: cost arcs"
    let pts = slope_g.slope(W(0), W(3))
    if pts[0] != (flow: 0, cost: 0) or pts[pts.len - 1].flow != top: bad = "op {k}: slope ends"
    for j in 0..pts.len - 1:
        let (p, q) = (pts[j], pts[j + 1])
        for f in p.flow..q.flow + 1:
            if (q.cost - p.cost) * (f - p.flow) != (best[f] - p.cost) * (q.flow - p.flow): bad = "op {k}: slope at {f}"
    // cancel_cycles from the zero flow: the cheapest circulation
    let zu = [rng.int(0, 2) for _ in 0..4]
    let zv = [rng.int(0, 2) for _ in 0..4]
    let zc = [rng.int(0, 2) for _ in 0..4]
    let zw = [rng.int(-3, 3) for _ in 0..4]
    var zg = CostFlow<U>()
    let zarcs = [zg.add(U(zu[i]), U(zv[i]), Some(zc[i]), zw[i]) for i in 0..4]
    var zbest = 0
    for f in vectors(List.filled(4, 0), zc):
        if net(zu, zv, f, 3).all(x => x == 0): zbest = min(zbest, (0..4).sum(i => f[i] * zw[i]))
    if zg.cancel_cycles() != zbest or (0..4).sum(i => zg.flow(zarcs[i]) * zw[i]) != zbest: bad = "op {k}: cancel_cycles"
    // circulations and bounded flows 0 -> 2
    let bu = [rng.int(0, 2) for _ in 0..4]
    let bv = [rng.int(0, 2) for _ in 0..4]
    let blo = [rng.int(0, 1) for _ in 0..4]
    let bhi = [blo[i] + rng.int(0, 2) for i in 0..4]
    let bw = [rng.int(-3, 3) for _ in 0..4]
    let bes: [B](u: U, v: U, lo: Int, hi: Int) = [B] e => (u: U(bu[e.ord]), v: U(bv[e.ord]), lo: blo[e.ord], hi: bhi[e.ord])
    let wes: [B](u: U, v: U, lo: Int, hi: Int, cost: Int) = [B] e => (u: U(bu[e.ord]), v: U(bv[e.ord]), lo: blo[e.ord], hi: bhi[e.ord], cost: bw[e.ord])
    var circ = false
    var ccost = big
    var vmax: ?Int = None
    var vmin: ?Int = None
    for f in vectors(blo, bhi):
        let nt3 = net(bu, bv, f, 3)
        if nt3.all(x => x == 0):
            circ = true
            ccost = min(ccost, (0..4).sum(i => f[i] * bw[i]))
        if nt3[1] == 0:
            if nt3[0] >= 0 and (vmax == None or nt3[0] > vmax!): vmax = Some(nt3[0])
            if nt3[0] >= 0 and (vmin == None or nt3[0] < vmin!): vmin = Some(nt3[0])
    fn fits(f: List<Int>, lo: List<Int>, hi: List<Int>) -> Bool = (0..lo.len).all(i => lo[i] <= f[i] and f[i] <= hi[i])
    if let f = circulation(bes):
        let fs = [f[e] for e in B]
        if not circ or not fits(fs, blo, bhi) or not net(bu, bv, fs, 3).all(x => x == 0): bad = "op {k}: circulation"
    elif circ: bad = "op {k}: circulation missed"
    if let r = min_cost_circulation(wes):
        let fs = [r.flow[e] for e in B]
        if r.cost != ccost or (0..4).sum(i => fs[i] * bw[i]) != ccost or not fits(fs, blo, bhi) or not net(bu, bv, fs, 3).all(x => x == 0): bad = "op {k}: min_cost_circulation"
    elif circ: bad = "op {k}: min_cost_circulation missed"
    for (r, want, name) in [(max_bounded_flow(bes, U(0), U(2)), vmax, 0), (min_bounded_flow(bes, U(0), U(2)), vmin, 1)]:
        if let x = r:
            let fs = [x.flow[e] for e in B]
            let nt4 = net(bu, bv, fs, 3)
            if want != Some(x.value) or not fits(fs, blo, bhi) or nt4[1] != 0 or nt4[0] != x.value: bad = "op {k}: bounded flow {name}"
        elif want != None: bad = "op {k}: bounded flow {name} missed"
    // max closure over every vertex subset
    let before: [V]List<V> = [V] _ => [V(rng.int(0, 4)) for _ in 0..rng.int(0, 2)]
    let profit = [rng.int(-4, 4) for _ in 0..5]
    var cbest = 0
    fn closed(before: [V]List<V>, mask: Int) -> Bool = V.all(u => before[u].all(v => (mask >> v.ord) & 1 == 0 or (mask >> u.ord) & 1 == 1))
    for mask in 0..32:
        if closed(before, mask): cbest = max(cbest, (0..5).sum(v => if (mask >> v) & 1 == 1: profit[v] else: 0))
    let cl = max_closure(before, profit: v => profit[v.ord])
    let cmask = (0..5).sum(v => if cl.take[V(v)]: 1 << v else: 0)
    if cl.value != cbest or not closed(before, cmask) or (0..5).sum(v => if cl.take[V(v)]: profit[v] else: 0) != cbest: bad = "op {k}: max_closure"
    // Gomory-Hu tree and global min cut of an undirected graph
    let gu = [rng.int(0, 4) for _ in 0..5]
    let gv = [rng.int(0, 4) for _ in 0..5]
    let gw = [rng.int(0, 3) for _ in 0..5]
    let ges: [G](u: V, v: V, w: Int) = [G] e => (u: V(gu[e.ord]), v: V(gv[e.ord]), w: gw[e.ord])
    let (uu, uv, uc) = (gu + gv, gv + gu, gw + gw)
    let tree = gomory_hu(ges)
    if tree.len != 4: bad = "op {k}: gomory_hu size"
    else:
        for s in 0..5:
            // least weight on the tree path from s to every vertex
            var low = List.filled(5, -1)
            low[s] = big
            for _ in 0..5:
                for e in tree:
                    for (a, b) in [(e.u.ord, e.v.ord), (e.v.ord, e.u.ord)]:
                        if low[a] >= 0 and low[b] < 0: low[b] = min(low[a], e.w)
            for t in 0..5:
                if t != s and low[t] != mincut(uu, uv, uc, 5, s, t): bad = "op {k}: gomory_hu {s} {t}"
    var gbest = big
    for mask in 1..31:
        var c = 0
        for i in 0..5:
            if ((mask >> gu[i]) & 1) != ((mask >> gv[i]) & 1): c += gw[i]
        gbest = min(gbest, c)
    let gm = global_min_cut(ges)
    var gc = 0
    for i in 0..5:
        if gm.side[V(gu[i])] != gm.side[V(gv[i])]: gc += gw[i]
    if gm.w != gbest or gc != gbest or gm.side[V(0)] or V.all(v => not gm.side[v]): bad = "op {k}: global_min_cut"
    if bad != "": break
print(if bad == "": "ok" else: bad)

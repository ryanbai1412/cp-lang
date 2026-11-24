// let: V = \sqrt n; L = \sqrt n; R = \sqrt n; F = \sqrt n; P = \sqrt n
// fixed: c
use std.flow.{CostFlow, Matching, best_matching, stable_matching, edge_coloring, min_cost_circulation}
// Graphs on V = sqrt n vertices and E = n arcs. The flow families need V augmenting phases: a
// shortcut 0 -> i of capacity 1 into a chain i -> t gives one path of each length, and the rest of
// the n arcs can carry nothing but are scanned by every phase.
let n: Int = read()
let k = n.isqrt
domain V = 0..k
domain L = 0..k
domain R = 0..k
domain E = 0..k * k
domain C = 0..k
let pad = k * k - 2 * k
var s = 0
fn chain() -> FlowGraph<V>:
    var g = FlowGraph<V>()
    for i in 1..k:
        g.add(V(0), V(i), Some(1))
        if i + 1 < k: g.add(V(i), V(i + 1), None)
    for j in 0..pad: g.add(V((j * 7919) % k), V((j * 104729 + 1) % k), Some(0))
    g
fn cost_chain() -> CostFlow<V>:
    var g = CostFlow<V>()
    for i in 1..k:
        g.add(V(0), V(i), Some(1), 0)
        if i + 1 < k: g.add(V(i), V(i + 1), None, 1)
    for j in 0..pad: g.add(V((j * 7919) % k), V((j * 104729 + 1) % k), Some(0), 1)
    g
// A chain k-1 -> ... -> 0 of cost -1 closed by 0 -> k-1: one negative cycle that Bellman-Ford
// needs V rounds to see, as it scans vertices upward.
fn negative_cycle() -> CostFlow<V>:
    var g = CostFlow<V>()
    for i in 0..k - 1: g.add(V(i + 1), V(i), Some(1), -1)
    g.add(V(0), V(k - 1), Some(1), 0)
    for j in 0..pad: g.add(V((j * 7919) % k), V((j * 104729 + 1) % k), Some(0), 1)
    g
let (src, dst) = (V(0), V(k - 1))
var g = chain()
s += g.max_flow(src, dst) // timed: flow.FlowGraph.max_flow
var cf = cost_chain()
s += cf.min_cost_flow(src, dst).cost // timed: flow.CostFlow.min_cost_flow
var cs = cost_chain()
s += cs.slope(src, dst).len // timed: flow.CostFlow.slope
var neg = negative_cycle()
s += neg.cancel_cycles() // timed: flow.CostFlow.cancel_cycles
for _ in 0..16:
    g.reset()
    s += g.push_relabel(src, dst) // timed: flow.FlowGraph.push_relabel
    for p in g.paths(src, dst): s += p.amount // timed: flow.FlowGraph.paths
// The same chain as a closure: v_i before v_{i-1}, profit 1 on each v_i but v_0, which costs k;
// the padding prerequisites point upward, so no path gets shorter.
let before = [V] v => do:
    var b = if v.ord + 1 < k: [V(v.ord + 1)] else: []
    if v.ord > 0: b += [V(j % v.ord) for j in 0..k - 2]
    b
let profit = [V] v => if v.ord == 0: -k else: 1
s += max_closure(before, profit: v => profit[v]).value // timed: flow.max_closure
let bounds: [E](u: V, v: V, lo: Int, hi: Int) = [E] e => do:
    let i = e.ord
    if i < k - 1: (u: V(0), v: V(i + 1), lo: 1, hi: 1)
    elif i < 2 * k - 2: (u: V(i - k + 2), v: V(i - k + 1), lo: 0, hi: k)
    else: (u: V((i * 7919) % k), v: V((i * 104729 + 1) % k), lo: 0, hi: 0)
let priced: [E](u: V, v: V, lo: Int, hi: Int, cost: Int) = [E] e => (u: bounds[e].u, v: bounds[e].v, lo: bounds[e].lo, hi: bounds[e].hi, cost: 0)
s += (circulation(bounds) ?? []).len // timed: flow.circulation
s += (max_bounded_flow(bounds, V(1), V(0)))!.value // timed: flow.max_bounded_flow
s += (min_bounded_flow(bounds, V(1), V(0)))!.value // timed: flow.min_bounded_flow
s += (min_cost_circulation(priced))!.cost // timed: flow.min_cost_circulation
let weighted: [E](u: V, v: V, w: Int) = [E] e => (u: V(e.ord / k), v: V(e.ord % k), w: (e.ord * 7919) % 97 + 1)
s += gomory_hu(weighted).len // timed: flow.gomory_hu
s += global_min_cut(weighted).w // timed: flow.global_min_cut
for _ in 0..3:
    s += max_weight_general_matching(weighted).total // timed: flow.max_weight_general_matching
// Greedy matches l_i to r_i and strands l_{k-1}, whose one augmenting path walks the whole
// chain; the padding edges l_i - r_j, j < i, only lead back along it.
let badj = [L] l => do:
    let i = l.ord
    var b = if i + 1 < k: [R(i), R(i + 1)] else: [R(0)]
    b += [R(j) for j in 0..i if i + 1 < k or j > 0]
    b
let adj = [V] v => [V((v.ord * 31 + j * 7) % k) for j in 0..k]
for _ in 0..48:
    s += general_matching(adj).size // timed: flow.general_matching
for _ in 0..112:
    s += max_matching(badj).size // timed: flow.max_matching
    s += min_vertex_cover(badj).left.count(x => x) // timed: flow.min_vertex_cover
for _ in 0..64:
    var m = Matching<L, R>()
    for l in L:
        for r in badj[l]: m.add(l, r)
    s += m.augment_all() // timed: flow.Matching.augment_all
for _ in 0..16:
    s += max_weight_matching((l: L, r: R) => (l.ord * 7919 + r.ord * 31) % 1009).total // timed: flow.max_weight_matching
s += min_weight_matching((l: L, r: R) => (l.ord * 7919 + r.ord * 31) % 1009).total // timed: flow.min_weight_matching
let bip: [E](l: L, r: R, w: Int) = [E] e => (l: L(e.ord / k), r: R(e.ord % k), w: (e.ord * 7919) % 1009)
// Everyone ranks the other side alike, so l_i is turned down by r_0..r_{i-1}: L R proposals.
let pl = [L] _ => [R(j) for j in 0..k]
let pr = [R] _ => [L(j) for j in 0..k]
let ends: [E](l: L, r: R) = [E] e => (l: bip[e].l, r: bip[e].r)
s += (best_matching(bip))!.total // timed: flow.best_matching
s += edge_coloring(ends).colors // timed: flow.edge_coloring
for _ in 0..32:
    s += stable_matching(pl, pr).len // timed: flow.stable_matching
print(s)

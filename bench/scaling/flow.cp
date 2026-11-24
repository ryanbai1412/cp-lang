// n: 32768
use std.flow.{FlowGraph, CostFlow, Arc, Matching, GraphicMatroid, ColorMatroid}
let n: Int = read()
domain V = 0..n
domain E = 0..n
domain C = 0..4
domain W = 0..n / 16
var s = 0
for _ in 0..1024:
    s += FlowGraph<W>().arcs.count() // timed: flow.FlowGraph.new
    s += CostFlow<W>().arcs.count() // timed: flow.CostFlow.new
for _ in 0..32768:
    s += (Matching<W, W>().back(W(n / 16 - 1)) ?? W(0)).ord // timed: flow.Matching.new
let es = [E] e => (u: V(e.ord), v: V((e.ord * 7 + 3) % n))
let col = [E] e => C(e.ord % 4)
let lim = [C] _ => n
for _ in 0..64:
    let gm = GraphicMatroid(es) // timed: flow.GraphicMatroid.new
    if gm.can_add(E(0)): s += 1
for _ in 0..1024:
    let cm = ColorMatroid(col, lim) // timed: flow.ColorMatroid.new
    if cm.can_add(E(0)): s += 1
var g = FlowGraph<V>()
var cf = CostFlow<V>()
var arcs = List<Arc>()
var carcs = List<Arc>()
for _ in 0..4:
    g = FlowGraph<V>()
    cf = CostFlow<V>()
    arcs = []
    carcs = []
    for i in 0..n:
        arcs.push(g.add(V(i), V((i + 1) % n), Some(i % 7 + 1))) // timed: flow.FlowGraph.add
        arcs.push(g.add_both(V(i), V((i * 7 + 3) % n), i % 5 + 1)) // timed: flow.FlowGraph.add_both
        carcs.push(cf.add(V(i), V((i + 1) % n), Some(i % 7 + 1), i % 3)) // timed: flow.CostFlow.add
for i in 1..n: g.add(V(0), V(i), Some(1))
for i in 0..8 * n:
    let a = arcs[i % arcs.len]
    let b = carcs[i % carcs.len]
    s += g.flow(a) // timed: flow.FlowGraph.flow
    s += g.cap(a) // timed: flow.FlowGraph.cap
    s += g.source(a).ord // timed: flow.FlowGraph.source
    s += g.target(a).ord // timed: flow.FlowGraph.target
    s += g.residual(a.rev) // timed: flow.FlowGraph.residual
    s += a.rev.rev.id // timed: flow.Arc.rev
    g.set_cap(a, Some(i % 7 + 1)) // timed: flow.FlowGraph.set_cap
    s += cf.flow(b) // timed: flow.CostFlow.flow
    s += cf.cap(b) // timed: flow.CostFlow.cap
    s += cf.cost(b) // timed: flow.CostFlow.cost
    s += cf.source(b).ord // timed: flow.CostFlow.source
    s += cf.target(b).ord // timed: flow.CostFlow.target
    cf.set_cap(b, Some(i % 7 + 1)) // timed: flow.CostFlow.set_cap
for _ in 0..16:
    for a in g.arcs: s += a.id // timed: flow.FlowGraph.arcs
    for a in g.out(V(0)): s += a.id // timed: flow.FlowGraph.out
    for a in cf.arcs: s += a.id // timed: flow.CostFlow.arcs
s += g.max_flow(V(0), V(n - 1))
for _ in 0..16:
    s += g.min_cut(V(0)).count(x => x) // timed: flow.FlowGraph.min_cut
    s += g.cut_arcs(V(0)).len // timed: flow.FlowGraph.cut_arcs
for _ in 0..64:
    g.reset() // timed: flow.FlowGraph.reset
// A chain l_i -> r_{i-1} matched, so augmenting l_0 walks all of it.
for _ in 0..16:
    var m = Matching<V, V>()
    for i in 1..n:
        m.add(V(i), V(i - 1))
        m.add(V(i), V(i))
    m.augment_all()
    m.add(V(0), V(0))
    if m.augment(V(0)): s += 1 // timed: flow.Matching.augment
    s += m.remove(V(n - 1)) // timed: flow.Matching.remove
    s += m.cover().left.count(x => x) // timed: flow.Matching.cover
var m = Matching<V, V>()
for _ in 0..4:
    m = Matching<V, V>()
    for i in 0..n:
        m.add(V(i), V((i * 7 + 1) % n)) // timed: flow.Matching.add
        m.add(V(i), V((i * 3 + 2) % n))
m.augment_all()
for i in 0..8 * n:
    s += (m.mate(V(i % n)) ?? V(0)).ord // timed: flow.Matching.mate
    s += (m.back(V(i % n)) ?? V(0)).ord // timed: flow.Matching.back
    s += m.len // timed: flow.Matching.len
    s %= 1000000007
print(s)

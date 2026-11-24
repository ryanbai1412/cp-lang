// let: V = \sqrt n; L = \sqrt n; R = \sqrt n; F = \sqrt n; P = \sqrt n
// fixed: c
use std.flow.{CostFlow}
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
print(s)

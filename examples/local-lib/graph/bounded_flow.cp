//! Flows where every arc has a lower bound as well as a capacity: a feasible
//! circulation, and the largest or smallest `s`-`t` flow that meets every bound.
//!
//! Built on `std.flow.FlowGraph` (Dinic): the lower bounds are paid up front and the
//! debt they leave at each vertex is settled by one ordinary maximum flow.
use std.flow.{Arc, FlowGraph}

/// A flow that is conserved at every vertex with `lo <= flow <= hi` on every arc, as
/// the flow of each arc in the order given, or `none` if the bounds cannot all be met.
pub fn circulation<V>(arcs: List<(u: V, v: V, lo: Int, hi: Int)>) -> ?List<Int>:
    let found = solve::<V>(arcs, none, false)
    if found == none: none else: found!.flow

/// The largest flow from `s` to `t` with `lo <= flow <= hi` on every arc, as its value
/// and the flow of each arc, or `none` if no flow meets the bounds. The value is the net
/// amount leaving `s`, and is never negative.
pub fn max_bounded_flow<V>(arcs: List<(u: V, v: V, lo: Int, hi: Int)>, s: V, t: V) -> ?(value: Int, flow: List<Int>):
    solve::<V>(arcs, (s, t), false)

/// The smallest flow from `s` to `t` with `lo <= flow <= hi` on every arc: the fewest
/// paths that cover every required arc, for one. `none` if no flow meets the bounds.
pub fn min_bounded_flow<V>(arcs: List<(u: V, v: V, lo: Int, hi: Int)>, s: V, t: V) -> ?(value: Int, flow: List<Int>):
    solve::<V>(arcs, (s, t), true)

fn solve<V>(arcs: List<(u: V, v: V, lo: Int, hi: Int)>, ends: ?(V, V), smallest: Bool) -> ?(value: Int, flow: List<Int>):
    // two extra vertices: `pay` hands out what the lower bounds owe, `take` collects it
    domain Node = 0..V.K + 2
    let (pay, take) = (Node.nth(V.K), Node.nth(V.K + 1))
    var g = FlowGraph<Node>()
    var owed: [V]Int = 0
    var slack = List<Arc>()
    for a in arcs:
        assert a.lo <= a.hi
        slack.push(g.add(Node.nth(a.u.to_idx), Node.nth(a.v.to_idx), a.hi - a.lo))
        owed[a.v] += a.lo
        owed[a.u] -= a.lo
    var debt = 0
    for v in V:
        if owed[v] > 0:
            g.add(pay, Node.nth(v.to_idx), owed[v])
            debt += owed[v]
        elif owed[v] < 0:
            g.add(Node.nth(v.to_idx), take, -owed[v])
    var value = 0
    if ends == none:
        if g.max_flow(pay, take) != debt: return none
    else:
        let (s, t) = (Node.nth(ends!.0.to_idx), Node.nth(ends!.1.to_idx))
        let room = 1 + sum(abs(a.lo) + abs(a.hi) for a in arcs)
        // settle what can be settled without `t -> s` first, so that the return arc
        // carries as little as possible
        var paid = if smallest: g.max_flow(pay, take) else: 0
        let home = g.add(t, s, room)
        paid += g.max_flow(pay, take)
        if paid != debt: return none
        // pushing on from `s` to `t` also sends the return arc's flow back, so the
        // amount pushed is the whole value
        value = if smallest: g.flow(home) else: g.max_flow(s, t)
    (value: value, flow: [arcs[i].lo + g.flow(slack[i]) for i in 0..arcs.len])

/// Every way to give each arc a flow within its bounds, as (net flow out of `s`, flows),
/// keeping those conserved at every vertex except `s` and `t`.
fn all_flows<V>(arcs: List<(u: V, v: V, lo: Int, hi: Int)>, s: ?V, t: ?V) -> List<(Int, List<Int>)>:
    var found: List<(Int, List<Int>)> = []
    var f = [a.lo for a in arcs]
    while true:
        var net: [V]Int = 0
        for i in 0..arcs.len:
            net[arcs[i].u] += f[i]
            net[arcs[i].v] -= f[i]
        if all(net[v] == 0 for v in V if v != s and v != t):
            found.push((if s == none: 0 else: net[s!], [x for x in f]))
        // count up, arc 0 fastest
        var i = 0
        while i < arcs.len and f[i] == arcs[i].hi:
            f[i] = arcs[i].lo
            i += 1
        if i == arcs.len: break
        f[i] += 1
    found

@test
fn circulation_exists_exactly_when_brute_force_finds_one():
    domain V = 0..4
    var (yes, no) = (0, 0)
    for _ in 0..60:
        let arcs = [(u: V.nth(rand(0, 3)), v: V.nth(rand(0, 3)), lo: lo, hi: lo + rand(0, 2)) for lo in [rand(0, 1) for _ in 0..rand(1, 6)]]
        let got = circulation::<V>(arcs)
        let every = all_flows::<V>(arcs, none, none)
        assert (got == none) == every.is_empty
        if got != none:
            assert every.find((0, got!)) != none
            yes += 1
        else:
            no += 1
    assert yes > 0 and no > 0

@test
fn largest_and_smallest_flow_match_brute_force():
    domain V = 0..4
    var feasible = 0
    for _ in 0..60:
        let arcs = [(u: V.nth(rand(0, 3)), v: V.nth(rand(0, 3)), lo: lo, hi: lo + rand(0, 2)) for lo in [rand(0, 5) / 5 for _ in 0..rand(1, 6)]]
        let (s, t) = (V.first, V.last)
        // a flow from `s` to `t` never has more coming back into `s` than leaving it
        let every = [e for e in all_flows::<V>(arcs, s, t) if e.0 >= 0]
        let (big, small) = (max_bounded_flow::<V>(arcs, s, t), min_bounded_flow::<V>(arcs, s, t))
        assert (big == none) == every.is_empty and (small == none) == every.is_empty
        if big == none: continue
        feasible += 1
        assert big!.value == max(e.0 for e in every) and small!.value == min(e.0 for e in every)
        assert every.find((big!.value, big!.flow)) != none
        assert every.find((small!.value, small!.flow)) != none
    assert feasible > 10

@test
fn fewest_paths_through_every_required_arc():
    // a DAG where three arcs must be walked at least once; walkers start at 0, stop at 5
    domain V = 0..6
    let must = [(1, 3), (2, 3), (3, 4)]
    let free = [(0, 1), (0, 2), (4, 5), (1, 4)]
    let arcs = [(u: V(a), v: V(b), lo: 1, hi: 9) for (a, b) in must] + [(u: V(a), v: V(b), lo: 0, hi: 9) for (a, b) in free]
    let plan = min_bounded_flow::<V>(arcs, V(0), V(5))!
    assert plan.value == 2 and plan.flow == [1, 1, 2, 1, 1, 2, 0]
    assert max_bounded_flow::<V>(arcs, V(0), V(5))!.value == 9
    // if every arc holds one walker, 3 -> 4 cannot carry the two that must reach it
    assert min_bounded_flow::<V>([(u: a.u, v: a.v, lo: a.lo, hi: 1) for a in arcs], V(0), V(5)) == none

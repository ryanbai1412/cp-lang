// CCO '24 P4 — Infiltration / Tree Rendezvous (DMOJ: cco24p4)
// original: corpus/submissions/6435484.cpp
// CP Lang v2 (locked spec, docs/one-page-cheat-sheet.md @ 3723dca)
//
// Given a tree on n = 100 vertices (0-indexed), output a schedule of length T
// specifying positions for two agents starting at each vertex so that for every
// distinct pair (u, v), agent 0 from u and agent 1 from v meet at some time
// t <= 13 * dist(u, v).
let n: 1..=100 = read()
if n != 100: return

domain V = 0..n
let edges: [1..n](V, V) = read()
let g = Graph.undirected(edges)

// Pick a tree center (eccentricity <= 50 for n = 100).
let rt = first(r for r in V if max(Rooted<V>(g, r).depth(v) for v in V)! <= 50)!
let tree = Rooted<V>(g, rt)

// Convert target height-gap checkpoints into alternating step bursts, then into a
// per-step move schedule for the two alternating agent parities.
fn build_schedule(gaps: List<Int>) -> List<Bool>:
    var strat: List<(Bool, Int)> = []
    for nx in gaps if nx != 0:
        strat.push((nx < 0, abs(nx)))
    var sched: List<Bool> = []
    for (ty, cnt) in strat:
        for _ in 0..cnt:
            if not sched.is_empty and ty and not sched.last!:
                sched[sched.len - 1] = true
            else:
                sched.push(not ty)
                sched.push(ty)
    sched

let sched = build_schedule([-3, 11, -22, 10, 25, -25, -30, 30])
domain T = 0..sched.len

var ans: [Bool, V, T]V = rt
for ty in [false, true]:
    for start in V:
        var u = start
        for t in T:
            if (t.to_idx % 2 == 1) == ty and sched[t]:
                u = tree.parent(u) ?? u
            ans[ty, start, t] = u

print(T.K)
for ty in [false, true]:
    for start in V:
        print(" ".join("{ans[ty, start, t]}" for t in T))

// Self-check every pair meets within 13 * dist(u, v) steps (local debug).
for u in V:
    for v in V if u != v:
        let meet = first(t.to_idx for t in T if ans[false, u, t] == ans[true, v, t])!
        assume meet <= 13 * tree.dist(u, v)

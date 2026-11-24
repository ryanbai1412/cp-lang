// Source: APIO 2023, Cyberland, https://dmoj.ca/problem/apio23p1
// limits: 2500 ms, 1024 MB
// Problem: n countries, m two-way roads with times c <= 1e9; travel from country 0 to h (the
//   trip ends on arrival). Arriving at a kind-0 country may reset the clock to 0, at a kind-2
//   country may halve it (at most k <= 1e6 halvings in all). Least time at h, or -1 if h is
//   unreachable; relative error 1e-6. Sum n, sum m <= 1e5 over the grader calls.
// Solution: a reset at a reachable kind-0 country is as good as starting there, so all of
//   them (and country 0) are Dijkstra sources, with no road leaving h. Then add halvings one
//   layer at a time: re-run Dijkstra from every country's best time plus, for kind-2 countries,
//   half that time plus each road. Times stay below 1e14 and 1e14 / 2^70 is under the
//   tolerance, so min(k, 70) layers suffice, and a layer whose halvings improve nothing is the
//   fixpoint. O(min(k, 70) (n + m) log n).
//
// The original is a grader function; this port reads the sample grader's input: T calls,
// each `n m k h`, the n kinds, then m lines `u v time`.
// original: corpus/submissions/5593318.cpp
// Bounds: n: 2..=1e5; m: 0..=1e5; k: 1..=1e6; goal: 1..=n - 1; kind: 0..=2; roads.u, roads.v: 0..=n - 1; roads.w: 1..=1e9

@!multitest
let (n, m, k): (2..=1e5, 0..=1e5, 1..=1e6) = read()
domain C = 0..n
let goal: 1..=n - 1 = read()
let h = C(goal)
let kind: [C]0..=2 = read()
let roads: [m](u: C, v: C, w: 1..=1e9) = read()
var out: [C]List<(to: C, w: F64)> = []
for (u, v, w) in roads:
    if u != h: out[u].push((to: v, w: F64(w)))  // nobody leaves h
    if v != h: out[v].push((to: u, w: F64(w)))

fn times(starts: List<(C, F64)>) -> [C]?F64 = dijkstra(out, from: starts).dist

let home = C.first!
let reach = times([(home, 0.0)])
if reach[h] == None:
    print(-1)
else:
    var best = times([(c, 0.0) for c in C if reach[c] != None and (c == home or kind[c] == 0)])
    for _ in 0..min(k, 70):
        var halved: List<(C, F64)> = []
        for c in C if kind[c] == 2:
            let t = best[c] else: continue
            for r in out[c] if t / 2 + r.w < best[r.to]!:
                halved.push((r.to, t / 2 + r.w))
        if halved.is_empty:
            break
        best = times([(c, t) for c in C if let t = best[c]] + halved)
    print(best[h]!, precision: 6)

// APIO 2023 Cyberland: n countries joined by m two-way roads with travel
// times. You start in country 0 with the clock at 0 and want to reach
// country h, where the trip ends at once. Country kinds: 0 sets your clock
// back to zero, 2 lets you halve it (at most k times in the whole trip),
// 1 does nothing. Print the least time on the clock at h, or -1.
//
// A clock at zero in a country you can reach is as good as starting there,
// so every reachable kind-0 country is a start. After that, layer by layer:
// with one more halving allowed, each kind-2 country offers its neighbours
// half of its best time plus the road. Thirty halvings shrink any time
// below the answer's tolerance, so more than 80 layers change nothing.
// A country that cannot be reached has no time at all (`none`), not a
// large number.
//
// The original is a grader function; this port reads
// `n m k h`, the n kinds, then m lines `u v time`.
// original: corpus/submissions/5593318.cpp
use std.graph.{dijkstra}
use real.float.{fixed}

let (n, m, k, h): (Int, Int, Int, Int) = read()
domain C = 0..n
var kind: [C]Int = 0
for c in C:
    let x: Int = read()
    kind[c] = x
var roads: [C]List<(to: C, time: Float)> = []
for _ in 0..m:
    let (u, v, time): (Int, Int, Float) = read()
    if u != h: roads[C(u)].push((to: C(v), time: time))        // nobody leaves h
    if v != h: roads[C(v)].push((to: C(u), time: time))

fn times(starts: List<(C, Float)>) -> [C]?Float = dijkstra(c => roads[c], starts, r => r.time)

let reach = times([(C(0), 0.0)])
if reach[C(h)] == none:
    print(-1)
else:
    var best = times([(c, 0.0) for c in C if reach[c] != none and (c == C(0) or kind[c] == 0)])
    for _ in 0..min(k, 80):
        var starts: List<(C, Float)> = []
        for c in C:
            let t = best[c] else: continue
            starts.push((c, t))
            if kind[c] == 2:
                for r in roads[c]: starts.push((r.to, t / 2 + r.time))
        best = times(starts)
    print(fixed(best[C(h)]!, 6))

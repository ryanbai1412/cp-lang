// Source: APIO 2023, Cyberland, https://dmoj.ca/problem/apio23p1
// Problem: n countries, m two-way roads with times c <= 1e9; travel from country 0 to h (the
//   trip ends on arrival). Arriving at a kind-0 country may reset the clock to 0, at a kind-2
//   country may halve it (at most k <= 1e6 halvings in all). Least time at h, or -1 if h is
//   unreachable; relative error 1e-6. Sum n, sum m <= 1e5 over the grader calls.
// Solution: a reset at a reachable kind-0 country is as good as starting there, so all of
//   them (and country 0) are Dijkstra sources, with no road leaving h. Then add halvings one
//   layer at a time: re-run Dijkstra from every country's best time plus, for kind-2 countries,
//   half that time plus each road. After ~70 halvings any time is below the tolerance, so
//   min(k, 80) layers suffice: O(min(k, 80) (n + m) log n).
//
// The original is a grader function; this port reads
// `n m k h`, the n kinds, then m lines `u v time`.
// original: corpus/submissions/5593318.cpp

let (n, m, k, h): (Int, Int, Int, Int) = read()
domain C = 0..n
var kind: [C]Int = 0
for c in C:
    let x: Int = read()
    kind[c] = x
var roads: [C]List<(to: C, w: F64)> = []
for _ in 0..m:
    let (u, v, time): (Int, Int, F64) = read()
    if u != h: roads[C(u)].push((to: C(v), w: time))  // nobody leaves h
    if v != h: roads[C(v)].push((to: C(u), w: time))

fn times(starts: List<(C, F64)>) -> [C]?F64:
    let paths = dijkstra(roads, from: starts)
    [C] c => paths[c]

let reach = times([(C(0), 0.0)])
if reach[C(h)] == None:
    print(-1)
else:
    var best = times([(c, 0.0) for c in C if reach[c] != None and (c == C(0) or kind[c] == 0)])
    for _ in 0..min(k, 80):
        var starts: List<(C, F64)> = []
        for c in C:
            let t = best[c] else: continue
            starts.push((c, t))
            if kind[c] == 2:
                for r in roads[c]: starts.push((r.to, t / 2 + r.w))
        best = times(starts)
    print(best[C(h)]!, precision: 6)

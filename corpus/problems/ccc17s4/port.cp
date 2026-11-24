// Source: CCC 2017 S4 (Minimum Cost Flow), https://dmoj.ca/problem/ccc17s4
// limits: 1400 ms, 256 MB
// Problem: N <= 1e5 buildings, M <= 2e5 weighted pipes; the first N - 1 form the current
//   spanning tree. Each day swap one active pipe for an inactive one. One pipe may be enhanced
//   from cost C to max(0, C - D). Print the fewest days to reach a spanning tree of minimum
//   cost over all trees and all choices of the enhanced pipe.
// Solution: Kruskal with old pipes ahead of new ones of equal cost keeps as many old pipes as a
//   cheapest tree can (`kruskal` breaks ties by edge order, and old pipes come first); the days
//   are its new pipes. The enhancer goes on the dearest tree pipe. If that cost is <= D, an
//   unused old pipe of cost <= D that reconnects the tree without its dearest new pipes can
//   replace one of them at no cost: one day less. O(M log M).
// original: corpus/submissions/4899336.cpp
let n: 1..=1e5 = read()
let m: 1..=2e5 = read()
let d: 0..=1e9 = read()
domain House = 1..=n
domain Pipe = 0..m
let pipes: [Pipe](u: House, v: House, w: 1..=1e9) = read()
let is_old = (p: Pipe) => p.ord < n - 1

let (_, take) = kruskal(pipes)
let days = Pipe.count(p => take[p] and not is_old(p))
// the tree without its new dearest pipes; an old pipe the enhancer makes free that still
// joins two of its parts replaces one of them
fn saves_a_day(dearest: Int) -> Bool:
    if dearest > d: return false
    var rest = Dsu<House>()
    for p in Pipe if take[p] and (is_old(p) or pipes[p].w < dearest): rest.unite(pipes[p].u, pipes[p].v)
    Pipe.any(p => not take[p] and is_old(p) and pipes[p].w <= d and not rest.same(pipes[p].u, pipes[p].v))
let dearest = max(pipes[p].w for p in Pipe if take[p])
print(days - Int(dearest.map(saves_a_day) ?? false))

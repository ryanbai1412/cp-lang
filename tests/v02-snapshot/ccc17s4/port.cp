// Source: CCC 2017 S4 (Minimum Cost Flow), https://dmoj.ca/problem/ccc17s4
// Problem: N <= 1e5 buildings, M <= 2e5 weighted pipes; the first N - 1 form the current
//   spanning tree. Each day swap one active pipe for an inactive one. One pipe may be enhanced
//   from cost C to max(0, C - D). Print the fewest days to reach a spanning tree of minimum
//   cost over all trees and all choices of the enhanced pipe.
// Solution: Kruskal with old pipes ahead of new ones of equal cost keeps as many old pipes as a
//   cheapest tree can; the days are its new pipes. The enhancer goes on the dearest tree pipe.
//   If that cost is <= D, an unused old pipe of cost <= D that reconnects the tree without its
//   dearest new pipes can replace one of them at no cost: one day less. Two DSU passes,
//   O(M log M).

let (n, m, d): (Int, Int, Int) = read()
domain House = 1..=n
domain Pipe = 0..m
let pipes: [Pipe](u: House, v: House, cost: Int) = read()
fn is_old(p: Pipe) -> Bool = p.ord < n - 1
let cheapest_first = sorted(Pipe, by: p => (pipes[p].cost, not is_old(p)))

var plan = Dsu<House>()
var chosen: [Pipe]Bool = false
var dearest = 0
for p in cheapest_first:
    if plan.unite(pipes[p].u, pipes[p].v):
        chosen[p] = true
        dearest = pipes[p].cost
var days = sum(1 for p in Pipe if chosen[p] and not is_old(p))

if dearest <= d:
    // the plan without its new dearest pipes; an old pipe that the enhancer
    // makes free and that still joins two parts replaces one of them
    var rest = Dsu<House>()
    for p in Pipe:
        if chosen[p] and (is_old(p) or pipes[p].cost < dearest): rest.unite(pipes[p].u, pipes[p].v)
    for p in Pipe:
        if not chosen[p] and is_old(p) and pipes[p].cost <= d and not rest.same(
            pipes[p].u,
            pipes[p].v,
        ):
            days -= 1
            break
print(days)

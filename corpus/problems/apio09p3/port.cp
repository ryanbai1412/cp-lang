// Source: APIO 2009 P3 (The Great ATM Robbery), https://dmoj.ca/problem/apio09p3
// Problem: a directed graph, N, M <= 5e5, with cash <= 4000 at each vertex, taken on the first
//   visit only. Walk from S to any of the P pubs (one is reachable); print the most cash taken.
// Solution: a strongly connected component can be robbed whole, so condense to a DAG with each
//   component's total cash; best[c] = loot[c] + the max best of its successors (0 at a pub
//   component), filled in reverse topological order. O(N + M).
// limits: 2000 ms, 64 MB
// Bounds: roads.u, roads.v, start, pubs: 1..=n

let (n, m): (Int, Int) = read()
domain Spot = 1..=n
domain Road = 0..m
let roads: [Road](u: Spot, v: Spot) = read()
let cash: [Spot]Int = read()
let (start, p): (Spot, Int) = read()
domain P = 0..p
let pubs: [P]Spot = read()

let g = Graph<Spot, Road>.directed(roads)
let (C, comp) = scc<Spot>(u => g.out(u))
let onward = condense<Spot>(u => g.out(u), comp)

var loot: [C]Int = 0
for v in Spot: loot[comp[v]] += cash[v]
var has_pub: [C]Bool = false
for v in pubs: has_pub[comp[v]] = true

// best[c]: the most a walk can take from c on, ending at a pub; none if it
// cannot end at one. Roads lead to later components, so fill from the back.
var best: [C]?Int = None
for c in C.rev:
    var after: ?Int = if has_pub[c]: 0 else: None
    for a in onward[c]:
        if let b = best[a.to]: after = max(after ?? b, b)
    if let x = after: best[c] = loot[c] + x
print(best[comp[start]]!)

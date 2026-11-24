// Tuned: Kruskal counting-sorts the roads by cost (at most 1e4) and tries only the roads
// left out of the tree (a kept road's path is itself, with no cheaper road to drop); the
// segment tree's combine picks the runner-up by cases, as the accepted C++ does, instead of
// building a list of the four candidates on every combine.
//
// Source: UTS Open 2015 P5 (Distribution Channel), https://dmoj.ca/problem/utso15p5
// limits: 3000 ms, 256 MB
// Problem: N <= 5e4 stores, M <= 1e5 possible connections (a, b, cost <= 1e4). Print the
//   cost of the second cheapest way to connect all stores with N - 1 connections (a set that
//   costs strictly more than the cheapest), or -1 if there is none.
// Solution: the runner-up is one swap away from a minimum spanning tree: add a road u-v,
//   drop the dearest tree road on the u-v path that is strictly cheaper. Build the MST by
//   Kruskal with a Dsu, root it, store each tree road at its lower town in a SegTree of the
//   two largest distinct costs, and fold every road's path over heavy-light chains.
//   O(M log^2 N).
// original: corpus/submissions/4207493.cpp
// Bounds: n: 1..=5e4; m: 1..=1e5; roads.u, roads.v: 1..=n; roads.cost: 1..=1e4
let (n, m): (1..=5e4, 1..=1e5) = read()
domain Town = 1..=n
domain Road = 0..m

let roads: [Road](u: Town, v: Town, cost: 1..=1e4) = read()

// the cheapest set: take roads cheapest first, skipping any that close a loop
var joined = Dsu<Town>()
var kept = List<(u: Town, v: Town, cost: Int)>()
var spare = List<(u: Town, v: Town, cost: Int)>()
domain Cost = 1..=1e4
var cheapest = roads
cheapest.counting_sort(by: r => Cost(r.cost))
for r in cheapest:
    if joined.unite(r.u, r.v): kept.push(r) else: spare.push(r)

// The two largest different costs in a set of roads; 0, below every cost, is max's identity.
type Top = (hi: Int, lo: Int)
fn both(a: Top, b: Top) -> Top:
    if a.hi == b.hi: (hi: a.hi, lo: max(a.lo, b.lo))
    elif a.hi > b.hi: (hi: a.hi, lo: max(a.lo, b.hi))
    else: (hi: b.hi, lo: max(a.hi, b.lo))

domain Link = 0..kept.len
let links = [Link] i => kept[i.idx]

if kept.len < n - 1:
    print(-1)
else:
    // Any other connecting set swaps roads in. One swap is enough for the
    // runner-up: add a road, and drop the dearest kept road on the path
    // between its ends that is strictly cheaper. Each kept road is stored
    // at its lower town, laid out so that a path is a few ranges.
    let tree = Rooted(Graph.undirected(links), Town.first!)
    var above = SegTree(Monoid<Top>((hi: 0, lo: 0), both), [tree.Pre] _ => (hi: 0, lo: 0))
    for (u, v, cost) in kept:
        let lower = if tree.parent(u) == Some(v): u else: v
        above.set(tree.pos(lower), (hi: cost, lo: 0))
    let base = sum(r.cost for r in kept)
    var best: ?Int = None
    for (u, v, cost) in spare:
        // the roads on the path: every town on it stands for the road above
        // it, except the highest town
        let path = tree.chains(u, v, edges: true).map(chain => above.fold(chain.range)).fold_from((hi: 0, lo: 0), both)
        let out = if path.hi < cost: path.hi else: path.lo
        if out != 0: best min= base + cost - out
    print(best ?? -1)

// Source: UTS Open 2015 P5 (Distribution Channel), https://dmoj.ca/problem/utso15p5
// Problem: N <= 5e4 stores, M <= 1e5 possible connections (a, b, cost <= 1e4). Print the
//   cost of the second cheapest way to connect all stores with N - 1 connections (a set that
//   costs strictly more than the cheapest), or -1 if there is none.
// Solution: the runner-up is one swap away from a minimum spanning tree: add a road u-v,
//   drop the dearest tree road on the u-v path that is strictly cheaper. Build the MST by
//   Kruskal with a Dsu, root it, store each tree road at its lower town in a SegTree of the
//   two largest distinct costs, and fold every road's path over heavy-light chains.
//   O(M log^2 N).
// original: corpus/submissions/4207493.cpp
// Bounds: n: 1..=5e4; m: 1..=1e5; cost: 1..=1e4
let (n, m): (1..=5e4, 1..=1e5) = read()
domain Town = 1..=n
domain Road = 0..m

let roads: [Road](u: Town, v: Town, cost: 1..=1e4) = read()

// the cheapest set: take roads cheapest first, skipping any that close a loop
var joined = Dsu<Town>()
let kept = [r for r in roads.sorted(by: r => r.cost) if joined.unite(r.u, r.v)]

// The two largest different costs in a set of roads; 0, below every cost, is max's identity.
type Top = (hi: Int, lo: Int)
fn both(a: Top, b: Top) -> Top:
    let hi = max(a.hi, b.hi)
    (hi, [x for x in [a.hi, a.lo, b.hi, b.lo] if x < hi].max() ?? 0)

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
    for (u, v, cost) in roads:
        // the roads on the path: every town on it stands for the road above
        // it, except the highest town
        let path = tree.chains(u, v, edges: true).map(chain => above.fold(chain.range)).fold(both, (0, 0))
        let out = if path.hi < cost: path.hi else: path.lo
        if out != 0: best min= base + cost - out
    print(best ?? -1)

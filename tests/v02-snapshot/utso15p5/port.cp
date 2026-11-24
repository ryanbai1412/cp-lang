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
let n: 1..=5e4 = read()
let m: 1..=1e5 = read()
domain Town = 1..=n
domain Road = 0..m

// generator: mostly connected maps, with few different costs so that ties are common
fn tied() -> [Road](u: Town, v: Town, cost: 1..=1e6):
    [Road] r => (
        if r.idx < n - 1 and n > 1: (
            Town(rand(1, r.idx + 1)),
            Town(min(n, r.idx + 2)),
            rand(1, 4),
        ) else: (Town(rand(1, n)), Town(rand(1, n)), rand(1, 4))
    )

let roads: [Road](u: Town, v: Town, cost: 1..=1e6) = read(gen: tied)

// the cheapest set: take roads cheapest first, skipping any that close a loop
var joined = Dsu<Town>()
var kept: List<(u: Town, v: Town, cost: Int)> = []
for (u, v, cost) in roads.sorted(by: r => r.cost):
    if joined.unite(u, v): kept.push((u, v, cost))

// The two largest different costs in a set of roads; 0 where there is none.
type Top = (hi: Int, lo: Int)
fn both(a: Top, b: Top) -> Top:
    let hi = max(a.hi, b.hi)
    (hi, [x for x in [a.hi, a.lo, b.hi, b.lo] if x < hi].max() ?? 0)

domain Link = 0..kept.len
let links: [Link](u: Town, v: Town, cost: Int) = [Link] i => kept[i.idx]

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
        above.set(tree.sub(if tree.parent(u) == Some(v): u else: v).first!, (hi: cost, lo: 0))
    let base = sum(r.cost for r in kept)
    var best: ?Int = None
    for (u, v, cost) in roads:
        // the roads on the path: every town on it stands for the road above
        // it, except the highest town
        var path: Top = (0, 0)
        for chain in tree.chains(u, v, edges: true): path = both(path, above.fold(chain.range))
        let out = if path.hi < cost: path.hi else: path.lo
        if out != 0: best min= base + cost - out
    print(best ?? -1)

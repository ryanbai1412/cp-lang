// original: corpus/submissions/4207493.cpp
// UTS Open 2015 problem 5: the cheapest set of roads that connects every
// town is known to everyone. Print the cost of the cheapest connecting set
// that costs strictly more, or -1 if there is none.
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
for (u, v, cost) in roads.sorted_by(.cost):
    if joined.unite(u, v): kept.push((u, v, cost))

// The two largest different costs in a set of roads; 0 where there is none.
type Top = (hi: Int, lo: Int)
fn both(a: Top, b: Top) -> Top:
    let hi = max(a.hi, b.hi)
    (hi, max(x for x in [a.hi, a.lo, b.hi, b.lo] if x < hi) ?? 0)

if kept.len < n - 1:
    print(-1)
else:
    // Any other connecting set swaps roads in. One swap is enough for the
    // runner-up: add a road, and drop the dearest kept road on the path
    // between its ends that is strictly cheaper. Each kept road is stored
    // at its lower town, laid out so that a path is a few ranges.
    let tree = Rooted(kept, Town.first)
    var above = SegTree<tree.Pre, Top>((0, 0), (a, b) => both(a, b))
    for (u, v, cost) in kept:
        above.set(tree.sub(if tree.parent(u) == v: u else: v).l, (cost, 0))
    let base = sum(r.cost for r in kept)
    var best: ?Int = None
    for (u, v, cost) in roads:
        // the roads on the path: every town on it stands for the road above
        // it, except the highest town
        let peak = tree.sub(tree.lca(u, v)).l
        var path: Top = (0, 0)
        for (l, r) in tree.chains(u, v):
            if l != peak: path = both(path, above.fold(l, r))
            elif let below = l + 1 and l != r: path = both(path, above.fold(below, r))
        let out = if path.hi < cost: path.hi else: path.lo
        if out != 0: best min= base + cost - out
    print(best ?? -1)
